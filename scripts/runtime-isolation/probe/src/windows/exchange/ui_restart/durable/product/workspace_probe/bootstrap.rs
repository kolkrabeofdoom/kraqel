//! Dedicated no-network worker. No legacy probe or GUI dispatch is reachable from here.
use super::*;
use windows_sys::Win32::System::Pipes::{CreatePipe, GetNamedPipeServerProcessId, PeekNamedPipe};

const BOOT: &[u8] = b"KRAQEL-P2B-BOOT\0\x01";
const RECEIPT: &[u8] = b"KRAQEL-P2B-RECEIPT\0\x01";
const LIMIT: usize = 8192;
#[derive(Clone, Copy)]
pub(super) enum Mode {
    Normal,
    Unassigned,
    BadMac,
    CrashLive,
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Bootstrap {
    version: u16,
    purpose: String,
    parent_pid: u32,
    launch: LaunchIdentity,
    execution: Execution,
    root: PathBuf,
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Receipt {
    version: u16,
    launch: LaunchIdentity,
    nonce: [u8; 32],
    worker_sha256: [u8; 32],
    fixtures_verified: bool,
}
pub(super) struct Outcome {
    pub receipt_sha256: [u8; 32],
    pub report: Value,
}

fn encode<T: Serialize>(value: &T, key: &[u8; 32], domain: &[u8]) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec(value).map_err(|_| invalid("probe-frame-json"))?;
    if bytes.is_empty() || bytes.len() > LIMIT {
        return Err(invalid("probe-frame-size"));
    }
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| invalid("probe-frame-key"))?;
    mac.update(domain);
    mac.update(&bytes);
    let mut wire = Vec::with_capacity(36 + bytes.len());
    wire.extend((bytes.len() as u32).to_le_bytes());
    wire.extend(mac.finalize().into_bytes());
    wire.extend(bytes);
    Ok(wire)
}
fn decode<T: serde::de::DeserializeOwned>(wire: &[u8], key: &[u8; 32], domain: &[u8]) -> Result<T> {
    if wire.len() < 37 || wire.len() > LIMIT + 36 {
        return Err(invalid("probe-frame-size"));
    }
    let len = u32::from_le_bytes(wire[..4].try_into().unwrap()) as usize;
    if len == 0 || len > LIMIT || wire.len() != len + 36 {
        return Err(invalid("probe-frame-length"));
    }
    let mut mac = HmacSha256::new_from_slice(key).map_err(|_| invalid("probe-frame-key"))?;
    mac.update(domain);
    mac.update(&wire[36..]);
    mac.verify_slice(&wire[4..36])
        .map_err(|_| invalid("probe-frame-mac"))?;
    serde_json::from_slice(&wire[36..]).map_err(|_| invalid("probe-frame-json"))
}
fn pipe() -> Result<(fs::File, fs::File)> {
    let (mut read, mut write) = (null_mut(), null_mut());
    check(
        unsafe { CreatePipe(&mut read, &mut write, null(), 16384) },
        "probe-pipe",
    )?;
    Ok(unsafe {
        (
            fs::File::from_raw_handle(read),
            fs::File::from_raw_handle(write),
        )
    })
}
fn receive(file: &mut fs::File, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    io(
        "probe-pipe-read",
        file.take((limit + 1) as u64).read_to_end(&mut bytes),
    )?;
    if bytes.len() > limit {
        return Err(invalid("probe-pipe-limit"));
    }
    Ok(bytes)
}
fn facts_valid(facts: &Value) -> bool {
    facts["isAppContainer"] == true
        && facts["capabilityCount"] == 0
        && facts["expectedIdentity"] == true
}

// A descendant could retain the writer after root exit. Never block on EOF from the container.
fn receive_before(file: &mut fs::File, deadline: Instant) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    loop {
        if Instant::now() >= deadline {
            return Err(invalid("probe-bootstrap-expired"));
        }
        let mut available = 0;
        if unsafe {
            PeekNamedPipe(
                file.as_raw_handle(),
                null_mut(),
                0,
                null_mut(),
                &mut available,
                null_mut(),
            )
        } == 0
        {
            if unsafe { GetLastError() } == ERROR_BROKEN_PIPE {
                return Ok(bytes);
            }
            return Err(fail("probe-receipt-peek"));
        }
        if available == 0 {
            thread::sleep(Duration::from_millis(10));
            continue;
        }
        if available as usize > LIMIT + 36 - bytes.len() {
            return Err(invalid("probe-pipe-limit"));
        }
        let mut block = [0; 4096];
        let count = block.len().min(available as usize);
        let count = io("probe-receipt-read", file.read(&mut block[..count]))?;
        bytes.extend_from_slice(&block[..count]);
    }
}

pub(super) fn run(
    store: &mut Store,
    execution: &Execution,
    profile: &Profile,
    launch: LaunchIdentity,
    mode: Mode,
) -> Result<Result<Outcome>> {
    let name = store.job_name();
    let job = Handle(unsafe { CreateJobObjectW(null(), name.as_ptr()) });
    if job.0.is_null() {
        return Err(fail("probe-job"));
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        return Err(invalid("probe-job-collision"));
    }
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    limits.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    limits.BasicLimitInformation.ActiveProcessLimit = 1;
    check(
        unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        },
        "probe-job-limit",
    )?;
    let outcome = run_with_job(store, execution, profile, launch, mode, &job);
    // Always stop/drain while the kernel object is still held, including failed bootstraps.
    check(unsafe { TerminateJobObject(job.0, 3) }, "probe-job-stop")?;
    let deadline = Instant::now() + Duration::from_secs(5);
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
            "probe-job-drain-query",
        )?;
        if info.ActiveProcesses == 0 {
            write_json(
                &root(execution).join("drained.json"),
                &json!({"activeProcesses":0,"totalProcesses":info.TotalProcesses,"jobHeldDuringQuery":true}),
            )?;
            break;
        }
        if Instant::now() >= deadline {
            return Err(invalid("probe-job-drain-timeout"));
        }
        thread::sleep(Duration::from_millis(20));
    }
    store.idle()?;
    Ok(outcome)
}

fn run_with_job(
    store: &mut Store,
    execution: &Execution,
    profile: &Profile,
    launch: LaunchIdentity,
    mode: Mode,
    job: &Handle,
) -> Result<Outcome> {
    let started = Instant::now();
    let path = root(execution);
    let data = path.join("data");
    // Add access exclusively to this newly created synthetic worker root. Never to Store.
    for p in [&path, &path.join("bin"), &data] {
        grant(p, profile.sid, FILE_GENERIC_READ | FILE_GENERIC_EXECUTE)?;
    }
    let exe = path.join("bin/worker.exe");
    let (_dirs, mut executable, sha) = binary_pin(&exe)?;
    package::verify_probe(&mut executable)?;
    if sha != execution.worker_sha256 {
        return Err(invalid("probe-staged-package"));
    }
    let boot = Bootstrap {
        version: 1,
        purpose: "synthetic-workspace-probe".into(),
        parent_pid: unsafe { GetCurrentProcessId() },
        launch,
        execution: execution.clone(),
        root: path.clone(),
    };
    let mut key = Zeroizing::new([0; 32]);
    getrandom::fill(key.as_mut()).map_err(|_| invalid("probe-key"))?;
    let mut wire = encode(&boot, &key, BOOT)?;
    if matches!(mode, Mode::BadMac) {
        wire[4] ^= 1;
    }
    let (startup, mut send) = pipe()?;
    io("probe-bootstrap-write", send.write_all(&wire))?;
    drop(send);
    let (secret, mut send) = pipe()?;
    io("probe-secret-write", send.write_all(key.as_ref()))?;
    drop(send);
    let (mut events, event_writer) = pipe()?;
    let handles = [
        startup.as_raw_handle(),
        secret.as_raw_handle(),
        event_writer.as_raw_handle(),
    ];
    for &h in &handles {
        check(
            unsafe { SetHandleInformation(h, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) },
            "probe-inherit",
        )?;
    }
    let caps = SECURITY_CAPABILITIES {
        AppContainerSid: profile.sid,
        Capabilities: null_mut(),
        CapabilityCount: 0,
        Reserved: 0,
    };
    let name = store.job_name();
    let jobs = [job.0];
    let attrs = Attributes::with_handles_and_jobs(
        &caps,
        &handles,
        if matches!(mode, Mode::Unassigned) {
            &[]
        } else {
            &jobs
        },
    )?;
    let mut startup_info: STARTUPINFOEXW = unsafe { zeroed() };
    startup_info.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    startup_info.lpAttributeList = attrs.ptr;
    let mut info: PROCESS_INFORMATION = unsafe { zeroed() };
    let mut command = wide(format!(
        "\"{}\" workspace-probe-worker {} {} {} {}",
        exe.display(),
        handles[0] as usize,
        handles[1] as usize,
        handles[2] as usize,
        Uuid::from_bytes(launch.id)
    ));
    let env = super::super::super::super::environment(&data);
    let ok = unsafe {
        CreateProcessW(
            wide(&exe).as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT
                | CREATE_UNICODE_ENVIRONMENT
                | CREATE_SUSPENDED
                | DETACHED_PROCESS,
            env.as_ptr().cast(),
            wide(&data).as_ptr(),
            &startup_info.StartupInfo,
            &mut info,
        )
    };
    let error = (ok == 0).then(|| fail("probe-create-process"));
    drop(attrs);
    let worker = (ok != 0).then(|| Worker(Handle(info.hProcess), Handle(info.hThread)));
    for &h in &handles {
        check(
            unsafe { SetHandleInformation(h, HANDLE_FLAG_INHERIT, 0) },
            "probe-uninherit",
        )?;
    }
    drop(startup);
    drop(secret);
    drop(event_writer);
    if let Some(error) = error {
        return Err(error);
    }
    let worker = worker.ok_or(invalid("probe-worker"))?;
    let identity = lifetime::capture_identity(worker.0.0)?;
    write_json(
        &path.join("spawn.json"),
        &json!({"pid":info.dwProcessId,"workerIdentity":identity,"suspendedAtRecord":true}),
    )?;
    let mut assigned = 0;
    check(
        unsafe { IsProcessInJob(worker.0.0, job.0, &mut assigned) },
        "probe-check-job",
    )?;
    if assigned == 0 {
        return Err(invalid("probe-not-in-job-before-resume"));
    }
    let facts = token_facts(worker.0.0, Some(profile.sid))?;
    if !facts_valid(&facts) {
        return Err(invalid("probe-token-before-resume"));
    }
    verify_denials(worker.0.0, &store.root)?;
    starter::verify_start_lease(&store.root, &name)?;
    lifetime::verify_single_process_job(job.0)?;
    write_json(
        &path.join("parent.json"),
        &json!({"pid":info.dwProcessId,"workerIdentity":identity,"singleProcessLimit":1,"token":facts,"exactJob":true,"privateDenials":6,"leaseHeld":true,"launch":launch}),
    )?;
    crash::hit(crash::Point::BeforeWorkerBound);
    if store.bind_suspended_worker(launch, worker.0.0)? != identity {
        return Err(invalid("probe-worker-identity-changed"));
    }
    crash::hit(crash::Point::WorkerBound);
    if started.elapsed() > Duration::from_secs(30) {
        return Err(invalid("probe-bootstrap-expired"));
    }
    if unsafe { ResumeThread(worker.1.0) } == u32::MAX {
        return Err(fail("probe-resume"));
    }
    if matches!(mode, Mode::CrashLive) {
        let wire = receive_before(&mut events, started + Duration::from_secs(30))?;
        let receipt: Receipt = decode(&wire, &key, RECEIPT)?;
        if receipt
            != (Receipt {
                version: 1,
                launch,
                nonce: execution.nonce,
                worker_sha256: execution.worker_sha256,
                fixtures_verified: true,
            })
            || unsafe { WaitForSingleObject(worker.0.0, 0) } != WAIT_TIMEOUT
        {
            return Err(invalid("probe-live-crash-not-running"));
        }
        write_json(
            &path.join("worker-live.json"),
            &json!({"authenticatedReceiptBeforeCrash":true,"workerWasAlive":true,"workerIdentity":identity}),
        )?;
        crash::hit(crash::Point::WorkerLive);
        return Err(invalid("probe-live-crash-not-armed"));
    }
    if unsafe { WaitForSingleObject(worker.0.0, 10000) } != WAIT_OBJECT_0 {
        return Err(invalid("probe-worker-timeout"));
    }
    let mut exit = 0;
    check(
        unsafe { GetExitCodeProcess(worker.0.0, &mut exit) },
        "probe-worker-exit",
    )?;
    write_json(&path.join("exit.json"), &json!({"exit":exit}))?;
    if exit != 0 {
        return Err(invalid("probe-worker-rejected-bootstrap"));
    }
    let wire = receive_before(&mut events, started + Duration::from_secs(30))?;
    let receipt: Receipt = decode(&wire, &key, RECEIPT)?;
    if receipt
        != (Receipt {
            version: 1,
            launch,
            nonce: execution.nonce,
            worker_sha256: execution.worker_sha256,
            fixtures_verified: true,
        })
    {
        return Err(invalid("probe-receipt-binding"));
    }
    drop(worker);
    Ok(Outcome {
        receipt_sha256: Sha256::digest(&wire).into(),
        report: json!({"authenticatedReceipt":true,"tokenAndExactJobBeforeResume":true,"privateDenials":6,"controlLeaseHeld":true,"workerExit":0,"fixturesVerified":true,"uiStarted":false,"importTaskCreated":false}),
    })
}

pub(in crate::windows) fn worker(args: &[OsString]) -> Result<()> {
    if args.len() != 4 {
        return Err(invalid("probe-worker-arguments"));
    }
    // Check restriction before trusting inherited channel numbers or reading their content.
    let facts = token_facts(unsafe { GetCurrentProcess() }, None)?;
    if facts["isAppContainer"] != true || facts["capabilityCount"] != 0 {
        return Err(invalid("probe-worker-unrestricted"));
    }
    let mut raw = Vec::new();
    for arg in &args[..3] {
        let h = arg
            .to_str()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|v| *v > 0 && *v != usize::MAX)
            .ok_or(invalid("probe-worker-handle"))?;
        if raw.contains(&h) {
            return Err(invalid("probe-worker-handle-alias"));
        }
        raw.push(h);
    }
    let expected = Uuid::parse_str(args[3].to_str().ok_or(invalid("probe-worker-launch"))?)
        .map_err(|_| invalid("probe-worker-launch"))?;
    let mut boot_file = unsafe { fs::File::from_raw_handle(raw[0] as HANDLE) };
    let mut key_file = unsafe { fs::File::from_raw_handle(raw[1] as HANDLE) };
    let mut receipt_file = unsafe { fs::File::from_raw_handle(raw[2] as HANDLE) };
    let mut peers = [0; 2];
    check(
        unsafe { GetNamedPipeServerProcessId(boot_file.as_raw_handle(), &mut peers[0]) },
        "probe-worker-peer",
    )?;
    check(
        unsafe { GetNamedPipeServerProcessId(key_file.as_raw_handle(), &mut peers[1]) },
        "probe-worker-key-peer",
    )?;
    let bytes = Zeroizing::new(receive(&mut key_file, 32)?);
    let key: &[u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| invalid("probe-worker-key-size"))?;
    let boot: Bootstrap = decode(&receive(&mut boot_file, LIMIT + 36)?, key, BOOT)?;
    if boot.version != 1
        || boot.purpose != "synthetic-workspace-probe"
        || boot.parent_pid == 0
        || peers != [boot.parent_pid; 2]
        || boot.parent_pid == unsafe { GetCurrentProcessId() }
        || boot.launch.id != *expected.as_bytes()
        || boot.launch.generation == 0
        || boot.execution.version != 3
        || boot.execution.live_hold_ms > 2000
        || boot.execution.probe_id.is_nil()
        || boot.execution.workspace_id == [0; 16]
        || boot.execution.nonce == [0; 32]
    {
        return Err(invalid("probe-worker-bootstrap-binding"));
    }
    let profile = profile(&boot.execution)?;
    if !facts_valid(&token_facts(
        unsafe { GetCurrentProcess() },
        Some(profile.sid),
    )?) {
        return Err(invalid("probe-worker-profile"));
    }
    let mut in_job = 0;
    check(
        unsafe { IsProcessInJob(GetCurrentProcess(), null_mut(), &mut in_job) },
        "probe-worker-job",
    )?;
    if in_job == 0 {
        return Err(invalid("probe-worker-no-job"));
    }
    // Root is canonical and syntactic; never traverse/pin host ancestors inside the container.
    let path = local_path(&boot.root)?;
    if root_hash(&io("probe-worker-root", path.canonicalize())?) != boot.execution.root_sha256
        || path.file_name()
            != Some(std::ffi::OsStr::new(&format!(
                "kraqel-isolation-boot-{}",
                boot.execution.probe_id.simple()
            )))
    {
        return Err(invalid("probe-worker-root-binding"));
    }
    let exe = io("probe-worker-exe", std::env::current_exe())?;
    if local_path(&exe)? != path.join("bin/worker.exe")
        || hash(&exe)? != boot.execution.worker_sha256
    {
        return Err(invalid("probe-worker-package"));
    }
    for (name, expected) in [
        ("policy.json", boot.execution.policy_sha256),
        ("sentinel.txt", boot.execution.sentinel_sha256),
    ] {
        if hash(&path.join("data").join(name))? != expected {
            return Err(invalid("probe-worker-fixture"));
        }
    }
    let receipt = Receipt {
        version: 1,
        launch: boot.launch,
        nonce: boot.execution.nonce,
        worker_sha256: boot.execution.worker_sha256,
        fixtures_verified: true,
    };
    io(
        "probe-worker-receipt",
        receipt_file.write_all(&encode(&receipt, key, RECEIPT)?),
    )?;
    drop(receipt_file);
    if boot.execution.live_hold_ms > 0 {
        thread::sleep(Duration::from_millis(u64::from(
            boot.execution.live_hold_ms,
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frame_domains_tampering_trailing_bytes_and_unknown_fields_are_rejected() {
        let value = Receipt {
            version: 1,
            launch: LaunchIdentity {
                id: [1; 16],
                generation: 1,
            },
            nonce: [2; 32],
            worker_sha256: [3; 32],
            fixtures_verified: true,
        };
        let key = [7; 32];
        let wire = encode(&value, &key, RECEIPT).unwrap();
        let (mut reader, mut writer) = pipe().unwrap();
        assert!(receive_before(&mut reader, Instant::now()).is_err());
        writer.write_all(&wire).unwrap();
        drop(writer);
        assert_eq!(
            receive_before(&mut reader, Instant::now() + Duration::from_secs(1)).unwrap(),
            wire
        );
        assert_eq!(decode::<Receipt>(&wire, &key, RECEIPT).unwrap(), value);
        assert!(decode::<Receipt>(&wire, &key, BOOT).is_err());
        assert!(decode::<Receipt>(&wire, &[8; 32], RECEIPT).is_err());
        let mut bad = wire.clone();
        bad[36] ^= 1;
        assert!(decode::<Receipt>(&bad, &key, RECEIPT).is_err());
        let mut tail = wire;
        tail.push(0);
        assert!(decode::<Receipt>(&tail, &key, RECEIPT).is_err());
        let mut unknown = serde_json::to_value(&value).unwrap();
        unknown["importTask"] = json!(true);
        assert!(
            decode::<Receipt>(&encode(&unknown, &key, RECEIPT).unwrap(), &key, RECEIPT).is_err()
        );
        assert!(decode::<Receipt>(&[0; 36], &key, RECEIPT).is_err());
    }
}
