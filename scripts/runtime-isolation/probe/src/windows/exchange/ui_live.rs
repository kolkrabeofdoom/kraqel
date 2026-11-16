//! Single fixed request from a running container; only synthetic native selection.
use super::*;
use kraqel_isolation_test_adapter::{FileOperation, FileTask, SYNTHETIC_ARCHIVE};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};
use std::time::{SystemTime, UNIX_EPOCH};
use windows_sys::Win32::System::Pipes::{CreatePipe, PeekNamedPipe};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumThreadWindows, GetClassNameW, GetWindowThreadProcessId, PostMessageW, WM_CLOSE,
};

fn pipe() -> Result<(fs::File, fs::File)> {
    let mut read = null_mut();
    let mut write = null_mut();
    check(
        unsafe { CreatePipe(&mut read, &mut write, null(), 4096) },
        "live-pipe",
    )?;
    Ok(unsafe {
        (
            fs::File::from_raw_handle(read),
            fs::File::from_raw_handle(write),
        )
    })
}

pub(in super::super) fn launch(root: &Path, app: &Path, hold_transfer: bool) -> Result<Value> {
    if !crate::policy::valid_root(root, false)
        || !root
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.starts_with("kraqel-isolation-ui-"))
    {
        return Err(invalid("live-needs-fresh-root"));
    }
    let host = host_for(root)?;
    io("live-host", fs::create_dir(&host))?;
    io(
        "live-fixture",
        fs::write(host.join("input.kraqel"), SYNTHETIC_ARCHIVE),
    )?;
    let (mut requests, request_writer) = pipe()?;
    let (response_reader, response_writer) = pipe()?;
    let mut response_writer = Some(response_writer);
    let id = uuid::Uuid::new_v4();
    let mut consumed = false;
    let stop = Arc::new(AtomicBool::new(false));
    let selection_thread_id = Arc::new(AtomicU32::new(0));
    let mut selection_error = None;
    let mut selection: Option<std::thread::JoinHandle<Result<()>>> = None;
    let mut report = run_program_serviced(
        root,
        &request_writer,
        &response_reader,
        Program::UiLive {
            executable: app,
            id,
        },
        |_| Ok(()),
        || {
            if consumed {
                if selection.as_ref().is_some_and(|task| task.is_finished()) {
                    selection_error = selection
                        .take()
                        .unwrap()
                        .join()
                        .map_err(|_| invalid("live-selection-thread"))?
                        .err();
                }
                return Ok(());
            }
            let mut available = 0;
            check(
                unsafe {
                    PeekNamedPipe(
                        requests.as_raw_handle(),
                        null_mut(),
                        0,
                        null_mut(),
                        &mut available,
                        null_mut(),
                    )
                },
                "live-peek",
            )?;
            if available == 0 {
                return Ok(());
            }
            consumed = true;
            let mut signal = [0u8; 1];
            io("live-request", requests.read_exact(&mut signal))?;
            let mut writer = response_writer.take().ok_or(invalid("live-replay"))?;
            if available != 1 || signal != [1] {
                return Err(invalid("live-invalid-request"));
            }
            let host = host.clone();
            let stop = Arc::clone(&stop);
            let thread_id = Arc::clone(&selection_thread_id);
            let data = root.join("data");
            selection = Some(std::thread::spawn(move || {
                thread_id.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);
                let Some(selected) = choose_named(&host, false, "input.kraqel", true)? else {
                    if stop.load(Ordering::SeqCst) {
                        return Ok(());
                    }
                    io("live-cancel", writer.write_all(&[0]))?;
                    return Ok(());
                };
                if stop.load(Ordering::SeqCst) {
                    return Ok(());
                }
                if selected != host.join("input.kraqel") {
                    return Err(invalid("live-only-synthetic"));
                }
                let mut input = io(
                    "live-open",
                    OpenOptions::new()
                        .read(true)
                        .share_mode(FILE_SHARE_READ)
                        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                        .open(selected),
                )?;
                let info = file_info(&input)?;
                let size = (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow);
                if size == 0 || size > kraqel_isolation_test_adapter::MAX_TASK_BYTES {
                    return Err(invalid("live-budget"));
                }
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| invalid("live-clock"))?
                    .as_millis() as u64;
                let task = FileTask {
                    contract_version: 1,
                    task_id: id,
                    operation: FileOperation::ImportSnapshotV3,
                    byte_count: size,
                    issued_unix_ms: now,
                    expires_unix_ms: now + 60000,
                };
                let task_bytes = serde_json::to_vec(&task).map_err(|_| invalid("live-task"))?;
                io("live-response", writer.write_all(&[1]))?;
                io(
                    "live-length",
                    writer.write_all(&(task_bytes.len() as u32).to_le_bytes()),
                )?;
                io("live-task", writer.write_all(&task_bytes))?;
                let copied = if hold_transfer {
                    // Research-only deterministic partial transport; never publishes a project.
                    let prefix = size.min(1024);
                    let copied = io(
                        "live-copy",
                        std::io::copy(&mut (&mut input).take(prefix), &mut writer),
                    )?;
                    if copied != prefix || prefix >= size {
                        return Err(invalid("live-hold-needs-partial-input"));
                    }
                    write_json(
                        &data.join("transfer-checkpoint.json"),
                        &json!({"bytesSent":copied,"expectedBytes":size,"complete":false}),
                    )?;
                    let deadline = Instant::now() + Duration::from_secs(60);
                    while !stop.load(Ordering::SeqCst) {
                        if Instant::now() >= deadline {
                            return Err(invalid("live-transfer-hold-timeout"));
                        }
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    return Ok(());
                } else {
                    io(
                        "live-copy",
                        std::io::copy(&mut (&mut input).take(size + 1), &mut writer),
                    )?
                };
                if copied != size {
                    return Err(invalid("live-size-changed"));
                }
                Ok(())
            }));
            Ok(())
        },
    )?;
    stop.store(true, Ordering::SeqCst);
    // Release controller duplicates so a writer cannot stay blocked after the app reader exits.
    drop(request_writer);
    drop(response_reader);
    let mut close_messages = 0;
    if let Some(task) = selection {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !task.is_finished() && Instant::now() < deadline {
            close_messages += close_owned_dialog(selection_thread_id.load(Ordering::SeqCst));
            std::thread::sleep(Duration::from_millis(20));
        }
        if !task.is_finished() {
            return Err(invalid("live-selection-unfinished"));
        }
        selection_error = task
            .join()
            .map_err(|_| invalid("live-selection-thread"))?
            .err();
    }
    report["liveSelectionRequested"] = json!(consumed);
    report["exchange"] = json!(true);
    report["uiMode"] = json!(true);
    report["channelDirection"] = json!("fixed-request-to-controller-archive-to-container");
    report["inheritedHandles"] = json!(2);
    report["hostDirectoryAclGranted"] = json!(false);
    let outcome = ui_grant::read_outcome(root)?.ok_or(invalid("live-missing-outcome"))?;
    let selection_failed = selection_error
        .as_ref()
        .is_some_and(|error| !expected_cancel_error(error, &outcome));
    if let Some(error) = &selection_error {
        report["selectionError"] = json!({"stage":error.stage,"code":error.code});
    }
    report["selectionThreadJoined"] = json!(true);
    report["dialogCloseMessages"] = json!(close_messages);
    report["transferHold"] = json!(hold_transfer);
    report["partialTransferObserved"] = json!(root.join("data/transfer-checkpoint.json").exists());
    report["passed"] = json!(
        !selection_failed
            && consumed
            && report["processExitCode"] == 0
            && report["activeJobProcessesAtEnd"] == 0
            && report["profileRemoved"] == true
            && (outcome["phase"] == "done" || outcome["phase"] == "cancelled")
    );
    report["uiOutcome"] = outcome;
    report["productionAcceptance"] = json!(false);
    write_json(&root.join("result.json"), &report)?;
    Ok(report)
}

struct DialogClose {
    thread_id: u32,
    posted: u32,
}
unsafe extern "system" fn close_dialog(window: HANDLE, parameter: isize) -> i32 {
    // EnumThreadWindows invokes synchronously; this stack-owned context remains live.
    let context = unsafe { &mut *(parameter as *mut DialogClose) };
    let mut process_id = 0;
    let owner_thread = unsafe { GetWindowThreadProcessId(window, &mut process_id) };
    let mut class = [0u16; 64];
    let length = unsafe { GetClassNameW(window, class.as_mut_ptr(), class.len() as i32) };
    if owner_thread == context.thread_id
        && process_id == unsafe { GetCurrentProcessId() }
        && length == 6
        && class[..6] == [35, 51, 50, 55, 55, 48]
        && unsafe { PostMessageW(window, WM_CLOSE, 0, 0) } != 0
    {
        context.posted += 1;
    }
    1
}
pub(super) fn close_owned_dialog(thread_id: u32) -> u32 {
    if thread_id == 0 {
        return 0;
    }
    let mut context = DialogClose {
        thread_id,
        posted: 0,
    };
    unsafe {
        EnumThreadWindows(
            thread_id,
            Some(close_dialog),
            (&mut context as *mut DialogClose) as isize,
        );
    }
    context.posted
}

fn expected_cancel_error(error: &Failure, outcome: &Value) -> bool {
    error.code == i64::from(ERROR_BROKEN_PIPE)
        && matches!(
            error.stage,
            "live-cancel" | "live-copy" | "live-task" | "live-length" | "live-response"
        )
        && outcome["phase"] == "cancelled"
        && outcome["error"] == "CANCELLED"
        && outcome["receipt"].is_null()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn broken_pipe_is_expected_only_for_cancelled_transport_without_receipt() {
        let outcome = json!({"phase":"cancelled","error":"CANCELLED","receipt":null});
        let error = Failure {
            stage: "live-copy",
            code: i64::from(ERROR_BROKEN_PIPE),
        };
        assert!(expected_cancel_error(&error, &outcome));
        assert!(!expected_cancel_error(
            &Failure {
                stage: "live-open",
                code: error.code
            },
            &outcome
        ));
        assert!(!expected_cancel_error(
            &Failure {
                stage: error.stage,
                code: 5
            },
            &outcome
        ));
        assert!(!expected_cancel_error(
            &error,
            &json!({"phase":"done","error":null,"receipt":{}})
        ));
        assert!(!expected_cancel_error(
            &error,
            &json!({"phase":"cancelled","error":"CANCELLED","receipt":{}})
        ));
    }
}
