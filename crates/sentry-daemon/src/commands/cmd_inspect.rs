//! Command: Inspect full telemetry and diagnostic for a specific incident.

use crate::cli::exit_codes::{EX_OK, EX_SOFTWARE, EX_UNAVAILABLE};
use crate::ipc::client::IpcClient;
use crate::ipc::protocol::{IpcRequest, IpcResponse};

/// Execute the `inspect` subcommand.
pub async fn execute_inspect(socket_path: &str, id: &str, json: bool) -> i32 {
    let mut client = match IpcClient::connect(socket_path).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: Unable to connect to daemon at {}: {}", socket_path, e);
            return EX_UNAVAILABLE;
        }
    };

    match client.send_request(&IpcRequest::InspectIncident { id: id.to_string() }).await {
        Ok(IpcResponse::Ok { data }) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&data).unwrap_or_default());
            } else {
                println!("============================================================");
                println!(" INCIDENT INSPECTION: {}", id);
                println!("============================================================");
                if let Some(ctx) = data.get("context") {
                    println!("Unit:       {}", ctx.get("unit_name").and_then(|v| v.as_str()).unwrap_or("-"));
                    println!("Failed At:  {}", ctx.get("failed_at").and_then(|v| v.as_str()).unwrap_or("-"));
                }
                if let Some(diag) = data.get("diagnostic") {
                    println!("Severity:   {}", diag.get("severity").and_then(|v| v.as_str()).unwrap_or("-"));
                    if let Some(rc) = diag.get("root_cause") {
                        println!("Root Cause: {}", rc.get("summary").and_then(|v| v.as_str()).unwrap_or("-"));
                        println!("Details:    {}", rc.get("description").and_then(|v| v.as_str()).unwrap_or("-"));
                    }
                    if let Some(rem) = diag.get("proposed_remediation") {
                        println!("\nProposed Remediation:");
                        println!("  Action:     {}", rem.get("action").and_then(|v| v.as_str()).unwrap_or("-"));
                        println!("  Rationale:  {}", rem.get("rationale").and_then(|v| v.as_str()).unwrap_or("-"));
                    }
                }
                println!("============================================================");
            }
            EX_OK
        }
        Ok(IpcResponse::Error { code, message }) => {
            eprintln!("Error (code {}): {}", code, message);
            EX_SOFTWARE
        }
        _ => EX_UNAVAILABLE,
    }
}
