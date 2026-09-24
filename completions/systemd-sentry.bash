# bash completion for systemd-sentry and sentry
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
