//! Unit tests for MCP prompts registry.

use sentry_mcp::prompts::list_prompts;

#[test]
fn test_list_prompts_contains_required_templates() {
    let prompts = list_prompts();
    let arr = prompts.as_array().expect("Prompts is array");

    let names: Vec<&str> = arr.iter().filter_map(|p| p["name"].as_str()).collect();
    assert!(names.contains(&"triage_incident"));
    assert!(names.contains(&"analyze_flapping_service"));
    assert_eq!(names.len(), 2);
}
