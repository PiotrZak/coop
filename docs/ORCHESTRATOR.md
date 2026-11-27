# Coop as A2A auth hub

**Purpose:** foreign agents authenticate to Coop; Coop routes to **any registered agent adapter**.  
Lindle Orleans / MCP are adapters in the same registry as AgentA / AgentB.

```text
Foreign A2A agent  --A2A-->  coop-gateway  --+--> agent adapters (A2A peers)
                                             +--> Lindle Orleans / MCP (also adapters)
```

## Env

| Variable | Required | Role |
|----------|----------|------|
| `AGENTS_SERVICE_URL` | for Orleans adapter | Orleans Agents base URL |
| `AGENTS_INTERNAL_SECRET` | with URL | Bearer / `X-Lindle-Agents-Secret` |
| `LINDLE_MCP_URL` | optional | Lindle Backend MCP URL |
| `MCP_SERVER_SECRET` | with MCP URL | MCP shared secret |
| `COOP_PEERS_JSON` | optional | A2A peer adapters JSON array |
| `COOP_OIDC_ISSUER` | optional | Enable OAuth2/OIDC JWT AuthN |
| `COOP_OIDC_AUDIENCE` | optional | JWT audience (default `coop-gateway`) |
| `COOP_OIDC_HS256_SECRET` | optional | HS256 secret for local JWT |
| `COOP_OIDC_JWKS_URL` | optional | JWKS URL override |
| `COOP_MTLS_PROXY_HEADERS` | optional | `1` to accept proxy mTLS headers |

Listen: `127.0.0.1:7090` (`--addr` to override).

## Skill map (example adapters)

| A2A skill | Typical adapter id | Min trust |
|-----------|--------------------|-----------|
| `supplier.read` | `lindle-mcp` (priority) or `lindle-orleans` | Low |
| `investigate` | `lindle-orleans` | Medium |
| `negotiate` | `lindle-orleans` | Medium |
| `peer.message` | any `A2aPeer` | Medium |

Pass `params.target_agent` to pin an adapter (e.g. `"lindle-orleans"` or `"agent-a"`).

## Trust score

After AuthN, Coop computes [`TrustScore`](../crates/coop-domain/src/trust.rs) from role + proof, then enforces a per-skill floor. Responses include `coop_trust` (`band`, `proof`, `target_agent`). See [A2A_AUTH_AND_FLOW.md](./A2A_AUTH_AND_FLOW.md).

## Observability

Langfuse: planned — see [TODO_LANGFUSE.md](./TODO_LANGFUSE.md) (doc only).
