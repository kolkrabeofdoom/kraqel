//! Synthetic crash controls exist only behind the authenticated research startup ticket.
use super::{
    Arc, DurableStore, Duration, ImportReceipt, Mutex, ProjectManager, TaskError, fs, now,
};
use kraqel_isolation_transport::{
    CrashPoint, RecoverySummary, RestartEventKind, RestartGrant, RestartTicket,
};
use std::io::{Cursor, Read};
use std::path::Path;

pub(super) struct RestartControl {
    pub ticket: RestartTicket,
    grant: Mutex<RestartGrant>,
}
impl RestartControl {
    pub fn start_automation(&self, window: &tauri::WebviewWindow) {
        if self.ticket.automatic {
            let _ = window.eval(automation(self.ticket.recovery));
        }
    }
    pub fn milestone(&self, point: kraqel_isolation_test_adapter::TaskMilestone) {
        use kraqel_isolation_test_adapter::TaskMilestone;
        self.checkpoint(match point {
            TaskMilestone::Prepared => CrashPoint::Prepared,
            TaskMilestone::CommitIntent => CrashPoint::CommitIntent,
            TaskMilestone::Published => CrashPoint::Published,
        });
    }
    pub fn new(grant: RestartGrant) -> Self {
        Self {
            ticket: grant.ticket.clone(),
            grant: Mutex::new(grant),
        }
    }
    pub fn event(&self, kind: RestartEventKind, summary: Option<RecoverySummary>) {
        if self
            .grant
            .lock()
            .map_err(|_| ())
            .and_then(|mut grant| grant.event(kind, summary).map_err(|_| ()))
            .is_err()
        {
            std::process::exit(2);
        }
    }
    pub fn checkpoint(&self, point: CrashPoint) {
        if !self.ticket.recovery && self.ticket.point == point {
            self.event(RestartEventKind::Checkpoint, None);
            // Controller must terminate this process/job; never emulate a graceful return.
            std::thread::sleep(Duration::from_secs(30));
            std::process::exit(2);
        }
    }
}
pub(super) struct CrashReader {
    pub inner: Cursor<&'static [u8]>,
    pub control: Arc<RestartControl>,
}
impl Read for CrashReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let bound = buffer.len().min(512);
        let count = self.inner.read(&mut buffer[..bound])?;
        if count > 0 {
            self.control.checkpoint(CrashPoint::Reading);
        }
        Ok(count)
    }
}
pub(super) fn synthetic(
    id: uuid::Uuid,
    control: Arc<RestartControl>,
) -> (
    kraqel_isolation_test_adapter::FileTask,
    Box<dyn Read + Send>,
) {
    use kraqel_isolation_test_adapter::{FileOperation, FileTask, SYNTHETIC_ARCHIVE};
    let issued = now();
    (
        FileTask {
            contract_version: 1,
            task_id: id,
            operation: FileOperation::ImportSnapshotV3,
            byte_count: SYNTHETIC_ARCHIVE.len() as u64,
            issued_unix_ms: issued,
            expires_unix_ms: issued + 60000,
        },
        Box::new(CrashReader {
            inner: Cursor::new(SYNTHETIC_ARCHIVE),
            control,
        }),
    )
}
pub(super) fn recover(
    store: &mut DurableStore,
    data: &Path,
    restart: &RestartControl,
) -> Result<Result<ImportReceipt, TaskError>, TaskError> {
    let result = store.outcome(restart.ticket.task_id)?;
    let blocked = store.claim(restart.ticket.task_id) == Err(TaskError::AlreadyConsumed);
    let manager =
        ProjectManager::new(data.join("app/projects")).map_err(|_| TaskError::StateUnavailable)?;
    let project_count = u32::try_from(
        manager
            .list_projects()
            .map_err(|_| TaskError::StateUnavailable)?
            .len(),
    )
    .map_err(|_| TaskError::StateUnavailable)?;
    let staging_empty = fs::read_dir(data.join("app/tasks/staging"))
        .map_err(|_| TaskError::StateUnavailable)?
        .next()
        .is_none();
    let summary = RecoverySummary {
        published: result.is_ok(),
        document_count: result.as_ref().map_or(0, |r| r.document_count),
        project_count,
        staging_empty,
        replay_blocked: blocked,
    };
    if !blocked
        || !staging_empty
        || result.as_ref().is_err_and(|e| {
            !matches!(
                e,
                TaskError::Interrupted
                    | TaskError::Cancelled
                    | TaskError::ReadFailed
                    | TaskError::ImportFailed
                    | TaskError::Expired
            )
        })
    {
        return Err(TaskError::StateUnavailable);
    }
    restart.event(RestartEventKind::Recovered, Some(summary));
    Ok(result)
}
pub(super) fn automation(recovery: bool) -> &'static str {
    if recovery {
        "(()=>{let step=0;const t=setInterval(()=>{const b=document.querySelector('#replay');if(!b||b.disabled)return;if(step===0){step=1;b.click()}else if(document.querySelector('#error')?.textContent.includes('bereits verbraucht')&&document.activeElement===b){clearInterval(t);b.click()}},100)})()"
    } else {
        "(()=>{for(const id of ['import','continue']){const t=setInterval(()=>{const b=document.getElementById(id);if(b&&!b.disabled){clearInterval(t);b.click()}},100)}})()"
    }
}
