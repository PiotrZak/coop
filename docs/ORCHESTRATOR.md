# Coop as A2A edge over Lindle

**Purpose:** foreign agents talk **A2A** to Coop; Coop calls **Lindle Orleans** and/or **Lindle MCP**.  
YCA construction tools are out of scope.

```text
Foreign A2A agent  --A2A-->  coop-gateway  --+--> Lindle /agents/*  (Orleans)
                                             +--> Lindle /mcp         (tools/call)
```

## Env

| Variable | Required | Role |
|----------|----------|------|
| `AGENTS_SERVICE_URL` | for Orleans skills | Orleans Agents base URL |
| `AGENTS_INTERNAL_SECRET` | with URL | Bearer / `X-Lindle-Agents-Secret` |
| `LINDLE_MCP_URL` | optional | Lindle Backend MCP URL (e.g. `http://127.0.0.1:8000/mcp`) |
| `MCP_SERVER_SECRET` | with MCP URL | Same secret YCA uses for `/mcp` |

Listen: `127.0.0.1:7090` (`--addr` to override).

## Skill map

| A2A skill | Backend | Min trust |
|-----------|---------|-----------|
| `supplier.read` | **MCP** `supplier-lookup` (if MCP up), else Orleans portfolio | Low |
| `investigate` | Orleans `POST /agents/investigate` | Medium |
| `negotiate` | Orleans `POST /agents/negotiate` | Medium |

## Trust score

After AuthN, Coop computes an identity-based [`TrustScore`](../crates/coop-domain/src/trust.rs) (`Untrusted`/`Low`/`Medium`/`High`) from role + proof strength, then enforces a per-skill floor. Attached on responses as `coop_trust`.

## Observability

Langfuse: planned — see [TODO_LANGFUSE.md](./TODO_LANGFUSE.md) (doc only).
