# Model Context Protocol (MCP) Server

`systemd-sentry` embeds a standard Model Context Protocol (MCP) server over `stdio`, enabling AI agent runtimes (such as Claude Desktop, Cursor, and Antigravity) to inspect system health, analyze incidents, and query telemetry safely.

---

## 1. Invocation

Launch the MCP server directly via standard Unix streams:
```bash
systemd-sentry mcp
# or using the sentry alias:
sentry mcp
```

---

## 2. Tools Exposed

### `sentry_list_incidents`
Lists recent system failure records with optional unit filtering:
```json
{
  "unit_name": "optional string (e.g. nginx.service)",
  "limit": 10
}
```

### `sentry_get_incident`
Fetches the full diagnostic record for a specific incident:
```json
{
  "incident_id": "string (e.g. inc-2026-0923-01)"
}
```

### `sentry_explain_incident`
Returns a structured technical root-cause breakdown including crash backtrace and suggested fixes:
```json
{
  "incident_id": "string",
  "detail_level": "brief | detailed"
}
```

### `sentry_get_unit_telemetry`
Queries live cgroups v2 memory statistics and kernel PSI pressure for a service:
```json
{
  "unit_name": "string (e.g. postgresql.service)"
}
```

---

## 3. Resources Exposed

* `sentry://incidents/{incident_id}`: Direct access to Markdown-formatted incident post-mortem.
* `sentry://telemetry/{unit_name}`: Live JSON cgroups and PSI metrics for a supervised unit.
* `sentry://policy`: Current active remediation policy merged with all `.d/` drop-ins.

---

## 4. Client Configurations

### Claude Desktop (`claude_desktop_config.json`)
```json
{
  "mcpServers": {
    "systemd-sentry": {
      "command": "/usr/local/bin/systemd-sentry",
      "args": ["mcp"]
    }
  }
}
```

### Cursor (`.cursor/mcp.json`)
```json
{
  "mcpServers": {
    "systemd-sentry": {
      "command": "systemd-sentry",
      "args": ["mcp"]
    }
  }
}
```
