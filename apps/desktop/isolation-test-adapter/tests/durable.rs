use kraqel_isolation_test_adapter::{
    DurableStore, FileOperation, FileTask, SYNTHETIC_ARCHIVE, TaskControl, TaskError, TaskMilestone,
};
use kraqel_storage::project_manager::ProjectManager;
use std::{
    fs,
    io::{self, Read},
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;

const ID: &str = "22222222-2222-4222-8222-222222222222";
fn task() -> FileTask {
    FileTask {
        contract_version: 1,
        task_id: Uuid::parse_str(ID).unwrap(),
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
struct Unreadable;
impl Read for Unreadable {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("replay must never read");
    }
}
fn open(root: &Path) -> DurableStore {
    fs::create_dir_all(root.join("projects")).unwrap();
    DurableStore::open(
        &root.join("tasks"),
        &ProjectManager::new(root.join("projects")).unwrap(),
    )
    .unwrap()
}

#[test]
fn durable_success_survives_reopen_and_never_reimports_the_same_id() {
    let root = tempfile::tempdir().unwrap();
    let task = task();
    let mut store = open(root.path());
    store.claim(task.task_id).unwrap();
    let receipt = store
        .execute(
            &task,
            &mut io::Cursor::new(SYNTHETIC_ARCHIVE),
            alive,
            alive,
            |_| {},
        )
        .unwrap();
    assert_eq!(receipt.document_count, 2);
    drop(store);
    let mut store = open(root.path());
    assert_eq!(store.claim(task.task_id), Err(TaskError::AlreadyConsumed));
    assert_eq!(
        store
            .execute(&task, &mut Unreadable, alive, alive, |_| {})
            .unwrap_err(),
        TaskError::AlreadyConsumed
    );
    assert_eq!(
        store.outcome(task.task_id).unwrap().unwrap().project_id,
        receipt.project_id
    );
    assert_eq!(
        fs::read_dir(root.path().join("projects")).unwrap().count(),
        1
    );
    assert_eq!(
        fs::read_dir(root.path().join("tasks/staging"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn refusal_expiry_invalid_stream_and_mismatched_id_stay_consumed() {
    for case in 0..5 {
        let root = tempfile::tempdir().unwrap();
        let mut store = open(root.path());
        let mut task = task();
        store.claim(task.task_id).unwrap();
        let error = match case {
            0 => {
                store.refuse(TaskError::Cancelled).unwrap();
                TaskError::Cancelled
            }
            1 => store
                .execute(
                    &task,
                    &mut Unreadable,
                    || TaskControl {
                        now_unix_ms: 61000,
                        cancelled: false,
                    },
                    alive,
                    |_| {},
                )
                .unwrap_err(),
            2 => {
                task.byte_count = 3;
                store
                    .execute(&task, &mut &b"bad"[..], alive, alive, |_| {})
                    .unwrap_err()
            }
            3 => {
                task.task_id = Uuid::new_v4();
                store
                    .execute(&task, &mut Unreadable, alive, alive, |_| {})
                    .unwrap_err()
            }
            _ => store
                .execute(
                    &task,
                    &mut io::Cursor::new(SYNTHETIC_ARCHIVE),
                    alive,
                    || TaskControl {
                        now_unix_ms: 1001,
                        cancelled: true,
                    },
                    |_| {},
                )
                .unwrap_err(),
        };
        drop(store);
        let mut store = open(root.path());
        assert_eq!(
            store.claim(crate::task().task_id),
            Err(TaskError::AlreadyConsumed)
        );
        assert_eq!(
            store.outcome(crate::task().task_id).unwrap().unwrap_err(),
            error
        );
        assert_eq!(
            fs::read_dir(root.path().join("projects")).unwrap().count(),
            0
        );
        assert_eq!(
            fs::read_dir(root.path().join("tasks/staging"))
                .unwrap()
                .count(),
            0
        );
    }
}

#[test]
fn exclusive_owner_and_project_root_binding_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    let store = open(root.path());
    let manager = ProjectManager::new(root.path().join("projects")).unwrap();
    assert!(matches!(
        DurableStore::open(&root.path().join("tasks"), &manager),
        Err(TaskError::StateUnavailable)
    ));
    drop(store);
    fs::create_dir(root.path().join("other-projects")).unwrap();
    let other = ProjectManager::new(root.path().join("other-projects")).unwrap();
    assert!(matches!(
        DurableStore::open(&root.path().join("tasks"), &other),
        Err(TaskError::StateUnavailable)
    ));
}

#[test]
fn late_refusal_and_same_instance_retry_cannot_erase_commit_intent() {
    for stop in [TaskMilestone::CommitIntent, TaskMilestone::Published] {
        let root = tempfile::tempdir().unwrap();
        let mut store = open(root.path());
        store.claim(task().task_id).unwrap();
        let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            store.execute(
                &task(),
                &mut io::Cursor::new(SYNTHETIC_ARCHIVE),
                alive,
                alive,
                |point| {
                    assert!(point != stop, "synthetic interrupted return path");
                },
            )
        }));
        assert!(interrupted.is_err());
        assert_eq!(
            store.refuse(TaskError::Cancelled),
            Err(TaskError::AlreadyConsumed)
        );
        assert_eq!(
            store
                .execute(&task(), &mut Unreadable, alive, alive, |_| {})
                .unwrap_err(),
            TaskError::AlreadyConsumed
        );
        drop(store);
        let store = open(root.path());
        assert_eq!(
            store.outcome(task().task_id).unwrap().is_ok(),
            stop == TaskMilestone::Published
        );
    }
}

#[test]
fn corrupt_unknown_and_ambiguous_state_is_preserved_without_guessing() {
    for case in 0..4 {
        let root = tempfile::tempdir().unwrap();
        drop(open(root.path()));
        let ledger = root.path().join("tasks/tasks.sqlite3");
        match case {
            0 => fs::write(&ledger, b"damaged journal").unwrap(),
            1 => {
                let db = rusqlite::Connection::open(&ledger).unwrap();
                db.execute_batch("PRAGMA user_version=99").unwrap();
            }
            2 => {
                fs::create_dir(root.path().join("tasks/staging/unowned")).unwrap();
            }
            _ => {
                let db = rusqlite::Connection::open(&ledger).unwrap();
                db.execute(
                    "INSERT INTO task(id,state,receipt) VALUES (?1,'committing','{}')",
                    [ID],
                )
                .unwrap();
            }
        }
        let before = fs::read(&ledger).unwrap();
        let manager = ProjectManager::new(root.path().join("projects")).unwrap();
        assert!(matches!(
            DurableStore::open(&root.path().join("tasks"), &manager),
            Err(TaskError::StateUnavailable)
        ));
        assert_eq!(fs::read(ledger).unwrap(), before);
        assert_eq!(fs::read_dir(manager.root()).unwrap().count(), 0);
    }
}

fn checkpoint(root: &Path, wanted: &str, current: &str) {
    if wanted == current {
        fs::write(root.join("checkpoint"), current).unwrap();
        loop {
            thread::sleep(Duration::from_millis(50));
        }
    }
}

struct Source<'a> {
    inner: io::Cursor<&'a [u8]>,
    root: &'a Path,
    wanted: &'a str,
}
impl Read for Source<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let bound = buffer.len().min(512);
        let count = self.inner.read(&mut buffer[..bound])?;
        checkpoint(self.root, self.wanted, "reading");
        Ok(count)
    }
}
/// Invoked only by the integration-test parent; not a product executable or runtime switch.
#[test]
#[ignore = "subprocess helper for deterministic process-kill tests"]
fn crash_child() {
    let root = std::path::PathBuf::from(std::env::var_os("KRAQEL_TEST_CRASH_ROOT").unwrap());
    let wanted = std::env::var("KRAQEL_TEST_CRASH_POINT").unwrap();
    let mut store = open(&root);
    let task = task();
    store.claim(task.task_id).unwrap();
    checkpoint(&root, &wanted, "claimed");
    let stage = root.join("tasks/staging").join(ID);
    store
        .execute(
            &task,
            &mut Source {
                inner: io::Cursor::new(SYNTHETIC_ARCHIVE),
                root: &root,
                wanted: &wanted,
            },
            || {
                if fs::read_dir(&stage).is_ok_and(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .any(|e| e.path().join("kraqel.sqlite3").is_file())
                }) {
                    checkpoint(&root, &wanted, "initializing");
                }
                alive()
            },
            alive,
            |phase| {
                checkpoint(
                    &root,
                    &wanted,
                    match phase {
                        TaskMilestone::Prepared => "prepared",
                        TaskMilestone::CommitIntent => "intent",
                        TaskMilestone::Published => "published",
                    },
                );
            },
        )
        .unwrap();
    checkpoint(&root, &wanted, "complete");
}

#[test]
fn actual_process_kills_reconcile_every_publication_boundary_idempotently() {
    for phase in [
        "claimed",
        "reading",
        "initializing",
        "prepared",
        "intent",
        "published",
        "complete",
    ] {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("projects")).unwrap();
        let manager = ProjectManager::new(root.path().join("projects")).unwrap();
        let baseline_id = kraqel_domain::ProjectId::new();
        let baseline = manager
            .create_project(
                baseline_id.to_string(),
                &kraqel_domain::Project {
                    id: baseline_id,
                    name: "Unberührtes Bestandsprojekt".into(),
                    schema_version: kraqel_domain::CURRENT_SCHEMA_VERSION,
                },
            )
            .unwrap();
        let baseline_db = baseline.directory().join("kraqel.sqlite3");
        drop(baseline);
        let baseline_bytes = fs::read(&baseline_db).unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "crash_child", "--ignored", "--nocapture"])
            .env("KRAQEL_TEST_CRASH_ROOT", root.path())
            .env("KRAQEL_TEST_CRASH_POINT", phase)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt as _;
            command.creation_flags(0x0800_0000);
        }
        let mut child = command.spawn().unwrap();
        let start = Instant::now();
        while !root.path().join("checkpoint").exists() {
            assert!(
                child.try_wait().unwrap().is_none(),
                "child exited before {phase}"
            );
            if start.elapsed() > Duration::from_secs(20) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("checkpoint timeout: {phase}");
            }
            thread::sleep(Duration::from_millis(20));
        }
        let manager = ProjectManager::new(root.path().join("projects")).unwrap();
        assert!(
            matches!(
                DurableStore::open(&root.path().join("tasks"), &manager),
                Err(TaskError::StateUnavailable)
            ),
            "active owner: {phase}"
        );
        child.kill().unwrap();
        child.wait().unwrap();
        let published = phase == "published" || phase == "complete";
        for _ in 0..2 {
            let mut store = open(root.path());
            assert_eq!(
                store.claim(task().task_id),
                Err(TaskError::AlreadyConsumed),
                "{phase}"
            );
            let outcome = store.outcome(task().task_id).unwrap();
            if published {
                assert_eq!(outcome.unwrap().document_count, 2);
            } else {
                assert_eq!(outcome.unwrap_err(), TaskError::Interrupted, "{phase}");
            }
            assert_eq!(
                fs::read_dir(root.path().join("projects")).unwrap().count(),
                1 + usize::from(published),
                "{phase}"
            );
            assert_eq!(
                fs::read(&baseline_db).unwrap(),
                baseline_bytes,
                "{phase}: existing project unchanged"
            );
            assert_eq!(
                fs::read_dir(root.path().join("tasks/staging"))
                    .unwrap()
                    .count(),
                0,
                "{phase}"
            );
        }
    }
}

#[test]
fn missing_journal_and_corrupt_published_copy_never_enable_replay_or_delete_data() {
    for missing in [true, false] {
        let root = tempfile::tempdir().unwrap();
        let mut store = open(root.path());
        store.claim(task().task_id).unwrap();
        let receipt = store
            .execute(
                &task(),
                &mut io::Cursor::new(SYNTHETIC_ARCHIVE),
                alive,
                alive,
                |_| {},
            )
            .unwrap();
        drop(store);
        let ledger = root.path().join("tasks/tasks.sqlite3");
        let database = root
            .path()
            .join("projects")
            .join(receipt.project_id.to_string())
            .join("kraqel.sqlite3");
        if missing {
            fs::remove_file(ledger).unwrap();
        } else {
            let db = rusqlite::Connection::open(ledger).unwrap();
            db.execute("UPDATE task SET state='committing'", [])
                .unwrap();
            drop(db);
            fs::write(&database, b"ambiguous published project").unwrap();
        }
        let before = fs::read(&database).unwrap();
        let manager = ProjectManager::new(root.path().join("projects")).unwrap();
        assert!(matches!(
            DurableStore::open(&root.path().join("tasks"), &manager),
            Err(TaskError::StateUnavailable)
        ));
        assert_eq!(fs::read(database).unwrap(), before);
        assert_eq!(fs::read_dir(manager.root()).unwrap().count(), 1);
    }
}

#[cfg(unix)]
#[test]
fn reconciliation_refuses_links_and_preserves_the_external_target() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let mut store = open(root.path());
    store.claim(task().task_id).unwrap();
    drop(store);
    let external = tempfile::tempdir().unwrap();
    fs::write(external.path().join("keep"), b"private unrelated data").unwrap();
    symlink(external.path(), root.path().join("tasks/staging").join(ID)).unwrap();
    let manager = ProjectManager::new(root.path().join("projects")).unwrap();
    assert!(matches!(
        DurableStore::open(&root.path().join("tasks"), &manager),
        Err(TaskError::StateUnavailable)
    ));
    assert_eq!(
        fs::read(external.path().join("keep")).unwrap(),
        b"private unrelated data"
    );
}
