//! Resilient helper to read bounded HTTP response bodies up to a fixed byte limit.

use sentry_core::error::DiagnosticError;
use serde::de::DeserializeOwned;

/// Default maximum allowable HTTP response size (512 KiB).
pub const MAX_HTTP_RESPONSE_BYTES: usize = 512 * 1024;

/// Reads an HTTP response body up to `max_bytes`, erroring if the limit is exceeded.
pub async fn read_bounded_bytes(
    mut resp: reqwest::Response,
    max_bytes: usize,
) -> Result<Vec<u8>, DiagnosticError> {
    if let Some(cl) = resp.content_length() {
        if cl > max_bytes as u64 {
            return Err(DiagnosticError::ProviderUnavailable(format!(
                "HTTP response Content-Length ({cl} bytes) exceeds limit of {max_bytes} bytes"
            )));
        }
    }

    let mut buf = Vec::new();
    while let Some(chunk) = resp
        .chunk()
        .await
        .map_err(|e| DiagnosticError::ProviderUnavailable(format!("Failed reading response chunk: {e}")))?
    {
        if buf.len() + chunk.len() > max_bytes {
            return Err(DiagnosticError::ProviderUnavailable(format!(
                "HTTP response stream exceeded size limit of {max_bytes} bytes"
            )));
        }
        buf.extend_from_slice(&chunk);
    }

    Ok(buf)
}

/// Reads and deserializes a JSON payload from an HTTP response, enforcing `max_bytes`.
pub async fn read_bounded_json<T: DeserializeOwned>(
    resp: reqwest::Response,
    max_bytes: usize,
) -> Result<T, DiagnosticError> {
    let bytes = read_bounded_bytes(resp, max_bytes).await?;
    serde_json::from_slice(&bytes)
        .map_err(|e| DiagnosticError::MalformedJson(format!("Invalid JSON response: {e}")))
}

/// Reads an HTTP response body as UTF-8 text up to `max_bytes`.
pub async fn read_bounded_text(
    resp: reqwest::Response,
    max_bytes: usize,
) -> Result<String, DiagnosticError> {
    let bytes = read_bounded_bytes(resp, max_bytes).await?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
