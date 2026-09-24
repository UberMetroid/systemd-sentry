//! 1:1 QA tests for shell auto-completion generators.

use sentry_daemon::cli::generate_completions;

#[test]
fn test_generate_bash_completions() {
    let script = generate_completions("bash").expect("Bash completions generation failed");
    assert!(script.contains("complete -F _systemd_sentry systemd-sentry sentry"));
    assert!(script.contains("triage"));
    assert!(script.contains("monitor"));
}

#[test]
fn test_generate_zsh_completions() {
    let script = generate_completions("zsh").expect("Zsh completions generation failed");
    assert!(script.contains("#compdef systemd-sentry sentry"));
    assert!(script.contains("incidents:List recent incident journal slices"));
}

#[test]
fn test_generate_fish_completions() {
    let script = generate_completions("fish").expect("Fish completions generation failed");
    assert!(script.contains("complete -c systemd-sentry"));
    assert!(script.contains("complete -c sentry"));
}

#[test]
fn test_unsupported_shell_returns_error() {
    let err = generate_completions("powershell").expect_err("Should reject unsupported shell");
    assert!(err.contains("Unsupported shell"));
}
