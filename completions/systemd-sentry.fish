# fish completion for systemd-sentry and sentry

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
