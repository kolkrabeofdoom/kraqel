//! Providerneutrale Bildfähigkeiten und Job-Verträge ohne aktive Bildgenerierung.

use std::{collections::BTreeSet, error::Error, fmt};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ProviderLocation;

/// Aktuelle Version des Bildgenerierungsvertrags.
pub const IMAGE_GENERATION_CONTRACT_VERSION: u16 = 1;

const MAX_PROVIDERS: usize = 64;
const MAX_MODELS_PER_PROVIDER: usize = 256;
const MAX_PROMPT_CHARACTERS: u32 = 200_000;
const MAX_REFERENCE_IMAGES: u16 = 32;
const MAX_OUTPUTS: u8 = 8;
const MAX_DIMENSION: u32 = 16_384;

/// Eine von einem konkreten Bildmodell angebotene Fähigkeit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ImageCapability {
    /// Erzeugt ein neues Bild aus einem freigegebenen Prompt.
    #[serde(rename = "image.generate")]
    Generate,
    /// Bearbeitet ein vorhandenes Bild.
    #[serde(rename = "image.edit")]
    Edit,
    /// Berücksichtigt Referenzbilder für wiederkehrende Merkmale.
    #[serde(rename = "image.reference")]
    Reference,
    /// Rendert ein späteres strukturiertes Kartenmodell.
    #[serde(rename = "image.map_render")]
    MapRender,
}

/// Harte, zur Laufzeit bekannte Grenzen eines Bildmodells.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageModelLimits {
    /// Maximale Promptlänge in Unicode-Zeichen.
    pub max_prompt_characters: u32,
    /// Maximale Anzahl freigegebener Referenzbilder.
    pub max_reference_images: u16,
    /// Unterstützte Bildbreiten.
    pub supported_widths: Vec<u32>,
    /// Unterstützte Bildhöhen.
    pub supported_heights: Vec<u32>,
    /// Maximale Zahl von Ergebnissen pro Auftrag.
    pub max_outputs: u8,
    /// Ob ein expliziter Seed unterstützt wird.
    pub supports_seed: bool,
}

/// Modellspezifische Fähigkeiten und Grenzen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageModelDescriptor {
    /// Providerinterne, in der Registry stabile Modellkennung.
    pub id: String,
    /// Menschenlesbarer Modellname.
    pub display_name: String,
    /// Nur die für dieses Modell nachgewiesenen Fähigkeiten.
    pub capabilities: Vec<ImageCapability>,
    /// Validierbare Grenzen für Aufträge.
    pub limits: ImageModelLimits,
}

/// Providerbeschreibung ohne Schlüssel, Pfade oder Netzwerkzugriff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageProviderDescriptor {
    /// Stabile, kleingeschriebene Providerkennung.
    pub id: String,
    /// Menschenlesbarer Providername.
    pub display_name: String,
    /// Lokale oder externe Verarbeitung.
    pub location: ProviderLocation,
    /// Ob der Provider in dieser Installation verwendbar ist.
    pub available: bool,
    /// Aktuell bekannte Modelle; eine leere Liste ist zulässig.
    pub models: Vec<ImageModelDescriptor>,
    /// Datenarme Erklärung bei Nichtverfügbarkeit.
    pub unavailable_reason: Option<String>,
}

/// Offline auswertbare Registry für Provider und deren konkrete Modelle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageCapabilityRegistry {
    providers: Vec<ImageProviderDescriptor>,
}

impl ImageCapabilityRegistry {
    /// Erstellt eine validierte, deterministisch sortierte Registry.
    ///
    /// # Errors
    ///
    /// Lehnt doppelte Kennungen, ungültige Grenzen und widersprüchliche Verfügbarkeit ab.
    pub fn new(mut providers: Vec<ImageProviderDescriptor>) -> Result<Self, ImageGenerationError> {
        if providers.len() > MAX_PROVIDERS {
            return Err(ImageGenerationError::invalid_request());
        }
        let mut provider_ids = BTreeSet::new();
        for provider in &mut providers {
            validate_provider(provider)?;
            if !provider_ids.insert(provider.id.clone()) {
                return Err(ImageGenerationError::invalid_request());
            }
            provider
                .models
                .sort_by(|left, right| left.id.cmp(&right.id));
        }
        providers.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(Self { providers })
    }

    /// Leere Registry für Installationen ohne Bildprovider.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Liefert Provider in stabiler Kennungsreihenfolge.
    #[must_use]
    pub fn providers(&self) -> &[ImageProviderDescriptor] {
        &self.providers
    }

    /// Prüft einen Auftrag vollständig, bevor ein Provider angesprochen werden dürfte.
    ///
    /// # Errors
    ///
    /// Liefert ausschließlich datenarme, stabile Fehler für ungültige oder nicht unterstützte
    /// Aufträge.
    pub fn validate_request(
        &self,
        request: &ImageGenerationRequest,
    ) -> Result<(), ImageGenerationError> {
        validate_request_identity(request)?;
        let provider = self
            .providers
            .iter()
            .find(|provider| provider.id == request.provider_id)
            .ok_or_else(ImageGenerationError::provider_unavailable)?;
        if !provider.available {
            return Err(ImageGenerationError::provider_unavailable());
        }
        let model = provider
            .models
            .iter()
            .find(|model| model.id == request.model_id)
            .ok_or_else(ImageGenerationError::model_unavailable)?;
        if !model.capabilities.contains(&request.capability)
            || (!request.context.reference_asset_ids.is_empty()
                && !model.capabilities.contains(&ImageCapability::Reference))
        {
            return Err(ImageGenerationError::unsupported_capability());
        }
        validate_settings(&request.settings, &model.limits)?;
        let prompt_characters = request.context.prompt_text.chars().count();
        if request.context.prompt_text.trim().is_empty()
            || prompt_characters > model.limits.max_prompt_characters as usize
            || request.context.reference_asset_ids.len()
                > model.limits.max_reference_images as usize
        {
            return Err(ImageGenerationError::invalid_request());
        }
        if !request.context.approved {
            return Err(ImageGenerationError::context_not_approved());
        }
        Ok(())
    }

    /// Validiert Registry und Freigabe, bevor der ausgewählte Provider aufgerufen wird.
    ///
    /// # Errors
    ///
    /// Lehnt unbekannte oder abweichend beschriebene Provider sowie ungültige Aufträge ab und
    /// reicht ausschließlich den bereinigten Providerfehler weiter.
    pub async fn run_with(
        &self,
        provider: &dyn ImageProvider,
        request: &ImageGenerationRequest,
    ) -> Result<ImageGenerationProvenance, ImageGenerationError> {
        let descriptor = provider.descriptor();
        let Some(registered) = self
            .providers
            .iter()
            .find(|registered| registered.id == request.provider_id)
        else {
            return Err(ImageGenerationError::provider_unavailable());
        };
        if registered != &descriptor {
            return Err(ImageGenerationError::provider_unavailable());
        }
        self.validate_request(request)?;
        provider.generate(request).await
    }
}

/// Explizit freigegebener Bildkontext; eine Promptansicht allein setzt `approved` nicht.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageContextRelease {
    /// Kennung des zuvor angezeigten Prompt-Entwurfs.
    pub prompt_draft_id: Uuid,
    /// Exakt der freigegebene Prompttext.
    pub prompt_text: String,
    /// Exakt die freigegebenen Referenzassets.
    pub reference_asset_ids: Vec<Uuid>,
    /// Nur eine unmittelbar bestätigte Vorschau darf diesen Wert setzen.
    pub approved: bool,
}

/// Providerneutrale Einstellungen eines Bildauftrags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationSettings {
    /// Zielbreite in Pixeln.
    pub width: u32,
    /// Zielhöhe in Pixeln.
    pub height: u32,
    /// Zahl gewünschter Ergebnisse.
    pub output_count: u8,
    /// Optionaler reproduzierbarer Seed, sofern unterstützt.
    pub seed: Option<u32>,
}

/// Versionierter Auftrag an einen späteren lokalen oder ausdrücklich freigegebenen Provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationRequest {
    /// Version dieses Datenvertrags.
    pub contract_version: u16,
    /// Vor dem Lauf erzeugte stabile Jobkennung.
    pub job_id: Uuid,
    /// Projekt, dem Entwurf und mögliche Ergebnisse gehören.
    pub project_id: Uuid,
    /// Gewählter Provider.
    pub provider_id: String,
    /// Gewähltes Modell.
    pub model_id: String,
    /// Gewünschte Primärfähigkeit.
    pub capability: ImageCapability,
    /// Angezeigter und explizit freigegebener Kontext.
    pub context: ImageContextRelease,
    /// Gegen Modellgrenzen zu prüfende Einstellungen.
    pub settings: ImageGenerationSettings,
}

/// Zustand eines Generation-Jobs; erst 0.2 führt echte Läufe aus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImageGenerationJobStatus {
    /// Auftrag ist validiert, aber noch nicht gestartet.
    Queued,
    /// Provider verarbeitet den Auftrag.
    Running,
    /// Auftrag endete mit verwerfbaren Ergebnissen.
    Succeeded,
    /// Auftrag endete mit einem bereinigten Fehler.
    Failed,
    /// Auftrag wurde ohne automatische Übernahme beendet.
    Cancelled,
}

/// Technische Herkunft eines späteren Ergebnisses ohne Lizenzbehauptung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationProvenance {
    /// Tatsächlich verwendeter Provider.
    pub provider_id: String,
    /// Tatsächlich verwendetes Modell.
    pub model_id: String,
    /// Tatsächlich ausgeführte Fähigkeit.
    pub capability: ImageCapability,
    /// Providerseitig gemeldeter oder angeforderter Seed.
    pub seed: Option<u32>,
    /// RFC-3339-Zeitpunkt als transportneutraler Text.
    pub generated_at: String,
}

/// Versionierter, lokal speicherbarer Jobzustand ohne Bilddatei- oder Übernahmepfad.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationJob {
    /// Version dieses Datenvertrags.
    pub contract_version: u16,
    /// Stabile Jobkennung.
    pub id: Uuid,
    /// Zugehöriges Projekt.
    pub project_id: Uuid,
    /// Providerkennung.
    pub provider_id: String,
    /// Modellkennung.
    pub model_id: String,
    /// Angeforderte Fähigkeit.
    pub capability: ImageCapability,
    /// Quell-Prompt-Entwurf.
    pub prompt_draft_id: Uuid,
    /// Exakt freigegebener Prompttext.
    pub prompt_text: String,
    /// Exakt freigegebene Referenzassets.
    pub reference_asset_ids: Vec<Uuid>,
    /// Validierte Einstellungen.
    pub settings: ImageGenerationSettings,
    /// Aktueller Zustand.
    pub status: ImageGenerationJobStatus,
    /// Nur im Fehlerzustand gesetzter bereinigter Fehler.
    pub error: Option<ImageGenerationError>,
    /// Nur nach erfolgreichem Lauf gesetzte Herkunft.
    pub provenance: Option<ImageGenerationProvenance>,
    /// RFC-3339-Erstellungszeitpunkt.
    pub created_at: String,
    /// RFC-3339-Zeitpunkt der letzten Zustandsänderung.
    pub updated_at: String,
}

impl ImageGenerationJob {
    /// Erstellt aus einem validierten Auftrag einen wartenden Job.
    #[must_use]
    pub fn queued(request: &ImageGenerationRequest, now: impl Into<String>) -> Self {
        let now = now.into();
        Self {
            contract_version: request.contract_version,
            id: request.job_id,
            project_id: request.project_id,
            provider_id: request.provider_id.clone(),
            model_id: request.model_id.clone(),
            capability: request.capability,
            prompt_draft_id: request.context.prompt_draft_id,
            prompt_text: request.context.prompt_text.clone(),
            reference_asset_ids: request.context.reference_asset_ids.clone(),
            settings: request.settings.clone(),
            status: ImageGenerationJobStatus::Queued,
            error: None,
            provenance: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Setzt einen erlaubten Folgezustand und hält Fehler/Herkunft konsistent.
    ///
    /// # Errors
    ///
    /// Lehnt Sprünge aus Endzuständen sowie fehlende oder unpassende Ergebnisdaten ab.
    pub fn transition(
        &mut self,
        status: ImageGenerationJobStatus,
        error: Option<ImageGenerationError>,
        provenance: Option<ImageGenerationProvenance>,
        now: impl Into<String>,
    ) -> Result<(), ImageGenerationError> {
        let allowed = matches!(
            (self.status, status),
            (
                ImageGenerationJobStatus::Queued,
                ImageGenerationJobStatus::Running
                    | ImageGenerationJobStatus::Failed
                    | ImageGenerationJobStatus::Cancelled
            ) | (
                ImageGenerationJobStatus::Running,
                ImageGenerationJobStatus::Succeeded
                    | ImageGenerationJobStatus::Failed
                    | ImageGenerationJobStatus::Cancelled
            )
        );
        if !allowed
            || (status == ImageGenerationJobStatus::Failed) != error.is_some()
            || (status == ImageGenerationJobStatus::Succeeded) != provenance.is_some()
            || (status != ImageGenerationJobStatus::Failed && error.is_some())
            || (status != ImageGenerationJobStatus::Succeeded && provenance.is_some())
        {
            return Err(ImageGenerationError::invalid_state());
        }
        if let Some(value) = &provenance
            && (value.provider_id != self.provider_id
                || value.model_id != self.model_id
                || value.capability != self.capability)
        {
            return Err(ImageGenerationError::invalid_state());
        }
        self.status = status;
        self.error = error;
        self.provenance = provenance;
        self.updated_at = now.into();
        Ok(())
    }
}

/// Stabiler Fehlercode für UI und persistierte Jobzustände.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ImageGenerationErrorCode {
    /// Anfrage oder Registryeintrag verletzt den Vertrag.
    InvalidRequest,
    /// Die gewünschte Fähigkeit wird vom Modell nicht angeboten.
    UnsupportedCapability,
    /// Provider ist nicht eingerichtet oder erreichbar.
    ProviderUnavailable,
    /// Modell ist nicht vorhanden oder nicht mehr verfügbar.
    ModelUnavailable,
    /// Kontext wurde nicht explizit freigegeben.
    ContextNotApproved,
    /// Jobzustand oder Zustandswechsel ist widersprüchlich.
    InvalidState,
    /// Providerantwort wurde aus Sicherheitsgründen verworfen.
    OutputRejected,
    /// Unklassifizierter interner Fehler ohne Diagnosedaten.
    Internal,
}

/// Datenarmer Fehler ohne Prompt, Pfad, Providerantwort oder Schlüssel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageGenerationError {
    /// Maschinenlesbarer stabiler Code.
    code: ImageGenerationErrorCode,
    /// Nutzer*innengeeignete statische Meldung.
    message: String,
    /// Ob ein späterer Versuch grundsätzlich sinnvoll sein kann.
    retryable: bool,
}

impl ImageGenerationError {
    /// Liefert den stabilen Fehlercode.
    #[must_use]
    pub const fn code(&self) -> ImageGenerationErrorCode {
        self.code
    }

    /// Liefert ausschließlich die bereinigte statische Meldung.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Gibt an, ob ein erneuter Versuch grundsätzlich sinnvoll sein kann.
    #[must_use]
    pub const fn retryable(&self) -> bool {
        self.retryable
    }

    fn invalid_request() -> Self {
        Self::new(
            ImageGenerationErrorCode::InvalidRequest,
            "Der Bildauftrag ist ungültig.",
            false,
        )
    }

    fn unsupported_capability() -> Self {
        Self::new(
            ImageGenerationErrorCode::UnsupportedCapability,
            "Das gewählte Bildmodell unterstützt diese Funktion nicht.",
            false,
        )
    }

    fn provider_unavailable() -> Self {
        Self::new(
            ImageGenerationErrorCode::ProviderUnavailable,
            "Der Bildprovider ist nicht verfügbar.",
            true,
        )
    }

    fn model_unavailable() -> Self {
        Self::new(
            ImageGenerationErrorCode::ModelUnavailable,
            "Das Bildmodell ist nicht verfügbar.",
            true,
        )
    }

    fn context_not_approved() -> Self {
        Self::new(
            ImageGenerationErrorCode::ContextNotApproved,
            "Der Bildkontext wurde nicht freigegeben.",
            false,
        )
    }

    fn invalid_state() -> Self {
        Self::new(
            ImageGenerationErrorCode::InvalidState,
            "Der Bildauftrag besitzt einen ungültigen Zustand.",
            false,
        )
    }

    /// Datenarmer Ablehnungsfehler für eine unbrauchbare Providerantwort.
    #[must_use]
    pub fn output_rejected() -> Self {
        Self::new(
            ImageGenerationErrorCode::OutputRejected,
            "Das Bildergebnis wurde verworfen.",
            false,
        )
    }

    /// Datenarmer interner Fehler ohne fremde Diagnoseinformationen.
    #[must_use]
    pub fn internal() -> Self {
        Self::new(
            ImageGenerationErrorCode::Internal,
            "Der Bildauftrag konnte nicht verarbeitet werden.",
            true,
        )
    }

    fn new(code: ImageGenerationErrorCode, message: &str, retryable: bool) -> Self {
        Self {
            code,
            message: message.to_owned(),
            retryable,
        }
    }
}

impl fmt::Display for ImageGenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ImageGenerationError {}

/// Spätere Providergrenze; KQ-016 liefert bewusst keine produktive Implementierung.
#[async_trait]
pub trait ImageProvider: Send + Sync {
    /// Beschreibt Modelle und Fähigkeiten ohne Netzwerkzugriff.
    fn descriptor(&self) -> ImageProviderDescriptor;

    /// Führt einen bereits registrierten und freigegebenen Auftrag aus.
    async fn generate(
        &self,
        request: &ImageGenerationRequest,
    ) -> Result<ImageGenerationProvenance, ImageGenerationError>;
}

fn validate_provider(provider: &ImageProviderDescriptor) -> Result<(), ImageGenerationError> {
    if !valid_provider_id(&provider.id)
        || !valid_label(&provider.display_name, 120)
        || provider.models.len() > MAX_MODELS_PER_PROVIDER
        || (provider.available && provider.models.is_empty())
        || (provider.available && provider.unavailable_reason.is_some())
        || (!provider.available
            && provider
                .unavailable_reason
                .as_ref()
                .is_none_or(|reason| !valid_label(reason, 240)))
    {
        return Err(ImageGenerationError::invalid_request());
    }
    let mut model_ids = BTreeSet::new();
    for model in &provider.models {
        if !valid_model_id(&model.id)
            || !valid_label(&model.display_name, 120)
            || model.capabilities.is_empty()
            || !model_ids.insert(model.id.clone())
        {
            return Err(ImageGenerationError::invalid_request());
        }
        let capabilities = model.capabilities.iter().copied().collect::<BTreeSet<_>>();
        if capabilities.len() != model.capabilities.len() {
            return Err(ImageGenerationError::invalid_request());
        }
        validate_limits(&model.limits)?;
    }
    Ok(())
}

fn validate_limits(limits: &ImageModelLimits) -> Result<(), ImageGenerationError> {
    if limits.max_prompt_characters == 0
        || limits.max_prompt_characters > MAX_PROMPT_CHARACTERS
        || limits.max_reference_images > MAX_REFERENCE_IMAGES
        || limits.max_outputs == 0
        || limits.max_outputs > MAX_OUTPUTS
        || limits.supported_widths.is_empty()
        || limits.supported_heights.is_empty()
        || !valid_dimensions(&limits.supported_widths)
        || !valid_dimensions(&limits.supported_heights)
    {
        return Err(ImageGenerationError::invalid_request());
    }
    Ok(())
}

fn validate_request_identity(request: &ImageGenerationRequest) -> Result<(), ImageGenerationError> {
    if request.contract_version != IMAGE_GENERATION_CONTRACT_VERSION
        || !valid_provider_id(&request.provider_id)
        || !valid_model_id(&request.model_id)
        || request.context.reference_asset_ids.len() > MAX_REFERENCE_IMAGES as usize
        || request
            .context
            .reference_asset_ids
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != request.context.reference_asset_ids.len()
    {
        return Err(ImageGenerationError::invalid_request());
    }
    Ok(())
}

fn validate_settings(
    settings: &ImageGenerationSettings,
    limits: &ImageModelLimits,
) -> Result<(), ImageGenerationError> {
    if !limits.supported_widths.contains(&settings.width)
        || !limits.supported_heights.contains(&settings.height)
        || settings.output_count == 0
        || settings.output_count > limits.max_outputs
        || (settings.seed.is_some() && !limits.supports_seed)
    {
        return Err(ImageGenerationError::invalid_request());
    }
    Ok(())
}

fn valid_dimensions(values: &[u32]) -> bool {
    values
        .iter()
        .all(|value| *value > 0 && *value <= MAX_DIMENSION)
        && values.iter().copied().collect::<BTreeSet<_>>().len() == values.len()
}

fn valid_provider_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_' | b'.')
        })
}

fn valid_model_id(value: &str) -> bool {
    !value.trim().is_empty() && value.chars().count() <= 200 && !value.chars().any(char::is_control)
}

fn valid_label(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.chars().count() <= max && !value.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[derive(Default)]
    struct LocalMockProvider {
        calls: AtomicUsize,
    }

    #[async_trait]
    impl ImageProvider for LocalMockProvider {
        fn descriptor(&self) -> ImageProviderDescriptor {
            mock_descriptor(true)
        }

        async fn generate(
            &self,
            request: &ImageGenerationRequest,
        ) -> Result<ImageGenerationProvenance, ImageGenerationError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            if request.context.prompt_text == "mock:reject" {
                return Err(ImageGenerationError::output_rejected());
            }
            Ok(ImageGenerationProvenance {
                provider_id: request.provider_id.clone(),
                model_id: request.model_id.clone(),
                capability: request.capability,
                seed: request.settings.seed,
                generated_at: "2026-11-10T08:00:00Z".to_owned(),
            })
        }
    }

    #[test]
    fn registry_is_offline_stable_and_model_specific() {
        let registry =
            ImageCapabilityRegistry::new(vec![unavailable_descriptor(), mock_descriptor(true)])
                .expect("registry");

        assert_eq!(registry.providers()[0].id, "cloud-placeholder");
        assert_eq!(registry.providers()[1].id, "local-mock");
        assert!(!registry.providers()[0].available);
        assert_eq!(
            registry.providers()[1].models[0].capabilities,
            vec![ImageCapability::Generate, ImageCapability::Reference]
        );
    }

    #[test]
    fn validates_only_registered_model_capabilities_and_limits() {
        let registry = registry();
        let valid = request();
        registry.validate_request(&valid).expect("valid request");

        let mut unsupported = valid.clone();
        unsupported.capability = ImageCapability::Edit;
        assert_eq!(
            registry
                .validate_request(&unsupported)
                .expect_err("edit must be unsupported")
                .code,
            ImageGenerationErrorCode::UnsupportedCapability
        );

        let mut oversized = valid.clone();
        oversized.settings.width = 2048;
        assert_eq!(
            registry
                .validate_request(&oversized)
                .expect_err("size must be rejected")
                .code,
            ImageGenerationErrorCode::InvalidRequest
        );
    }

    #[tokio::test]
    async fn refuses_unapproved_context_before_provider_execution() {
        let provider = LocalMockProvider::default();
        let registry = ImageCapabilityRegistry::new(vec![provider.descriptor()]).expect("registry");
        let mut unapproved = request();
        unapproved.context.approved = false;

        let error = registry
            .run_with(&provider, &unapproved)
            .await
            .expect_err("release must be explicit");

        assert_eq!(error.code, ImageGenerationErrorCode::ContextNotApproved);
        assert!(!error.message.contains("Kolkrabe"));
        assert_eq!(provider.calls.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn local_mock_exercises_job_lifecycle_without_image_or_network() {
        let provider = LocalMockProvider::default();
        let registry = ImageCapabilityRegistry::new(vec![provider.descriptor()]).expect("registry");
        let request = request();
        let mut job = ImageGenerationJob::queued(&request, "2026-11-10T07:59:00Z");
        job.transition(
            ImageGenerationJobStatus::Running,
            None,
            None,
            "2026-11-10T07:59:01Z",
        )
        .expect("running");

        let provenance = registry
            .run_with(&provider, &request)
            .await
            .expect("mock result");
        job.transition(
            ImageGenerationJobStatus::Succeeded,
            None,
            Some(provenance),
            "2026-11-10T08:00:00Z",
        )
        .expect("succeeded");

        assert_eq!(job.status, ImageGenerationJobStatus::Succeeded);
        assert!(job.provenance.is_some());
        assert!(job.error.is_none());
        assert_eq!(job.prompt_text, "Kolkrabe über der solidarischen Stadt");
        assert_eq!(provider.calls.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn terminal_jobs_cannot_be_reopened_or_misattributed() {
        let request = request();
        let mut job = ImageGenerationJob::queued(&request, "2026-11-10T07:59:00Z");
        job.transition(
            ImageGenerationJobStatus::Cancelled,
            None,
            None,
            "2026-11-10T08:00:00Z",
        )
        .expect("cancelled");
        assert_eq!(
            job.transition(
                ImageGenerationJobStatus::Running,
                None,
                None,
                "2026-11-10T08:01:00Z",
            )
            .expect_err("terminal state")
            .code,
            ImageGenerationErrorCode::InvalidState
        );

        let mut running = ImageGenerationJob::queued(&request, "2026-11-10T07:59:00Z");
        running
            .transition(
                ImageGenerationJobStatus::Running,
                None,
                None,
                "2026-11-10T07:59:01Z",
            )
            .expect("running");
        let mut provenance = provenance();
        provenance.provider_id = "fremd".to_owned();
        assert_eq!(
            running
                .transition(
                    ImageGenerationJobStatus::Succeeded,
                    None,
                    Some(provenance),
                    "2026-11-10T08:00:00Z",
                )
                .expect_err("provenance mismatch")
                .code,
            ImageGenerationErrorCode::InvalidState
        );
    }

    #[test]
    fn wire_contract_has_stable_names_and_sanitized_errors() {
        let request = request();
        let json = serde_json::to_value(&request).expect("request JSON");
        assert_eq!(json["contractVersion"], 1);
        assert_eq!(json["capability"], "image.generate");
        assert_eq!(json["settings"]["outputCount"], 1);
        assert_eq!(
            json["context"]["referenceAssetIds"]
                .as_array()
                .map(Vec::len),
            Some(1)
        );

        let error = ImageGenerationError::internal();
        assert_eq!(
            serde_json::to_value(error).expect("error JSON"),
            serde_json::json!({
                "code": "INTERNAL",
                "message": "Der Bildauftrag konnte nicht verarbeitet werden.",
                "retryable": true
            })
        );
    }

    fn registry() -> ImageCapabilityRegistry {
        ImageCapabilityRegistry::new(vec![mock_descriptor(true)]).expect("registry")
    }

    fn mock_descriptor(available: bool) -> ImageProviderDescriptor {
        ImageProviderDescriptor {
            id: "local-mock".to_owned(),
            display_name: "Lokaler Testprovider".to_owned(),
            location: ProviderLocation::Local,
            available,
            models: vec![ImageModelDescriptor {
                id: "mock-v1".to_owned(),
                display_name: "Mock v1".to_owned(),
                capabilities: vec![ImageCapability::Generate, ImageCapability::Reference],
                limits: ImageModelLimits {
                    max_prompt_characters: 4_000,
                    max_reference_images: 2,
                    supported_widths: vec![512, 1024],
                    supported_heights: vec![512, 1024],
                    max_outputs: 2,
                    supports_seed: true,
                },
            }],
            unavailable_reason: (!available).then(|| "Nicht eingerichtet.".to_owned()),
        }
    }

    fn unavailable_descriptor() -> ImageProviderDescriptor {
        ImageProviderDescriptor {
            id: "cloud-placeholder".to_owned(),
            display_name: "Cloud-Platzhalter".to_owned(),
            location: ProviderLocation::Cloud,
            available: false,
            models: Vec::new(),
            unavailable_reason: Some("Nicht eingerichtet.".to_owned()),
        }
    }

    fn request() -> ImageGenerationRequest {
        ImageGenerationRequest {
            contract_version: IMAGE_GENERATION_CONTRACT_VERSION,
            job_id: Uuid::from_u128(1),
            project_id: Uuid::from_u128(2),
            provider_id: "local-mock".to_owned(),
            model_id: "mock-v1".to_owned(),
            capability: ImageCapability::Generate,
            context: ImageContextRelease {
                prompt_draft_id: Uuid::from_u128(3),
                prompt_text: "Kolkrabe über der solidarischen Stadt".to_owned(),
                reference_asset_ids: vec![Uuid::from_u128(4)],
                approved: true,
            },
            settings: ImageGenerationSettings {
                width: 1024,
                height: 1024,
                output_count: 1,
                seed: Some(42),
            },
        }
    }

    fn provenance() -> ImageGenerationProvenance {
        ImageGenerationProvenance {
            provider_id: "local-mock".to_owned(),
            model_id: "mock-v1".to_owned(),
            capability: ImageCapability::Generate,
            seed: Some(42),
            generated_at: "2026-11-10T08:00:00Z".to_owned(),
        }
    }
}
