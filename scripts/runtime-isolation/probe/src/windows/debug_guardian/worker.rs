//! Fixed synthetic roles only. No path/target/program supplied by bootstrap or command line.
use super::*;

fn input(handle: &File, expected: u32) -> Result<()> {
    let mut owner = 0;
    check(
        unsafe { GetNamedPipeServerProcessId(handle.as_raw_handle(), &mut owner) },
        "bp3-pipe-owner",
    )?;
    if owner != expected {
        return Err(invalid("bp3-pipe-owner"));
    }
    Ok(())
}
pub(in crate::windows) fn run(args: &[OsString]) -> Result<()> {
    // Reject direct host invocation before touching claimed handles.
    own_token()?;
    if args.len() != 3 {
        return Err(invalid("bp3-worker-arguments"));
    }
    let mut raw = Vec::new();
    for arg in args {
        let n = arg
            .to_str()
            .and_then(|s| s.parse::<usize>().ok())
            .filter(|n| *n > 0 && *n != usize::MAX)
            .ok_or(invalid("bp3-worker-handle"))?;
        let handle = n as HANDLE;
        if raw.contains(&handle) || unsafe { GetFileType(handle) } != FILE_TYPE_PIPE {
            return Err(invalid("bp3-worker-handle"));
        }
        raw.push(handle);
    }
    inheritable(&raw, false)?;
    let (mut boot, mut events, mut go) = unsafe {
        (
            File::from_raw_handle(raw[0]),
            File::from_raw_handle(raw[1]),
            File::from_raw_handle(raw[2]),
        )
    };
    let deadline = Instant::now() + Duration::from_secs(8);
    let b: Boot = receive(&mut boot, deadline)?;
    if b.purpose != PURPOSE
        || b.nonce == [0; 32]
        || b.parent == 0
        || b.parent == unsafe { GetCurrentProcessId() }
    {
        return Err(invalid("bp3-bootstrap"));
    }
    input(&boot, b.parent)?;
    input(&go, b.parent)?;
    drop(boot);
    let me = identity(unsafe { GetCurrentProcess() })?;
    let mut report = |kind, child, code| {
        send(
            &mut events,
            &Report {
                nonce: b.nonce,
                kind,
                process: me,
                child,
                code,
            },
        )
    };
    if b.mode == Mode::Leaf {
        return report(Kind::LeafMarker, None, 0);
    }
    report(Kind::RootStarted, None, 0)?;
    let leaf_boot = prepaid(&Boot {
        purpose: PURPOSE.into(),
        nonce: b.nonce,
        parent: me.pid,
        mode: Mode::Leaf,
    })?;
    // A leaf still validates its go pipe creator but never reads a command from it.
    let (leaf_go, leaf_go_write) = pipe()?;
    let handles = [
        leaf_boot.as_raw_handle(),
        events.as_raw_handle(),
        leaf_go.as_raw_handle(),
    ];
    inheritable(&handles, true)?;
    let mut si: STARTUPINFOEXW = unsafe { zeroed() };
    si.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    let mut bytes = 0;
    unsafe {
        InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut bytes);
    }
    let mut storage = vec![0usize; bytes.div_ceil(size_of::<usize>())];
    si.lpAttributeList = storage.as_mut_ptr().cast();
    check(
        unsafe { InitializeProcThreadAttributeList(si.lpAttributeList, 1, 0, &mut bytes) },
        "bp3-child-attributes",
    )?;
    struct Attr(LPPROC_THREAD_ATTRIBUTE_LIST);
    impl Drop for Attr {
        fn drop(&mut self) {
            unsafe {
                DeleteProcThreadAttributeList(self.0);
            }
        }
    }
    let _attr = Attr(si.lpAttributeList);
    check(
        unsafe {
            UpdateProcThreadAttribute(
                si.lpAttributeList,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr().cast(),
                size_of_val(&handles),
                null_mut(),
                null(),
            )
        },
        "bp3-child-handles",
    )?;
    let exe = io("bp3-self-exe", std::env::current_exe())?;
    let mut command = wide(format!(
        "\"{}\" bp3-debug-worker {} {} {}",
        exe.display(),
        handles[0] as usize,
        handles[1] as usize,
        handles[2] as usize
    ));
    let mut pi: PROCESS_INFORMATION = unsafe { zeroed() };
    let flags = CREATE_SUSPENDED
        | CREATE_NO_WINDOW
        | EXTENDED_STARTUPINFO_PRESENT
        | if b.mode == Mode::NestedDebug {
            DEBUG_PROCESS
        } else {
            0
        };
    let ok = unsafe {
        CreateProcessW(
            wide(&exe).as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            1,
            flags,
            null(),
            null(),
            &si.StartupInfo,
            &mut pi,
        )
    };
    let error = unsafe { GetLastError() };
    inheritable(&handles, false)?;
    drop(leaf_boot);
    drop(leaf_go);
    drop(leaf_go_write);
    if ok == 0 {
        send(
            &mut events,
            &Report {
                nonce: b.nonce,
                kind: Kind::ChildFailed,
                process: me,
                child: None,
                code: error,
            },
        )?;
        return Ok(());
    }
    let child = Process(Handle(pi.hProcess), Handle(pi.hThread));
    let child_id = identity(child.0.0)?;
    send(
        &mut events,
        &Report {
            nonce: b.nonce,
            kind: Kind::ChildCreated,
            process: me,
            child: Some(child_id),
            code: 0,
        },
    )?;
    let permission: Go = receive(&mut go, deadline)?;
    if permission.nonce != b.nonce || permission.child != child_id {
        return Err(invalid("bp3-child-go"));
    }
    drop(go);
    if unsafe { ResumeThread(child.1.0) } == u32::MAX {
        return Err(fail("bp3-child-resume"));
    }
    if b.mode == Mode::NestedDebug {
        check(
            unsafe { DebugSetProcessKillOnExit(1) },
            "bp3-inner-kill-on-exit",
        )?;
        let mut active = std::collections::BTreeSet::new();
        let mut leaf_ended = false;
        loop {
            if Instant::now() >= deadline {
                return Err(invalid("bp3-inner-deadline"));
            }
            let mut event: DEBUG_EVENT = unsafe { zeroed() };
            let ok = unsafe { WaitForDebugEvent(&mut event, 50) };
            if ok == 0 {
                if unsafe { GetLastError() } == ERROR_SEM_TIMEOUT {
                    continue;
                }
                return Err(fail("bp3-inner-wait"));
            }
            let decision = (|| {
                if event.dwDebugEventCode == CREATE_PROCESS_DEBUG_EVENT {
                    let handle = unsafe { event.u.CreateProcessInfo.hProcess };
                    let actual = identity(handle)?;
                    let kind = if actual == child_id {
                        Kind::InnerCreate
                    } else if inner_console(handle)? {
                        Kind::InnerConsole
                    } else {
                        return Err(invalid("bp3-inner-generation"));
                    };
                    if active.len() >= 2 || !active.insert(actual.pid) {
                        return Err(invalid("bp3-inner-budget"));
                    }
                    send(
                        &mut events,
                        &Report {
                            nonce: b.nonce,
                            kind,
                            process: me,
                            child: Some(actual),
                            code: 0,
                        },
                    )?;
                } else if !active.contains(&event.dwProcessId) {
                    return Err(invalid("bp3-inner-context"));
                }
                if event.dwDebugEventCode == EXIT_PROCESS_DEBUG_EVENT {
                    active.remove(&event.dwProcessId);
                    if event.dwProcessId == child_id.pid {
                        leaf_ended = true;
                    }
                }
                Ok(())
            })();
            close_debug_file(&event);
            debug_continue(&event)?;
            decision?;
            if leaf_ended && active.is_empty() {
                break;
            }
        }
    }
    if unsafe { WaitForSingleObject(child.0.0, 2000) } != WAIT_OBJECT_0 {
        return Err(invalid("bp3-child-end"));
    }
    let mut exit = 0;
    check(
        unsafe { GetExitCodeProcess(child.0.0, &mut exit) },
        "bp3-child-exit",
    )?;
    if exit != 0 {
        return Err(invalid("bp3-child-failure"));
    }
    send(
        &mut events,
        &Report {
            nonce: b.nonce,
            kind: Kind::RootFinished,
            process: me,
            child: Some(child_id),
            code: 0,
        },
    )
}
