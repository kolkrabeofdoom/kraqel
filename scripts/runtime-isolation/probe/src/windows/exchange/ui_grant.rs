//! One-way launcher grant. No post-launch channel from the renderer to the host.
use super::*;
use kraqel_isolation_test_adapter::{FileOperation, FileTask, SYNTHETIC_ARCHIVE};
use kraqel_isolation_transport::write_frame;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum UiPhase {
    Done,
    Cancelled,
    Failed,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct UiReceipt {
    project_id: Uuid,
    document_count: u32,
    asset_count: u32,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UiOutcome {
    phase: UiPhase,
    error: Option<kraqel_isolation_test_adapter::TaskError>,
    receipt: Option<UiReceipt>,
}

use windows_sys::Win32::System::Pipes::CreatePipe;

pub(in super::super) fn launch(root: &Path, app: &Path, picker: bool) -> Result<Value> {
    if !crate::policy::valid_root(root, false)
        || !root
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.starts_with("kraqel-isolation-ui-"))
    {
        return Err(invalid("ui-needs-fresh-synthetic-root"));
    }
    let host = host_for(root)?;
    io("create-ui-host", fs::create_dir(&host))?;
    io(
        "seed-ui-archive",
        fs::write(host.join("input.kraqel"), SYNTHETIC_ARCHIVE),
    )?;
    io(
        "seed-ui-withheld",
        fs::write(host.join("withheld.txt"), b"Synthetic unselected file"),
    )?;
    let source = if picker {
        let Some(selected) = choose_named(&host, false, "input.kraqel", true)? else {
            return Ok(
                json!({"exchange":true,"passed":true,"cancelled":true,"productionAcceptance":false}),
            );
        };
        if selected != host.join("input.kraqel") {
            return Err(invalid("ui-only-synthetic-archive"));
        }
        selected
    } else {
        host.join("input.kraqel")
    };
    let input = io(
        "open-ui-archive",
        OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(source),
    )?;
    let info = file_info(&input)?;
    let size = (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow);
    let mut read = null_mut();
    let mut write = null_mut();
    check(
        unsafe { CreatePipe(&mut read, &mut write, null(), 4096) },
        "create-one-way-grant-pipe",
    )?;
    let reader = unsafe { fs::File::from_raw_handle(read) };
    let mut writer = Some(unsafe { fs::File::from_raw_handle(write) });
    let id = Uuid::new_v4();
    let mut report = run_program(
        root,
        &input,
        &reader,
        Program::Ui {
            executable: app,
            id,
        },
        |_| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|v| u64::try_from(v.as_millis()).ok())
                .ok_or(invalid("grant-clock"))?;
            let task = FileTask {
                contract_version: 1,
                task_id: id,
                operation: FileOperation::ImportSnapshotV3,
                byte_count: size,
                issued_unix_ms: now,
                expires_unix_ms: now + 60000,
            };
            write_frame(writer.as_mut().unwrap(), &task)
                .map_err(|_| invalid("write-grant-frame"))?;
            drop(writer.take()); // EOF is required before the receiver can accept the only frame.
            Ok(())
        },
    )?;
    report["exchange"] = json!(true);
    report["uiMode"] = json!(true);
    report["passed"] = json!(true);
    report["inheritedHandles"] = json!(2);
    report["channelDirection"] = json!("controller-to-container-only");
    report["hostDirectoryAclGranted"] = json!(false);
    if let Some(outcome) = read_outcome(root)? {
        report["uiOutcome"] = outcome;
    }
    write_json(&root.join("result.json"), &report)?;
    Ok(report)
}

pub(super) fn read_outcome(root: &Path) -> Result<Option<Value>> {
    let result_path = root.join("data/ui-result.json");
    if result_path.exists() {
        let mut bytes = Vec::new();
        io(
            "read-ui-outcome",
            io("open-ui-outcome", fs::File::open(result_path))?
                .take(4097)
                .read_to_end(&mut bytes),
        )?;
        if bytes.len() > 4096 {
            return Err(invalid("ui-outcome-budget"));
        }
        let outcome: UiOutcome =
            serde_json::from_slice(&bytes).map_err(|_| invalid("ui-outcome-invalid"))?;
        let consistent = match outcome.phase {
            UiPhase::Done => outcome.receipt.is_some() && outcome.error.is_none(),
            UiPhase::Cancelled => {
                outcome.receipt.is_none()
                    && outcome.error == Some(kraqel_isolation_test_adapter::TaskError::Cancelled)
            }
            UiPhase::Failed => outcome.receipt.is_none() && outcome.error.is_some(),
        };
        if !consistent {
            return Err(invalid("ui-outcome-inconsistent"));
        }
        return serde_json::to_value(outcome)
            .map(Some)
            .map_err(|_| invalid("ui-outcome-encoding"));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn controller_never_accepts_paths_or_free_text_in_outcome() {
        let valid = json!({"phase":"done","error":null,"receipt":{"projectId":Uuid::new_v4(),"documentCount":2,"assetCount":0}});
        assert!(serde_json::from_value::<UiOutcome>(valid.clone()).is_ok());
        let mut extra = valid.clone();
        extra["path"] = json!("C:\\private");
        assert!(serde_json::from_value::<UiOutcome>(extra).is_err());
        let mut path = valid.clone();
        path["receipt"]["projectId"] = json!("../private");
        assert!(serde_json::from_value::<UiOutcome>(path).is_err());
        let mut error = valid;
        error["error"] = json!("private parser detail");
        assert!(serde_json::from_value::<UiOutcome>(error).is_err());
    }
}
