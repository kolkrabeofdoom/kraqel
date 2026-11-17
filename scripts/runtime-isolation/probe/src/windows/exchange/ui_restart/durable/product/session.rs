//! Native control-record authority for one persistent synthetic session per installation.
use super::*;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Binding {
    session: Uuid,
    task: Uuid,
    launch: Uuid,
    app: [u8; 32],
    root: [u8; 32],
    point: CrashPoint,
}
#[derive(Clone)]
pub(super) struct BoundSession {
    binding: Binding,
    launches: Vec<Uuid>,
    retired: bool,
}
pub(super) fn apply(state: &mut State, action: &Action) -> Result<()> {
    if state.schema != 3 || state.pending.is_some() {
        return Err(invalid("product-session-schema-or-pending"));
    }
    match action {
        Action::BindSession { binding }
            if state.session.is_none()
                && !binding.session.is_nil()
                && !binding.task.is_nil()
                && !binding.launch.is_nil()
                && binding.app != [0; 32]
                && binding.root != [0; 32] =>
        {
            state.session = Some(BoundSession {
                binding: binding.clone(),
                launches: vec![binding.launch],
                retired: false,
            });
        }
        Action::RecoverSession { launch } => {
            let s = state
                .session
                .as_mut()
                .ok_or(invalid("product-session-unbound"))?;
            if s.retired || launch.is_nil() || s.launches.contains(launch) || s.launches.len() >= 16
            {
                return Err(invalid("product-session-replay-or-retired"));
            }
            s.launches.push(*launch);
        }
        Action::RetireSession => {
            let s = state
                .session
                .as_mut()
                .ok_or(invalid("product-session-unbound"))?;
            if s.retired {
                return Err(invalid("product-session-retired"));
            }
            s.retired = true;
        }
        _ => return Err(invalid("product-session-transition")),
    }
    Ok(())
}

impl Store {
    fn session_guard(&self, actor: [u8; 32], head: [u8; 32]) -> Result<()> {
        self.actor(actor)?;
        if self.state.schema != 3 || self.state.pending.is_some() || self.state.head != head {
            return Err(invalid("product-session-state"));
        }
        self.idle()?;
        let raw = unsafe { OpenJobObjectW(JOB_OBJECT_QUERY, 0, self.job_name().as_ptr()) };
        if !raw.is_null() {
            drop(Handle(raw));
            return Err(invalid("product-session-job-collision"));
        }
        if unsafe { GetLastError() } != ERROR_FILE_NOT_FOUND {
            return Err(fail("product-session-job-open"));
        }
        Ok(())
    }
    fn session_parent(&self) -> Result<PathBuf> {
        Ok(self
            .root
            .parent()
            .ok_or(invalid("product-session-parent"))?
            .join(".kraqel-app-preview")
            .join(self.state.installation.simple().to_string()))
    }
    fn session_root(&self, b: &Binding) -> Result<PathBuf> {
        Ok(self.session_parent()?.join(format!(
            "kraqel-isolation-ui-restart-{}",
            b.session.simple()
        )))
    }
    pub(super) fn bind_session(
        &mut self,
        actor: [u8; 32],
        head: [u8; 32],
        app: &Path,
        expected: [u8; 32],
        point: CrashPoint,
    ) -> Result<Value> {
        self.bind_session_inner(actor, head, app, expected, point, false)
    }
    fn bind_session_inner(
        &mut self,
        actor: [u8; 32],
        head: [u8; 32],
        app: &Path,
        expected: [u8; 32],
        point: CrashPoint,
        crash_controller: bool,
    ) -> Result<Value> {
        self.session_guard(actor, head)?;
        if self.state.session.is_some() {
            return Err(invalid("product-session-already-bound"));
        }
        let (_parents, _source, actual) = binary_pin(app)?;
        if actual != expected {
            return Err(invalid("product-session-app-hash"));
        }
        let mut kind = 0;
        check(
            unsafe { GetBinaryTypeW(path_wide(app)?.as_ptr(), &mut kind) },
            "product-session-app-pe",
        )?;
        if kind != 6 {
            return Err(invalid("product-session-app-x64"));
        }
        let parent = self.session_parent()?;
        let base = parent.parent().ok_or(invalid("product-session-base"))?;
        let _profile_pins = pin_directories(
            base.parent()
                .ok_or(invalid("product-session-base-parent"))?,
        )?;
        if base.exists() {
            private_acl(base, true)?;
        } else {
            secure_directory(base)?;
        }
        let _base_pins = pin_directories(base)?;
        secure_directory(&parent)?;
        let _parent_pins = pin_directories(&parent)?;
        let mut binding = Binding {
            session: Uuid::new_v4(),
            task: Uuid::new_v4(),
            launch: Uuid::new_v4(),
            app: expected,
            root: [0; 32],
            point,
        };
        let root = self.session_root(&binding)?;
        let (_, mut profile, pin) =
            super::super::super::initialize_bound(&root, app, binding.session)?;
        if pin != expected {
            return Err(invalid("product-session-staged-app"));
        }
        let _pins = pin_directories(&root)?;
        binding.root = root_hash(&io("product-session-canonical", root.canonicalize())?);
        // Once committing authority starts, failures must retain the profile, never reset it.
        profile.active = false;
        self.append(actor, head, Action::BindSession { binding })?;
        self.run_session(false, crash_controller)
    }
    pub(super) fn recover_session(&mut self, actor: [u8; 32], head: [u8; 32]) -> Result<Value> {
        self.actor(actor)?;
        if self.state.schema != 3 || self.state.head != head || self.state.pending.is_some() {
            return Err(invalid("product-session-state"));
        }
        // Old kill-on-close processes may still be draining after controller death. Wait only;
        // never terminate/adopt a foreign job, and never release the control lease meanwhile.
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let raw = unsafe { OpenJobObjectW(JOB_OBJECT_QUERY, 0, self.job_name().as_ptr()) };
            if raw.is_null() {
                if unsafe { GetLastError() } != ERROR_FILE_NOT_FOUND {
                    return Err(fail("product-session-old-job"));
                }
                break;
            }
            drop(Handle(raw));
            if Instant::now() >= deadline {
                return Err(invalid("product-session-old-job-not-gone"));
            }
            thread::sleep(Duration::from_millis(20));
        }
        self.session_guard(actor, head)?;
        let s = self
            .state
            .session
            .as_ref()
            .ok_or(invalid("product-session-unbound"))?;
        if s.retired {
            return Err(invalid("product-session-retired"));
        }
        let root = self.session_root(&s.binding)?;
        // Never create journals/profile/data while taking over. Validate package before recording
        // a launch; then consume a fresh launch record before starting any child process.
        let _pins = pin_directories(&root)?;
        self.verify_session(&root, &s.binding)?;
        for relative in [
            "data/app/tasks/tasks.sqlite3",
            "data/app/tasks/owner.sqlite3",
        ] {
            let path = root.join(relative);
            let _dirs = pin_directories(
                path.parent()
                    .ok_or(invalid("product-session-journal-parent"))?,
            )?;
            let _journal = open_file(&path, false, false)?;
        }
        self.append(
            actor,
            head,
            Action::RecoverSession {
                launch: Uuid::new_v4(),
            },
        )?;
        self.run_session(true, false)
    }
    fn verify_session(&self, root: &Path, b: &Binding) -> Result<()> {
        if root_hash(&io("product-session-root-canonical", root.canonicalize())?) != b.root {
            return Err(invalid("product-session-root-binding"));
        }
        private_acl(&self.session_parent()?, true)?;
        private_acl(self.session_parent()?.parent().unwrap(), true)?;
        let (_dirs, _file, actual) = binary_pin(&root.join("bin/kraqel-desktop.exe"))?;
        if actual != b.app {
            return Err(invalid("product-session-package-changed"));
        }
        Ok(())
    }
    fn run_session(&self, recovery: bool, crash_controller: bool) -> Result<Value> {
        let s = self
            .state
            .session
            .as_ref()
            .ok_or(invalid("product-session-unbound"))?;
        let b = &s.binding;
        let root = self.session_root(b)?;
        let _pins = pin_directories(&root)?;
        self.verify_session(&root, b)?;
        let (_app_dirs, _app_file, actual) = binary_pin(&root.join("bin/kraqel-desktop.exe"))?;
        if actual != b.app {
            return Err(invalid("product-session-package-changed"));
        }
        let profile = super::super::reopen_profile(b.session)?;
        let ticket = RestartTicket {
            version: RestartTicket::VERSION,
            session_id: b.session,
            task_id: b.task,
            launch_id: *s.launches.last().unwrap(),
            recovery,
            automatic: true,
            source: kraqel_isolation_transport::RestartSource::Synthetic,
            point: b.point,
            app_sha256: b.app,
        };
        let name = self.job_name();
        let index = s.launches.len() - 1;
        let summary = generation_with_options(
            &root,
            &profile,
            &ticket,
            ticket.launch_id,
            GenerationOptions {
                expect: if recovery {
                    Expect::Recovery
                } else {
                    Expect::Crash
                },
                index,
                durable_job: Some(&name),
                crash_controller,
                live_mode: None,
                product_root: Some(&self.root),
            },
        )?;
        if recovery
            && !summary.as_ref().is_some_and(|s| {
                super::super::super::valid_summary(s, b.point, u32::from(b.point.publishes()))
            })
        {
            return Err(invalid("product-session-recovery-summary"));
        }
        self.idle()?;
        Ok(
            json!({"nativePreviewOnly":true,"productionAcceptance":false,"generation":index,
            "recovery":recovery,"summary":summary,"localSessionRoot":root,
            "controlLeaseHeld":true,"namedProductJob":true,"activeJobProcessesAtEnd":0,
            "profileRetained":true,"actualContainerDenials":6,"networkMeasurement":false}),
        )
    }
    pub(super) fn retire_session(&mut self, actor: [u8; 32], head: [u8; 32]) -> Result<Value> {
        self.session_guard(actor, head)?;
        let s = self
            .state
            .session
            .as_ref()
            .ok_or(invalid("product-session-unbound"))?;
        let id = s.binding.session;
        self.append(actor, head, Action::RetireSession)?;
        let mut profile = super::super::reopen_profile(id)?;
        if !profile.remove() {
            return Err(invalid("product-session-retire-profile"));
        }
        Ok(
            json!({"nativePreviewOnly":true,"productionAcceptance":false,"retired":true,
            "profileRemoved":true,"activeJobProcessesAtEnd":0,"networkMeasurement":false}),
        )
    }
}

pub(in crate::windows) fn child(args: &[OsString]) -> Result<Value> {
    if args.len() != 5 {
        return Err(invalid("product-session-child-args"));
    }
    let root = Path::new(&args[0]);
    super::crash::scoped(root)?;
    let action = args[1]
        .to_str()
        .ok_or(invalid("product-session-child-action"))?;
    let app = Path::new(&args[2]);
    let expected = parse_hash(&args[3])?;
    let head = parse_hash(&args[4])?;
    let exe = io("product-session-exe", std::env::current_exe())?;
    let (_dirs, _file, actor) = binary_pin(&exe)?;
    if action == "init" {
        return Ok(json!({"control":Store::initialize(root,actor)?.summary()}));
    }
    let mut store = Store::open(root)?;
    store.actor(actor)?;
    let outcome = match action {
        "migrate" => {
            store.append(actor, head, Action::Migrate { from: 1, to: 2 })?;
            store.append(actor, store.state.head, Action::Migrate { from: 2, to: 3 })?;
            json!(null)
        }
        "prepared" => {
            store.bind_session_inner(actor, head, app, expected, CrashPoint::Prepared, true)?
        }
        "completed" => {
            store.bind_session_inner(actor, head, app, expected, CrashPoint::Completed, true)?
        }
        "recover" => store.recover_session(actor, head)?,
        "retire" => store.retire_session(actor, head)?,
        _ => return Err(invalid("product-session-child-action")),
    };
    Ok(json!({"control":store.summary(),"outcome":outcome}))
}

pub(in crate::windows) fn series(
    root: &Path,
    app: &Path,
    expected: &std::ffi::OsStr,
) -> Result<Value> {
    if !crate::policy::valid_root(root, false)
        || !root
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("kraqel-isolation-controller-"))
    {
        return Err(invalid("product-session-series-root"));
    }
    let expected = parse_hash(expected)?;
    let exe = io("product-session-series-exe", std::env::current_exe())?;
    let actor = hash(&exe)?;
    let _parents = pin_directories(
        root.parent()
            .ok_or(invalid("product-session-series-parent"))?,
    )?;
    secure_directory(root)?;
    let mut processes = Vec::new();
    let mut held = Vec::new();
    let mut cases = Vec::new();
    for (index, point) in ["prepared", "completed"].iter().enumerate() {
        let case = root.join(format!("case-{index:02}"));
        secure_directory(&case)?;
        let store_root = case.join("store");
        let mut invoke = |action: &str, head: [u8; 32], expected_code: i32| -> Result<Value> {
            let ordinal = processes.len();
            let out = root.join(format!("{ordinal:02}.stdout"));
            let mut process = io(
                "product-session-series-spawn",
                Command::new(&exe)
                    .arg("product-session-child")
                    .arg(&store_root)
                    .arg(action)
                    .arg(app)
                    .arg(hex(&expected))
                    .arg(hex(&head))
                    .creation_flags(CREATE_NO_WINDOW)
                    .stdout(io("product-session-out", fs::File::create(&out))?)
                    .stderr(io(
                        "product-session-err",
                        fs::File::create(root.join(format!("{ordinal:02}.stderr"))),
                    )?)
                    .spawn(),
            )?;
            let pid = process.id();
            let end = Instant::now() + Duration::from_secs(75);
            let code = loop {
                if let Some(status) = io("product-session-wait", process.try_wait())? {
                    break status.code().unwrap_or(-1);
                }
                if Instant::now() > end {
                    let _ = process.kill();
                    let _ = process.wait();
                    return Err(invalid("product-session-timeout"));
                }
                thread::sleep(Duration::from_millis(20));
            };
            processes.push(json!({"pid":pid,"action":action,"case":index,"exitCode":code,"expectedExitCode":expected_code}));
            held.push(process);
            write_json(&root.join("processes.json"), &json!(processes))?;
            if code != expected_code {
                return Err(invalid("product-session-unexpected-exit"));
            }
            if code == 0 {
                serde_json::from_slice(&io("product-session-result-read", fs::read(out))?)
                    .map_err(|_| invalid("product-session-result-shape"))
            } else {
                Ok(json!(null))
            }
        };
        invoke("init", [0; 32], 0)?;
        let get_head = || Ok::<_, Failure>(Store::open(&store_root)?.state.head);
        invoke("migrate", get_head()?, 0)?;
        invoke(point, get_head()?, 86)?;
        let session_root = {
            let store = Store::open(&store_root)?;
            store.session_root(
                &store
                    .state
                    .session
                    .as_ref()
                    .ok_or(invalid("product-session-test-binding"))?
                    .binding,
            )?
        };
        let initial = json!({"controllerCrash":true,"localSessionRoot":session_root});
        let bound_head = get_head()?;
        let key = hash(&store_root.join("control/authority.key"))?;
        let first = invoke("recover", bound_head, 0)?;
        let after_first = get_head()?;
        invoke("recover", bound_head, 1)?;
        if get_head()? != after_first {
            return Err(invalid("product-session-stale-mutated"));
        }
        let second = invoke("recover", get_head()?, 0)?;
        if index == 0 {
            io(
                "product-session-test-remove-journal",
                fs::rename(
                    session_root.join("data/app/tasks/tasks.sqlite3"),
                    session_root.join("data/app/tasks/tasks.missing"),
                ),
            )?;
        } else {
            let mut file = io(
                "product-session-test-corrupt-app",
                OpenOptions::new()
                    .append(true)
                    .open(session_root.join("bin/kraqel-desktop.exe")),
            )?;
            io(
                "product-session-test-corrupt-write",
                file.write_all(b"synthetic tamper"),
            )?;
        }
        let head = get_head()?;
        invoke("recover", head, 1)?;
        if get_head()? != head {
            return Err(invalid("product-session-negative-mutated"));
        }
        let retired = invoke("retire", head, 0)?;
        let retired_head = get_head()?;
        invoke("recover", retired_head, 1)?;
        if get_head()? != retired_head || hash(&store_root.join("control/authority.key"))? != key {
            return Err(invalid("product-session-reset"));
        }
        cases.push(json!({"point":point,"localStoreRoot":store_root,"initial":initial,
            "recoveries":[first,second],"retirement":retired,"negativeStateUnchanged":true,"keyUnchanged":true}));
    }
    let result = json!({"nativePreviewOnly":true,"productionAcceptance":false,"passed":true,
        "controllerSha256":hex(&actor),"appSha256":hex(&expected),"cases":cases,
        "childProcesses":processes.len(),"controllerCrashes":2,"nativeGuiStarts":6,"recoveries":4,
        "negativeRecoveries":6,"actualContainerDenials":36,"networkMeasurement":false});
    write_json(&root.join("result.json"), &result)?;
    Ok(result)
}
