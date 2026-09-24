//! Local Ollama LLM provider implementation.

use crate::provider::bounded_body::{read_bounded_json, read_bounded_text, MAX_HTTP_RESPONSE_BYTES};
use crate::provider::LlmProvider;
use crate::schema::{DiagnosticPrompt, ProviderHealth, RawLlmResponse};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};
use sentry_core::error::DiagnosticError;
use std::time::{Duration, Instant};

/// Local Ollama inference provider communicating over pure Rust HTTP.
#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    model: String,
    client: Client,
    temperature: f32,
    timeout: Duration,
}

impl OllamaClient {
    /// Constructs a new Ollama client.
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

    /// Sets inference temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    /// Returns the target model name.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Returns base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns configured timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Dispatches prompt to `/api/chat` with structured JSON format enforcement.
    pub async fn chat(&self, prompt: &DiagnosticPrompt) -> Result<RawLlmResponse, DiagnosticError> {
        let start = Instant::now();
        let url = format!("{}/api/chat", self.base_url);

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": DiagnosticPrompt::SYSTEM_PROMPT },
                { "role": "user", "content": prompt.to_prompt_json() }
            ],
            "stream": false,
            "format": "json",
            "options": {
                "temperature": self.temperature,
                "num_predict": 2048
            }
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| DiagnosticError::ProviderUnavailable(format!("Ollama chat request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = read_bounded_text(resp, MAX_HTTP_RESPONSE_BYTES)
                .await
                .unwrap_or_default();
            return Err(DiagnosticError::ProviderUnavailable(format!(
                "Ollama chat error status {status}: {text}"
            )));
        }

        let resp_json: Value = read_bounded_json(resp, MAX_HTTP_RESPONSE_BYTES).await?;

        let raw_text = resp_json["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let prompt_tokens = resp_json["prompt_eval_count"].as_u64().map(|v| v as u32);
        let completion_tokens = resp_json["eval_count"].as_u64().map(|v| v as u32);

        Ok(RawLlmResponse {
            raw_text,
            model: self.model.clone(),
            prompt_tokens,
            completion_tokens,
            latency: start.elapsed(),
        })
    }

    /// Dispatches prompt to `/api/generate`.
    pub async fn generate(&self, prompt: &DiagnosticPrompt) -> Result<RawLlmResponse, DiagnosticError> {
        let start = Instant::now();
        let url = format!("{}/api/generate", self.base_url);

        let body = json!({
            "model": self.model,
            "prompt": prompt.to_prompt_json(),
            "system": DiagnosticPrompt::SYSTEM_PROMPT,
            "stream": false,
            "format": "json",
            "options": {
                "temperature": self.temperature,
                "num_predict": 2048
            }
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| DiagnosticError::ProviderUnavailable(format!("Ollama generate request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = read_bounded_text(resp, MAX_HTTP_RESPONSE_BYTES)
                .await
                .unwrap_or_default();
            return Err(DiagnosticError::ProviderUnavailable(format!(
                "Ollama generate error status {status}: {text}"
            )));
        }

        let resp_json: Value = read_bounded_json(resp, MAX_HTTP_RESPONSE_BYTES).await?;

        let raw_text = resp_json["response"].as_str().unwrap_or("").to_string();
        let prompt_tokens = resp_json["prompt_eval_count"].as_u64().map(|v| v as u32);
        let completion_tokens = resp_json["eval_count"].as_u64().map(|v| v as u32);

        Ok(RawLlmResponse {
            raw_text,
            model: self.model.clone(),
            prompt_tokens,
            completion_tokens,
            latency: start.elapsed(),
        })
    }

    /// Queries installed models via `GET /api/tags`.
    pub async fn tags(&self) -> Result<Vec<String>, DiagnosticError> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| DiagnosticError::ProviderUnavailable(format!("Ollama tags query failed: {e}")))?;

        let resp_json: Value = read_bounded_json(resp, MAX_HTTP_RESPONSE_BYTES).await?;

        let mut models = Vec::new();
        if let Some(arr) = resp_json["models"].as_array() {
            for item in arr {
                if let Some(name) = item["name"].as_str() {
                    models.push(name.to_string());
                }
            }
        }

        Ok(models)
    }

    /// Queries daemon version via fast `GET /api/version`.
    pub async fn version(&self) -> Result<String, DiagnosticError> {
        let url = format!("{}/api/version", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| DiagnosticError::ProviderUnavailable(format!("Ollama version query failed: {e}")))?;

        let resp_json: Value = read_bounded_json(resp, MAX_HTTP_RESPONSE_BYTES).await?;

        Ok(resp_json["version"].as_str().unwrap_or("unknown").to_string())
    }
}

#[async_trait]
impl LlmProvider for OllamaClient {
    async fn ping(&self) -> Result<ProviderHealth, DiagnosticError> {
        let start = Instant::now();
        let ver = self.version().await?;
        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(ProviderHealth {
            available: true,
            provider_name: "ollama".to_string(),
            model_name: self.model.clone(),
            latency_ms,
            details: Some(format!("version: {ver}")),
        })
    }

    async fn complete(&self, prompt: &DiagnosticPrompt) -> Result<RawLlmResponse, DiagnosticError> {
        self.chat(prompt).await
    }

    fn id(&self) -> &'static str {
        "ollama"
    }
}
