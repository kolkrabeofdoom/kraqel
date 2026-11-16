//! Single fixed request from a running container; only synthetic native selection.
use super::*;
use kraqel_isolation_test_adapter::{FileOperation, FileTask, SYNTHETIC_ARCHIVE};
use std::time::{SystemTime, UNIX_EPOCH};
use windows_sys::Win32::System::Pipes::{CreatePipe, PeekNamedPipe};

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

pub(in super::super) fn launch(root: &Path, app: &Path) -> Result<Value> {
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
    let mut selection_failed = false;
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
                    selection_failed = !matches!(selection.take().unwrap().join(), Ok(Ok(())));
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
            selection = Some(std::thread::spawn(move || {
                let Some(selected) = choose_named(&host, false, "input.kraqel", true)? else {
                    io("live-cancel", writer.write_all(&[0]))?;
                    return Ok(());
                };
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
                let copied = io(
                    "live-copy",
                    std::io::copy(&mut (&mut input).take(size + 1), &mut writer),
                )?;
                if copied != size {
                    return Err(invalid("live-size-changed"));
                }
                Ok(())
            }));
            Ok(())
        },
    )?;
    if let Some(task) = selection {
        if !task.is_finished() {
            return Err(invalid("live-selection-unfinished"));
        }
        selection_failed = !matches!(task.join(), Ok(Ok(())));
    }
    report["liveSelectionRequested"] = json!(consumed);
    report["exchange"] = json!(true);
    report["uiMode"] = json!(true);
    report["channelDirection"] = json!("fixed-request-to-controller-archive-to-container");
    report["inheritedHandles"] = json!(2);
    report["hostDirectoryAclGranted"] = json!(false);
    let outcome = ui_grant::read_outcome(root)?.ok_or(invalid("live-missing-outcome"))?;
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
