//! Same live controller, exact profile/data/executable; each generation gets fresh kernel pipes.
use super::*;
use kraqel_isolation_transport::{
    CrashPoint, RecoverySummary, RestartEvent, RestartEventKind, RestartTicket, write_frame,
};
use sha2::{Digest as _, Sha256};
use uuid::Uuid;
use windows_sys::Win32::System::Pipes::{CreatePipe, PeekNamedPipe};
mod durable;
mod live;
pub(in super::super) use durable::product_storage;
pub(in super::super) use durable::{combined_series, next_live, seed_live};
pub(in super::super) use durable::{
    product_crash_child, product_crash_series, product_session_child, product_session_series,
    product_start_series, product_transition_child, product_transition_series,
};
pub(in super::super) use durable::{
    recover as recover_broker, seed as seed_broker, series as controller_crash_series,
};

fn pipe() -> Result<(fs::File, fs::File)> {
    let mut read = null_mut();
    let mut write = null_mut();
    check(
        unsafe { CreatePipe(&mut read, &mut write, null(), 4096) },
        "restart-create-pipe",
    )?;
    Ok(unsafe {
        (
            fs::File::from_raw_handle(read),
            fs::File::from_raw_handle(write),
        )
    })
}
fn hash(path: &Path) -> Result<[u8; 32]> {
    let mut file = io("restart-hash-open", fs::File::open(path))?;
    let mut hash = Sha256::new();
    let mut block = [0u8; 16384];
    loop {
        let n = io("restart-hash-read", file.read(&mut block))?;
        if n == 0 {
            break;
        }
        hash.update(&block[..n]);
    }
    Ok(hash.finalize().into())
}
fn environment(data: &Path) -> Vec<u16> {
    let system = std::env::var_os("SystemRoot").unwrap_or("C:\\Windows".into());
    let mut environment = Vec::new();
    for (key, value) in [
        ("APPDATA", data.to_path_buf()),
        ("LOCALAPPDATA", data.to_path_buf()),
        ("PATH", PathBuf::from(&system).join("System32")),
        ("SystemRoot", PathBuf::from(&system)),
        ("TEMP", data.to_path_buf()),
        ("TMP", data.to_path_buf()),
        ("USERPROFILE", data.to_path_buf()),
        ("WINDIR", PathBuf::from(&system)),
    ] {
        environment.extend(wide(format!("{key}={}", value.display())));
    }
    environment.push(0);
    environment
}
#[derive(Clone, Copy)]
enum Expect {
    Crash,
    Recovery,
    Reject,
    ReuseDenied,
    ManualRecovery,
    Failed,
    WindowClose,
}
fn valid_summary(value: &RecoverySummary, point: CrashPoint, projects: u32) -> bool {
    value.published == point.publishes()
        && value.project_count == projects
        && value.document_count == if point.publishes() { 2 } else { 0 }
        && value.staging_empty
        && value.replay_blocked
}

fn generation(
    root: &Path,
    profile: &Profile,
    ticket: &RestartTicket,
    argument: Uuid,
    expect: Expect,
    index: usize,
) -> Result<Option<RecoverySummary>> {
    generation_with_options(
        root,
        profile,
        ticket,
        argument,
        GenerationOptions {
            expect,
            index,
            durable_job: None,
            crash_controller: false,
            live_mode: None,
            product_root: None,
        },
    )
}

struct GenerationOptions<'a> {
    expect: Expect,
    index: usize,
    durable_job: Option<&'a [u16]>,
    crash_controller: bool,
    live_mode: Option<live::LiveMode>,
    product_root: Option<&'a Path>,
}

fn generation_with_options(
    root: &Path,
    profile: &Profile,
    ticket: &RestartTicket,
    argument: Uuid,
    options: GenerationOptions<'_>,
) -> Result<Option<RecoverySummary>> {
    let GenerationOptions {
        expect,
        index,
        durable_job,
        crash_controller,
        live_mode,
        product_root,
    } = options;
    let executable = root.join("bin/kraqel-desktop.exe");
    // Negative hash tickets still start only the original pinned executable.
    let (startup, mut send) = pipe()?;
    write_frame(&mut send, ticket).map_err(|_| invalid("restart-ticket-frame"))?;
    drop(send);
    let (mut events, event_writer) = pipe()?;
    let mut bridge = if ticket.live_channel() {
        Some(live::Bridge::new()?)
    } else {
        None
    };
    let mut handles = vec![startup.as_raw_handle(), event_writer.as_raw_handle()];
    if let Some(live) = &bridge {
        handles.push(live.request_child.as_ref().unwrap().as_raw_handle());
        handles.push(live.response_child.as_ref().unwrap().as_raw_handle());
    }
    for &h in &handles {
        check(
            unsafe { SetHandleInformation(h, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) },
            "restart-inherit",
        )?;
    }
    let caps = SECURITY_CAPABILITIES {
        AppContainerSid: profile.sid,
        Capabilities: null_mut(),
        CapabilityCount: 0,
        Reserved: 0,
    };
    let mut command = wide(format!(
        "\"{}\" ui-restart {} {} {argument}",
        executable.display(),
        handles[0] as usize,
        handles[1] as usize
    ));
    if handles.len() == 4 {
        command.pop();
        command.extend(wide(format!(
            " {} {}",
            handles[2] as usize, handles[3] as usize
        )));
    }
    let data = root.join("data");
    let env = environment(&data);
    // A durable job name is native authenticated state, never renderer input. Reusing a live
    // object would silently adopt an older process tree, so even an empty collision fails closed.
    let job =
        Handle(unsafe { CreateJobObjectW(null(), durable_job.map_or(null(), |n| n.as_ptr())) });
    if job.0.is_null() {
        return Err(fail("restart-create-job"));
    }
    if durable_job.is_some() && unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        return Err(invalid("broker-job-already-exists"));
    }
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    check(
        unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        },
        "restart-job-limits",
    )?;
    let job_handles = [job.0];
    let jobs = if durable_job.is_some() {
        &job_handles[..]
    } else {
        &[]
    };
    let attrs = Attributes::with_handles_and_jobs(&caps, &handles, jobs)?;
    let mut startup_info: STARTUPINFOEXW = unsafe { zeroed() };
    startup_info.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    startup_info.lpAttributeList = attrs.ptr;
    let mut info: PROCESS_INFORMATION = unsafe { zeroed() };
    let created = unsafe {
        CreateProcessW(
            wide(&executable).as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            1,
            EXTENDED_STARTUPINFO_PRESENT
                | CREATE_UNICODE_ENVIRONMENT
                | CREATE_SUSPENDED
                | CREATE_NO_WINDOW,
            env.as_ptr().cast(),
            wide(&data).as_ptr(),
            &startup_info.StartupInfo,
            &mut info,
        )
    };
    let error = (created == 0).then(|| fail("restart-create-process"));
    drop(attrs);
    let worker = (created != 0).then(|| Worker(Handle(info.hProcess), Handle(info.hThread)));
    for &h in &handles {
        check(
            unsafe { SetHandleInformation(h, HANDLE_FLAG_INHERIT, 0) },
            "restart-reset-inheritance",
        )?;
    }
    drop(startup);
    drop(event_writer);
    if let Some(live) = &mut bridge {
        drop(live.request_child.take());
        drop(live.response_child.take());
    }
    if let Some(error) = error {
        return Err(error);
    }
    let worker = worker.ok_or(invalid("restart-worker"))?;
    if durable_job.is_none() {
        check(
            unsafe { AssignProcessToJobObject(job.0, worker.0.0) },
            "restart-assign-job",
        )?;
    }
    let mut in_job = 0;
    check(
        unsafe { IsProcessInJob(worker.0.0, job.0, &mut in_job) },
        "restart-check-job",
    )?;
    if in_job == 0 {
        return Err(invalid("restart-not-in-job"));
    }
    let token = token_facts(worker.0.0, Some(profile.sid))?;
    if token["isAppContainer"] != true
        || token["capabilityCount"] != 0
        || token["expectedIdentity"] != true
    {
        return Err(invalid("restart-token"));
    }
    let product_start = durable_job.is_some() && product_root.is_some();
    if durable_job.is_some() && !product_start {
        durable::verify_container_denials(worker.0.0, root)?;
    }
    if let Some(product_root) = product_root {
        durable::verify_product_denials(worker.0.0, product_root)?;
        if let Some(name) = durable_job {
            durable::verify_start_lease(product_root, name)?;
        }
    }
    write_json(
        &root.join(format!("launch-{index:02}.json")),
        &json!({"pid":info.dwProcessId,"token":token,"recovery":ticket.recovery,"point":ticket.point,"privateBrokerFilesDenied":durable_job.is_some() && !product_start,"privateProductFilesDenied":product_root.is_some(),"productControlLeaseVerified":product_start}),
    )?;
    if unsafe { ResumeThread(worker.1.0) } == u32::MAX {
        return Err(fail("restart-resume"));
    }
    let expected: &[RestartEventKind] = match expect {
        Expect::Crash => &[RestartEventKind::Renderer, RestartEventKind::Checkpoint],
        Expect::Recovery => &[
            RestartEventKind::Recovered,
            RestartEventKind::Renderer,
            RestartEventKind::Replay,
            RestartEventKind::Replay,
        ],
        Expect::Reject => &[],
        Expect::ReuseDenied => &[RestartEventKind::Renderer, RestartEventKind::ReuseRejected],
        Expect::ManualRecovery => &[
            RestartEventKind::Recovered,
            RestartEventKind::Renderer,
            RestartEventKind::Replay,
        ],
        Expect::Failed => &[RestartEventKind::Renderer, RestartEventKind::Finished],
        Expect::WindowClose => &[RestartEventKind::Renderer, RestartEventKind::Finished],
    };
    let start = Instant::now();
    let mut received = 0;
    let mut buffer = Vec::new();
    let mut summary = None;
    let mut window_exit_seen = false;
    loop {
        if let Some(live) = &mut bridge {
            live.service(
                root,
                ticket,
                live_mode.ok_or(invalid("combined-missing-live-policy"))?,
                index,
            )?;
        }
        let mut available = 0;
        let peek = unsafe {
            PeekNamedPipe(
                events.as_raw_handle(),
                null_mut(),
                0,
                null_mut(),
                &mut available,
                null_mut(),
            )
        };
        if peek == 0
            && std::io::Error::last_os_error().raw_os_error() != Some(ERROR_BROKEN_PIPE as i32)
        {
            return Err(fail("restart-peek"));
        }
        if available > 0 {
            if buffer.len() + available as usize > 4096 {
                return Err(invalid("restart-event-budget"));
            }
            let old = buffer.len();
            buffer.resize(old + available as usize, 0);
            io("restart-event-read", events.read_exact(&mut buffer[old..]))?;
            while buffer.len() >= 4 {
                let size = u32::from_le_bytes(buffer[..4].try_into().unwrap()) as usize;
                if size == 0 || size > 2048 {
                    return Err(invalid("restart-event-frame"));
                }
                if buffer.len() < 4 + size {
                    break;
                }
                let event: RestartEvent = serde_json::from_slice(&buffer[4..4 + size])
                    .map_err(|_| invalid("restart-event-shape"))?;
                buffer.drain(..4 + size);
                if event.launch_id != ticket.launch_id
                    || !expected_event(expect, expected, received, event.kind)
                    || (event.kind == RestartEventKind::Recovered) != event.summary.is_some()
                {
                    return Err(invalid("restart-event-binding-or-order"));
                }
                if let Some(value) = event.summary {
                    summary = Some(value);
                }
                received += 1;
            }
        }
        let exited = unsafe { WaitForSingleObject(worker.0.0, 20) } == WAIT_OBJECT_0;
        if matches!(expect, Expect::WindowClose) && exited && !window_exit_seen {
            // Drain once more after observing exit: Finished may arrive during WaitForSingleObject.
            window_exit_seen = true;
            continue;
        }
        if matches!(expect, Expect::ManualRecovery | Expect::WindowClose) && exited {
            if received < expected.len() || !buffer.is_empty() {
                return Err(invalid("manual-recovery-closed-before-replay"));
            }
            break;
        }
        if matches!(expect, Expect::Reject) && exited {
            let mut code = 0;
            check(
                unsafe { GetExitCodeProcess(worker.0.0, &mut code) },
                "restart-rejected-exit",
            )?;
            if code != 2 || received != 0 || !buffer.is_empty() {
                return Err(invalid("restart-not-rejected"));
            }
            break;
        }
        if !matches!(
            expect,
            Expect::Reject | Expect::ManualRecovery | Expect::WindowClose
        ) && received == expected.len()
            && buffer.is_empty()
        {
            break;
        }
        let limit = if matches!(expect, Expect::ManualRecovery) || !ticket.automatic {
            600
        } else {
            45
        };
        if exited || root.join("STOP").exists() || start.elapsed() > Duration::from_secs(limit) {
            return Err(invalid("restart-checkpoint-or-recovery-missing"));
        }
    }
    if matches!(expect, Expect::WindowClose) {
        let mode = live_mode.ok_or(invalid("combined-close-policy"))?;
        let report = bridge
            .as_mut()
            .ok_or(invalid("combined-close-bridge"))?
            .finish_window_close(mode)?;
        write_json(&root.join("window-close-transport.json"), &report)?;
    }
    if matches!(expect, Expect::ReuseDenied) && bridge.as_ref().is_some_and(live::Bridge::selected)
    {
        return Err(invalid("combined-stale-request-selected"));
    }
    if crash_controller {
        if live_mode == Some(live::LiveMode::Partial) {
            let deadline = Instant::now() + Duration::from_secs(5);
            let sender = root.join(format!("control/live-{index:02}.json"));
            while !sender.exists() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
            let sent: Value =
                serde_json::from_slice(&io("combined-partial-sent", fs::read(sender))?)
                    .map_err(|_| invalid("combined-partial-sent-shape"))?;
            let received: Value = serde_json::from_slice(&io(
                "combined-partial-received",
                fs::read(root.join("data/transfer-received.json")),
            )?)
            .map_err(|_| invalid("combined-partial-received-shape"))?;
            if sent["bytesSent"] != 1024
                || received["payloadBytesReceived"] != 1024
                || sent["partial"] != true
            {
                return Err(invalid("combined-partial-not-proved"));
            }
        }
        // Actual controller death: no destructors, no explicit worker/job termination first.
        // The only job handle is non-inheritable; Windows must kill the entire contained tree.
        check(
            unsafe { TerminateProcess(GetCurrentProcess(), 86) },
            "broker-crash-self",
        )?;
        return Err(invalid("broker-crash-returned"));
    }
    // Includes WebView descendants; wait for zero before reusing the same identity/data root.
    check(
        unsafe { TerminateJobObject(job.0, 77) },
        "restart-terminate-job",
    )?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let mut accounting: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
        check(
            unsafe {
                QueryInformationJobObject(
                    job.0,
                    JobObjectBasicAccountingInformation,
                    (&mut accounting as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                    null_mut(),
                )
            },
            "restart-job-accounting",
        )?;
        if accounting.ActiveProcesses == 0 {
            break;
        }
        if Instant::now() >= deadline {
            return Err(invalid("restart-job-not-empty"));
        }
        thread::sleep(Duration::from_millis(20));
    }
    let mut code = 0;
    check(
        unsafe { GetExitCodeProcess(worker.0.0, &mut code) },
        "restart-exit-code",
    )?;
    if matches!(expect, Expect::ManualRecovery | Expect::WindowClose) && code != 0 {
        return Err(invalid("manual-recovery-unclean-exit"));
    }
    if !matches!(
        expect,
        Expect::Reject | Expect::ManualRecovery | Expect::WindowClose
    ) && code != 77
    {
        return Err(invalid("restart-wrong-kill-exit"));
    }
    write_json(
        &root.join(format!("generation-{index:02}.json")),
        &json!({"processExitCode":code,"activeJobProcessesAtEnd":0,"receivedEvents":received,"recovery":ticket.recovery,"summary":summary}),
    )?;
    Ok(summary)
}

fn initialize(root: &Path, app: &Path) -> Result<(Uuid, Profile, [u8; 32])> {
    if !crate::policy::valid_root(root, false) {
        return Err(invalid("restart-needs-fresh-root"));
    }
    let session = root
        .file_name()
        .and_then(|s| s.to_str())
        .and_then(|s| s.strip_prefix("kraqel-isolation-ui-restart-"))
        .and_then(|s| Uuid::parse_str(s).ok())
        .filter(|u| !u.is_nil())
        .ok_or(invalid("restart-root-binding"))?;
    if root.file_name()
        != Some(std::ffi::OsStr::new(&format!(
            "kraqel-isolation-ui-restart-{}",
            session.simple()
        )))
    {
        return Err(invalid("restart-root-canonical-name"));
    }
    initialize_bound(root, app, session)
}
// Caller must authenticate/pin a separately derived private product session parent first.
fn initialize_bound(root: &Path, app: &Path, session: Uuid) -> Result<(Uuid, Profile, [u8; 32])> {
    io("restart-create-root", fs::create_dir(root))?;
    initialize_created(root, app, session)
}
// Product schema 4 creates/pins the private root and authenticates its preparation first.
fn initialize_created(root: &Path, app: &Path, session: Uuid) -> Result<(Uuid, Profile, [u8; 32])> {
    let profile = Profile::create(&format!("ui-restart-{}", session.simple()))?;
    grant(root, profile.sid, FILE_GENERIC_READ | FILE_GENERIC_EXECUTE)?;
    let data = root.join("data");
    let bin = root.join("bin");
    io("restart-create-data", fs::create_dir(&data))?;
    io("restart-create-bin", fs::create_dir(&bin))?;
    grant(
        &data,
        profile.sid,
        FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE | DELETE | FILE_DELETE_CHILD,
    )?;
    low_label(&data)?;
    let executable = bin.join("kraqel-desktop.exe");
    io("restart-stage-app", fs::copy(app, &executable))?;
    let pin = hash(&executable)?;
    Ok((session, profile, pin))
}
fn expected_event(
    expect: Expect,
    expected: &[RestartEventKind],
    received: usize,
    kind: RestartEventKind,
) -> bool {
    if matches!(expect, Expect::ManualRecovery) && received >= 2 {
        received < 16 && kind == RestartEventKind::Replay
    } else {
        expected.get(received) == Some(&kind)
    }
}
pub(in super::super) fn launch(root: &Path, app: &Path) -> Result<Value> {
    let (session, mut profile, pin) = initialize(root, app)?;
    let data = root.join("data");
    let executable = root.join("bin/kraqel-desktop.exe");
    let mut cases = Vec::new();
    let mut projects = 0;
    let mut index = 0;
    for point in CrashPoint::ALL {
        if hash(&executable)? != pin {
            return Err(invalid("restart-executable-changed"));
        }
        let mut ticket = RestartTicket {
            version: RestartTicket::VERSION,
            session_id: session,
            task_id: Uuid::new_v4(),
            launch_id: Uuid::new_v4(),
            recovery: false,
            automatic: true,
            source: kraqel_isolation_transport::RestartSource::Synthetic,
            point,
            app_sha256: pin,
        };
        generation(
            root,
            &profile,
            &ticket,
            ticket.launch_id,
            Expect::Crash,
            index,
        )?;
        index += 1;
        ticket.recovery = true;
        projects += u32::from(point.publishes());
        for _ in 0..2 {
            ticket.launch_id = Uuid::new_v4();
            let summary = generation(
                root,
                &profile,
                &ticket,
                ticket.launch_id,
                Expect::Recovery,
                index,
            )?
            .ok_or(invalid("restart-no-summary"))?;
            index += 1;
            if !valid_summary(&summary, point, projects) {
                return Err(invalid("restart-reconciliation-mismatch"));
            }
        }
        ticket.recovery = false;
        ticket.launch_id = Uuid::new_v4();
        let before = hash(&data.join("app/tasks/tasks.sqlite3"))?;
        generation(
            root,
            &profile,
            &ticket,
            ticket.launch_id,
            Expect::ReuseDenied,
            index,
        )?;
        index += 1;
        if hash(&data.join("app/tasks/tasks.sqlite3"))? != before {
            return Err(invalid("restart-reuse-modified-journal"));
        }
        cases.push(json!({"point":point,"passed":true,"published":point.publishes(),"projectCount":projects,"stagingEmpty":true,"replayBlocked":true,"recoveryPasses":2,"staleImportBlocked":true}));
    }
    let journal = hash(&data.join("app/tasks/tasks.sqlite3"))?;
    for bad in 0..4 {
        let mut ticket = RestartTicket {
            version: RestartTicket::VERSION,
            session_id: session,
            task_id: Uuid::new_v4(),
            launch_id: Uuid::new_v4(),
            recovery: true,
            automatic: true,
            source: kraqel_isolation_transport::RestartSource::Synthetic,
            point: CrashPoint::Prepared,
            app_sha256: pin,
        };
        let argument = if bad == 0 {
            Uuid::new_v4()
        } else {
            ticket.launch_id
        };
        match bad {
            1 => ticket.session_id = Uuid::new_v4(),
            2 => ticket.app_sha256 = [0; 32],
            3 => ticket.version = 1,
            _ => (),
        }
        generation(root, &profile, &ticket, argument, Expect::Reject, index)?;
        index += 1;
        if hash(&data.join("app/tasks/tasks.sqlite3"))? != journal {
            return Err(invalid("restart-rejected-start-modified-journal"));
        }
    }
    let removed = profile.remove();
    let report = json!({"formatVersion":1,"researchOnly":true,"productionAcceptance":false,"passed":removed,"profileRemoved":removed,"activeJobProcessesAtEnd":0,"sameIdentityAndData":true,"generations":index,"crashCases":cases,"rejectedStarts":4,"newProjects":projects,"networkMeasurement":false});
    write_json(&root.join("result.json"), &report)?;
    if !removed {
        return Err(invalid("restart-profile-cleanup"));
    }
    Ok(report)
}

pub(in super::super) fn launch_assistive(
    root: &Path,
    app: &Path,
    completed: bool,
) -> Result<Value> {
    let (session, mut profile, pin) = initialize(root, app)?;
    let point = if completed {
        CrashPoint::Completed
    } else {
        CrashPoint::Prepared
    };
    let mut ticket = RestartTicket {
        version: RestartTicket::VERSION,
        session_id: session,
        task_id: Uuid::new_v4(),
        launch_id: Uuid::new_v4(),
        recovery: false,
        automatic: true,
        source: kraqel_isolation_transport::RestartSource::Synthetic,
        point,
        app_sha256: pin,
    };
    generation(root, &profile, &ticket, ticket.launch_id, Expect::Crash, 0)?;
    ticket.recovery = true;
    ticket.automatic = false;
    ticket.launch_id = Uuid::new_v4();
    let summary = generation(
        root,
        &profile,
        &ticket,
        ticket.launch_id,
        Expect::ManualRecovery,
        1,
    )?
    .ok_or(invalid("manual-recovery-summary"))?;
    if !valid_summary(&summary, point, u32::from(completed)) {
        return Err(invalid("manual-recovery-result"));
    }
    let removed = profile.remove();
    let report = json!({"formatVersion":1,"researchOnly":true,"productionAcceptance":false,"passed":removed,"profileRemoved":removed,"activeJobProcessesAtEnd":0,"manualRecovery":true,"screenreaderAcceptanceRequiresUserReport":true,"point":point,"generations":2,"summary":summary,"networkMeasurement":false});
    write_json(&root.join("result.json"), &report)?;
    if !removed {
        return Err(invalid("manual-recovery-profile-cleanup"));
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn window_close_requires_terminal_event_and_never_accepts_recovery_or_replay() {
        let expected = [RestartEventKind::Renderer, RestartEventKind::Finished];
        assert!(expected_event(
            Expect::WindowClose,
            &expected,
            0,
            RestartEventKind::Renderer
        ));
        assert!(expected_event(
            Expect::WindowClose,
            &expected,
            1,
            RestartEventKind::Finished
        ));
        for event in [
            RestartEventKind::Replay,
            RestartEventKind::Recovered,
            RestartEventKind::Checkpoint,
        ] {
            assert!(!expected_event(Expect::WindowClose, &expected, 1, event));
        }
        assert!(!expected_event(
            Expect::WindowClose,
            &expected,
            0,
            RestartEventKind::Finished
        ));
        assert!(!expected_event(
            Expect::WindowClose,
            &expected,
            2,
            RestartEventKind::Finished
        ));
    }
    #[test]
    fn manual_recovery_waits_for_real_replay_and_bounds_additional_calls() {
        let expected = [
            RestartEventKind::Recovered,
            RestartEventKind::Renderer,
            RestartEventKind::Replay,
        ];
        assert!(expected_event(
            Expect::ManualRecovery,
            &expected,
            0,
            RestartEventKind::Recovered
        ));
        assert!(!expected_event(
            Expect::ManualRecovery,
            &expected,
            0,
            RestartEventKind::Replay
        ));
        assert!(expected_event(
            Expect::ManualRecovery,
            &expected,
            3,
            RestartEventKind::Replay
        ));
        assert!(!expected_event(
            Expect::ManualRecovery,
            &expected,
            3,
            RestartEventKind::Checkpoint
        ));
        assert!(!expected_event(
            Expect::ManualRecovery,
            &expected,
            16,
            RestartEventKind::Replay
        ));
        assert!(!expected_event(
            Expect::Recovery,
            &expected,
            3,
            RestartEventKind::Replay
        ));
    }
    #[test]
    fn recovery_evidence_requires_counts_cleanup_and_consumption() {
        let value = RecoverySummary {
            published: true,
            document_count: 2,
            project_count: 1,
            staging_empty: true,
            replay_blocked: true,
        };
        assert!(valid_summary(&value, CrashPoint::Published, 1));
        assert!(!valid_summary(&value, CrashPoint::Prepared, 1));
        assert!(!valid_summary(&value, CrashPoint::Published, 2));
        for index in 0..4 {
            let mut bad = value.clone();
            match index {
                0 => bad.published = false,
                1 => bad.document_count = 3,
                2 => bad.staging_empty = false,
                _ => bad.replay_blocked = false,
            }
            assert!(!valid_summary(&bad, CrashPoint::Published, 1));
        }
    }
}
