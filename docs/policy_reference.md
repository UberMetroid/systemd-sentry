# Policy Reference Guide

`systemd-sentry` enforces deterministic guardrails around all system recovery actions using declarative TOML policies.

---

## 1. Drop-In Configuration Precedence

Policy files are resolved and merged across the following search paths:

1. `/etc/systemd-sentry/policy.toml`: System baseline configuration.
2. `/etc/systemd-sentry/policy.d/*.toml`: Administrator drop-in overrides.
3. `/run/systemd-sentry/policy.d/*.toml`: Ephemeral runtime drop-in overrides.

Files within drop-in directories are loaded in **alphanumeric sorting order** (e.g. `10-database.toml` is merged before `90-override.toml`). Unit-specific settings in later files override earlier definitions.

### DoS Protection
* Every policy file must be **&le; 64 KiB** in size. Files exceeding this threshold are rejected to safeguard daemon memory.
* Syntax errors in individual drop-in files are safely logged and skipped; the supervisor continues operating using the last known valid configuration.

---

## 2. Configuration Schema

### `[global]`
| Field | Type | Default | Description |
|---|---|---|---|
| `protected_units` | `list<string>` | Standard system units | Services immune to supervisor restarts or resets. |
| `allowed_actions` | `list<string>` | `["RestartWithBackoff", "Reload", "ResetFailed", "NotifyOnly"]` | Permitted automated remediation actions. |
| `rate_limit_per_minute`| `integer` | `10` | Global cap on automated actions across all units per 60s. |
| `require_confirmation`| `boolean` | `false` | When true, automations require operator approval. |

### `[circuit]`
| Field | Type | Default | Description |
|---|---|---|---|
| `max_failures` | `integer` | `3` | Number of failures within `window_secs` before tripping the circuit. |
| `window_secs` | `integer` | `60` | Duration of the sliding failure observation window. |
| `cooldown_secs` | `integer` | `30` | Base cooldown before transitioning to `HalfOpen`. |
| `max_cooldown_secs` | `integer` | `1800` | Upper limit for exponential cooldown backoff (30 minutes). |
| `flap_threshold` | `integer` | `3` | Number of circuit trips within `flap_window_secs` before permanent lockout. |
| `flap_window_secs` | `integer` | `900` | Observation window for flapping detection (15 minutes). |

### `[units."<unit-name>"]`
| Field | Type | Default | Description |
|---|---|---|---|
| `allowed_actions` | `list<string>` | Inherits `[global]` | Permissible actions for this specific unit. |
| `max_failures` | `integer` | Inherits `[circuit]` | Custom failure threshold for this unit. |
| `auto_remediate` | `boolean` | `true` | Enable or disable automated restarts for this unit. |

---

## 3. Circuit Breaker State Machine

```
     +-------------------------------------------------------+
     |                                                       |
     v                                                       |
+----------+      Failures >= max_failures      +----------+ | Cooldown expires
|  CLOSED  | ---------------------------------> |   OPEN   | | (HalfOpen probe)
+----------+                                    +----------+ |
     ^                                                |      |
     |                                                v      |
     | Success                                  +----------+ |
     +----------------------------------------- | HALFOPEN | -+
                                                +----------+
                                                      |
                                                      | Trip count >= flap_threshold
                                                      v
                                            +--------------------+
                                            | PERMANENTLY LOCKED |
                                            +--------------------+
                                                      |
                                                      | Operator: sentry reset <unit>
                                                      v
                                                 (Back to CLOSED)
```

### Exponential Cooldown Backoff
When a circuit trips open, the cooldown duration scales exponentially with the number of trips recorded within the flapping window:
$$\text{cooldown} = \min\left(\text{cooldown\_secs} \times 2^{\text{trip\_count} - 1},\, \text{max\_cooldown\_secs}\right)$$

* Trip 1: 30s
* Trip 2: 60s
* Trip 3: 120s
* Trip 4+: Capped at `max_cooldown_secs` (1800s / 30m) or locked permanently if `trip_count >= flap_threshold`.

---

## 4. Protected Units Blacklist

The supervisor refuses any automated restart, reload, or kill targeting protected infrastructure:
* `systemd-journald.service`
* `systemd-logind.service`
* `systemd-udevd.service`
* `systemd-resolved.service`
* `dbus.service`
* `sshd.service`
* `ssh.service`
* `init.scope`
* `systemd-sentry.service`
