//! Control-bound synthetic starter. No adoption/recovery of existing application data.
use super::*;

impl Store {
    pub(super) fn start_guard(&self, actor: [u8; 32], head: [u8; 32]) -> Result<()> {
        self.actor(actor)?;
        if self.state.schema != 2 || self.state.pending.is_some() || self.state.head != head {
            return Err(invalid("product-start-state"));
        }
        self.idle()?;
        // Even an empty pre-existing object is a collision, never a job we may adopt.
        let raw = unsafe { OpenJobObjectW(JOB_OBJECT_QUERY, 0, self.job_name().as_ptr()) };
        if !raw.is_null() {
            drop(Handle(raw));
            return Err(invalid("product-start-job-collision"));
        }
        if unsafe { GetLastError() } != ERROR_FILE_NOT_FOUND {
            return Err(fail("product-start-job-open"));
        }
        Ok(())
    }

    pub(super) fn start_check(
        &self,
        actor: [u8; 32],
        head: [u8; 32],
        app: &Path,
        expected_app: [u8; 32],
    ) -> Result<Value> {
        self.start_guard(actor, head)?;
        let (_app_parents, _app_file, actual_app) = binary_pin(app)?;
        if actual_app != expected_app {
            return Err(invalid("product-start-app-hash"));
        }
        let mut kind = 0;
        check(
            unsafe { GetBinaryTypeW(path_wide(app)?.as_ptr(), &mut kind) },
            "product-start-app-pe",
        )?;
        if kind != 6 {
            return Err(invalid("product-start-app-x64"));
        }
        let root = std::env::temp_dir().join(format!(
            "kraqel-isolation-ui-restart-{}",
            Uuid::new_v4().simple()
        ));
        // The private Store (including its exclusive owner.lock) stays borrowed until the
        // entire GUI/job lifetime ends. Container grants apply only to a fresh sibling TEMP
        // session: never to Store, control/, bin/ or the reserved private data/ directory.
        let (session, mut profile, pin) = super::super::super::initialize(&root, app)?;
        if pin != expected_app {
            return Err(invalid("product-start-staged-app-hash"));
        }
        let (_staged_dirs, _staged_file, staged_hash) =
            binary_pin(&root.join("bin/kraqel-desktop.exe"))?;
        if staged_hash != expected_app {
            return Err(invalid("product-start-staged-app-changed"));
        }
        let ticket = RestartTicket {
            version: RestartTicket::VERSION,
            session_id: session,
            task_id: Uuid::new_v4(),
            launch_id: Uuid::new_v4(),
            recovery: false,
            automatic: true,
            source: kraqel_isolation_transport::RestartSource::Synthetic,
            point: CrashPoint::Completed,
            app_sha256: pin,
        };
        let name = self.job_name();
        generation_with_options(
            &root,
            &profile,
            &ticket,
            ticket.launch_id,
            GenerationOptions {
                expect: Expect::Crash,
                index: 0,
                durable_job: Some(&name),
                crash_controller: false,
                live_mode: None,
                product_root: Some(&self.root),
            },
        )?;
        if !profile.remove() {
            return Err(invalid("product-start-profile-cleanup"));
        }
        self.idle()?;
        private_acl(&self.root, true)?;
        private_acl(&self.root.join("data"), true)?;
        Ok(
            json!({"nativePreviewOnly":true,"productionAcceptance":false,
            "controlLeaseHeld":true,"namedProductJob":true,"actualContainerDenials":6,
            "appSha256":hex(&pin),"activeJobProcessesAtEnd":0,"profileRemoved":true,
            "localDiagnosticRoot":root,"existingDataAdopted":false,"networkMeasurement":false}),
        )
    }
}

// Invoked after suspended assignment/token checks and before ResumeThread. Sharing violation
// must be observed, not merely an arbitrary failure opening a missing/inaccessible lock.
pub(in super::super::super) fn verify_start_lease(root: &Path, name: &[u16]) -> Result<()> {
    match open_file(&root.join("control/owner.lock"), true, false) {
        Err(error) if error.code == i64::from(ERROR_SHARING_VIOLATION) => {}
        _ => return Err(invalid("product-start-lease-not-held")),
    }
    let job = Handle(unsafe { OpenJobObjectW(JOB_OBJECT_QUERY, 0, name.as_ptr()) });
    if job.0.is_null() {
        return Err(fail("product-start-assigned-job-open"));
    }
    let mut info: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
    check(
        unsafe {
            QueryInformationJobObject(
                job.0,
                JobObjectBasicAccountingInformation,
                (&mut info as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                null_mut(),
            )
        },
        "product-start-assigned-job-query",
    )?;
    if info.ActiveProcesses != 1 {
        return Err(invalid("product-start-assigned-job-count"));
    }
    Ok(())
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
        return Err(invalid("product-start-series-root"));
    }
    let expected_app = parse_hash(expected)?;
    let exe = io("product-start-series-exe", std::env::current_exe())?;
    let (_dirs, _file, actor) = binary_pin(&exe)?;
    let _parent = pin_directories(
        root.parent()
            .ok_or(invalid("product-start-series-parent"))?,
    )?;
    secure_directory(root)?;
    let store_root = root.join("store");
    let mut store = Store::initialize(&store_root, actor)?;
    if store.start_guard(actor, store.state.head).is_ok() {
        return Err(invalid("product-start-bootstrap-accepted"));
    }
    store.append(actor, store.state.head, Action::Migrate { from: 1, to: 2 })?;
    let head = store.state.head;
    if store.start_guard([0; 32], head).is_ok()
        || store.start_guard(actor, [0; 32]).is_ok()
        || store.start_check(actor, head, app, [0; 32]).is_ok()
    {
        return Err(invalid("product-start-negative-accepted"));
    }
    store.append(actor, head, Action::Prepare { binary: [8; 32] })?;
    if store.start_guard(actor, store.state.head).is_ok() {
        return Err(invalid("product-start-pending-accepted"));
    }
    store.append(actor, store.state.head, Action::Abort)?;
    let head = store.state.head;
    let key = hash(&store_root.join("control/authority.key"))?;
    let mut runs = Vec::new();
    runs.push(store.start_check(actor, head, app, expected_app)?);
    drop(store);
    let store = Store::open(&store_root)?;
    runs.push(store.start_check(actor, head, app, expected_app)?);
    if store.state.head != head
        || hash(&store_root.join("control/authority.key"))? != key
        || io(
            "product-start-private-data-list",
            fs::read_dir(store_root.join("data")),
        )?
        .next()
        .is_some()
    {
        return Err(invalid("product-start-control-mutated"));
    }
    let result = json!({"nativePreviewOnly":true,"productionAcceptance":false,"passed":true,
        "controllerSha256":hex(&actor),"appSha256":hex(&expected_app),"negativeGuards":5,
        "starts":runs,"actualContainerDenials":12,"controlHeadUnchanged":true,
        "controlKeyUnchanged":true,"reservedPrivateDataEmpty":true,"networkMeasurement":false});
    write_json(&root.join("result.json"), &result)?;
    Ok(result)
}
