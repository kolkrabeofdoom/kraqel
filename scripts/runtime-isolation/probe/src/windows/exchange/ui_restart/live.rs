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
            let selected = if mode == LiveMode::Native {
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
            let amount = if mode == LiveMode::Partial {
                1024
            } else {
                size
            };
            let copied = io(
                "combined-copy",
                std::io::copy(&mut input.take(amount), &mut writer),
            )?;
            if copied != amount {
                return Err(invalid("combined-truncated-source"));
            }
            write_json(
                &root.join(format!("control/live-{index:02}.json")),
                &json!({"requestReceived":true,"bytesSent":copied,"expectedBytes":size,"partial":mode==LiveMode::Partial,"nativeDialog":mode==LiveMode::Native}),
            )?;
            if mode == LiveMode::Partial {
                let deadline = Instant::now() + Duration::from_secs(45);
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
