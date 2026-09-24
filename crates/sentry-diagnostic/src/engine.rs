//! Diagnostic engine orchestrating LLM providers and deterministic fallback triage.

use crate::fallback::DeterministicFallbackEngine;
use crate::provider::LlmProvider;
use crate::sanitize::SanitizationPipeline;
use crate::schema::DiagnosticPrompt;
use sentry_core::models::{DiagnosticPayload, IncidentContext};
use std::sync::Arc;

/// Diagnostic engine coordinating LLM inference with deterministic fallback.
#[derive(Clone)]
pub struct DiagnosticEngine {
    provider: Option<Arc<dyn LlmProvider>>,
    sanitizer: SanitizationPipeline,
    fallback: DeterministicFallbackEngine,
}

impl DiagnosticEngine {
    /// Creates a new diagnostic engine with an optional LLM provider.
    pub fn new(provider: Option<Arc<dyn LlmProvider>>) -> Self {
        Self {
            provider,
            sanitizer: SanitizationPipeline::new(),
            fallback: DeterministicFallbackEngine::new(),
        }
    }

    /// Sets or replaces the LLM provider.
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

    /// Diagnoses an incident by attempting LLM inference, then falling back to deterministic triage.
    ///
    /// Never panics in runtime paths; guarantees returning a valid `DiagnosticPayload`.
    pub async fn diagnose(&self, ctx: &IncidentContext) -> DiagnosticPayload {
        if let Some(provider) = &self.provider {
            let prompt = DiagnosticPrompt::from_incident_context(ctx);

            match provider.complete(&prompt).await {
                Ok(raw_resp) => {
                    match self.sanitizer.process(&raw_resp.raw_text) {
                        Ok(mut payload) => {
                            // Ensure unit name and incident ID match ground truth
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
                Err(e) => {
                    tracing::warn!(
                        "LLM provider {} completion failed for unit {}: {}. Triggering fallback triage.",
                        provider.id(),
                        ctx.unit,
                        e
                    );
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
