//! Diagnostic domain schemas and prompt definitions.

pub mod json_schema;
pub mod prompt;
pub mod response;

pub use json_schema::{diagnostic_payload_json_schema, openai_response_format};
pub use prompt::DiagnosticPrompt;
pub use response::{ProviderHealth, RawLlmResponse};
