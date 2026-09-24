# Original User Request

## Initial Request — 2026-09-24T00:16:09Z

Build **systemd-sentry** (`syntry`), an autonomous, zero-trust system supervisor written in 100% pure Rust that brings intelligent root-cause triage, deterministic circuit-breaking, and guarded self-healing to `systemd`-managed Linux environments.

Working directory: `/home/jeryd/Projects/UberMetroid/systemd-sentry`
Integrity mode: development

## Requirements

### R1. Architectural Constraints & Code Structure
- Must be written in **100% pure Rust** with zero C dynamic library dependencies (pure Rust D-Bus via `zbus`, pure Rust socket writer for `sd_notify`/watchdog, pure Rust TLS via `rustls`, zero `libsystemd.so` or `libdbus-1.so`).
- **Strict File Limits**: Every source file must contain at most **256 lines of code**.
- **Single-Function Isolation**: Every file must be limited to a **single primary function** (or a single struct + constructor/handler).
- Logical, nested directory structure with clean module re-exports.

### R2. System Integration & Deep Host Connectivity
- **Native systemd Citizen**:
  - `Type=notify` and `WatchdogSec` heartbeat implementation writing directly to `$NOTIFY_SOCKET`.
  - Native socket activation support parsing `$LISTEN_FDS` (file descriptors `3..N`).
  - Standard packaging: `sysusers.d` (provisions `sentry` user in `systemd-journal` group), `tmpfiles.d` (provisions runtime/log/state dirs), `dbus-1/system.d` policy, and hardened systemd service unit.
- **Subsystem Hooks**:
  - Passive D-Bus listener (`org.freedesktop.systemd1`) tracking unit lifecycle and failure events.
  - Streaming journal ingestion from `systemd-journald` socket (`/run/systemd/journal/io.systemd.journal`).
  - Kernel telemetry parser for cgroups v2 (`/sys/fs/cgroup/`) and PSI (`/proc/pressure/{cpu,memory,io}`).
  - `systemd-coredump` extraction for top crash backtraces on `SIGSEGV`/`SIGABRT`.
  - `systemd-logind` session discovery for desktop notifications.

### R3. Diagnostic Engine & Dual LLM Providers
- Pluggable provider layer supporting:
  1. Local inference via Ollama (`/api/generate` or `/api/chat`) and `llama.cpp` server.
  2. Cloud OpenAI-compatible endpoints with API key support (`/v1/chat/completions`).
- Strict JSON schema enforcement for diagnostic payloads (root cause, evidence, severity, proposed remediation).
- Model Context Protocol (MCP) server implementation allowing external AI tools to query incident logs and unit telemetry.

### R4. Zero-Trust Safety Engine & Circuit Breaker
- The LLM is strictly advisory—it has **no shell access** and cannot execute arbitrary commands.
- Deterministic sliding-window circuit breaker that tracks failure rates and locks out flapping services.
- Declarative `/etc/systemd-sentry/policy.toml` controlling allowed actions (e.g. restart with backoff, reload, reset-failed).

### R5. User Experience, Notifications & Setup Wizard
- Desktop toast alerts via `org.freedesktop.Notifications` dispatched to active graphical sessions.
- Terminal wall alerts for headless sessions.
- Interactive CLI setup wizard (`systemd-sentry --setup`) that detects local Ollama, prompts for API keys, validates with a live ping, and writes configuration.
- Operator CLI subcommands: `status`, `incidents`, `inspect <id>`, `reset <unit>`.

### R6. QA, Edge Testing & Fuzzing
- **1:1 Function Tests**: Every production function has a corresponding unit QA test in `qa/unit/`.
- **Edge-Case Suite**: Stress tests for corrupted journal streams, non-UTF8 logs, 10,000 crashes/sec failure storms, and malformed LLM responses.
- **Fuzzing**: `cargo-fuzz` targets for journal parsing, D-Bus decoding, PSI text parsing, and JSON triage decoding.
- **Crash Fixtures**: Simulated crashing services (`segfault_service`, `oom_service`, `flapper_service`).

### R7. Installation, Documentation & Website
- `install/install.sh`: Idempotent installer managing binary deployment, sysusers, tmpfiles, D-Bus policy, unit files, completions, and man pages.
- `README.md` & `website/`: Built from first principles, zero trust, and designed for end-user clarity.
- `docs/`: Architecture, hardening, policy reference, provider guides, and MCP spec.

## Acceptance Criteria

### Compilation & Static Analysis
- [ ] `cargo check --workspace --all-targets` passes with 0 warnings/errors.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes cleanly.
- [ ] A verification script asserts that **zero files** exceed 256 lines of code.
- [ ] Binaries link with zero dynamic C dependencies (`ldd` shows no `libdbus` or `libsystemd`).

### Functional & Unit Verification
- [ ] `cargo test --workspace` passes 100% of 1:1 unit QA tests.
- [ ] Edge test suite passes (non-UTF8 log slicing, malformed LLM output, rapid flap rate limiting).
- [ ] All `cargo-fuzz` targets compile cleanly under `qa/fuzz/`.

### System & Packaging Verification
- [ ] `install/install.sh --dry-run` or validation passes syntax and permission checks.
- [ ] Systemd service, socket, sysusers, tmpfiles, and D-Bus policy files pass `systemd-analyze verify`.
- [ ] CLI `--setup` correctly validates local and OpenAI-compatible configuration.
