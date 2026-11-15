use kraqel_domain::{CURRENT_SCHEMA_VERSION, Project, ProjectId};
use kraqel_isolation_test_adapter::{
    FileOperation, FileTask, MAX_TASK_BYTES, OneShotAdapter, SYNTHETIC_ARCHIVE, TaskControl,
    TaskError,
};
use kraqel_storage::project_manager::ProjectManager;
use std::{
    fs,
    io::{self, Read},
};
use uuid::Uuid;

fn task() -> FileTask {
    FileTask {
        contract_version: 1,
        task_id: Uuid::new_v4(),
        operation: FileOperation::ImportSnapshotV3,
        byte_count: SYNTHETIC_ARCHIVE.len() as u64,
        issued_unix_ms: 1000,
        expires_unix_ms: 61000,
    }
}
fn alive() -> TaskControl {
    TaskControl {
        now_unix_ms: 1001,
        cancelled: false,
    }
}
fn empty(manager: &ProjectManager) {
    assert_eq!(manager.list_projects().unwrap().len(), 0);
    assert_eq!(
        fs::read_dir(manager.root()).unwrap().count(),
        0,
        "no staging residue"
    );
}
struct Unreadable;
impl Read for Unreadable {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("grant must be rejected before reading");
    }
}

#[test]
fn import_is_a_new_copy_and_replay_never_reads_or_publishes_again() {
    let root = tempfile::tempdir().unwrap();
    let manager = ProjectManager::new(root.path()).unwrap();
    let task = task();
    let mut adapter = OneShotAdapter::new(task.task_id);
    let receipt = adapter
        .execute(
            &task,
            &mut io::Cursor::new(SYNTHETIC_ARCHIVE),
            &manager,
            alive,
        )
        .unwrap();
    assert_ne!(
        receipt.project_id.to_string(),
        "11111111-1111-4111-8111-111111111111"
    );
    assert_eq!(receipt.document_count, 2);
    let opened = manager
        .open_project(receipt.project_id.to_string())
        .unwrap();
    let text: String = opened
        .database()
        .connection()
        .query_row(
            "SELECT plain_text FROM document WHERE kind = 'scene'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(text, "Importprobe: ÄÖÜ ß Ελληνικά Кириллица.");
    drop(opened);
    assert_eq!(
        adapter
            .execute(&task, &mut Unreadable, &manager, alive)
            .unwrap_err(),
        TaskError::AlreadyConsumed
    );
    assert_eq!(manager.list_projects().unwrap().len(), 1);
    assert_eq!(adapter.cancel(), Err(TaskError::AlreadyConsumed));
    let task = crate::task();
    let again = OneShotAdapter::new(task.task_id)
        .execute(
            &task,
            &mut io::Cursor::new(SYNTHETIC_ARCHIVE),
            &manager,
            alive,
        )
        .unwrap();
    assert_ne!(again.project_id, receipt.project_id); // A fresh process/session needs a fresh grant in production.
}

#[test]
fn wrong_binding_version_budget_and_lifetime_consume_the_grant_without_reading() {
    for index in 0..6 {
        let root = tempfile::tempdir().unwrap();
        let manager = ProjectManager::new(root.path()).unwrap();
        let mut task = task();
        let id = task.task_id;
        match index {
            0 => task.task_id = Uuid::new_v4(),
            1 => task.contract_version = 2,
            2 => task.byte_count = MAX_TASK_BYTES + 1,
            3 => task.byte_count = 0,
            4 => task.expires_unix_ms = 100_000,
            _ => task.issued_unix_ms = 70000,
        }
        let mut adapter = OneShotAdapter::new(id);
        assert_eq!(
            adapter
                .execute(&task, &mut Unreadable, &manager, alive)
                .unwrap_err(),
            TaskError::InvalidTask
        );
        assert_eq!(
            adapter
                .execute(&task, &mut Unreadable, &manager, alive)
                .unwrap_err(),
            TaskError::AlreadyConsumed
        );
        empty(&manager);
    }
}

#[test]
fn expiry_and_prestart_cancellation_never_read() {
    let root = tempfile::tempdir().unwrap();
    let manager = ProjectManager::new(root.path()).unwrap();
    let task = task();
    let mut adapter = OneShotAdapter::new(task.task_id);
    assert_eq!(
        adapter
            .execute(&task, &mut Unreadable, &manager, || TaskControl {
                now_unix_ms: 61000,
                cancelled: false
            })
            .unwrap_err(),
        TaskError::Expired
    );
    let mut adapter = OneShotAdapter::new(task.task_id);
    adapter.cancel().unwrap();
    assert_eq!(
        adapter
            .execute(&task, &mut Unreadable, &manager, alive)
            .unwrap_err(),
        TaskError::Cancelled
    );
    empty(&manager);
}

#[test]
fn late_cancel_and_expiry_discard_fully_initialized_unpublished_project() {
    for cancelled in [true, false] {
        let root = tempfile::tempdir().unwrap();
        let manager = ProjectManager::new(root.path()).unwrap();
        let task = task();
        let mut saw_staging = false;
        let result = OneShotAdapter::new(task.task_id).execute(
            &task,
            &mut io::Cursor::new(SYNTHETIC_ARCHIVE),
            &manager,
            || {
                let staged = fs::read_dir(root.path())
                    .unwrap()
                    .any(|entry| entry.unwrap().path().join("kraqel.sqlite3").is_file());
                saw_staging |= staged;
                if staged {
                    TaskControl {
                        now_unix_ms: if cancelled { 1001 } else { 61000 },
                        cancelled,
                    }
                } else {
                    alive()
                }
            },
        );
        assert!(saw_staging, "guard must run after initializing SQLite");
        assert_eq!(
            result.unwrap_err(),
            if cancelled {
                TaskError::Cancelled
            } else {
                TaskError::Expired
            }
        );
        empty(&manager);
    }
}

#[test]
fn invalid_and_mismatched_streams_leave_existing_project_byte_identical() {
    let root = tempfile::tempdir().unwrap();
    let manager = ProjectManager::new(root.path()).unwrap();
    let id = ProjectId::new();
    let opened = manager
        .create_project(
            id.to_string(),
            &Project {
                id,
                name: "Unverändert".into(),
                schema_version: CURRENT_SCHEMA_VERSION,
            },
        )
        .unwrap();
    let db = opened.directory().join("kraqel.sqlite3");
    drop(opened);
    let before = fs::read(&db).unwrap();
    for input in [
        &b"not a ZIP archive"[..],
        &SYNTHETIC_ARCHIVE[..SYNTHETIC_ARCHIVE.len() - 1],
    ] {
        let mut task = task();
        task.byte_count = input.len() as u64;
        assert_eq!(
            OneShotAdapter::new(task.task_id)
                .execute(&task, &mut io::Cursor::new(input), &manager, alive)
                .unwrap_err(),
            TaskError::ImportFailed
        );
    }
    let task = task();
    assert_eq!(
        OneShotAdapter::new(task.task_id)
            .execute(&task, &mut &b"short"[..], &manager, alive)
            .unwrap_err(),
        TaskError::LengthMismatch
    );
    assert_eq!(fs::read(db).unwrap(), before);
    assert_eq!(manager.list_projects().unwrap().len(), 1);
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
}

#[test]
fn envelope_rejects_shell_verbs_paths_and_unknown_operations() {
    let task = task();
    let mut value = serde_json::to_value(&task).unwrap();
    value["path"] = serde_json::json!("C:\\private.txt");
    assert!(serde_json::from_value::<FileTask>(value).is_err());
    let mut value = serde_json::to_value(&task).unwrap();
    value["operation"] = serde_json::json!("shellExecute");
    assert!(serde_json::from_value::<FileTask>(value).is_err());
}

#[test]
fn read_failures_return_only_a_stable_code_and_consume_the_grant() {
    struct Failing;
    impl Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other(
                "private path and parser detail must never escape",
            ))
        }
    }
    let root = tempfile::tempdir().unwrap();
    let manager = ProjectManager::new(root.path()).unwrap();
    let task = task();
    let mut adapter = OneShotAdapter::new(task.task_id);
    let error = adapter
        .execute(&task, &mut Failing, &manager, alive)
        .unwrap_err();
    assert_eq!(error, TaskError::ReadFailed);
    assert_eq!(serde_json::to_string(&error).unwrap(), "\"READ_FAILED\"");
    assert_eq!(
        adapter
            .execute(&task, &mut Unreadable, &manager, alive)
            .unwrap_err(),
        TaskError::AlreadyConsumed
    );
    empty(&manager);
}
