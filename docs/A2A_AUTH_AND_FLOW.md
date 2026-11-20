# A2A auth + gateway flow

## AuthN / AuthZ / Trust

1. Agent Card advertises Bearer (and future OAuth2 / mTLS).
2. Caller sends `Authorization: Bearer …`.
3. `coop-identity` maps key → `Principal`.
4. Domain: RBAC scope check **and** identity [`TrustScore`](../crates/coop-domain/src/trust.rs) floor.
5. Gateway dispatches to Lindle Orleans or MCP.

## Flow

```text
foreign/demo agent --A2A--> coop-gateway
                              | AuthN → TrustScore → AuthZ
                              +--> Lindle /mcp        (supplier.read)
                              +--> Lindle /agents/*   (negotiate, investigate)
```

Demo: `coop-demo` → `http://127.0.0.1:7090` with `nego-a-demo-key`.
