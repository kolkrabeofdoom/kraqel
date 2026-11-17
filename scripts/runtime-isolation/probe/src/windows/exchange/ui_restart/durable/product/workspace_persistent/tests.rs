use super::*;

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
