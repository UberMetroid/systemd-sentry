# systemd-sentry (`syntry`)

> **Autonomous, zero-trust systemd supervisor in pure Rust.**  
> Intercepts crashes at the D-Bus and kernel layer, slices causal logs, provides instant root-cause analysis via local or cloud AI, and enforces deterministic circuit-breaking to halt flapping.

[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Language: Rust](https://img.shields.io/badge/Language-Pure%20Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform: Linux / systemd](https://img.shields.io/badge/Platform-Linux%20%2F%20systemd-red.svg)](https://systemd.io/)
[![Zero C Dependencies](https://img.shields.io/badge/Dependencies-Zero%20C%20Libs-green.svg)](#architecture)

---

## Why Sentry? (First Principles)

1. **The Flapping Trap**: When a service crashes, standard supervisors (`systemd`, `supervisord`, `k8s`) blindly restart it. If the cause is unrecoverable (syntax error in config, missing database migration, out of disk), the service restarts in a tight loop—wasting CPU, thrashing logs, and hammering dependencies.
2. **The 3 AM Triage Gap**: Operators are woken up to manually run `journalctl -u foo --since "5 min ago"`, inspect `/proc`, parse stack traces, and guess what broke.
3. **The Solution**: Sentry operates natively on the system D-Bus. When a failure occurs, it slices the causal binary journal window, extracts cgroup v2 & kernel PSI telemetry, diagnoses root cause using local or cloud AI, and stops retries if a permanent defect is detected.

---

## Zero-Trust Architecture (The AI Never Gets Root)

Sentry treats LLM inference strictly as an **untrusted advisory component**:

* **No Shell Execution**: The AI engine has zero access to shell execution (`sh`, `bash`, `exec`). It can never run arbitrary scripts or modify files.
* **Deterministic Whitelists**: Remediation is limited to hardcoded state transitions (`RestartUnit`, `ReloadUnit`, `ResetFailedUnit`) executed via typed D-Bus method calls if—and only if—explicitly permitted by `/etc/systemd-sentry/policy.toml`.
* **Kernel & Host Sandboxing**: `systemd-sentry` runs under an unprivileged `sentry` user with dropped capabilities (`ProtectSystem=strict`, `NoNewPrivileges=yes`, `MemoryDenyWriteExecute=yes`, `ProtectHome=yes`).

```
Crash Signal (D-Bus)
       │
       ▼
Telemetry Slicer (Journal + cgroups v2 + PSI)
       │
       ▼
Advisory AI Engine (Ollama / llama.cpp / Cloud API)
       │  (JSON Diagnosis Only)
       ▼
Deterministic Policy Engine (policy.toml + Circuit Breaker)
       │  (Verified Against Whitelist)
       ▼
Guarded D-Bus Action (Restart, Backoff, or Lockout) ──> Desktop Toast / Alert
```

---

## Quickstart (60 Seconds)

### 1. Installation
Clone and run the zero-trust installer:
```bash
git clone https://github.com/UberMetroid/systemd-sentry.git
cd systemd-sentry
sudo ./install/install.sh
```

The installer:
* Provisions the unprivileged `sentry` user and adds it to `systemd-journal`.
* Sets up `/run`, `/var/log`, and `/var/lib` paths via `tmpfiles.d`.
* Installs systemd units, D-Bus system bus policies, shell completions, and man pages.

### 2. Interactive Setup
Run the guided configuration wizard:
```bash
systemd-sentry --setup
```
The wizard auto-detects local Ollama/llama.cpp instances, configures API keys with `systemd-creds` encryption, and tests connection health.

### 3. Start the Supervisor
```bash
sudo systemctl enable --now systemd-sentry.service
```

---

## Configuration

Sentry separates global daemon settings from per-unit safety policies.

### Global Configuration (`/etc/systemd-sentry/config.toml`)
```toml
[daemon]
log_level = "info"
listen_socket = "/run/systemd-sentry/sentry.sock"
incident_dir = "/var/log/systemd-sentry/incidents"
notify_desktop = true    # Emits desktop toasts via org.freedesktop.Notifications
notify_wall = false       # Emits terminal alerts to active TTYs

[inference]
# Supported: "ollama", "llama_cpp", "openai_compatible"
provider = "ollama"
endpoint = "http://127.0.0.1:11434"
model = "qwen2.5-coder:7b"
timeout_secs = 30
temperature = 0.1
# api_key = "..."         # Optional for OpenAI-compatible cloud services

[mcp]
enabled = true
socket_path = "/run/systemd-sentry/mcp.sock"
```

### Safety Policy & Circuit Breakers (`/etc/systemd-sentry/policy.toml`)
```toml
[defaults]
max_restarts = 3          # Max allowed failures in window before tripping
window_seconds = 300      # 5-minute sliding window
cooldown_seconds = 600    # Lockout cooldown duration
allow_restart = true      # Permit autonomous restart on transient errors

[units."nginx.service"]
max_restarts = 5
window_seconds = 180
allow_restart = true

[units."postgresql.service"]
max_restarts = 1
allow_restart = false     # Never auto-restart databases without human approval
```

---

## CLI & Operator Tooling

Inspect system health, review post-mortems, and manage circuit breakers:

```bash
# Check daemon status and supervised units
systemd-sentry status

# List recent incident post-mortems
systemd-sentry incidents

# View full triage report for an incident
systemd-sentry inspect inc-20260923-01

# Manually reset a tripped circuit breaker
systemd-sentry reset nginx.service
```

---

## Desktop Toasts & Alerts

When a service fails or a circuit breaker trips, Sentry queries `systemd-logind` to find active graphical sessions (Wayland / X11) and dispatches native desktop toasts:

> **🚨 Service Failure: nginx.service**  
> *Root Cause*: Syntax error in `/etc/nginx/nginx.conf` line 42.  
> *Action*: Restart prevented to avoid flapping. Circuit breaker active.  
> `[ Inspect Report ]` `[ Mute ]`

---

## Specifications

* **Language**: 100% Pure Rust (no C dynamic libraries: `zbus`, pure Rust socket writer, `rustls`).
* **Source Constraints**: Strictly $\le 256$ lines per file; single function per file.
* **Testing**: 1:1 unit QA tests for every function, edge test suite, and `cargo-fuzz` targets.
* **System Integration**: `Type=notify`, `WatchdogSec`, socket activation, `sysusers.d`, `tmpfiles.d`, `dbus-1` policies, cgroups v2, kernel PSI, and `systemd-coredump`.

---

## License

[Apache 2.0](LICENSE) © UberMetroid & Contributors
