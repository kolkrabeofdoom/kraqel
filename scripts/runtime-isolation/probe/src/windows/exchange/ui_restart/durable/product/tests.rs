use super::*;
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "kraqel-product-control-test-{}",
            Uuid::new_v4().simple()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn root(&self) -> PathBuf {
        self.0.join("store")
    }
    fn store(&self) -> Store {
        Store::initialize(&self.root(), [1; 32]).unwrap()
    }
    fn candidate(&self) -> PathBuf {
        let path = self.0.join("candidate.exe");
        fs::copy(std::env::current_exe().unwrap(), &path).unwrap();
        OpenOptions::new()
            .append(true)
            .open(&path)
            .unwrap()
            .write_all(b"KRAQEL synthetic update fixture")
            .unwrap();
        path
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
fn secure_store_is_exclusive_and_missing_keys_are_never_regenerated() {
    let f = Fixture::new();
    let store = f.store();
    assert!(Store::open(&f.root()).is_err());
    assert!(Store::initialize(&f.root(), [1; 32]).is_err());
    drop(store);
    let key = f.root().join("control/authority.key");
    fs::remove_file(&key).unwrap();
    assert!(Store::open(&f.root()).is_err());
    assert!(!key.exists());
}
#[test]
fn prepared_update_survives_reopen_and_only_explicit_activation_changes_actor() {
    let f = Fixture::new();
    let mut store = f.store();
    let candidate = f.candidate();
    let sha = hash(&candidate).unwrap();
    fs::write(
        f.root().join("data/consumed-task-sentinel"),
        b"already consumed; never reimport",
    )
    .unwrap();
    let key_hash = hash(&f.root().join("control/authority.key")).unwrap();
    let head = store.state.head;
    assert!(store.prepare([1; 32], head, &candidate, [7; 32]).is_err());
    assert_eq!(store.state.sequence, 0);
    assert!(store.prepare([1; 32], head, &candidate, sha).is_err());
    store
        .append([1; 32], head, Action::Migrate { from: 1, to: 2 })
        .unwrap();
    let head = store.state.head;
    store.prepare([1; 32], head, &candidate, sha).unwrap();
    drop(store);
    let mut store = Store::open(&f.root()).unwrap();
    assert_eq!(store.state.pending, Some(sha));
    assert_eq!(store.state.actor, [1; 32]);
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::Migrate { from: 1, to: 2 }
            )
            .is_err()
    );
    assert!(store.activate([1; 32], head).is_err());
    store.activate([1; 32], store.state.head).unwrap();
    drop(store);
    let mut store = Store::open(&f.root()).unwrap();
    assert!(store.actor([1; 32]).is_err());
    store.actor(sha).unwrap();
    let head = store.state.head;
    assert!(
        store
            .append(sha, head, Action::Migrate { from: 1, to: 2 })
            .is_err()
    );
    assert!(
        store
            .append(sha, head, Action::Migrate { from: 1, to: 2 })
            .is_err()
    );
    assert!(
        store
            .append(sha, store.state.head, Action::Migrate { from: 2, to: 1 })
            .is_err()
    );
    assert!(
        store
            .append(sha, store.state.head, Action::Migrate { from: 2, to: 4 })
            .is_err()
    );
    assert!(
        store
            .append(sha, store.state.head, Action::Prepare { binary: [1; 32] })
            .is_err()
    );
    assert_eq!(store.state.schema, 2);
    assert_eq!(
        key_hash,
        hash(&f.root().join("control/authority.key")).unwrap()
    );
    assert_eq!(
        fs::read(f.root().join("data/consumed-task-sentinel")).unwrap(),
        b"already consumed; never reimport"
    );
}
#[test]
fn tampered_candidate_never_activates_and_abort_does_not_reset_data_or_key() {
    let f = Fixture::new();
    let mut store = f.store();
    let candidate = f.candidate();
    let sha = hash(&candidate).unwrap();
    store
        .append(
            [1; 32],
            store.state.head,
            Action::Migrate { from: 1, to: 2 },
        )
        .unwrap();
    store
        .prepare([1; 32], store.state.head, &candidate, sha)
        .unwrap();
    fs::write(store.package(sha), b"tampered").unwrap();
    assert!(store.activate([1; 32], store.state.head).is_err());
    store
        .append([1; 32], store.state.head, Action::Abort)
        .unwrap();
    drop(store);
    let store = Store::open(&f.root()).unwrap();
    assert!(store.state.pending.is_none());
    assert_eq!(store.state.actor, [1; 32]);
}
#[test]
fn record_tampering_gaps_tail_loss_and_uncommitted_records_fail_closed() {
    for case in 0..4 {
        let f = Fixture::new();
        let mut store = f.store();
        store
            .append(
                [1; 32],
                store.state.head,
                Action::Migrate { from: 1, to: 2 },
            )
            .unwrap();
        drop(store);
        let last = f.root().join("control/records/0001.mac");
        match case {
            0 => {
                let mut bytes = fs::read(&last).unwrap();
                *bytes.last_mut().unwrap() ^= 1;
                fs::write(&last, bytes).unwrap();
            }
            1 => fs::rename(&last, last.with_file_name("0002.mac")).unwrap(),
            2 => fs::remove_file(&last).unwrap(),
            _ => {
                fs::write(f.root().join("control/head.next"), b"incomplete commit").unwrap();
            }
        }
        assert!(Store::open(&f.root()).is_err());
    }
    let f = Fixture::new();
    let store = f.store();
    let record = Record {
        version: 1,
        sequence: 1,
        installation: store.state.installation,
        root_sha256: root_hash(&store.root),
        previous_sha256: store.state.head,
        operation: Uuid::new_v4(),
        actor: [1; 32],
        action: Action::Migrate { from: 1, to: 2 },
    };
    write_record(
        &store.root,
        1,
        &sealed(&serde_json::to_vec(&record).unwrap(), &store.key).unwrap(),
    )
    .unwrap();
    drop(store);
    assert!(Store::open(&f.root()).is_err());
    assert!(f.root().join("control/records/0001.mac").exists());
}
#[test]
fn moved_roots_foreign_macs_legacy_seals_and_broadened_acls_are_rejected() {
    let f = Fixture::new();
    drop(f.store());
    let moved = f.0.join("moved");
    fs::rename(f.root(), &moved).unwrap();
    assert!(Store::open(&moved).is_err());
    let a = Fixture::new();
    let b = Fixture::new();
    drop(a.store());
    drop(b.store());
    fs::copy(
        a.root().join("control/records/0000.mac"),
        b.root().join("control/records/0000.mac"),
    )
    .unwrap();
    assert!(Store::open(&b.root()).is_err());
    let f = Fixture::new();
    drop(f.store());
    fs::write(
        f.root().join("control/records/0000.mac"),
        b"KRAQEL-BROKER-AUTHORITY\0\x03legacy research state",
    )
    .unwrap();
    assert!(Store::open(&f.root()).is_err());
    let f = Fixture::new();
    drop(f.store());
    let mut raw = null_mut();
    check(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide("D:P(A;OICI;FA;;;WD)").as_ptr(),
                1,
                &mut raw,
                null_mut(),
            )
        },
        "test-acl",
    )
    .unwrap();
    let sd = Local(raw);
    let mut acl = null_mut();
    let mut present = 0;
    let mut defaulted = 0;
    check(
        unsafe { GetSecurityDescriptorDacl(sd.0, &mut present, &mut acl, &mut defaulted) },
        "test-acl-dacl",
    )
    .unwrap();
    assert_eq!(
        unsafe {
            SetNamedSecurityInfoW(
                wide(f.root()).as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                null_mut(),
                null_mut(),
                acl,
                null(),
            )
        },
        0
    );
    assert!(Store::open(&f.root()).is_err());
}
#[test]
fn hardlinked_keys_and_remote_paths_are_rejected_before_state_changes() {
    for path in [
        r"\\server\share\folder",
        r"\\?\UNC\server\share",
        r"relative\folder",
        r"C:\folder\..\another",
    ] {
        assert!(pin_directories(Path::new(path)).is_err());
    }
    let f = Fixture::new();
    drop(f.store());
    fs::hard_link(
        f.root().join("control/authority.key"),
        f.0.join("alias.key"),
    )
    .unwrap();
    assert!(Store::open(&f.root()).is_err());
}
#[test]
fn product_and_research_mac_domains_are_not_interchangeable() {
    let key = [2; 32];
    let payload = b"bounded metadata";
    let wire = sealed(payload, &key).unwrap();
    assert_eq!(unsealed(&wire, &key).unwrap(), payload);
    assert!(unsealed(&wire, &[3; 32]).is_err());
    assert!(verify(&wire, &key).is_err());
    assert!(unsealed(&sign(payload, &key).unwrap(), &key).is_err());
}
#[test]
#[ignore = "bounded subprocess helper, invoked only by the active-job test"]
fn job_child() {
    assert_eq!(std::env::var("KRAQEL_PRODUCT_TEST_CHILD").unwrap(), "1");
    thread::sleep(Duration::from_secs(5));
}
#[test]
fn active_native_job_blocks_updates_and_migration_without_terminating_the_job() {
    let f = Fixture::new();
    let mut store = f.store();
    let name = wide(format!(
        "Local\\KRAQEL.Product.Broker.{}",
        store.state.installation.simple()
    ));
    let raw = unsafe { CreateJobObjectW(null(), name.as_ptr()) };
    assert!(!raw.is_null());
    assert_ne!(unsafe { GetLastError() }, ERROR_ALREADY_EXISTS);
    let job = Handle(raw);
    let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    check(
        unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        },
        "test-product-job-limit",
    )
    .unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "windows::exchange::ui_restart::durable::product::tests::job_child",
        ])
        .env("KRAQEL_PRODUCT_TEST_CHILD", "1")
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    check(
        unsafe { AssignProcessToJobObject(job.0, child.as_raw_handle()) },
        "test-product-job-assign",
    )
    .unwrap();
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::Migrate { from: 1, to: 2 }
            )
            .is_err()
    );
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::Prepare { binary: [2; 32] }
            )
            .is_err()
    );
    assert_eq!(store.state.sequence, 0);
    assert!(child.try_wait().unwrap().is_none());
    check(
        unsafe { TerminateJobObject(job.0, 77) },
        "test-product-job-stop",
    )
    .unwrap();
    child.wait().unwrap();
    store
        .append(
            [1; 32],
            store.state.head,
            Action::Migrate { from: 1, to: 2 },
        )
        .unwrap();
}

#[test]
fn starter_rejects_bootstrap_wrong_actor_stale_head_pending_and_wrong_app_without_mutation() {
    let f = Fixture::new();
    let mut store = f.store();
    assert!(store.start_guard([1; 32], store.state.head).is_err());
    store
        .append(
            [1; 32],
            store.state.head,
            Action::Migrate { from: 1, to: 2 },
        )
        .unwrap();
    let head = store.state.head;
    let key = hash(&f.root().join("control/authority.key")).unwrap();
    assert!(store.start_guard([2; 32], head).is_err());
    assert!(store.start_guard([1; 32], [0; 32]).is_err());
    assert!(
        store
            .start_check([1; 32], head, &std::env::current_exe().unwrap(), [0; 32])
            .is_err()
    );
    assert_eq!(store.state.head, head);
    store
        .append([1; 32], head, Action::Prepare { binary: [8; 32] })
        .unwrap();
    assert!(store.start_guard([1; 32], store.state.head).is_err());
    store
        .append([1; 32], store.state.head, Action::Abort)
        .unwrap();
    assert!(store.start_guard([1; 32], store.state.head).is_ok());
    assert_eq!(hash(&f.root().join("control/authority.key")).unwrap(), key);
    assert!(
        fs::read_dir(f.root().join("data"))
            .unwrap()
            .next()
            .is_none()
    );
}

#[test]
fn starter_rejects_even_empty_named_job_and_requires_a_held_lease_and_assigned_root() {
    let f = Fixture::new();
    let mut store = f.store();
    store
        .append(
            [1; 32],
            store.state.head,
            Action::Migrate { from: 1, to: 2 },
        )
        .unwrap();
    let name = store.job_name();
    let job = Handle(unsafe { CreateJobObjectW(null(), name.as_ptr()) });
    assert!(!job.0.is_null());
    assert!(store.idle().is_ok());
    assert_eq!(
        store
            .start_guard([1; 32], store.state.head)
            .unwrap_err()
            .stage,
        "product-start-job-collision"
    );
    assert_eq!(
        verify_start_lease(&f.root(), &name).unwrap_err().stage,
        "product-start-assigned-job-count"
    );
    drop(job);
    assert!(store.start_guard([1; 32], store.state.head).is_ok());
    drop(store);
    assert_eq!(
        verify_start_lease(&f.root(), &name).unwrap_err().stage,
        "product-start-lease-not-held"
    );
}

#[test]
#[ignore = "bounded subprocess helper, invoked only by the starter lease test"]
fn starter_lock_child() {
    let root = PathBuf::from(std::env::var_os("KRAQEL_SYNTHETIC_STARTER_LOCK_ROOT").unwrap());
    match Store::open(&root) {
        Err(error) => assert_eq!(error.code, i64::from(ERROR_SHARING_VIOLATION)),
        Ok(_) => panic!("competing process acquired the starter control lease"),
    }
}

#[test]
fn starter_control_lease_excludes_an_independent_controller_process() {
    let f = Fixture::new();
    let _store = f.store();
    let result = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "windows::exchange::ui_restart::durable::product::tests::starter_lock_child",
            "--ignored",
        ])
        .env("KRAQEL_SYNTHETIC_STARTER_LOCK_ROOT", f.root())
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .unwrap();
    assert!(result.status.success(), "independent lease check failed");
}

fn session_binding() -> Binding {
    serde_json::from_value(json!({"session":Uuid::new_v4(),"task":Uuid::new_v4(),
        "launch":Uuid::new_v4(),"app":vec![9u8;32],"root":vec![8u8;32],"point":"completed"}))
    .unwrap()
}

#[test]
fn session_schema_binding_and_recovery_survive_authenticated_reopen() {
    let f = Fixture::new();
    let mut store = f.store();
    let b = session_binding();
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::BindSession { binding: b.clone() }
            )
            .is_err()
    );
    store
        .append(
            [1; 32],
            store.state.head,
            Action::Migrate { from: 1, to: 2 },
        )
        .unwrap();
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::BindSession { binding: b.clone() }
            )
            .is_err()
    );
    store
        .append(
            [1; 32],
            store.state.head,
            Action::Migrate { from: 2, to: 3 },
        )
        .unwrap();
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::Migrate { from: 3, to: 2 }
            )
            .is_err()
    );
    store
        .append(
            [1; 32],
            store.state.head,
            Action::BindSession { binding: b.clone() },
        )
        .unwrap();
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::BindSession {
                    binding: session_binding()
                }
            )
            .is_err()
    );
    let launch = Uuid::new_v4();
    store
        .append([1; 32], store.state.head, Action::RecoverSession { launch })
        .unwrap();
    let head = store.state.head;
    drop(store);
    let mut store = Store::open(&f.root()).unwrap();
    assert_eq!(store.state.schema, 3);
    assert_eq!(store.state.head, head);
    assert!(
        store
            .append([1; 32], head, Action::RecoverSession { launch })
            .is_err()
    );
    assert!(
        store
            .append(
                [1; 32],
                head,
                Action::RecoverSession {
                    launch: Uuid::nil()
                }
            )
            .is_err()
    );
    assert_eq!(store.state.head, head);
    // A root/package unavailable under the authenticated binding cannot become a fresh import.
    assert!(store.recover_session([1; 32], head).is_err());
    assert_eq!(store.state.head, head);
}

#[test]
fn session_retirement_replay_budget_and_stale_heads_never_reset_control() {
    let f = Fixture::new();
    let mut store = f.store();
    store
        .append(
            [1; 32],
            store.state.head,
            Action::Migrate { from: 1, to: 2 },
        )
        .unwrap();
    store
        .append(
            [1; 32],
            store.state.head,
            Action::Migrate { from: 2, to: 3 },
        )
        .unwrap();
    let stale = store.state.head;
    store
        .append(
            [1; 32],
            stale,
            Action::BindSession {
                binding: session_binding(),
            },
        )
        .unwrap();
    assert!(
        store
            .append(
                [1; 32],
                stale,
                Action::RecoverSession {
                    launch: Uuid::new_v4()
                }
            )
            .is_err()
    );
    let key = hash(&f.root().join("control/authority.key")).unwrap();
    for _ in 1..16 {
        store
            .append(
                [1; 32],
                store.state.head,
                Action::RecoverSession {
                    launch: Uuid::new_v4(),
                },
            )
            .unwrap();
    }
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::RecoverSession {
                    launch: Uuid::new_v4()
                }
            )
            .is_err()
    );
    store
        .append([1; 32], store.state.head, Action::RetireSession)
        .unwrap();
    let head = store.state.head;
    drop(store);
    let mut store = Store::open(&f.root()).unwrap();
    assert!(
        store
            .append(
                [1; 32],
                head,
                Action::RecoverSession {
                    launch: Uuid::new_v4()
                }
            )
            .is_err()
    );
    assert!(
        store
            .append(
                [1; 32],
                head,
                Action::BindSession {
                    binding: session_binding()
                }
            )
            .is_err()
    );
    assert_eq!(store.state.head, head);
    assert_eq!(hash(&f.root().join("control/authority.key")).unwrap(), key);
}

#[test]
fn schema4_requires_matching_preparation_and_never_recovers_a_prepared_or_retired_session() {
    let f = Fixture::new();
    let mut store = f.store();
    for from in 1..4 {
        store
            .append(
                [1; 32],
                store.state.head,
                Action::Migrate { from, to: from + 1 },
            )
            .unwrap();
    }
    let binding = session_binding();
    assert!(
        store
            .append(
                [1; 32],
                store.state.head,
                Action::BindSession {
                    binding: binding.clone()
                }
            )
            .is_err()
    );
    store
        .append(
            [1; 32],
            store.state.head,
            Action::PrepareSession {
                binding: binding.clone(),
            },
        )
        .unwrap();
    let head = store.state.head;
    drop(store);
    let mut store = Store::open(&f.root()).unwrap();
    assert!(
        store
            .append(
                [1; 32],
                head,
                Action::RecoverSession {
                    launch: Uuid::new_v4()
                }
            )
            .is_err()
    );
    assert!(
        store
            .append(
                [1; 32],
                head,
                Action::BindSession {
                    binding: session_binding()
                }
            )
            .is_err()
    );
    assert!(store.cleanup_profile([1; 32], head).is_err());
    store.append([1; 32], head, Action::RetireSession).unwrap();
    store
        .append([1; 32], store.state.head, Action::ProfileCleaned)
        .unwrap();
    let head = store.state.head;
    drop(store);
    let mut store = Store::open(&f.root()).unwrap();
    assert!(
        store
            .append([1; 32], head, Action::BindSession { binding })
            .is_err()
    );
    assert!(
        store
            .append(
                [1; 32],
                head,
                Action::RecoverSession {
                    launch: Uuid::new_v4()
                }
            )
            .is_err()
    );
    assert!(store.append([1; 32], head, Action::ProfileCleaned).is_err());
    assert_eq!(store.state.head, head);
}

#[test]
fn cleanup_rejects_unbound_active_wrong_actor_and_stale_head_before_os_calls() {
    let f = Fixture::new();
    let mut store = f.store();
    for from in 1..4 {
        store
            .append(
                [1; 32],
                store.state.head,
                Action::Migrate { from, to: from + 1 },
            )
            .unwrap();
    }
    let head = store.state.head;
    let key = hash(&f.root().join("control/authority.key")).unwrap();
    assert!(store.cleanup_profile([1; 32], head).is_err());
    let binding = session_binding();
    store
        .append(
            [1; 32],
            head,
            Action::PrepareSession {
                binding: binding.clone(),
            },
        )
        .unwrap();
    store
        .append([1; 32], store.state.head, Action::BindSession { binding })
        .unwrap();
    let active = store.state.head;
    assert!(store.cleanup_profile([1; 32], active).is_err());
    assert!(store.cleanup_profile([2; 32], active).is_err());
    assert!(store.cleanup_profile([1; 32], head).is_err());
    assert_eq!(store.state.head, active);
    assert_eq!(hash(&f.root().join("control/authority.key")).unwrap(), key);
}

#[test]
fn schema4_updates_preserve_bound_state_key_data_and_allow_abort_without_new_authority() {
    for activate in [true, false] {
        let f = Fixture::new();
        let mut store = f.store();
        for from in 1..4 {
            store
                .append(
                    [1; 32],
                    store.state.head,
                    Action::Migrate { from, to: from + 1 },
                )
                .unwrap();
        }
        let binding = session_binding();
        store
            .append(
                [1; 32],
                store.state.head,
                Action::PrepareSession {
                    binding: binding.clone(),
                },
            )
            .unwrap();
        store
            .append([1; 32], store.state.head, Action::BindSession { binding })
            .unwrap();
        fs::write(
            f.root().join("data/consumed-sentinel"),
            b"consumed; never reset",
        )
        .unwrap();
        let key = hash(&f.root().join("control/authority.key")).unwrap();
        let candidate = f.candidate();
        let sha = hash(&candidate).unwrap();
        let stale = store.state.head;
        store.prepare([1; 32], stale, &candidate, sha).unwrap();
        assert!(store.state.session.is_some());
        assert!(
            store
                .append(
                    [1; 32],
                    store.state.head,
                    Action::RecoverSession {
                        launch: Uuid::new_v4()
                    }
                )
                .is_err()
        );
        assert!(store.activate([1; 32], stale).is_err());
        if activate {
            store.activate([1; 32], store.state.head).unwrap();
            assert!(store.actor([1; 32]).is_err());
            assert!(store.actor(sha).is_ok());
        } else {
            OpenOptions::new()
                .append(true)
                .open(store.package(sha))
                .unwrap()
                .write_all(b"tamper")
                .unwrap();
            assert!(store.activate([1; 32], store.state.head).is_err());
            store
                .append([1; 32], store.state.head, Action::Abort)
                .unwrap();
            assert!(store.actor([1; 32]).is_ok());
        }
        drop(store);
        let store = Store::open(&f.root()).unwrap();
        assert_eq!(store.state.schema, 4);
        assert!(store.state.pending.is_none());
        assert!(store.state.session.is_some());
        assert_eq!(hash(&f.root().join("control/authority.key")).unwrap(), key);
        assert_eq!(
            fs::read(f.root().join("data/consumed-sentinel")).unwrap(),
            b"consumed; never reset"
        );
    }
}

fn schema4_pending(f: &Fixture) -> Store {
    let mut store = f.store();
    for from in 1..4 {
        store
            .append(
                [1; 32],
                store.state.head,
                Action::Migrate { from, to: from + 1 },
            )
            .unwrap();
    }
    let binding = session_binding();
    store
        .append(
            [1; 32],
            store.state.head,
            Action::PrepareSession {
                binding: binding.clone(),
            },
        )
        .unwrap();
    store
        .append([1; 32], store.state.head, Action::BindSession { binding })
        .unwrap();
    let candidate = f.candidate();
    store
        .prepare(
            [1; 32],
            store.state.head,
            &candidate,
            hash(&candidate).unwrap(),
        )
        .unwrap();
    store
}

#[test]
fn schema4_bad_pending_package_requires_authenticated_abort_and_keeps_the_lease() {
    for fault in ["missing", "empty", "partial", "tampered"] {
        let f = Fixture::new();
        let store = schema4_pending(&f);
        let package = store.package(store.state.pending.unwrap());
        let key = hash(&f.root().join("control/authority.key")).unwrap();
        fs::write(f.root().join("data/consumed"), b"one consumed task").unwrap();
        drop(store);
        match fault {
            "missing" => fs::remove_file(&package).unwrap(),
            "empty" => OpenOptions::new()
                .write(true)
                .open(&package)
                .unwrap()
                .set_len(0)
                .unwrap(),
            "partial" => OpenOptions::new()
                .write(true)
                .open(&package)
                .unwrap()
                .set_len(1024)
                .unwrap(),
            _ => fs::write(&package, b"tampered").unwrap(),
        }
        let mut store = Store::open(&f.root()).unwrap();
        let pending_head = store.state.head;
        let records = fs::read_dir(f.root().join("control/records"))
            .unwrap()
            .count();
        assert!(Store::open(&f.root()).is_err());
        assert!(store.activate([1; 32], pending_head).is_err());
        assert!(store.append([2; 32], pending_head, Action::Abort).is_err());
        assert!(store.append([1; 32], [0; 32], Action::Abort).is_err());
        assert_eq!(store.state.head, pending_head);
        assert_eq!(
            fs::read_dir(f.root().join("control/records"))
                .unwrap()
                .count(),
            records
        );
        store.append([1; 32], pending_head, Action::Abort).unwrap();
        let terminal_head = store.state.head;
        assert!(Store::open(&f.root()).is_err());
        assert!(store.append([1; 32], terminal_head, Action::Abort).is_err());
        assert!(store.activate([1; 32], terminal_head).is_err());
        assert_eq!(store.state.head, terminal_head);
        drop(store);
        let store = Store::open(&f.root()).unwrap();
        assert!(store.state.pending.is_none());
        assert_eq!(store.state.actor, [1; 32]);
        assert!(store.state.session.is_some());
        assert_eq!(hash(&f.root().join("control/authority.key")).unwrap(), key);
        assert_eq!(
            fs::read(f.root().join("data/consumed")).unwrap(),
            b"one consumed task"
        );
    }
}

#[test]
fn schema4_uncommitted_abort_stays_blocked_without_head_or_key_repair() {
    let f = Fixture::new();
    let store = schema4_pending(&f);
    let key = hash(&f.root().join("control/authority.key")).unwrap();
    let head = hash(&f.root().join("control/head.mac")).unwrap();
    let record = Record {
        version: 1,
        sequence: store.state.sequence + 1,
        installation: store.state.installation,
        root_sha256: root_hash(&store.root),
        previous_sha256: store.state.head,
        operation: Uuid::new_v4(),
        actor: store.state.actor,
        action: Action::Abort,
    };
    let wire = sealed(&serde_json::to_vec(&record).unwrap(), &store.key).unwrap();
    write_record(&store.root, record.sequence, &wire).unwrap();
    drop(store);
    for _ in 0..2 {
        assert!(Store::open(&f.root()).is_err());
        assert_eq!(hash(&f.root().join("control/head.mac")).unwrap(), head);
        assert_eq!(hash(&f.root().join("control/authority.key")).unwrap(), key);
        assert_eq!(
            fs::read(
                f.root()
                    .join(format!("control/records/{:04}.mac", record.sequence))
            )
            .unwrap(),
            wire
        );
    }
}

#[test]
fn schema4_retry_replaces_only_aborted_packages_before_new_pending() {
    for fault in ["complete", "missing", "empty", "partial", "tampered"] {
        let f = Fixture::new();
        let mut store = schema4_pending(&f);
        let sha = store.state.pending.unwrap();
        let path = store.package(sha);
        let stale = store.state.head;
        store.append([1; 32], stale, Action::Abort).unwrap();
        let head = store.state.head;
        let key = hash(&f.root().join("control/authority.key")).unwrap();
        match fault {
            "missing" => fs::remove_file(&path).unwrap(),
            "empty" => OpenOptions::new()
                .write(true)
                .open(&path)
                .unwrap()
                .set_len(0)
                .unwrap(),
            "partial" => OpenOptions::new()
                .write(true)
                .open(&path)
                .unwrap()
                .set_len(1024)
                .unwrap(),
            "tampered" => fs::write(&path, b"synthetic corruption").unwrap(),
            _ => {}
        }
        drop(store);
        let mut store = Store::open(&f.root()).unwrap();
        let candidate = f.0.join("candidate.exe");
        assert!(store.prepare([1; 32], stale, &candidate, sha).is_err());
        assert!(store.prepare([2; 32], head, &candidate, sha).is_err());
        if fault != "missing" {
            let locked = protected_file(&path).unwrap();
            let before = hash(&path).unwrap();
            assert!(store.prepare([1; 32], head, &candidate, sha).is_err());
            assert_eq!(hash(&path).unwrap(), before);
            drop(locked);
        }
        assert_eq!(store.state.head, head);
        assert!(store.state.pending.is_none());
        store.prepare([1; 32], head, &candidate, sha).unwrap();
        assert_eq!(hash(&path).unwrap(), sha);
        assert_eq!(store.state.pending, Some(sha));
        assert_eq!(hash(&f.root().join("control/authority.key")).unwrap(), key);
        store.activate([1; 32], store.state.head).unwrap();
        let activated_head = store.state.head;
        assert!(store.prepare(sha, activated_head, &candidate, sha).is_err());
        assert_eq!(store.state.head, activated_head);
        assert_eq!(hash(&path).unwrap(), sha);
    }
}

#[test]
fn unrecorded_package_collision_never_signs_pending_or_deletes_bytes() {
    let f = Fixture::new();
    let mut store = f.store();
    for from in 1..4 {
        store
            .append(
                [1; 32],
                store.state.head,
                Action::Migrate { from, to: from + 1 },
            )
            .unwrap();
    }
    let candidate = f.candidate();
    let sha = hash(&candidate).unwrap();
    fs::copy(&candidate, store.package(sha)).unwrap();
    let head = store.state.head;
    assert!(store.prepare([1; 32], head, &candidate, sha).is_err());
    assert_eq!(store.state.head, head);
    assert!(store.state.pending.is_none());
    assert_eq!(hash(&store.package(sha)).unwrap(), sha);
    drop(store);
    let store = Store::open(&f.root()).unwrap();
    assert_eq!(store.state.head, head);
    assert!(store.state.pending.is_none());
}

#[test]
fn aborted_package_hardlink_is_not_deleted_or_reauthorized() {
    let f = Fixture::new();
    let mut store = schema4_pending(&f);
    let sha = store.state.pending.unwrap();
    store
        .append([1; 32], store.state.head, Action::Abort)
        .unwrap();
    let path = store.package(sha);
    let other = f.0.join("must-survive.exe");
    fs::hard_link(&path, &other).unwrap();
    let head = store.state.head;
    assert!(
        store
            .prepare([1; 32], head, &f.0.join("candidate.exe"), sha)
            .is_err()
    );
    assert_eq!(store.state.head, head);
    assert!(store.state.pending.is_none());
    assert_eq!(hash(&path).unwrap(), sha);
    assert_eq!(hash(&other).unwrap(), sha);
}

#[test]
fn win32_package_binding_is_durable_and_cannot_cross_activation() {
    use win32_binding::Operation::{Bind, Verify};
    let f = Fixture::new();
    let mut store = schema4_pending(&f);
    let actor = store.state.pending.unwrap();
    store.activate([1; 32], store.state.head).unwrap();
    let dir = f.0.join("source-win32");
    secure_directory(&dir).unwrap();
    let mut entries = Vec::new();
    for name in crate::native_win32_package::IMAGES {
        let path = dir.join(name);
        fs::write(&path, name).unwrap();
        entries.push(json!({"name":name,"hash":hex(&hash(&path).unwrap())}));
    }
    let manifest = dir.join("anchor.json");
    fs::write(&manifest, json!({"files":entries}).to_string()).unwrap();
    let digest = hash(&manifest).unwrap();
    let head = store.state.head;
    assert!(
        store
            .win32_binding(actor, head, Bind(&dir, [9; 32]))
            .is_err()
    );
    assert_eq!(store.state.head, head);
    store
        .win32_binding(actor, head, Bind(&dir, digest))
        .unwrap();
    drop(store);
    let mut store = Store::open(&f.root()).unwrap();
    store
        .win32_binding(actor, store.state.head, Verify)
        .unwrap();
    assert!(store.win32_binding(actor, head, Verify).is_err());
    let managed_manifest = store
        .root
        .join("bin")
        .join(format!("win32-{}", hex(&actor)))
        .join("anchor.json");
    fs::OpenOptions::new()
        .append(true)
        .open(&managed_manifest)
        .unwrap()
        .write_all(b" ")
        .unwrap();
    assert!(
        store
            .win32_binding(actor, store.state.head, Verify)
            .is_err()
    );
    fs::copy(&manifest, &managed_manifest).unwrap();
    let next = f.0.join("next.exe");
    fs::copy(f.candidate(), &next).unwrap();
    OpenOptions::new()
        .append(true)
        .open(&next)
        .unwrap()
        .write_all(b"next")
        .unwrap();
    // An older record reader must not be activated after enrollment.
    let incompatible = f.0.join("incompatible.exe");
    let mut bytes = fs::read(&next).unwrap();
    let section = bytes
        .windows(8)
        .position(|name| name == b".kqwin\0\0")
        .unwrap();
    bytes[section + 1] = b'z';
    fs::write(&incompatible, bytes).unwrap();
    let head = store.state.head;
    assert!(
        store
            .prepare(actor, head, &incompatible, hash(&incompatible).unwrap())
            .is_err()
    );
    assert_eq!(store.state.head, head);
    let next_actor = hash(&next).unwrap();
    store
        .prepare(actor, store.state.head, &next, next_actor)
        .unwrap();
    assert!(
        store
            .win32_binding(actor, store.state.head, Verify)
            .is_err()
    );
    store.activate(actor, store.state.head).unwrap();
    assert_eq!(store.state.win32_anchor, Some((actor, digest)));
    assert!(
        store
            .win32_binding(next_actor, store.state.head, Verify)
            .is_err()
    );
    assert!(
        store
            .win32_binding(actor, store.state.head, Bind(&dir, digest))
            .is_err()
    );
}
