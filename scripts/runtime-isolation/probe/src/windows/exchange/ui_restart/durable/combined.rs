//! Append-only host authority for a genuinely new live task after authenticated recovery.
use super::super::live::LiveMode;
use super::*;

const SETTLED: &[u8] = b"KRAQEL-SETTLED\0";
fn reject_next(root: &Path) -> Result<()> {
    let journal = hash(&root.join("data/app/tasks/tasks.sqlite3"))?;
    let launches = || -> Result<usize> {
        let mut count = 0;
        for entry in io("combined-launch-directory", fs::read_dir(root))? {
            if io("combined-launch-entry", entry)?
                .file_name()
                .to_string_lossy()
                .starts_with("launch-")
            {
                count += 1;
            }
        }
        Ok(count)
    };
    let before = launches()?;
    child(&["ui-broker-live-next".as_ref(), root.as_os_str()], 1)?;
    if launches()? != before || hash(&root.join("data/app/tasks/tasks.sqlite3"))? != journal {
        return Err(invalid("combined-rejected-next-mutated-state"));
    }
    Ok(())
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Settlement {
    version: u16,
    authority_sha256: [u8; 32],
    projects: u32,
}

fn settlement_path(root: &Path, sequence: u32) -> PathBuf {
    root.join(format!("control/settled-{sequence:04}.mac"))
}
fn settlement(root: &Path, authority: &Authority) -> Result<Settlement> {
    let key = read_key(root)?;
    let decoded = read_authenticated(&settlement_path(root, authority.sequence), &key)?;
    let value: Settlement = serde_json::from_slice(
        decoded
            .strip_prefix(SETTLED)
            .ok_or(invalid("combined-settlement-domain"))?,
    )
    .map_err(|_| invalid("combined-settlement-shape"))?;
    if value.version != 1
        || value.authority_sha256 != hash(&authority_path(root, authority.sequence))?
    {
        return Err(invalid("combined-settlement-binding"));
    }
    Ok(value)
}
pub(super) fn previous_projects(root: &Path, authority: &Authority) -> Result<u32> {
    Ok(settlement(root, authority)?.projects)
}
pub(super) fn settle(root: &Path, authority: &Authority, projects: u32) -> Result<()> {
    let path = settlement_path(root, authority.sequence);
    if io("combined-settlement-exists", path.try_exists())? {
        if settlement(root, authority)?.projects != projects {
            return Err(invalid("combined-settlement-changed"));
        }
        return Ok(());
    }
    let value = Settlement {
        version: 1,
        authority_sha256: hash(&authority_path(root, authority.sequence))?,
        projects,
    };
    let mut bytes = SETTLED.to_vec();
    bytes.extend(serde_json::to_vec(&value).map_err(|_| invalid("combined-settlement-encode"))?);
    let wire = sign(&bytes, &read_key(root)?)?;
    let mut file = io(
        "combined-settlement-create",
        OpenOptions::new().write(true).create_new(true).open(path),
    )?;
    io("combined-settlement-write", file.write_all(&wire))?;
    io("combined-settlement-sync", file.sync_all())
}
pub(in super::super::super::super) fn seed_live(
    root: &Path,
    app: &Path,
    point: &str,
    mode: &str,
) -> Result<Value> {
    let mode: LiveMode =
        serde_json::from_value(json!(mode)).map_err(|_| invalid("combined-live-mode"))?;
    let point: CrashPoint =
        serde_json::from_value(json!(point)).map_err(|_| invalid("combined-crash-point"))?;
    if mode == LiveMode::Partial && point != CrashPoint::Reading {
        return Err(invalid("combined-partial-point"));
    }
    // Completed is deliberately unreachable before the manual confirmation. No crash pause
    // may block cancellation in the window-close paths.
    if mode.window_close() && point != CrashPoint::Completed {
        return Err(invalid("combined-close-point"));
    }
    seed_with(root, app, point, Some(mode))
}
pub(in super::super::super::super) fn next_live(root: &Path) -> Result<Value> {
    let root = bound_root(root)?;
    let _ownership = own(&root)?;
    let previous = authenticate(&root)?;
    if previous.live_mode.is_none() || previous.sequence >= 63 {
        return Err(invalid("combined-next-not-live-or-budget"));
    }
    let settled = settlement(&root, &previous)?;
    drain(previous.ticket.session_id)?;
    let profile = reopen_profile(previous.ticket.session_id)?;
    let mut authority = previous.clone();
    authority.sequence += 1;
    authority.previous_sha256 = hash(&authority_path(&root, previous.sequence))?;
    authority.previous_projects = settled.projects;
    authority.live_mode = Some(LiveMode::Fixture);
    authority.ticket.task_id = Uuid::new_v4();
    authority.ticket.launch_id = Uuid::new_v4();
    authority.ticket.automatic = true;
    authority.ticket.source = kraqel_isolation_transport::RestartSource::Live;
    authority.ticket.point = CrashPoint::Completed;
    seal(&root, &authority)?;
    let name = job_name(authority.ticket.session_id);
    generation_with_options(
        &root,
        &profile,
        &authority.ticket,
        authority.ticket.launch_id,
        GenerationOptions {
            expect: Expect::Crash,
            index: authority.sequence as usize * 8,
            durable_job: Some(&name),
            crash_controller: true,
            live_mode: authority.live_mode,
        },
    )?;
    Err(invalid("combined-next-did-not-crash"))
}

pub(in super::super::super::super) fn series(root: &Path, app: &Path) -> Result<Value> {
    if !crate::policy::valid_root(root, false)
        || !root
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with("kraqel-isolation-controller-"))
    {
        return Err(invalid("combined-series-root"));
    }
    io("combined-series-create", fs::create_dir(root))?;
    let cases = CrashPoint::ALL
        .into_iter()
        .map(|point| (point, "fixture"))
        .chain([
            (CrashPoint::Completed, "cancel"),
            (CrashPoint::Completed, "invalid"),
            (CrashPoint::Reading, "partial"),
        ]);
    let mut results = Vec::new();
    for (point, mode) in cases {
        let case_root = root.parent().unwrap().join(format!(
            "kraqel-isolation-ui-restart-{}",
            Uuid::new_v4().simple()
        ));
        let point_name = serde_json::to_value(point).unwrap();
        write_json(
            &root.join("current-case.json"),
            &json!({"point":point,"mode":mode,"localRoot":case_root}),
        )?;
        child(
            &[
                "ui-broker-live-seed".as_ref(),
                case_root.as_os_str(),
                app.as_os_str(),
                point_name.as_str().unwrap().as_ref(),
                mode.as_ref(),
            ],
            86,
        )?;
        // Cannot mint the next grant from diagnostics, a dead PID or an unreconciled journal.
        reject_next(&case_root)?;
        if authority_path(&case_root, 1).exists() {
            return Err(invalid("combined-premature-next-created-authority"));
        }
        child(
            &[
                "ui-broker-recover".as_ref(),
                case_root.as_os_str(),
                "first".as_ref(),
            ],
            0,
        )?;
        let marker = settlement_path(&case_root, 0);
        let original = io("combined-marker-read", fs::read(&marker))?;
        let mut corrupt = original.clone();
        corrupt[MAGIC.len()] ^= 1;
        io("combined-marker-corrupt", fs::write(&marker, corrupt))?;
        let rejected = reject_next(&case_root);
        io("combined-marker-restore", fs::write(&marker, original))?;
        rejected?;
        // Append a broken chain entry; an invalid chain must not be repaired or ignored.
        let gap = authority_path(&case_root, 2);
        io(
            "combined-gap-fixture",
            fs::copy(authority_path(&case_root, 0), &gap),
        )?;
        reject_next(&case_root)?;
        io("combined-gap-restore", fs::remove_file(gap))?;
        let authority = authenticate(&bound_root(&case_root)?)?;
        let mut forged = authority.clone();
        forged.sequence = 1;
        forged.ticket.task_id = Uuid::new_v4();
        forged.previous_sha256 = [0; 32];
        seal(&case_root, &forged)?;
        let rejected = reject_next(&case_root);
        io(
            "combined-parent-restore",
            fs::remove_file(authority_path(&case_root, 1)),
        )?;
        rejected?;
        child(&["ui-broker-live-next".as_ref(), case_root.as_os_str()], 86)?;
        reject_next(&case_root)?;
        if authority_path(&case_root, 2).exists() {
            return Err(invalid("combined-unsettled-next-created-authority"));
        }
        child(
            &[
                "ui-broker-recover".as_ref(),
                case_root.as_os_str(),
                "first".as_ref(),
            ],
            0,
        )?;
        child(
            &[
                "ui-broker-recover".as_ref(),
                case_root.as_os_str(),
                "second".as_ref(),
            ],
            0,
        )?;
        child(
            &[
                "ui-broker-recover".as_ref(),
                case_root.as_os_str(),
                "retire".as_ref(),
            ],
            0,
        )?;
        reject_next(&case_root)?;
        results.push(json!({"point":point,"mode":mode,"passed":true,"controllerCrashes":2,"recoveries":3,"newLiveTasks":2,"appGenerations":8,"rejectedNextGrants":6,"profileRemoved":true,"localRoot":case_root}));
        write_json(&root.join("progress.json"), &json!({"cases":results}))?;
    }
    let report = json!({"researchOnly":true,"productionAcceptance":false,"passed":true,"controllerCrashes":18,"recoveries":27,"appGenerations":72,"newLiveTasks":18,"rejectedNextGrants":54,"cases":results,"networkMeasurement":false});
    write_json(&root.join("result.json"), &report)?;
    Ok(report)
}
