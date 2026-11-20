use super::*;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!(
            "kraqel-isolation-controller-{}",
            Uuid::new_v4().simple()
        ));
        fs::create_dir(&base).unwrap();
        fs::create_dir(base.join("case-00")).unwrap();
        Self(base)
    }
    fn root(&self) -> PathBuf {
        self.0.join("case-00/store")
    }
    fn bound(&self) -> Store {
        let mut store = Store::initialize(&self.root(), [1; 32]).unwrap();
        for from in 1..5 {
            store
                .append(
                    [1; 32],
                    store.state.head,
                    Action::Migrate { from, to: from + 1 },
                )
                .unwrap();
        }
        let app = std::env::current_exe().unwrap();
        let expected = hash(&app).unwrap();
        store
            .bind_workspace([1; 32], store.state.head, &app, expected)
            .unwrap();
        store
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        assert_eq!(
            self.0.parent().unwrap().canonicalize().unwrap(),
            std::env::temp_dir().canonicalize().unwrap()
        );
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn records_reconstruct_consumed_launches_and_preserve_workspace_without_tasks() {
    let f = Fixture::new();
    let mut store = f.bound();
    let key = hash(&f.root().join("control/authority.key")).unwrap();
    let binding = store.state.workspace.as_ref().unwrap().binding().clone();
    let proposal = store.workspace_proposal(store.state.head, false).unwrap();
    let first = proposal.launch_id;
    store
        .append(
            [1; 32],
            store.state.head,
            Action::CommitWorkspaceLaunch { proposal },
        )
        .unwrap();
    assert!(Store::open(&f.root()).is_err());
    drop(store);
    let mut store = Store::open(&f.root()).unwrap();
    store.verify_workspace_files().unwrap();
    let model = store.state.workspace.as_ref().unwrap();
    let launch = model.pending_launch().unwrap();
    assert_eq!(launch.id, first);
    assert_eq!(model.generation(), 1);
    store
        .append(
            [1; 32],
            store.state.head,
            Action::ReconcileWorkspaceLaunch { launch },
        )
        .unwrap();
    drop(store);
    let mut store = Store::open(&f.root()).unwrap();
    let head = store.state.head;
    let replay = store.workspace_proposal(head, true).unwrap();
    assert!(
        store
            .append(
                [1; 32],
                head,
                Action::CommitWorkspaceLaunch { proposal: replay }
            )
            .is_err()
    );
    assert_eq!(store.state.head, head);
    let proposal = store.workspace_proposal(head, false).unwrap();
    assert_ne!(proposal.launch_id, first);
    store
        .append([1; 32], head, Action::CommitWorkspaceLaunch { proposal })
        .unwrap();
    drop(store);
    let store = Store::open(&f.root()).unwrap();
    let model = store.state.workspace.as_ref().unwrap();
    assert_eq!(model.generation(), 2);
    assert_eq!(model.binding(), &binding);
    assert_eq!(hash(&f.root().join("control/authority.key")).unwrap(), key);
    assert_eq!(
        fs::read(store.workspace_root().join("sentinel.txt")).unwrap(),
        SENTINEL
    );
    assert!(!store.workspace_root().join("tasks").exists());
    assert!(store.state.session.is_none());
}

#[test]
fn wrong_actor_head_binding_and_changed_files_never_advance_control() {
    let f = Fixture::new();
    let mut store = f.bound();
    let head = store.state.head;
    let key = hash(&f.root().join("control/authority.key")).unwrap();
    let proposal = store.workspace_proposal(head, false).unwrap();
    assert!(
        store
            .append(
                [2; 32],
                head,
                Action::CommitWorkspaceLaunch {
                    proposal: proposal.clone()
                }
            )
            .is_err()
    );
    assert!(
        store
            .append(
                [1; 32],
                [0; 32],
                Action::CommitWorkspaceLaunch {
                    proposal: proposal.clone()
                }
            )
            .is_err()
    );
    let mut bad = proposal;
    bad.binding.workspace_root_sha256 = [99; 32];
    assert!(
        store
            .append(
                [1; 32],
                head,
                Action::CommitWorkspaceLaunch { proposal: bad }
            )
            .is_err()
    );
    assert_eq!(store.state.head, head);
    assert_eq!(
        fs::read_dir(f.root().join("control/records"))
            .unwrap()
            .count(),
        6
    );
    fs::write(store.workspace_root().join("policy.json"), b"changed").unwrap();
    assert!(store.verify_workspace_files().is_err());
    assert_eq!(hash(&f.root().join("control/authority.key")).unwrap(), key);
    let app = std::env::current_exe().unwrap();
    assert!(
        store
            .bind_workspace([1; 32], head, &app, hash(&app).unwrap())
            .is_err()
    );
    assert_eq!(store.state.head, head);
}

#[test]
fn authenticated_unpublished_tail_and_missing_key_stay_blocked_without_repair() {
    for missing_key in [false, true] {
        let f = Fixture::new();
        let store = f.bound();
        let head = hash(&f.root().join("control/head.mac")).unwrap();
        if missing_key {
            fs::remove_file(f.root().join("control/authority.key")).unwrap();
        } else {
            let record = Record {
                version: 1,
                sequence: store.state.sequence + 1,
                installation: store.state.installation,
                root_sha256: root_hash(&store.root),
                previous_sha256: store.state.head,
                operation: Uuid::new_v4(),
                actor: [1; 32],
                action: Action::CommitWorkspaceLaunch {
                    proposal: store.workspace_proposal(store.state.head, false).unwrap(),
                },
            };
            write_record(
                &store.root,
                record.sequence,
                &sealed(&serde_json::to_vec(&record).unwrap(), &store.key).unwrap(),
            )
            .unwrap();
        }
        drop(store);
        for _ in 0..2 {
            assert!(Store::open(&f.root()).is_err());
            assert_eq!(hash(&f.root().join("control/head.mac")).unwrap(), head);
        }
        if missing_key {
            assert!(!f.root().join("control/authority.key").exists());
        }
    }
}
