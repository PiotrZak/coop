# A2A auth + Negotiation peer flow

## What authentication does A2A use?

**Standard web AuthN on the HTTP edge** — not a custom A2A login message.

1. Server publishes schemes on the **Agent Card** (`security`).
2. Client gets credentials **out of band** (API key, OAuth2/OIDC, mTLS cert).
3. Every request sends credentials in headers (`Authorization: Bearer …`).
4. Server AuthN (`coop-identity`) → `Principal::authorize` in `coop-domain`.

Generic roles (`Reader` / `Contributor` / `Operator` / `Admin` / `Service`) each have a `max_level`. A skill is allowed when that level ≥ the skill’s required `RbacLevel`. See [`rbac.rs`](../crates/coop-domain/src/rbac.rs).

Coop demo uses shared Bearer API keys. Swap for OAuth2 tokens without changing skill payloads.

## Gateway-first (preferred)

1. Run Supplier (`:7102`) and Negotiation (`:7101`).
2. Run `coop-gateway` (`:7090`) — public AuthN/AuthZ + skill routing.
3. Clients call `http://127.0.0.1:7090` with a caller Bearer (`nego-a-demo-key`).
4. Gateway forwards with `gateway-service-key` to the upstream for that skill.

## Negotiation Agent → Negotiation Agent

Caller invokes skill `negotiate` (min level `Act`, e.g. role `Operator`) with ask + optional `supplier_id`.

## Negotiation consults Supplier

If `supplier_id` is set, Negotiation-B (as A2A **client**) calls Supplier skill `supplier.read` (min RBAC `Read`) with that id, then embeds the brief in the negotiation pack artifact. HITL remains required before any commercial send.
