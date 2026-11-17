//! Additional transition crash series, restricted to fresh synthetic fixtures.
use super::*;
use crate::windows::exchange::ui_restart::durable::product::crash::{self, Point};

fn allowed(action: &str, p: Point) -> bool {
    let common = matches!(
        p,
        Point::BeforeRecord
            | Point::RecordCreated
            | Point::RecordSynced
            | Point::HeadStaged
            | Point::HeadPublished
    );
    match action {
        "bind" => {
            common
                || matches!(
                    p,
                    Point::SessionStaged | Point::BeforeSessionRun | Point::SessionPrepared
                )
        }
        "recover" => common || p == Point::BeforeSessionRun,
        "retire" => common || matches!(p, Point::RetireCommitted | Point::ProfileRemoved),
        _ => false,
    }
}
pub(in crate::windows) fn child(args: &[OsString]) -> Result<Value> {
    if args.len() != 6 {
        return Err(invalid("transition-child-args"));
    }
    let root = Path::new(&args[0]);
    crash::scoped(root)?;
    let action = args[1].to_str().ok_or(invalid("transition-action"))?;
    let point: Point = serde_json::from_value(json!(args[2].to_string_lossy()))
        .map_err(|_| invalid("transition-point"))?;
    if !allowed(action, point) {
        return Err(invalid("transition-action-point"));
    }
    let app = Path::new(&args[3]);
    let expected = parse_hash(&args[4])?;
    let head = parse_hash(&args[5])?;
    let exe = io("transition-current-exe", std::env::current_exe())?;
    let (_dirs, _file, actor) = binary_pin(&exe)?;
    let mut store = Store::open(root)?;
    store.actor(actor)?;
    crash::arm(root, point)?;
    match action {
        "bind" => store.bind_session(actor, head, app, expected, CrashPoint::Completed),
        "recover" => store.recover_session(actor, head),
        "retire" => store.retire_session(actor, head),
        _ => Err(invalid("transition-action")),
    }
}
struct Run {
    root: PathBuf,
    app: PathBuf,
    exe: PathBuf,
    app_sha: [u8; 32],
    processes: Vec<Value>,
    held: Vec<std::process::Child>,
}
impl Run {
    fn invoke(
        &mut self,
        store: &Path,
        action: &str,
        point: Option<Point>,
        head: [u8; 32],
        expected: i32,
    ) -> Result<Value> {
        let index = self.processes.len();
        let out = self.root.join(format!("{index:03}.stdout"));
        let mut command = Command::new(&self.exe);
        if let Some(p) = point {
            command
                .arg("product-transition-child")
                .arg(store)
                .arg(action)
                .arg(serde_json::to_value(p).unwrap().as_str().unwrap());
        } else {
            command.arg("product-session-child").arg(store).arg(action);
        }
        let mut child = io(
            "transition-spawn",
            command
                .arg(&self.app)
                .arg(hex(&self.app_sha))
                .arg(hex(&head))
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(io("transition-out", fs::File::create(&out))?)
                .stderr(io(
                    "transition-err",
                    fs::File::create(self.root.join(format!("{index:03}.stderr"))),
                )?)
                .spawn(),
        )?;
        let pid = child.id();
        let deadline = Instant::now() + Duration::from_secs(75);
        let code = loop {
            if let Some(status) = io("transition-wait", child.try_wait())? {
                break status.code().unwrap_or(-1);
            }
            if Instant::now() > deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err(invalid("transition-timeout"));
            }
            thread::sleep(Duration::from_millis(20));
        };
        self.processes.push(
            json!({"pid":pid,"case":store.parent().unwrap().file_name().unwrap().to_string_lossy(),
            "action":action,"point":point,"exitCode":code,"expectedExitCode":expected}),
        );
        self.held.push(child);
        write_json(&self.root.join("processes.json"), &json!(self.processes))?;
        if code != expected {
            return Err(invalid("transition-unexpected-exit"));
        }
        if code == 0 {
            serde_json::from_slice(&io("transition-read", fs::read(out))?)
                .map_err(|_| invalid("transition-result"))
        } else {
            Ok(json!(null))
        }
    }
    fn head(store: &Path) -> Result<[u8; 32]> {
        Ok(Store::open(store)?.state.head)
    }
}
fn wait_gone(name: &[u16]) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let raw = unsafe { OpenJobObjectW(JOB_OBJECT_QUERY, 0, name.as_ptr()) };
        if raw.is_null() {
            if unsafe { GetLastError() } != ERROR_FILE_NOT_FOUND {
                return Err(fail("transition-old-job"));
            }
            return Ok(());
        }
        drop(Handle(raw));
        if Instant::now() > deadline {
            return Err(invalid("transition-old-job-timeout"));
        }
        thread::sleep(Duration::from_millis(20));
    }
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
        return Err(invalid("transition-series-root"));
    }
    let _parent = pin_directories(root.parent().ok_or(invalid("transition-parent"))?)?;
    secure_directory(root)?;
    let exe = io("transition-exe", std::env::current_exe())?;
    let actor = hash(&exe)?;
    let app_sha = parse_hash(expected)?;
    let mut run = Run {
        root: root.to_path_buf(),
        app: app.to_path_buf(),
        exe,
        app_sha,
        processes: Vec::new(),
        held: Vec::new(),
    };
    let common = [
        Point::BeforeRecord,
        Point::RecordCreated,
        Point::RecordSynced,
        Point::HeadStaged,
        Point::HeadPublished,
    ];
    let mut cases = Vec::new();
    let mut blocked = 0;
    let mut recoveries = 0;
    for action in ["bind", "recover", "retire"] {
        let extra = match action {
            "bind" => vec![Point::SessionStaged, Point::BeforeSessionRun],
            "recover" => vec![Point::BeforeSessionRun],
            _ => vec![Point::RetireCommitted, Point::ProfileRemoved],
        };
        for point in common.into_iter().chain(extra) {
            let case = root.join(format!("case-{:02}", cases.len()));
            secure_directory(&case)?;
            let store = case.join("store");
            run.invoke(&store, "init", None, [0; 32], 0)?;
            run.invoke(&store, "migrate", None, Run::head(&store)?, 0)?;
            if action != "bind" {
                run.invoke(&store, "completed", None, Run::head(&store)?, 86)?;
            }
            let (parent, name) = {
                let s = Store::open(&store)?;
                (s.session_parent()?, s.job_name())
            };
            wait_gone(&name)?;
            let key = hash(&store.join("control/authority.key"))?;
            run.invoke(&store, action, Some(point), Run::head(&store)?, 86)?;
            let before = crash::snapshot(&store)?;
            let ready = !matches!(
                point,
                Point::RecordCreated | Point::RecordSynced | Point::HeadStaged
            );
            // Fresh actor-authenticated status process; no repair or key regeneration.
            run.invoke(&store, "status", None, [0; 32], if ready { 0 } else { 1 })?;
            if crash::snapshot(&store)? != before {
                return Err(invalid("transition-status-mutated"));
            }
            let recoverable = ready
                && ((action == "recover") || (action == "retire" && point == Point::BeforeRecord));
            let head = if ready { Run::head(&store)? } else { [0; 32] };
            let recovery = run.invoke(
                &store,
                "recover",
                None,
                head,
                if recoverable { 0 } else { 1 },
            )?;
            if recoverable {
                recoveries += 1;
                run.invoke(&store, "recover", None, head, 1)?;
            } else if crash::snapshot(&store)? != before {
                return Err(invalid("transition-negative-mutated"));
            }
            if hash(&store.join("control/authority.key"))? != key {
                return Err(invalid("transition-key-reset"));
            }
            if !ready {
                blocked += 1;
            }
            let dirs = io("transition-session-list", fs::read_dir(&parent))?
                .map(|e| e.map(|x| x.path()))
                .collect::<std::io::Result<Vec<_>>>()
                .map_err(|_| invalid("transition-session-entry"))?;
            if dirs.len() != 1 {
                return Err(invalid("transition-session-count"));
            }
            let session_root = &dirs[0];
            let id = session_root
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_prefix("kraqel-isolation-ui-restart-"))
                .and_then(|n| Uuid::parse_str(n).ok())
                .filter(|id| !id.is_nil())
                .ok_or(invalid("transition-profile-name"))?;
            wait_gone(&name)?;
            let before_cleanup = crash::snapshot(&store)?;
            // Cleanup of THIS newly created synthetic profile only. Never repairs a blocked store
            // or treats an orphan directory/diagnostic as import/recovery authority.
            if point != Point::ProfileRemoved {
                let mut profile = super::super::super::reopen_profile(id)?;
                if !profile.remove() {
                    return Err(invalid("transition-fixture-profile-cleanup"));
                }
            }
            if crash::snapshot(&store)? != before_cleanup {
                return Err(invalid("transition-cleanup-mutated-control"));
            }
            cases.push(json!({"action":action,"point":point,"controlReady":ready,
                "explicitRecovery":recoverable,"recovery":recovery,"localStoreRoot":store,
                "localSessionRoot":session_root,"statusReadOnly":true,"keyUnchanged":true,
                "testProfileRemoved":true,"fixtureCleanupOnly":true}));
            write_json(&root.join("progress.json"), &json!(cases))?;
        }
    }
    if cases.len() != 20 || blocked != 9 || recoveries != 4 {
        return Err(invalid("transition-incomplete"));
    }
    let result = json!({"nativePreviewOnly":true,"productionAcceptance":false,"passed":true,
        "controllerSha256":hex(&actor),"appSha256":hex(&app_sha),"transitionCrashes":20,
        "seedControllerCrashes":13,"blockedCases":9,"controlReadyCases":11,"explicitRecoveries":4,
        "nativeGuiStarts":17,"actualContainerDenials":102,"childProcesses":run.processes.len(),
        "profilesRemoved":20,"cases":cases,"networkMeasurement":false});
    write_json(&root.join("result.json"), &result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fault_points_are_typed_and_scoped_to_their_transition() {
        assert!(allowed("bind", Point::SessionStaged));
        assert!(!allowed("recover", Point::SessionStaged));
        assert!(!allowed("bind", Point::ProfileRemoved));
        assert!(!allowed("status", Point::BeforeRecord));
        assert!(!allowed("retire", Point::BeforeKey));
        assert!(
            crash::arm(
                Path::new(r"C:\Users\person\.kraqel-broker-preview"),
                Point::BeforeRecord
            )
            .is_err()
        );
    }
}
