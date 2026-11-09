use kraqel_domain::{
    CommandError, CommandRequest, CommandResponse, is_valid_project_name, require_current_contract,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ApplicationStatusRequest {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApplicationStatusResponse {
    version: &'static str,
    schema_version: u32,
    offline_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ValidateProjectNameRequest {
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct ValidateProjectNameResponse {
    valid: bool,
}

#[tauri::command]
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri übergibt deserialisierte Command-Argumente im Besitz des Handlers"
)]
fn application_status(
    request: CommandRequest<ApplicationStatusRequest>,
) -> Result<CommandResponse<ApplicationStatusResponse>, CommandError> {
    require_current_contract(&request)?;
    let ApplicationStatusRequest {} = request.payload;

    Ok(CommandResponse::current(
        request.request_id,
        ApplicationStatusResponse {
            version: env!("CARGO_PKG_VERSION"),
            schema_version: kraqel_domain::CURRENT_SCHEMA_VERSION.0,
            offline_ready: true,
        },
    ))
}

#[tauri::command]
fn validate_project_name(
    request: CommandRequest<ValidateProjectNameRequest>,
) -> Result<CommandResponse<ValidateProjectNameResponse>, CommandError> {
    require_current_contract(&request)?;
    let name = request.payload.name;

    Ok(CommandResponse::current(
        request.request_id,
        ValidateProjectNameResponse {
            valid: is_valid_project_name(&name),
        },
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Startet die Desktop-Anwendung.
///
/// # Panics
///
/// Bricht ab, wenn die Tauri-Laufzeit nicht initialisiert oder ausgeführt werden kann.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            application_status,
            validate_project_name
        ])
        .run(tauri::generate_context!())
        .expect("KRAQEL konnte nicht gestartet werden");
}

#[cfg(test)]
mod tests {
    use kraqel_domain::{COMMAND_CONTRACT_VERSION, CommandErrorCode, CommandRequest};
    use uuid::Uuid;

    use super::{
        ApplicationStatusRequest, ValidateProjectNameRequest, application_status,
        validate_project_name,
    };

    const REQUEST_ID: Uuid = Uuid::from_u128(0x018f_47ea_9b2d_7cc1_98e8_b5f4_f6b7_41a2);

    #[test]
    fn status_reports_versioned_offline_foundation() {
        let response = application_status(CommandRequest {
            contract_version: COMMAND_CONTRACT_VERSION,
            request_id: REQUEST_ID,
            payload: ApplicationStatusRequest {},
        })
        .expect("current contract should succeed");

        assert_eq!(response.request_id, REQUEST_ID);
        assert_eq!(response.data.version, "0.1.0");
        assert!(response.data.offline_ready);
    }

    #[test]
    fn validation_returns_only_the_result_not_the_supplied_name() {
        let response = validate_project_name(CommandRequest {
            contract_version: COMMAND_CONTRACT_VERSION,
            request_id: REQUEST_ID,
            payload: ValidateProjectNameRequest {
                name: "Die linke Hand der Dunkelheit".to_owned(),
            },
        })
        .expect("valid request should succeed");
        let json = serde_json::to_string(&response).expect("response should serialize");

        assert!(response.data.valid);
        assert!(!json.contains("Dunkelheit"));
    }

    #[test]
    fn unsupported_contract_returns_sanitized_error() {
        let error = application_status(CommandRequest {
            contract_version: 0,
            request_id: REQUEST_ID,
            payload: ApplicationStatusRequest {},
        })
        .expect_err("old contract should fail");

        assert_eq!(error.code, CommandErrorCode::UnsupportedContractVersion);
        assert!(!error.retryable);
    }
}
