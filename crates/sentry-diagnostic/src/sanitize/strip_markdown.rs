//! Markdown code fence stripping for LLM output.

/// Strips markdown code fences (```json or ```) and surrounding whitespace.
pub fn strip_markdown_fences(input: &str) -> &str {
    let trimmed = input.trim();

    // Strip starting markdown fence
    let without_prefix = if let Some(stripped) = trimmed.strip_prefix("```json") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("```JSON") {
        stripped
    } else if let Some(stripped) = trimmed.strip_prefix("```") {
        stripped
    } else {
        trimmed
    };

    let without_prefix = without_prefix.trim_start();

    // Strip trailing markdown fence
    if let Some(stripped) = without_prefix.strip_suffix("```") {
        stripped.trim()
    } else {
        without_prefix.trim()
    }
}
