//! Explicit, path-scoped synthetic crash series. No environment switches in product commands.
use super::*;
use std::cell::Cell;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Point {
    BeforeKey,
    KeySynced,
    BeforeRecord,
    RecordCreated,
    RecordSynced,
    HeadStaged,
    HeadPublished,
    PackagePartial,
    PackageSynced,
}
thread_local! {static ARMED:Cell<Option<Point>>=const {Cell::new(None)};}
pub(super) fn hit(point: Point) {
    if ARMED.with(|state| state.get() == Some(point)) {
        // Abrupt process death: no Rust destructors and no deliberate file/job cleanup first.
        if unsafe { TerminateProcess(GetCurrentProcess(), 86) } == 0 {
            std::process::abort();
        }
        std::process::abort();
    }
}
fn point_name(point: Option<Point>) -> String {
    point.map_or_else(
        || "none".to_owned(),
        |p| {
            serde_json::to_value(p)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        },
    )
}
pub(super) fn scoped(root: &Path) -> Result<PathBuf> {
    if root.file_name() != Some(std::ffi::OsStr::new("store")) {
        return Err(invalid("product-crash-store-name"));
    }
    let parent = root.parent().ok_or(invalid("product-crash-parent"))?;
    let name = parent
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(invalid("product-crash-case-name"))?;
    if name.len() != 7
        || !name.starts_with("case-")
        || !name[5..].bytes().all(|b| b.is_ascii_digit())
    {
        return Err(invalid("product-crash-case-name"));
    }
    let series = parent
        .parent()
        .ok_or(invalid("product-crash-series-parent"))?;
    if !crate::policy::valid_root(series, true)
        || !series
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("kraqel-isolation-controller-"))
    {
        return Err(invalid("product-crash-root-scope"));
    }
    let _pins = pin_directories(parent)?;
    Ok(series.to_path_buf())
}
pub(in crate::windows) fn child(args: &[OsString]) -> Result<Value> {
    if args.len() != 5 {
        return Err(invalid("product-crash-child-arguments"));
    }
    let root = Path::new(&args[0]);
    let series = scoped(root)?;
    let action = args[1].to_str().ok_or(invalid("product-crash-action"))?;
    let point = if args[2] == "none" {
        None
    } else {
        Some(
            serde_json::from_value::<Point>(json!(args[2].to_string_lossy()))
                .map_err(|_| invalid("product-crash-point"))?,
        )
    };
    let expected = parse_hash(&args[3])?;
    let exe = io("product-crash-current-exe", std::env::current_exe())?;
    let (_dirs, _exe, actor) = binary_pin(&exe)?;
    if !matches!(
        action,
        "init" | "migrate" | "prepare" | "activate" | "abort" | "status" | "check"
    ) || (point.is_some() && matches!(action, "status" | "check"))
    {
        return Err(invalid("product-crash-action"));
    }
    ARMED.with(|state| state.set(point));
    if action == "init" {
        return Ok(Store::initialize(root, actor)?.summary());
    }
    let mut store = Store::open(root)?;
    store.actor(actor)?;
    match action {
        "status" => {}
        "migrate" => store.append(actor, expected, Action::Migrate { from: 1, to: 2 })?,
        "prepare" => {
            let candidate = series.join("candidate.exe");
            let sha = hash(&candidate)?;
            store.prepare(actor, expected, &candidate, sha)?;
        }
        "activate" => store.activate(actor, expected)?,
        "abort" => store.append(actor, expected, Action::Abort)?,
        "check" => return store.check_container(Path::new(&args[4])),
        _ => return Err(invalid("product-crash-action")),
    }
    Ok(store.summary())
}

struct Run {
    root: PathBuf,
    app: PathBuf,
    base: PathBuf,
    candidate: PathBuf,
    base_sha: [u8; 32],
    candidate_sha: [u8; 32],
    processes: Vec<Value>,
    held: Vec<std::process::Child>,
}
impl Run {
    fn invoke(
        &mut self,
        root: &Path,
        action: &str,
        point: Option<Point>,
        new_actor: bool,
        head: [u8; 32],
        expected: i32,
    ) -> Result<Value> {
        let exe = if new_actor {
            &self.candidate
        } else {
            &self.base
        };
        let role = format!("{:04}", self.processes.len());
        let out = self.root.join(format!("{role}.stdout"));
        let err = self.root.join(format!("{role}.stderr"));
        let mut child = io(
            "product-series-child",
            Command::new(exe)
                .arg("product-control-child")
                .arg(root)
                .arg(action)
                .arg(point_name(point))
                .arg(hex(&head))
                .arg(&self.app)
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(io("product-series-stdout", fs::File::create(&out))?)
                .stderr(io("product-series-stderr", fs::File::create(&err))?)
                .spawn(),
        )?;
        let pid = child.id();
        let deadline = Instant::now() + Duration::from_secs(75);
        let code = loop {
            if let Some(status) = io("product-series-wait", child.try_wait())? {
                break status.code().unwrap_or(-1);
            }
            if Instant::now() > deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err(invalid("product-series-timeout"));
            }
            thread::sleep(Duration::from_millis(20));
        };
        self.processes.push(json!({"pid":pid,"action":action,"point":point_name(point),"newActor":new_actor,"exitCode":code,"expectedExitCode":expected,"case":root.parent().unwrap().file_name().unwrap().to_string_lossy()}));
        self.held.push(child);
        write_json(&self.root.join("processes.json"), &json!(self.processes))?;
        if code != expected {
            return Err(invalid("product-series-unexpected-exit"));
        }
        if code == 0 {
            serde_json::from_slice(&io("product-series-read-result", fs::read(out))?)
                .map_err(|_| invalid("product-series-result"))
        } else {
            Ok(json!(null))
        }
    }
    fn head(root: &Path) -> Result<[u8; 32]> {
        Ok(Store::open(root)?.state.head)
    }
    fn normal(
        &mut self,
        root: &Path,
        action: &str,
        new_actor: bool,
        expected: i32,
    ) -> Result<Value> {
        let head = if action == "init" {
            [0; 32]
        } else {
            Self::head(root)?
        };
        self.invoke(root, action, None, new_actor, head, expected)
    }
}
fn snapshot(root: &Path) -> Result<[u8; 32]> {
    fn walk(path: &Path, hashing: &mut Sha256) -> Result<()> {
        let mut names = io("product-series-snapshot-list", fs::read_dir(path))?
            .map(|e| e.map(|x| x.path()))
            .collect::<std::io::Result<Vec<_>>>()
            .map_err(|_| invalid("product-series-snapshot-entry"))?;
        names.sort();
        for name in names {
            hashing.update(name.file_name().unwrap().as_encoded_bytes());
            if name.is_dir() {
                walk(&name, hashing)?;
            } else {
                hashing.update(hash(&name)?);
            }
        }
        Ok(())
    }
    let mut hashing = Sha256::new();
    walk(root, &mut hashing)?;
    Ok(hashing.finalize().into())
}
pub(in crate::windows) fn series(root: &Path, app: &Path) -> Result<Value> {
    if !crate::policy::valid_root(root, false)
        || !root
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("kraqel-isolation-controller-"))
    {
        return Err(invalid("product-series-fresh-root"));
    }
    let _parent = pin_directories(root.parent().ok_or(invalid("product-series-parent"))?)?;
    secure_directory(root)?;
    let base = io("product-series-exe", std::env::current_exe())?;
    let base_sha = hash(&base)?;
    let candidate = root.join("candidate.exe");
    io("product-series-candidate-copy", fs::copy(&base, &candidate))?;
    let mut file = io(
        "product-series-candidate-overlay",
        OpenOptions::new().append(true).open(&candidate),
    )?;
    io(
        "product-series-overlay-write",
        file.write_all(b"KRAQEL isolated control crash candidate"),
    )?;
    io("product-series-overlay-sync", file.sync_all())?;
    drop(file);
    let candidate_sha = hash(&candidate)?;
    let mut run = Run {
        root: root.to_path_buf(),
        app: app.to_path_buf(),
        base,
        candidate,
        base_sha,
        candidate_sha,
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
    for action in ["init", "migrate", "prepare", "activate", "abort"] {
        let points = match action {
            "init" => vec![
                Point::BeforeKey,
                Point::KeySynced,
                Point::RecordCreated,
                Point::RecordSynced,
                Point::HeadStaged,
                Point::HeadPublished,
            ],
            "prepare" => common
                .into_iter()
                .chain([Point::PackagePartial, Point::PackageSynced])
                .collect(),
            _ => common.to_vec(),
        };
        for point in points {
            let case = root.join(format!("case-{:02}", cases.len()));
            secure_directory(&case)?;
            let store = case.join("store");
            if action != "init" {
                run.normal(&store, "init", false, 0)?;
            }
            if matches!(action, "prepare" | "activate" | "abort") {
                run.normal(&store, "migrate", false, 0)?;
            }
            if matches!(action, "activate" | "abort") {
                run.normal(&store, "prepare", false, 0)?;
            }
            if action != "init" {
                io(
                    "product-series-data-sentinel",
                    fs::write(
                        store.join("data/consumed-sentinel"),
                        b"synthetic consumed state; never reset",
                    ),
                )?;
            }
            let key_before = if action == "init" {
                None
            } else {
                Some(hash(&store.join("control/authority.key"))?)
            };
            let old_head = if action == "init" {
                [0; 32]
            } else {
                Run::head(&store)?
            };
            run.invoke(&store, action, Some(point), false, old_head, 86)?;
            let before = snapshot(&store)?;
            let ready = matches!(
                point,
                Point::BeforeRecord
                    | Point::HeadPublished
                    | Point::PackagePartial
                    | Point::PackageSynced
            );
            let new_active = action == "activate" && point == Point::HeadPublished;
            run.invoke(
                &store,
                "status",
                None,
                false,
                [0; 32],
                if ready && !new_active { 0 } else { 1 },
            )?;
            run.invoke(
                &store,
                "status",
                None,
                true,
                [0; 32],
                if ready && new_active { 0 } else { 1 },
            )?;
            if snapshot(&store)? != before {
                return Err(invalid("product-series-restart-mutated-state"));
            }
            if let Some(previous) = key_before
                && hash(&store.join("control/authority.key"))? != previous
            {
                return Err(invalid("product-series-key-changed"));
            }
            let initial = if ready {
                Some(Store::open(&store)?.summary())
            } else {
                blocked += 1;
                None
            };
            let mut reconciled = false;
            if ready {
                let state = Store::open(&store)?;
                let pending = state.state.pending;
                let schema = state.state.schema;
                drop(state);
                if action == "prepare" && point == Point::BeforeRecord {
                    run.normal(&store, "prepare", false, 0)?;
                    run.normal(&store, "activate", false, 0)?;
                    reconciled = true;
                } else if action == "prepare"
                    && matches!(point, Point::HeadPublished | Point::PackagePartial)
                {
                    run.normal(&store, "activate", false, 1)?;
                    run.normal(&store, "abort", false, 0)?;
                    reconciled = true;
                } else if pending.is_some() {
                    run.normal(
                        &store,
                        if action == "abort" {
                            "abort"
                        } else {
                            "activate"
                        },
                        false,
                        0,
                    )?;
                    reconciled = true;
                } else if schema == 1 {
                    run.normal(&store, "migrate", false, 0)?;
                    reconciled = true;
                }
                if action != "init" {
                    run.invoke(&store, action, None, false, old_head, 1)?;
                }
            }
            if action != "init"
                && io(
                    "product-series-sentinel-read",
                    fs::read(store.join("data/consumed-sentinel")),
                )? != b"synthetic consumed state; never reset"
            {
                return Err(invalid("product-series-data-changed"));
            }
            cases.push(json!({"action":action,"point":point,"readyAfterCrash":ready,"newActorAfterCrash":new_active,"initial":initial,"explicitlyReconciled":reconciled,"restartReadOnly":true,"keyPreserved":true,"dataPreserved":true,"localRoot":store}));
            write_json(&root.join("progress.json"), &json!({"cases":cases}))?;
        }
    }
    // Happy path includes real suspended-token denial checks before/after a local program change.
    let happy = root.join("case-28");
    secure_directory(&happy)?;
    let store = happy.join("store");
    run.normal(&store, "init", false, 0)?;
    run.normal(&store, "migrate", false, 0)?;
    let first = run.normal(&store, "check", false, 0)?;
    run.normal(&store, "prepare", false, 0)?;
    run.normal(&store, "activate", false, 0)?;
    let second = run.normal(&store, "check", true, 0)?;
    run.normal(&store, "status", false, 1)?;
    run.normal(&store, "status", true, 0)?;
    if cases.len() != 28
        || blocked != 17
        || first["profileRemoved"] != true
        || second["profileRemoved"] != true
    {
        return Err(invalid("product-series-incomplete"));
    }
    let report = json!({"nativePreviewOnly":true,"productionAcceptance":false,"passed":true,"crashCases":28,"controllerCrashes":28,"blockedCases":17,"readyCases":11,"childProcesses":run.processes.len(),"baseBinarySha256":hex(&run.base_sha),"candidateBinarySha256":hex(&run.candidate_sha),"cases":cases,"happyRoot":store,"containerChecks":[first,second],"actualContainerDenials":12,"networkMeasurement":false});
    write_json(&root.join("result.json"), &report)?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fault_target_is_default_off_and_child_scope_rejects_product_paths() {
        assert!(ARMED.with(|state| state.get().is_none()));
        assert!(scoped(Path::new(r"C:\Users\person\.kraqel-broker-preview")).is_err());
        assert!(scoped(Path::new(r"\\host\share\case-00\store")).is_err());
        hit(Point::BeforeRecord);
        assert!(ARMED.with(|state| state.get().is_none()));
    }
}
