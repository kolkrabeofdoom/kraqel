//! Native preview control store. Not a normal-product launcher or publisher-trust updater.
use super::*;
use std::ffi::OsString;
use windows_sys::Win32::Security::{
    ACCESS_ALLOWED_ACE, ACL, ACL_SIZE_INFORMATION, AclSizeInformation,
};
use zeroize::Zeroizing;
mod crash;
mod package;
mod secure_fs;
mod session;
mod starter;
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

const DOMAIN: &[u8] = b"KRAQEL-PRODUCT-CONTROL\0\x01";
const MAX_RECORDS: usize = 256;
const FIXED_DRIVE: u32 = 3;
const BAD_ATTRIBUTES: u32 = FILE_ATTRIBUTE_REPARSE_POINT
    | FILE_ATTRIBUTE_OFFLINE
    | FILE_ATTRIBUTE_RECALL_ON_OPEN
    | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum Action {
    Initialize { binary: [u8; 32] },
    Prepare { binary: [u8; 32] },
    Activate,
    Abort,
    Migrate { from: u16, to: u16 },
    BindSession { binding: Binding },
    RecoverSession { launch: Uuid },
    RetireSession,
    PrepareSession { binding: Binding },
    ProfileCleaned,
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
struct State {
    installation: Uuid,
    sequence: usize,
    head: [u8; 32],
    actor: [u8; 32],
    schema: u16,
    pending: Option<[u8; 32]>,
    activated: std::collections::HashSet<[u8; 32]>,
    session: Option<BoundSession>,
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
                self.pending = None
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
            _ => return Err(invalid("product-invalid-transition")),
        }
        Ok(())
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
impl Store {
    fn initialize(root: &Path, actor: [u8; 32]) -> Result<Self> {
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
            action: Action::Initialize { binary: actor },
        };
        let bytes = serde_json::to_vec(&record).map_err(|_| invalid("product-record-encode"))?;
        let wire = sealed(&bytes, &key)?;
        write_record(root, 0, &wire)?;
        write_head(root, &key, installation, 0, Sha256::digest(&wire).into())?;
        Self::open(root)
    }
    fn open(root: &Path) -> Result<Self> {
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
        if root
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
            let mut file = protected_file(&root.join("control/records").join(name))?;
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
                if record.action
                    != (Action::Initialize {
                        binary: record.actor,
                    })
                    || record.actor == [0; 32]
                {
                    return Err(invalid("product-invalid-genesis"));
                }
                state = Some(State {
                    installation: record.installation,
                    sequence: 0,
                    head: [0; 32],
                    actor: record.actor,
                    schema: 1,
                    pending: None,
                    activated: std::collections::HashSet::from([record.actor]),
                    session: None,
                });
            } else {
                let s = state.as_mut().ok_or(invalid("product-genesis-missing"))?;
                if s.installation != record.installation {
                    return Err(invalid("product-installation-binding"));
                }
                s.apply(record.actor, &record.action)?;
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
        Ok(Self {
            root,
            _pins: pins,
            _lock: lock,
            key,
            state,
        })
    }
    fn actor(&self, actor: [u8; 32]) -> Result<()> {
        if actor != self.state.actor {
            return Err(invalid("product-wrong-executable"));
        }
        Ok(())
    }
    fn idle(&self) -> Result<()> {
        let name = self.job_name();
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
    fn append(&mut self, actor: [u8; 32], head: [u8; 32], action: Action) -> Result<()> {
        self.actor(actor)?;
        self.idle()?;
        if self.state.head != head || self.state.sequence + 1 >= MAX_RECORDS {
            return Err(invalid("product-stale-head-or-budget"));
        }
        // Validate before any write. Persist first; any incomplete record blocks later opens.
        let mut next = State {
            installation: self.state.installation,
            sequence: self.state.sequence,
            head: self.state.head,
            actor: self.state.actor,
            schema: self.state.schema,
            pending: self.state.pending,
            activated: self.state.activated.clone(),
            session: self.state.session.clone(),
        };
        next.apply(actor, &action)?;
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
        }
        self.append(actor, head, Action::Prepare { binary: actual })?;
        use std::io::{Seek, SeekFrom};
        io("product-source-rewind", source.seek(SeekFrom::Start(0)))?;
        let path = self.package(actual);
        let mut dest = io(
            "product-copy-create",
            OpenOptions::new().write(true).create_new(true).open(&path),
        )?;
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
        }
        self.append(actor, head, Action::Activate)
    }
    fn summary(&self) -> Value {
        json!({"nativePreviewOnly":true,"productionAcceptance":false,"sequence":self.state.sequence,"controlSchema":self.state.schema,"pendingUpdate":self.state.pending.is_some(),"headSha256":hex(&self.state.head),"activeBinarySha256":hex(&self.state.actor),"networkMeasurement":false})
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
fn write_head(
    root: &Path,
    key: &[u8; 32],
    installation: Uuid,
    sequence: usize,
    record_sha256: [u8; 32],
) -> Result<()> {
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
    let wire = sealed(&bytes, key)?;
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
    let root = default_root()?;
    let exe = io("product-current-exe", std::env::current_exe())?;
    let (_parents, _exe, actor) = binary_pin(&exe)?;
    if args.len() == 1 && args[0] == "init" {
        return Ok(Store::initialize(&root, actor)?.summary());
    }
    let mut store = Store::open(&root)?;
    store.actor(actor)?;
    if args.len() == 1 && args[0] == "status" {
        return Ok(store.summary());
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
