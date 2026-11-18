//! Test-only host. No normal CLI, store records, product keys or C4 admission path.
use super::*;
use sha2::{Digest as _, Sha256};
use std::{collections::BTreeMap, fs::OpenOptions, os::windows::fs::OpenOptionsExt};
use windows_sys::Win32::System::Diagnostics::ToolHelp::*;

struct Job(Handle);
impl Job {
    fn counts(&self) -> Result<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION> {
        let mut value = unsafe { zeroed::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() };
        check(
            unsafe {
                QueryInformationJobObject(
                    self.0.0,
                    JobObjectBasicAccountingInformation,
                    (&mut value as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                    size_of_val(&value) as u32,
                    null_mut(),
                )
            },
            "bp3-job-count",
        )?;
        Ok(value)
    }
    fn stop(&self) -> Result<()> {
        check(unsafe { TerminateJobObject(self.0.0, 3) }, "bp3-job-stop")
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            TerminateJobObject(self.0.0, 3);
        }
    }
}
fn sha(path: &Path) -> Result<[u8; 32]> {
    Ok(Sha256::digest(io("bp3-code-read", fs::read(path))?).into())
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|v| format!("{v:02x}")).collect()
}
fn environment(root: &Path) -> Result<Vec<u16>> {
    let mut name = [0u16; 32768];
    let n = unsafe {
        windows_sys::Win32::System::SystemInformation::GetWindowsDirectoryW(
            name.as_mut_ptr(),
            name.len() as u32,
        )
    } as usize;
    if n == 0 || n >= name.len() {
        return Err(invalid("bp3-windows-directory"));
    }
    let windows = String::from_utf16(&name[..n]).map_err(|_| invalid("bp3-windows-directory"))?;
    let mut values = vec![
        format!("SystemRoot={windows}"),
        format!("TEMP={}", root.display()),
        format!("TMP={}", root.display()),
        format!("WINDIR={windows}"),
    ];
    // AppContainer creation requires its OS profile environment. Only these standard folder
    // variables are copied, never PATH, arbitrary session variables, credentials or API keys.
    for name in [
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
        "PROGRAMDATA",
        "ALLUSERSPROFILE",
    ] {
        if let Some(value) = std::env::var_os(name) {
            let value = value.to_str().ok_or(invalid("bp3-environment-encoding"))?;
            values.push(format!("{name}={value}"));
        }
    }
    values.sort_by_key(|s| s.to_ascii_uppercase());
    let mut block = Vec::new();
    for value in values {
        block.extend(wide(value));
    }
    block.push(0);
    Ok(block)
}
fn process(p: HANDLE, job: &Job, profile: &Profile, exe: &Path) -> Result<Identity> {
    let facts = token_facts(p, Some(profile.sid))?;
    let mut member = 0;
    check(
        unsafe { IsProcessInJob(p, job.0.0, &mut member) },
        "bp3-exact-job",
    )?;
    let mut name = [0u16; 32768];
    let mut n = name.len() as u32;
    check(
        unsafe { QueryFullProcessImageNameW(p, 0, name.as_mut_ptr(), &mut n) },
        "bp3-process-image",
    )?;
    let image = PathBuf::from(
        String::from_utf16(&name[..n as usize]).map_err(|_| invalid("bp3-process-image"))?,
    );
    if member == 0 {
        return Err(invalid("bp3-process-job"));
    }
    if facts["isAppContainer"] != true {
        return Err(invalid("bp3-process-not-container"));
    }
    if facts["capabilityCount"] != 0 {
        return Err(invalid("bp3-process-capabilities"));
    }
    if facts["expectedIdentity"] != true {
        return Err(invalid("bp3-process-profile"));
    }
    if io("bp3-image-canonical", image.canonicalize())?
        != io("bp3-image-canonical", exe.canonicalize())?
    {
        return Err(invalid("bp3-process-image-binding"));
    }
    identity(p)
}
fn parent(child: u32, expected: u32) -> Result<()> {
    let snap = Handle(unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) });
    if snap.0 == INVALID_HANDLE_VALUE {
        return Err(fail("bp3-parent-snapshot"));
    }
    let mut e = unsafe { zeroed::<PROCESSENTRY32W>() };
    e.dwSize = size_of_val(&e) as u32;
    let mut ok = unsafe { Process32FirstW(snap.0, &mut e) };
    while ok != 0 {
        if e.th32ProcessID == child {
            return if e.th32ParentProcessID == expected {
                Ok(())
            } else {
                Err(invalid("bp3-parent"))
            };
        }
        ok = unsafe { Process32NextW(snap.0, &mut e) };
    }
    Err(invalid("bp3-parent-absent"))
}
fn cleanup_debug(job: &Job) -> Result<()> {
    job.stop()?;
    let deadline = Instant::now() + Duration::from_secs(4);
    while job.counts()?.ActiveProcesses != 0 {
        if Instant::now() >= deadline {
            return Err(invalid("bp3-cleanup-drain"));
        }
        let mut event = unsafe { zeroed::<DEBUG_EVENT>() };
        if unsafe { WaitForDebugEvent(&mut event, 20) } != 0 {
            close_debug_file(&event);
            debug_continue(&event)?;
        } else if unsafe { GetLastError() } != ERROR_SEM_TIMEOUT {
            return Err(fail("bp3-cleanup-wait"));
        }
    }
    Ok(())
}
struct DebugCleanup<'a>(&'a Job);
impl Drop for DebugCleanup<'_> {
    fn drop(&mut self) {
        let _ = cleanup_debug(self.0);
    }
}

fn case(source: &Path, mode: Mode, reject: bool) -> Result<Value> {
    let suffix = uuid::Uuid::new_v4().simple().to_string();
    let root = std::env::temp_dir().join(format!("kraqel-isolation-bp3dbg-{suffix}"));
    if !crate::policy::valid_root(&root, false) {
        return Err(invalid("bp3-temp-root"));
    }
    io("bp3-temp-create", fs::create_dir(&root))?;
    let mut profile = Profile::create(&format!("bp3dbg-{suffix}"))?;
    let exe = root.join("worker.exe");
    io("bp3-code-copy", fs::copy(source, &exe))?;
    grant(&root, profile.sid, FILE_GENERIC_READ | FILE_GENERIC_EXECUTE)?;
    grant(&exe, profile.sid, FILE_GENERIC_READ | FILE_GENERIC_EXECUTE)?;
    let pin = io(
        "bp3-code-pin",
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&exe),
    )?;
    let console = console_path()?;
    let console_pin = io(
        "bp3-console-pin",
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(&console),
    )?;
    let source_sha = sha(source)?;
    if sha(&exe)? != source_sha {
        return Err(invalid("bp3-code-copy-hash"));
    }
    let raw = unsafe { CreateJobObjectW(null(), null()) };
    if raw.is_null() {
        return Err(fail("bp3-create-job"));
    }
    let job = Job(Handle(raw));
    let mut limits = unsafe { zeroed::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() };
    limits.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
    // Two synthetic workers plus at most two OS console helpers; no breakaway.
    limits.BasicLimitInformation.ActiveProcessLimit = 4;
    check(
        unsafe {
            SetInformationJobObject(
                job.0.0,
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                size_of_val(&limits) as u32,
            )
        },
        "bp3-job-limits",
    )?;
    let mut nonce = [0; 32];
    getrandom::fill(&mut nonce).map_err(|_| invalid("bp3-nonce"))?;
    let boot = prepaid(&Boot {
        purpose: PURPOSE.into(),
        nonce,
        parent: unsafe { GetCurrentProcessId() },
        mode,
    })?;
    let (mut events, writer) = pipe()?;
    let (go, mut write_go) = pipe()?;
    let handles = [
        boot.as_raw_handle(),
        writer.as_raw_handle(),
        go.as_raw_handle(),
    ];
    inheritable(&handles, true)?;
    let caps = SECURITY_CAPABILITIES {
        AppContainerSid: profile.sid,
        Capabilities: null_mut(),
        CapabilityCount: 0,
        Reserved: 0,
    };
    let jobs = [job.0.0];
    let attrs = Attributes::with_handles_and_jobs(&caps, &handles, &jobs)?;
    let mut si = unsafe { zeroed::<STARTUPINFOEXW>() };
    si.StartupInfo.cb = size_of_val(&si) as u32;
    si.lpAttributeList = attrs.ptr;
    let mut command = wide(format!(
        "\"{}\" bp3-debug-worker {} {} {}",
        exe.display(),
        handles[0] as usize,
        handles[1] as usize,
        handles[2] as usize
    ));
    let mut pi = unsafe { zeroed::<PROCESS_INFORMATION>() };
    let environment = environment(&root)?;
    check(
        unsafe {
            CreateProcessW(
                wide(&exe).as_ptr(),
                command.as_mut_ptr(),
                null(),
                null(),
                1,
                CREATE_SUSPENDED
                    | CREATE_NO_WINDOW
                    | DEBUG_PROCESS
                    | EXTENDED_STARTUPINFO_PRESENT
                    | CREATE_UNICODE_ENVIRONMENT,
                environment.as_ptr().cast(),
                wide(&root).as_ptr(),
                &si.StartupInfo,
                &mut pi,
            )
        },
        "bp3-create-root",
    )?;
    let started = Process(Handle(pi.hProcess), Handle(pi.hThread));
    // Once a process exists, only explicit deletion after a confirmed drain is permitted.
    // An early error/unproved end must retain this fresh research profile, never auto-adopt it.
    profile.active = false;
    let cleanup = DebugCleanup(&job);
    inheritable(&handles, false)?;
    drop(boot);
    drop(writer);
    drop(go);
    let root_id = process(started.0.0, &job, &profile, &exe)?;
    check(unsafe { DebugSetProcessKillOnExit(1) }, "bp3-kill-on-exit")?;
    if unsafe { ResumeThread(started.1.0) } == u32::MAX {
        return Err(fail("bp3-resume-root"));
    }
    let result = (|| {
        let deadline = Instant::now() + Duration::from_secs(12);
        let mut creates = BTreeMap::<u32, Identity>::new();
        let mut consoles = Vec::<Identity>::new();
        let mut ends = Vec::new();
        let mut reports = Vec::<Report>::new();
        let mut held_child = None;
        let mut child_id = None;
        let mut root_rejected = false;
        loop {
            if Instant::now() >= deadline {
                return Err(invalid("bp3-controller-deadline"));
            }
            while available(&events)? > 0 {
                if reports.len() >= 16 {
                    return Err(invalid("bp3-report-budget"));
                }
                let report: Report = receive(&mut events, deadline)?;
                if report.nonce != nonce {
                    return Err(invalid("bp3-report-nonce"));
                }
                if report.kind == Kind::ChildCreated {
                    if report.process != root_id || child_id.is_some() {
                        return Err(invalid("bp3-child-report"));
                    }
                    let id = report.child.ok_or(invalid("bp3-child-report"))?;
                    let handle = Handle(unsafe {
                        OpenProcess(
                            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                            0,
                            id.pid,
                        )
                    });
                    if handle.0.is_null() {
                        return Err(fail("bp3-child-hold"));
                    }
                    if process(handle.0, &job, &profile, &exe)? != id {
                        return Err(invalid("bp3-child-generation"));
                    }
                    parent(id.pid, root_id.pid)?;
                    child_id = Some(id);
                    held_child = Some(handle);
                    send(&mut write_go, &Go { nonce, child: id })?;
                }
                reports.push(report);
            }
            if !ends.is_empty() && ends.contains(&root_id.pid) && job.counts()?.ActiveProcesses == 0
            {
                break;
            }
            let mut event = unsafe { zeroed::<DEBUG_EVENT>() };
            if unsafe { WaitForDebugEvent(&mut event, 20) } == 0 {
                if unsafe { GetLastError() } == ERROR_SEM_TIMEOUT {
                    continue;
                }
                return Err(fail("bp3-outer-wait"));
            }
            // Always continue an obtained event, including after a failed native assertion.
            let decision = (|| {
                if event.dwDebugEventCode == CREATE_PROCESS_DEBUG_EVENT {
                    let handle = unsafe { event.u.CreateProcessInfo.hProcess };
                    let is_console = io("bp3-event-image", image_path(handle)?.canonicalize())?
                        == io("bp3-event-image", console.canonicalize())?;
                    let actual = process(
                        handle,
                        &job,
                        &profile,
                        if is_console { &console } else { &exe },
                    )?;
                    if is_console {
                        consoles.push(actual);
                    }
                    if creates.len() >= 4 || creates.insert(actual.pid, actual).is_some() {
                        return Err(invalid("bp3-duplicate-create"));
                    }
                    if actual == root_id && reject {
                        if available(&events)? != 0 {
                            return Err(invalid("bp3-marker-before-root-gate"));
                        }
                        check(
                            unsafe { TerminateProcess(started.0.0, 3) },
                            "bp3-reject-root",
                        )?;
                        root_rejected = true;
                    }
                } else if event.dwDebugEventCode == EXIT_PROCESS_DEBUG_EVENT {
                    if !creates.contains_key(&event.dwProcessId) {
                        return Err(invalid("bp3-unseen-exit"));
                    }
                    ends.push(event.dwProcessId);
                }
                Ok(())
            })();
            close_debug_file(&event);
            debug_continue(&event)?;
            decision?;
        }
        // Final writes precede process exit, but drain the pipe after the last exit event too.
        while available(&events)? > 0 {
            if reports.len() >= 16 {
                return Err(invalid("bp3-report-budget"));
            }
            let r: Report = receive(&mut events, deadline)?;
            if r.nonce != nonce {
                return Err(invalid("bp3-report-nonce"));
            }
            reports.push(r);
        }
        if unsafe { WaitForSingleObject(started.0.0, 1000) } != WAIT_OBJECT_0 {
            return Err(invalid("bp3-root-unsignaled"));
        }
        if let Some(handle) = held_child.as_ref()
            && unsafe { WaitForSingleObject(handle.0, 1000) } != WAIT_OBJECT_0
        {
            return Err(invalid("bp3-child-unsignaled"));
        }
        if let Some(handle) = held_child.as_ref() {
            let mut code = 0;
            check(
                unsafe { GetExitCodeProcess(handle.0, &mut code) },
                "bp3-held-child-exit",
            )?;
            if code != 0 {
                return Err(invalid("bp3-held-child-failure"));
            }
        }
        let mut exit = 0;
        check(
            unsafe { GetExitCodeProcess(started.0.0, &mut exit) },
            "bp3-root-exit",
        )?;
        if exit != if reject { 3 } else { 0 } {
            return Err(invalid("bp3-root-failed"));
        }
        validate_reports(&reports, nonce, root_id, child_id, mode, reject)?;
        let leaf = reports.iter().find(|r| r.kind == Kind::LeafMarker);
        let inner = reports.iter().find(|r| r.kind == Kind::InnerCreate);
        let failed = reports.iter().find(|r| r.kind == Kind::ChildFailed);
        if reject {
            if !root_rejected || !reports.is_empty() || creates.len() != 1 {
                return Err(invalid("bp3-root-denial-evidence"));
            }
        } else if let Some(id) = child_id {
            if leaf.map(|r| r.process) != Some(id)
                || leaf.is_some_and(|r| r.child.is_some() || r.code != 0)
                || !reports.iter().any(|r| {
                    r.kind == Kind::RootFinished && r.process == root_id && r.child == Some(id)
                })
            {
                return Err(invalid("bp3-leaf-evidence"));
            }
            if mode == Mode::Inherit && (creates.get(&id.pid) != Some(&id) || inner.is_some()) {
                return Err(invalid("bp3-normal-inheritance"));
            }
            if mode == Mode::NestedDebug
                && (inner.map(|r| r.child) != Some(Some(id)) || creates.contains_key(&id.pid))
            {
                return Err(invalid("bp3-nested-evidence"));
            }
        } else if failed.is_none() {
            return Err(invalid("bp3-no-child-outcome"));
        }
        let counts = job.counts()?;
        let console_count = consoles.len()
            + reports
                .iter()
                .filter(|r| r.kind == Kind::InnerConsole)
                .count();
        let expected = if child_id.is_some() { 2 } else { 1 } + console_count as u32;
        if counts.TotalProcesses != expected
            || counts.ActiveProcesses != 0
            || creates.len() != ends.len()
        {
            return Err(invalid("bp3-job-evidence"));
        }
        Ok(
            json!({"mode":format!("{mode:?}"),"rejectRoot":reject,"root":root_id,"child":child_id,"outerCreates":creates.values().collect::<Vec<_>>(),"outerConsoles":consoles,"outerEnds":ends,"reports":reports,"rootExit":exit,"rootRejectedBeforeUserMode":root_rejected,"totalJobProcesses":counts.TotalProcesses,"activeAtEnd":counts.ActiveProcesses,"childHeldAndVerified":held_child.is_some(),"childCreateError":failed.map(|r|r.code),"childRanOutsideOuterDebugChain":mode==Mode::NestedDebug&&child_id.is_some(),"runtimeAuthority":false,"dataAuthority":false}),
        )
    })();
    let drain = cleanup_debug(&job);
    drop(cleanup);
    drop(started);
    drop(job);
    drop(pin);
    drop(console_pin);
    drop(events);
    drop(write_go);
    drain?;
    if !profile.remove() {
        return Err(invalid("bp3-profile-remove"));
    }
    let canonical = io("bp3-temp-resolve", root.canonicalize())?;
    if !crate::policy::valid_root(&canonical, true) {
        return Err(invalid("bp3-cleanup-scope"));
    }
    // The test harness may still hold an image reference. Its external runner removes only
    // recorded, resolved own fixtures after this debugger process has exited. Never weaken ACLs.
    let removed = match fs::remove_dir_all(&canonical) {
        Ok(()) => true,
        Err(e) if matches!(e.raw_os_error(), Some(5 | 32)) => false,
        Err(e) => {
            return Err(Failure {
                stage: "bp3-temp-remove",
                code: e.raw_os_error().unwrap_or(-1).into(),
            });
        }
    };
    let mut report = result?;
    report["profileRemoved"] = json!(true);
    report["fixtureRemoved"] = json!(removed);
    report["fixtureRoot"] = json!(canonical);
    Ok(report)
}

#[test]
#[ignore = "explicit synthetic BP3 debugger inheritance series; no WebView/store/projects"]
fn bp3_debug_chain_native_series() {
    let saved = std::env::var_os("KRAQEL_BP3_DEBUG_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("kraqel-bp3-debug-build-20261118-01"));
    let resolved = saved.canonicalize().unwrap();
    assert_eq!(
        resolved.parent(),
        Some(std::env::temp_dir().canonicalize().unwrap().as_path())
    );
    let name = resolved.file_name().unwrap().to_str().unwrap();
    assert!(
        name.starts_with("kraqel-bp3-debug-build-")
            && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
    );
    let source = saved.join("controller.exe");
    assert!(source.is_file());
    let denied = case(&source, Mode::Inherit, true).unwrap();
    let inherited = case(&source, Mode::Inherit, false).unwrap();
    let nested = case(&source, Mode::NestedDebug, false).unwrap();
    let report = json!({"purpose":PURPOSE,"sourceSha256":hex(&sha(&source).unwrap()),"denied":denied,"inherited":inherited,"nested":nested,"networkMeasured":false,"exactModuleBindingProven":false,"webviewRoleBindingProven":false});
    fs::write(
        saved.join("run.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    assert!(
        report["inherited"]["childCreateError"].is_null(),
        "{report}"
    );
    assert_eq!(report["inherited"]["childHeldAndVerified"], true);
    // A blocked nested attempt is an inconclusive compatibility result, never a pass for BP3.
    assert!(report["nested"]["childCreateError"].is_null(), "{report}");
    assert_eq!(report["nested"]["childRanOutsideOuterDebugChain"], true);
}
