//! Synthetic P2b worker bootstrap, never an authoring UI or an import grant.
use super::*;
use crate::product_start::{LaunchIdentity, ModelConditions, ModelJob, Settlement};
mod bootstrap;
pub(super) mod lifetime;
mod recovery;
pub(in crate::windows) use bootstrap::worker;

const POLICY: &[u8] = b"{\"kind\":\"p2b-worker-probe\",\"version\":1,\"network\":false}\n";
const SENTINEL: &[u8] = "P2b: ÄÖÜ ß Ελληνικά Кириллица. Kein Importtask.\n".as_bytes();

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Execution {
    version: u16,
    probe_id: Uuid,
    workspace_id: [u8; 16],
    nonce: [u8; 32],
    worker_sha256: [u8; 32],
    root_sha256: [u8; 32],
    policy_sha256: [u8; 32],
    sentinel_sha256: [u8; 32],
    live_hold_ms: u16,
}
#[derive(Clone)]
pub(super) struct Pending {
    execution: Execution,
    launch: LaunchIdentity,
    ready: bool,
    run_intent: bool,
    drained: bool,
    worker: Option<lifetime::WorkerIdentity>,
}

pub(super) fn apply(
    state: &mut State,
    actor: [u8; 32],
    action: &Action,
    digest: [u8; 32],
) -> Result<()> {
    if state.schema != 5
        || actor != state.actor
        || state.session.is_some()
        || state.pending.is_some()
    {
        return Err(invalid("probe-control-state"));
    }
    let conditions = ModelConditions {
        consistent_control: true,
        pending: false,
        job: ModelJob::Empty,
    };
    let model = state
        .workspace
        .as_mut()
        .ok_or(invalid("probe-workspace-unbound"))?;
    match action {
        Action::CommitWorkspaceProbe {
            proposal,
            execution,
        } if state.probe.is_none()
            && MAX_RECORDS.saturating_sub(state.sequence + 1) >= 6
            && execution.version == 3
            && execution.live_hold_ms <= 2000
            && !execution.probe_id.is_nil()
            && execution.workspace_id == model.binding().workspace_id
            && execution.probe_id.as_bytes() != &model.binding().profile_id
            && execution.worker_sha256 == actor
            && execution.nonce != [0; 32]
            && execution.root_sha256 != [0; 32]
            && execution.policy_sha256 == <[u8; 32]>::from(Sha256::digest(POLICY))
            && execution.sentinel_sha256 == <[u8; 32]>::from(Sha256::digest(SENTINEL)) =>
        {
            let parsed = crate::product_start::parse_payload(
                &serde_json::to_vec(proposal).map_err(|_| invalid("probe-proposal"))?,
            )
            .map_err(|_| invalid("probe-proposal"))?;
            model
                .commit(&parsed, digest, conditions)
                .map_err(|_| invalid("probe-commit-rejected"))?;
            state.probe = Some(Pending {
                execution: execution.clone(),
                launch: LaunchIdentity {
                    id: proposal.launch_id,
                    generation: proposal.generation,
                },
                ready: false,
                run_intent: false,
                drained: false,
                worker: None,
            });
        }
        Action::WorkspaceProbeProfileReady { launch } => {
            let pending = state.probe.as_mut().ok_or(invalid("probe-no-resource"))?;
            if pending.ready || pending.launch != *launch {
                return Err(invalid("probe-profile-record"));
            }
            pending.ready = true;
            // Non-lifecycle record: synchronize the rule state's head after authenticated replay.
            model
                .sync_control_head(state.head, digest)
                .map_err(|_| invalid("probe-profile-head"))?;
        }
        Action::FinishWorkspaceProbe {
            launch,
            receipt_sha256,
        } => {
            let pending = state.probe.as_ref().ok_or(invalid("probe-no-resource"))?;
            if !pending.drained
                || pending.worker.is_none()
                || pending.launch != *launch
                || *receipt_sha256 == [0; 32]
            {
                return Err(invalid("probe-finish-record"));
            }
            model
                .mark_running(
                    *launch,
                    state.head,
                    ModelConditions {
                        job: ModelJob::OwnedSuspendedChecked,
                        ..conditions
                    },
                )
                .map_err(|_| invalid("probe-running-record"))?;
            model
                .settle(
                    *launch,
                    state.head,
                    digest,
                    Settlement::NormalClose,
                    conditions,
                )
                .map_err(|_| invalid("probe-finish-rejected"))?;
            state.probe = None;
        }
        Action::ReconcileWorkspaceProbe { launch } => {
            let pending = state.probe.as_ref().ok_or(invalid("probe-no-resource"))?;
            if !pending.ready
                || (pending.run_intent && !pending.drained)
                || pending.launch != *launch
            {
                return Err(invalid("probe-profile-ownership-unconfirmed"));
            }
            model
                .settle(
                    *launch,
                    state.head,
                    digest,
                    Settlement::Interrupted,
                    conditions,
                )
                .map_err(|_| invalid("probe-reconcile-rejected"))?;
            state.probe = None;
        }
        Action::WorkspaceProbeRunIntent { launch } | Action::WorkspaceProbeDrained { launch } => {
            let pending = state.probe.as_mut().ok_or(invalid("probe-no-resource"))?;
            let draining = matches!(action, Action::WorkspaceProbeDrained { .. });
            if pending.launch != *launch
                || !pending.ready
                || pending.drained
                || pending.run_intent != draining
            {
                return Err(invalid("probe-resource-order"));
            }
            model
                .sync_control_head(state.head, digest)
                .map_err(|_| invalid("probe-resource-head"))?;
            if draining {
                pending.drained = true;
            } else {
                pending.run_intent = true;
            }
        }
        Action::AbortUncreatedWorkspaceProbe { launch } => {
            let pending = state.probe.as_ref().ok_or(invalid("probe-no-resource"))?;
            if pending.ready || pending.run_intent || pending.launch != *launch {
                return Err(invalid("probe-created-state"));
            }
            model
                .settle(
                    *launch,
                    state.head,
                    digest,
                    Settlement::Interrupted,
                    conditions,
                )
                .map_err(|_| invalid("probe-uncreated-settlement"))?;
            state.probe = None;
        }
        Action::WorkspaceProbeWorkerBound {
            launch,
            worker,
            process_limit,
        } => {
            let pending = state.probe.as_mut().ok_or(invalid("probe-no-resource"))?;
            if pending.launch != *launch
                || !pending.run_intent
                || pending.drained
                || pending.worker.is_some()
                || !worker.valid()
                || *process_limit != 1
            {
                return Err(invalid("probe-worker-binding-order"));
            }
            model
                .sync_control_head(state.head, digest)
                .map_err(|_| invalid("probe-worker-binding-head"))?;
            pending.worker = Some(*worker);
        }
        Action::WorkspaceProbeRecoveredDrain {
            launch,
            worker,
            evidence,
        } => {
            let pending = state.probe.as_mut().ok_or(invalid("probe-no-resource"))?;
            if pending.launch != *launch
                || !pending.run_intent
                || pending.drained
                || pending.worker != Some(*worker)
                || !evidence.valid_for(*worker)
            {
                return Err(invalid("probe-worker-end-proof"));
            }
            model
                .sync_control_head(state.head, digest)
                .map_err(|_| invalid("probe-worker-end-head"))?;
            pending.drained = true;
        }
        _ => return Err(invalid("probe-invalid-transition")),
    }
    Ok(())
}

fn profile(execution: &Execution) -> Result<Profile> {
    let name = wide(format!(
        "org.kraqel.probe.boot-{}",
        execution.probe_id.simple()
    ));
    let mut sid = null_mut();
    let hr = unsafe { DeriveAppContainerSidFromAppContainerName(name.as_ptr(), &mut sid) };
    if hr < 0 {
        return Err(Failure {
            stage: "probe-derive-profile",
            code: hr.into(),
        });
    }
    Ok(Profile {
        name,
        sid,
        active: false,
    })
}
fn root(execution: &Execution) -> PathBuf {
    std::env::temp_dir().join(format!(
        "kraqel-isolation-boot-{}",
        execution.probe_id.simple()
    ))
}
fn remove_profile(execution: &Execution) -> Result<()> {
    let p = profile(execution)?;
    let hr = unsafe { DeleteAppContainerProfile(p.name.as_ptr()) };
    if hr < 0 && !matches!(hr as u32, 0x80070490 | 0x80070002) {
        return Err(Failure {
            stage: "probe-profile-delete",
            code: hr.into(),
        });
    }
    Ok(())
}
fn stage(
    source: &mut fs::File,
    actor: [u8; 32],
    workspace_id: [u8; 16],
    live_hold_ms: u16,
) -> Result<Execution> {
    let id = Uuid::new_v4();
    let path = std::env::temp_dir().join(format!("kraqel-isolation-boot-{}", id.simple()));
    if !crate::policy::valid_root(&path, false) {
        return Err(invalid("probe-needs-fresh-root"));
    }
    secure_directory(&path)?;
    secure_directory(&path.join("bin"))?;
    secure_directory(&path.join("data"))?;
    for (name, bytes) in [("policy.json", POLICY), ("sentinel.txt", SENTINEL)] {
        let mut file = io(
            "probe-fixture-create",
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path.join("data").join(name)),
        )?;
        io("probe-fixture-write", file.write_all(bytes))?;
        io("probe-fixture-sync", file.sync_all())?;
    }
    use std::io::{Seek, SeekFrom};
    io("probe-source-rewind", source.seek(SeekFrom::Start(0)))?;
    let mut file = io(
        "probe-package-create",
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path.join("bin/worker.exe")),
    )?;
    io("probe-package-copy", std::io::copy(source, &mut file))?;
    io("probe-package-sync", file.sync_all())?;
    drop(file);
    if hash(&path.join("bin/worker.exe"))? != actor {
        return Err(invalid("probe-copy-hash"));
    }
    let mut nonce = [0; 32];
    getrandom::fill(&mut nonce).map_err(|_| invalid("probe-nonce"))?;
    Ok(Execution {
        version: 3,
        probe_id: id,
        workspace_id,
        nonce,
        worker_sha256: actor,
        root_sha256: root_hash(&io("probe-root-canonical", path.canonicalize())?),
        policy_sha256: Sha256::digest(POLICY).into(),
        sentinel_sha256: Sha256::digest(SENTINEL).into(),
        live_hold_ms,
    })
}

impl Store {
    fn start_workspace_probe(
        &mut self,
        actor: [u8; 32],
        head: [u8; 32],
        mode: bootstrap::Mode,
    ) -> Result<Value> {
        self.append_guard(actor, head)?;
        self.verify_workspace_files()?;
        if self.state.probe.is_some() {
            return Err(invalid("probe-resource-pending"));
        }
        let proposal = self.workspace_proposal(head, false)?;
        let exe = io("probe-source-exe", std::env::current_exe())?;
        let (_pins, mut source, actual) = binary_pin(&exe)?;
        package::verify_probe(&mut source)?;
        if actual != actor {
            return Err(invalid("probe-source-actor"));
        }
        // Validate the core decision before staging any new scratch files.
        let mut core = self
            .state
            .workspace
            .as_ref()
            .ok_or(invalid("probe-unbound"))?
            .clone();
        let parsed = crate::product_start::parse_payload(
            &serde_json::to_vec(&proposal).map_err(|_| invalid("probe-proposal"))?,
        )
        .map_err(|_| invalid("probe-proposal"))?;
        let mut check_head = head;
        check_head[0] ^= 1;
        if check_head == [0; 32] {
            check_head[1] = 1;
        }
        core.commit(
            &parsed,
            check_head,
            ModelConditions {
                consistent_control: true,
                pending: false,
                job: ModelJob::Empty,
            },
        )
        .map_err(|_| invalid("probe-preflight"))?;
        if MAX_RECORDS.saturating_sub(self.state.sequence + 1) < 6 {
            return Err(invalid("probe-record-budget"));
        }
        let execution = stage(
            &mut source,
            actor,
            proposal.binding.workspace_id,
            if matches!(mode, bootstrap::Mode::CrashLive) {
                2000
            } else {
                0
            },
        )?;
        self.append(
            actor,
            head,
            Action::CommitWorkspaceProbe {
                proposal,
                execution: execution.clone(),
            },
        )?;
        let pending = self.state.probe.as_ref().unwrap();
        let launch = pending.launch;
        let execution = pending.execution.clone();
        crash::hit(crash::Point::ProbeCommitted);
        let mut profile = Profile::create(&format!("boot-{}", execution.probe_id.simple()))?;
        // From this point an incomplete record must retain the profile for fail-closed diagnosis.
        profile.active = false;
        crash::hit(crash::Point::ProbeCreated);
        self.append(
            actor,
            self.state.head,
            Action::WorkspaceProbeProfileReady { launch },
        )?;
        crash::hit(crash::Point::ProbeReady);
        self.append(
            actor,
            self.state.head,
            Action::WorkspaceProbeRunIntent { launch },
        )?;
        let result = bootstrap::run(self, &execution, &profile, launch, mode)?;
        crash::hit(crash::Point::WorkerExited);
        self.append(
            actor,
            self.state.head,
            Action::WorkspaceProbeDrained { launch },
        )?;
        self.idle()?;
        remove_profile(&execution)?;
        crash::hit(crash::Point::ProbeRemoved);
        let end = match &result {
            Ok(value) => {
                json!({"passed":true,"receiptSha256":value.receipt_sha256,"profileRemoved":true,"uiStarted":false,"importTaskCreated":false})
            }
            Err(error) => {
                json!({"passed":false,"stage":error.stage,"code":error.code,"profileRemoved":true,"uiStarted":false,"importTaskCreated":false})
            }
        };
        write_json(
            &self
                .root
                .parent()
                .unwrap()
                .join(format!("probe-{}-end.json", launch.generation)),
            &end,
        )?;
        match result {
            Ok(value) => {
                self.append(
                    actor,
                    self.state.head,
                    Action::FinishWorkspaceProbe {
                        launch,
                        receipt_sha256: value.receipt_sha256,
                    },
                )?;
                Ok(value.report)
            }
            Err(error) => {
                self.append(
                    actor,
                    self.state.head,
                    Action::ReconcileWorkspaceProbe { launch },
                )?;
                Err(error)
            }
        }
    }
}

pub(in crate::windows) fn child(args: &[OsString]) -> Result<Value> {
    if args.len() != 3 {
        return Err(invalid("probe-child-arguments"));
    }
    let path = Path::new(&args[0]);
    crash::scoped(path)?;
    let action = args[1].to_str().ok_or(invalid("probe-child-action"))?;
    let head = parse_hash(&args[2])?;
    if !matches!(
        action,
        "start"
            | "start-unassigned"
            | "start-bad-mac"
            | "status"
            | "reconcile"
            | "crash-before-create"
            | "crash-after-create"
            | "crash-after-ready"
            | "crash-after-remove"
            | "crash-before-bound"
            | "crash-after-bound"
            | "crash-live"
            | "crash-after-exit"
    ) {
        return Err(invalid("probe-child-action"));
    }
    let exe = io("probe-exe", std::env::current_exe())?;
    let (_pins, mut file, actor) = binary_pin(&exe)?;
    package::verify_probe(&mut file)?;
    let mut store = Store::open(path)?;
    store.actor(actor)?;
    store.verify_workspace_files()?;
    let point = match action {
        "crash-before-create" => Some(crash::Point::ProbeCommitted),
        "crash-after-create" => Some(crash::Point::ProbeCreated),
        "crash-after-ready" => Some(crash::Point::ProbeReady),
        "crash-after-remove" => Some(crash::Point::ProbeRemoved),
        "crash-before-bound" => Some(crash::Point::BeforeWorkerBound),
        "crash-after-bound" => Some(crash::Point::WorkerBound),
        "crash-live" => Some(crash::Point::WorkerLive),
        "crash-after-exit" => Some(crash::Point::WorkerExited),
        _ => None,
    };
    if let Some(point) = point {
        crash::arm(path, point)?;
    }
    let outcome = match action {
        "status" => json!(null),
        "reconcile" => store.reconcile_probe_resources(actor, head)?,
        _ => store.start_workspace_probe(
            actor,
            head,
            match action {
                "start-unassigned" => bootstrap::Mode::Unassigned,
                "start-bad-mac" => bootstrap::Mode::BadMac,
                "crash-live" => bootstrap::Mode::CrashLive,
                _ => bootstrap::Mode::Normal,
            },
        )?,
    };
    Ok(
        json!({"nativePreviewOnly":true,"productionAcceptance":false,"workspace":store.workspace_summary(),"probePending":store.state.probe.is_some(),"outcome":outcome,"networkMeasurement":false}),
    )
}

#[cfg(test)]
mod tests;
