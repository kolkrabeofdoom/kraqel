use super::*;
use serde_json::{Value, json};

fn entry(path: &str) -> InventoryEntry {
    InventoryEntry {
        path: path.into(),
        size: 0,
        sha256: [1; 32],
        role: FileRole::Resource,
    }
}
fn root(n: u8) -> RootIdentityV1 {
    RootIdentityV1 {
        canonical_path_sha256: [n; 32],
        volume_id: 1,
        file_id: [n; 16],
    }
}
fn binding() -> WorkspaceBindingV2 {
    WorkspaceBindingV2 {
        version: 2,
        control_schema: 6,
        ui_abi: 1,
        installation_id: [1; 16],
        workspace_id: [2; 16],
        profile_name: "kraqel-synthetic-p2c".into(),
        profile_sid: "S-1-15-2-1-2-3-4-5-6-7".into(),
        projects: root(3),
        webview: root(4),
        session_root: root(5),
        host_package_sha256: [6; 32],
        ui_package_sha256: [7; 32],
        config_sha256: [8; 32],
        commands_sha256: [9; 32],
        job_policy_sha256: [10; 32],
        acl_policy_sha256: [11; 32],
        workspace_format: 1,
        db_read_min: 6,
        db_read_max: 6,
        db_write: 6,
        max_processes: 8,
        revision: 1,
    }
}
fn proposal() -> UiStartProposalV1 {
    UiStartProposalV1 {
        version: 1,
        purpose: Purpose::WorkspaceUiStart,
        binding: binding(),
        launch: LaunchIdentity {
            id: [12; 16],
            generation: 1,
        },
        expected_head: [13; 32],
        timeout_ms: 30_000,
    }
}
fn parse(p: &UiStartProposalV1) -> UntrustedProposal {
    parse_proposal(&serde_json::to_vec(p).unwrap()).unwrap()
}
fn observation(count: Option<u32>) -> ModelObservation {
    ModelObservation {
        lease_held: true,
        control_consistent: true,
        job: count.map_or(ModelJob::NoGroup, |active_processes| ModelJob::Owned {
            job_id: [14; 16],
            guardian_epoch: [15; 16],
            active_processes,
        }),
    }
}
fn bound() -> UiRootBoundV1 {
    UiRootBoundV1 {
        launch: proposal().launch,
        job_id: [14; 16],
        guardian_epoch: [15; 16],
        pid: 42,
        creation_time: 12345,
    }
}
fn frame(p: UiStartProposalV1, role: BootstrapRole) -> BootstrapFrameV1 {
    BootstrapFrameV1 {
        domain: BOOTSTRAP_DOMAIN.into(),
        version: 1,
        sequence: match role {
            BootstrapRole::Grant => 0,
            BootstrapRole::Accepted => 1,
            BootstrapRole::Continue => 2,
        },
        role,
        grant: UiStartGrantV1 {
            proposal: p,
            job_id: [14; 16],
            guardian_epoch: [15; 16],
            nonce: [16; 32],
        },
    }
}
fn wire(f: BootstrapFrameV1) -> UntrustedFrame {
    parse_bootstrap(&serde_json::to_vec(&f).unwrap()).unwrap()
}
fn end() -> UiGroupDrainedV1 {
    UiGroupDrainedV1 {
        launch: bound().launch,
        job_id: bound().job_id,
        guardian_epoch: bound().guardian_epoch,
    }
}
fn model(records: u16) -> LifecycleModel {
    LifecycleModel::new(binding(), proposal().expected_head, records).unwrap()
}
fn committed() -> LifecycleModel {
    let mut m = model(12);
    m.commit(&parse(&proposal()), [17; 32], observation(None))
        .unwrap();
    m
}
fn ready() -> LifecycleModel {
    let mut m = committed();
    m.bind_root(
        bound(),
        &wire(frame(proposal(), BootstrapRole::Grant)),
        [17; 32],
        [18; 32],
        observation(Some(1)),
    )
    .unwrap();
    m
}
fn running(records: u16) -> LifecycleModel {
    let mut m = model(records);
    m.commit(&parse(&proposal()), [17; 32], observation(None))
        .unwrap();
    m.bind_root(
        bound(),
        &wire(frame(proposal(), BootstrapRole::Grant)),
        [17; 32],
        [18; 32],
        observation(Some(1)),
    )
    .unwrap();
    m.bootstrap(
        &wire(frame(proposal(), BootstrapRole::Accepted)),
        10,
        [18; 32],
        [19; 32],
        observation(Some(2)),
    )
    .unwrap();
    m.bootstrap(
        &wire(frame(proposal(), BootstrapRole::Continue)),
        20,
        [19; 32],
        [20; 32],
        observation(Some(2)),
    )
    .unwrap();
    m
}

#[test]
fn inventory_has_independent_golden_bytes_hash_and_roundtrip() {
    let e = entry("a.txt");
    let mut expected =
        b"KRAQEL-P2C-PACKAGESET\0\x01\x02\x01\x00\x00\x00\x05\x00\x00\x00a.txt".to_vec();
    expected.extend([0; 8]);
    expected.extend([1; 32]);
    expected.push(3);
    let bytes = encode_inventory(PackageRole::Ui, std::slice::from_ref(&e)).unwrap();
    assert_eq!(bytes, expected);
    let hash: String = inventory_digest(PackageRole::Ui, std::slice::from_ref(&e))
        .unwrap()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(
        hash,
        "78ea47a3fe725f1ad898cb2f5a94da69f5f6c100dfe12f998b9e55669b4e4e67"
    );
    assert_eq!(
        decode_inventory(&bytes).unwrap(),
        (PackageRole::Ui, vec![e])
    );
    assert_ne!(
        bytes,
        encode_inventory(PackageRole::Host, &[entry("a.txt")]).unwrap()
    );
}

#[test]
fn inventory_sorting_and_all_roles_are_bound() {
    let mut a = entry("ä/bild.png");
    a.role = FileRole::Program;
    let mut b = entry("a.dll");
    b.role = FileRole::Library;
    let mut c = entry("config.json");
    c.role = FileRole::Configuration;
    let bytes = encode_inventory(PackageRole::Ui, &[a.clone(), b.clone(), c.clone()]).unwrap();
    assert_eq!(
        bytes,
        encode_inventory(PackageRole::Ui, &[c.clone(), a.clone(), b.clone()]).unwrap()
    );
    assert_eq!(decode_inventory(&bytes).unwrap().1, vec![b, c, a]);
    let base = inventory_digest(PackageRole::Ui, &[entry("a")]).unwrap();
    for e in [
        InventoryEntry {
            size: 1,
            ..entry("a")
        },
        InventoryEntry {
            sha256: [2; 32],
            ..entry("a")
        },
        InventoryEntry {
            role: FileRole::Program,
            ..entry("a")
        },
        entry("b"),
    ] {
        assert_ne!(base, inventory_digest(PackageRole::Ui, &[e]).unwrap());
    }
}

#[test]
fn inventory_rejects_paths_aliases_and_file_directory_conflicts() {
    for p in [
        "",
        "/a",
        "a/",
        "a//b",
        "a/../b",
        "a/./b",
        "C:/a",
        "a\\b",
        "a:x",
        "a?b",
        "a\0b",
        "a. ",
        "aux.txt",
        "COM1",
        "lpt².log",
        "conout$",
        "e\u{301}.png",
    ] {
        assert!(
            encode_inventory(PackageRole::Ui, &[entry(p)]).is_err(),
            "{p:?}"
        );
    }
    for (a, b) in [
        ("a", "a"),
        ("a", "A"),
        ("Ä.png", "ä.png"),
        ("A", "a/b"),
        ("a/b", "A"),
    ] {
        assert_eq!(
            encode_inventory(PackageRole::Ui, &[entry(a), entry(b)]),
            Err(Rejection::Duplicate)
        );
    }
    assert!(encode_inventory(PackageRole::Ui, &[entry(&"x".repeat(1025))]).is_err());
    assert!(encode_inventory(PackageRole::Ui, &[entry(&vec!["a"; 33].join("/"))]).is_err());
}

#[test]
fn inventory_bounds_and_noncanonical_or_unknown_wire_are_rejected() {
    assert!(encode_inventory(PackageRole::Ui, &[]).is_err());
    assert!(encode_inventory(PackageRole::Ui, &vec![entry("a"); MAX_FILES + 1]).is_err());
    assert!(
        encode_inventory(
            PackageRole::Ui,
            &[InventoryEntry {
                size: MAX_FILE + 1,
                ..entry("a")
            }]
        )
        .is_err()
    );
    let large: Vec<_> = (0..5)
        .map(|i| InventoryEntry {
            size: MAX_FILE,
            ..entry(&i.to_string())
        })
        .collect();
    assert!(encode_inventory(PackageRole::Ui, &large).is_err());
    let long: Vec<_> = (0..4100)
        .map(|i| entry(&format!("{i:04}{}", "a".repeat(1020))))
        .collect();
    assert!(encode_inventory(PackageRole::Ui, &long).is_err());
    let good = encode_inventory(PackageRole::Ui, &[entry("a")]).unwrap();
    for n in 0..good.len() {
        assert!(decode_inventory(&good[..n]).is_err());
    }
    let mut bad = good.clone();
    bad.push(0);
    assert!(decode_inventory(&bad).is_err());
    let mut bad = good.clone();
    bad[INVENTORY_DOMAIN.len()] = 3;
    assert!(decode_inventory(&bad).is_err());
    let mut bad = good.clone();
    *bad.last_mut().unwrap() = 5;
    assert!(decode_inventory(&bad).is_err());
    let mut bad = good.clone();
    bad[INVENTORY_DOMAIN.len() + 1..INVENTORY_DOMAIN.len() + 5]
        .copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(decode_inventory(&bad).is_err());
    let mut unsorted = encode_inventory(PackageRole::Ui, &[entry("a"), entry("b")]).unwrap();
    let first = INVENTORY_DOMAIN.len() + 5;
    unsorted[first + 4] = b'b';
    unsorted[first + 46 + 4] = b'a';
    assert!(decode_inventory(&unsorted).is_err());
}

#[test]
fn proposal_shape_version_purpose_and_unknown_fields_are_strict() {
    let value = serde_json::to_value(proposal()).unwrap();
    for (key, bad) in [
        ("version", json!(2)),
        ("purpose", json!("workspace-start")),
        ("timeoutMs", json!(0)),
        ("timeoutMs", json!(30001)),
        ("importTask", json!("no")),
    ] {
        let mut v = value.clone();
        v[key] = bad;
        assert!(parse_proposal(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    for (key, bad) in [
        ("controlSchema", json!(5)),
        ("uiAbi", json!(0)),
        ("revision", json!(0)),
        ("dbWrite", json!(7)),
        ("maxProcesses", json!(1)),
        ("profileSid", json!("S-1-5-18")),
        ("profileName", json!("../profile")),
        ("taskId", json!("no")),
    ] {
        let mut v = value.clone();
        v["binding"][key] = bad;
        assert!(parse_proposal(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    let mut p = proposal();
    p.binding.webview = p.binding.projects.clone();
    assert!(parse_proposal(&serde_json::to_vec(&p).unwrap()).is_err());
    p.binding.webview.canonical_path_sha256 = [99; 32]; // same native identity, changed path claim
    assert!(parse_proposal(&serde_json::to_vec(&p).unwrap()).is_err());
    assert!(parse_proposal(&vec![b' '; MAX_FRAME + 1]).is_err());
    assert!(parse_proposal(br#"{"version":1,"version":1}"#).is_err());
}

#[test]
fn bootstrap_rejects_wrong_domain_sequence_and_nested_unknown_fields() {
    let value = serde_json::to_value(frame(proposal(), BootstrapRole::Accepted)).unwrap();
    for (key, bad) in [
        ("domain", json!("import")),
        ("version", json!(2)),
        ("sequence", json!(0)),
        ("extra", json!(true)),
    ] {
        let mut v = value.clone();
        v[key] = bad;
        assert!(parse_bootstrap(&serde_json::to_vec(&v).unwrap()).is_err());
    }
    let mut v = value;
    v["grant"]["authenticated"] = Value::Bool(true);
    assert!(parse_bootstrap(&serde_json::to_vec(&v).unwrap()).is_err());
    assert!(parse_bootstrap(&vec![b' '; MAX_FRAME + 1]).is_err());
}

#[test]
fn successful_lifecycle_keeps_workspace_profile_and_consumes_launch() {
    let mut m = running(12);
    m.request_close().unwrap();
    m.drain(end(), [20; 32], [21; 32], observation(Some(0)))
        .unwrap();
    m.finish([21; 32], [22; 32], observation(Some(0))).unwrap();
    assert_eq!(m.phase(), Phase::Idle);
    assert_eq!(m.binding, binding());
    assert_eq!(m.remaining, 6);
    assert!(m.used.contains(&proposal().launch.id));
    let mut next = proposal();
    next.expected_head = [22; 32];
    next.launch.generation = 2;
    let before = m.clone();
    assert_eq!(
        m.commit(&parse(&next), [23; 32], observation(None)),
        Err(Rejection::Replay)
    );
    assert_eq!(m, before);
    next.launch.id = [24; 16];
    m.commit(&parse(&next), [23; 32], observation(None))
        .unwrap();
    assert_eq!(m.generation, 2);
}

#[test]
fn failed_commit_is_atomic_for_stale_head_generation_binding_budget_and_lease() {
    for case in 0..7 {
        let mut m = model(if case == 3 { 5 } else { 6 });
        let mut p = proposal();
        let mut o = observation(None);
        match case {
            0 => p.expected_head = [99; 32],
            1 => p.launch.generation = 2,
            2 => p.binding.revision = 2,
            3 => {}
            4 => o.lease_held = false,
            5 => o.control_consistent = false,
            _ => o.job = ModelJob::Unknown,
        }
        let before = m.clone();
        assert!(m.commit(&parse(&p), [17; 32], o).is_err());
        assert_eq!(m, before);
    }
    let mut m = committed();
    let before = m.clone();
    assert!(
        m.commit(&parse(&proposal()), [99; 32], observation(None))
            .is_err()
    );
    assert_eq!(m, before);
}

#[test]
fn grant_tuple_nonce_deadline_and_bootstrap_order_are_checked_atomically() {
    for case in 0..5 {
        let mut m = committed();
        let mut root = bound();
        let mut count = 1;
        let mut grant = frame(proposal(), BootstrapRole::Grant);
        match case {
            0 => root.launch.generation = 2,
            1 => root.pid = 0,
            2 => count = 0,
            3 => count = 2,
            _ => grant.grant.proposal.binding.config_sha256 = [99; 32],
        }
        let before = m.clone();
        assert!(
            m.bind_root(
                root,
                &wire(grant),
                [17; 32],
                [18; 32],
                observation(Some(count))
            )
            .is_err()
        );
        assert_eq!(m, before);
    }
    let mut m = ready();
    for case in 0..6 {
        let mut f = frame(proposal(), BootstrapRole::Accepted);
        let mut elapsed = 10;
        match case {
            0 => f.grant.nonce = [99; 32],
            1 => f.grant.job_id = [99; 16],
            2 => f.grant.proposal.binding.revision = 2,
            3 => elapsed = 30000,
            4 => f = frame(proposal(), BootstrapRole::Continue),
            _ => f.grant.guardian_epoch = [99; 16],
        }
        let before = m.clone();
        assert!(
            m.bootstrap(&wire(f), elapsed, [18; 32], [19; 32], observation(Some(1)))
                .is_err()
        );
        assert_eq!(m, before);
    }
    m.bootstrap(
        &wire(frame(proposal(), BootstrapRole::Accepted)),
        10,
        [18; 32],
        [19; 32],
        observation(Some(1)),
    )
    .unwrap();
    let before = m.clone();
    assert_eq!(
        m.bootstrap(
            &wire(frame(proposal(), BootstrapRole::Continue)),
            9,
            [19; 32],
            [20; 32],
            observation(Some(1))
        ),
        Err(Rejection::Deadline)
    );
    assert_eq!(m, before);
}

#[test]
fn root_end_with_live_children_never_releases_group_or_allows_second_writer() {
    let mut m = running(12);
    m.request_close().unwrap();
    let before = m.clone();
    assert_eq!(
        m.drain(end(), [20; 32], [21; 32], observation(Some(1))),
        Err(Rejection::Witness)
    );
    assert_eq!(m, before);
    assert!(m.finish([20; 32], [21; 32], observation(Some(0))).is_err());
    assert_eq!(m, before);
    assert!(
        m.commit(&parse(&proposal()), [21; 32], observation(None))
            .is_err()
    );
    assert_eq!(m, before);
    m.drain(end(), [20; 32], [21; 32], observation(Some(0)))
        .unwrap();
}

#[test]
fn unknown_or_different_guardian_job_generation_cannot_drain() {
    for case in 0..7 {
        let mut m = running(6);
        let mut o = observation(Some(0));
        let mut e = end();
        match case {
            0 => o.job = ModelJob::Unknown,
            1 => o.job = ModelJob::NoGroup,
            2 => o.lease_held = false,
            3 => {
                o.job = ModelJob::Owned {
                    job_id: [99; 16],
                    guardian_epoch: [15; 16],
                    active_processes: 0,
                }
            }
            4 => {
                o.job = ModelJob::Owned {
                    job_id: [14; 16],
                    guardian_epoch: [99; 16],
                    active_processes: 0,
                }
            }
            5 => e.launch.generation = 2,
            _ => o.control_consistent = false,
        }
        let before = m.clone();
        assert!(m.drain(e, [20; 32], [21; 32], o).is_err());
        assert_eq!(m, before);
    }
}

#[test]
fn guardian_loss_blocks_committed_bound_running_and_drained_recovery() {
    let mut drained = running(6);
    drained
        .drain(end(), [20; 32], [21; 32], observation(Some(0)))
        .unwrap();
    for mut m in [committed(), ready(), running(6), drained] {
        m.guardian_lost().unwrap();
        assert_eq!(m.phase(), Phase::Blocked);
        let before = m.clone();
        assert!(
            m.drain(end(), m.head, [99; 32], observation(Some(0)))
                .is_err()
        );
        assert!(m.finish(m.head, [99; 32], observation(Some(0))).is_err());
        assert!(
            m.commit(&parse(&proposal()), [99; 32], observation(None))
                .is_err()
        );
        assert_eq!(m, before);
    }
}

#[test]
fn early_close_can_drain_without_bootstrap_but_never_without_root_binding() {
    let mut m = committed();
    let before = m.clone();
    assert!(
        m.drain(end(), [17; 32], [18; 32], observation(Some(0)))
            .is_err()
    );
    assert_eq!(m, before);
    let mut m = ready();
    m.request_close().unwrap();
    m.drain(end(), [18; 32], [19; 32], observation(Some(0)))
        .unwrap();
    m.finish([19; 32], [20; 32], observation(Some(0))).unwrap();
    assert_eq!(m.binding, binding());
}
