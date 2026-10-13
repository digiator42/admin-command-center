# Admin Command Center — API contract for the frontend

Backend: `admin-command-center` (GritShield 0.3.1 + SeaORM/Postgres). Default
address `http://127.0.0.1:8000`. All JSON bodies use
`Content-Type: application/json`. This document is the contract the new
`veloasm` WASM frontend is built against.

> Sibling contracts this API mirrors: `LuxShield/docs/api.md`,
  `logs-soc/docs/api.md` (CyberFlow), `GritIRT/docs/api.md`.

---

## Configuration

| Setting        | Env var                 | Default                                            |
|----------------|-------------------------|----------------------------------------------------|
| Bind host      | `ACC_BIND`              | `127.0.0.1`                                        |
| Port           | `ACC_PORT`              | `8000`                                             |
| Postgres DSN   | `PG_URL`                | *(required)*                                       |
| JWT secret     | `ACC_JWT_SECRET`        | `acc-insecure-development-secret-change-me`        |
| LuxShield base | `ACC_LUXSHIELD_URL`     | `http://luxshield-backend:8020`                    |
| LuxShield key  | `ACC_LUXSHIELD_API_KEY` | *(unset)*                                          |
| CyberFlow base | `ACC_CYBERFLOW_URL`     | `http://cyberflow-backend:8010`                    |
| CyberFlow key  | `ACC_CYBERFLOW_API_KEY` | *(unset)*                                          |
| GritIRT base   | `ACC_GRITIRT_URL`       | `http://gritirt-backend:8030`                      |
| GritJira base  | `ACC_GRITJIRA_URL`      | `http://gritjira-backend:8040`                     |

`ACC_JWT_SECRET` **must** be set to a long random value in production; the
default is a development placeholder and a warning is logged at boot.

---

## Authentication — JWT (HS256)

Auth is a **stateless JWT** validated inside the GritShield `AuthMiddleware`
(`AuthMiddleware::new_jwt`). The token carries the standard claims
`{ sub, role, exp }` where:

- `sub` — the numeric user id (as a string)
- `role` — one of `Admin`, `Operator`, `Auditor`
- `exp` — expiry as a Unix timestamp (seconds); tokens live **12 hours**

### Flow

1. `POST /api/v1/auth/login` with `{ "username", "password" }`.
2. The response contains `token` (and `token_type: "Bearer"`).
3. Send it on every other call:

   ```
   Authorization: Bearer <token>
   ```

4. On expiry (`401`) the frontend must call `/login` again.

### Role model (fixed hierarchy)

| Role       | Grants                                                       |
|------------|--------------------------------------------------------------|
| `Admin`    | everything; manages users, deletes services/docs, shutdown   |
| `Operator` | create/update services & docs, control ops, firewall changes |
| `Auditor`  | read-only                                                    |

`Admin` implicitly satisfies any lower requirement; `Operator` satisfies
`Operator`/`Auditor`; `Auditor` satisfies only `Auditor`.

### Public (unauthenticated) routes

`GET /health`, `POST /api/v1/auth/login`, `POST /api/v1/auth/logout`,
`ws://…/api/live-telemetry`, and `POST /mcp`. **Everything else requires a
valid Bearer token**; without one the middleware answers `401`.

### Seeded accounts

| Username   | Password      | Role       |
|------------|---------------|------------|
| `admin`    | `admin123`    | `Admin`    |
| `operator` | `operator123` | `Operator` |
| `auditor`  | `auditor123`  | `Auditor`  |

Change these before any non-local deployment.

---

## Common response envelopes

- Success: the resource object (often under a named key, e.g. `{ "service": … }`)
  plus, for lists, a `count`.
- Error: `{ "error": "<code>", ... }` with a matching HTTP status.

Hardening headers are attached to every response:
`X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy`, `Cache-Control`
and `X-ACC-Response-Time-Ms`. CORS is open (`Access-Control-Allow-Origin: *`).

---

## Health

### `GET /health` (public)

```json
{
  "service": "admin-command-center",
  "status": "ok",
  "version": "0.1.0",
  "db": true,
  "uptime_seconds": 1234
}
```

`status` is `degraded` and `db` is `false` when the Postgres ping fails.

---

## Auth

### `POST /api/v1/auth/login` (public)

Body:

```json
{ "username": "admin", "password": "admin123" }
```

`200`:

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9....",
  "token_type": "Bearer",
  "expires_in": 43200,
  "expires_at": 1791502110,
  "user": { "id": 1, "username": "admin", "role": "Admin", "created_at": "…" }
}
```

`400 { "error": "missing_credentials" }` — empty username/password.
`400 { "error": "invalid_json" }` — malformed body.
`401 { "error": "invalid_credentials" }` — unknown user or wrong password.

### `POST /api/v1/auth/logout` (public)

Stateless JWT — the server has nothing to revoke. Discard the token client-side.

```json
{ "status": "ok", "detail": "stateless JWT: discard the token client-side" }
```

### `GET /api/v1/auth/me` (auth)

```json
{
  "user": { "id": 1, "username": "admin", "role": "Admin", "created_at": "…" },
  "claims": { "sub": "1", "role": "Admin", "exp": 1791502110 }
}
```

---

## System telemetry

### `GET /api/v1/system/metrics` (auth)

Framework counters plus monitored-service rollups.

```json
{
  "active_connections": 3,
  "total_requests": 512,
  "total_blocked_ips": 0,
  "total_rate_limited_requests": 0,
  "total_errors_4xx": 4,
  "total_errors_5xx": 0,
  "avg_latency_ms": 1.42,
  "uptime_seconds": 910,
  "timestamp": "2026-…Z",
  "services_total": 4,
  "services_up": 4
}
```

### `GET /api/v1/system/info` (auth)

```json
{
  "service": "admin-command-center",
  "version": "0.1.0",
  "started_at": 1791500000,
  "uptime_seconds": 910,
  "auth": "JWT (HS256) via Authorization: Bearer",
  "claims": { "sub": "1", "role": "Admin", "exp": 1791502110 },
  "connectors": [
    { "name": "luxshield", "url": "http://luxshield-backend:8020", "configured_key": false }
  ]
}
```

### `GET /api/v1/system/stream` (auth) — SSE

Server-Sent Events telemetry tail, **one frame per second** under the event name
`metrics` (plus an immediate initial frame). `data` is the same shape as
`/system/metrics`.

```
event: metrics
data: {"active_connections":1,"total_requests":42,…}
```

> Browsers cannot set headers on `EventSource`. Since this endpoint is
> authenticated, use a polyfill fetch-based SSE client (WASM frontend) or the
> WebSocket below, and pass the token via `Authorization`.

### `ws://…/api/live-telemetry` (public) — WebSocket

A JSON telemetry frame per second:

```json
{
  "active_connections": 1,
  "total_blocked_ips": 0,
  "cpu_worker_latency_ms": 12,
  "memory_usage_bytes": 44040192
}
```

Kept public because browsers cannot attach an `Authorization` header to a
WebSocket handshake.

---

## Monitored services

Rows come from the `monitored_services` table, probed every 30 s by the monitor
worker; each probe also appends a row to `service_heartbeats`.

`service` object:

```json
{
  "id": 1,
  "name": "LuxShield Backend API",
  "target_url": "http://luxshield-backend:8020/health",
  "expected_status": 200,
  "ping_interval_seconds": 30,
  "current_status": "UP",
  "updated_at": "2026-…"
}
```

### `GET /api/v1/services` (auth)

```json
{ "count": 4, "services": [ { … } ] }
```

### `GET /api/v1/services/:id` (auth)

```json
{ "service": { … } }
```

`404 { "error": "not_found", "id": 99 }`.

### `POST /api/v1/services` (auth, Operator+)

Body (only `name` + `target_url` required):

```json
{ "name": "New API", "target_url": "http://svc:9000/health",
  "expected_status": 200, "ping_interval_seconds": 30 }
```

`201 { "service": { … "current_status": "PENDING" } }`.
`400 { "error": "missing_fields", "required": ["name", "target_url"] }`.

### `PUT /api/v1/services/:id` (auth, Operator+)

Body: any of `name`, `target_url`, `expected_status`, `ping_interval_seconds`.

`200 { "service": { … } }` — `404` when the id is unknown.

### `DELETE /api/v1/services/:id` (auth, Admin)

```json
{ "deleted": 1, "id": 1 }
```

Cascades to that service's heartbeats.

### `GET /api/v1/services/:id/heartbeats?limit=100` (auth)

Newest first. `limit` default 100, max 1000.

```json
{
  "count": 2,
  "heartbeats": [
    { "id": 9, "service_id": 1, "latency_ms": 2, "http_status_code": 200,
      "status_msg": "UP", "error_log": null, "checked_at": "2026-…" }
  ]
}
```

### `GET /api/v1/heartbeats?limit=100` (auth)

Same shape, across **all** services.

---

## Users / RBAC

All endpoints here are **Admin only**.

### `GET /api/v1/users` (auth, Admin)

```json
{ "count": 3, "users": [ { "id": 1, "username": "admin", "role": "Admin", "created_at": "…" } ] }
```

### `POST /api/v1/users` (auth, Admin)

```json
{ "username": "analyst", "password": "s3cret!", "role": "Auditor" }
```

`role` defaults to `Auditor`. Password is hashed with bcrypt (cost 12).

`201 { "user": { … } }`.
`400 { "error": "missing_fields", "required": ["username","password"] }`.
`400 { "error": "invalid_role", "allowed": ["Admin","Operator","Auditor"] }`.
`409 { "error": "username_taken" }`.

### `PUT /api/v1/users/:id` (auth, Admin)

Body: optional `role` and/or `password` (min 6 chars).

`200 { "user": { … } }`.
`400 { "error": "weak_password", … }` / `invalid_role`.

### `DELETE /api/v1/users/:id` (auth, Admin)

```json
{ "deleted": 1, "id": 4 }
```

---

## Platform docs

Markdown documents keyed by a `slug` (which may contain slashes, so the slug
travels in the body or a query param, never a path segment).

### `GET /api/v1/docs` (auth)

Metadata list (no markdown body):

```json
{ "count": 1, "docs": [ { "id": 1, "slug": "security/rate-limiting", "title": "…", "created_at": "…" } ] }
```

### `GET /api/v1/docs?slug=security/rate-limiting` (auth)

Full document including `markdown_body`:

```json
{ "doc": { "id": 1, "slug": "…", "title": "…", "markdown_body": "…", "created_at": "…" } }
```

`404 { "error": "not_found", "slug": "…" }`.

### `POST /api/v1/docs` (auth, Operator+)

```json
{ "slug": "runbooks/onboarding", "title": "Onboarding", "markdown_body": "# …" }
```

`201 { "doc": { … } }` — `409 { "error": "slug_taken" }` on a duplicate slug.

### `PUT /api/v1/docs` (auth, Operator+)

```json
{ "slug": "runbooks/onboarding", "title": "New title", "markdown_body": "# …" }
```

`slug` is required; `title`/`markdown_body` optional. `200 { "doc": { … } }`.

### `DELETE /api/v1/docs?slug=runbooks/onboarding` (auth, Admin)

```json
{ "deleted": 1, "slug": "runbooks/onboarding" }
```

`400 { "error": "missing_slug" }` when absent.

---

## Control plane

### `POST /api/v1/control/flush-counters` (auth, Operator+)

Zeroes the process-wide telemetry registers.

```json
{ "status": "ok", "message": "telemetry counters reset to baseline" }
```

### `POST /api/v1/control/emergency-shutdown` (auth, Admin)

Schedules a process exit ~500 ms after the response is flushed.

`202 { "status": "ok", "message": "emergency shutdown sequence initialized" }`

### `POST /api/v1/control/block-ip` (auth, Operator+)

Convenience proxy to LuxShield `POST /api/v1/firewall/rules`.
Body forwarded verbatim, e.g.:

```json
{ "ip": "203.0.113.66", "duration_secs": 120, "reason": "incident xyz" }
```

### `POST /api/v1/control/unblock-ip?ip=203.0.113.66` (auth, Operator+)

Convenience proxy to LuxShield `DELETE /api/v1/firewall/rules?ip=…`.

`400 { "error": "missing_ip" }` when `ip` is absent.

---

## Downstream integrations

ACC keeps a small connector registry (`luxshield`, `cyberflow`, `gritirt`,
`gritjira`) and can both **probe** and **proxy** to them. All proxy endpoints
relay the upstream HTTP status and JSON body (malformed upstream bodies are
wrapped as `{ "raw": "<text>" }`). An unreachable connector yields
`503 { "error": "upstream_unreachable", "connector": "…" }`; an unconfigured one
yields `503 { "error": "connector_not_configured", "connector": "…" }`.

When a connector key is configured on ACC, it is sent upstream as `X-API-Key`.

### `GET /api/v1/integrations` (auth)

Probes each connector's `/health`.

```json
{
  "integrations": [
    {
      "name": "luxshield",
      "url": "http://luxshield-backend:8020",
      "reachable": true,
      "status_code": 200,
      "latency_ms": 4,
      "health": { "service": "luxshield-backend", "status": "ok", … }
    }
  ]
}
```

### LuxShield proxy

| Method & path                                  | Upstream                                    |
|------------------------------------------------|---------------------------------------------|
| `GET /api/v1/luxshield/health` (auth)          | `GET /health`                               |
| `GET /api/v1/luxshield/stats` (auth)           | `GET /api/v1/network/stats`                 |
| `GET /api/v1/luxshield/firewall/rules` (auth)  | `GET /api/v1/firewall/rules`                |
| `POST /api/v1/luxshield/firewall/rules` (auth, Operator+) | `POST /api/v1/firewall/rules`    |
| `DELETE /api/v1/luxshield/firewall/rules?ip=` (auth, Operator+) | `DELETE /api/v1/firewall/rules` |

### CyberFlow proxy

| Method & path                          | Upstream                  |
|----------------------------------------|---------------------------|
| `GET /api/v1/cyberflow/health` (auth)  | `GET /health`             |
| `POST /api/v1/cyberflow/logs` (auth, Operator+)    | `POST /api/v1/logs`    |
| `POST /api/v1/cyberflow/threats` (auth, Operator+) | `POST /api/v1/threats` |

> CyberFlow exposes **no** GET threats-list endpoint, so ACC does not proxy one.
> For a live log tail, connect the frontend directly to
> `GET {ACC_CYBERFLOW_URL}/api/v1/stream` (SSE).

---

## MCP

The GritShield MCP surface is mounted at `POST /mcp` (JSON-RPC 2.0) and left
public. `MCP_HTTP_PREFIX` can relocate it.

---

## Error model

- `400` malformed JSON / missing or invalid fields
  (`invalid_json`, `missing_fields`, `invalid_id`, `invalid_role`, `weak_password`, `missing_slug`, `missing_ip`)
- `401` missing/invalid/expired JWT (`invalid_credentials`, or the middleware's 401)
- `403` authenticated but under-privileged (`forbidden` with `required_role`)
- `404` unknown id or slug (`not_found`)
- `409` uniqueness conflict (`username_taken`, `slug_taken`)
- `500` persistence/serialization failure
- `503` downstream connector unreachable or not configured

---

## Frontend quickstart

```js
// 1. login
const r = await fetch("http://127.0.0.1:8000/api/v1/auth/login", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ username: "admin", password: "admin123" }),
});
const { token } = await r.json();

// 2. authenticated call
const metrics = await fetch("http://127.0.0.1:8000/api/v1/system/metrics", {
  headers: { Authorization: `Bearer ${token}` },
}).then((x) => x.json());
```

Live telemetry from WASM: prefer the WebSocket `ws://127.0.0.1:8000/api/live-telemetry`
(no auth needed) or a fetch-based SSE reader on `/api/v1/system/stream`.
