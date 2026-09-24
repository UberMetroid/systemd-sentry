#!/usr/bin/env sh
# ==============================================================================
# systemd-sentry Idempotent System Installer & Uninstaller
# ==============================================================================
# Usage:
#   sudo ./install/install.sh [OPTIONS]
#
# Options:
#   --prefix <DIR>       Installation prefix (default: /usr/local)
#   --sysconfdir <DIR>   System configuration directory (default: /etc)
#   --dry-run            Print actions without modifying the filesystem
#   --uninstall          Remove all installed systemd-sentry components
#   -h, --help           Show this help message and exit
# ==============================================================================

set -eu

PREFIX="/usr/local"
SYSCONFDIR="/etc"
DRY_RUN=0
UNINSTALL=0

print_usage() {
    cat <<EOF
systemd-sentry installer

Usage:
  $(basename "$0") [OPTIONS]

Options:
  --prefix <DIR>       Installation prefix (default: /usr/local)
  --sysconfdir <DIR>   System configuration directory (default: /etc)
  --dry-run            Preview changes without modifying the filesystem
  --uninstall          Remove all installed systemd-sentry files and units
  -h, --help           Show this help message and exit

Examples:
  sudo ./install/install.sh
  ./install/install.sh --dry-run
  sudo ./install/install.sh --uninstall
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)
            PREFIX="$2"
            shift 2
            ;;
        --prefix=*)
            PREFIX="${1#*=}"
            shift
            ;;
        --sysconfdir)
            SYSCONFDIR="$2"
            shift 2
            ;;
        --sysconfdir=*)
            SYSCONFDIR="${1#*=}"
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        --uninstall)
            UNINSTALL=1
            shift
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            echo "Error: Unknown argument: $1" >&2
            print_usage >&2
            exit 64
            ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN_DIR="${PREFIX}/bin"
MAN_DIR="${PREFIX}/share/man"
UNIT_DIR="${SYSCONFDIR}/systemd/system"
SYSUSERS_DIR="${SYSCONFDIR}/sysusers.d"
TMPFILES_DIR="${SYSCONFDIR}/tmpfiles.d"
DBUS_DIR="${SYSCONFDIR}/dbus-1/system.d"
CONFIG_DIR="${SYSCONFDIR}/systemd-sentry"
BASH_COMP_DIR="/usr/share/bash-completion/completions"
ZSH_COMP_DIR="/usr/share/zsh/site-functions"
FISH_COMP_DIR="/usr/share/fish/vendor_completions.d"

run_cmd() {
    if [ "$DRY_RUN" -eq 1 ]; then
        printf "[DRY-RUN] %s\n" "$*"
    else
        "$@"
    fi
}

check_root() {
    if [ "$DRY_RUN" -eq 0 ] && [ "$(id -u)" -ne 0 ]; then
        echo "Error: Installation requires root privileges. Please re-run with sudo." >&2
        exit 77
    fi
}

locate_binary() {
    if [ -f "${SCRIPT_DIR}/target/release/systemd-sentry" ]; then
        echo "${SCRIPT_DIR}/target/release/systemd-sentry"
    elif [ -f "${SCRIPT_DIR}/target/debug/systemd-sentry" ]; then
        echo "${SCRIPT_DIR}/target/debug/systemd-sentry"
    else
        echo ""
    fi
}

do_uninstall() {
    check_root
    echo "==> Uninstalling systemd-sentry..."

    if command -v systemctl >/dev/null 2>&1 && [ "$DRY_RUN" -eq 0 ]; then
        systemctl stop systemd-sentry.service 2>/dev/null || true
        systemctl disable systemd-sentry.service 2>/dev/null || true
        systemctl stop systemd-sentry.socket 2>/dev/null || true
        systemctl disable systemd-sentry.socket 2>/dev/null || true
    fi

    echo "Removing executables and symlinks..."
    run_cmd rm -f "${BIN_DIR}/systemd-sentry" "${BIN_DIR}/sentry"

    echo "Removing systemd units and configs..."
    run_cmd rm -f "${UNIT_DIR}/systemd-sentry.service" "${UNIT_DIR}/systemd-sentry.socket"
    run_cmd rm -f "${SYSUSERS_DIR}/systemd-sentry.conf"
    run_cmd rm -f "${TMPFILES_DIR}/systemd-sentry.conf"
    run_cmd rm -f "${DBUS_DIR}/org.freedesktop.SystemdSentry.conf"

    echo "Removing man pages..."
    run_cmd rm -f "${MAN_DIR}/man1/systemd-sentry.1" "${MAN_DIR}/man1/sentry.1"
    run_cmd rm -f "${MAN_DIR}/man5/systemd-sentry.conf.5"
    run_cmd rm -f "${MAN_DIR}/man8/systemd-sentry.service.8" "${MAN_DIR}/man8/sentry.service.8"

    echo "Removing shell completions..."
    run_cmd rm -f "${BASH_COMP_DIR}/systemd-sentry" "${BASH_COMP_DIR}/sentry"
    run_cmd rm -f "${ZSH_COMP_DIR}/_systemd-sentry"
    run_cmd rm -f "${FISH_COMP_DIR}/systemd-sentry.fish"

    if command -v systemctl >/dev/null 2>&1; then
        run_cmd systemctl daemon-reload
    fi

    echo "[SUCCESS] systemd-sentry uninstallation complete."
    echo "Note: /var/lib/systemd-sentry, /var/log/systemd-sentry, and /etc/systemd-sentry were preserved."
    exit 0
}

do_install() {
    check_root
    echo "==> Installing systemd-sentry..."

    SRC_BIN="$(locate_binary)"
    if [ -z "$SRC_BIN" ]; then
        if [ "$DRY_RUN" -eq 1 ]; then
            SRC_BIN="${SCRIPT_DIR}/target/release/systemd-sentry (simulated)"
        else
            echo "Notice: Compiled binary not found in target/. Compiling release binary now..."
            (cd "${SCRIPT_DIR}" && cargo build --release --bin systemd-sentry)
            SRC_BIN="${SCRIPT_DIR}/target/release/systemd-sentry"
        fi
    fi

    echo "Creating target directories..."
    run_cmd mkdir -p "${BIN_DIR}"
    run_cmd mkdir -p "${UNIT_DIR}"
    run_cmd mkdir -p "${SYSUSERS_DIR}"
    run_cmd mkdir -p "${TMPFILES_DIR}"
    run_cmd mkdir -p "${DBUS_DIR}"
    run_cmd mkdir -p "${CONFIG_DIR}/policy.d"
    run_cmd mkdir -p "${MAN_DIR}/man1" "${MAN_DIR}/man5" "${MAN_DIR}/man8"
    run_cmd mkdir -p "${BASH_COMP_DIR}" "${ZSH_COMP_DIR}" "${FISH_COMP_DIR}"

    echo "Installing binary and symlinks..."
    run_cmd install -m 0755 "${SRC_BIN}" "${BIN_DIR}/systemd-sentry"
    run_cmd ln -sf systemd-sentry "${BIN_DIR}/sentry"

    echo "Installing systemd units and drop-ins..."
    run_cmd install -m 0644 "${SCRIPT_DIR}/systemd/systemd-sentry.service" "${UNIT_DIR}/systemd-sentry.service"
    run_cmd install -m 0644 "${SCRIPT_DIR}/systemd/systemd-sentry.socket" "${UNIT_DIR}/systemd-sentry.socket"
    run_cmd install -m 0644 "${SCRIPT_DIR}/systemd/sysusers.d/systemd-sentry.conf" "${SYSUSERS_DIR}/systemd-sentry.conf"
    run_cmd install -m 0644 "${SCRIPT_DIR}/systemd/tmpfiles.d/systemd-sentry.conf" "${TMPFILES_DIR}/systemd-sentry.conf"
    run_cmd install -m 0644 "${SCRIPT_DIR}/systemd/dbus-1/system.d/org.freedesktop.SystemdSentry.conf" "${DBUS_DIR}/org.freedesktop.SystemdSentry.conf"

    echo "Installing default configuration templates if missing..."
    if [ ! -f "${CONFIG_DIR}/config.toml" ] && [ -f "${SCRIPT_DIR}/config/config.toml.example" ]; then
        run_cmd install -m 0644 "${SCRIPT_DIR}/config/config.toml.example" "${CONFIG_DIR}/config.toml"
    fi
    if [ ! -f "${CONFIG_DIR}/policy.toml" ] && [ -f "${SCRIPT_DIR}/config/policy.toml.example" ]; then
        run_cmd install -m 0644 "${SCRIPT_DIR}/config/policy.toml.example" "${CONFIG_DIR}/policy.toml"
    fi
    if [ ! -f "${CONFIG_DIR}/policy.d/00-default.toml" ] && [ -f "${SCRIPT_DIR}/config/policy.d/00-default.toml" ]; then
        run_cmd install -m 0644 "${SCRIPT_DIR}/config/policy.d/00-default.toml" "${CONFIG_DIR}/policy.d/00-default.toml"
    fi

    echo "Installing manual pages..."
    run_cmd install -m 0644 "${SCRIPT_DIR}/man/man1/systemd-sentry.1" "${MAN_DIR}/man1/systemd-sentry.1"
    run_cmd ln -sf systemd-sentry.1 "${MAN_DIR}/man1/sentry.1"
    run_cmd install -m 0644 "${SCRIPT_DIR}/man/man5/systemd-sentry.conf.5" "${MAN_DIR}/man5/systemd-sentry.conf.5"
    run_cmd install -m 0644 "${SCRIPT_DIR}/man/man8/systemd-sentry.service.8" "${MAN_DIR}/man8/systemd-sentry.service.8"
    run_cmd ln -sf systemd-sentry.service.8 "${MAN_DIR}/man8/sentry.service.8"

    echo "Installing shell completions..."
    run_cmd install -m 0644 "${SCRIPT_DIR}/completions/systemd-sentry.bash" "${BASH_COMP_DIR}/systemd-sentry"
    run_cmd ln -sf systemd-sentry "${BASH_COMP_DIR}/sentry"
    run_cmd install -m 0644 "${SCRIPT_DIR}/completions/_systemd-sentry" "${ZSH_COMP_DIR}/_systemd-sentry"
    run_cmd install -m 0644 "${SCRIPT_DIR}/completions/systemd-sentry.fish" "${FISH_COMP_DIR}/systemd-sentry.fish"

    if [ "$DRY_RUN" -eq 0 ]; then
        echo "Configuring system accounts and runtime directories..."
        if command -v systemd-sysusers >/dev/null 2>&1; then
            systemd-sysusers "${SYSUSERS_DIR}/systemd-sentry.conf" || true
        fi
        if command -v systemd-tmpfiles >/dev/null 2>&1; then
            systemd-tmpfiles --create "${TMPFILES_DIR}/systemd-sentry.conf" || true
        fi
        if command -v systemctl >/dev/null 2>&1; then
            systemctl daemon-reload || true
        fi
    else
        printf "[DRY-RUN] systemd-sysusers %s\n" "${SYSUSERS_DIR}/systemd-sentry.conf"
        printf "[DRY-RUN] systemd-tmpfiles --create %s\n" "${TMPFILES_DIR}/systemd-sentry.conf"
        printf "[DRY-RUN] systemctl daemon-reload\n"
    fi

    echo ""
    echo "[SUCCESS] systemd-sentry installed successfully!"
    echo "To activate:"
    echo "  sudo systemctl enable --now systemd-sentry.socket"
    echo "  sudo systemctl start systemd-sentry.service"
    echo ""
    echo "To test:"
    echo "  sentry status"
    echo "  sentry check"
}

if [ "$UNINSTALL" -eq 1 ]; then
    do_uninstall
else
    do_install
fi
