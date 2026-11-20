# Coop as agent orchestrator

Backends share generic `AgentBackendConfig`. **Lindle is one entity** of that family.

## Env (minimum)

| Variable | When |
|----------|------|
| `AGENTS_SERVICE_URL` | Lindle Orleans base URL — if set, Lindle is primary + required |
| `AGENTS_INTERNAL_SECRET` | Same secret as Lindle Host (`Bearer` / `X-Lindle-Agents-Secret`) |

No other Coop env vars are required. Listen address defaults to `127.0.0.1:7090` (`--addr` to override).

```bash
export AGENTS_SERVICE_URL=http://127.0.0.1:8080
export AGENTS_INTERNAL_SECRET=...   # from Lindle — never commit
cargo run -p coop-a2a --bin coop-gateway
```

Local-only (no Lindle): unset both vars, start local bins, then gateway.

## Skill map → Lindle

| A2A skill | Lindle endpoint |
|-----------|-----------------|
| `negotiate` | `POST /agents/negotiate` |
| `supplier.read` | `POST /agents/optimize-portfolio` |
| `investigate` | `POST /agents/investigate` |
