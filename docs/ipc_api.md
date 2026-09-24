# Local IPC & API Specification

`systemd-sentry` adheres strictly to the Unix philosophy, Linus Torvalds' kernel pragmatism, and systemd conventions. It deliberately **never opens a TCP network port**. All communication occurs through local kernel-enforced interfaces.

---

## 1. Local UNIX Domain Socket IPC

* **Socket Path**: `/run/systemd-sentry/sentry.sock`
* **Socket Activation**: Managed on-demand via `systemd-sentry.socket` (`ListenStream=...`).
* **Protocol**: Line-delimited JSON-RPC 2.0 requests and responses.
* **Kernel Credential Verification (`SO_PEERCRED`)**:
  * On every client connection, the daemon queries `SO_PEERCRED` via `rustix::net::sockopt::get_socket_peer_credentials`.
  * The Linux kernel authoritative `uid` and `gid` are inspected:
    * Read commands (`status`, `incidents`, `inspect`) permitted for `uid == 0` or members of groups `wheel` / `adm`.
    * Mutating commands (`reset`) permitted strictly for `uid == 0`.
  * Zero passwords, zero API tokens, zero credential leaks.

---

## 2. System D-Bus Interface

Sentry registers a well-known service on the Linux system bus:

* **Bus Name**: `org.freedesktop.SystemdSentry`
* **Object Path**: `/org/freedesktop/SystemdSentry`
* **Interface**: `org.freedesktop.SystemdSentry1.Manager`
* **Methods**:
  * `GetStatus() -> (s: status, u: active_units, u: tripped_units)`
  * `ListIncidents() -> a(ssss)`: Returns array of recent incidents.
  * `InspectIncident(s: incident_id) -> s`: Returns full Markdown post-mortem.
  * `ResetCircuit(s: unit_name) -> b`: Resets a tripped circuit breaker.
* **Security Policy (`/usr/share/dbus-1/system.d/org.freedesktop.SystemdSentry.conf`)**:
  * Root/sentry user can claim the bus name.
  * Local users can query status.
  * Privileged operations gated by D-Bus policy / Polkit.

---

## 3. Model Context Protocol (MCP) via stdio

For integration with external AI agent runtimes (such as Antigravity, Claude Desktop, or custom scripts), Sentry provides an MCP server:

* **Command**: `systemd-sentry mcp`
* **Transport**: Standard Unix `stdio` (stdin/stdout pipe streams).
* **Protocol**: JSON-RPC 2.0 Model Context Protocol schema.
* **Tools Exposed**:
  * `sentry_status`: Query active daemon state and supervised units.
  * `sentry_list_incidents`: Fetch recent failure events and triage summaries.
  * `sentry_inspect_incident`: Get root-cause diagnosis, evidence, and cgroup stats.
  * `sentry_reset_unit`: Supervised action to clear a tripped breaker.

---

## 4. Architectural Guarantees

1. **Zero Network Attack Surface**: Sentry does not bind to `0.0.0.0` or `127.0.0.1` TCP ports.
2. **Local Security Boundaries**: All authorization relies on the Linux kernel's process and filesystem credentials.
3. **Pipeable Unix Streams**: Non-interactive outputs emit line-oriented JSON or clean text suitable for `jq` and `awk`.
