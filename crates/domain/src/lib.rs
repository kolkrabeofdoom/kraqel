//! Gemeinsame Domänentypen und versionierte Verträge der privilegierten Rust-Grenze.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Aktuelle Version des lokalen Projektschemas.
pub const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion(1);

/// Aktuelle Version des Command-Vertrags zwischen UI und Rust.
pub const COMMAND_CONTRACT_VERSION: u16 = 1;

/// Stabile Projektkennung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectId(Uuid);

impl ProjectId {
    /// Erzeugt eine neue zufällige Projektkennung.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

/// Stabile Kennung einer projektbezogenen Entität.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Erzeugt eine neue zufällige Entitätskennung.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

/// Version eines gespeicherten Projektschemas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(pub u32);

/// Dokumentart innerhalb des Kapitel- und Szenenbaums.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DocumentKind {
    /// Struktureller Kapitelknoten.
    Chapter,
    /// Schreibbare Szene.
    Scene,
}

/// Minimales Projektmodell für Storage- und Command-Verträge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Stabile Kennung.
    pub id: ProjectId,
    /// Von Schreibenden vergebener Anzeigename.
    pub name: String,
    /// Version des zugrunde liegenden Datenschemas.
    pub schema_version: SchemaVersion,
}

/// Minimales Dokumentmodell für Kapitel und Szenen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentNode {
    /// Stabile Kennung.
    pub id: EntityId,
    /// Zugehöriges Projekt.
    pub project_id: ProjectId,
    /// Übergeordneter Knoten oder `None` für einen Wurzelknoten.
    pub parent_id: Option<EntityId>,
    /// Art des Dokuments.
    pub kind: DocumentKind,
    /// Anzeigename.
    pub title: String,
    /// Nullbasierte Position unter den Geschwisterknoten.
    pub position: u32,
    /// Monoton steigende fachliche Revision.
    pub revision: u64,
}

/// Versionierte Anfragehülle für Tauri-Commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandRequest<T> {
    /// Version des Command-Vertrags.
    pub contract_version: u16,
    /// Korrelationskennung ohne fachliche Bedeutung.
    pub request_id: Uuid,
    /// Commandspezifische Nutzlast.
    pub payload: T,
}

/// Versionierte Erfolgshülle für Tauri-Commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResponse<T> {
    /// Version des Command-Vertrags.
    pub contract_version: u16,
    /// Korrelationskennung der Anfrage.
    pub request_id: Uuid,
    /// Commandspezifische Antwortdaten.
    pub data: T,
}

impl<T> CommandResponse<T> {
    /// Erzeugt eine Antwort mit der aktuellen Vertragsversion.
    #[must_use]
    pub const fn current(request_id: Uuid, data: T) -> Self {
        Self {
            contract_version: COMMAND_CONTRACT_VERSION,
            request_id,
            data,
        }
    }
}

/// Stabile, maschinenlesbare Fehlerklassen der Command-Grenze.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CommandErrorCode {
    /// Anfrage verletzt einen öffentlichen Vertrag.
    InvalidRequest,
    /// Angefragte Vertragsversion wird nicht unterstützt.
    UnsupportedContractVersion,
    /// Unerwarteter interner Fehler ohne offengelegte Details.
    Internal,
}

/// Bereinigter Fehlervertrag ohne Pfade, Eingaben oder interne Diagnosedaten.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    /// Version des Command-Vertrags.
    pub contract_version: u16,
    /// Korrelationskennung der Anfrage.
    pub request_id: Uuid,
    /// Maschinenlesbare Fehlerklasse.
    pub code: CommandErrorCode,
    /// Stabile, nutzer*innengeeignete Meldung.
    pub message: &'static str,
    /// Ob ein unveränderter erneuter Versuch sinnvoll sein kann.
    pub retryable: bool,
}

impl CommandError {
    /// Meldet eine nicht unterstützte Vertragsversion ohne die empfangene Nutzlast offenzulegen.
    #[must_use]
    pub const fn unsupported_contract(request_id: Uuid) -> Self {
        Self {
            contract_version: COMMAND_CONTRACT_VERSION,
            request_id,
            code: CommandErrorCode::UnsupportedContractVersion,
            message: "Die Command-Vertragsversion wird nicht unterstützt.",
            retryable: false,
        }
    }

    /// Meldet eine ungültige Anfrage mit einer vorab definierten, bereinigten Nachricht.
    #[must_use]
    pub const fn invalid_request(request_id: Uuid, message: &'static str) -> Self {
        Self {
            contract_version: COMMAND_CONTRACT_VERSION,
            request_id,
            code: CommandErrorCode::InvalidRequest,
            message,
            retryable: false,
        }
    }
}

/// Prüft die Vertragsversion einer Anfrage.
///
/// # Errors
///
/// Liefert einen bereinigten Fehler, wenn die Version nicht aktuell ist.
pub const fn require_current_contract<T>(request: &CommandRequest<T>) -> Result<(), CommandError> {
    if request.contract_version == COMMAND_CONTRACT_VERSION {
        Ok(())
    } else {
        Err(CommandError::unsupported_contract(request.request_id))
    }
}

/// Prüft die fachlichen Grenzen eines Projektnamens.
#[must_use]
pub fn is_valid_project_name(name: &str) -> bool {
    let trimmed = name.trim();
    !trimmed.is_empty() && trimmed.chars().count() <= 120
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::{
        COMMAND_CONTRACT_VERSION, CommandErrorCode, CommandRequest, CommandResponse,
        is_valid_project_name, require_current_contract,
    };

    const REQUEST_ID: Uuid = Uuid::from_u128(0x018f_47ea_9b2d_7cc1_98e8_b5f4_f6b7_41a2);

    #[test]
    fn request_and_response_use_camel_case_contracts() {
        let request = CommandRequest {
            contract_version: COMMAND_CONTRACT_VERSION,
            request_id: REQUEST_ID,
            payload: json!({"name": "Morgenrot"}),
        };
        let response = CommandResponse::current(REQUEST_ID, json!({"valid": true}));

        assert_eq!(
            serde_json::to_value(request).expect("request should serialize"),
            json!({
                "contractVersion": 1,
                "requestId": "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a2",
                "payload": {"name": "Morgenrot"}
            })
        );
        assert_eq!(
            serde_json::to_value(response).expect("response should serialize"),
            json!({
                "contractVersion": 1,
                "requestId": "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a2",
                "data": {"valid": true}
            })
        );
    }

    #[test]
    fn contract_error_discloses_no_payload_or_path() {
        let request = CommandRequest {
            contract_version: 99,
            request_id: REQUEST_ID,
            payload: "D:\\private\\unfinished-manuscript.md",
        };

        let error = require_current_contract(&request).expect_err("version should be rejected");
        let serialized = serde_json::to_string(&error).expect("error should serialize");

        assert_eq!(error.code, CommandErrorCode::UnsupportedContractVersion);
        assert!(!serialized.contains("private"));
        assert!(!serialized.contains("manuscript"));
        assert!(!serialized.contains("99"));
    }

    #[test]
    fn project_name_is_trimmed_and_unicode_aware() {
        assert!(is_valid_project_name(" Die Stadt unter dem Meer "));
        assert!(!is_valid_project_name("   "));
        assert!(!is_valid_project_name(&"Ä".repeat(121)));
    }
}
