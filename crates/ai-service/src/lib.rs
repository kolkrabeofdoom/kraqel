//! Providerunabhängige Text-KI-Grenze für lokale und explizit freigegebene Cloudläufe.

mod cloud;
mod credentials;

pub use cloud::{CloudProviderId, CloudTextProvider};
pub use credentials::{CredentialError, CredentialStore, OsCredentialStore};

use std::{
    error::Error,
    fmt,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use async_trait::async_trait;
use futures_util::StreamExt as _;
use reqwest::{Client, StatusCode, Url, redirect::Policy};
use serde::{Deserialize, Serialize};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const MAX_INSTRUCTION_CHARACTERS: usize = 4_000;
const MAX_CONTEXT_CHARACTERS: usize = 100_000;
const MAX_MODEL_CHARACTERS: usize = 200;
const MAX_MODEL_LIST_BYTES: usize = 1024 * 1024;
const MAX_STREAM_LINE_BYTES: usize = 256 * 1024;
const MAX_GENERATED_CHARACTERS: usize = 500_000;

/// Stabile Kennung des lokalen Ollama-Providers.
pub const OLLAMA_PROVIDER_ID: &str = "ollama";

/// Von einem Textprovider angebotene Funktion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TextCapability {
    /// Neuen Text formulieren.
    #[serde(rename = "text.generate")]
    Generate,
    /// Vorhandenen Text analysieren oder überarbeiten.
    #[serde(rename = "text.analyze")]
    Analyze,
}

/// Ausführungsort eines Providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderLocation {
    /// Provider läuft auf demselben Gerät.
    Local,
    /// Provider verarbeitet freigegebene Daten außerhalb des Geräts.
    Cloud,
}

/// Datenarmer Providervertrag für UI und Serviceauswahl.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextProviderDescriptor {
    /// Stabile Providerkennung.
    pub id: &'static str,
    /// Anzeigename.
    pub display_name: &'static str,
    /// Provider läuft ausschließlich lokal.
    pub location: ProviderLocation,
    /// Tatsächlich angebotene Fähigkeiten.
    pub capabilities: &'static [TextCapability],
}

/// Ein lokal installiertes Textmodell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextModel {
    /// Von Ollama gelieferte Modellkennung.
    pub id: String,
}

/// Exakt der Text, dessen Übertragung in der Vorschau freigegeben wurde.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleasedTextContext {
    /// Nutzer*innengeeignete Bezeichnung der Quelle.
    pub label: String,
    /// Freigegebener Klartext.
    pub text: String,
    /// Explizite Freigabe unmittelbar vor dem Lauf.
    pub approved: bool,
}

/// Providerunabhängige Anfrage für einen verwerfbaren Textentwurf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextGenerationRequest {
    /// Ausgewähltes lokales Modell.
    pub model: String,
    /// Arbeitsauftrag der schreibenden Person.
    pub instruction: String,
    /// Zuvor angezeigter und explizit freigegebener Kontext.
    pub context: ReleasedTextContext,
}

/// Geordnete Ereignisse eines Textstreams.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TextStreamEvent {
    /// Provider hat die Anfrage angenommen.
    Started {
        /// Providerkennung.
        provider_id: &'static str,
        /// Verwendetes Modell.
        model: String,
    },
    /// Ein neues Textfragment.
    Delta {
        /// An den bisherigen Entwurf anzuhängender Text.
        text: String,
    },
    /// Stream wurde regulär beendet; der Text bleibt ein Entwurf.
    Completed {
        /// Herkunftshinweis für die Darstellung.
        provenance: &'static str,
    },
}

/// Fachlich unterscheidbare, datenarme Providerfehler.
#[derive(Debug)]
pub enum TextProviderError {
    /// Kontext wurde nicht explizit freigegeben.
    ContextNotApproved,
    /// Anfrage überschreitet Grenzen oder enthält ungültige Werte.
    InvalidRequest,
    /// Lokaler Provider ist nicht erreichbar.
    Unavailable,
    /// Providerantwort verletzt den erwarteten Vertrag.
    InvalidResponse,
    /// Antwort überschreitet ein Größenlimit.
    ResponseTooLarge,
    /// Empfänger*in des Streams ist nicht mehr erreichbar.
    StreamClosed,
    /// Für den gewählten Cloudprovider fehlt ein Schlüssel.
    CredentialMissing,
    /// Der Cloudprovider hat den gespeicherten Schlüssel abgelehnt.
    CredentialRejected,
}

impl fmt::Display for TextProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContextNotApproved => formatter.write_str("Der Kontext wurde nicht freigegeben."),
            Self::InvalidRequest => formatter.write_str("Die Textanfrage ist ungültig."),
            Self::Unavailable => formatter.write_str("Der Textprovider ist nicht erreichbar."),
            Self::InvalidResponse => {
                formatter.write_str("Der Textprovider hat ungültig geantwortet.")
            }
            Self::ResponseTooLarge => formatter.write_str("Die Providerantwort ist zu groß."),
            Self::StreamClosed => formatter.write_str("Der Textstream wurde geschlossen."),
            Self::CredentialMissing => {
                formatter.write_str("Für den Cloudprovider fehlt ein API-Schlüssel.")
            }
            Self::CredentialRejected => {
                formatter.write_str("Der Cloudprovider hat den API-Schlüssel abgelehnt.")
            }
        }
    }
}

impl Error for TextProviderError {}

/// Asynchrone, providerunabhängige Grenze für lokale oder spätere freigegebene Textprovider.
#[async_trait]
pub trait TextProvider: Send + Sync {
    /// Beschreibt Identität und Fähigkeiten.
    fn descriptor(&self) -> TextProviderDescriptor;

    /// Listet aktuell verfügbare Modelle.
    async fn list_models(&self) -> Result<Vec<TextModel>, TextProviderError>;

    /// Erzeugt einen geordneten Stream, ohne kanonischen Text zu verändern.
    async fn stream_draft(
        &self,
        request: &TextGenerationRequest,
        sink: &mut (dyn FnMut(TextStreamEvent) -> Result<(), TextProviderError> + Send),
    ) -> Result<(), TextProviderError>;
}

/// Ollama-Adapter, dessen Ziel bereits beim Bau auf eine Loopback-Adresse begrenzt wird.
pub struct OllamaProvider {
    client: Client,
    endpoint: Url,
}

impl OllamaProvider {
    /// Bindet den Adapter an die übliche lokale Ollama-Adresse.
    ///
    /// # Errors
    ///
    /// Liefert einen Fehler, wenn der HTTP-Client nicht sicher konfiguriert werden kann.
    pub fn local_default() -> Result<Self, TextProviderError> {
        Self::on_loopback(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 11_434))
    }

    /// Bindet den Adapter an einen expliziten Loopback-Port, etwa für lokale Installationen und Tests.
    ///
    /// # Errors
    ///
    /// Nicht-Loopback-Adressen sowie ungültige Clientkonfigurationen werden abgelehnt.
    pub fn on_loopback(address: SocketAddr) -> Result<Self, TextProviderError> {
        if !address.ip().is_loopback() {
            return Err(TextProviderError::InvalidRequest);
        }
        let endpoint = Url::parse(&format!("http://{address}/"))
            .map_err(|_| TextProviderError::InvalidRequest)?;
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .redirect(Policy::none())
            .no_proxy()
            .build()
            .map_err(|_| TextProviderError::Unavailable)?;
        Ok(Self { client, endpoint })
    }

    fn url(&self, path: &str) -> Result<Url, TextProviderError> {
        self.endpoint
            .join(path)
            .map_err(|_| TextProviderError::InvalidRequest)
    }
}

#[async_trait]
impl TextProvider for OllamaProvider {
    fn descriptor(&self) -> TextProviderDescriptor {
        const CAPABILITIES: &[TextCapability] =
            &[TextCapability::Generate, TextCapability::Analyze];
        TextProviderDescriptor {
            id: OLLAMA_PROVIDER_ID,
            display_name: "Ollama (lokal)",
            location: ProviderLocation::Local,
            capabilities: CAPABILITIES,
        }
    }

    async fn list_models(&self) -> Result<Vec<TextModel>, TextProviderError> {
        let response = self
            .client
            .get(self.url("api/tags")?)
            .send()
            .await
            .map_err(|_| TextProviderError::Unavailable)?;
        if response.status() != StatusCode::OK {
            return Err(TextProviderError::Unavailable);
        }
        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| TextProviderError::InvalidResponse)?;
            if bytes.len().saturating_add(chunk.len()) > MAX_MODEL_LIST_BYTES {
                return Err(TextProviderError::ResponseTooLarge);
            }
            bytes.extend_from_slice(&chunk);
        }
        let payload: OllamaTagsResponse =
            serde_json::from_slice(&bytes).map_err(|_| TextProviderError::InvalidResponse)?;
        let mut models = payload
            .models
            .into_iter()
            .filter(|model| valid_model(&model.name))
            .map(|model| TextModel { id: model.name })
            .collect::<Vec<_>>();
        models.sort_by_key(|model| model.id.to_lowercase());
        models.dedup_by(|left, right| left.id == right.id);
        Ok(models)
    }

    async fn stream_draft(
        &self,
        request: &TextGenerationRequest,
        sink: &mut (dyn FnMut(TextStreamEvent) -> Result<(), TextProviderError> + Send),
    ) -> Result<(), TextProviderError> {
        validate_request(request)?;
        let response = self
            .client
            .post(self.url("api/generate")?)
            .json(&OllamaGenerateRequest {
                model: &request.model,
                prompt: build_prompt(request),
                stream: true,
            })
            .send()
            .await
            .map_err(|_| TextProviderError::Unavailable)?;
        if response.status() != StatusCode::OK {
            return Err(TextProviderError::Unavailable);
        }
        sink(TextStreamEvent::Started {
            provider_id: OLLAMA_PROVIDER_ID,
            model: request.model.clone(),
        })?;

        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();
        let mut generated_characters = 0;
        let mut completed = false;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| TextProviderError::InvalidResponse)?;
            buffer.extend_from_slice(&chunk);
            while let Some(newline) = buffer.iter().position(|byte| *byte == b'\n') {
                let line = buffer.drain(..=newline).collect::<Vec<_>>();
                if line.len() > MAX_STREAM_LINE_BYTES {
                    return Err(TextProviderError::ResponseTooLarge);
                }
                completed |= process_line(&line, &mut generated_characters, sink)?;
            }
            if buffer.len() > MAX_STREAM_LINE_BYTES {
                return Err(TextProviderError::ResponseTooLarge);
            }
        }
        if !buffer.is_empty() {
            completed |= process_line(&buffer, &mut generated_characters, sink)?;
        }
        if !completed {
            return Err(TextProviderError::InvalidResponse);
        }
        sink(TextStreamEvent::Completed {
            provenance: "KI-Entwurf – Ollama (lokal)",
        })
    }
}

pub(crate) fn validate_request(request: &TextGenerationRequest) -> Result<(), TextProviderError> {
    if !request.context.approved {
        return Err(TextProviderError::ContextNotApproved);
    }
    if !valid_model(&request.model)
        || request.instruction.trim().is_empty()
        || request.instruction.chars().count() > MAX_INSTRUCTION_CHARACTERS
        || request.context.text.chars().count() > MAX_CONTEXT_CHARACTERS
        || request.context.label.trim().is_empty()
    {
        return Err(TextProviderError::InvalidRequest);
    }
    Ok(())
}

pub(crate) fn valid_model(model: &str) -> bool {
    !model.trim().is_empty()
        && model.chars().count() <= MAX_MODEL_CHARACTERS
        && !model.chars().any(char::is_control)
}

pub(crate) fn build_prompt(request: &TextGenerationRequest) -> String {
    format!(
        "Arbeitsauftrag:\n{}\n\nFreigegebener Kontext – {}:\n{}",
        request.instruction.trim(),
        request.context.label.trim(),
        request.context.text
    )
}

fn process_line(
    line: &[u8],
    generated_characters: &mut usize,
    sink: &mut (dyn FnMut(TextStreamEvent) -> Result<(), TextProviderError> + Send),
) -> Result<bool, TextProviderError> {
    let line = line
        .strip_suffix(b"\n")
        .unwrap_or(line)
        .strip_suffix(b"\r")
        .unwrap_or(line);
    if line.is_empty() {
        return Ok(false);
    }
    let event: OllamaGenerateResponse =
        serde_json::from_slice(line).map_err(|_| TextProviderError::InvalidResponse)?;
    if event.error.is_some() {
        return Err(TextProviderError::InvalidResponse);
    }
    if !event.response.is_empty() {
        *generated_characters = generated_characters.saturating_add(event.response.chars().count());
        if *generated_characters > MAX_GENERATED_CHARACTERS {
            return Err(TextProviderError::ResponseTooLarge);
        }
        sink(TextStreamEvent::Delta {
            text: event.response,
        })?;
    }
    Ok(event.done)
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

#[derive(Serialize)]
struct OllamaGenerateRequest<'request> {
    model: &'request str,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    #[serde(default)]
    response: String,
    #[serde(default)]
    done: bool,
    #[serde(default)]
    error: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tokio::{
        io::{AsyncReadExt as _, AsyncWriteExt as _},
        net::TcpListener,
    };

    use super::*;

    #[tokio::test]
    async fn lists_models_stably_from_loopback() {
        let (address, request) = server_once(
            "application/json",
            r#"{"models":[{"name":"zeta:latest"},{"name":"Alpha:7b"},{"name":"Alpha:7b"}]}"#,
        )
        .await;
        let provider = OllamaProvider::on_loopback(address).expect("loopback provider");

        let models = provider.list_models().await.expect("models");

        assert_eq!(
            models,
            vec![
                TextModel {
                    id: "Alpha:7b".to_owned(),
                },
                TextModel {
                    id: "zeta:latest".to_owned(),
                },
            ]
        );
        assert!(
            request
                .lock()
                .expect("request")
                .starts_with("GET /api/tags ")
        );
    }

    #[tokio::test]
    async fn streams_ordered_draft_and_sends_only_released_context() {
        let body = concat!(
            "{\"response\":\"Hallo \u{1f426}\",\"done\":false}\n",
            "{\"response\":\"Kolkrabe\",\"done\":false}\n",
            "{\"response\":\"\",\"done\":true}\n"
        );
        let (address, captured) = server_once("application/x-ndjson", body).await;
        let provider = OllamaProvider::on_loopback(address).expect("loopback provider");
        let request = approved_request();
        let mut events = Vec::new();

        provider
            .stream_draft(&request, &mut |event| {
                events.push(event);
                Ok(())
            })
            .await
            .expect("stream");

        assert_eq!(
            events,
            vec![
                TextStreamEvent::Started {
                    provider_id: OLLAMA_PROVIDER_ID,
                    model: "rabe:7b".to_owned(),
                },
                TextStreamEvent::Delta {
                    text: "Hallo 🐦".to_owned(),
                },
                TextStreamEvent::Delta {
                    text: "Kolkrabe".to_owned(),
                },
                TextStreamEvent::Completed {
                    provenance: "KI-Entwurf – Ollama (lokal)",
                },
            ]
        );
        let captured = captured.lock().expect("request");
        assert!(captured.starts_with("POST /api/generate "));
        assert!(captured.contains("Nur dieser Text"));
        assert!(!captured.contains("nicht freigegeben"));
    }

    #[tokio::test]
    async fn rejects_missing_release_before_network_access() {
        let mut request = approved_request();
        request.context.approved = false;
        let provider =
            OllamaProvider::on_loopback(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9))
                .expect("loopback provider");
        let mut called = false;

        let error = provider
            .stream_draft(&request, &mut |_| {
                called = true;
                Ok(())
            })
            .await
            .expect_err("release must be required");

        assert!(matches!(error, TextProviderError::ContextNotApproved));
        assert!(!called);
    }

    #[test]
    fn rejects_non_loopback_endpoints() {
        let result = OllamaProvider::on_loopback(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)),
            11_434,
        ));
        assert!(matches!(result, Err(TextProviderError::InvalidRequest)));
    }

    #[test]
    fn provider_and_stream_contracts_have_stable_wire_names() {
        let provider = OllamaProvider::local_default().expect("provider");
        assert_eq!(
            serde_json::to_value(provider.descriptor()).expect("descriptor JSON"),
            serde_json::json!({
                "id": "ollama",
                "displayName": "Ollama (lokal)",
                "location": "local",
                "capabilities": ["text.generate", "text.analyze"]
            })
        );
        assert_eq!(
            serde_json::to_value(TextStreamEvent::Delta {
                text: "Rabe".to_owned(),
            })
            .expect("event JSON"),
            serde_json::json!({"type": "delta", "text": "Rabe"})
        );
    }

    fn approved_request() -> TextGenerationRequest {
        TextGenerationRequest {
            model: "rabe:7b".to_owned(),
            instruction: "Formuliere vorsichtig weiter.".to_owned(),
            context: ReleasedTextContext {
                label: "Szene 1".to_owned(),
                text: "Nur dieser Text".to_owned(),
                approved: true,
            },
        }
    }

    async fn server_once(
        content_type: &'static str,
        response_body: &'static str,
    ) -> (SocketAddr, Arc<Mutex<String>>) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("test listener");
        let address = listener.local_addr().expect("listener address");
        let captured = Arc::new(Mutex::new(String::new()));
        let captured_for_task = Arc::clone(&captured);
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("test connection");
            let mut request = Vec::new();
            let mut buffer = [0_u8; 512];
            loop {
                let read = socket.read(&mut buffer).await.expect("read request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                if request_complete(&request) {
                    break;
                }
            }
            *captured_for_task.lock().expect("captured request") =
                String::from_utf8(request).expect("UTF-8 request");
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                response_body.len()
            );
            socket.write_all(headers.as_bytes()).await.expect("headers");
            for chunk in response_body.as_bytes().chunks(7) {
                socket.write_all(chunk).await.expect("response chunk");
            }
            socket.shutdown().await.expect("shutdown");
        });
        (address, captured)
    }

    fn request_complete(request: &[u8]) -> bool {
        let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            return false;
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                line.to_ascii_lowercase()
                    .strip_prefix("content-length: ")
                    .and_then(|value| value.parse::<usize>().ok())
            })
            .unwrap_or(0);
        request.len() >= header_end + 4 + content_length
    }
}
