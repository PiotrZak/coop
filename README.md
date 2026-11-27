# Coop

A2A **auth hub**: agents authenticate here, then Coop routes to any registered adapter (AgentA, AgentB, Lindle Orleans/MCP, …).

```
Agent A ──┐
Agent B ──┼──► coop-gateway ──► adapters (by target_agent or skill)
Agent N ──┘         │ AuthN + trust + RBAC
```

More detail: [docs/ORCHESTRATOR.md](docs/ORCHESTRATOR.md) · [docs/A2A_AUTH_AND_FLOW.md](docs/A2A_AUTH_AND_FLOW.md)

## Run

```bash
export AGENTS_SERVICE_URL=http://127.0.0.1:8080
export AGENTS_INTERNAL_SECRET=…   # never commit
# optional: LINDLE_MCP_URL + MCP_SERVER_SECRET
# optional: COOP_PEERS_JSON='[{"id":"agent-a","url":"http://127.0.0.1:7091","secret":"…"}]'

cargo run -p coop-a2a --bin coop-gateway   # http://127.0.0.1:7090
cargo run -p coop-a2a --bin coop-demo
```

Demo keys: `nego-a-demo-key` (Operator), `nego-b-demo-key` (Admin), `supplier-demo-key` (Reader).

## Communication samples

All calls: `POST /a2a` with `Authorization: Bearer <key>` (or JWT / mTLS headers).

### Skill route (no target — hub picks by priority)

```bash
curl -s http://127.0.0.1:7090/a2a \
  -H 'Authorization: Bearer nego-a-demo-key' \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": "1",
    "method": "message/send",
    "params": {
      "skill": "negotiate",
      "message": {
        "role": "user",
        "parts": [{
          "kind": "data",
          "data": {
            "counterparty": "Acme Cloud sales",
            "category": "SaaS",
            "ask": "Renewal down 10%",
            "supplier_id": "acme-cloud"
          }
        }]
      }
    }
  }'
```

### Pin an adapter (`target_agent`)

```bash
curl -s http://127.0.0.1:7090/a2a \
  -H 'Authorization: Bearer nego-a-demo-key' \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": "2",
    "method": "message/send",
    "params": {
      "skill": "negotiate",
      "target_agent": "lindle-orleans",
      "message": {
        "role": "user",
        "parts": [{ "kind": "text", "text": "run negotiate on Orleans adapter" }]
      }
    }
  }'
```

### Agent → agent (registered peer)

```bash
curl -s http://127.0.0.1:7090/a2a \
  -H 'Authorization: Bearer nego-b-demo-key' \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": "3",
    "method": "message/send",
    "params": {
      "skill": "peer.message",
      "target_agent": "agent-a",
      "message": {
        "role": "user",
        "parts": [{ "kind": "text", "text": "hello from negotiation-b" }]
      }
    }
  }'
```

### Trust on the response

Successful results include `coop_trust`:

```json
{
  "coop_trust": {
    "band": "medium",
    "proof": "shared_secret",
    "agent_id": "negotiation-a",
    "tenant_id": "buyer-tenant",
    "target_agent": "lindle-orleans"
  }
}
```

### Agent card

```bash
curl -s http://127.0.0.1:7090/.well-known/agent.json
```

## Skills (min trust)

| Skill | Trust | Typical `target_agent` |
|-------|-------|------------------------|
| `supplier.read` | Low | `lindle-mcp` / `lindle-orleans` |
| `negotiate` / `investigate` | Medium | `lindle-orleans` |
| `peer.message` | Medium | peer id from `COOP_PEERS_JSON` |

## Crates

| Crate | Role |
|-------|------|
| `coop-domain` | RBAC + TrustScore |
| `coop-application` | Agent registry config |
| `coop-identity` | AuthN (secret / OAuth2 / OIDC / mTLS) |
| `coop-a2a` | Hub gateway + adapters |
| `coop-tests` | Tests |

Apache-2.0
