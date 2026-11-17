//! C2: fresh synthetic data/profile and package binding only. No UI/task/process launch authority.
use super::*;
use crate::product_start_p2c::{
    self as contract, FileRole, InventoryEntry, PackageRole, RootIdentityV1, WorkspaceBindingV2,
};
use std::io::{Seek, SeekFrom};
mod c4_preflight;
mod filesystem;
mod group;
use filesystem::*;
pub(super) use group::{GroupEvent, GroupState, apply as apply_group};
pub(in crate::windows) use group::{child as group_child, worker as group_worker};

const POLICY: &[u8] =
    b"{\"kind\":\"p2c-resource-fixture\",\"version\":1,\"network\":false,\"uiExecutable\":false}\n";
const TEXT: &[u8] =
    "C2: persistente synthetische Daten. ÄÖÜ, Kolkrabe, keine Importaufgabe.\n".as_bytes();
const EDITED: &[u8] = "C2: geändert und nach erneutem Controllerstart erhalten.\n".as_bytes();
const MODIFY: u32 = FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE | DELETE;
const RX: u32 = FILE_GENERIC_READ | FILE_GENERIC_EXECUTE;

#[derive(Clone)]
pub(super) struct Persistent {
    id: Uuid,
    profile_ready: bool,
    binding: Option<WorkspaceBindingV2>,
    roots: Option<BoundRoots>,
    retiring: bool,
    retired: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct BoundRoots {
    execution: RootIdentityV1,
    host_package: RootIdentityV1,
    ui_package: RootIdentityV1,
}
fn name(id: Uuid) -> String {
    format!("kraqel-p2c-{}", id.simple())
}
fn data_root(id: Uuid) -> PathBuf {
    std::env::temp_dir().join(format!("kraqel-isolation-p2c-{}", id.simple()))
}
pub(super) fn scope(root: &Path) -> Result<()> {
    crash::scoped(root).map(|_| ())
}

pub(super) fn apply(state: &mut State, actor: [u8; 32], action: &Action) -> Result<()> {
    if state.schema != 6
        || actor != state.actor
        || state.pending.is_some()
        || state.session.is_some()
        || state.workspace.is_some()
        || state.probe.is_some()
        || state.c3.pending()
    {
        return Err(invalid("c2-control-state"));
    }
    match action {
        Action::PrepareC2Workspace { workspace } if state.c2.is_none() && !workspace.is_nil() => {
            state.c2 = Some(Persistent {
                id: *workspace,
                profile_ready: false,
                binding: None,
                roots: None,
                retiring: false,
                retired: false,
            });
        }
        Action::C2ProfileReady => {
            let p = state.c2.as_mut().ok_or(invalid("c2-unprepared"))?;
            if p.profile_ready || p.retiring {
                return Err(invalid("c2-profile-order"));
            }
            p.profile_ready = true;
        }
        Action::BindC2Workspace { binding, roots } => {
            let p = state.c2.as_mut().ok_or(invalid("c2-unprepared"))?;
            if !p.profile_ready
                || p.binding.is_some()
                || p.retiring
                || !binding.valid()
                || binding.installation_id != *state.installation.as_bytes()
                || binding.workspace_id != *p.id.as_bytes()
                || binding.profile_name != name(p.id)
                || binding.revision != 1
                || !roots.execution.valid()
                || !roots.host_package.valid()
                || !roots.ui_package.valid()
            {
                return Err(invalid("c2-binding-order-or-shape"));
            }
            p.binding = Some(binding.clone());
            p.roots = Some(roots.clone());
        }
        Action::RetireC2Workspace => {
            let p = state.c2.as_mut().ok_or(invalid("c2-unprepared"))?;
            if !p.profile_ready || p.retiring {
                return Err(invalid("c2-retire-unproved-profile"));
            }
            p.retiring = true;
        }
        Action::C2ProfileRetired => {
            let p = state.c2.as_mut().ok_or(invalid("c2-unprepared"))?;
            if !p.retiring || p.retired {
                return Err(invalid("c2-retire-order"));
            }
            p.retired = true;
        }
        _ => return Err(invalid("c2-invalid-transition")),
    }
    Ok(())
}

fn profile(id: Uuid) -> Result<Profile> {
    let name = wide(name(id));
    let mut sid = null_mut();
    let hr = unsafe { DeriveAppContainerSidFromAppContainerName(name.as_ptr(), &mut sid) };
    if hr < 0 {
        return Err(Failure {
            stage: "c2-derive-profile",
            code: hr.into(),
        });
    }
    Ok(Profile {
        name,
        sid,
        active: false,
    }) // Persistent: Drop frees SID only, never deletes a profile.
}
fn present(p: &Profile) -> Result<bool> {
    workspace_probe::recovery::profile_present(p)
}
fn sid_text(sid: PSID) -> Result<String> {
    let mut ptr = null_mut();
    check(
        unsafe { ConvertSidToStringSidW(sid, &mut ptr) },
        "c2-sid-text",
    )?;
    let _local = Local(ptr.cast());
    let mut len = 0;
    while len < 184 && unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    if len == 184 {
        return Err(invalid("c2-sid-bound"));
    }
    String::from_utf16(unsafe { std::slice::from_raw_parts(ptr, len) })
        .map_err(|_| invalid("c2-sid-text"))
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f = io(
        "c2-create-file",
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .share_mode(0)
            .open(path),
    )?;
    io("c2-write", f.write_all(bytes))?;
    io("c2-sync", f.sync_all())
}
fn inventory_file(store: &Store, role: PackageRole) -> PathBuf {
    store.root.join(match role {
        PackageRole::Host => "control/c2-host.inventory",
        PackageRole::Ui => "control/c2-ui.inventory",
    })
}
fn read_inventory(store: &Store, role: PackageRole) -> Result<(Vec<u8>, Vec<InventoryEntry>)> {
    let mut f = protected_file(&inventory_file(store, role))?;
    let mut bytes = Vec::new();
    io(
        "c2-inventory-read",
        (&mut f).take(4 * 1024 * 1024 + 1).read_to_end(&mut bytes),
    )?;
    let (actual, entries) =
        contract::decode_inventory(&bytes).map_err(|_| invalid("c2-inventory-shape"))?;
    if actual != role {
        return Err(invalid("c2-inventory-role"));
    }
    Ok((bytes, entries))
}

impl Store {
    fn c2_guard(&self, actor: [u8; 32], head: [u8; 32]) -> Result<()> {
        scope(&self.root)?;
        self.append_guard(actor, head)?;
        if self.state.schema != 6
            || self.state.pending.is_some()
            || self.state.session.is_some()
            || self.state.workspace.is_some()
            || self.state.probe.is_some()
        {
            return Err(invalid("c2-not-data-only-store"));
        }
        // C3 pending launches are blocked by append_guard even after a lost guardian/job handle.
        Ok(())
    }
    fn prepare_c2(&mut self, actor: [u8; 32], head: [u8; 32], source: &mut fs::File) -> Result<()> {
        self.c2_guard(actor, head)?;
        if self.state.c2.is_some() || MAX_RECORDS.saturating_sub(self.state.sequence + 1) < 5 {
            return Err(invalid("c2-bind-state-or-budget"));
        }
        let id = Uuid::new_v4();
        let p = profile(id)?;
        let root = data_root(id);
        if !crate::policy::valid_root(&root, false) || present(&p)? {
            return Err(invalid("c2-existing-resource"));
        }
        self.append(actor, head, Action::PrepareC2Workspace { workspace: id })?;
        secure_directory(&root)?;
        let _resource_pins = pin_directories(&root)?;
        for dir in ["projects", "webview", "sessions", "package"] {
            secure_directory(&root.join(dir))?;
        }
        let host = self.root.join("bin/c2-host");
        secure_directory(&host)?;
        io("c2-source-rewind", source.seek(SeekFrom::Start(0)))?;
        let mut dest = io(
            "c2-copy-create",
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .share_mode(0)
                .open(host.join("controller.exe")),
        )?;
        io("c2-copy", std::io::copy(source, &mut dest))?;
        io("c2-copy-sync", dest.sync_all())?;
        drop(dest);
        if hash(&host.join("controller.exe"))? != actor {
            return Err(invalid("c2-host-copy-hash"));
        }
        write_new(&root.join("projects/synthetic.txt"), TEXT)?;
        for (file, bytes) in [("policy.json", POLICY), ("ä.txt", TEXT)] {
            write_new(&root.join("package").join(file), bytes)?;
        }
        // Creation is never adoption. A profile appearing after the precheck also causes failure.
        let mut created = null_mut();
        let hr = unsafe {
            CreateAppContainerProfile(
                p.name.as_ptr(),
                p.name.as_ptr(),
                wide("C2 synthetic persistent data; no UI").as_ptr(),
                null(),
                0,
                &mut created,
            )
        };
        if hr < 0 {
            return Err(Failure {
                stage: "c2-create-profile",
                code: hr.into(),
            });
        }
        let owned = Profile {
            name: p.name.clone(),
            sid: created,
            active: false,
        };
        if unsafe { EqualSid(owned.sid, p.sid) } == 0 || !present(&p)? {
            return Err(invalid("c2-created-profile-identity"));
        }
        self.append(actor, self.state.head, Action::C2ProfileReady)?;
        // Only freshly created C2 roots are granted the exact new SID. Never promote P2a fixtures.
        for (path, dir, rights) in [
            (&root, true, RX),
            (&root.join("package"), true, RX),
            (&root.join("projects"), true, MODIFY),
            (&root.join("webview"), true, MODIFY),
            (&root.join("sessions"), true, MODIFY),
            (&root.join("projects/synthetic.txt"), false, MODIFY),
            (&root.join("package/policy.json"), false, RX),
            (&root.join("package/ä.txt"), false, RX),
        ] {
            set_profile_acl(path, dir, p.sid, rights)?;
        }
        let host_entries = vec![InventoryEntry {
            path: "controller.exe".into(),
            size: io("c2-source-size", source.metadata())?.len(),
            sha256: actor,
            role: FileRole::Program,
        }];
        let ui_entries = vec![
            InventoryEntry {
                path: "policy.json".into(),
                size: POLICY.len() as u64,
                sha256: Sha256::digest(POLICY).into(),
                role: FileRole::Configuration,
            },
            InventoryEntry {
                path: "ä.txt".into(),
                size: TEXT.len() as u64,
                sha256: Sha256::digest(TEXT).into(),
                role: FileRole::Resource,
            },
        ];
        let mut digests = Vec::new();
        let mut package_pins = Vec::new();
        for (role, dir, entries, access) in [
            (PackageRole::Host, host, host_entries, None),
            (
                PackageRole::Ui,
                root.join("package"),
                ui_entries,
                Some(p.sid),
            ),
        ] {
            let bytes = contract::encode_inventory(role, &entries)
                .map_err(|_| invalid("c2-inventory-encode"))?;
            package_pins.extend(verify_package(&dir, &entries, access)?);
            write_new(&inventory_file(self, role), &bytes)?;
            digests.push(Sha256::digest(&bytes).into());
        }
        let binding = WorkspaceBindingV2 {
            version: 2,
            control_schema: 6,
            ui_abi: 1,
            installation_id: *self.state.installation.as_bytes(),
            workspace_id: *id.as_bytes(),
            profile_name: name(id),
            profile_sid: sid_text(p.sid)?,
            projects: root_identity(&root.join("projects"))?,
            webview: root_identity(&root.join("webview"))?,
            session_root: root_identity(&root.join("sessions"))?,
            host_package_sha256: digests[0],
            ui_package_sha256: digests[1],
            config_sha256: Sha256::digest(POLICY).into(),
            commands_sha256: Sha256::digest(b"C2:no-ui-or-import-commands").into(),
            job_policy_sha256: Sha256::digest(b"C2:no-execution").into(),
            acl_policy_sha256: Sha256::digest(
                b"C2:owner-system-full;profile-root-package-rx;data-modify;v1",
            )
            .into(),
            workspace_format: 1,
            db_read_min: 6,
            db_read_max: 6,
            db_write: 6,
            max_processes: 2,
            revision: 1,
        };
        let roots = BoundRoots {
            execution: root_identity(&root)?,
            host_package: root_identity(&self.root.join("bin/c2-host"))?,
            ui_package: root_identity(&root.join("package"))?,
        };
        self.append(
            actor,
            self.state.head,
            Action::BindC2Workspace { binding, roots },
        )?;
        drop(package_pins);
        self.verify_c2(actor, self.state.head).map(|_| ())
    }
    fn verify_c2(&self, actor: [u8; 32], head: [u8; 32]) -> Result<Vec<fs::File>> {
        self.c2_guard(actor, head)?;
        let state = self.state.c2.as_ref().ok_or(invalid("c2-unprepared"))?;
        let binding = state
            .binding
            .as_ref()
            .ok_or(invalid("c2-incomplete-preparation"))?;
        if state.retiring {
            return Err(invalid("c2-retiring-or-retired"));
        }
        let p = profile(state.id)?;
        if !present(&p)? || binding.profile_sid != sid_text(p.sid)? {
            return Err(invalid("c2-profile-missing-or-mismatch"));
        }
        let root = data_root(state.id);
        let mut pins = pin_directories(&root)?;
        let roots = state.roots.as_ref().ok_or(invalid("c2-roots-unbound"))?;
        if roots
            != &(BoundRoots {
                execution: root_identity(&root)?,
                host_package: root_identity(&self.root.join("bin/c2-host"))?,
                ui_package: root_identity(&root.join("package"))?,
            })
        {
            return Err(invalid("c2-package-root-identity"));
        }
        for (dir, rights, expected) in [
            ("", RX, None),
            ("package", RX, None),
            ("projects", MODIFY, Some(&binding.projects)),
            ("webview", MODIFY, Some(&binding.webview)),
            ("sessions", MODIFY, Some(&binding.session_root)),
        ] {
            let path = root.join(dir);
            pins.extend(pin_directories(&path)?);
            check_profile_acl(&path, true, p.sid, rights)?;
            if let Some(expected) = expected
                && &root_identity(&path)? != expected
            {
                return Err(invalid("c2-root-identity"));
            }
        }
        for (role, dir, expected, access) in [
            (
                PackageRole::Host,
                self.root.join("bin/c2-host"),
                binding.host_package_sha256,
                None,
            ),
            (
                PackageRole::Ui,
                root.join("package"),
                binding.ui_package_sha256,
                Some(p.sid),
            ),
        ] {
            let (bytes, entries) = read_inventory(self, role)?;
            if Sha256::digest(&bytes).as_slice() != expected {
                return Err(invalid("c2-manifest-digest"));
            }
            pins.extend(verify_package(&dir, &entries, access)?);
        }
        Ok(pins)
    }
    fn retire_c2(&mut self, actor: [u8; 32], head: [u8; 32]) -> Result<()> {
        self.c2_guard(actor, head)?;
        let s = self.state.c2.as_ref().ok_or(invalid("c2-unprepared"))?;
        if s.retired {
            return Err(invalid("c2-already-retired"));
        }
        if !s.profile_ready {
            return Err(invalid("c2-profile-ownership-unconfirmed"));
        }
        let id = s.id;
        if !s.retiring {
            if MAX_RECORDS.saturating_sub(self.state.sequence + 1) < 2 {
                return Err(invalid("c2-retire-budget"));
            }
            self.append(actor, head, Action::RetireC2Workspace)?;
        }
        let p = profile(id)?;
        if present(&p)? {
            let hr = unsafe { DeleteAppContainerProfile(p.name.as_ptr()) };
            if hr < 0 {
                return Err(Failure {
                    stage: "c2-retire-profile",
                    code: hr.into(),
                });
            }
        }
        if present(&p)? {
            return Err(invalid("c2-profile-still-present"));
        }
        self.append(actor, self.state.head, Action::C2ProfileRetired)
    }
    fn c2_summary(&self) -> Value {
        let p = self.state.c2.as_ref();
        json!({"nativePreviewOnly":true,"productionAcceptance":false,"control":self.summary(),
            "c2Bound":p.is_some_and(|s|s.binding.is_some()),"profileConfirmed":p.is_some_and(|s|s.profile_ready),
            "profileRetired":p.is_some_and(|s|s.retired),"uiStarted":false,"uiExecutable":false,"uiAbiSupported":0,
            "importTaskCreated":false,"networkMeasurement":false})
    }
}

pub(in crate::windows) fn child(args: &[OsString]) -> Result<Value> {
    if args.len() != 3 {
        return Err(invalid("c2-arguments"));
    }
    let root = Path::new(&args[0]);
    scope(root)?;
    let action = args[1].to_str().ok_or(invalid("c2-action"))?;
    if !matches!(
        action,
        "init" | "prepare" | "verify" | "exercise" | "retire" | "retired-status"
    ) {
        return Err(invalid("c2-action"));
    }
    let head = parse_hash(&args[2])?;
    let exe = io("c2-current-exe", std::env::current_exe())?;
    let (_parents, mut source, actor) = binary_pin(&exe)?;
    package::verify_persistent_records(&mut source)?;
    if action == "init" {
        if head != [0; 32] {
            return Err(invalid("c2-init-head"));
        }
        return Ok(Store::initialize_mode(root, actor, true)?.c2_summary());
    }
    let mut store = Store::open(root)?;
    store.c2_guard(actor, head)?;
    match action {
        "prepare" => store.prepare_c2(actor, head, &mut source)?,
        "retire" => store.retire_c2(actor, head)?,
        "retired-status" => {
            let p = store.state.c2.as_ref().ok_or(invalid("c2-unprepared"))?;
            if !p.retired || present(&profile(p.id)?)? {
                return Err(invalid("c2-not-retired"));
            }
        }
        "verify" | "exercise" => {
            let _pins = store.verify_c2(actor, head)?;
            if action == "exercise" {
                let id = store.state.c2.as_ref().unwrap().id;
                let path = data_root(id).join("projects/synthetic.txt");
                let mut f = open_file(&path, true, false)?;
                let p = profile(id)?;
                check_profile_acl(&path, false, p.sid, MODIFY)?;
                io("c2-exercise-truncate", f.set_len(0))?;
                io("c2-exercise-write", f.write_all(EDITED))?;
                io("c2-exercise-sync", f.sync_all())?;
            }
        }
        _ => return Err(invalid("c2-action")),
    }
    Ok(store.c2_summary())
}

#[cfg(test)]
mod tests;
