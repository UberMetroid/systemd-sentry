//! Diagnostic engine orchestrating LLM providers, circuit breaking, and fallback triage.

use crate::circuit::{AdaptiveTimeoutConfig, ProviderAction, ProviderBreaker};
use crate::fallback::DeterministicFallbackEngine;
use crate::provider::LlmProvider;
use crate::sanitize::{DiagnosticSanitizer, SanitizationPipeline};
use crate::schema::DiagnosticPrompt;
use sentry_core::models::{DiagnosticPayload, IncidentContext};
use std::sync::{Arc, Mutex};

/// Diagnostic engine coordinating LLM inference with adaptive circuit breaking and deterministic fallback.
#[derive(Clone)]
pub struct DiagnosticEngine {
    provider: Option<Arc<dyn LlmProvider>>,
    sanitizer: SanitizationPipeline,
    fallback: DeterministicFallbackEngine,
    circuit: Arc<Mutex<ProviderBreaker>>,
}

impl DiagnosticEngine {
    /// Creates a new diagnostic engine with default adaptive circuit breaking.
    pub fn new(provider: Option<Arc<dyn LlmProvider>>) -> Self {
        Self::with_config(provider, AdaptiveTimeoutConfig::default())
    }

    /// Creates a new diagnostic engine with explicit adaptive timeout configuration.
    pub fn with_config(provider: Option<Arc<dyn LlmProvider>>, config: AdaptiveTimeoutConfig) -> Self {
        Self {
            provider,
            sanitizer: SanitizationPipeline::new(),
            fallback: DeterministicFallbackEngine::new(),
            circuit: Arc::new(Mutex::new(ProviderBreaker::new(config))),
        }
    }

    /// Constructs an engine with explicit circuit breaker configuration and components.
    pub fn with_circuit(
        provider: Option<Arc<dyn LlmProvider>>,
        fallback: DeterministicFallbackEngine,
        sanitizer: DiagnosticSanitizer,
        config: AdaptiveTimeoutConfig,
    ) -> Self {
        Self {
            provider,
            sanitizer,
            fallback,
            circuit: Arc::new(Mutex::new(ProviderBreaker::new(config))),
        }
    }

    /// Sets or replaces the LLM provider while preserving circuit state.
    pub fn with_provider(mut self, provider: Arc<dyn LlmProvider>) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Returns reference to fallback engine.
    pub fn fallback_engine(&self) -> &DeterministicFallbackEngine {
        &self.fallback
    }

    /// Returns reference to sanitization pipeline.
    pub fn sanitization_pipeline(&self) -> &SanitizationPipeline {
        &self.sanitizer
    }

    /// Returns handle to the provider circuit breaker.
    pub fn circuit(&self) -> Arc<Mutex<ProviderBreaker>> {
        Arc::clone(&self.circuit)
    }

    /// Diagnoses an incident by attempting LLM inference, then falling back to deterministic triage.
    ///
    /// Never panics in runtime paths; guarantees returning a valid `DiagnosticPayload`.
    pub async fn diagnose(&self, ctx: &IncidentContext) -> DiagnosticPayload {
        if let Some(provider) = &self.provider {
            let action = {
                let mut breaker = self.circuit.lock().unwrap_or_else(|e| e.into_inner());
                breaker.before_request()
            };

            match action {
                ProviderAction::ShortCircuit => {
                    tracing::warn!(
                        "Circuit breaker open for provider {}; short-circuiting to deterministic fallback for unit {}.",
                        provider.id(),
                        ctx.unit
                    );
                    return self.fallback.triage(ctx);
                }
                ProviderAction::Proceed { timeout } => {
                    let prompt = DiagnosticPrompt::from_incident_context(ctx);

                    match tokio::time::timeout(timeout, provider.complete(&prompt)).await {
                        Ok(Ok(raw_resp)) => {
                            {
                                let mut breaker =
                                    self.circuit.lock().unwrap_or_else(|e| e.into_inner());
                                breaker.on_success(raw_resp.latency);
                            }

                            match self.sanitizer.process(&raw_resp.raw_text) {
                                Ok(mut payload) => {
                                    // Ground truth preservation: override hallucinated unit and incident ID
                                    payload.unit_name = ctx.unit.clone();
                                    payload.incident_id = ctx.incident_id;
                                    return payload;
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Sanitization of LLM output failed for unit {}: {}. Triggering fallback triage.",
                                        ctx.unit,
                                        e
                                    );
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            tracing::warn!(
                                "LLM provider {} completion failed for unit {}: {}. Triggering fallback triage.",
                                provider.id(),
                                ctx.unit,
                                e
                            );
                            {
                                let mut breaker =
                                    self.circuit.lock().unwrap_or_else(|e| e.into_inner());
                                breaker.on_error();
                            }
                        }
                        Err(_elapsed) => {
                            tracing::warn!(
                                "LLM provider {} completion timed out after {:?} for unit {}. Cleanly aborting inference.",
                                provider.id(),
                                timeout,
                                ctx.unit
                            );
                            {
                                let mut breaker =
                                    self.circuit.lock().unwrap_or_else(|e| e.into_inner());
                                breaker.on_timeout();
                            }
                        }
                    }
                }
            }
        }

        // Deterministic fallback path
        self.fallback.triage(ctx)
    }
}

impl Default for DiagnosticEngine {
    fn default() -> Self {
        Self::new(None)
    }
}
