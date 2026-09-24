//! MCP Prompts handling and registry.

pub mod generator;
pub mod registry;

pub use generator::generate_prompt;
pub use registry::list_prompts;
