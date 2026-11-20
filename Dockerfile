# Coop A2A edge — default process: Negotiation Agent
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p coop-a2a --bin coop-negotiation --bin coop-supplier

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 coop

WORKDIR /app
COPY --from=build --chown=coop:coop /src/target/release/coop-negotiation /app/coop-negotiation
COPY --from=build --chown=coop:coop /src/target/release/coop-supplier /app/coop-supplier
COPY --chown=coop:coop agents /app/agents

ENV COOP_NEGO_ADDR=0.0.0.0:8080
ENV COOP_NEGO_NAME=negotiation
ENV RUST_LOG=info
EXPOSE 8080

USER coop

HEALTHCHECK --interval=30s --timeout=5s --retries=3 --start-period=10s \
  CMD curl -fsS http://127.0.0.1:8080/.well-known/agent.json || exit 1

ENTRYPOINT ["/app/coop-negotiation"]
