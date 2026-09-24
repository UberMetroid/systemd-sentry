# System Architecture & Design Principles

`systemd-sentry` is engineered at the intersection of classical Unix philosophy, Linus Torvalds' kernel pragmatism, and modern `systemd` ecosystem patterns.

---

## 1. Classical Unix Philosophy

* **Do One Thing Well**: `sentry` is a systemd-native failure supervisor. It intercepts failures, extracts causal telemetry, generates structured diagnostics, and applies deterministic circuit-breaking. It is not an init system, nor a general-purpose monitoring dashboard.
* **Separation of Mechanism and Policy**:
  * **Mechanism**: Intercepting signals, slicing journals, and executing D-Bus transitions live in `sentry-driver`.
  * **Policy**: Rules governing failure thresholds, backoff curves, and permitted remediation live in declarative TOML policies (`policy.toml` and `policy.d/`).
* **Text Streams & Composability**:
  * The CLI outputs plain, parseable text by default when stdout is not a TTY (`--json` and line-delimited outputs for piping into `jq`, `grep`, or `awk`).
  * Silence is golden: CLI subcommands return code `0` silently on success unless verbose mode is requested.
* **Standard Exit Codes**: Follows `sysexits.h` conventions (`0` = OK, `64` = usage, `70` = internal error, `78` = configuration error).

---

## 2. Torvalds Pragmatism (Linux Kernel Alignment)

* **Pragmatism Over Dogma**: Simple, fast, deterministic algorithms over convoluted abstractions or academic object hierarchies.
* **Respect Linux Kernel Semantics**:
  * Direct interaction with `/sys/fs/cgroup/` (cgroups v2) and `/proc/pressure/` (PSI).
  * Direct syscalls via `rustix` and `std::os::unix` for low overhead and precise error codes.
  * Non-blocking I/O and bounded buffers to prevent memory bloat (< 15MB RSS daemon target).
* **Never Break Userspace**:
  * The daemon never panics in production event-handling loops.
  * Corrupted journal lines, malformed PSI tokens, or invalid D-Bus properties are handled gracefully with fallbacks and resynchronization.
* **Zero-Copy & Low Allocation**: Log slicing and telemetry parsing operate over borrowed byte slices (`&[u8]`) where possible.

---

## 3. systemd Ecosystem Integration

* **Drop-In Configuration Directories**:
  * Supports `/etc/systemd-sentry/policy.d/*.toml` and `/run/systemd-sentry/policy.d/*.toml` drop-ins, following systemd's alphabetical precedence (`10-default.toml`, `50-override.toml`).
* **Native Lifecycle Protocols**:
  * Implements `Type=notify` (`READY=1`, `STATUS=...`, `STOPPING=1`) and `WatchdogSec` heartbeat loops directly over `$NOTIFY_SOCKET`.
  * Binds to activated file descriptors passed via `$LISTEN_FDS` (`3..N`).
* **Declarative Host Provisioning**:
  * `sysusers.d` for unprivileged system user provisioning.
  * `tmpfiles.d` for ephemeral directory creation (`/run/systemd-sentry/`, `/var/log/systemd-sentry/`).
  * `dbus-1/system.d` policy files for system bus permissions.
