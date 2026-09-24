//! Factory for instantiating LLM providers from configuration.

use crate::provider::llama_cpp::LlamaCppClient;
use crate::provider::ollama::OllamaClient;
use crate::provider::openai::OpenAiClient;
use crate::provider::LlmProvider;
use std::time::Duration;

/// Supported LLM provider types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    /// Local Ollama daemon.
    Ollama,
    /// Local or remote llama.cpp server.
    LlamaCpp,
    /// Cloud OpenAI or OpenAI-compatible endpoint.
    OpenAi,
}

/// Unified provider configuration parameters.
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    /// Provider kind.
    pub kind: ProviderKind,
    /// Base URL (e.g. "http://127.0.0.1:11434" or "https://api.openai.com/v1").
    pub base_url: String,
    /// Model name.
    pub model: String,
    /// Optional API key for cloud providers.
    pub api_key: Option<String>,
    /// Request timeout.
    pub timeout: Duration,
    /// Sampling temperature (default 0.1).
    pub temperature: f32,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            kind: ProviderKind::Ollama,
            base_url: "http://127.0.0.1:11434".to_string(),
            model: "llama3.2:latest".to_string(),
            api_key: None,
            timeout: Duration::from_secs(45),
            temperature: 0.1,
        }
    }
}

/// Constructs a boxed provider matching the given configuration.
pub fn create_provider(config: &ProviderConfig) -> Box<dyn LlmProvider> {
    match config.kind {
        ProviderKind::Ollama => {
            let client = OllamaClient::new(&config.base_url, &config.model, config.timeout)
                .with_temperature(config.temperature);
            Box::new(client)
        }
        ProviderKind::LlamaCpp => {
            let client = LlamaCppClient::new(&config.base_url, &config.model, config.timeout)
                .with_temperature(config.temperature);
            Box::new(client)
        }
        ProviderKind::OpenAi => {
            let client = OpenAiClient::new(
                &config.base_url,
                config.api_key.clone(),
                &config.model,
                config.timeout,
            )
            .with_temperature(config.temperature);
            Box::new(client)
        }
    }
}
