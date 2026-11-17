use super::*;
use serde_json::{Value, json};

fn binding() -> WorkspaceBindingV1 {
    WorkspaceBindingV1 {
        version: 1,
        control_schema: 5,
        installation_id: [1; 16],
        workspace_id: [2; 16],
        profile_id: [3; 16],
        broker_package_sha256: [4; 32],
        ui_package_sha256: [5; 32],
        config_sha256: [6; 32],
        workspace_root_sha256: [7; 32],
        workspace_format_version: 1,
        offline_policy_version: 1,
    }
}
#[test]
fn auxiliary_profile_record_preserves_terminal_budget_and_launch_identity() {
    let mut core = model(3);
    let p = proposal();
    core.commit(&parse(&p), [10; 32], idle()).unwrap();
    let launch = core.pending_launch().unwrap();
    assert!(core.sync_control_head([9; 32], [11; 32]).is_err());
    core.sync_control_head([10; 32], [11; 32]).unwrap();
    assert_eq!(core.pending_launch(), Some(launch));
    assert!(core.sync_control_head([11; 32], [12; 32]).is_err());
    core.settle(launch, [11; 32], [12; 32], Settlement::Interrupted, idle())
        .unwrap();
    assert_eq!(core.generation(), 1);
    assert_eq!(core.binding(), &binding());
}
fn proposal() -> StartProposalV1 {
    StartProposalV1 {
        version: 1,
        purpose: Purpose::WorkspaceStart,
        binding: binding(),
        launch_id: [8; 16],
        generation: 1,
        expected_head: [9; 32],
        bootstrap_timeout_ms: 30_000,
    }
}
fn parse(p: &StartProposalV1) -> UntrustedStartProposal {
    parse_payload(&serde_json::to_vec(p).unwrap()).unwrap()
}
fn model(records: u16) -> StartModel {
    StartModel::new(binding(), [9; 32], records).unwrap()
}
fn idle() -> ModelConditions {
    ModelConditions {
        consistent_control: true,
        pending: false,
        job: ModelJob::Empty,
    }
}
fn suspended() -> ModelConditions {
    ModelConditions {
        job: ModelJob::OwnedSuspendedChecked,
        ..idle()
    }
}
fn launch() -> LaunchIdentity {
    LaunchIdentity {
        id: [8; 16],
        generation: 1,
    }
}
fn rejected(
    model: &mut StartModel,
    error: Rejection,
    operation: impl FnOnce(&mut StartModel) -> Result<(), Rejection>,
) {
    let before = model.clone();
    assert_eq!(operation(model), Err(error));
    assert_eq!(*model, before);
}

#[test]
fn bounded_parser_rejects_import_authority_unknowns_and_malformed_shapes() {
    let p = proposal();
    let bytes = serde_json::to_vec(&p).unwrap();
    let mut padded = bytes.clone();
    padded.resize(MAX_PAYLOAD, b' ');
    assert!(parse_payload(&padded).is_ok());
    padded.push(b' ');
    assert!(matches!(parse_payload(&padded), Err(Rejection::Payload)));
    assert!(parse_payload(b"").is_err());
    assert!(parse_payload(b"{}").is_err());
    let mut trailing = bytes.clone();
    trailing.extend_from_slice(b"{}");
    assert!(parse_payload(&trailing).is_err());
    let duplicate = format!(
        "{{\"version\":1,{}",
        std::str::from_utf8(&bytes[1..]).unwrap()
    );
    assert!(parse_payload(duplicate.as_bytes()).is_err());
    let original = serde_json::to_value(p).unwrap();
    let mut cases = Vec::new();
    for (name, value) in [
        ("taskId", json!(vec![1; 16])),
        ("automatic", json!(true)),
        ("path", json!("untrusted")),
        ("version", json!(3)),
        ("purpose", json!("import")),
        ("generation", json!(0)),
        ("bootstrapTimeoutMs", json!(30_001)),
        ("launchId", json!(vec![0; 16])),
        ("expectedHead", json!(vec![0; 32])),
    ] {
        let mut value_case = original.clone();
        value_case[name] = value;
        cases.push(value_case);
    }
    for (name, value) in [
        ("controlSchema", json!(4)),
        ("offlinePolicyVersion", json!(2)),
        ("workspaceFormatVersion", json!(0)),
        ("workspaceId", json!(vec![0; 16])),
        ("uiPackageSha256", json!(vec![0; 32])),
        ("rootPath", json!("untrusted")),
    ] {
        let mut value_case = original.clone();
        value_case["binding"][name] = value;
        cases.push(value_case);
    }
    for value in cases {
        assert!(
            parse_payload(&serde_json::to_vec(&value).unwrap()).is_err(),
            "{value}"
        );
    }
    let initial = model(8);
    let _untrusted = parse_payload(&bytes).unwrap();
    assert_eq!(initial.phase, ModelPhase::Idle); // Parsing has no model/host side effect.
    assert_eq!(initial.generation, 0);
}

#[test]
fn every_binding_component_is_context_bound_before_commit() {
    for field in [
        "installationId",
        "workspaceId",
        "profileId",
        "brokerPackageSha256",
        "uiPackageSha256",
        "configSha256",
        "workspaceRootSha256",
    ] {
        let mut wire: Value = serde_json::to_value(proposal()).unwrap();
        let size = if field.ends_with("Sha256") { 32 } else { 16 };
        wire["binding"][field] = json!(vec![42; size]);
        let untrusted = parse_payload(&serde_json::to_vec(&wire).unwrap()).unwrap();
        let mut state = model(8);
        rejected(&mut state, Rejection::Binding, |s| {
            s.commit(&untrusted, [10; 32], idle())
        });
    }
}

#[test]
fn inconsistent_pending_or_nonempty_jobs_never_commit_a_launch() {
    let p = parse(&proposal());
    for (conditions, error) in [
        (
            ModelConditions {
                consistent_control: false,
                ..idle()
            },
            Rejection::Control,
        ),
        (
            ModelConditions {
                pending: true,
                ..idle()
            },
            Rejection::Pending,
        ),
        (suspended(), Rejection::Job),
        (
            ModelConditions {
                job: ModelJob::Running,
                ..idle()
            },
            Rejection::Job,
        ),
        (
            ModelConditions {
                job: ModelJob::ForeignOrUnknown,
                ..idle()
            },
            Rejection::Job,
        ),
    ] {
        let mut state = model(8);
        rejected(&mut state, error, |s| s.commit(&p, [10; 32], conditions));
    }
    let mut state = model(8);
    state.commit(&p, [10; 32], idle()).unwrap();
    rejected(&mut state, Rejection::Phase, |s| {
        s.commit(&p, [11; 32], idle())
    });
}

#[test]
fn committed_generation_precedes_running_and_normal_close_preserves_binding() {
    let mut state = model(8);
    assert_eq!(state.used_launch(), None);
    state.commit(&parse(&proposal()), [10; 32], idle()).unwrap();
    assert_eq!(state.phase, ModelPhase::Committed(launch()));
    assert_eq!(state.used_launch(), Some(launch().id));
    rejected(&mut state, Rejection::Job, |s| {
        s.mark_running(launch(), [10; 32], idle())
    });
    rejected(&mut state, Rejection::Head, |s| {
        s.mark_running(launch(), [9; 32], suspended())
    });
    rejected(&mut state, Rejection::Phase, |s| {
        s.mark_running(
            LaunchIdentity {
                generation: 2,
                ..launch()
            },
            [10; 32],
            suspended(),
        )
    });
    rejected(&mut state, Rejection::Phase, |s| {
        s.settle(
            launch(),
            [10; 32],
            [11; 32],
            Settlement::NormalClose,
            idle(),
        )
    });
    state.mark_running(launch(), [10; 32], suspended()).unwrap();
    rejected(&mut state, Rejection::Job, |s| {
        s.settle(
            launch(),
            [10; 32],
            [11; 32],
            Settlement::NormalClose,
            ModelConditions {
                job: ModelJob::Running,
                ..idle()
            },
        )
    });
    state
        .settle(
            launch(),
            [10; 32],
            [11; 32],
            Settlement::NormalClose,
            idle(),
        )
        .unwrap();
    assert_eq!(state.binding, binding());
    assert_eq!(state.phase, ModelPhase::Idle);
    let mut next = proposal();
    next.generation = 2;
    next.expected_head = [11; 32];
    rejected(&mut state, Rejection::Replay, |s| {
        s.commit(&parse(&next), [12; 32], idle())
    });
    next.launch_id = [18; 16];
    state.commit(&parse(&next), [12; 32], idle()).unwrap();
    assert_eq!(state.generation, 2);
    assert_eq!(state.used_launches.len(), 2);
}

#[test]
fn interrupted_committed_or_running_launch_is_consumed_and_never_becomes_import() {
    for running in [false, true] {
        let mut state = model(8);
        state.commit(&parse(&proposal()), [10; 32], idle()).unwrap();
        if running {
            state.mark_running(launch(), [10; 32], suspended()).unwrap();
        }
        rejected(&mut state, Rejection::Phase, |s| {
            s.settle(
                LaunchIdentity {
                    id: [99; 16],
                    ..launch()
                },
                [10; 32],
                [11; 32],
                Settlement::Interrupted,
                idle(),
            )
        });
        rejected(&mut state, Rejection::Head, |s| {
            s.settle(launch(), [9; 32], [11; 32], Settlement::Interrupted, idle())
        });
        state
            .settle(
                launch(),
                [10; 32],
                [11; 32],
                Settlement::Interrupted,
                idle(),
            )
            .unwrap();
        assert_eq!(state.binding, binding());
        assert_eq!(state.generation, 1);
        assert!(state.used_launches.contains(&launch().id));
        rejected(&mut state, Rejection::Phase, |s| {
            s.settle(
                launch(),
                [11; 32],
                [12; 32],
                Settlement::Interrupted,
                idle(),
            )
        });
        let mut next = proposal();
        next.expected_head = [11; 32];
        next.generation = 2;
        next.launch_id = [19; 16];
        state.commit(&parse(&next), [12; 32], idle()).unwrap();
    }
}

#[test]
fn head_generation_and_budget_fail_without_mutation_and_reserve_settlement() {
    for next_head in [[0; 32], [9; 32]] {
        rejected(&mut model(8), Rejection::Head, |s| {
            s.commit(&parse(&proposal()), next_head, idle())
        });
    }
    let mut stale = proposal();
    stale.expected_head = [88; 32];
    rejected(&mut model(8), Rejection::Head, |s| {
        s.commit(&parse(&stale), [10; 32], idle())
    });
    let mut gap = proposal();
    gap.generation = 2;
    rejected(&mut model(8), Rejection::Generation, |s| {
        s.commit(&parse(&gap), [10; 32], idle())
    });
    let mut overflow = model(8);
    overflow.generation = u64::MAX;
    rejected(&mut overflow, Rejection::Generation, |s| {
        s.commit(&parse(&proposal()), [10; 32], idle())
    });
    for records in [0, 1] {
        rejected(&mut model(records), Rejection::Budget, |s| {
            s.commit(&parse(&proposal()), [10; 32], idle())
        });
    }
    let mut state = model(2);
    state.commit(&parse(&proposal()), [10; 32], idle()).unwrap();
    state
        .settle(
            launch(),
            [10; 32],
            [11; 32],
            Settlement::Interrupted,
            idle(),
        )
        .unwrap();
    assert_eq!(state.remaining_records, 0);
    let mut next = proposal();
    next.launch_id = [20; 16];
    next.generation = 2;
    next.expected_head = [11; 32];
    rejected(&mut state, Rejection::Budget, |s| {
        s.commit(&parse(&next), [12; 32], idle())
    });
}

#[test]
fn invalid_model_seed_and_wrong_wire_types_do_not_create_usable_state() {
    let mut b = binding();
    b.profile_id = [0; 16];
    assert_eq!(
        StartModel::new(b, [9; 32], 8).unwrap_err(),
        Rejection::Binding
    );
    assert_eq!(
        StartModel::new(binding(), [0; 32], 8).unwrap_err(),
        Rejection::Head
    );
    assert_eq!(
        StartModel::new(binding(), [9; 32], MAX_MODEL_RECORDS + 1).unwrap_err(),
        Rejection::Budget
    );
    for (name, value) in [
        ("generation", json!(-1)),
        ("generation", json!(true)),
        ("launchId", json!([1, 2, 3])),
        ("expectedHead", json!("hash")),
        ("bootstrapTimeoutMs", json!(0)),
    ] {
        let mut wire = serde_json::to_value(proposal()).unwrap();
        wire[name] = value;
        assert!(parse_payload(&serde_json::to_vec(&wire).unwrap()).is_err());
    }
}
