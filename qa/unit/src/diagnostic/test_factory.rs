//! Unit tests for provider factory.

use sentry_diagnostic::provider::{create_provider, ProviderConfig, ProviderKind};
use std::time::Duration;

#[test]
fn test_create_ollama_provider() {
    let config = ProviderConfig {
        kind: ProviderKind::Ollama,
        base_url: "http://127.0.0.1:11434".to_string(),
        model: "llama3.2".to_string(),
        api_key: None,
        timeout: Duration::from_secs(30),
        temperature: 0.1,
        adaptive: Default::default(),
    };

    let provider = create_provider(&config);
    assert_eq!(provider.id(), "ollama");
}

#[test]
fn test_create_llama_cpp_provider() {
    let config = ProviderConfig {
        kind: ProviderKind::LlamaCpp,
        base_url: "http://127.0.0.1:8080".to_string(),
        model: "qwen2.5".to_string(),
        api_key: None,
        timeout: Duration::from_secs(30),
        temperature: 0.1,
        adaptive: Default::default(),
    };

    let provider = create_provider(&config);
    assert_eq!(provider.id(), "llama.cpp");
}

#[test]
fn test_create_openai_provider() {
    let config = ProviderConfig {
        kind: ProviderKind::OpenAi,
        base_url: "https://api.openai.com/v1".to_string(),
        model: "gpt-4o-mini".to_string(),
        api_key: Some("secret".to_string()),
        timeout: Duration::from_secs(30),
        temperature: 0.1,
        adaptive: Default::default(),
    };

    let provider = create_provider(&config);
    assert_eq!(provider.id(), "openai");
}
