//! P2a: authenticated record-only workspace lifecycle. No UI/profile/token creation or import.
use super::*;
use crate::product_start::{
    ModelConditions, ModelJob, Purpose, Settlement, StartModel, StartProposalV1,
    WorkspaceBindingV1, parse_payload,
};

const POLICY: &[u8] = b"{\"kind\":\"p2a-workspace-fixture\",\"version\":1,\"offline\":true}\n";
const SENTINEL: &[u8] = "P2a: ÄÖÜ ß Ελληνικά Кириллица. Keine Importaufgabe.\n".as_bytes();

pub(super) fn apply(
    state: &mut State,
    actor: [u8; 32],
    action: &Action,
    digest: [u8; 32],
) -> Result<()> {
    if actor != state.actor
        || state.schema != 5
        || state.pending.is_some()
        || state.session.is_some()
    {
        return Err(invalid("workspace-control-state"));
    }
    // Pure historical replay: Store::open returns no state until the final head matches too.
    // Current native calls additionally run append_guard (owner lease + actual OS job check).
    let conditions = ModelConditions {
        consistent_control: true,
        pending: false,
        job: ModelJob::Empty,
    };
    if state.probe.is_some() {
        return Err(invalid("workspace-probe-pending"));
    }
    match action {
        Action::BindWorkspace { binding }
            if state.workspace.is_none()
                && binding.installation_id == *state.installation.as_bytes()
                && binding.broker_package_sha256 == actor =>
        {
            let remaining = MAX_RECORDS
                .checked_sub(state.sequence + 2)
                .ok_or(invalid("workspace-budget"))?;
            state.workspace = Some(
                StartModel::new(binding.clone(), digest, remaining as u16)
                    .map_err(|_| invalid("workspace-binding"))?,
            );
        }
        Action::CommitWorkspaceLaunch { proposal } => {
            let parsed = parse_payload(
                &serde_json::to_vec(proposal).map_err(|_| invalid("workspace-proposal"))?,
            )
            .map_err(|_| invalid("workspace-proposal"))?;
            state
                .workspace
                .as_mut()
                .ok_or(invalid("workspace-unbound"))?
                .commit(&parsed, digest, conditions)
                .map_err(|_| invalid("workspace-launch-rejected"))?;
        }
        Action::ReconcileWorkspaceLaunch { launch } => {
            state
                .workspace
                .as_mut()
                .ok_or(invalid("workspace-unbound"))?
                .settle(
                    *launch,
                    state.head,
                    digest,
                    Settlement::Interrupted,
                    conditions,
                )
                .map_err(|_| invalid("workspace-reconcile-rejected"))?;
        }
        _ => return Err(invalid("workspace-invalid-transition")),
    }
    Ok(())
}

impl Store {
    fn workspace_root(&self) -> PathBuf {
        self.root.join("data/workspace")
    }
    pub(super) fn workspace_summary(&self) -> Value {
        let model = self.state.workspace.as_ref();
        json!({"nativePreviewOnly":true,"productionAcceptance":false,"control":self.summary(),
            "workspaceBound":model.is_some(),"generation":model.map_or(0,StartModel::generation),
            "pendingLaunch":model.and_then(StartModel::pending_launch),"uiStarted":false,
            "profileCreated":false,"importTaskCreated":false,"networkMeasurement":false})
    }
    pub(super) fn verify_workspace_files(&self) -> Result<()> {
        let Some(model) = &self.state.workspace else {
            return Ok(());
        };
        let binding = model.binding();
        let root = self.workspace_root();
        let _pins = pin_directories(&root)?;
        private_acl(&root, true)?;
        let mut policy = protected_file(&root.join("policy.json"))?;
        let mut bytes = Vec::new();
        io(
            "workspace-policy-read",
            (&mut policy).take(1025).read_to_end(&mut bytes),
        )?;
        if bytes != POLICY
            || Sha256::digest(&bytes).as_slice() != binding.config_sha256
            || root_hash(&io("workspace-root-canonical", root.canonicalize())?)
                != binding.workspace_root_sha256
        {
            return Err(invalid("workspace-root-or-policy"));
        }
        let (_parents, _app, sha) = binary_pin(&self.root.join("bin/p2a-ui.exe"))?;
        private_acl(&self.root.join("bin/p2a-ui.exe"), false)?;
        if sha != binding.ui_package_sha256 {
            return Err(invalid("workspace-ui-hash"));
        }
        Ok(())
    }
    pub(super) fn bind_workspace(
        &mut self,
        actor: [u8; 32],
        head: [u8; 32],
        app: &Path,
        expected: [u8; 32],
    ) -> Result<()> {
        self.append_guard(actor, head)?;
        if self.state.schema != 5
            || self.state.workspace.is_some()
            || self.state.session.is_some()
            || self.state.pending.is_some()
        {
            return Err(invalid("workspace-bind-state"));
        }
        let (_parents, mut source, actual) = binary_pin(app)?;
        if actual != expected {
            return Err(invalid("workspace-bind-ui-hash"));
        }
        let root = self.workspace_root();
        // Never adopt an existing root/package left by an interrupted first bind.
        secure_directory(&root)?;
        for (name, bytes) in [("policy.json", POLICY), ("sentinel.txt", SENTINEL)] {
            let mut file = io(
                "workspace-fixture-create",
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join(name)),
            )?;
            io("workspace-fixture-write", file.write_all(bytes))?;
            io("workspace-fixture-sync", file.sync_all())?;
        }
        use std::io::{Seek, SeekFrom};
        io("workspace-ui-rewind", source.seek(SeekFrom::Start(0)))?;
        let mut dest = io(
            "workspace-ui-create",
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .share_mode(0)
                .open(self.root.join("bin/p2a-ui.exe")),
        )?;
        io("workspace-ui-copy", std::io::copy(&mut source, &mut dest))?;
        io("workspace-ui-sync", dest.sync_all())?;
        drop(dest);
        private_acl(&self.root.join("bin/p2a-ui.exe"), false)?;
        if hash(&self.root.join("bin/p2a-ui.exe"))? != expected {
            return Err(invalid("workspace-copy-hash"));
        }
        self.append(
            actor,
            head,
            Action::BindWorkspace {
                binding: WorkspaceBindingV1 {
                    version: 1,
                    control_schema: 5,
                    installation_id: *self.state.installation.as_bytes(),
                    workspace_id: *Uuid::new_v4().as_bytes(),
                    profile_id: *Uuid::new_v4().as_bytes(),
                    broker_package_sha256: actor,
                    ui_package_sha256: expected,
                    config_sha256: Sha256::digest(POLICY).into(),
                    workspace_root_sha256: root_hash(&io(
                        "workspace-root-canonical",
                        root.canonicalize(),
                    )?),
                    workspace_format_version: 1,
                    offline_policy_version: 1,
                },
            },
        )
    }
    pub(super) fn workspace_proposal(
        &self,
        head: [u8; 32],
        replay: bool,
    ) -> Result<StartProposalV1> {
        let model = self
            .state
            .workspace
            .as_ref()
            .ok_or(invalid("workspace-unbound"))?;
        Ok(StartProposalV1 {
            version: 1,
            purpose: Purpose::WorkspaceStart,
            binding: model.binding().clone(),
            launch_id: if replay {
                model.used_launch().ok_or(invalid("workspace-no-replay"))?
            } else {
                *Uuid::new_v4().as_bytes()
            },
            generation: model
                .generation()
                .checked_add(1)
                .ok_or(invalid("workspace-generation"))?,
            expected_head: head,
            bootstrap_timeout_ms: 30_000,
        })
    }
}

pub(in crate::windows) fn child(args: &[OsString]) -> Result<Value> {
    if args.len() != 5 {
        return Err(invalid("workspace-child-arguments"));
    }
    let root = Path::new(&args[0]);
    crash::scoped(root)?;
    let action = args[1].to_str().ok_or(invalid("workspace-child-action"))?;
    if !matches!(
        action,
        "init"
            | "migrate5"
            | "bind"
            | "commit"
            | "replay"
            | "reconcile"
            | "status"
            | "commit-crash-before"
            | "commit-crash-after"
            | "commit-crash-tail"
    ) {
        return Err(invalid("workspace-child-action"));
    }
    let app = Path::new(&args[2]);
    let expected = parse_hash(&args[3])?;
    let head = parse_hash(&args[4])?;
    let exe = io("workspace-exe", std::env::current_exe())?;
    let (_parents, mut file, actor) = binary_pin(&exe)?;
    package::verify_workspace_records(&mut file)?;
    if action == "init" {
        return Ok(Store::initialize(root, actor)?.workspace_summary());
    }
    let mut store = Store::open(root)?;
    store.actor(actor)?;
    store.verify_workspace_files()?;
    if let Some(model) = &store.state.workspace
        && model.binding().ui_package_sha256 != expected
    {
        return Err(invalid("workspace-expected-ui"));
    }
    match action {
        "status" => {}
        "migrate5" => {
            if !(1..5).contains(&store.state.schema)
                || store.state.session.is_some()
                || store.state.pending.is_some()
            {
                return Err(invalid("workspace-migration-state"));
            }
            store.append_guard(actor, head)?;
            for from in store.state.schema..5 {
                store.append(
                    actor,
                    if from == 1 { head } else { store.state.head },
                    Action::Migrate { from, to: from + 1 },
                )?;
            }
        }
        "bind" => store.bind_workspace(actor, head, app, expected)?,
        "commit"
        | "replay"
        | "commit-crash-before"
        | "commit-crash-after"
        | "commit-crash-tail" => {
            match action {
                "commit-crash-before" => crash::arm(root, crash::Point::BeforeRecord)?,
                "commit-crash-after" => crash::arm(root, crash::Point::HeadPublished)?,
                "commit-crash-tail" => crash::arm(root, crash::Point::RecordSynced)?,
                _ => {}
            }
            let proposal = store.workspace_proposal(head, action == "replay")?;
            store.append(actor, head, Action::CommitWorkspaceLaunch { proposal })?;
        }
        "reconcile" => {
            let launch = store
                .state
                .workspace
                .as_ref()
                .and_then(StartModel::pending_launch)
                .ok_or(invalid("workspace-no-pending-launch"))?;
            store.append(actor, head, Action::ReconcileWorkspaceLaunch { launch })?;
        }
        _ => return Err(invalid("workspace-child-action")),
    }
    Ok(store.workspace_summary())
}

#[cfg(test)]
mod tests;
