# A2A auth hub flow

## Model

Coop is an **auth hub**. Callers authenticate once; the hub routes to any **registered agent adapter** (AgentA, AgentB, Lindle Orleans, Lindle MCP, …). Lindle is not special — it is just adapters in the same registry.

```text
Caller  --A2A-->  coop-gateway (auth hub)
                    | AuthN (mTLS | OIDC/OAuth2 | shared secret)
                    | AuthZ + TrustScore
                    | route by params.target_agent (or skill priority)
                    +--> agent-a / agent-b / lindle-orleans / lindle-mcp / …
```

## AuthN / AuthZ / Trust

1. Agent Card advertises Bearer, and OAuth2 / OpenID Connect / mTLS when configured.
2. Gateway picks a proof (strongest first):
   - **mTLS** — proxy headers (`COOP_MTLS_PROXY_HEADERS=1`)
   - **OIDC / OAuth2** — Bearer JWT (`COOP_OIDC_ISSUER` + HS256 or JWKS)
   - **Shared secret** — demo Bearer / `X-API-Key`
3. Domain: RBAC + identity [`TrustScore`](../crates/coop-domain/src/trust.rs).
4. Hub dispatches to the chosen adapter.

## Routing

| Param | Behavior |
|-------|----------|
| `target_agent` | Select adapter by id (`lindle-orleans`, `agent-a`, …). Skill must be supported. |
| (omitted) | Pick lowest-priority adapter that supports `skill` (compat / fallback). |

## Env

| Variable | Role |
|----------|------|
| `AGENTS_SERVICE_URL` / `AGENTS_INTERNAL_SECRET` | Register Lindle Orleans adapter |
| `LINDLE_MCP_URL` / `MCP_SERVER_SECRET` | Register Lindle MCP adapter |
| `COOP_PEERS_JSON` | Register A2A peer adapters JSON array |
| `COOP_OIDC_ISSUER` (+ secret or JWKS) | Enable JWT AuthN |
| `COOP_MTLS_PROXY_HEADERS` | `1` — trust proxy mTLS headers |

Example peers:

```bash
export COOP_PEERS_JSON='[{"id":"agent-a","url":"http://127.0.0.1:7091","secret":"…","skills":["peer.message"]}]'
```

Demo: `coop-demo` → `http://127.0.0.1:7090` with `nego-a-demo-key` (skill route to Lindle adapter when registered).
