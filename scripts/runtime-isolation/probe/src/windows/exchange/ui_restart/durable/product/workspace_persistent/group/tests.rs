use super::*;

fn sample() -> Grant {
    Grant {
        version: 1,
        purpose: "c3-synthetic-group".into(),
        role: "root".into(),
        parent_pid: 20,
        guardian: Identity {
            pid: 20,
            created: 1,
        },
        launch: crate::product_start_p2c::LaunchIdentity {
            id: [1; 16],
            generation: 1,
        },
        epoch: Uuid::new_v4(),
        nonce: [2; 32],
        workspace: Uuid::new_v4(),
        root: PathBuf::from("C:/synthetic"),
        profile_name: "synthetic".into(),
        code: Code {
            root: contract::RootIdentityV1 {
                canonical_path_sha256: [3; 32],
                volume_id: 1,
                file_id: [4; 16],
            },
            sha256: [5; 32],
            size: 42,
        },
        binding_sha256: [6; 32],
        data_sha256: [7; 32],
        bad_leaf: false,
    }
}
#[test]
fn c3_frames_bind_group_role_nonce_sequence_and_reject_malformed_input() {
    let grant = sample();
    let key = [9; 32];
    let wire = encode(&grant, &key).unwrap();
    assert_eq!(decode::<Grant>(&wire, &key).unwrap(), grant);
    assert!(decode::<Grant>(&wire, &[8; 32]).is_err());
    let mut changed = wire.clone();
    changed[36] ^= 1;
    assert!(decode::<Grant>(&changed, &key).is_err());
    let mut trailing = wire;
    trailing.push(0);
    assert!(decode::<Grant>(&trailing, &key).is_err());
    let mut unknown = serde_json::to_value(&grant).unwrap();
    unknown["importTask"] = json!(true);
    assert!(decode::<Grant>(&encode(&unknown, &key).unwrap(), &key).is_err());
    assert!(decode::<Grant>(&[0; 36], &key).is_err());
    let receipt = message(
        &grant,
        "accepted",
        1,
        Identity {
            pid: 21,
            created: 2,
        },
        0,
    )
    .unwrap();
    assert!(verify_message(&receipt, &grant, "accepted", 1).is_ok());
    assert!(verify_message(&receipt, &grant, "continue", 2).is_err());
    let mut other = grant.clone();
    other.nonce[0] ^= 1;
    assert!(verify_message(&receipt, &other, "accepted", 1).is_err());
    other = grant.clone();
    other.role = "leaf".into();
    assert!(verify_message(&receipt, &other, "accepted", 1).is_err());
}

#[test]
fn c3_pipe_deadlines_and_framing_do_not_wait_for_descendant_eof() {
    let grant = sample();
    let key = [9; 32];
    let a = encode(&grant, &key).unwrap();
    let (mut reader, mut writer) = pipe().unwrap();
    assert!(read_frame(&mut reader, Instant::now()).is_err());
    writer.write_all(&a).unwrap();
    writer.write_all(&a).unwrap();
    // Writer stays open: exactly one bounded frame is consumed, not the entire stream/EOF.
    for _ in 0..2 {
        assert_eq!(
            read_frame(&mut reader, Instant::now() + Duration::from_secs(1)).unwrap(),
            a
        );
    }
    writer.write_all(&u32::MAX.to_le_bytes()).unwrap();
    assert!(read_frame(&mut reader, Instant::now() + Duration::from_secs(1)).is_err());
}

fn synthetic_state() -> State {
    let grant = sample();
    let root = grant.code.root.clone();
    let binding = WorkspaceBindingV2 {
        version: 2,
        control_schema: 6,
        ui_abi: 1,
        installation_id: [1; 16],
        workspace_id: [2; 16],
        profile_name: "synthetic".into(),
        profile_sid: "S-1-15-2-1-2-3-4-5-6-7".into(),
        projects: root.clone(),
        webview: RootIdentityV1 {
            file_id: [5; 16],
            canonical_path_sha256: [5; 32],
            ..root.clone()
        },
        session_root: RootIdentityV1 {
            file_id: [6; 16],
            canonical_path_sha256: [6; 32],
            ..root
        },
        host_package_sha256: [1; 32],
        ui_package_sha256: [2; 32],
        config_sha256: [3; 32],
        commands_sha256: [4; 32],
        job_policy_sha256: [5; 32],
        acl_policy_sha256: [6; 32],
        workspace_format: 1,
        db_read_min: 6,
        db_read_max: 6,
        db_write: 6,
        max_processes: 2,
        revision: 1,
    };
    State {
        installation: Uuid::from_bytes([1; 16]),
        sequence: 3,
        head: [3; 32],
        actor: grant.code.sha256,
        schema: 6,
        pending: None,
        window_session: Default::default(),
        win32_anchor: None,
        activated: Default::default(),
        aborted_packages: Default::default(),
        session: None,
        workspace: None,
        probe: None,
        c2: Some(Persistent {
            id: Uuid::from_bytes([2; 16]),
            profile_ready: true,
            binding: Some(binding),
            roots: None,
            retiring: false,
            retired: false,
        }),
        c3: Default::default(),
        c4_probe: Default::default(),
        bp2_probe: Default::default(),
    }
}
#[test]
fn c3_replay_pending_and_group_end_order_never_create_cleanup_authority() {
    let mut s = synthetic_state();
    let g = sample();
    let actor = s.actor;
    apply(
        &mut s,
        actor,
        &GroupEvent::CodeBound {
            code: g.code.clone(),
        },
    )
    .unwrap();
    let commit = GroupEvent::Commit {
        launch: g.launch,
        epoch: g.epoch,
        guardian: g.guardian,
        nonce: g.nonce,
    };
    apply(&mut s, actor, &commit).unwrap();
    assert!(s.c3.pending());
    assert!(apply(&mut s, actor, &commit).is_err());
    assert!(super::super::apply(&mut s, actor, &Action::RetireC2Workspace).is_err());
    assert!(apply(&mut s, actor, &GroupEvent::Finish { passed: true }).is_err());
    assert!(apply(&mut s, actor, &GroupEvent::Accepted).is_err());
    apply(
        &mut s,
        actor,
        &GroupEvent::Staged {
            root: g.code.root.clone(),
        },
    )
    .unwrap();
    apply(
        &mut s,
        actor,
        &GroupEvent::RootBound {
            identity: Identity {
                pid: 21,
                created: 2,
            },
        },
    )
    .unwrap();
    apply(&mut s, actor, &GroupEvent::Accepted).unwrap();
    apply(&mut s, actor, &GroupEvent::Running).unwrap();
    apply(
        &mut s,
        actor,
        &GroupEvent::ChildBound {
            identity: Identity {
                pid: 22,
                created: 3,
            },
        },
    )
    .unwrap();
    // Simulated Root end creates no transition. Only the native guardian can attest Drained.
    assert!(apply(&mut s, actor, &GroupEvent::Finish { passed: true }).is_err());
    apply(&mut s, actor, &GroupEvent::Drained).unwrap();
    apply(&mut s, actor, &GroupEvent::Finish { passed: true }).unwrap();
    assert!(!s.c3.pending());
    assert!(apply(&mut s, actor, &commit).is_err());
    let mut next = g.launch;
    next.id = [8; 16];
    next.generation = 2;
    apply(
        &mut s,
        actor,
        &GroupEvent::Commit {
            launch: next,
            epoch: Uuid::new_v4(),
            guardian: g.guardian,
            nonce: [8; 32],
        },
    )
    .unwrap();
    assert!(s.c3.pending());
}
#[test]
fn c3_declaration_is_separate_from_ui_and_pending_is_not_native_recovery() {
    let (_pins, mut file, _hash) = binary_pin(&std::env::current_exe().unwrap()).unwrap();
    package::verify_group(&mut file).unwrap();
    let mut s = synthetic_state();
    let g = sample();
    let actor = s.actor;
    apply(&mut s, actor, &GroupEvent::CodeBound { code: g.code }).unwrap();
    apply(
        &mut s,
        actor,
        &GroupEvent::Commit {
            launch: g.launch,
            epoch: g.epoch,
            guardian: g.guardian,
            nonce: g.nonce,
        },
    )
    .unwrap();
    // A copied/replayed pending model does not have the same live guardian's native identity.
    assert!(fresh_guardian(s.c3.pending.as_ref().unwrap()).is_err());
}
