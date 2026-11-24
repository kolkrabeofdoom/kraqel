//! Native preview control store. Not a normal-product launcher or publisher-trust updater.
use super::*;
use std::ffi::OsString;
use windows_sys::Win32::Security::{
    ACCESS_ALLOWED_ACE, ACL, ACL_SIZE_INFORMATION, AclSizeInformation,
};
use zeroize::Zeroizing;
mod crash;
#[cfg(feature = "desktop-review")]
pub(crate) mod manuscript;
mod p3_fixture;
mod package;
mod secure_fs;
mod session;
mod starter;
mod win32_binding;
mod workspace;
mod workspace_persistent;
mod workspace_probe;
pub(in super::super::super::super) use crash::{child as crash_child, series as crash_series};
use secure_fs::{
    binary_pin, local_path, path_wide, pin_directories, private_acl, protected_file,
    secure_directory,
};
use session::{Binding, BoundSession};
pub(in crate::windows) use session::{child as session_child, series as session_series};
pub(in crate::windows) use session::{transition_child, transition_series};
pub(in super::super::super::super) use starter::series as start_series;
pub(in super::super) use starter::verify_start_lease;
pub(in crate::windows) use workspace::child as workspace_child;
pub(in crate::windows) use workspace_persistent::child as workspace_persistent_child;
pub(in crate::windows) use workspace_persistent::{bp2_worker, c4_root_worker};
pub(in crate::windows) use workspace_persistent::{group_child, group_worker};
pub(in crate::windows) use workspace_probe::{
    child as workspace_probe_child, worker as workspace_probe_worker,
};

const DOMAIN: &[u8] = b"KRAQEL-PRODUCT-CONTROL\0\x01";
const MAX_RECORDS: usize = crate::window_receipt::SCENE_REVISION_LIMIT as usize;
const FIXED_DRIVE: u32 = 3;
const BAD_ATTRIBUTES: u32 = FILE_ATTRIBUTE_REPARSE_POINT
    | FILE_ATTRIBUTE_OFFLINE
    | FILE_ATTRIBUTE_RECALL_ON_OPEN
    | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum Action {
    P3FixtureEvidence {
        slot: u8,
        digest: [u8; 32],
    },
    P3FixtureRemoved {
        count: u32,
    },
    P3FixtureRemoval {
        step: u32,
    },
    P3FixtureCleanup {
        digest: [u8; 32],
    },
    BeginP3Delivery {
        binding: [u8; 32],
    },
    P3Profile {
        begin: [u8; 32],
        data: bool,
        folder: [u8; 32],
        removed: bool,
    },
    BeginP3PipePreview {
        packages: (u64, [u8; 16]),
        binding: [u8; 32],
    },
    BeginWin32Window {
        anchor: [u8; 32],
    },
    RebindWin32Scene {
        begin: [u8; 32],
        publication: [u8; 32],
        revision: u32,
    },
    PublishReviewedWin32Scene {
        begin: [u8; 32],
        basis: u32,
        snapshot: win32_binding::Snapshot,
    },
    PublishWin32Scene {
        begin: [u8; 32],
        snapshot: win32_binding::Snapshot,
    },
    EndWin32SceneSave {
        begin: [u8; 32],
        snapshot: win32_binding::Snapshot,
    },
    EndWin32Window {
        begin: [u8; 32],
    },
    AdoptWin32Scene {
        previous_owner: [u8; 32],
        snapshot: win32_binding::Snapshot,
        anchor: [u8; 32],
    },
    UpgradeWin32Package {
        previous: [u8; 32],
        anchor: [u8; 32],
    },
    BindWin32Package {
        anchor: [u8; 32],
    },
    Bp2LeafProbe {
        event: workspace_persistent::Bp2Event,
    },
    C4RootProbe {
        event: workspace_persistent::C4ProbeEvent,
    },
    C3 {
        event: workspace_persistent::GroupEvent,
    },
    InitializeC2 {
        binary: [u8; 32],
    },
    PrepareC2Workspace {
        workspace: Uuid,
    },
    C2ProfileReady,
    BindC2Workspace {
        binding: crate::product_start_p2c::WorkspaceBindingV2,
        roots: workspace_persistent::BoundRoots,
    },
    RetireC2Workspace,
    C2ProfileRetired,
    Initialize {
        binary: [u8; 32],
    },
    Prepare {
        binary: [u8; 32],
    },
    Activate,
    Abort,
    Migrate {
        from: u16,
        to: u16,
    },
    BindSession {
        binding: Binding,
    },
    RecoverSession {
        launch: Uuid,
    },
    RetireSession,
    PrepareSession {
        binding: Binding,
    },
    ProfileCleaned,
    BindWorkspace {
        binding: crate::product_start::WorkspaceBindingV1,
    },
    CommitWorkspaceLaunch {
        proposal: crate::product_start::StartProposalV1,
    },
    ReconcileWorkspaceLaunch {
        launch: crate::product_start::LaunchIdentity,
    },
    CommitWorkspaceProbe {
        proposal: crate::product_start::StartProposalV1,
        execution: workspace_probe::Execution,
    },
    WorkspaceProbeProfileReady {
        launch: crate::product_start::LaunchIdentity,
    },
    WorkspaceProbeRunIntent {
        launch: crate::product_start::LaunchIdentity,
    },
    WorkspaceProbeDrained {
        launch: crate::product_start::LaunchIdentity,
    },
    WorkspaceProbeWorkerBound {
        launch: crate::product_start::LaunchIdentity,
        worker: workspace_probe::lifetime::WorkerIdentity,
        process_limit: u32,
    },
    WorkspaceProbeRecoveredDrain {
        launch: crate::product_start::LaunchIdentity,
        worker: workspace_probe::lifetime::WorkerIdentity,
        evidence: workspace_probe::lifetime::EndEvidence,
    },
    AbortUncreatedWorkspaceProbe {
        launch: crate::product_start::LaunchIdentity,
    },
    FinishWorkspaceProbe {
        launch: crate::product_start::LaunchIdentity,
        receipt_sha256: [u8; 32],
    },
    ReconcileWorkspaceProbe {
        launch: crate::product_start::LaunchIdentity,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    version: u16,
    sequence: usize,
    installation: Uuid,
    root_sha256: [u8; 32],
    previous_sha256: [u8; 32],
    operation: Uuid,
    actor: [u8; 32],
    action: Action,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Head {
    version: u16,
    installation: Uuid,
    sequence: usize,
    record_sha256: [u8; 32],
    root_sha256: [u8; 32],
}
#[derive(Clone)]
struct State {
    installation: Uuid,
    sequence: usize,
    head: [u8; 32],
    actor: [u8; 32],
    schema: u16,
    pending: Option<[u8; 32]>,
    p3_removed: bool,
    p3_removal: u32,
    p3_cleanup: Option<[u8; 32]>,
    p3_profiles: [Option<[u8; 32]>; 2],
    p3_packages: Option<(u64, [u8; 16])>,
    p3_pipe_begin: Option<[u8; 32]>,
    p3_evidence: [Option<[u8; 32]>; 3],
    scene_snapshot: Option<([u8; 32], win32_binding::Snapshot)>,
    window_session: win32_binding::Session,
    scene_adopted: bool,
    win32_upgraded: bool,
    win32_anchor: Option<([u8; 32], [u8; 32])>,
    activated: std::collections::HashSet<[u8; 32]>,
    aborted_packages: std::collections::HashSet<[u8; 32]>,
    session: Option<BoundSession>,
    workspace: Option<crate::product_start::StartModel>,
    probe: Option<workspace_probe::Pending>,
    c2: Option<workspace_persistent::Persistent>,
    c3: workspace_persistent::GroupState,
    c4_probe: workspace_persistent::C4ProbeState,
    bp2_probe: workspace_persistent::Bp2State,
}
impl State {
    fn apply(&mut self, actor: [u8; 32], action: &Action) -> Result<()> {
        if actor != self.actor {
            return Err(invalid("product-wrong-executable"));
        }
        if matches!(
            action,
            Action::BindSession { .. }
                | Action::RecoverSession { .. }
                | Action::RetireSession
                | Action::PrepareSession { .. }
                | Action::ProfileCleaned
        ) {
            return session::apply(self, action);
        }
        match action {
            Action::P3FixtureEvidence { slot, digest }
                if self.schema == 1
                    && self.p3_cleanup.is_none()
                    && self.sequence == usize::from(*slot)
                    && usize::from(*slot) < self.p3_evidence.len()
                    && *digest != [0; 32]
                    && self.p3_evidence.iter().take_while(|e| e.is_some()).count()
                        == usize::from(*slot) =>
            {
                self.p3_evidence[usize::from(*slot)] = Some(*digest);
            }
            Action::P3FixtureRemoved { count }
                if self.p3_cleanup.is_some()
                    && !self.p3_removed
                    && *count > 0
                    && self.p3_removal == *count =>
            {
                self.p3_removed = true;
            }
            Action::P3FixtureRemoval { step }
                if !self.p3_removed
                    && self.p3_cleanup.is_some()
                    && self.p3_removal.checked_add(1) == Some(*step) =>
            {
                self.p3_removal = *step;
            }
            Action::P3FixtureCleanup { digest }
                if self.schema == 1
                    && self.sequence == 2
                    && self.p3_cleanup.is_none()
                    && matches!(self.p3_evidence, [Some(_), Some(_), None])
                    && *digest != [0; 32] =>
            {
                self.p3_cleanup = Some(*digest);
            }
            Action::AdoptWin32Scene {
                previous_owner,
                snapshot,
                anchor,
            } if self.schema == 4
                && self.pending.is_none()
                && self.window_session == win32_binding::Session::Closed
                && self.win32_anchor == Some((actor, *anchor))
                && *previous_owner != actor
                && self.scene_snapshot == Some((*previous_owner, *snapshot)) =>
            {
                self.scene_snapshot = Some((actor, *snapshot));
                self.scene_adopted = true;
            }
            Action::BindWin32Package { anchor }
                if self.schema == 4
                    && self.pending.is_none()
                    && self.activated.len() > 1
                    && self.win32_anchor.is_none_or(|(owner, _)| owner != actor)
                    && *anchor != [0; 32] =>
            {
                self.win32_upgraded = false;
                self.win32_anchor = Some((actor, *anchor));
            }
            Action::UpgradeWin32Package { previous, anchor }
                if self.schema == 4
                    && self.pending.is_none()
                    && !self.win32_upgraded
                    && self.win32_anchor == Some((actor, *previous))
                    && anchor != previous
                    && *anchor != [0; 32] =>
            {
                self.win32_upgraded = true;
                self.win32_anchor = Some((actor, *anchor));
            }
            Action::Prepare { binary }
                if matches!(self.schema, 2 | 4)
                    && self.pending.is_none()
                    && *binary != [0; 32]
                    && !self.activated.contains(binary) =>
            {
                self.pending = Some(*binary)
            }
            Action::Activate if matches!(self.schema, 2 | 4) && self.pending.is_some() => {
                self.actor = self.pending.take().unwrap();
                self.activated.insert(self.actor);
            }
            Action::Abort if matches!(self.schema, 2 | 4) && self.pending.is_some() => {
                self.aborted_packages.insert(self.pending.take().unwrap());
            }
            Action::Migrate { from: 1, to: 2 } if self.pending.is_none() && self.schema == 1 => {
                self.schema = 2
            }
            Action::Migrate { from: 2, to: 3 } if self.pending.is_none() && self.schema == 2 => {
                self.schema = 3
            }
            Action::Migrate { from: 3, to: 4 } if self.pending.is_none() && self.schema == 3 => {
                self.schema = 4
            }
            Action::Migrate { from: 4, to: 5 }
                if self.pending.is_none()
                    && self.schema == 4
                    && self.session.is_none()
                    && self.workspace.is_none() =>
            {
                self.schema = 5
            }
            _ => return Err(invalid("product-invalid-transition")),
        }
        Ok(())
    }
    fn apply_record(&mut self, actor: [u8; 32], action: &Action, digest: [u8; 32]) -> Result<()> {
        if let Action::RebindWin32Scene {
            begin,
            publication,
            revision,
        } = action
        {
            // A publication advances the head while retaining the same open session token.
            if actor != self.actor
                || self.window_session != win32_binding::Session::Open(*begin)
                || *publication == *begin
                || self.head != *publication
                || !self
                    .scene_snapshot
                    .is_some_and(|(owner, snap)| owner == actor && snap.revision == *revision)
            {
                return Err(invalid("scene-rebind-publication"));
            }
            self.window_session = win32_binding::Session::Open(digest);
            return Ok(());
        }
        if let Action::PublishReviewedWin32Scene {
            begin,
            basis,
            snapshot,
        } = action
        {
            let previous = self
                .scene_snapshot
                .map_or((actor, 0), |(owner, s)| (owner, s.revision));
            if actor != self.actor
                || previous != (actor, *basis)
                || self.head != *begin
                || self.window_session != win32_binding::Session::Open(*begin)
                || snapshot.revision.saturating_sub(*basis) < 2
                || snapshot.revision >= MAX_RECORDS as u32
            {
                return Err(invalid("scene-reviewed-publication"));
            }
            self.scene_snapshot = Some((actor, *snapshot));
            return Ok(());
        }
        if let Action::P3Profile {
            begin,
            data,
            folder,
            removed,
        } = action
        {
            if actor != self.actor
                || self.p3_pipe_begin != Some(*begin)
                || self.window_session != win32_binding::Session::Open(*begin)
            {
                return Err(invalid("p3-profile-generation"));
            }
            let entry = &mut self.p3_profiles[usize::from(*data)];
            if *removed && *entry == Some(*folder) {
                *entry = None;
            } else if !removed && entry.is_none() {
                *entry = Some(*folder);
            } else {
                return Err(invalid("p3-profile-transition"));
            }
            return Ok(());
        }
        if let Action::EndWin32Window { begin }
        | Action::PublishWin32Scene { begin, .. }
        | Action::EndWin32SceneSave { begin, .. } = action
        {
            if self.p3_profiles != [None; 2]
                || actor != self.actor
                || self.window_session != win32_binding::Session::Open(*begin)
            {
                return Err(invalid("win32-window-end-binding"));
            }
            if let Action::EndWin32SceneSave { snapshot, .. }
            | Action::PublishWin32Scene { snapshot, .. } = action
            {
                if self.head != *begin {
                    return Err(invalid("scene-publication-already-recorded"));
                }
                let valid = match self.scene_snapshot {
                    None => snapshot.revision == 1,
                    Some((owner, previous)) => {
                        owner == actor
                            && previous.revision.checked_add(1) == Some(snapshot.revision)
                            && snapshot.revision <= MAX_RECORDS as u32
                    }
                };
                if !valid {
                    return Err(invalid("scene-snapshot-revision"));
                }
                self.scene_snapshot = Some((actor, *snapshot));
            }
            if matches!(action, Action::PublishWin32Scene { .. }) {
                return Ok(());
            }
            self.window_session = win32_binding::Session::Closed;
            return Ok(());
        }
        self.window_session.require_closed()?;
        if let Action::Bp2LeafProbe { event } = action {
            return workspace_persistent::apply_bp2(self, actor, event);
        }
        if self.bp2_probe.pending() {
            return Err(invalid("bp2-probe-pending"));
        }
        if let Action::C4RootProbe { event } = action {
            return workspace_persistent::apply_c4_probe(self, actor, event);
        }
        if self.c4_probe.pending() {
            return Err(invalid("c4-root-probe-pending"));
        }
        if let Action::C3 { event } = action {
            return workspace_persistent::apply_group(self, actor, event);
        }
        if let Action::BeginP3Delivery { binding } | Action::BeginP3PipePreview { binding, .. } =
            action
        {
            if actor != self.actor
                || self.schema != 1
                || self.p3_cleanup.is_some()
                || self.p3_evidence[0] != Some(*binding)
                || self.p3_evidence[2].is_none()
            {
                return Err(invalid("p3-delivery-session-binding"));
            }
            self.window_session.require_closed()?;
            self.p3_packages = match action {
                Action::BeginP3PipePreview { packages, .. } => Some(*packages),
                _ => None,
            };
            self.p3_pipe_begin =
                matches!(action, Action::BeginP3PipePreview { .. }).then_some(digest);
            self.window_session = win32_binding::Session::Open(digest);
            return Ok(());
        }
        if let Action::BeginWin32Window { anchor } = action {
            if actor != self.actor
                || self.schema != 4
                || self.pending.is_some()
                || self.win32_anchor != Some((actor, *anchor))
            {
                return Err(invalid("win32-window-start-binding"));
            }
            self.window_session = win32_binding::Session::Open(digest);
            return Ok(());
        }
        if matches!(
            action,
            Action::PrepareC2Workspace { .. }
                | Action::C2ProfileReady
                | Action::BindC2Workspace { .. }
                | Action::RetireC2Workspace
                | Action::C2ProfileRetired
        ) {
            return workspace_persistent::apply(self, actor, action);
        }
        if matches!(
            action,
            Action::CommitWorkspaceProbe { .. }
                | Action::WorkspaceProbeProfileReady { .. }
                | Action::WorkspaceProbeRunIntent { .. }
                | Action::WorkspaceProbeDrained { .. }
                | Action::WorkspaceProbeWorkerBound { .. }
                | Action::WorkspaceProbeRecoveredDrain { .. }
                | Action::AbortUncreatedWorkspaceProbe { .. }
                | Action::FinishWorkspaceProbe { .. }
                | Action::ReconcileWorkspaceProbe { .. }
        ) {
            return workspace_probe::apply(self, actor, action, digest);
        }
        if matches!(
            action,
            Action::BindWorkspace { .. }
                | Action::CommitWorkspaceLaunch { .. }
                | Action::ReconcileWorkspaceLaunch { .. }
        ) {
            workspace::apply(self, actor, action, digest)
        } else {
            self.apply(actor, action)
        }
    }
}

fn sealed(payload: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if payload.is_empty() || payload.len() > 8192 {
        return Err(invalid("product-record-budget"));
    }
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| invalid("product-mac-key"))?;
    mac.update(DOMAIN);
    mac.update(payload);
    let mut out = DOMAIN.to_vec();
    out.extend_from_slice(&mac.finalize().into_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}
fn unsealed<'a>(wire: &'a [u8], key: &[u8; 32]) -> Result<&'a [u8]> {
    let offset = DOMAIN.len() + 32;
    if wire.len() <= offset || wire.len() > offset + 8192 || !wire.starts_with(DOMAIN) {
        return Err(invalid("product-record-frame"));
    }
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| invalid("product-mac-key"))?;
    mac.update(DOMAIN);
    mac.update(&wire[offset..]);
    mac.verify_slice(&wire[DOMAIN.len()..offset])
        .map_err(|_| invalid("product-record-authentication"))?;
    Ok(&wire[offset..])
}

fn hex(hash: &[u8; 32]) -> String {
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn parse_hash(value: &std::ffi::OsStr) -> Result<[u8; 32]> {
    let text = value.to_str().ok_or(invalid("product-hash-text"))?;
    if text.len() != 64
        || !text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(invalid("product-hash-format"));
    }
    let mut hash = [0; 32];
    for (i, byte) in hash.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16)
            .map_err(|_| invalid("product-hash-format"))?;
    }
    Ok(hash)
}

struct Store {
    root: PathBuf,
    _pins: Vec<fs::File>,
    _lock: fs::File,
    key: Zeroizing<[u8; 32]>,
    state: State,
}
struct AdoptionTail {
    actor: [u8; 32],
    action: Action,
    digest: [u8; 32],
    record_file: fs::File,
}

impl Store {
    fn initialize(root: &Path, actor: [u8; 32]) -> Result<Self> {
        Self::initialize_mode(root, actor, false)
    }
    fn initialize_mode(root: &Path, actor: [u8; 32], c2: bool) -> Result<Self> {
        if c2 {
            workspace_persistent::scope(root)?;
        }
        let _parents = pin_directories(root.parent().ok_or(invalid("product-root-parent"))?)?;
        secure_directory(root)?;
        for path in [
            root.join("control"),
            root.join("control/records"),
            root.join("bin"),
            root.join("data"),
        ] {
            secure_directory(&path)?;
        }
        crash::hit(crash::Point::BeforeKey);
        let lock = io(
            "product-create-lock",
            OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .share_mode(0)
                .open(root.join("control/owner.lock")),
        )?;
        let mut key = Zeroizing::new([0; 32]);
        getrandom::fill(key.as_mut()).map_err(|_| invalid("product-random-key"))?;
        let mut file = io(
            "product-create-key",
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join("control/authority.key")),
        )?;
        io("product-key-write", file.write_all(key.as_ref()))?;
        io("product-key-sync", file.sync_all())?;
        crash::hit(crash::Point::KeySynced);
        drop(file);
        drop(lock);
        let installation = Uuid::new_v4();
        let record = Record {
            version: 1,
            sequence: 0,
            installation,
            root_sha256: root_hash(
                &root
                    .canonicalize()
                    .map_err(|_| invalid("product-root-canonical"))?,
            ),
            previous_sha256: [0; 32],
            operation: Uuid::new_v4(),
            actor,
            action: if c2 {
                Action::InitializeC2 { binary: actor }
            } else {
                Action::Initialize { binary: actor }
            },
        };
        let bytes = serde_json::to_vec(&record).map_err(|_| invalid("product-record-encode"))?;
        let wire = sealed(&bytes, &key)?;
        write_record(root, 0, &wire)?;
        write_head(root, &key, installation, 0, Sha256::digest(&wire).into())?;
        Self::open(root)
    }
    fn open(root: &Path) -> Result<Self> {
        Self::open_control(root, false).map(|(store, _)| store)
    }
    // Only the explicit recovery command may inspect one unpublished adoption.
    fn open_control(root: &Path, recover_adoption: bool) -> Result<(Self, Option<AdoptionTail>)> {
        let mut tail = None;
        let mut pins = pin_directories(root)?;
        for suffix in ["", "control", "control/records", "bin", "data"] {
            let path = root.join(suffix);
            pins.extend(pin_directories(&path)?);
            private_acl(&path, true)?;
        }
        let root = root
            .canonicalize()
            .map_err(|_| invalid("product-root-canonical"))?;
        let lock = open_file(&root.join("control/owner.lock"), true, false)?;
        private_acl(&root.join("control/owner.lock"), false)?;
        let mut key_file = protected_file(&root.join("control/authority.key"))?;
        let mut bytes = Vec::new();
        io(
            "product-key-read",
            (&mut key_file).take(33).read_to_end(&mut bytes),
        )?;
        let bytes = Zeroizing::new(bytes);
        let key = Zeroizing::new(
            bytes
                .as_slice()
                .try_into()
                .map_err(|_| invalid("product-key-size"))?,
        );
        if !recover_adoption
            && root
                .join("control/head.next")
                .try_exists()
                .map_err(|_| invalid("product-head-exists"))?
        {
            return Err(invalid("product-incomplete-head"));
        }
        let mut names = io(
            "product-list-records",
            fs::read_dir(root.join("control/records")),
        )?
        .take(MAX_RECORDS + 1)
        .map(|entry| entry.map(|e| e.file_name()))
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|_| invalid("product-list-record"))?;
        names.sort();
        if names.is_empty() || names.len() > MAX_RECORDS {
            return Err(invalid("product-record-count"));
        }
        let mut state = None;
        let mut previous = [0; 32];
        let mut operations = std::collections::HashSet::new();
        for (sequence, name) in names.iter().enumerate() {
            if name != &OsString::from(format!("{sequence:04}.mac")) {
                return Err(invalid("product-record-gap"));
            }
            let recovery_tail = recover_adoption && sequence + 1 == names.len();
            let mut file = secure_fs::protected_file_for_sync(
                &root.join("control/records").join(name),
                recovery_tail,
            )?;
            let mut wire = Vec::new();
            io(
                "product-record-read",
                (&mut file).take(16385).read_to_end(&mut wire),
            )?;
            let record: Record = serde_json::from_slice(unsealed(&wire, &key)?)
                .map_err(|_| invalid("product-record-shape"))?;
            if record.version != 1
                || record.sequence != sequence
                || record.previous_sha256 != previous
                || record.installation.is_nil()
                || record.operation.is_nil()
                || !operations.insert(record.operation)
                || record.root_sha256 != root_hash(&root)
            {
                return Err(invalid("product-record-binding"));
            }
            if sequence == 0 {
                let c2 = record.action
                    == (Action::InitializeC2 {
                        binary: record.actor,
                    });
                if (!c2
                    && record.action
                        != (Action::Initialize {
                            binary: record.actor,
                        }))
                    || record.actor == [0; 32]
                {
                    return Err(invalid("product-invalid-genesis"));
                }
                state = Some(State {
                    installation: record.installation,
                    sequence: 0,
                    head: [0; 32],
                    actor: record.actor,
                    schema: if c2 {
                        workspace_persistent::scope(&root)?;
                        6
                    } else {
                        1
                    },
                    pending: None,
                    p3_removed: false,
                    p3_removal: 0,
                    p3_cleanup: None,
                    p3_profiles: [None; 2],
                    p3_packages: None,
                    p3_pipe_begin: None,
                    p3_evidence: [None; 3],
                    scene_snapshot: None,
                    window_session: Default::default(),
                    scene_adopted: false,
                    win32_upgraded: false,
                    win32_anchor: None,
                    activated: std::collections::HashSet::from([record.actor]),
                    aborted_packages: std::collections::HashSet::new(),
                    session: None,
                    workspace: None,
                    probe: None,
                    c2: None,
                    c3: Default::default(),
                    c4_probe: Default::default(),
                    bp2_probe: Default::default(),
                });
            } else {
                let s = state.as_mut().ok_or(invalid("product-genesis-missing"))?;
                if s.installation != record.installation {
                    return Err(invalid("product-installation-binding"));
                }
                if recovery_tail {
                    if !matches!(record.action, Action::AdoptWin32Scene { .. }) {
                        return Err(invalid("scene-adopt-no-pending-record"));
                    }
                    tail = Some(AdoptionTail {
                        actor: record.actor,
                        action: record.action,
                        digest: Sha256::digest(&wire).into(),
                        record_file: file,
                    });
                    break;
                }
                s.apply_record(record.actor, &record.action, Sha256::digest(&wire).into())?;
            }
            previous = Sha256::digest(&wire).into();
            let s = state.as_mut().unwrap();
            s.sequence = sequence;
            s.head = previous;
        }
        let state = state.ok_or(invalid("product-empty-store"))?;
        let mut file = protected_file(&root.join("control/head.mac"))?;
        let mut wire = Vec::new();
        io(
            "product-head-read",
            (&mut file).take(16385).read_to_end(&mut wire),
        )?;
        let head: Head = serde_json::from_slice(
            unsealed(&wire, &key)?
                .strip_prefix(b"HEAD\0")
                .ok_or(invalid("product-head-domain"))?,
        )
        .map_err(|_| invalid("product-head-shape"))?;
        if head.version != 1
            || head.installation != state.installation
            || head.sequence != state.sequence
            || head.record_sha256 != state.head
            || head.root_sha256 != root_hash(&root)
        {
            return Err(invalid("product-uncommitted-or-truncated-chain"));
        }
        Ok((
            Self {
                root,
                _pins: pins,
                _lock: lock,
                key,
                state,
            },
            tail,
        ))
    }
    fn actor(&self, actor: [u8; 32]) -> Result<()> {
        if actor != self.state.actor {
            return Err(invalid("product-wrong-executable"));
        }
        Ok(())
    }
    fn idle(&self) -> Result<()> {
        Self::job_idle(&self.job_name())
    }
    fn job_idle(name: &[u16]) -> Result<()> {
        let raw = unsafe { OpenJobObjectW(JOB_OBJECT_QUERY, 0, name.as_ptr()) };
        if raw.is_null() {
            if unsafe { GetLastError() } == ERROR_FILE_NOT_FOUND {
                return Ok(());
            }
            return Err(fail("product-job-open"));
        }
        let job = Handle(raw);
        let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
        check(
            unsafe {
                QueryInformationJobObject(
                    job.0,
                    JobObjectBasicAccountingInformation,
                    (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                    null_mut(),
                )
            },
            "product-job-query",
        )?;
        if info.ActiveProcesses != 0 {
            return Err(invalid("product-job-active"));
        }
        Ok(())
    }
    fn job_name(&self) -> Vec<u16> {
        wide(format!(
            "Local\\KRAQEL.Product.Broker.{}",
            self.state.installation.simple()
        ))
    }
    fn append_guard(&self, actor: [u8; 32], head: [u8; 32]) -> Result<()> {
        self.actor(actor)?;
        self.state.window_session.require_closed()?;
        if self.state.bp2_probe.pending() {
            return Err(invalid("bp2-probe-pending"));
        }
        if self.state.c3.pending() {
            return Err(invalid("c3-group-pending"));
        }
        if self.state.c4_probe.pending() {
            return Err(invalid("c4-root-probe-pending"));
        }
        self.idle()?;
        if self.state.head != head || self.state.sequence + 1 >= MAX_RECORDS {
            return Err(invalid("product-stale-head-or-budget"));
        }
        Ok(())
    }
    fn next_state(&self, actor: [u8; 32], head: [u8; 32], action: &Action) -> Result<State> {
        self.append_guard(actor, head)?;
        let mut next = self.state.clone();
        next.apply(actor, action)?;
        Ok(next)
    }
    fn append(&mut self, actor: [u8; 32], head: [u8; 32], action: Action) -> Result<()> {
        self.append_guard(actor, head)?;
        self.append_with_checked_context(actor, head, action)
    }
    // Serialization only. Callers must first prove idle control OR the exact suspended worker
    // under the held lease; this method never grants native execution/cleanup authority itself.
    fn append_with_checked_context(
        &mut self,
        actor: [u8; 32],
        head: [u8; 32],
        action: Action,
    ) -> Result<()> {
        self.actor(actor)?;
        if self.state.head != head || self.state.sequence + 1 >= MAX_RECORDS {
            return Err(invalid("product-stale-head-or-budget"));
        }
        let record = Record {
            version: 1,
            sequence: self.state.sequence + 1,
            installation: self.state.installation,
            root_sha256: root_hash(&self.root),
            previous_sha256: head,
            operation: Uuid::new_v4(),
            actor,
            action,
        };
        let bytes = serde_json::to_vec(&record).map_err(|_| invalid("product-record-encode"))?;
        let wire = sealed(&bytes, &self.key)?;
        let mut next = self.state.clone();
        next.apply_record(actor, &record.action, Sha256::digest(&wire).into())?;
        crash::hit(crash::Point::BeforeRecord);
        write_record(&self.root, record.sequence, &wire)?;
        write_head(
            &self.root,
            &self.key,
            record.installation,
            record.sequence,
            Sha256::digest(&wire).into(),
        )?;
        next.sequence = record.sequence;
        next.head = Sha256::digest(wire).into();
        self.state = next;
        Ok(())
    }
    fn prepare(
        &mut self,
        actor: [u8; 32],
        head: [u8; 32],
        candidate: &Path,
        expected: [u8; 32],
    ) -> Result<()> {
        let (_parents, mut source, actual) = binary_pin(candidate)?;
        if actual != expected {
            return Err(invalid("product-candidate-hash"));
        }
        let mut kind = 0;
        check(
            unsafe { GetBinaryTypeW(path_wide(candidate)?.as_ptr(), &mut kind) },
            "product-candidate-not-executable",
        )?;
        if kind != 6 {
            return Err(invalid("product-candidate-not-x64"));
        }
        if self.state.schema == 4 {
            package::verify(&mut source)?;
            if self.state.win32_anchor.is_some() {
                package::verify_win32_binding(&mut source)?;
                package::verify_legacy_read_policy(&mut source)?;
                if self.state.scene_adopted {
                    package::verify_scene_adoption(&mut source)?;
                }
                if self.state.window_session != win32_binding::Session::Unused {
                    package::verify_window_session(&mut source)?;
                    if self.state.scene_snapshot.is_some() {
                        package::verify_scene_snapshot(&mut source)?;
                        if self
                            .state
                            .scene_snapshot
                            .is_some_and(|(_, s)| s.revision > 1)
                        {
                            package::verify_scene_revisions(&mut source)?;
                        }
                    }
                }
            }
        }
        let action = Action::Prepare { binary: actual };
        // Authorize before touching leftovers. The same owner lease is held throughout.
        self.next_state(actor, head, &action)?;
        let path = self.package(actual);
        if self.state.schema == 4 && self.state.aborted_packages.contains(&actual) {
            secure_fs::remove_aborted_package(&path)?;
            crash::hit(crash::Point::ArtifactRemoved);
        }
        // A collision/locked/unsafe leftover must fail before signing a new pending intent.
        let mut dest = io(
            "product-copy-create",
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .share_mode(0)
                .open(&path),
        )?;
        private_acl(&path, false)?;
        crash::hit(crash::Point::PackageCreated);
        use std::io::{Seek, SeekFrom};
        io("product-source-rewind", source.seek(SeekFrom::Start(0)))?;
        self.append(actor, head, action)?;
        crash::hit(crash::Point::PrepareRecorded);
        io(
            "product-copy-prefix",
            std::io::copy(&mut (&mut source).take(1024), &mut dest),
        )?;
        crash::hit(crash::Point::PackagePartial);
        io("product-copy", std::io::copy(&mut source, &mut dest))?;
        io("product-copy-sync", dest.sync_all())?;
        crash::hit(crash::Point::PackageSynced);
        drop(dest);
        if hash(&path)? != actual {
            return Err(invalid("product-copy-hash"));
        }
        private_acl(&path, false)?;
        Ok(())
    }
    fn package(&self, hash: [u8; 32]) -> PathBuf {
        self.root.join("bin").join(format!("{}.exe", hex(&hash)))
    }
    fn activate(&mut self, actor: [u8; 32], head: [u8; 32]) -> Result<()> {
        let binary = self
            .state
            .pending
            .ok_or(invalid("product-no-pending-update"))?;
        let path = self.package(binary);
        let mut file = protected_file(&path)?;
        if hash(&path)? != binary {
            return Err(invalid("product-candidate-tampered"));
        }
        if self.state.schema == 4 {
            package::verify(&mut file)?;
            if self.state.win32_anchor.is_some() {
                package::verify_win32_binding(&mut file)?;
                package::verify_legacy_read_policy(&mut file)?;
                if self.state.scene_adopted {
                    package::verify_scene_adoption(&mut file)?;
                }
                if self.state.window_session != win32_binding::Session::Unused {
                    package::verify_window_session(&mut file)?;
                    if self.state.scene_snapshot.is_some() {
                        package::verify_scene_snapshot(&mut file)?;
                        if self
                            .state
                            .scene_snapshot
                            .is_some_and(|(_, s)| s.revision > 1)
                        {
                            package::verify_scene_revisions(&mut file)?;
                        }
                    }
                }
            }
        }
        self.append(actor, head, Action::Activate)
    }
    fn summary(&self) -> Value {
        json!({"nativePreviewOnly":true,"productionAcceptance":false,"sequence":self.state.sequence,"controlSchema":self.state.schema,"pendingUpdate":self.state.pending.is_some(),"windowSessionOpen":matches!(self.state.window_session,win32_binding::Session::Open(_)),"headSha256":hex(&self.state.head),"activeBinarySha256":hex(&self.state.actor),"networkMeasurement":false})
    }
    fn check_container(&self, app: &Path) -> Result<Value> {
        let root = std::env::temp_dir().join(format!(
            "kraqel-isolation-ui-restart-{}",
            Uuid::new_v4().simple()
        ));
        let (session, mut profile, pin) = super::super::initialize(&root, app)?;
        let ticket = RestartTicket {
            version: RestartTicket::VERSION,
            session_id: session,
            task_id: Uuid::new_v4(),
            launch_id: Uuid::new_v4(),
            recovery: false,
            automatic: true,
            source: kraqel_isolation_transport::RestartSource::Synthetic,
            point: CrashPoint::Completed,
            app_sha256: pin,
        };
        generation_with_options(
            &root,
            &profile,
            &ticket,
            ticket.launch_id,
            GenerationOptions {
                expect: Expect::Crash,
                index: 0,
                durable_job: None,
                crash_controller: false,
                live_mode: None,
                product_root: Some(&self.root),
            },
        )?;
        if !profile.remove() {
            return Err(invalid("product-container-profile-cleanup"));
        }
        Ok(
            json!({"nativePreviewOnly":true,"productionAcceptance":false,"actualContainerDenials":6,"token":{"isAppContainer":true,"capabilityCount":0,"expectedIdentity":true},"activeJobProcessesAtEnd":0,"profileRemoved":true,"localDiagnosticRoot":root,"networkMeasurement":false}),
        )
    }
}
pub(in super::super) fn verify_denials(process: HANDLE, root: &Path) -> Result<()> {
    let mut raw = null_mut();
    check(
        unsafe { OpenProcessToken(process, TOKEN_QUERY | TOKEN_DUPLICATE, &mut raw) },
        "product-container-token",
    )?;
    let token = Handle(raw);
    check(
        unsafe { ImpersonateLoggedOnUser(token.0) },
        "product-container-impersonate",
    )?;
    let result = (|| {
        for (path, rights) in [
            ("control/authority.key", GENERIC_READ),
            ("control/authority.key", GENERIC_WRITE),
            ("control/head.mac", GENERIC_READ),
            ("control/head.mac", GENERIC_WRITE),
            ("control/owner.lock", GENERIC_READ),
            ("control/records/0000.mac", GENERIC_READ),
        ] {
            let file = unsafe {
                CreateFileW(
                    path_wide(&root.join(path))?.as_ptr(),
                    rights,
                    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                    null(),
                    OPEN_EXISTING,
                    FILE_FLAG_OPEN_REPARSE_POINT,
                    null_mut(),
                )
            };
            if file != INVALID_HANDLE_VALUE {
                drop(Handle(file));
                return Err(invalid("product-container-access-not-denied"));
            }
            if unsafe { GetLastError() } != ERROR_ACCESS_DENIED {
                return Err(fail("product-container-denial-not-calibrated"));
            }
        }
        Ok(())
    })();
    if unsafe { RevertToSelf() } == 0 {
        std::process::abort();
    }
    result
}
fn write_record(root: &Path, sequence: usize, bytes: &[u8]) -> Result<()> {
    let mut file = io(
        "product-record-create",
        OpenOptions::new().write(true).create_new(true).open(
            root.join("control/records")
                .join(format!("{sequence:04}.mac")),
        ),
    )?;
    crash::hit(crash::Point::RecordCreated);
    io("product-record-write", file.write_all(bytes))?;
    io("product-record-sync", file.sync_all())?;
    crash::hit(crash::Point::RecordSynced);
    Ok(())
}
fn head_wire(
    root: &Path,
    key: &[u8; 32],
    installation: Uuid,
    sequence: usize,
    record_sha256: [u8; 32],
) -> Result<Vec<u8>> {
    let head = Head {
        version: 1,
        installation,
        sequence,
        record_sha256,
        root_sha256: root_hash(
            &root
                .canonicalize()
                .map_err(|_| invalid("product-head-root"))?,
        ),
    };
    let mut bytes = b"HEAD\0".to_vec();
    bytes.extend(serde_json::to_vec(&head).map_err(|_| invalid("product-head-encode"))?);
    sealed(&bytes, key)
}

fn write_head(
    root: &Path,
    key: &[u8; 32],
    installation: Uuid,
    sequence: usize,
    record_sha256: [u8; 32],
) -> Result<()> {
    let wire = head_wire(root, key, installation, sequence, record_sha256)?;
    let pending = root.join("control/head.next");
    let mut file = io(
        "product-head-create",
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pending),
    )?;
    io("product-head-write", file.write_all(&wire))?;
    io("product-head-sync", file.sync_all())?;
    crash::hit(crash::Point::HeadStaged);
    drop(file);
    check(
        unsafe {
            MoveFileExW(
                path_wide(&pending)?.as_ptr(),
                path_wide(&root.join("control/head.mac"))?.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        },
        "product-head-commit",
    )?;
    crash::hit(crash::Point::HeadPublished);
    Ok(())
}
fn default_root() -> Result<PathBuf> {
    let local = std::env::var_os("USERPROFILE").ok_or(invalid("product-local-data-missing"))?;
    let path = local_path(&PathBuf::from(local))?;
    let _pins = pin_directories(&path)?;
    // Separate preview namespace; never adopts normal app data or TEMP research roots.
    // A packaged host can virtualize LocalAppData into its own removable cache. The explicit
    // private local home directory is stable across host packages; remote/cloud homes are denied.
    Ok(path.join(".kraqel-broker-preview"))
}
pub(in super::super::super::super) fn command(args: &[OsString]) -> Result<Value> {
    if let [operation, run] = args {
        if operation == "inspect-p3-fixture-catalog" {
            use kraqel_storage::project_manager::ProjectManager;
            let run = run
                .to_str()
                .filter(|s| s.len() == 32 && s.bytes().all(|b| b.is_ascii_hexdigit()))
                .ok_or(invalid("p3-catalog-scope"))?;
            let catalog = local_path(&std::env::temp_dir())?
                .join(format!("kraqel-p3-writable-{run}"))
                .join("projects");
            let _pins = pin_directories(&catalog)?;
            let candidates = ProjectManager::new(catalog)
                .and_then(|manager| manager.existing_project_candidates())
                .map_err(|_| invalid("p3-catalog-unavailable"))?;
            return Ok(json!({"candidates": candidates}));
        }
    }
    let root = default_root()?;
    let exe = io("product-current-exe", std::env::current_exe())?;
    let (_parents, running_image, actor) = binary_pin(&exe)?;
    if let [name] = args {
        use p3_fixture::Operation;
        let operation = match name.to_str() {
            Some("authorize-p3-fixture") => Some(Operation::Authorize),
            Some("reopen-p3-fixture") => Some(Operation::Reopen),
            Some("confirm-p3-fixture-cleanup") => Some(Operation::Cleanup),
            Some("authorize-p3-comparison") => Some(Operation::AuthorizeComparison),
            Some("read-p3-comparison") => Some(Operation::ReadComparison),
            Some("read-p3-fixture") => Some(Operation::Read),
            Some("retire-p3-preview-profiles") => Some(Operation::Profiles),
            _ => None,
        };
        if let Some(operation) = operation {
            return p3_fixture::serve(&root, &running_image, actor, operation);
        }
    }
    if args.len() == 1 && args[0] == "init" {
        return Ok(Store::initialize(&root, actor)?.summary());
    }
    let recovery = args.len() == 4 && args[0] == "recover-win32-scene-adoption";
    let (mut store, tail) = Store::open_control(&root, recovery)?;
    store.actor(actor)?;
    if args.len() == 1 && args[0] == "status" {
        return Ok(store.summary());
    }
    store.state.window_session.require_closed()?;
    if (args.len() == 4
        && (args[0] == "bind-win32-package"
            || args[0] == "upgrade-win32-package"
            || args[0] == "adopt-win32-scene"
            || recovery))
        || (args.len() == 2
            && (args[0] == "verify-win32-package"
                || args[0] == "check-win32-suspended"
                || args[0] == "check-win32-receipt"
                || args[0] == "check-win32-surface"
                || args[0] == "open-win32-surface"
                || args[0] == "open-win32-scene"
                || args[0] == "check-win32-scene-denied-save"
                || args[0] == "check-win32-scene-edit-conflict"
                || args[0] == "continue-win32-scene"
                || args[0] == "checkpoint-win32-scene"
                || args[0] == "edit-win32-scene"
                || args[0] == "save-win32-scene"
                || args[0] == "read-saved-win32-scene"))
    {
        // The running broker itself must be the managed file, not a byte-identical copy.
        let running = file_info(&running_image)?;
        let managed = protected_file(&store.package(actor))?;
        let identity = file_info(&managed)?;
        if identity.nNumberOfLinks != 1
            || running.dwVolumeSerialNumber != identity.dwVolumeSerialNumber
            || running.nFileIndexHigh != identity.nFileIndexHigh
            || running.nFileIndexLow != identity.nFileIndexLow
        {
            return Err(invalid("win32-broker-not-managed-image"));
        }
        let head = parse_hash(args.last().unwrap())?;
        let operation = if recovery || (args.len() == 4 && args[0] == "adopt-win32-scene") {
            let revision = args[1]
                .to_str()
                .and_then(|v| v.parse::<u32>().ok())
                .filter(|v| *v > 0)
                .ok_or(invalid("scene-adopt-revision"))?;
            let digest = parse_hash(&args[2])?;
            if recovery {
                win32_binding::Operation::Recover(
                    revision,
                    digest,
                    tail.as_ref()
                        .ok_or(invalid("scene-adopt-no-pending-record"))?,
                )
            } else {
                win32_binding::Operation::Adopt(revision, digest)
            }
        } else if args.len() == 4 && args[0] == "upgrade-win32-package" {
            win32_binding::Operation::Upgrade(Path::new(&args[1]), parse_hash(&args[2])?)
        } else if args.len() == 4 {
            win32_binding::Operation::Bind(Path::new(&args[1]), parse_hash(&args[2])?)
        } else if args[0] == "check-win32-suspended" {
            win32_binding::Operation::CheckWindow(win32_binding::WindowMode::Suspended)
        } else if args[0] == "check-win32-receipt" {
            win32_binding::Operation::CheckWindow(win32_binding::WindowMode::Receipt)
        } else if args[0] == "open-win32-scene" {
            win32_binding::Operation::CheckWindow(win32_binding::WindowMode::Scene(
                crate::window_receipt::SceneOperation::Once,
            ))
        } else if args[0] == "continue-win32-scene" {
            win32_binding::Operation::CheckWindow(win32_binding::WindowMode::Continue)
        } else if args[0] == "checkpoint-win32-scene" {
            win32_binding::Operation::CheckWindow(win32_binding::WindowMode::Checkpoint)
        } else if args[0] == "edit-win32-scene"
            || args[0] == "save-win32-scene"
            || args[0] == "read-saved-win32-scene"
        {
            use crate::window_receipt::SceneOperation;
            win32_binding::Operation::CheckWindow(win32_binding::WindowMode::Scene(
                if args[0] == "edit-win32-scene" {
                    SceneOperation::KeyboardEdit
                } else if args[0] == "save-win32-scene" {
                    SceneOperation::SavePersistent
                } else {
                    SceneOperation::ReadPersistent
                },
            ))
        } else if args[0] == "check-win32-scene-edit-conflict" {
            win32_binding::Operation::CheckWindow(win32_binding::WindowMode::Scene(
                crate::window_receipt::SceneOperation::CheckEditConflict,
            ))
        } else if args[0] == "check-win32-scene-denied-save" {
            win32_binding::Operation::CheckWindow(win32_binding::WindowMode::Scene(
                crate::window_receipt::SceneOperation::CheckDeniedSave,
            ))
        } else if args[0] == "open-win32-surface" {
            win32_binding::Operation::CheckWindow(win32_binding::WindowMode::Interactive)
        } else if args[0] == "check-win32-surface" {
            win32_binding::Operation::CheckWindow(win32_binding::WindowMode::Surface)
        } else {
            win32_binding::Operation::Verify
        };
        return store.win32_binding(actor, head, operation);
    }
    if args.len() == 2 && args[0] == "check-container" {
        return store.check_container(Path::new(&args[1]));
    }
    if args.len() == 4 && args[0] == "start-check" {
        return store.start_check(
            actor,
            parse_hash(&args[3])?,
            Path::new(&args[1]),
            parse_hash(&args[2])?,
        );
    }
    if args.len() == 5 && args[0] == "bind-session" {
        let point: CrashPoint = serde_json::from_value(json!(args[3].to_string_lossy()))
            .map_err(|_| invalid("product-session-point"))?;
        return store.bind_session(
            actor,
            parse_hash(&args[4])?,
            Path::new(&args[1]),
            parse_hash(&args[2])?,
            point,
        );
    }
    if args.len() == 2 && args[0] == "recover-session" {
        return store.recover_session(actor, parse_hash(&args[1])?);
    }
    if args.len() == 2 && args[0] == "retire-session" {
        return store.retire_session(actor, parse_hash(&args[1])?);
    }
    if args.len() == 2 && args[0] == "cleanup-session" {
        return store.cleanup_profile(actor, parse_hash(&args[1])?);
    }
    if args.len() == 4 && args[0] == "prepare" {
        store.prepare(
            actor,
            parse_hash(&args[3])?,
            Path::new(&args[1]),
            parse_hash(&args[2])?,
        )?;
    } else if args.len() == 2 && args[0] == "activate" {
        store.activate(actor, parse_hash(&args[1])?)?;
    } else if args.len() == 2 && args[0] == "abort" {
        store.append(actor, parse_hash(&args[1])?, Action::Abort)?;
    } else if args.len() == 2 && args[0] == "migrate-v4" {
        store.append(
            actor,
            parse_hash(&args[1])?,
            Action::Migrate { from: 3, to: 4 },
        )?;
    } else if args.len() == 2 && args[0] == "migrate-v3" {
        store.append(
            actor,
            parse_hash(&args[1])?,
            Action::Migrate { from: 2, to: 3 },
        )?;
    } else if args.len() == 2 && args[0] == "migrate-v2" {
        store.append(
            actor,
            parse_hash(&args[1])?,
            Action::Migrate { from: 1, to: 2 },
        )?;
    } else {
        return Err(invalid("product-command"));
    }
    Ok(store.summary())
}

#[cfg(test)]
mod tests;
