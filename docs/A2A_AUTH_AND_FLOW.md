# A2A auth + Negotiation peer flow

## What authentication does A2A use?

**Standard web AuthN on the HTTP edge** — not a custom A2A login message.

1. Server publishes schemes on the **Agent Card** (`security`).
2. Client gets credentials **out of band** (API key, OAuth2/OIDC, mTLS cert).
3. Every request sends credentials in headers (`Authorization: Bearer …`).
4. Server validates → identity (`Principal`) → **skill AuthZ** (`negotiate`, `supplier.read`).

Coop demo uses shared Bearer API keys. Swap for OAuth2 tokens without changing skill payloads.

## Negotiation Agent → Negotiation Agent

Caller Negotiation-A invokes peer skill `negotiate` with ask + optional `supplier_id`.

## Negotiation consults Supplier

If `supplier_id` is set, Negotiation-B (as A2A **client**) calls Supplier skill `supplier.read` with that id, then embeds the brief in the negotiation pack artifact. HITL remains required before any commercial send.
