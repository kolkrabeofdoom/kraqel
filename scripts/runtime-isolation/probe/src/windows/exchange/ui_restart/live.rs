//! Fixed live request/response inside an authenticated restart generation. No renderer paths.
use super::*;
use kraqel_isolation_test_adapter::{FileOperation, FileTask, SYNTHETIC_ARCHIVE};
use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(super) enum LiveMode {
    Fixture,
    Native,
    Cancel,
    Invalid,
    Partial,
    CloseDialog,
    ClosePartial,
}
impl LiveMode {
    pub(super) fn manual_selection(self) -> bool {
        self == Self::Native || self.window_close()
    }
    pub(super) fn window_close(self) -> bool {
        matches!(self, Self::CloseDialog | Self::ClosePartial)
    }
}

pub(super) struct Bridge {
    requests: fs::File,
    response: Option<fs::File>,
    pub(super) request_child: Option<fs::File>,
    pub(super) response_child: Option<fs::File>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<Result<()>>>,
    thread_id: Arc<AtomicU32>,
    consumed: bool,
}
impl Bridge {
    pub(super) fn new() -> Result<Self> {
        let (requests, request_child) = pipe()?;
        let (response_child, response) = pipe()?;
        Ok(Self {
            requests,
            response: Some(response),
            request_child: Some(request_child),
            response_child: Some(response_child),
            stop: Arc::new(AtomicBool::new(false)),
            thread: None,
            thread_id: Arc::new(AtomicU32::new(0)),
            consumed: false,
        })
    }
    pub(super) fn service(
        &mut self,
        root: &Path,
        ticket: &RestartTicket,
        mode: LiveMode,
        index: usize,
    ) -> Result<()> {
        if self.consumed {
            return Ok(());
        }
        let mut available = 0;
        let ok = unsafe {
            PeekNamedPipe(
                self.requests.as_raw_handle(),
                null_mut(),
                0,
                null_mut(),
                &mut available,
                null_mut(),
            )
        };
        if ok == 0 {
            if unsafe { GetLastError() } == ERROR_BROKEN_PIPE {
                return Ok(());
            }
            return Err(fail("combined-request-peek"));
        }
        if available == 0 {
            return Ok(());
        }
        let mut signal = [0];
        io(
            "combined-request-read",
            self.requests.read_exact(&mut signal),
        )?;
        if available != 1 || signal != [1] {
            return Err(invalid("combined-invalid-request"));
        }
        self.consumed = true;
        let mut writer = self
            .response
            .take()
            .ok_or(invalid("combined-response-reuse"))?;
        let root = root.to_path_buf();
        let id = ticket.task_id;
        let stop = Arc::clone(&self.stop);
        let thread_id = Arc::clone(&self.thread_id);
        self.thread = Some(std::thread::spawn(move || {
            thread_id.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);
            if mode == LiveMode::Cancel {
                io("combined-cancel", writer.write_all(&[0]))?;
                return Ok(());
            }
            let base = host_for(&root)?;
            let host = base.with_file_name(format!("kraqel-combined-input-{}", id.simple()));
            io("combined-create-host", fs::create_dir(&host))?;
            let bytes: &[u8] = if mode == LiveMode::Invalid {
                b"invalid synthetic archive"
            } else {
                SYNTHETIC_ARCHIVE
            };
            let mut fixture = io(
                "combined-create-fixture",
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(host.join("input.kraqel")),
            )?;
            io("combined-write-fixture", fixture.write_all(bytes))?;
            drop(fixture);
            let selected = if matches!(
                mode,
                LiveMode::Native | LiveMode::CloseDialog | LiveMode::ClosePartial
            ) {
                choose_named(&host, false, "input.kraqel", true)?
            } else {
                Some(host.join("input.kraqel"))
            };
            let Some(selected) = selected else {
                io("combined-dialog-cancel", writer.write_all(&[0]))?;
                return Ok(());
            };
            if selected != host.join("input.kraqel") {
                return Err(invalid("combined-only-synthetic-selection"));
            }
            if stop.load(Ordering::SeqCst) {
                return Ok(());
            }
            let input = io(
                "combined-open-selected",
                OpenOptions::new()
                    .read(true)
                    .share_mode(FILE_SHARE_READ)
                    .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                    .open(selected),
            )?;
            let info = file_info(&input)?;
            let size = (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow);
            if size == 0 || size > kraqel_isolation_test_adapter::MAX_TASK_BYTES {
                return Err(invalid("combined-input-budget"));
            }
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| invalid("combined-clock"))?
                .as_millis() as u64;
            let task = FileTask {
                contract_version: 1,
                task_id: id,
                operation: FileOperation::ImportSnapshotV3,
                byte_count: size,
                issued_unix_ms: now,
                expires_unix_ms: now + 60000,
            };
            let frame = serde_json::to_vec(&task).map_err(|_| invalid("combined-task"))?;
            io("combined-selected", writer.write_all(&[1]))?;
            io(
                "combined-header",
                writer.write_all(&(frame.len() as u32).to_le_bytes()),
            )?;
            io("combined-task", writer.write_all(&frame))?;
            let partial = matches!(mode, LiveMode::Partial | LiveMode::ClosePartial);
            let amount = if partial { 1024 } else { size };
            let copied = io(
                "combined-copy",
                std::io::copy(&mut input.take(amount), &mut writer),
            )?;
            if copied != amount {
                return Err(invalid("combined-truncated-source"));
            }
            write_json(
                &root.join(format!("control/live-{index:02}.json")),
                &json!({"requestReceived":true,"bytesSent":copied,"expectedBytes":size,"partial":partial,"nativeDialog":matches!(mode, LiveMode::Native | LiveMode::CloseDialog | LiveMode::ClosePartial)}),
            )?;
            if partial {
                let deadline = Instant::now()
                    + Duration::from_secs(if mode == LiveMode::ClosePartial {
                        60
                    } else {
                        45
                    });
                while !stop.load(Ordering::SeqCst) && Instant::now() < deadline {
                    thread::sleep(Duration::from_millis(20));
                }
            }
            Ok(())
        }));
        Ok(())
    }
    pub(super) fn selected(&self) -> bool {
        self.consumed
    }
    pub(super) fn finish_window_close(&mut self, mode: LiveMode) -> Result<Value> {
        if !mode.window_close() || !self.consumed {
            return Err(invalid("combined-close-no-selection"));
        }
        self.stop.store(true, Ordering::SeqCst);
        drop(self.request_child.take());
        drop(self.response_child.take());
        drop(self.response.take());
        let task = self
            .thread
            .take()
            .ok_or(invalid("combined-close-no-thread"))?;
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut messages = 0;
        while !task.is_finished() && Instant::now() < deadline {
            messages +=
                super::super::ui_live::close_owned_dialog(self.thread_id.load(Ordering::SeqCst));
            thread::sleep(Duration::from_millis(20));
        }
        if !task.is_finished() {
            // Never detach a privileged selection thread after a successful report.
            std::process::exit(2);
        }
        if let Err(error) = task.join().map_err(|_| invalid("combined-close-thread"))?
            && !expected_close_error(mode, &error)
        {
            return Err(error);
        }
        if mode == LiveMode::CloseDialog && messages == 0 {
            return Err(invalid("combined-close-dialog-not-observed"));
        }
        Ok(json!({"selectionThreadJoined":true,"dialogCloseMessages":messages}))
    }
}
fn expected_close_error(mode: LiveMode, error: &Failure) -> bool {
    mode == LiveMode::CloseDialog
        && error.stage == "combined-dialog-cancel"
        && matches!(error.code, 109 | 232)
}
impl Drop for Bridge {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        drop(self.request_child.take());
        drop(self.response_child.take());
        drop(self.response.take());
        if let Some(task) = self.thread.take() {
            let deadline = Instant::now() + Duration::from_secs(5);
            while !task.is_finished() && Instant::now() < deadline {
                super::super::ui_live::close_owned_dialog(self.thread_id.load(Ordering::SeqCst));
                thread::sleep(Duration::from_millis(20));
            }
            // Never detach a live privileged dialog/transfer after claiming successful cleanup.
            if !task.is_finished() {
                std::process::exit(2);
            }
            let _ = task.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manual_selection_is_limited_to_native_and_window_close_modes() {
        for mode in [
            LiveMode::Native,
            LiveMode::CloseDialog,
            LiveMode::ClosePartial,
        ] {
            assert!(mode.manual_selection());
        }
        for mode in [
            LiveMode::Fixture,
            LiveMode::Partial,
            LiveMode::Cancel,
            LiveMode::Invalid,
        ] {
            assert!(!mode.manual_selection());
        }
    }
    #[test]
    fn close_accepts_only_the_cancel_reply_broken_pipe_after_dialog_teardown() {
        for code in [109, 232] {
            let error = Failure {
                stage: "combined-dialog-cancel",
                code,
            };
            assert!(expected_close_error(LiveMode::CloseDialog, &error));
            assert!(!expected_close_error(LiveMode::ClosePartial, &error));
        }
        assert!(!expected_close_error(
            LiveMode::CloseDialog,
            &Failure {
                stage: "combined-dialog-cancel",
                code: 5
            }
        ));
        assert!(!expected_close_error(
            LiveMode::CloseDialog,
            &Failure {
                stage: "combined-copy",
                code: 109
            }
        ));
    }
}
