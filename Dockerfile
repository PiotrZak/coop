# Coop A2A gateway — Lindle Orleans edge
FROM rust:1-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p coop-a2a --bin coop-gateway

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 10001 coop

WORKDIR /app
COPY --from=build --chown=coop:coop /src/target/release/coop-gateway /app/coop-gateway

ENV RUST_LOG=info
EXPOSE 8080

USER coop

HEALTHCHECK --interval=30s --timeout=5s --retries=3 --start-period=10s \
  CMD curl -fsS http://127.0.0.1:8080/health || exit 1

ENTRYPOINT ["/app/coop-gateway"]
CMD ["--addr", "0.0.0.0:8080"]
