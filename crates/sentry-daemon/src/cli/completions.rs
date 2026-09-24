//! Shell auto-completion generators for bash, zsh, and fish.
//!
//! Generates completion definitions for `systemd-sentry` and alias `sentry`.

/// Generate auto-completion script for the target shell.
pub fn generate_completions(shell: &str) -> Result<String, String> {
    match shell.to_lowercase().as_str() {
        "bash" => Ok(generate_bash_completions()),
        "zsh" => Ok(generate_zsh_completions()),
        "fish" => Ok(generate_fish_completions()),
        other => Err(format!(
            "Unsupported shell '{}'. Supported shells: bash, zsh, fish",
            other
        )),
    }
}

fn generate_bash_completions() -> String {
    r#"# bash completion for systemd-sentry and sentry
_systemd_sentry() {
    local cur prev words cword
    _init_completion || return

    local commands="daemon status check triage monitor incidents inspect reset mcp completions setup help"
    local options="-c --config -s --socket --setup -h --help -V --version"

    if [[ $cword -eq 1 ]]; then
        if [[ "$cur" == -* ]]; then
            COMPREPLY=($(compgen -W "$options" -- "$cur"))
        else
            COMPREPLY=($(compgen -W "$commands" -- "$cur"))
        fi
        return 0
    fi

    case "${words[1]}" in
        status)
            COMPREPLY=($(compgen -W "--json" -- "$cur"))
            ;;
        check)
            COMPREPLY=($(compgen -W "--verbose" -- "$cur"))
            ;;
        triage|reset)
            COMPREPLY=($(compgen -W "$(systemctl list-unit-files --no-legend 2>/dev/null | awk '{print $1}')" -- "$cur"))
            ;;
        incidents)
            COMPREPLY=($(compgen -W "--limit --json" -- "$cur"))
            ;;
        inspect)
            COMPREPLY=($(compgen -W "--json" -- "$cur"))
            ;;
        completions)
            COMPREPLY=($(compgen -W "bash zsh fish" -- "$cur"))
            ;;
        *)
            ;;
    esac
}

complete -F _systemd_sentry systemd-sentry sentry
"#
    .to_string()
}

fn generate_zsh_completions() -> String {
    r#"#compdef systemd-sentry sentry

_systemd_sentry() {
    local -a commands
    commands=(
        'daemon:Run supervisor daemon'
        'status:Query runtime status and circuit states'
        'check:Validate configuration and drop-in policies'
        'triage:On-demand root-cause triage of a systemd unit'
        'monitor:Stream live crash and remediation events'
        'incidents:List recent incident journal slices'
        'inspect:Inspect detailed diagnostic for an incident ID'
        'reset:Reset circuit breaker lockout for a unit'
        'mcp:Run Model Context Protocol server on stdio'
        'completions:Generate shell completion script'
        'setup:Launch interactive configuration wizard'
    )

    _arguments -C \
        '(-c --config)'{-c,--config}'[Path to config file]:config file:_files' \
        '(-s --socket)'{-s,--socket}'[Path to socket]:socket file:_files' \
        '--setup[Launch interactive setup wizard]' \
        '(-h --help)'{-h,--help}'[Print help information]' \
        '(-V --version)'{-V,--version}'[Print version information]' \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _describe -t commands 'systemd-sentry commands' commands
            ;;
    esac
}

_systemd_sentry "$@"
"#
    .to_string()
}

fn generate_fish_completions() -> String {
    r#"# fish completion for systemd-sentry and sentry

complete -c systemd-sentry -f
complete -c sentry -f

set -l commands daemon status check triage monitor incidents inspect reset mcp completions setup

complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -s c -l config -d "Path to config file" -r
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -s s -l socket -d "Path to socket" -r
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -l setup -d "Launch setup wizard"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -s h -l help -d "Print help"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -s V -l version -d "Print version"

complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -a daemon -d "Run supervisor daemon"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -a status -d "Query status and circuit states"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -a check -d "Validate configuration and policies"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -a triage -d "On-demand unit triage"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -a monitor -d "Stream live events"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -a incidents -d "List recent incidents"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -a inspect -d "Inspect incident diagnostic"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -a reset -d "Reset circuit lockout"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -a mcp -d "Run MCP stdio server"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -a completions -d "Generate completions"
complete -c systemd-sentry -n "not __fish_seen_subcommand_from $commands" -a setup -d "Launch setup wizard"
"#
    .to_string()
}
