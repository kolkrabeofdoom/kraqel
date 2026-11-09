use std::{fmt, str::FromStr, time::Duration};

use async_trait::async_trait;
use futures_util::StreamExt as _;
use reqwest::{
    Client, RequestBuilder, StatusCode, Url,
    header::{AUTHORIZATION, HeaderValue},
    redirect::Policy,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use zeroize::Zeroizing;

use crate::{
    MAX_GENERATED_CHARACTERS, MAX_MODEL_LIST_BYTES, MAX_STREAM_LINE_BYTES, ProviderLocation,
    TextCapability, TextGenerationRequest, TextModel, TextProvider, TextProviderDescriptor,
    TextProviderError, TextStreamEvent, build_prompt, valid_model, validate_request,
};

const CLOUD_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const CLOUD_REQUEST_TIMEOUT: Duration = Duration::from_secs(180);
const MAX_OUTPUT_TOKENS: u32 = 4_096;

/// Unterstützte Cloud-Textprovider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CloudProviderId {
    /// `OpenAI` Responses API.
    Openai,
    /// Anthropic Messages API.
    Anthropic,
    /// Google Gemini API.
    Gemini,
    /// Mistral Chat Completions API.
    Mistral,
}

impl CloudProviderId {
    /// Alle Cloudprovider in stabiler UI-Reihenfolge.
    pub const ALL: [Self; 4] = [Self::Openai, Self::Anthropic, Self::Gemini, Self::Mistral];

    /// Stabile Providerkennung.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::Mistral => "mistral",
        }
    }

    /// Anzeigename.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Openai => "OpenAI",
            Self::Anthropic => "Anthropic Claude",
            Self::Gemini => "Google Gemini",
            Self::Mistral => "Mistral AI",
        }
    }

    /// Öffentlicher Providervertrag ohne Netzwerk- oder Schlüsselzugriff.
    #[must_use]
    pub fn descriptor(self) -> TextProviderDescriptor {
        const CAPABILITIES: &[TextCapability] =
            &[TextCapability::Generate, TextCapability::Analyze];
        TextProviderDescriptor {
            id: self.id(),
            display_name: self.display_name(),
            location: ProviderLocation::Cloud,
            capabilities: CAPABILITIES,
        }
    }

    const fn endpoint(self) -> &'static str {
        match self {
            Self::Openai => "https://api.openai.com/v1/",
            Self::Anthropic => "https://api.anthropic.com/v1/",
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta/",
            Self::Mistral => "https://api.mistral.ai/v1/",
        }
    }

    const fn provenance(self) -> &'static str {
        match self {
            Self::Openai => "KI-Entwurf – OpenAI (Cloud)",
            Self::Anthropic => "KI-Entwurf – Anthropic Claude (Cloud)",
            Self::Gemini => "KI-Entwurf – Google Gemini (Cloud)",
            Self::Mistral => "KI-Entwurf – Mistral AI (Cloud)",
        }
    }
}

impl fmt::Display for CloudProviderId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.id())
    }
}

impl FromStr for CloudProviderId {
    type Err = TextProviderError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "openai" => Ok(Self::Openai),
            "anthropic" => Ok(Self::Anthropic),
            "gemini" => Ok(Self::Gemini),
            "mistral" => Ok(Self::Mistral),
            _ => Err(TextProviderError::InvalidRequest),
        }
    }
}

/// Cloudadapter mit festem Produktionsendpunkt und kurzlebigem Schlüssel im Rust-Prozess.
pub struct CloudTextProvider {
    provider: CloudProviderId,
    client: Client,
    endpoint: Url,
    api_key: Zeroizing<String>,
}

impl CloudTextProvider {
    /// Erzeugt einen Adapter für den unveränderlichen offiziellen HTTPS-Endpunkt.
    ///
    /// # Errors
    ///
    /// Ungültige Schlüssel oder eine nicht initialisierbare TLS-Konfiguration werden abgelehnt.
    pub fn production(
        provider: CloudProviderId,
        api_key: Zeroizing<String>,
    ) -> Result<Self, TextProviderError> {
        if !valid_api_key(&api_key) {
            return Err(TextProviderError::CredentialMissing);
        }
        let endpoint =
            Url::parse(provider.endpoint()).map_err(|_| TextProviderError::InvalidRequest)?;
        if endpoint.scheme() != "https" {
            return Err(TextProviderError::InvalidRequest);
        }
        Self::new(provider, endpoint, api_key)
    }

    fn new(
        provider: CloudProviderId,
        endpoint: Url,
        api_key: Zeroizing<String>,
    ) -> Result<Self, TextProviderError> {
        let client = Client::builder()
            .connect_timeout(CLOUD_CONNECT_TIMEOUT)
            .timeout(CLOUD_REQUEST_TIMEOUT)
            .redirect(Policy::none())
            .no_proxy()
            .build()
            .map_err(|_| TextProviderError::Unavailable)?;
        Ok(Self {
            provider,
            client,
            endpoint,
            api_key,
        })
    }

    #[cfg(test)]
    fn for_test(
        provider: CloudProviderId,
        endpoint: Url,
        api_key: &str,
    ) -> Result<Self, TextProviderError> {
        if endpoint.scheme() != "http"
            || endpoint
                .host_str()
                .and_then(|host| host.parse::<std::net::IpAddr>().ok())
                .is_none_or(|address| !address.is_loopback())
        {
            return Err(TextProviderError::InvalidRequest);
        }
        Self::new(provider, endpoint, Zeroizing::new(api_key.to_owned()))
    }

    fn url(&self, path: &str) -> Result<Url, TextProviderError> {
        self.endpoint
            .join(path)
            .map_err(|_| TextProviderError::InvalidRequest)
    }

    fn authenticated(&self, request: RequestBuilder) -> Result<RequestBuilder, TextProviderError> {
        let mut key = HeaderValue::from_str(&self.api_key)
            .map_err(|_| TextProviderError::CredentialMissing)?;
        key.set_sensitive(true);
        Ok(match self.provider {
            CloudProviderId::Openai | CloudProviderId::Mistral => {
                let mut bearer = HeaderValue::from_str(&format!("Bearer {}", *self.api_key))
                    .map_err(|_| TextProviderError::CredentialMissing)?;
                bearer.set_sensitive(true);
                request.header(AUTHORIZATION, bearer)
            }
            CloudProviderId::Anthropic => request
                .header("x-api-key", key)
                .header("anthropic-version", "2026-10-21"),
            CloudProviderId::Gemini => request.header("x-goog-api-key", key),
        })
    }

    fn stream_url(&self, model: &str) -> Result<Url, TextProviderError> {
        if !valid_cloud_model(model) {
            return Err(TextProviderError::InvalidRequest);
        }
        match self.provider {
            CloudProviderId::Openai => self.url("responses"),
            CloudProviderId::Anthropic => self.url("messages"),
            CloudProviderId::Gemini => {
                self.url(&format!("models/{model}:streamGenerateContent?alt=sse"))
            }
            CloudProviderId::Mistral => self.url("chat/completions"),
        }
    }

    fn request_body(&self, request: &TextGenerationRequest) -> Value {
        let prompt = build_prompt(request);
        match self.provider {
            CloudProviderId::Openai => json!({
                "model": request.model,
                "input": prompt,
                "stream": true,
                "store": false,
                "max_output_tokens": MAX_OUTPUT_TOKENS,
            }),
            CloudProviderId::Anthropic | CloudProviderId::Mistral => json!({
                "model": request.model,
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": MAX_OUTPUT_TOKENS,
                "stream": true,
            }),
            CloudProviderId::Gemini => json!({
                "contents": [{"role": "user", "parts": [{"text": prompt}]}],
                "generationConfig": {"maxOutputTokens": MAX_OUTPUT_TOKENS},
            }),
        }
    }
}

#[async_trait]
impl TextProvider for CloudTextProvider {
    fn descriptor(&self) -> TextProviderDescriptor {
        self.provider.descriptor()
    }

    async fn list_models(&self) -> Result<Vec<TextModel>, TextProviderError> {
        let request = self.authenticated(self.client.get(self.url("models")?))?;
        let response = request
            .send()
            .await
            .map_err(|_| TextProviderError::Unavailable)?;
        check_status(response.status())?;
        let payload = read_limited(response, MAX_MODEL_LIST_BYTES).await?;
        let value: Value =
            serde_json::from_slice(&payload).map_err(|_| TextProviderError::InvalidResponse)?;
        let source = if self.provider == CloudProviderId::Gemini {
            value.get("models")
        } else {
            value.get("data")
        }
        .and_then(Value::as_array)
        .ok_or(TextProviderError::InvalidResponse)?;
        let mut models = source
            .iter()
            .filter(|model| {
                self.provider != CloudProviderId::Gemini
                    || model
                        .get("supportedGenerationMethods")
                        .and_then(Value::as_array)
                        .is_none_or(|methods| methods.iter().any(|item| item == "generateContent"))
            })
            .filter_map(|model| {
                model
                    .get(if self.provider == CloudProviderId::Gemini {
                        "name"
                    } else {
                        "id"
                    })
                    .and_then(Value::as_str)
                    .map(|id| id.strip_prefix("models/").unwrap_or(id))
                    .filter(|id| valid_cloud_model(id))
                    .map(|id| TextModel { id: id.to_owned() })
            })
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
        if !valid_cloud_model(&request.model) {
            return Err(TextProviderError::InvalidRequest);
        }
        let request_builder = self
            .client
            .post(self.stream_url(&request.model)?)
            .json(&self.request_body(request));
        let response = self
            .authenticated(request_builder)?
            .send()
            .await
            .map_err(|_| TextProviderError::Unavailable)?;
        check_status(response.status())?;
        sink(TextStreamEvent::Started {
            provider_id: self.provider.id(),
            model: request.model.clone(),
        })?;

        let mut stream = response.bytes_stream();
        let mut buffer = Vec::new();
        let mut generated_characters = 0_usize;
        let mut completed = false;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|_| TextProviderError::InvalidResponse)?;
            buffer.extend_from_slice(&chunk);
            while let Some(newline) = buffer.iter().position(|byte| *byte == b'\n') {
                let line = buffer.drain(..=newline).collect::<Vec<_>>();
                if line.len() > MAX_STREAM_LINE_BYTES {
                    return Err(TextProviderError::ResponseTooLarge);
                }
                completed |=
                    process_sse_line(self.provider, &line, &mut generated_characters, sink)?;
            }
            if buffer.len() > MAX_STREAM_LINE_BYTES {
                return Err(TextProviderError::ResponseTooLarge);
            }
        }
        if !buffer.is_empty() {
            completed |= process_sse_line(self.provider, &buffer, &mut generated_characters, sink)?;
        }
        if !completed {
            return Err(TextProviderError::InvalidResponse);
        }
        sink(TextStreamEvent::Completed {
            provenance: self.provider.provenance(),
        })
    }
}

fn valid_api_key(api_key: &str) -> bool {
    api_key.chars().count() >= 8
        && api_key.chars().count() <= 4_096
        && !api_key.chars().any(char::is_whitespace)
        && !api_key.chars().any(char::is_control)
}

fn valid_cloud_model(model: &str) -> bool {
    valid_model(model)
        && model
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_.:".contains(character))
}

fn check_status(status: StatusCode) -> Result<(), TextProviderError> {
    if status.is_success() {
        Ok(())
    } else if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        Err(TextProviderError::CredentialRejected)
    } else {
        Err(TextProviderError::Unavailable)
    }
}

async fn read_limited(
    response: reqwest::Response,
    limit: usize,
) -> Result<Vec<u8>, TextProviderError> {
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| TextProviderError::InvalidResponse)?;
        if bytes.len().saturating_add(chunk.len()) > limit {
            return Err(TextProviderError::ResponseTooLarge);
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn process_sse_line(
    provider: CloudProviderId,
    line: &[u8],
    generated_characters: &mut usize,
    sink: &mut (dyn FnMut(TextStreamEvent) -> Result<(), TextProviderError> + Send),
) -> Result<bool, TextProviderError> {
    let line = line.strip_suffix(b"\n").unwrap_or(line);
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    let Some(data) = line.strip_prefix(b"data:") else {
        return Ok(false);
    };
    let data = data.strip_prefix(b" ").unwrap_or(data);
    if data == b"[DONE]" {
        return Ok(provider == CloudProviderId::Mistral);
    }
    let value: Value =
        serde_json::from_slice(data).map_err(|_| TextProviderError::InvalidResponse)?;
    let (delta, completed, failed) = match provider {
        CloudProviderId::Openai => {
            let kind = value
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            (
                (kind == "response.output_text.delta")
                    .then(|| value.get("delta").and_then(Value::as_str))
                    .flatten(),
                kind == "response.completed",
                matches!(kind, "error" | "response.failed" | "response.incomplete"),
            )
        }
        CloudProviderId::Anthropic => {
            let kind = value
                .get("type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let delta = value.get("delta");
            (
                (kind == "content_block_delta"
                    && delta
                        .and_then(|item| item.get("type"))
                        .and_then(Value::as_str)
                        == Some("text_delta"))
                .then(|| {
                    delta
                        .and_then(|item| item.get("text"))
                        .and_then(Value::as_str)
                })
                .flatten(),
                kind == "message_stop",
                kind == "error",
            )
        }
        CloudProviderId::Gemini => {
            let candidate = value
                .get("candidates")
                .and_then(Value::as_array)
                .and_then(|items| items.first());
            let delta = candidate
                .and_then(|item| item.get("content"))
                .and_then(|item| item.get("parts"))
                .and_then(Value::as_array)
                .and_then(|parts| parts.iter().find_map(|part| part.get("text")))
                .and_then(Value::as_str);
            (
                delta,
                candidate
                    .and_then(|item| item.get("finishReason"))
                    .is_some(),
                value.get("error").is_some(),
            )
        }
        CloudProviderId::Mistral => {
            let choice = value
                .get("choices")
                .and_then(Value::as_array)
                .and_then(|items| items.first());
            (
                choice
                    .and_then(|item| item.get("delta"))
                    .and_then(|item| item.get("content"))
                    .and_then(Value::as_str),
                choice
                    .and_then(|item| item.get("finish_reason"))
                    .is_some_and(|reason| !reason.is_null()),
                value.get("error").is_some(),
            )
        }
    };
    if failed {
        return Err(TextProviderError::InvalidResponse);
    }
    if let Some(delta) = delta.filter(|delta| !delta.is_empty()) {
        *generated_characters = generated_characters.saturating_add(delta.chars().count());
        if *generated_characters > MAX_GENERATED_CHARACTERS {
            return Err(TextProviderError::ResponseTooLarge);
        }
        sink(TextStreamEvent::Delta {
            text: delta.to_owned(),
        })?;
    }
    Ok(completed)
}

#[cfg(test)]
mod tests {
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::{Arc, Mutex},
    };

    use tokio::{
        io::{AsyncReadExt as _, AsyncWriteExt as _},
        net::TcpListener,
    };

    use super::*;
    use crate::ReleasedTextContext;

    #[tokio::test]
    async fn streams_each_cloud_protocol_with_provider_specific_authentication() {
        let cases = [
            (
                CloudProviderId::Openai,
                concat!(
                    "data: {\"type\":\"response.output_text.delta\",\"delta\":\"Open\"}\n\n",
                    "data: {\"type\":\"response.completed\"}\n\n"
                ),
            ),
            (
                CloudProviderId::Anthropic,
                concat!(
                    "event: content_block_delta\n",
                    "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Claude\"}}\n\n",
                    "event: message_stop\n",
                    "data: {\"type\":\"message_stop\"}\n\n"
                ),
            ),
            (
                CloudProviderId::Gemini,
                concat!(
                    "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Gemini\"}]}}]}\n\n",
                    "data: {\"candidates\":[{\"finishReason\":\"STOP\"}]}\n\n"
                ),
            ),
            (
                CloudProviderId::Mistral,
                concat!(
                    "data: {\"choices\":[{\"delta\":{\"content\":\"Mistral\"},\"finish_reason\":null}]}\n\n",
                    "data: [DONE]\n\n"
                ),
            ),
        ];

        for (provider_id, body) in cases {
            let (endpoint, captured) = server_once(body).await;
            let provider = CloudTextProvider::for_test(provider_id, endpoint, "secret-test-key")
                .expect("test provider");
            let mut events = Vec::new();
            provider
                .stream_draft(&approved_request(), &mut |event| {
                    events.push(event);
                    Ok(())
                })
                .await
                .unwrap_or_else(|error| panic!("{provider_id}: {error:?}"));

            assert!(matches!(
                events.first(),
                Some(TextStreamEvent::Started { .. })
            ));
            assert!(matches!(
                events.last(),
                Some(TextStreamEvent::Completed { .. })
            ));
            let request = captured.lock().expect("request");
            assert!(request.contains("Nur freigegebener Text"));
            assert!(request.contains("Schreibe weiter"));
            let request_lower = request.to_ascii_lowercase();
            match provider_id {
                CloudProviderId::Openai | CloudProviderId::Mistral => {
                    assert!(request_lower.contains("authorization: bearer secret-test-key"));
                }
                CloudProviderId::Anthropic => {
                    assert!(request_lower.contains("x-api-key: secret-test-key"));
                    assert!(request_lower.contains("anthropic-version: 2026-10-21"));
                }
                CloudProviderId::Gemini => {
                    assert!(request_lower.contains("x-goog-api-key: secret-test-key"));
                }
            }
        }
    }

    #[tokio::test]
    async fn refuses_cloud_network_before_context_release() {
        let endpoint = Url::parse("http://127.0.0.1:9/").expect("URL");
        let provider =
            CloudTextProvider::for_test(CloudProviderId::Openai, endpoint, "secret-test-key")
                .expect("provider");
        let mut request = approved_request();
        request.context.approved = false;

        let error = provider
            .stream_draft(&request, &mut |_| Ok(()))
            .await
            .expect_err("release required");

        assert!(matches!(error, TextProviderError::ContextNotApproved));
    }

    #[tokio::test]
    async fn lists_models_for_all_provider_response_shapes() {
        let cases = [
            (
                CloudProviderId::Openai,
                r#"{"data":[{"id":"gpt-test"}]}"#,
                "gpt-test",
            ),
            (
                CloudProviderId::Anthropic,
                r#"{"data":[{"id":"claude-test"}]}"#,
                "claude-test",
            ),
            (
                CloudProviderId::Gemini,
                r#"{"models":[{"name":"models/gemini-test","supportedGenerationMethods":["generateContent"]},{"name":"models/embed-only","supportedGenerationMethods":["embedContent"]}]}"#,
                "gemini-test",
            ),
            (
                CloudProviderId::Mistral,
                r#"{"data":[{"id":"mistral-test"}]}"#,
                "mistral-test",
            ),
        ];

        for (provider_id, body, expected) in cases {
            let (endpoint, _) = server_once_with_content_type("application/json", body).await;
            let provider = CloudTextProvider::for_test(provider_id, endpoint, "secret-test-key")
                .expect("test provider");

            let models = provider.list_models().await.expect("models");

            assert_eq!(
                models,
                vec![TextModel {
                    id: expected.to_owned()
                }]
            );
        }
    }

    fn approved_request() -> TextGenerationRequest {
        TextGenerationRequest {
            model: "model-test".to_owned(),
            instruction: "Schreibe weiter".to_owned(),
            context: ReleasedTextContext {
                label: "Szene 1".to_owned(),
                text: "Nur freigegebener Text".to_owned(),
                approved: true,
            },
        }
    }

    async fn server_once(body: &'static str) -> (Url, Arc<Mutex<String>>) {
        server_once_with_content_type("text/event-stream", body).await
    }

    async fn server_once_with_content_type(
        content_type: &'static str,
        body: &'static str,
    ) -> (Url, Arc<Mutex<String>>) {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("listener");
        let address = listener.local_addr().expect("address");
        let captured = Arc::new(Mutex::new(String::new()));
        let for_task = Arc::clone(&captured);
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.expect("connection");
            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];
            loop {
                let read = socket.read(&mut buffer).await.expect("request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
                if request_complete(&request) {
                    break;
                }
            }
            *for_task.lock().expect("captured") = String::from_utf8_lossy(&request).into_owned();
            let headers = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            socket.write_all(headers.as_bytes()).await.expect("headers");
            for chunk in body.as_bytes().chunks(5) {
                socket.write_all(chunk).await.expect("chunk");
            }
            socket.shutdown().await.expect("shutdown");
        });
        (
            Url::parse(&format!(
                "http://{}/",
                SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), address.port())
            ))
            .expect("URL"),
            captured,
        )
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
