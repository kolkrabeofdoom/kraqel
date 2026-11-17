use super::*;
fn fixture() -> (PathBuf, Store) {
    let root = std::env::temp_dir().join(format!(
        "kraqel-isolation-controller-{}",
        Uuid::new_v4().simple()
    ));
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("case-00")).unwrap();
    let mut store = Store::initialize(&root.join("case-00/store"), [1; 32]).unwrap();
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
    let sha = hash(&app).unwrap();
    store
        .bind_workspace([1; 32], store.state.head, &app, sha)
        .unwrap();
    (root, store)
}
fn commit(store: &mut Store) -> LaunchIdentity {
    let proposal = store.workspace_proposal(store.state.head, false).unwrap();
    let execution = Execution {
        version: 2,
        probe_id: Uuid::new_v4(),
        workspace_id: proposal.binding.workspace_id,
        nonce: [2; 32],
        worker_sha256: [1; 32],
        root_sha256: [3; 32],
        policy_sha256: Sha256::digest(POLICY).into(),
        sentinel_sha256: Sha256::digest(SENTINEL).into(),
    };
    store
        .append(
            [1; 32],
            store.state.head,
            Action::CommitWorkspaceProbe {
                proposal,
                execution,
            },
        )
        .unwrap();
    store.state.probe.as_ref().unwrap().launch
}
fn clean(root: PathBuf, store: Store) {
    drop(store);
    assert_eq!(
        root.parent().unwrap().canonicalize().unwrap(),
        std::env::temp_dir().canonicalize().unwrap()
    );
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn ready_record_and_receipt_settlement_survive_authenticated_reopen_without_replay() {
    let (root, mut store) = fixture();
    let binding = store.state.workspace.as_ref().unwrap().binding().clone();
    let key = hash(&store.root.join("control/authority.key")).unwrap();
    let launch = commit(&mut store);
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::FinishWorkspaceProbe {
                    launch,
                    receipt_sha256: [4; 32]
                }
            )
            .is_err()
    );
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::ReconcileWorkspaceProbe { launch }
            )
            .is_err()
    );
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::ReconcileWorkspaceLaunch { launch }
            )
            .is_err()
    );
    store
        .append(
            [1; 32],
            store.state.head,
            Action::WorkspaceProbeProfileReady { launch },
        )
        .unwrap();
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::WorkspaceProbeProfileReady { launch }
            )
            .is_err()
    );
    let path = store.root.clone();
    drop(store);
    let mut store = Store::open(&path).unwrap();
    assert!(store.state.probe.as_ref().unwrap().ready);
    store
        .append(
            [1; 32],
            store.state.head,
            Action::WorkspaceProbeRunIntent { launch },
        )
        .unwrap();
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::ReconcileWorkspaceProbe { launch }
            )
            .is_err()
    );
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::FinishWorkspaceProbe {
                    launch,
                    receipt_sha256: [4; 32]
                }
            )
            .is_err()
    );
    store
        .append(
            [1; 32],
            store.state.head,
            Action::WorkspaceProbeDrained { launch },
        )
        .unwrap();
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::FinishWorkspaceProbe {
                    launch,
                    receipt_sha256: [0; 32]
                }
            )
            .is_err()
    );
    store
        .append(
            [1; 32],
            store.state.head,
            Action::FinishWorkspaceProbe {
                launch,
                receipt_sha256: [4; 32],
            },
        )
        .unwrap();
    drop(store);
    let mut store = Store::open(&path).unwrap();
    assert!(store.state.probe.is_none());
    assert_eq!(store.state.workspace.as_ref().unwrap().binding(), &binding);
    assert_eq!(hash(&path.join("control/authority.key")).unwrap(), key);
    let proposal = store.workspace_proposal(store.state.head, true).unwrap();
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::CommitWorkspaceLaunch { proposal }
            )
            .is_err()
    );
    clean(root, store);
}
#[test]
fn resource_reconciliation_reopens_uncreated_and_ready_states_but_blocks_unproved_run() {
    let (root, mut store) = fixture();
    let path = store.root.clone();
    let first = commit(&mut store);
    let stale = store.state.head;
    store.reconcile_probe_resources([1; 32], stale).unwrap();
    assert!(store.reconcile_probe_resources([1; 32], stale).is_err());
    let second = commit(&mut store);
    assert_ne!(first.id, second.id);
    store
        .append(
            [1; 32],
            store.state.head,
            Action::WorkspaceProbeProfileReady { launch: second },
        )
        .unwrap();
    let head = store.state.head;
    assert!(store.reconcile_probe_resources([9; 32], head).is_err());
    assert!(store.reconcile_probe_resources([1; 32], stale).is_err());
    store
        .append(
            [1; 32],
            head,
            Action::WorkspaceProbeRunIntent { launch: second },
        )
        .unwrap();
    drop(store);
    let mut store = Store::open(&path).unwrap();
    let head = store.state.head;
    assert!(store.reconcile_probe_resources([1; 32], head).is_err());
    assert_eq!(store.state.head, head);
    store
        .append(
            [1; 32],
            head,
            Action::WorkspaceProbeDrained { launch: second },
        )
        .unwrap();
    drop(store);
    let mut store = Store::open(&path).unwrap();
    store
        .reconcile_probe_resources([1; 32], store.state.head)
        .unwrap();
    assert_eq!(store.state.workspace.as_ref().unwrap().generation(), 2);
    assert!(store.state.probe.is_none());
    clean(root, store);
}
#[test]
fn interruption_preserves_launch_history_and_next_generation() {
    let (root, mut store) = fixture();
    let launch = commit(&mut store);
    let mut wrong = launch;
    wrong.generation += 1;
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::WorkspaceProbeProfileReady { launch: wrong }
            )
            .is_err()
    );
    store
        .append(
            [1; 32],
            store.state.head,
            Action::WorkspaceProbeProfileReady { launch },
        )
        .unwrap();
    store
        .append(
            [1; 32],
            store.state.head,
            Action::ReconcileWorkspaceProbe { launch },
        )
        .unwrap();
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::ReconcileWorkspaceProbe { launch }
            )
            .is_err()
    );
    let next = commit(&mut store);
    assert_eq!(next.generation, launch.generation + 1);
    assert_ne!(next.id, launch.id);
    clean(root, store);
}
