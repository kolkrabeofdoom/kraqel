//! Durable native recovery authority, deliberately gated to synthetic research sessions.
//! Local HMAC authority; no account recovery, domain backup or DPAPI path.
use super::*;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use windows_sys::Win32::System::SystemServices::JOB_OBJECT_QUERY;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Authority {
    version: u16,
    root_sha256: [u8; 32],
    controller_sha256: [u8; 32],
    ticket: RestartTicket,
}

const MAGIC: &[u8] = b"KRAQEL-BROKER-AUTHORITY\0\x02";
const MAX_SEAL: usize = 32768;
const TAG_LEN: usize = 32;
type HmacSha256 = Hmac<Sha256>;

fn sign(bytes: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if bytes.is_empty() || bytes.len() > MAX_SEAL - MAGIC.len() - TAG_LEN {
        return Err(invalid("broker-seal-size"));
    }
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| invalid("broker-mac-key"))?;
    mac.update(MAGIC);
    mac.update(bytes);
    let mut sealed = MAGIC.to_vec();
    sealed.extend_from_slice(&mac.finalize().into_bytes());
    sealed.extend_from_slice(bytes);
    Ok(sealed)
}
fn verify<'a>(sealed: &'a [u8], key: &[u8; 32]) -> Result<&'a [u8]> {
    let offset = MAGIC.len() + TAG_LEN;
    if sealed.len() <= offset || sealed.len() > MAX_SEAL || !sealed.starts_with(MAGIC) {
        return Err(invalid("broker-seal-frame"));
    }
    let bytes = &sealed[offset..];
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| invalid("broker-mac-key"))?;
    mac.update(MAGIC);
    mac.update(bytes);
    mac.verify_slice(&sealed[MAGIC.len()..offset])
        .map_err(|_| invalid("broker-seal-authentication"))?;
    Ok(bytes)
}
fn read_key(root: &Path) -> Result<[u8; 32]> {
    let file = open_file(&root.join("control/authority.key"), false, false)?;
    let mut bytes = Vec::new();
    io("broker-key-read", file.take(33).read_to_end(&mut bytes))?;
    bytes.try_into().map_err(|_| invalid("broker-key-size"))
}

fn private_directory(path: &Path) -> Result<()> {
    let mut raw = null_mut();
    check(
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) },
        "broker-user-token",
    )?;
    let token = Handle(raw);
    let mut size = 0;
    unsafe { GetTokenInformation(token.0, TokenUser, null_mut(), 0, &mut size) };
    if !(size_of::<TOKEN_USER>()..=65536).contains(&(size as usize)) {
        return Err(invalid("broker-user-token-size"));
    }
    let mut buffer = vec![0usize; (size as usize).div_ceil(size_of::<usize>())];
    check(
        unsafe {
            GetTokenInformation(
                token.0,
                TokenUser,
                buffer.as_mut_ptr().cast(),
                size,
                &mut size,
            )
        },
        "broker-user-token-read",
    )?;
    let sid = unsafe { (*buffer.as_ptr().cast::<TOKEN_USER>()).User.Sid };
    let mut text = null_mut();
    check(
        unsafe { ConvertSidToStringSidW(sid, &mut text) },
        "broker-user-sid",
    )?;
    let _text = Local(text.cast());
    let mut len = 0;
    while len < 256 && unsafe { *text.add(len) } != 0 {
        len += 1;
    }
    if len == 256 {
        return Err(invalid("broker-user-sid-size"));
    }
    let sid = String::from_utf16(unsafe { std::slice::from_raw_parts(text, len) })
        .map_err(|_| invalid("broker-user-sid-text"))?;
    let mut sd = null_mut();
    // Protected DACL, only this account and SYSTEM. No package SID, ALL APPLICATION PACKAGES,
    // Users, Everyone or inherited research-root entries. AppContainer needs BOTH token grants.
    check(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide(format!("D:P(A;OICI;FA;;;{sid})(A;OICI;FA;;;SY)")).as_ptr(),
                1,
                &mut sd,
                null_mut(),
            )
        },
        "broker-private-descriptor",
    )?;
    let _sd = Local(sd);
    let mut acl = null_mut();
    let mut present = 0;
    let mut defaulted = 0;
    check(
        unsafe { GetSecurityDescriptorDacl(sd, &mut present, &mut acl, &mut defaulted) },
        "broker-private-dacl",
    )?;
    let status = unsafe {
        SetNamedSecurityInfoW(
            wide(path).as_ptr(),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            null_mut(),
            null_mut(),
            acl,
            null(),
        )
    };
    if status != 0 {
        return Err(Failure {
            stage: "broker-private-acl",
            code: status.into(),
        });
    }
    Ok(())
}

fn bound_root(root: &Path) -> Result<PathBuf> {
    if !crate::policy::valid_root(root, true) {
        return Err(invalid("broker-root-scope"));
    }
    plain(root, true)?;
    let canonical = io("broker-root-canonical", root.canonicalize())?;
    let session = session(&canonical)?;
    if canonical.file_name()
        != Some(std::ffi::OsStr::new(&format!(
            "kraqel-isolation-ui-restart-{}",
            session.simple()
        )))
    {
        return Err(invalid("broker-root-name"));
    }
    Ok(canonical)
}
fn session(root: &Path) -> Result<Uuid> {
    root.file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.strip_prefix("kraqel-isolation-ui-restart-"))
        .and_then(|n| Uuid::parse_str(n).ok())
        .filter(|u| !u.is_nil())
        .ok_or(invalid("broker-session"))
}
fn plain(path: &Path, directory: bool) -> Result<()> {
    let meta = io("broker-path-metadata", fs::symlink_metadata(path))?;
    if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || meta.is_dir() != directory
        || (!directory && !meta.is_file())
    {
        return Err(invalid("broker-linked-or-wrong-path"));
    }
    Ok(())
}
fn open_file(path: &Path, write: bool, create: bool) -> Result<fs::File> {
    let file = io(
        "broker-state-open",
        OpenOptions::new()
            .read(true)
            .write(write)
            .create(create)
            .share_mode(if write { 0 } else { FILE_SHARE_READ })
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path),
    )?;
    let info = file_info(&file)?;
    if info.dwFileAttributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) != 0
        || info.nNumberOfLinks != 1
    {
        return Err(invalid("broker-state-file-kind"));
    }
    Ok(file)
}
struct Ownership {
    _directories: Vec<fs::File>,
    _lock: fs::File,
    _executable: fs::File,
}
fn own(root: &Path) -> Result<Ownership> {
    let mut dirs = Vec::new();
    for path in [
        root.to_path_buf(),
        root.join("control"),
        root.join("bin"),
        root.join("data"),
    ] {
        plain(&path, true)?;
        let handle = io(
            "broker-directory-pin",
            OpenOptions::new()
                .access_mode(FILE_READ_ATTRIBUTES)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(path),
        )?;
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { zeroed() };
        check(
            unsafe { GetFileInformationByHandle(handle.as_raw_handle(), &mut info) },
            "broker-directory-info",
        )?;
        if info.dwFileAttributes
            & (FILE_ATTRIBUTE_REPARSE_POINT
                | FILE_ATTRIBUTE_OFFLINE
                | FILE_ATTRIBUTE_RECALL_ON_OPEN
                | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS)
            != 0
            || info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        {
            return Err(invalid("broker-directory-reparse"));
        }
        dirs.push(handle);
    }
    let lock = open_file(&root.join("control/owner.lock"), true, true)?;
    let executable = open_file(&root.join("bin/kraqel-desktop.exe"), false, false)?;
    Ok(Ownership {
        _directories: dirs,
        _lock: lock,
        _executable: executable,
    })
}
fn root_hash(root: &Path) -> [u8; 32] {
    let bytes = root
        .as_os_str()
        .encode_wide()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    Sha256::digest(bytes).into()
}
fn seal(root: &Path, authority: &Authority) -> Result<()> {
    let bytes = serde_json::to_vec(authority).map_err(|_| invalid("broker-authority-encode"))?;
    let mut key = [0u8; 32];
    getrandom::fill(&mut key).map_err(|_| invalid("broker-key-random"))?;
    let mut key_file = io(
        "broker-key-create",
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("control/authority.key")),
    )?;
    io("broker-key-write", key_file.write_all(&key))?;
    io("broker-key-sync", key_file.sync_all())?;
    let protected = sign(&bytes, &key)?;
    let mut file = io(
        "broker-seal-create",
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(root.join("control/authority.mac")),
    )?;
    io("broker-seal-write", file.write_all(&protected))?;
    io("broker-seal-sync", file.sync_all())
}
fn authenticate(root: &Path) -> Result<Authority> {
    if root.join("control/retired").exists() {
        return Err(invalid("broker-retired"));
    }
    let file = open_file(&root.join("control/authority.mac"), false, false)?;
    let mut bytes = Vec::new();
    io("broker-seal-read", file.take(32769).read_to_end(&mut bytes))?;
    let key = read_key(root)?;
    let decoded = verify(&bytes, &key)?;
    let authority: Authority =
        serde_json::from_slice(decoded).map_err(|_| invalid("broker-seal-shape"))?;
    let ticket = &authority.ticket;
    if authority.version != 2
        || authority.root_sha256 != root_hash(root)
        || authority.controller_sha256 != hash(&io("broker-current-exe", std::env::current_exe())?)?
        || ticket.session_id != session(root)?
        || !ticket.valid_for(ticket.launch_id)
        || ticket.recovery
        || !ticket.automatic
        || ticket.app_sha256 != hash(&root.join("bin/kraqel-desktop.exe"))?
    {
        return Err(invalid("broker-authority-binding"));
    }
    Ok(authority)
}
fn job_name(session: Uuid) -> Vec<u16> {
    wide(format!(
        "Local\\KRAQEL.Research.Broker.{}",
        session.simple()
    ))
}

// Test actual file operations using the suspended root's exact restricted OS token.
// This is not inferred from an ACL string or a token SID alone.
pub(super) fn verify_container_denials(process: HANDLE, root: &Path) -> Result<()> {
    let mut raw = null_mut();
    check(
        unsafe { OpenProcessToken(process, TOKEN_QUERY | TOKEN_DUPLICATE, &mut raw) },
        "broker-denial-token",
    )?;
    let token = Handle(raw);
    check(
        unsafe { ImpersonateLoggedOnUser(token.0) },
        "broker-denial-impersonate",
    )?;
    let result = (|| {
        for (path, rights) in [
            (root.join("control/authority.mac"), GENERIC_READ),
            (root.join("control/authority.mac"), GENERIC_WRITE),
            (root.join("control/owner.lock"), GENERIC_READ),
            (root.join("control/authority.key"), GENERIC_READ),
            (root.join("control/authority.key"), GENERIC_WRITE),
            (root.join("bin/kraqel-desktop.exe"), GENERIC_WRITE),
        ] {
            let raw = unsafe {
                CreateFileW(
                    wide(path).as_ptr(),
                    rights,
                    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                    null(),
                    OPEN_EXISTING,
                    FILE_FLAG_OPEN_REPARSE_POINT,
                    null_mut(),
                )
            };
            if raw != INVALID_HANDLE_VALUE {
                drop(Handle(raw));
                return Err(invalid("broker-container-access-not-denied"));
            }
            if unsafe { GetLastError() } != ERROR_ACCESS_DENIED {
                return Err(fail("broker-container-denial-not-calibrated"));
            }
        }
        Ok(())
    })();
    // Continuing privileged work with the wrong token is never an error recovery path.
    if unsafe { RevertToSelf() } == 0 {
        std::process::abort();
    }
    result
}
fn drain(session: Uuid) -> Result<()> {
    let raw = unsafe { OpenJobObjectW(JOB_OBJECT_QUERY, 0, job_name(session).as_ptr()) };
    if raw.is_null() {
        if unsafe { GetLastError() } == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        return Err(fail("broker-old-job-open"));
    }
    let job = Handle(raw);
    let end = Instant::now() + Duration::from_secs(10);
    loop {
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
            "broker-old-job-accounting",
        )?;
        if info.ActiveProcesses == 0 {
            return Ok(());
        }
        if Instant::now() >= end {
            return Err(invalid("broker-old-job-active"));
        }
        thread::sleep(Duration::from_millis(20));
    }
}
fn reopen_profile(session: Uuid) -> Result<Profile> {
    let name = wide(format!("org.kraqel.probe.ui-restart-{}", session.simple()));
    let mut sid = null_mut();
    let hr = unsafe { DeriveAppContainerSidFromAppContainerName(name.as_ptr(), &mut sid) };
    if hr < 0 {
        return Err(Failure {
            stage: "broker-profile-sid",
            code: hr.into(),
        });
    }
    // No automatic deletion on an error/recoverable broker exit. Explicit retirement only.
    Ok(Profile {
        name,
        sid,
        active: false,
    })
}
pub(in super::super::super) fn seed(root: &Path, app: &Path, point: &str) -> Result<Value> {
    let point: CrashPoint =
        serde_json::from_value(json!(point)).map_err(|_| invalid("broker-crash-point"))?;
    let (session, mut profile, pin) = initialize(root, app)?;
    let root = bound_root(root)?;
    io(
        "broker-control-create",
        fs::create_dir(root.join("control")),
    )?;
    private_directory(&root.join("control"))?;
    let _ownership = own(&root)?;
    let ticket = RestartTicket {
        version: RestartTicket::VERSION,
        session_id: session,
        task_id: Uuid::new_v4(),
        launch_id: Uuid::new_v4(),
        recovery: false,
        automatic: true,
        point,
        app_sha256: pin,
    };
    seal(
        &root,
        &Authority {
            version: 2,
            root_sha256: root_hash(&root),
            controller_sha256: hash(&io("broker-current-exe", std::env::current_exe())?)?,
            ticket: ticket.clone(),
        },
    )?;
    profile.active = false;
    let name = job_name(session);
    generation_with_options(
        &root,
        &profile,
        &ticket,
        ticket.launch_id,
        GenerationOptions {
            expect: Expect::Crash,
            index: 0,
            durable_job: Some(&name),
            crash_controller: true,
        },
    )?;
    Err(invalid("broker-seed-did-not-crash"))
}

pub(in super::super::super) fn recover(root: &Path, action: &str) -> Result<Value> {
    if !matches!(action, "first" | "second" | "retire") {
        return Err(invalid("broker-recovery-action"));
    }
    let root = bound_root(root)?;
    let _ownership = own(&root)?;
    let authority = authenticate(&root)?;
    let mut ticket = authority.ticket;
    // No journal creation is permitted during takeover, even when the sealed seed exists.
    for path in [
        "data/app/tasks/tasks.sqlite3",
        "data/app/tasks/owner.sqlite3",
    ] {
        plain(&root.join(path), false)?;
    }
    drain(ticket.session_id)?;
    let mut profile = reopen_profile(ticket.session_id)?;
    if action == "retire" {
        let marker = io(
            "broker-retire-marker",
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(root.join("control/retired")),
        )?;
        io("broker-retire-sync", marker.sync_all())?;
        if !profile.remove() {
            return Err(invalid("broker-retire-profile"));
        }
        return Ok(json!({"retired":true,"profileRemoved":true,"productionAcceptance":false}));
    }
    let name = job_name(ticket.session_id);
    ticket.recovery = true;
    ticket.launch_id = Uuid::new_v4();
    let index = if action == "first" { 1 } else { 3 };
    let summary = generation_with_options(
        &root,
        &profile,
        &ticket,
        ticket.launch_id,
        GenerationOptions {
            expect: Expect::Recovery,
            index,
            durable_job: Some(&name),
            crash_controller: false,
        },
    )?
    .ok_or(invalid("broker-recovery-summary"))?;
    if !valid_summary(&summary, ticket.point, u32::from(ticket.point.publishes())) {
        return Err(invalid("broker-recovery-mismatch"));
    }
    // A stale original import request must fail before reading or journal mutation.
    ticket.recovery = false;
    ticket.launch_id = Uuid::new_v4();
    let before = hash(&root.join("data/app/tasks/tasks.sqlite3"))?;
    generation_with_options(
        &root,
        &profile,
        &ticket,
        ticket.launch_id,
        GenerationOptions {
            expect: Expect::ReuseDenied,
            index: index + 1,
            durable_job: Some(&name),
            crash_controller: false,
        },
    )?;
    if hash(&root.join("data/app/tasks/tasks.sqlite3"))? != before {
        return Err(invalid("broker-replay-mutated-journal"));
    }
    let report = json!({"researchOnly":true,"productionAcceptance":false,"passed":true,"authorityAuthenticated":true,"controllerRestarted":true,"activeJobProcessesAtEnd":0,"summary":summary,"staleImportBlocked":true,"networkMeasurement":false});
    write_json(&root.join(format!("broker-{action}.json")), &report)?;
    Ok(report)
}

fn child(arguments: &[&std::ffi::OsStr], expected: i32) -> Result<()> {
    let mut child = io(
        "broker-series-spawn",
        Command::new(io("broker-series-exe", std::env::current_exe())?)
            .args(arguments)
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(std::process::Stdio::null())
            .stderr(if expected == 1 {
                std::process::Stdio::null()
            } else {
                std::process::Stdio::inherit()
            })
            .spawn(),
    )?;
    let end = Instant::now() + Duration::from_secs(90);
    loop {
        if let Some(status) = io("broker-series-wait", child.try_wait())? {
            return if status.code() == Some(expected) {
                Ok(())
            } else {
                Err(invalid("broker-series-child-exit"))
            };
        }
        if Instant::now() >= end {
            let _ = child.kill();
            let _ = child.wait();
            return Err(invalid("broker-series-timeout"));
        }
        thread::sleep(Duration::from_millis(50));
    }
}
fn negative_recovery_checks(root: &Path) -> Result<usize> {
    let authority_path = root.join("control/authority.mac");
    let original = io("broker-negative-read-seal", fs::read(&authority_path))?;
    let journal = hash(&root.join("data/app/tasks/tasks.sqlite3"))?;
    let rejected = || {
        child(
            &[
                "ui-broker-recover".as_ref(),
                root.as_os_str(),
                "first".as_ref(),
            ],
            1,
        )?;
        let journal_path = root.join("data/app/tasks/tasks.sqlite3");
        let journal_path = if journal_path.exists() {
            journal_path
        } else {
            journal_path.with_extension("negative-backup")
        };
        if root.join("launch-01.json").exists() || hash(&journal_path)? != journal {
            return Err(invalid("broker-negative-start-or-mutation"));
        }
        Ok(())
    };
    // Deliberate test-host mutations, never a renderer capability.
    let key = read_key(root)?;
    let decoded = verify(&original, &key)?;
    for case in 0..6 {
        let mut authority: Authority =
            serde_json::from_slice(decoded).map_err(|_| invalid("broker-negative-decode"))?;
        match case {
            1 => authority.version = 1,
            2 => authority.root_sha256 = [0; 32],
            3 => authority.controller_sha256 = [0; 32],
            4 => authority.ticket.app_sha256 = [0; 32],
            5 => authority.ticket.session_id = Uuid::new_v4(),
            _ => (),
        }
        let mut sealed = sign(
            &serde_json::to_vec(&authority).map_err(|_| invalid("broker-negative-encode"))?,
            &key,
        )?;
        if case == 0 {
            let index = sealed.len() / 2;
            sealed[index] ^= 1;
        }
        io("broker-negative-write", fs::write(&authority_path, sealed))?;
        let outcome = rejected();
        io(
            "broker-negative-restore",
            fs::write(&authority_path, &original),
        )?;
        outcome?;
    }
    for relative in [
        "control/authority.mac",
        "control/authority.key",
        "data/app/tasks/tasks.sqlite3",
    ] {
        let path = root.join(relative);
        let backup = path.with_extension("negative-backup");
        io("broker-negative-hide", fs::rename(&path, &backup))?;
        let outcome = rejected();
        io("broker-negative-unhide", fs::rename(&backup, &path))?;
        outcome?;
    }
    let lock = open_file(&root.join("control/owner.lock"), true, false)?;
    rejected()?;
    drop(lock);
    for bad_key in [vec![0; 31], vec![0; 32]] {
        io(
            "broker-negative-key-write",
            fs::write(root.join("control/authority.key"), bad_key),
        )?;
        let outcome = rejected();
        io(
            "broker-negative-key-restore",
            fs::write(root.join("control/authority.key"), key),
        )?;
        outcome?;
    }
    Ok(12)
}

pub(in super::super::super) fn series(root: &Path, app: &Path) -> Result<Value> {
    if !crate::policy::valid_root(root, false)
        || !root
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("kraqel-isolation-controller-"))
    {
        return Err(invalid("broker-series-root"));
    }
    io("broker-series-create", fs::create_dir(root))?;
    let mut cases = Vec::new();
    for point in CrashPoint::ALL {
        let case_root = root.parent().unwrap().join(format!(
            "kraqel-isolation-ui-restart-{}",
            Uuid::new_v4().simple()
        ));
        let point_name = serde_json::to_value(point).unwrap();
        write_json(
            &root.join("current-case.json"),
            &json!({"point":point,"localRoot":case_root}),
        )?;
        child(
            &[
                "ui-broker-seed".as_ref(),
                case_root.as_os_str(),
                app.as_os_str(),
                point_name.as_str().unwrap().as_ref(),
            ],
            86,
        )?;
        let negative_checks = negative_recovery_checks(&case_root)?;
        for action in ["first", "second", "retire"] {
            child(
                &[
                    "ui-broker-recover".as_ref(),
                    case_root.as_os_str(),
                    action.as_ref(),
                ],
                0,
            )?;
        }
        child(
            &[
                "ui-broker-recover".as_ref(),
                case_root.as_os_str(),
                "first".as_ref(),
            ],
            1,
        )?;
        cases.push(json!({"point":point,"controllerCrashExitCode":86,"recoveryProcesses":2,"appGenerations":5,"negativeChecks":negative_checks+1,"passed":true,"profileRemoved":true,"localRoot":case_root}));
        write_json(&root.join("progress.json"), &json!({"cases":cases}))?;
    }
    let report = json!({"formatVersion":1,"researchOnly":true,"productionAcceptance":false,"passed":true,"controllerCrashes":6,"recoveryProcesses":12,"appGenerations":30,"negativeChecks":78,"cases":cases,"networkMeasurement":false});
    write_json(&root.join("result.json"), &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hmac_authenticates_and_rejects_wrong_keys_mutation_truncation_and_legacy() {
        let key = [7; 32];
        let bytes = b"synthetic recovery authority, no manuscript";
        let sealed = sign(bytes, &key).unwrap();
        assert_eq!(verify(&sealed, &key).unwrap(), bytes);
        assert!(verify(&sealed, &[8; 32]).is_err());
        for i in 0..sealed.len() {
            let mut bad = sealed.clone();
            bad[i] ^= 1;
            assert!(verify(&bad, &key).is_err());
            assert!(verify(&sealed[..i], &key).is_err());
        }
        let mut trailing = sealed.clone();
        trailing.push(0);
        assert!(verify(&trailing, &key).is_err());
        assert!(verify(b"legacy-dpapi-data", &key).is_err());
        assert!(sign(&[], &key).is_err());
        assert!(sign(&vec![0; MAX_SEAL], &key).is_err());
    }
    #[test]
    fn exclusive_owner_lock_releases_after_drop_and_rejects_hardlinks() {
        let dir = std::env::temp_dir().join(format!(
            "kraqel-broker-lock-test-{}",
            Uuid::new_v4().simple()
        ));
        fs::create_dir(&dir).unwrap();
        let path = dir.join("owner");
        let lock = open_file(&path, true, true).unwrap();
        assert!(open_file(&path, true, true).is_err());
        drop(lock);
        drop(open_file(&path, true, false).unwrap());
        fs::hard_link(&path, dir.join("alias")).unwrap();
        assert!(open_file(&path, true, false).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
}
