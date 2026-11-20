//! Headless native test adapter for existing application services. No Tauri command or broker.
//! No paths, shell verbs or format bytes are accepted in the typed task envelope.
#![forbid(unsafe_code)]
use kraqel_domain::ProjectId;
use kraqel_git_service::import_snapshot_archive_with_publish_guard;
use kraqel_storage::project_manager::{ProjectError, ProjectManager};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::io::Read;
use uuid::Uuid;

mod durable;
pub use durable::{DurableStore, TaskMilestone};

/// Narrow research transport budget; the product archive parser separately enforces 128 `MiB`.
pub const MAX_TASK_BYTES: u64 = 8 * 1024 * 1024;
/// Maximum authorization lifetime, independently checked in the isolated adapter.
pub const MAX_LIFETIME_MS: u64 = 60_000;
/// Reproducible synthetic archive, unrelated to user data.
pub const SYNTHETIC_ARCHIVE: &[u8] =
    include_bytes!("../../../../scripts/runtime-isolation/fixtures/input.kraqel");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileOperation {
    #[serde(rename = "importSnapshotV3")]
    ImportSnapshotV3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileTask {
    pub contract_version: u16,
    pub task_id: Uuid,
    pub operation: FileOperation,
    pub byte_count: u64,
    pub issued_unix_ms: u64,
    pub expires_unix_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct TaskControl {
    pub now_unix_ms: u64,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskError {
    InvalidTask,
    Expired,
    Cancelled,
    AlreadyConsumed,
    ReadFailed,
    LengthMismatch,
    ImportFailed,
    Interrupted,
    StateUnavailable,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportReceipt {
    pub contract_version: u16,
    pub task_id: Uuid,
    pub project_id: ProjectId,
    pub document_count: u32,
    pub entity_count: u32,
    pub asset_count: u32,
    pub byte_count: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Ready,
    Reading,
    Committing,
    Consumed,
    Cancelled,
}

/// One adapter instance owns one controller-bound grant; all attempts consume it.
pub struct OneShotAdapter {
    expected_id: Uuid,
    state: State,
}
impl OneShotAdapter {
    #[must_use]
    pub const fn new(expected_id: Uuid) -> Self {
        Self {
            expected_id,
            state: State::Ready,
        }
    }

    /// Cancels an unused grant. During execution the control callback supplies cancellation.
    /// # Errors
    /// A consumed grant or started commit cannot be cancelled or reused.
    pub fn cancel(&mut self) -> Result<(), TaskError> {
        if self.state != State::Ready {
            return Err(TaskError::AlreadyConsumed);
        }
        self.state = State::Cancelled;
        Ok(())
    }

    /// Reads only the delegated stream, then calls the normal atomic project importer.
    /// # Errors
    /// Stable data-poor codes only; no paths or parser diagnostics cross this boundary.
    pub fn execute(
        &mut self,
        task: &FileTask,
        input: &mut impl Read,
        manager: &ProjectManager,
        mut control: impl FnMut() -> TaskControl,
    ) -> Result<ImportReceipt, TaskError> {
        if self.state == State::Cancelled {
            return Err(TaskError::Cancelled);
        }
        if self.state != State::Ready {
            return Err(TaskError::AlreadyConsumed);
        }
        self.state = State::Reading;
        let result = self.run(task, input, manager, &mut control);
        self.state = State::Consumed;
        result
    }

    fn run(
        &mut self,
        task: &FileTask,
        input: &mut impl Read,
        manager: &ProjectManager,
        control: &mut impl FnMut() -> TaskControl,
    ) -> Result<ImportReceipt, TaskError> {
        if task.contract_version != 1
            || task.task_id != self.expected_id
            || task.task_id.is_nil()
            || task.byte_count == 0
            || task.byte_count > MAX_TASK_BYTES
            || task
                .expires_unix_ms
                .checked_sub(task.issued_unix_ms)
                .is_none_or(|v| v == 0 || v > MAX_LIFETIME_MS)
        {
            return Err(TaskError::InvalidTask);
        }
        check_control(task, control())?;
        let mut bytes = Vec::new();
        let mut buffer = [0u8; 16384];
        let mut remaining = task.byte_count;
        while remaining > 0 {
            check_control(task, control())?;
            let bound = usize::try_from(remaining.min(buffer.len() as u64))
                .map_err(|_| TaskError::InvalidTask)?;
            let count = input
                .read(&mut buffer[..bound])
                .map_err(|_| TaskError::ReadFailed)?;
            if count == 0 {
                return Err(TaskError::LengthMismatch);
            }
            bytes.extend_from_slice(&buffer[..count]);
            remaining -= count as u64;
        }
        if input
            .read(&mut buffer[..1])
            .map_err(|_| TaskError::ReadFailed)?
            != 0
        {
            return Err(TaskError::LengthMismatch);
        }
        check_control(task, control())?;
        let sha256 = format!("{:x}", Sha256::digest(&bytes));
        let mut refusal = None;
        let imported = import_snapshot_archive_with_publish_guard(manager, &bytes, || {
            if let Err(error) = check_control(task, control()) {
                refusal = Some(error);
                return Err(ProjectError::InvalidImportedData);
            }
            // Commit fence: after this synchronous approval, late cancellation is too late.
            self.state = State::Committing;
            Ok(())
        });
        let (opened, summary) = imported.map_err(|_| refusal.unwrap_or(TaskError::ImportFailed))?;
        drop(opened);
        Ok(ImportReceipt {
            contract_version: 1,
            task_id: task.task_id,
            project_id: summary.project_id,
            document_count: summary.document_count,
            entity_count: summary.entity_count,
            asset_count: summary.asset_count,
            byte_count: task.byte_count,
            sha256,
        })
    }
}

fn check_control(task: &FileTask, control: TaskControl) -> Result<(), TaskError> {
    if control.cancelled {
        return Err(TaskError::Cancelled);
    }
    if control.now_unix_ms < task.issued_unix_ms {
        return Err(TaskError::InvalidTask);
    }
    if control.now_unix_ms >= task.expires_unix_ms {
        return Err(TaskError::Expired);
    }
    Ok(())
}
