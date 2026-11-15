//! Headless native application-service test. Fixed synthetic fixture only, no user path argument.
use super::*;
use kraqel_domain::{CURRENT_SCHEMA_VERSION, Project, ProjectId};
use kraqel_isolation_test_adapter::{
    FileOperation, FileTask, ImportReceipt, OneShotAdapter, SYNTHETIC_ARCHIVE, TaskControl,
    TaskError,
};
use kraqel_storage::project_manager::ProjectManager;
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|v| u64::try_from(v.as_millis()).ok())
        .unwrap_or(0)
}
fn bounded(path: &Path, maximum: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    io(
        "read-bounded-task-report",
        io("open-task-report", fs::File::open(path))?
            .take(maximum as u64 + 1)
            .read_to_end(&mut bytes),
    )?;
    if bytes.len() > maximum {
        return Err(invalid("task-report-budget"));
    }
    Ok(bytes)
}
fn parsed_handle(value: &OsString) -> Result<usize> {
    value
        .to_str()
        .and_then(|v| v.parse().ok())
        .filter(|n| *n != 0 && *n != usize::MAX)
        .ok_or(invalid("import-handle"))
}

pub(in super::super) fn child(args: &[OsString]) -> Result<Value> {
    let data = Path::new(&args[0]);
    let token = token_facts(unsafe { GetCurrentProcess() }, None)?;
    if token["isAppContainer"] != true || token["capabilityCount"] != 0 {
        return Err(invalid("import-child-token"));
    }
    let input_handle = parsed_handle(&args[1])?;
    let output_handle = parsed_handle(&args[2])?;
    if input_handle == output_handle {
        return Err(invalid("import-handle-alias"));
    }
    let expected_id =
        Uuid::parse_str(&args[3].to_string_lossy()).map_err(|_| invalid("import-expected-task"))?;
    let mut input = unsafe { fs::File::from_raw_handle(input_handle as HANDLE) };
    let mut output = unsafe { fs::File::from_raw_handle(output_handle as HANDLE) };
    let info = file_info(&input)?;
    file_info(&output)?;
    let task: FileTask = serde_json::from_slice(&bounded(&data.join("task.json"), 2048)?)
        .map_err(|_| invalid("typed-task-invalid"))?;
    if task.byte_count != (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow) {
        return Err(invalid("task-handle-size-mismatch"));
    }
    let host = host_for(data.parent().ok_or(invalid("import-root"))?)?;
    let denied = fs::File::open(host.join("withheld.txt"))
        .err()
        .and_then(|e| e.raw_os_error())
        == Some(5);
    if !denied {
        return Err(invalid("import-unselected-file-readable"));
    }

    // Separate synthetic baseline, prepared inside containment before dispatching the task.
    io(
        "create-adapter-project-root",
        fs::create_dir_all(data.join("app/projects")),
    )?;
    let manager = io(
        "adapter-project-manager",
        ProjectManager::new(data.join("app/projects")).map_err(std::io::Error::other),
    )?;
    let baseline_id = ProjectId::new();
    let baseline = manager
        .create_project(
            baseline_id.to_string(),
            &Project {
                id: baseline_id,
                name: "Unveränderte Testbasis".into(),
                schema_version: CURRENT_SCHEMA_VERSION,
            },
        )
        .map_err(|_| invalid("adapter-baseline"))?;
    let baseline_path = baseline.directory().join("kraqel.sqlite3");
    drop(baseline);
    let baseline_hash = format!(
        "{:x}",
        Sha256::digest(io("baseline-before", fs::read(&baseline_path))?)
    );
    let mut adapter = OneShotAdapter::new(expected_id);
    let result = adapter.execute(&task, &mut input, &manager, || {
        let staged = fs::read_dir(manager.root()).is_ok_and(|mut entries| {
            entries.any(|e| {
                e.is_ok_and(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .starts_with(".kraqel-stage-")
                })
            })
        });
        let expiry = data.join("EXPIRE_AT_PUBLISH").exists() && staged;
        TaskControl {
            now_unix_ms: if expiry { task.expires_unix_ms } else { now() },
            cancelled: data.join("CANCEL").exists()
                || (data.join("CANCEL_AT_PUBLISH").exists() && staged),
        }
    });
    let replay = adapter
        .execute(&task, &mut input, &manager, || TaskControl {
            now_unix_ms: now(),
            cancelled: false,
        })
        .err();
    if replay != Some(TaskError::AlreadyConsumed) {
        return Err(invalid("adapter-replay-not-rejected"));
    }
    let projects = manager
        .list_projects()
        .map_err(|_| invalid("adapter-catalog"))?;
    let remaining = io("adapter-final-directories", fs::read_dir(manager.root()))?.count();
    let unchanged = format!(
        "{:x}",
        Sha256::digest(io("baseline-after", fs::read(&baseline_path))?)
    ) == baseline_hash;
    let (receipt, error) = match result {
        Ok(receipt) => (Some(receipt), None),
        Err(error) => (None, Some(error)),
    };
    let expected_projects = if receipt.is_some() { 2 } else { 1 };
    if !unchanged || projects.len() != expected_projects || remaining != expected_projects {
        return Err(invalid("adapter-import-not-atomic"));
    }
    let report = json!({"formatVersion":1,"taskId":expected_id,"receipt":receipt,"error":error,"replayError":replay,
        "token":token,"unselectedFileAccessDenied":denied,"baselineProjectId":baseline_id,"baselineSha256":baseline_hash,
        "existingProjectUnchanged":unchanged,"catalogProjects":projects.len(),"noStagingResidue":true});
    let bytes = serde_json::to_vec(&report).map_err(|_| invalid("adapter-response"))?;
    if bytes.len() > 4096 {
        return Err(invalid("adapter-response-budget"));
    }
    io("adapter-write-receipt", output.write_all(&bytes))?;
    io("adapter-sync-receipt", output.sync_all())?;
    Ok(json!({"appImportChildComplete":true}))
}

pub(in super::super) fn launch(root: &Path, scenario: &str) -> Result<Value> {
    let expected_error = match scenario {
        "valid" => None,
        "invalid" => Some(TaskError::ImportFailed),
        "expired" | "expire-at-publish" => Some(TaskError::Expired),
        "cancelled" | "cancel-at-publish" => Some(TaskError::Cancelled),
        "wrong-id" => Some(TaskError::InvalidTask),
        _ => return Err(invalid("unknown-import-scenario")),
    };
    if !crate::policy::valid_root(root, false) {
        return Err(invalid("import-needs-fresh-temp-root"));
    }
    let host = host_for(root)?;
    io("create-import-host", fs::create_dir(&host))?;
    let fixture = if scenario == "invalid" {
        &b"not a snapshot archive"[..]
    } else {
        SYNTHETIC_ARCHIVE
    };
    io(
        "seed-import-fixture",
        fs::write(host.join("input.kraqel"), fixture),
    )?;
    io(
        "seed-import-withheld",
        fs::write(host.join("withheld.txt"), b"Synthetic unselected file"),
    )?;
    let input = io(
        "open-import-fixture",
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(host.join("input.kraqel")),
    )?;
    file_info(&input)?;
    let mut pending = Pending::create(host.join("pending-receipt.tmp"))?;
    let id = Uuid::new_v4();
    let mut diagnostics = run_worker(
        root,
        &input,
        pending.file.as_ref().unwrap(),
        "import-child",
        &id.to_string(),
        |data| {
            let issued = now();
            let task = FileTask {
                contract_version: 1,
                task_id: if scenario == "wrong-id" {
                    Uuid::new_v4()
                } else {
                    id
                },
                operation: FileOperation::ImportSnapshotV3,
                byte_count: fixture.len() as u64,
                issued_unix_ms: if scenario == "expired" {
                    issued.saturating_sub(2000)
                } else {
                    issued
                },
                expires_unix_ms: if scenario == "expired" {
                    issued.saturating_sub(1)
                } else {
                    issued + 60000
                },
            };
            io(
                "write-typed-task",
                fs::write(data.join("task.json"), serde_json::to_vec(&task).unwrap()),
            )?;
            let flag = match scenario {
                "cancelled" => Some("CANCEL"),
                "cancel-at-publish" => Some("CANCEL_AT_PUBLISH"),
                "expire-at-publish" => Some("EXPIRE_AT_PUBLISH"),
                _ => None,
            };
            if let Some(flag) = flag {
                io(
                    "set-synthetic-control",
                    fs::write(data.join(flag), b"synthetic test"),
                )?;
            }
            Ok(())
        },
    )?;
    drop(input);
    pending.close();
    let response: Value = serde_json::from_slice(&bounded(&pending.path, 4096)?)
        .map_err(|_| invalid("invalid-import-response"))?;
    let returned_error: Option<TaskError> = serde_json::from_value(response["error"].clone())
        .map_err(|_| invalid("invalid-import-error"))?;
    if response["taskId"] != id.to_string()
        || returned_error != expected_error
        || response["existingProjectUnchanged"] != true
        || response["noStagingResidue"] != true
        || response["token"]["isAppContainer"] != true
        || response["token"]["capabilityCount"] != 0
        || response["unselectedFileAccessDenied"] != true
        || response["replayError"] != "ALREADY_CONSUMED"
    {
        return Err(invalid("import-response-mismatch"));
    }
    if expected_error.is_none() {
        let receipt: ImportReceipt = serde_json::from_value(response["receipt"].clone())
            .map_err(|_| invalid("invalid-import-receipt"))?;
        if receipt.contract_version != 1
            || receipt.task_id != id
            || receipt.byte_count != fixture.len() as u64
            || receipt.document_count != 2
            || receipt.sha256.len() != 64
        {
            return Err(invalid("import-receipt-binding"));
        }
    } else if !response["receipt"].is_null() {
        return Err(invalid("rejected-import-returned-project"));
    }
    pending.publish(&host.join("receipt.json"))?;
    diagnostics["formatVersion"] = json!(1);
    diagnostics["exchange"] = json!(true);
    diagnostics["appImport"] = json!(true);
    diagnostics["passed"] = json!(true);
    diagnostics["scenario"] = json!(scenario);
    diagnostics["worker"] = response;
    diagnostics["hostDirectoryAclGranted"] = json!(false);
    diagnostics["inheritedFileHandles"] = json!(2);
    write_json(&root.join("result.json"), &diagnostics)?;
    Ok(diagnostics)
}
