use super::*;

#[test]
fn bp2_record_order_blocks_other_actions_and_launch_replay() {
    let f = Fixture::new();
    let store = f.prepared();
    let mut state = store.state.clone();
    let mut e = bp2::Bp2Event {
        version: 1,
        stage: bp2::Stage::Commit,
        launch: Uuid::new_v4(),
        workspace: state.c2.as_ref().unwrap().id,
        package: [1; 32],
        guardian: bp2::Identity {
            pid: 42,
            created: 1,
        },
        nonce: [2; 32],
        root: None,
        receipt: [0; 32],
    };
    state
        .apply_record(f.actor, &Action::Bp2LeafProbe { event: e.clone() }, [3; 32])
        .unwrap();
    assert!(
        state
            .apply_record(f.actor, &Action::RetireC2Workspace, [3; 32])
            .is_err()
    );
    assert!(
        state
            .apply_record(f.actor, &Action::Bp2LeafProbe { event: e.clone() }, [3; 32])
            .is_err()
    );
    e.root = Some(bp2::Identity {
        pid: 43,
        created: 2,
    });
    e.stage = bp2::Stage::RootBound;
    state
        .apply_record(f.actor, &Action::Bp2LeafProbe { event: e.clone() }, [3; 32])
        .unwrap();
    let mut bad = e.clone();
    bad.stage = bp2::Stage::Finish;
    assert!(
        state
            .apply_record(f.actor, &Action::Bp2LeafProbe { event: bad }, [3; 32])
            .is_err()
    );
    e.stage = bp2::Stage::Drained;
    assert!(
        state
            .apply_record(f.actor, &Action::Bp2LeafProbe { event: e.clone() }, [3; 32])
            .is_err()
    );
    e.receipt = [4; 32];
    state
        .apply_record(f.actor, &Action::Bp2LeafProbe { event: e.clone() }, [3; 32])
        .unwrap();
    e.stage = bp2::Stage::Finish;
    state
        .apply_record(f.actor, &Action::Bp2LeafProbe { event: e.clone() }, [3; 32])
        .unwrap();
    assert!(!state.bp2_probe.pending());
    e.stage = bp2::Stage::Commit;
    e.root = None;
    e.receipt = [0; 32];
    assert!(
        state
            .apply_record(f.actor, &Action::Bp2LeafProbe { event: e }, [3; 32])
            .is_err()
    );
}

#[test]
#[ignore = "explicit new BP2 native leaf capability series; no UI/runtime/projects"]
fn bp2_native_leaf_capability_series() {
    let saved = std::env::temp_dir().join("kraqel-bp2-build-20261118-01");
    let executable = saved.join("controller.exe");
    assert!(executable.is_file());
    let f = Fixture::new();
    let mut store = f.prepared();
    let head = store.state.head;
    let sequence = store.state.sequence;
    let unsupported = bp2::run(&executable, &mut store, bp2::Mode::UnsupportedRequirement).unwrap();
    assert_eq!(unsupported["rejectedBeforeCreate"], true);
    assert_eq!(head, store.state.head);
    assert_eq!(sequence, store.state.sequence);
    let mismatch = bp2::run(&executable, &mut store, bp2::Mode::ReadbackMismatch).unwrap();
    assert_eq!(mismatch["receipt"]["resumed"], false, "{mismatch}");
    assert_eq!(
        mismatch["receipt"]["error"]["stage"],
        "bp2-policy-readback-mismatch"
    );
    let bad = bp2::run(&executable, &mut store, bp2::Mode::BadMac).unwrap();
    assert_eq!(bad["receipt"]["accepted"], false, "{bad}");
    let live = bp2::run(&executable, &mut store, bp2::Mode::Normal).unwrap();
    // Persist diagnostics before assertions; an unsupported native result stays a failed case.
    let id = store.state.c2.as_ref().unwrap().id;
    let audit = saved.join(format!("audit-{}", Uuid::new_v4().simple()));
    secure_directory(&audit).unwrap();
    secure_directory(&audit.join("records")).unwrap();
    for name in ["authority.key", "head.mac"] {
        write_new(
            &audit.join(name),
            &fs::read(f.root.join("control").join(name)).unwrap(),
        )
        .unwrap();
    }
    for entry in fs::read_dir(f.root.join("control/records")).unwrap() {
        let entry = entry.unwrap();
        write_new(
            &audit.join("records").join(entry.file_name()),
            &fs::read(entry.path()).unwrap(),
        )
        .unwrap();
    }
    let report = json!({"unsupported":unsupported,"mismatch":mismatch,"badMac":bad,"live":live,"controlRoot":f.root,"dataRoot":data_root(id),"auditRoot":audit,"records":store.state.sequence,"sourceSha256":hex(&hash(&executable).unwrap())});
    fs::write(
        saved.join("run.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    assert!(live["receipt"]["error"].is_null(), "{live}");
    assert_eq!(live["receipt"]["result"]["childError"], 367);
    assert_eq!(live["receipt"]["jobTotalProcesses"], 1);
    assert_eq!(live["controlFinished"], true);
    assert_eq!(store.state.bp2_probe.used_count_for_test(), 3);
    drop(store);
    let reopened = Store::open(&f.root).unwrap();
    assert!(!reopened.state.bp2_probe.pending());
    assert_eq!(reopened.state.bp2_probe.used_count_for_test(), 3);
    drop(reopened);
}

struct Fixture {
    base: PathBuf,
    root: PathBuf,
    actor: [u8; 32],
}
impl Fixture {
    fn new() -> Self {
        let base = std::env::temp_dir().join(format!(
            "kraqel-isolation-controller-{}",
            Uuid::new_v4().simple()
        ));
        secure_directory(&base).unwrap();
        secure_directory(&base.join("case-01")).unwrap();
        let root = base.join("case-01/store");
        let actor = hash(&std::env::current_exe().unwrap()).unwrap();
        Self { base, root, actor }
    }
    fn init(&self) -> Store {
        Store::initialize_mode(&self.root, self.actor, true).unwrap()
    }
    fn prepared(&self) -> Store {
        let mut store = self.init();
        let (_dirs, mut source, sha) = binary_pin(&std::env::current_exe().unwrap()).unwrap();
        assert_eq!(sha, self.actor);
        package::verify_persistent_records(&mut source).unwrap();
        store
            .prepare_c2(self.actor, store.state.head, &mut source)
            .unwrap();
        store
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let mut data = None;
        if self.root.exists() {
            let mut store = Store::open(&self.root).unwrap();
            if let Some(p) = &store.state.c2 {
                data = Some(data_root(p.id));
                if p.profile_ready && !p.retired {
                    store.retire_c2(self.actor, store.state.head).unwrap();
                }
            }
            drop(store);
        }
        let temp = std::env::temp_dir().canonicalize().unwrap();
        // Only these explicitly owned, resolved TEMP fixture roots may be recursively removed.
        for path in data.into_iter().chain([self.base.clone()]) {
            if !path.exists() {
                continue;
            }
            let resolved = path.canonicalize().unwrap();
            assert_eq!(resolved.parent(), Some(temp.as_path()));
            assert!(crate::policy::valid_root(&resolved, true));
            assert_eq!(resolved.file_name(), path.file_name());
            fs::remove_dir_all(&resolved).unwrap();
        }
    }
}

fn c4_candidate(f: &Fixture, store: &Store, real_runtime: bool) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    c4_candidate_source(f, store, real_runtime, None)
}
fn c4_candidate_source(
    f: &Fixture,
    store: &Store,
    real_runtime: bool,
    executable: Option<&Path>,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    use crate::product_start_p2c::c4::{
        LoaderV1, Role, RolePolicyV1, UiPolicyV1, UiRuntimePackageV1,
    };
    let id = store.state.c2.as_ref().unwrap().id;
    let root = data_root(id).join("c4-candidate");
    secure_directory(&root).unwrap();
    let host = root.join("host");
    secure_directory(&host).unwrap();
    let ui = root.join("ui");
    secure_directory(&ui).unwrap();
    let runtime = ui.join("runtime");
    secure_directory(&runtime).unwrap();
    let source = executable
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_exe().unwrap());
    let bytes = fs::read(&source).unwrap();
    write_new(&host.join("host.exe"), &bytes).unwrap();
    write_new(&ui.join("ui.exe"), &bytes).unwrap();
    if real_runtime {
        let source=std::env::temp_dir().join("kraqel-c4-runtime-20261117-01/expanded/Microsoft.WebView2.FixedVersionRuntime.157.0.4341.60.x64");
        let mut todo = vec![(source, runtime.clone())];
        while let Some((from, to)) = todo.pop() {
            for entry in fs::read_dir(from).unwrap() {
                let entry = entry.unwrap();
                let kind = entry.file_type().unwrap();
                assert!(!kind.is_symlink());
                let target = to.join(entry.file_name());
                if kind.is_dir() {
                    secure_directory(&target).unwrap();
                    todo.push((entry.path(), target));
                } else {
                    assert!(kind.is_file());
                    write_new(&target, &fs::read(entry.path()).unwrap()).unwrap();
                }
            }
        }
    } else {
        write_new(&runtime.join("msedgewebview2.exe"), &bytes).unwrap();
    }
    fn inventory(root: &Path, role: PackageRole) -> (Vec<u8>, [u8; 32]) {
        let mut todo = vec![root.to_path_buf()];
        let mut entries = Vec::new();
        while let Some(path) = todo.pop() {
            for entry in fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    todo.push(entry.path());
                } else {
                    let path = entry.path();
                    let name = path
                        .strip_prefix(root)
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .replace('\\', "/");
                    let file_role = match path.extension().and_then(|s| s.to_str()) {
                        Some("exe") => FileRole::Program,
                        Some("dll") => FileRole::Library,
                        _ => FileRole::Resource,
                    };
                    entries.push(InventoryEntry {
                        path: name,
                        size: fs::metadata(&path).unwrap().len(),
                        sha256: hash(&path).unwrap(),
                        role: file_role,
                    });
                }
            }
        }
        (
            contract::encode_inventory(role, &entries).unwrap(),
            contract::inventory_digest(role, &entries).unwrap(),
        )
    }
    let (host_bytes, host_digest) = inventory(&host, PackageRole::Host);
    let (ui_bytes, ui_digest) = inventory(&ui, PackageRole::Ui);
    let candidate = UiRuntimePackageV1 {
        version: 1,
        host_inventory_sha256: host_digest,
        ui_inventory_sha256: ui_digest,
        ui_exe: "ui.exe".into(),
        runtime_root: "runtime".into(),
        runtime_version: if real_runtime {
            [157, 0, 4341, 60]
        } else {
            [1, 0, 0, 1]
        },
        loader: LoaderV1::StaticInUi,
        frontend_sha256: [1; 32],
        configuration_sha256: [2; 32],
        policy: UiPolicyV1 {
            version: 1,
            bootstrap_version: 1,
            command_version: 1,
            record_version: 1,
            max_processes: if real_runtime { 16 } else { 3 },
            job_sha256: [3; 32],
            acl_sha256: [4; 32],
            environment_sha256: [5; 32],
            commands_sha256: [6; 32],
            roles: if real_runtime {
                vec![
                    Role::Root,
                    Role::Browser,
                    Role::Renderer,
                    Role::Gpu,
                    Role::Utility,
                    Role::CrashHandler,
                ]
            } else {
                vec![Role::Root, Role::Browser, Role::Renderer]
            }
            .into_iter()
            .map(|role| RolePolicyV1 {
                role,
                image: if role == Role::Root {
                    "ui.exe".into()
                } else {
                    "runtime/msedgewebview2.exe".into()
                },
                max_instances: if real_runtime {
                    match role {
                        Role::Renderer => 4,
                        Role::Utility => 8,
                        _ => 1,
                    }
                } else {
                    1
                },
                arguments_sha256: [7; 32],
                token_sha256: [8; 32],
            })
            .collect(),
        },
        db_read_min: 6,
        db_read_max: 6,
        db_write: 6,
        webview_data_version: 1,
        binding_revision: 1,
    };
    assert!(f.root.starts_with(&f.base));
    (candidate.encode().unwrap(), host_bytes, ui_bytes)
}

#[test]
fn c4_native_candidate_is_pinned_and_rejections_do_not_write_control() {
    let f = Fixture::new();
    let store = f.prepared();
    let head = store.state.head;
    let (descriptor, host, ui) = c4_candidate(&f, &store, false);
    let before_key = hash(&f.root.join("control/authority.key")).unwrap();
    let pins = store
        .verify_c4_candidate(f.actor, head, &descriptor, &host, &ui)
        .unwrap();
    let id = store.state.c2.as_ref().unwrap().id;
    let exe = data_root(id).join("c4-candidate/ui/ui.exe");
    assert!(OpenOptions::new().write(true).open(&exe).is_err());
    assert!(fs::rename(&exe, exe.with_extension("old")).is_err());
    assert!(Store::open(&f.root).is_err());
    drop(pins);
    let mut wrong = descriptor.clone();
    wrong[0] ^= 1;
    assert!(
        store
            .verify_c4_candidate(f.actor, head, &wrong, &host, &ui)
            .is_err()
    );
    assert!(
        store
            .verify_c4_candidate([99; 32], head, &descriptor, &host, &ui)
            .is_err()
    );
    assert!(
        store
            .verify_c4_candidate(f.actor, [99; 32], &descriptor, &host, &ui)
            .is_err()
    );
    let extra = exe.with_file_name("extra.dll");
    write_new(&extra, b"not declared").unwrap();
    assert!(
        store
            .verify_c4_candidate(f.actor, head, &descriptor, &host, &ui)
            .is_err()
    );
    fs::remove_file(&extra).unwrap();
    let mut altered = OpenOptions::new().write(true).open(&exe).unwrap();
    altered.set_len(0).unwrap();
    altered.write_all(b"tampered").unwrap();
    drop(altered);
    assert!(
        store
            .verify_c4_candidate(f.actor, head, &descriptor, &host, &ui)
            .is_err()
    );
    assert_eq!(store.state.head, head);
    assert_eq!(
        hash(&f.root.join("control/authority.key")).unwrap(),
        before_key
    );
}

#[test]
fn c4_native_candidate_rejects_hardlinks_streams_and_retired_workspace() {
    let f = Fixture::new();
    let mut store = f.prepared();
    let head = store.state.head;
    let (descriptor, host, ui) = c4_candidate(&f, &store, false);
    let id = store.state.c2.as_ref().unwrap().id;
    let exe = data_root(id).join("c4-candidate/ui/ui.exe");
    let link = f.base.join("case-01/extra-link");
    fs::hard_link(&exe, &link).unwrap();
    assert!(
        store
            .verify_c4_candidate(f.actor, head, &descriptor, &host, &ui)
            .is_err()
    );
    fs::remove_file(&link).unwrap();
    let ads = PathBuf::from(format!("{}:hidden", exe.display()));
    fs::write(&ads, b"hidden").unwrap();
    assert!(
        store
            .verify_c4_candidate(f.actor, head, &descriptor, &host, &ui)
            .is_err()
    );
    fs::remove_file(&ads).unwrap();
    store
        .verify_c4_candidate(f.actor, head, &descriptor, &host, &ui)
        .unwrap();
    store.retire_c2(f.actor, head).unwrap();
    assert!(
        store
            .verify_c4_candidate(f.actor, store.state.head, &descriptor, &host, &ui)
            .is_err()
    );
}

#[test]
#[ignore = "explicit local Microsoft Fixed-Version package inventory; no runtime execution"]
fn c4_downloaded_fixed_runtime_native_preflight() {
    let f = Fixture::new();
    let store = f.prepared();
    let head = store.state.head;
    let (descriptor, host, ui) = c4_candidate(&f, &store, true);
    let pins = store
        .verify_c4_candidate(f.actor, head, &descriptor, &host, &ui)
        .unwrap();
    assert_eq!(store.state.head, head);
    assert!(Store::open(&f.root).is_err());
    drop(pins);
}

#[test]
fn c4_native_suspended_pair_is_born_contained_and_never_resumed() {
    let f = Fixture::new();
    let store = f.prepared();
    let head = store.state.head;
    let (descriptor, host, ui) = c4_candidate(&f, &store, false);
    let pins = store
        .verify_c4_candidate(f.actor, head, &descriptor, &host, &ui)
        .unwrap();
    pins.probe_suspended_pair(&store).unwrap();
    assert_eq!(store.state.head, head);
    store.idle().unwrap();
    drop(pins);
}

#[test]
fn c4_root_records_block_other_actions_replay_and_forged_drain_order() {
    use c4_preflight::{Identity, Stage};
    let f = Fixture::new();
    let store = f.prepared();
    let mut state = store.state.clone();
    let mut event = C4ProbeEvent {
        version: 1,
        stage: Stage::Commit,
        launch: Uuid::new_v4(),
        workspace: state.c2.as_ref().unwrap().id,
        package: [1; 32],
        guardian: Identity {
            pid: 42,
            created: 3,
        },
        nonce: [2; 32],
        root: None,
    };
    let actor = state.actor;
    state
        .apply_record(
            actor,
            &Action::C4RootProbe {
                event: event.clone(),
            },
            [3; 32],
        )
        .unwrap();
    assert!(
        state
            .apply_record(actor, &Action::RetireC2Workspace, [3; 32])
            .is_err()
    );
    let mut wrong = event.clone();
    wrong.stage = Stage::Finish;
    assert!(
        state
            .apply_record(actor, &Action::C4RootProbe { event: wrong }, [3; 32])
            .is_err()
    );
    event.stage = Stage::RootBound;
    event.root = Some(Identity {
        pid: 43,
        created: 4,
    });
    state
        .apply_record(
            actor,
            &Action::C4RootProbe {
                event: event.clone(),
            },
            [3; 32],
        )
        .unwrap();
    let mut wrong = event.clone();
    wrong.stage = Stage::Accepted;
    wrong.nonce = [5; 32];
    assert!(
        state
            .apply_record(actor, &Action::C4RootProbe { event: wrong }, [3; 32])
            .is_err()
    );
    let pending = state.clone();
    assert!(pending.c4_probe.pending()); // Reconstructed state cannot infer drain from PID absence.
    for stage in [
        Stage::Accepted,
        Stage::Running,
        Stage::Drained,
        Stage::Finish,
    ] {
        event.stage = stage;
        state
            .apply_record(
                actor,
                &Action::C4RootProbe {
                    event: event.clone(),
                },
                [3; 32],
            )
            .unwrap();
    }
    assert!(!state.c4_probe.pending());
    event.stage = Stage::Commit;
    event.root = None;
    assert!(
        state
            .apply_record(actor, &Action::C4RootProbe { event }, [3; 32])
            .is_err()
    );
}

#[test]
#[ignore = "explicit local authenticated blank Fixed-Version WebView research; no project content"]
fn c4_authenticated_root_rejects_external_modules_before_continue() {
    let saved = std::env::temp_dir().join("kraqel-c4b-module-gate-20261118-03");
    let executable = saved.join("controller.exe");
    assert!(executable.is_file());
    let f = Fixture::new();
    let mut store = f.prepared();
    let head = store.state.head;
    let (descriptor, host, ui) = c4_candidate_source(&f, &store, true, Some(&executable));
    let pins = store
        .verify_c4_candidate(f.actor, head, &descriptor, &host, &ui)
        .unwrap();
    let origins: Vec<Value> = serde_json::from_slice(
        &fs::read(
            std::env::temp_dir()
                .join("kraqel-c4b-observation-build-20261118-01/module-origins.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let paths: Vec<_> = origins
        .iter()
        .map(|v| PathBuf::from(v["path"].as_str().unwrap()))
        .collect();
    assert_eq!(paths.len(), 2);
    let module_checks = c4_preflight::test_module_paths(&pins, &paths).unwrap();
    assert!(module_checks.iter().all(|entry| entry["allowed"] == false));
    let bad = c4_preflight::run_root_probe(&pins, &mut store, true).unwrap();
    assert_eq!(bad["rootAuthenticated"], false);
    assert_eq!(bad["continueSent"], false);
    assert_eq!(bad["drained"], true);
    let live = c4_preflight::run_root_probe(&pins, &mut store, false).unwrap();
    // This local workstation's external monitor must stop the authenticated root before WebView.
    assert_eq!(live["rootAuthenticated"], true, "{live}");
    assert_eq!(live["continueSent"], false);
    assert_eq!(live["controllerError"]["stage"], "c4-module-outside-policy");
    assert!(live["runtimeOutcome"].is_null());
    assert_eq!(live["jobTotalProcesses"], 1);
    assert_eq!(live["unobservedProcessCount"], 0);
    assert_eq!(live["controlFinished"], true);
    let id = store.state.c2.as_ref().unwrap().id;
    let audit = saved.join(format!("audit-{}", Uuid::new_v4().simple()));
    secure_directory(&audit).unwrap();
    secure_directory(&audit.join("records")).unwrap();
    for name in ["authority.key", "head.mac"] {
        write_new(
            &audit.join(name),
            &fs::read(f.root.join("control").join(name)).unwrap(),
        )
        .unwrap();
    }
    for entry in fs::read_dir(f.root.join("control/records")).unwrap() {
        let entry = entry.unwrap();
        write_new(
            &audit.join("records").join(entry.file_name()),
            &fs::read(entry.path()).unwrap(),
        )
        .unwrap();
    }
    let report = json!({"badMac":bad,"live":live,"moduleChecks":module_checks,"controlRoot":f.root,"dataRoot":data_root(id),"auditRoot":audit,"records":store.state.sequence,"sourceSha256":hex(&hash(&executable).unwrap())});
    fs::write(
        saved.join("run.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    drop(pins);
    drop(store);
    let reopened = Store::open(&f.root).unwrap();
    assert!(!reopened.state.c4_probe.pending());
    assert!(reopened.state.c4_probe.used_count_for_test() == 2);
    drop(reopened);
}

#[test]
fn persistent_profile_data_and_package_identity_survive_close_and_authenticated_reopen() {
    let f = Fixture::new();
    let store = f.prepared();
    let id = store.state.c2.as_ref().unwrap().id;
    let binding = store.state.c2.as_ref().unwrap().binding.clone().unwrap();
    let root = data_root(id);
    let key = hash(&f.root.join("control/authority.key")).unwrap();
    let head = store.state.head;
    assert!(present(&profile(id).unwrap()).unwrap());
    assert_eq!(store.state.schema, 6);
    assert!(Store::open(&f.root).is_err());
    let pins = store.verify_c2(f.actor, head).unwrap();
    assert!(
        OpenOptions::new()
            .write(true)
            .open(root.join("package/policy.json"))
            .is_err()
    );
    assert!(fs::rename(root.join("package/ä.txt"), root.join("package/other.txt")).is_err());
    drop(pins);
    fs::write(root.join("projects/synthetic.txt"), EDITED).unwrap();
    drop(store);
    let mut store = Store::open(&f.root).unwrap();
    let pins = store.verify_c2(f.actor, head).unwrap();
    assert!(present(&profile(id).unwrap()).unwrap());
    assert_eq!(
        store.state.c2.as_ref().unwrap().binding.as_ref(),
        Some(&binding)
    );
    assert_eq!(
        fs::read(root.join("projects/synthetic.txt")).unwrap(),
        EDITED
    );
    assert_eq!(hash(&f.root.join("control/authority.key")).unwrap(), key);
    assert_eq!(store.state.sequence, 3);
    drop(pins);
    store.retire_c2(f.actor, head).unwrap();
    assert!(!present(&profile(id).unwrap()).unwrap());
    assert!(root.join("projects/synthetic.txt").exists());
    assert!(store.verify_c2(f.actor, store.state.head).is_err());
    assert!(store.retire_c2(f.actor, store.state.head).is_err());
}

#[test]
fn native_inventory_rejects_content_extras_streams_hardlinks_and_unsafe_paths_without_new_records()
{
    let f = Fixture::new();
    let store = f.prepared();
    let root = data_root(store.state.c2.as_ref().unwrap().id);
    let head = store.state.head;
    let sequence = store.state.sequence;
    let policy = root.join("package/policy.json");
    let rejects = || {
        assert!(store.verify_c2(f.actor, head).is_err());
        assert_eq!(store.state.head, head);
        assert_eq!(store.state.sequence, sequence);
    };
    fs::write(&policy, b"changed").unwrap();
    rejects();
    fs::write(&policy, POLICY).unwrap();
    fs::write(root.join("package/extra.dll"), b"not declared").unwrap();
    rejects();
    fs::remove_file(root.join("package/extra.dll")).unwrap();
    fs::create_dir(root.join("package/extra")).unwrap();
    rejects();
    fs::remove_dir(root.join("package/extra")).unwrap();
    let ads = root.join("package/policy.json:unexpected");
    fs::write(&ads, b"hidden").unwrap();
    rejects();
    fs::remove_file(&ads).unwrap();
    let link = f.base.join("case-01/linked");
    fs::hard_link(&policy, &link).unwrap();
    rejects();
    fs::remove_file(&link).unwrap();
    assert!(ordinal_equal("ä.txt", "Ä.TXT").unwrap());
    assert!(!ordinal_equal("ä.txt", "a.txt").unwrap());
    assert!(
        verify_package(
            Path::new("\\\\example.invalid\\share"),
            &[InventoryEntry {
                path: "a".into(),
                size: 0,
                sha256: [1; 32],
                role: FileRole::Resource
            }],
            None
        )
        .is_err()
    );
    let _pins = store.verify_c2(f.actor, head).unwrap();
}

#[test]
fn root_identity_and_exact_profile_acl_are_checked_and_control_acl_remains_private() {
    let f = Fixture::new();
    let store = f.prepared();
    let state = store.state.c2.as_ref().unwrap();
    let root = data_root(state.id);
    let p = profile(state.id).unwrap();
    let head = store.state.head;
    private_acl(&f.root, true).unwrap();
    private_acl(&f.root.join("control/authority.key"), false).unwrap();
    let projects = root.join("projects");
    let old = root.join("old-projects");
    fs::rename(&projects, &old).unwrap();
    secure_directory(&projects).unwrap();
    set_profile_acl(&projects, true, p.sid, MODIFY).unwrap();
    assert!(store.verify_c2(f.actor, head).is_err());
    fs::remove_dir(&projects).unwrap();
    fs::rename(&old, &projects).unwrap();
    set_profile_acl(&root.join("package"), true, p.sid, MODIFY).unwrap();
    assert!(store.verify_c2(f.actor, head).is_err());
    set_profile_acl(&root.join("package"), true, p.sid, RX).unwrap();
    let _pins = store.verify_c2(f.actor, head).unwrap();
}

#[test]
fn c2_requires_fresh_schema_actor_head_budget_and_never_adopts_unconfirmed_preparation() {
    let f = Fixture::new();
    let mut store = f.init();
    let (_dirs, mut source, _sha) = binary_pin(&std::env::current_exe().unwrap()).unwrap();
    let head = store.state.head;
    assert!(store.prepare_c2([99; 32], head, &mut source).is_err());
    assert!(store.prepare_c2(f.actor, [99; 32], &mut source).is_err());
    assert!(store.state.c2.is_none());
    let sequence = store.state.sequence;
    store.state.sequence = MAX_RECORDS - 4;
    assert!(store.prepare_c2(f.actor, head, &mut source).is_err());
    assert!(store.state.c2.is_none());
    store.state.sequence = sequence;
    assert!(
        store
            .append(f.actor, head, Action::Migrate { from: 5, to: 6 })
            .is_err()
    );
    let id = Uuid::new_v4();
    store
        .append(f.actor, head, Action::PrepareC2Workspace { workspace: id })
        .unwrap();
    let sequence = store.state.sequence;
    let head = store.state.head;
    assert!(!present(&profile(id).unwrap()).unwrap());
    assert!(store.prepare_c2(f.actor, head, &mut source).is_err());
    assert!(store.verify_c2(f.actor, head).is_err());
    assert!(store.retire_c2(f.actor, head).is_err());
    assert_eq!(store.state.sequence, sequence);
    drop(store);
    let store = Store::open(&f.root).unwrap();
    assert!(store.verify_c2(f.actor, head).is_err());
    assert!(scope(Path::new("C:/not-a-synthetic-store")).is_err());
}
