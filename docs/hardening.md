# Security Hardening & Systemd Integrations

`systemd-sentry` enforces multi-layer defense-in-depth through Linux kernel security mechanisms, `systemd` sandbox primitives, and TPM2 hardware credential sealing.

---

## 1. Service Sandbox Directives

The `systemd-sentry.service` unit configures strict isolation parameters:

```ini
[Service]
# Unprivileged execution
User=sentry
Group=sentry
SupplementaryGroups=systemd-journal

# Kernel Capability Sandboxing
CapabilityBoundingSet=CAP_DAC_READ_SEARCH CAP_KILL
AmbientCapabilities=CAP_DAC_READ_SEARCH

# Filesystem Restrictions
ProtectSystem=strict
ProtectHome=yes
ProtectKernelTunables=yes
ProtectControlGroups=false
PrivateTmp=yes

# Memory & Process Protections
MemoryDenyWriteExecute=yes
RestrictRealtime=yes
RestrictSUIDSGID=yes

# Strictly Bounded Writable Paths
ReadWritePaths=/var/lib/systemd-sentry /var/log/systemd-sentry /run/systemd-sentry
```

### Rationale
* **`CAP_DAC_READ_SEARCH`**: Permits the daemon to read system journals and cgroup attributes without granting write permissions.
* **`CAP_KILL`**: Grants authority to send signals to managed service processes.
* **`MemoryDenyWriteExecute=yes`**: Prevents runtime code injection and JIT vulnerabilities.
* **`ProtectSystem=strict`**: Mounts the entire OS filesystem (`/usr`, `/etc`, `/boot`) read-only to the daemon.

---

## 2. Hardware Credential Sealing with `systemd-creds`

Rather than storing cloud inference API keys in plaintext configuration files, `systemd-sentry` leverages `systemd-creds` to seal credentials to the host's TPM2 chip.

### Sealing an OpenAI API Key
```bash
# Encrypt and store the credential securely
sudo systemd-creds encrypt \
  --name=openai_api_key \
  --with-key=tpm2 \
  /etc/credstore.encrypted/openai_api_key
```

### Service Credential Mount
Add a drop-in override for `systemd-sentry.service`:
```ini
# /etc/systemd/system/systemd-sentry.service.d/override.conf
[Service]
SetCredentialEncrypted=openai_api_key:/etc/credstore.encrypted/openai_api_key
```

### Runtime Discovery
When `systemd-sentry` starts, `systemd` automatically decrypts the key into a secure, volatile memory-mapped directory mounted at `$CREDENTIALS_DIRECTORY/openai_api_key`. The `sentry-diagnostic` credential loader checks this path automatically with zero disk plaintext exposure.

---

## 3. `systemd-oomd` Out-of-Memory Integration

When memory pressure escalates on low-memory servers:
1. `systemd-oomd` or the Linux kernel OOM killer terminates rogue processes.
2. `systemd` transitions the unit to `failed` with D-Bus property `Result=oom-kill` and exit code `137` (`SIGKILL`).
3. `sentry-driver` reads cgroup v2 `memory.events` counters (`oom_kill` and `high`) and PSI `/proc/pressure/memory`.
4. `sentry-diagnostic` classifies the failure immediately as a **High-Severity OOM Event**.
5. `sentry-safety` applies an exponential backoff (`RestartWithBackoff`) to prevent rapid reboot loops while the host memory stabilizes.

---

## 4. Zero-Copy `systemd-coredump` Inspection

Traditional crash dump analyzers load multi-hundred-megabyte core files into memory, easily crashing low-spec 512MB RAM nodes.

`systemd-sentry` bypasses core file contents entirely:
1. Upon `SIGSEGV` or `SIGABRT` crash transitions, `sentry-driver` inspects `/var/lib/systemd/coredump/`.
2. Sentry reads Linux extended attributes (`xattrs`) stored directly in inode metadata:
   * `user.coredump.signal`
   * `user.coredump.comm`
   * `user.coredump.backtrace` (first 10 stack frames)
3. Stack inspection uses a fixed 1 KiB buffer (`rustix::fs::getxattr`), consuming under 4 KiB of heap memory.

---

## 5. `systemd-inhibit` Shutdown & Sleep Delay Locks

During system shutdown, reboot, or suspend transitions:
1. `systemd-sentry` acquires an unprivileged delay inhibitor lock via `org.freedesktop.login1.Manager.Inhibit`.
2. Sentry continuously monitors `PrepareForShutdown` and `PrepareForSleep` D-Bus signals.
3. When active, Sentry suppresses false-positive crash alerts for services undergoing scheduled teardown.
4. Active diagnostic summaries and incident buffers are cleanly flushed to `/var/lib/systemd-sentry/incidents.db` before releasing the inhibitor file descriptor.

---

## 6. `systemd-resolved` DNS Fault Isolation

Network services frequently fail due to upstream DNS outages rather than application code defects:
1. `sentry-driver` queries `org.freedesktop.resolve1.Manager` to verify resolver health upon connection failures.
2. If `systemd-resolved` reports lookup failure or server timeouts, `sentry-diagnostic` tags the incident as an infrastructure DNS fault.
3. `sentry-safety` applies `RestartWithBackoff` rather than permanent circuit breaker trips.

---

## 7. `systemd-networkd` Link Carrier Correlation

1. `sentry-driver` queries `org.freedesktop.network1` for interface `OperationalState` (`routable`, `degraded`, `carrier`, `no-carrier`).
2. When services crash due to `EHOSTUNREACH`, `ENETUNREACH`, or carrier drops, Sentry correlates the crash with network layer unreachability.
3. Flapping lockouts are withheld until network carrier restoration is confirmed.

---

## 8. `systemd-timesyncd` Clock Verification

1. `sentry-driver` queries `org.freedesktop.timesync1.Manager` for clock synchronization state and NTP server reachability.
2. When TLS handshakes fail due to `CERT_HAS_EXPIRED` or `CERT_NOT_YET_VALID`, Sentry checks if the host clock has drifted or is unsynchronized.
3. Prevents incorrect application debugging when host time drift is the root cause.

---

## 9. `systemd-pstore` Kernel Panic Post-Mortem

1. On reboot after a kernel panic or hardware watchdog reset, `sentry-driver` scans `/sys/fs/pstore/` and `/var/lib/systemd/pstore/`.
2. Reads bounded panic logs (`dmesg-ramoops-*`, `console-ramoops-*`) using a fixed 4 KiB stack buffer (max 32 entries).
3. Indexes panic reports into forensic incident storage for operator inspection.

---

## 10. `systemd --user` Rootless Session Supervision

1. Discovers active user sessions via `/run/user/<UID>/bus` sockets (bounded to 64 active UIDs).
2. Connects to per-user D-Bus session instances to monitor user-level systemd units and applications.
3. Enforces unprivileged safety policies identical to system units.

