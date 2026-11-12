//! Gemeinsame Domänentypen und versionierte Verträge der privilegierten Rust-Grenze.

use std::{collections::BTreeMap, fmt};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Aktuelle Version des lokalen Projektschemas.
pub const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion(1);

/// Aktuelle Version des gespeicherten Tiptap-Dokumentvertrags.
pub const EDITOR_DOCUMENT_VERSION: u16 = 1;

/// Aktuelle Version des Command-Vertrags zwischen UI und Rust.
pub const COMMAND_CONTRACT_VERSION: u16 = 1;

/// Stabile Projektkennung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectId(Uuid);

impl ProjectId {
    /// Erzeugt eine neue zufällige Projektkennung.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Liefert die zugrunde liegende UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }

    /// Liest eine kanonische UUID als Projektkennung.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Uuid::parse_str(value).ok().map(Self)
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

/// Stabile Kennung einer projektbezogenen Entität.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(Uuid);

impl EntityId {
    /// Erzeugt eine neue zufällige Entitätskennung.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Liefert die zugrunde liegende UUID.
    #[must_use]
    pub const fn as_uuid(self) -> Uuid {
        self.0
    }

    /// Liest eine kanonische UUID als Entitätskennung.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Uuid::parse_str(value).ok().map(Self)
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
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

/// Fachlich unterstützte Arten von Story-Bibel-Einträgen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntityKind {
    /// Handelnde oder erwähnte Figur.
    Character,
    /// Räumlicher Schauplatz.
    Location,
    /// Organisierte Gruppe oder Partei.
    Faction,
    /// Bedeutender Gegenstand.
    Item,
}

/// Ein Eintrag der Story-Bibel mit frei benennbaren Eigenschaften.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryEntity {
    /// Stabile Kennung.
    pub id: EntityId,
    /// Zugehöriges Projekt.
    pub project_id: ProjectId,
    /// Fachliche Art des Eintrags.
    pub kind: EntityKind,
    /// Anzeigename.
    pub name: String,
    /// Alphabetisch stabile freie Eigenschaften.
    pub properties: BTreeMap<String, String>,
}

/// Gerichtete Beziehung zwischen zwei Story-Bibel-Einträgen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryRelation {
    /// Stabile Kennung der Beziehung.
    pub id: EntityId,
    /// Zugehöriges Projekt.
    pub project_id: ProjectId,
    /// Ausgangspunkt der gerichteten Beziehung.
    pub source_entity_id: EntityId,
    /// Ziel der gerichteten Beziehung.
    pub target_entity_id: EntityId,
    /// Frei benannte Beziehungsart, beispielsweise `kennt`.
    pub kind: String,
    /// Alphabetisch stabile freie Eigenschaften.
    pub properties: BTreeMap<String, String>,
}

/// Rückverweis von einem Story-Bibel-Eintrag auf ein Dokument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryBacklink {
    /// Kennung des verknüpften Dokuments.
    pub document_id: EntityId,
    /// Anzeigename des Dokuments.
    pub document_title: String,
    /// Art des Dokuments.
    pub document_kind: DocumentKind,
    /// Fachliche Rolle der Verknüpfung.
    pub role: String,
}

/// Detailansicht eines Story-Bibel-Eintrags samt Beziehungen und Rückverweisen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryEntityDetails {
    /// Der ausgewählte Eintrag.
    pub entity: StoryEntity,
    /// Vom Eintrag ausgehende Beziehungen.
    pub outgoing_relations: Vec<StoryRelation>,
    /// Auf den Eintrag gerichtete Beziehungen.
    pub incoming_relations: Vec<StoryRelation>,
    /// Dokumente, die auf den Eintrag verweisen.
    pub backlinks: Vec<StoryBacklink>,
}

/// Bearbeitungsstand einer Szenenkarte im Plotboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlotStatus {
    /// Geplante, noch nicht ausgearbeitete Szene.
    Planned,
    /// Szene im Entwurf.
    Draft,
    /// Überarbeitete Szene.
    Revised,
    /// Für den aktuellen Stand abgeschlossene Szene.
    Done,
}

/// Projektweiter Handlungsstrang.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Storyline {
    /// Stabile Kennung.
    pub id: EntityId,
    /// Zugehöriges Projekt.
    pub project_id: ProjectId,
    /// Anzeigename.
    pub name: String,
    /// Darstellungsfarbe als Hex-Wert.
    pub color: String,
    /// Stabile Reihenfolge.
    pub position: u32,
}

/// Persistente Plotboard-Metadaten einer Szene.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotCard {
    /// Kennung der Szene.
    pub document_id: EntityId,
    /// Szenentitel für die Darstellung.
    pub title: String,
    /// Optionaler Handlungsstrang.
    pub storyline_id: Option<EntityId>,
    /// Optionale POV-Figur.
    pub pov_entity_id: Option<EntityId>,
    /// Bearbeitungsstand.
    pub status: PlotStatus,
    /// Stabile Reihenfolge im Plotboard.
    pub position: u32,
}

/// Vollständiger Plotboard-Stand eines Projekts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlotBoard {
    /// Handlungsstränge in stabiler Reihenfolge.
    pub storylines: Vec<Storyline>,
    /// Szenenkarten in stabiler Reihenfolge.
    pub cards: Vec<PlotCard>,
}

/// Ordnungsmodell einer Zeitebene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimelineMode {
    /// Ereignisse sind ausschließlich relativ zueinander geordnet.
    Relative,
    /// Ereignisse besitzen zusätzlich einen expliziten Zeitpunkt.
    Absolute,
}

/// Parallele Zeitebene eines Projekts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    /// Stabile Kennung.
    pub id: EntityId,
    /// Zugehöriges Projekt.
    pub project_id: ProjectId,
    /// Anzeigename.
    pub name: String,
    /// Relatives oder absolutes Ordnungsmodell.
    pub mode: TimelineMode,
    /// Stabile Reihenfolge der Zeitebenen.
    pub position: u32,
}

/// Ereignis mit nachvollziehbaren Szenen- und Entitätslinks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEvent {
    /// Stabile Kennung.
    pub id: EntityId,
    /// Zugehöriges Projekt.
    pub project_id: ProjectId,
    /// Zugehörige Zeitebene.
    pub timeline_id: EntityId,
    /// Anzeigename.
    pub title: String,
    /// Optionale Beschreibung.
    pub description: String,
    /// Expliziter Zeitpunkt bei absoluten Zeitebenen.
    pub time_value: Option<String>,
    /// Stabile Reihenfolge innerhalb der Zeitebene.
    pub position: u32,
    /// Verknüpfte Szenen.
    pub document_ids: Vec<EntityId>,
    /// Verknüpfte Story-Bibel-Einträge.
    pub entity_ids: Vec<EntityId>,
}

/// Vollständige Zeitebene mit stabil sortierten Ereignissen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineDetails {
    /// Metadaten der Zeitebene.
    pub timeline: Timeline,
    /// Ereignisse in fachlicher Reihenfolge.
    pub events: Vec<TimelineEvent>,
}

/// Aktuell unterstützte Medienart der lokalen Bibliothek.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    /// Lokal importierte Rastergrafik.
    Image,
}

/// Nachvollziehbare Herkunft eines Medienassets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaProvenance {
    /// Bewusst von einer schreibenden Person importiert.
    Imported,
}

/// Fachliche Zielart einer Assetverknüpfung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AssetTargetKind {
    Document,
    Entity,
    Event,
}

/// Stabile Verknüpfung eines Assets mit einem Projektobjekt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetLink {
    pub target_kind: AssetTargetKind,
    pub target_id: EntityId,
    pub role: String,
}

/// Metadaten eines lokalen, inhaltsgeprüften Medienassets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAsset {
    pub id: EntityId,
    pub project_id: ProjectId,
    pub kind: MediaKind,
    pub relative_path: String,
    pub thumbnail_path: String,
    pub original_name: String,
    pub title: String,
    pub mime_type: String,
    pub checksum: String,
    pub byte_count: u64,
    pub width: u32,
    pub height: u32,
    pub provenance: MediaProvenance,
    pub tags: Vec<String>,
    pub links: Vec<AssetLink>,
}

/// Konsistenzvorgaben für die visuelle Darstellung eines Story-Bibel-Eintrags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualProfile {
    pub entity_id: EntityId,
    pub project_id: ProjectId,
    pub traits: BTreeMap<String, String>,
    pub canonical_asset_id: Option<EntityId>,
    pub exclusions: Vec<String>,
}

/// Projektweite, wiederverwendbare Regeln für einen visuellen Stil.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StylePreset {
    pub id: EntityId,
    pub project_id: ProjectId,
    pub name: String,
    pub visual_rules: BTreeMap<String, String>,
    pub applicable_kinds: Vec<EntityKind>,
}

/// Lokal aus explizit ausgewählten Projektquellen zusammengesetzter Bildprompt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePromptComposition {
    pub project_id: ProjectId,
    pub document_id: EntityId,
    pub document_title: String,
    pub document_revision: u64,
    pub entity_ids: Vec<EntityId>,
    pub style_preset_id: Option<EntityId>,
    pub prompt_text: String,
}

/// Projektgebundener, vollständig editierbarer Bildprompt-Entwurf.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagePromptDraft {
    pub id: EntityId,
    pub project_id: ProjectId,
    pub name: String,
    pub source_document_id: EntityId,
    pub source_document_revision: u64,
    pub entity_ids: Vec<EntityId>,
    pub style_preset_id: Option<EntityId>,
    pub prompt_text: String,
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

/// Versionierter, in `SQLite` gespeicherter Tiptap-Dokumentinhalt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorDocument {
    /// Version des Editor-Datenvertrags.
    pub version: u16,
    /// Tiptap-/ProseMirror-JSON mit einem Wurzelknoten vom Typ `doc`.
    pub document: serde_json::Value,
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
    /// Optimistischer Schreibstand ist nicht mehr aktuell.
    Conflict,
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

    /// Meldet einen optimistischen Revisionskonflikt.
    #[must_use]
    pub const fn conflict(request_id: Uuid) -> Self {
        Self {
            contract_version: COMMAND_CONTRACT_VERSION,
            request_id,
            code: CommandErrorCode::Conflict,
            message: "Das Dokument wurde zwischenzeitlich verändert.",
            retryable: false,
        }
    }

    /// Meldet einen bereinigten internen Fehler.
    #[must_use]
    pub const fn internal(request_id: Uuid) -> Self {
        Self {
            contract_version: COMMAND_CONTRACT_VERSION,
            request_id,
            code: CommandErrorCode::Internal,
            message: "Die lokale Anwendung konnte die Anfrage nicht verarbeiten.",
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
