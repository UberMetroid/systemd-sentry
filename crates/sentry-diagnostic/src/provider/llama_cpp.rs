//! Local llama.cpp server inference provider.

use crate::provider::bounded_body::{read_bounded_json, read_bounded_text, MAX_HTTP_RESPONSE_BYTES};
use crate::provider::LlmProvider;
use crate::schema::{diagnostic_payload_json_schema, DiagnosticPrompt, ProviderHealth, RawLlmResponse};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use sentry_core::error::DiagnosticError;
use std::time::{Duration, Instant};

/// Local llama.cpp server compatibility client (`llama-server`).
#[derive(Debug, Clone)]
pub struct LlamaCppClient {
    base_url: String,
    model: String,
    client: Client,
    temperature: f32,
    timeout: Duration,
}

impl LlamaCppClient {
    /// Constructs a new llama.cpp server client.
    pub fn new(base_url: impl Into<String>, model: impl Into<String>, timeout: Duration) -> Self {
        let base = base_url.into().trim_end_matches('/').to_string();
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or_default();

        Self {
            base_url: base,
            model: model.into(),
            client,
            temperature: 0.1,
            timeout,
        }
    }

    /// Sets temperature.
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    /// Returns configured timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Checks server readiness via `GET /health`.
    pub async fn health(&self) -> Result<String, DiagnosticError> {
        let url = format!("{}/health", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| DiagnosticError::ProviderUnavailable(format!("llama.cpp health check failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(DiagnosticError::ProviderUnavailable(format!(
                "llama.cpp health returned status {}",
                resp.status()
            )));
        }

        let resp_json: Value = read_bounded_json(resp, MAX_HTTP_RESPONSE_BYTES).await?;

        Ok(resp_json["status"].as_str().unwrap_or("ok").to_string())
    }

    /// Dispatches prompt to `/v1/chat/completions` with JSON schema enforcement.
    pub async fn chat_completions(&self, prompt: &DiagnosticPrompt) -> Result<RawLlmResponse, DiagnosticError> {
        let start = Instant::now();
        let url = format!("{}/v1/chat/completions", self.base_url);

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": DiagnosticPrompt::SYSTEM_PROMPT },
                { "role": "user", "content": prompt.to_prompt_json() }
            ],
            "response_format": {
                "type": "json_object",
                "schema": diagnostic_payload_json_schema()
            },
            "temperature": self.temperature,
            "stream": false
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| DiagnosticError::ProviderUnavailable(format!("llama.cpp chat completion failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = read_bounded_text(resp, MAX_HTTP_RESPONSE_BYTES)
                .await
                .unwrap_or_default();
            return Err(DiagnosticError::ProviderUnavailable(format!(
                "llama.cpp chat returned status {status}: {text}"
            )));
        }

        let resp_json: Value = read_bounded_json(resp, MAX_HTTP_RESPONSE_BYTES).await?;

        let raw_text = resp_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let prompt_tokens = resp_json["usage"]["prompt_tokens"].as_u64().map(|v| v as u32);
        let completion_tokens = resp_json["usage"]["completion_tokens"].as_u64().map(|v| v as u32);

        Ok(RawLlmResponse {
            raw_text,
            model: self.model.clone(),
            prompt_tokens,
            completion_tokens,
            latency: start.elapsed(),
        })
    }

    /// Dispatches prompt to native `/completion` with GBNF JSON grammar.
    pub async fn native_completion(&self, prompt: &DiagnosticPrompt) -> Result<RawLlmResponse, DiagnosticError> {
        let start = Instant::now();
        let url = format!("{}/completion", self.base_url);

        let body = json!({
            "prompt": format!("{}\n\n{}", DiagnosticPrompt::SYSTEM_PROMPT, prompt.to_prompt_json()),
            "temperature": self.temperature,
            "n_predict": 2048,
            "stream": false,
            "json_schema": diagnostic_payload_json_schema()
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| DiagnosticError::ProviderUnavailable(format!("llama.cpp native completion failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = read_bounded_text(resp, MAX_HTTP_RESPONSE_BYTES)
                .await
                .unwrap_or_default();
            return Err(DiagnosticError::ProviderUnavailable(format!(
                "llama.cpp native completion returned status {status}: {text}"
            )));
        }

        let resp_json: Value = read_bounded_json(resp, MAX_HTTP_RESPONSE_BYTES).await?;

        let raw_text = resp_json["content"].as_str().unwrap_or("").to_string();

        Ok(RawLlmResponse {
            raw_text,
            model: self.model.clone(),
            prompt_tokens: None,
            completion_tokens: None,
            latency: start.elapsed(),
        })
    }
}

#[async_trait]
impl LlmProvider for LlamaCppClient {
    async fn ping(&self) -> Result<ProviderHealth, DiagnosticError> {
        let start = Instant::now();
        let status = self.health().await?;
        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(ProviderHealth {
            available: true,
            provider_name: "llama.cpp".to_string(),
            model_name: self.model.clone(),
            latency_ms,
            details: Some(format!("health status: {status}")),
        })
    }

    async fn complete(&self, prompt: &DiagnosticPrompt) -> Result<RawLlmResponse, DiagnosticError> {
        self.chat_completions(prompt).await
    }

    fn id(&self) -> &'static str {
        "llama.cpp"
    }
}
