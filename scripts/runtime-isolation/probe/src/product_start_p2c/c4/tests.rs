use super::*;

fn package() -> UiRuntimePackageV1 {
    UiRuntimePackageV1 {
        version: 1,
        host_inventory_sha256: [1; 32],
        ui_inventory_sha256: [2; 32],
        ui_exe: "ui.exe".into(),
        runtime_root: "runtime".into(),
        runtime_version: [1, 2, 3, 4],
        loader: LoaderV1::StaticInUi,
        frontend_sha256: [3; 32],
        configuration_sha256: [4; 32],
        policy: UiPolicyV1 {
            version: 1,
            bootstrap_version: 1,
            command_version: 1,
            record_version: 1,
            max_processes: 3,
            job_sha256: [5; 32],
            acl_sha256: [6; 32],
            environment_sha256: [7; 32],
            commands_sha256: [8; 32],
            roles: [Role::Root, Role::Browser, Role::Renderer]
                .into_iter()
                .map(|role| RolePolicyV1 {
                    role,
                    image: if role == Role::Root {
                        "ui.exe".into()
                    } else {
                        "runtime/browser.exe".into()
                    },
                    max_instances: 1,
                    arguments_sha256: [role as u8; 32],
                    token_sha256: [9; 32],
                })
                .collect(),
        },
        db_read_min: 6,
        db_read_max: 6,
        db_write: 6,
        webview_data_version: 1,
        binding_revision: 1,
    }
}
fn binding() -> WorkspaceBindingV2 {
    let p = package();
    let root = |n| super::super::RootIdentityV1 {
        canonical_path_sha256: [n; 32],
        volume_id: 1,
        file_id: [n; 16],
    };
    WorkspaceBindingV2 {
        version: 2,
        control_schema: 6,
        ui_abi: 1,
        installation_id: [1; 16],
        workspace_id: [2; 16],
        profile_name: "kraqel-c4-synthetic".into(),
        profile_sid: "S-1-15-2-1-2-3-4-5-6-7".into(),
        projects: root(3),
        webview: root(4),
        session_root: root(5),
        host_package_sha256: p.host_inventory_sha256,
        ui_package_sha256: p.ui_inventory_sha256,
        config_sha256: p.configuration_sha256,
        commands_sha256: p.policy.commands_sha256,
        job_policy_sha256: p.policy.job_sha256,
        acl_policy_sha256: p.policy.acl_sha256,
        workspace_format: 1,
        db_read_min: 6,
        db_read_max: 6,
        db_write: 6,
        max_processes: 3,
        revision: 1,
    }
}
fn observation(count: Option<u32>) -> ModelObservationV1 {
    ModelObservationV1 {
        lease_held: true,
        control_consistent: true,
        package_verified: true,
        runtime_roles_verified: true,
        root: Some(RootClaim {
            pid: 42,
            created: 123,
        }),
        job: count.map_or(ModelJob::NoGroup, |active_processes| ModelJob::Owned {
            job_id: [10; 16],
            guardian_epoch: [11; 16],
            active_processes,
        }),
        elapsed_ms: 0,
    }
}
fn record(model: &LifecycleModelV1, step: StepV1) -> ModelRecordV1 {
    ModelRecordV1 {
        version: 1,
        step,
        workspace: model.binding.workspace_id,
        binding_revision: model.binding.revision,
        package_sha256: model.package,
        binding_sha256: model.binding_sha256,
        launch: model.launch.unwrap_or(LaunchIdentity {
            id: [12; 16],
            generation: model.generation + 1,
        }),
        expected_head: model.head,
        next_head: Sha256::digest(model.head).into(),
    }
}
fn model() -> LifecycleModelV1 {
    LifecycleModelV1::new(&package(), binding(), [13; 32], 256).unwrap()
}
fn advance(m: &mut LifecycleModelV1, step: StepV1, count: Option<u32>) {
    m.apply(&record(m, step), observation(count)).unwrap();
}
fn running() -> LifecycleModelV1 {
    let mut m = model();
    for (s, c) in [
        (StepV1::Commit, None),
        (StepV1::RootBound, Some(1)),
        (StepV1::Accepted, Some(1)),
        (StepV1::Running, Some(1)),
    ] {
        advance(&mut m, s, c);
    }
    m
}
fn rejected(m: &mut LifecycleModelV1, r: &ModelRecordV1, o: ModelObservationV1) {
    let before = m.clone();
    assert!(m.apply(r, o).is_err());
    assert_eq!(*m, before);
}

#[test]
fn c4_golden_package_and_canonical_round_trip() {
    let p = package();
    let bytes = p.encode().unwrap();
    assert_eq!(UiRuntimePackageV1::decode(&bytes).unwrap(), p);
    // Independently calculated by artifacts/kq069-c4-golden.py, not Rust serialization.
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "5ff2f3bc766630978c254b578561d3f168fc5e7d4a443630b4f1d0c6ca8fec39"
    );
    let mut reordered = p.clone();
    reordered.policy.roles.reverse();
    assert_eq!(reordered.encode().unwrap(), bytes);
    assert_eq!(reordered.digest().unwrap(), p.digest().unwrap());
    let policy = p.policy.encode().unwrap();
    assert_eq!(UiPolicyV1::decode(&policy).unwrap(), p.policy);
    assert_eq!(
        p.policy.digest().unwrap(),
        <Digest>::from(Sha256::digest(&policy))
    );
}
#[test]
fn c4_every_package_field_binds_digest() {
    let p = package();
    let original = p.digest().unwrap();
    let mutations: Vec<fn(&mut UiRuntimePackageV1)> = vec![
        |p| p.host_inventory_sha256 = [20; 32],
        |p| p.ui_inventory_sha256 = [20; 32],
        |p| p.runtime_version[0] += 1,
        |p| {
            p.loader = LoaderV1::Dynamic {
                path: "loader.dll".into(),
                sha256: [20; 32],
            }
        },
        |p| p.frontend_sha256 = [20; 32],
        |p| p.configuration_sha256 = [20; 32],
        |p| p.policy.environment_sha256 = [20; 32],
        |p| p.policy.job_sha256 = [20; 32],
        |p| p.policy.acl_sha256 = [20; 32],
        |p| p.policy.commands_sha256 = [20; 32],
        |p| p.policy.roles[1].arguments_sha256 = [20; 32],
        |p| p.policy.roles[1].token_sha256 = [20; 32],
        |p| p.db_read_min = 5,
        |p| p.db_read_max = 7,
        |p| p.webview_data_version = 2,
        |p| p.binding_revision = 2,
    ];
    for mutate in mutations {
        let mut q = p.clone();
        mutate(&mut q);
        assert_ne!(q.digest().unwrap(), original);
        assert_eq!(UiRuntimePackageV1::decode(&q.encode().unwrap()).unwrap(), q);
    }
}
#[test]
fn c4_strict_decoder_domains_versions_truncation_and_trailing() {
    let bytes = package().encode().unwrap();
    for n in 0..bytes.len() {
        assert!(UiRuntimePackageV1::decode(&bytes[..n]).is_err());
    }
    for pos in [
        0,
        PACKAGE_DOMAIN.len() - 1,
        PACKAGE_DOMAIN.len(),
        PACKAGE_DOMAIN.len() + 2,
        PACKAGE_DOMAIN.len() + 3,
    ] {
        let mut bad = bytes.clone();
        bad[pos] = 255;
        assert!(UiRuntimePackageV1::decode(&bad).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(UiRuntimePackageV1::decode(&extra).is_err());
    assert!(UiRuntimePackageV1::decode(&vec![0; MAX_DESCRIPTOR + 1]).is_err());
    assert!(
        UiRuntimePackageV1::decode(
            &super::super::encode_inventory(
                super::super::PackageRole::Ui,
                &[super::super::InventoryEntry {
                    path: "a".into(),
                    size: 0,
                    sha256: [1; 32],
                    role: super::super::FileRole::Resource
                }]
            )
            .unwrap()
        )
        .is_err()
    );
    assert!(UiRuntimePackageV1::decode(b"KRAQEL-C3-SYNTHETIC-GROUP\0\x01").is_err());
}
#[test]
fn c4_paths_policy_limits_and_unknown_roles() {
    for path in [
        "../ui.exe",
        "C:/ui.exe",
        "ui.exe:stream",
        "NUL.txt",
        "a/./b",
        "e\u{301}.exe",
    ] {
        let mut p = package();
        p.ui_exe = path.into();
        assert!(p.encode().is_err());
    }
    for mutate in [
        |p: &mut UiRuntimePackageV1| p.policy.max_processes = 0,
        |p: &mut UiRuntimePackageV1| p.policy.max_processes = 65,
        |p: &mut UiRuntimePackageV1| p.policy.roles[0].max_instances = 2,
        |p: &mut UiRuntimePackageV1| p.policy.roles.push(p.policy.roles[0].clone()),
        |p: &mut UiRuntimePackageV1| p.policy.roles[1].role = Role::Gpu,
        |p: &mut UiRuntimePackageV1| p.policy.roles[1].image = "other/browser.exe".into(),
        |p: &mut UiRuntimePackageV1| p.db_write = 7,
        |p: &mut UiRuntimePackageV1| p.policy.environment_sha256 = [0; 32],
        |p: &mut UiRuntimePackageV1| p.policy.bootstrap_version = 2,
    ] {
        let mut p = package();
        mutate(&mut p);
        assert!(p.encode().is_err());
    }
    let mut bytes = package().policy.encode().unwrap();
    let first_role = POLICY_DOMAIN.len() + 10 + 1 + 128 + 1;
    bytes[first_role] = 255;
    assert!(UiPolicyV1::decode(&bytes).is_err());
    let mut bytes = package().policy.encode().unwrap();
    bytes[first_role] = Role::Browser as u8;
    assert!(UiPolicyV1::decode(&bytes).is_err());
    let mut bytes = package().policy.encode().unwrap();
    bytes[POLICY_DOMAIN.len() + 10] = 0;
    assert!(UiPolicyV1::decode(&bytes).is_err());
}
#[test]
fn c4_binding_rejects_c2_no_execution_and_mismatches() {
    let p = package();
    p.matches(&binding()).unwrap();
    for mutate in [
        |b: &mut WorkspaceBindingV2| {
            b.job_policy_sha256 = Sha256::digest(b"C2:no-execution").into()
        },
        |b: &mut WorkspaceBindingV2| b.ui_abi = 0,
        |b: &mut WorkspaceBindingV2| b.control_schema = 5,
        |b: &mut WorkspaceBindingV2| b.ui_package_sha256 = [30; 32],
        |b: &mut WorkspaceBindingV2| b.revision = 2,
        |b: &mut WorkspaceBindingV2| b.max_processes = 8,
    ] {
        let mut b = binding();
        mutate(&mut b);
        assert!(LifecycleModelV1::new(&p, b, [13; 32], 256).is_err());
    }
}
#[test]
fn c4_versioned_records_round_trip_and_reject_cross_format() {
    let m = model();
    let r = record(&m, StepV1::Commit);
    let bytes = r.encode().unwrap();
    assert_eq!(ModelRecordV1::decode(&bytes).unwrap(), r);
    for n in 0..bytes.len() {
        assert!(ModelRecordV1::decode(&bytes[..n]).is_err());
    }
    for pos in [
        0,
        RECORD_DOMAIN.len() - 1,
        RECORD_DOMAIN.len(),
        RECORD_DOMAIN.len() + 2,
    ] {
        let mut bad = bytes.clone();
        bad[pos] = 255;
        assert!(ModelRecordV1::decode(&bad).is_err());
    }
    let mut bad = bytes.clone();
    bad.push(0);
    assert!(ModelRecordV1::decode(&bad).is_err());
    assert!(ModelRecordV1::decode(&package().encode().unwrap()).is_err());
}
#[test]
fn c4_success_reopen_keeps_binding_and_consumes_launch() {
    let mut m = running();
    advance(&mut m, StepV1::Ready, Some(3));
    assert_eq!(m.phase(), PhaseV1::Ready);
    let binding = m.binding.clone();
    let old = m.launch.unwrap();
    for (s, c) in [
        (StepV1::Closing, 3),
        (StepV1::Drained, 0),
        (StepV1::Finish, 0),
    ] {
        advance(&mut m, s, Some(c));
    }
    assert_eq!(m.phase(), PhaseV1::Idle);
    assert_eq!(m.binding, binding);
    let replay = record(&m, StepV1::Commit);
    rejected(&mut m, &replay, observation(None));
    let mut fresh = replay;
    fresh.launch.id = [14; 16];
    m.apply(&fresh, observation(None)).unwrap();
    assert_eq!(m.generation, old.generation + 1);
}
#[test]
fn c4_rejections_are_atomic_and_require_current_context() {
    let mut m = model();
    let r = record(&m, StepV1::Commit);
    for mutate in [
        |r: &mut ModelRecordV1| r.version = 2,
        |r: &mut ModelRecordV1| r.workspace = [33; 16],
        |r: &mut ModelRecordV1| r.binding_revision = 2,
        |r: &mut ModelRecordV1| r.package_sha256 = [33; 32],
        |r: &mut ModelRecordV1| r.expected_head = [33; 32],
        |r: &mut ModelRecordV1| r.next_head = r.expected_head,
        |r: &mut ModelRecordV1| r.launch.generation = 2,
    ] {
        let mut bad = r.clone();
        mutate(&mut bad);
        rejected(&mut m, &bad, observation(None));
    }
    for mutate in [
        |o: &mut ModelObservationV1| o.lease_held = false,
        |o: &mut ModelObservationV1| o.control_consistent = false,
        |o: &mut ModelObservationV1| o.package_verified = false,
        |o: &mut ModelObservationV1| o.job = ModelJob::Unknown,
    ] {
        let mut o = observation(None);
        mutate(&mut o);
        rejected(&mut m, &r, o);
    }
    m.remaining = 7;
    rejected(&mut m, &r, observation(None));
}
#[test]
fn c4_runtime_ready_requires_roles_and_root_binding() {
    let mut m = running();
    let r = record(&m, StepV1::Ready);
    let mut bad = observation(Some(3));
    bad.runtime_roles_verified = false;
    rejected(&mut m, &r, bad);
    let mut bad = observation(Some(3));
    bad.root = Some(RootClaim {
        pid: 42,
        created: 124,
    });
    rejected(&mut m, &r, bad);
    rejected(&mut m, &r, observation(Some(2)));
    rejected(&mut m, &r, observation(Some(4)));
    let second = record(&m, StepV1::Commit);
    rejected(&mut m, &second, observation(None));
}
#[test]
fn c4_root_end_never_replaces_group_drain() {
    let mut m = running();
    advance(&mut m, StepV1::Closing, Some(1));
    let r = record(&m, StepV1::Drained);
    rejected(&mut m, &r, observation(Some(1)));
    let mut wrong = observation(Some(0));
    wrong.job = ModelJob::Owned {
        job_id: [99; 16],
        guardian_epoch: [11; 16],
        active_processes: 0,
    };
    rejected(&mut m, &r, wrong);
    let mut wrong = observation(Some(0));
    wrong.job = ModelJob::Owned {
        job_id: [10; 16],
        guardian_epoch: [99; 16],
        active_processes: 0,
    };
    rejected(&mut m, &r, wrong);
    advance(&mut m, StepV1::Drained, Some(0));
    let r = record(&m, StepV1::Finish);
    rejected(&mut m, &r, observation(Some(1)));
    advance(&mut m, StepV1::Finish, Some(0));
}
#[test]
fn c4_guardian_loss_stays_blocked_even_after_claimed_empty_job() {
    for before_drain in [true, false] {
        let mut m = running();
        advance(&mut m, StepV1::Closing, Some(1));
        if !before_drain {
            advance(&mut m, StepV1::Drained, Some(0));
        }
        m.guardian_lost();
        assert_eq!(m.phase(), PhaseV1::Blocked);
        for step in [
            StepV1::Commit,
            StepV1::RootBound,
            StepV1::Drained,
            StepV1::Finish,
        ] {
            let r = record(&m, step);
            rejected(&mut m, &r, observation(Some(0)));
        }
    }
    let mut m = model();
    advance(&mut m, StepV1::Commit, None);
    let close = record(&m, StepV1::Closing);
    rejected(&mut m, &close, observation(Some(0)));
    m.guardian_lost();
    assert_eq!(m.phase(), PhaseV1::Blocked);
}

#[test]
fn c4_full_workspace_binding_and_bootstrap_deadline() {
    let p = package();
    let b = binding();
    let original = binding_digest(&p, &b).unwrap();
    for mutate in [
        |b: &mut WorkspaceBindingV2| b.projects.file_id = [55; 16],
        |b: &mut WorkspaceBindingV2| b.profile_name = "other-profile".into(),
        |b: &mut WorkspaceBindingV2| b.installation_id = [55; 16],
    ] {
        let mut other = b.clone();
        mutate(&mut other);
        assert_ne!(binding_digest(&p, &other).unwrap(), original);
    }
    let mut m = model();
    let mut r = record(&m, StepV1::Commit);
    r.binding_sha256 = [55; 32];
    rejected(&mut m, &r, observation(None));
    advance(&mut m, StepV1::Commit, None);
    let r = record(&m, StepV1::RootBound);
    let mut o = observation(Some(1));
    o.elapsed_ms = 30_000;
    rejected(&mut m, &r, o);
    o.elapsed_ms = 29_999;
    m.apply(&r, o).unwrap();
    let r = record(&m, StepV1::Accepted);
    rejected(&mut m, &r, observation(Some(1)));
    o.elapsed_ms = 30_000;
    rejected(&mut m, &r, o);
    // Expired bootstrap can close a bound group, but cannot become Running.
    let close = record(&m, StepV1::Closing);
    m.apply(&close, o).unwrap();
}
