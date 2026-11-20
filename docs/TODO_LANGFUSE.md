# TODO — Langfuse observability (not implemented)

Intent only. No SDK, env vars, or spans in Coop yet.

## Why

Trace foreign-agent A2A calls through Coop next to Lindle/YCA Langfuse usage:

`AuthN → TrustScore → skill → Lindle MCP | Orleans`

## Instrument later

| Span / attribute | Notes |
|------------------|--------|
| `a2a.message_send` | gateway `/a2a` |
| `trust.band` | `low` / `medium` / `high` |
| `skill` | e.g. `negotiate`, `supplier.read` |
| `backend.kind` | `lindle` (Orleans) or `lindle_mcp` |
| `agent_id` / `tenant_id` | from `Principal` — no secrets |
| upstream latency / HTTP errors | MCP `tools/call` or `/agents/*` |

## Non-goals (v0)

- No Langfuse dependency in `Cargo.toml`
- No `LANGFUSE_*` env on coop-gateway
- Align with Lindle/YCA Langfuse patterns when implementing
