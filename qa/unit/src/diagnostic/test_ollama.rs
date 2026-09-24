//! Unit tests for Ollama provider client.

use sentry_diagnostic::provider::LlmProvider;
use sentry_diagnostic::provider::OllamaClient;
use std::time::Duration;

#[test]
fn test_ollama_client_construction() {
    let client = OllamaClient::new("http://127.0.0.1:11434", "llama3.2:latest", Duration::from_secs(30));
    assert_eq!(client.base_url(), "http://127.0.0.1:11434");
    assert_eq!(client.model(), "llama3.2:latest");
    assert_eq!(client.timeout(), Duration::from_secs(30));
    assert_eq!(client.id(), "ollama");
}

#[test]
fn test_ollama_base_url_trimming() {
    let client = OllamaClient::new("http://127.0.0.1:11434///", "llama3.2", Duration::from_secs(10));
    assert_eq!(client.base_url(), "http://127.0.0.1:11434");
}

#[test]
fn test_ollama_temperature_setting() {
    let client = OllamaClient::new("http://127.0.0.1:11434", "llama3.2", Duration::from_secs(10))
        .with_temperature(0.5);
    assert_eq!(client.id(), "ollama");
}
