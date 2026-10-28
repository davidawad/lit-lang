# syntax=docker/dockerfile:1

# litgraph — litigation procedure graphs as stochastic games.
# Multi-stage build: cargo-chef caches dependency compilation separately
# from source changes, then a minimal non-root runtime image ships the
# one `litigate` binary beside the lit/ and cases/ library it compiles.

ARG RUST_VERSION=1.97.1

# --- planner: compute the cargo-chef recipe (dependency graph only) --------
FROM rust:${RUST_VERSION}-slim-bookworm AS chef
WORKDIR /build
RUN cargo install cargo-chef --locked
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY editors editors
RUN cargo chef prepare --recipe-path recipe.json

# --- builder: build deps from the recipe (cached), then the real source ----
FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY editors editors
# The hand-authored clock library; crates/litgraph-packs builds the packs,
# calibration sets and named scenarios from lit/ and cases/ at compile time.
COPY packs packs
COPY lit lit
COPY cases cases
# litigate-cli embeds the language reference.
COPY docs/litigate/SPEC.md docs/litigate/SPEC.md
COPY sources sources
RUN cargo build --release -p litigate-cli && \
    strip target/release/litigate

# --- runtime: minimal non-root image ---------------------------------------
FROM debian:bookworm-slim AS runtime

ARG BUILD_DATE
ARG VCS_REF

LABEL org.opencontainers.image.title="litgraph" \
      org.opencontainers.image.description="Litigation procedure graphs as stochastic games: load, compose, weight, solve, simulate." \
      org.opencontainers.image.source="https://github.com/davidawad/litgraph" \
      org.opencontainers.image.licenses="GPL-3.0-or-later" \
      org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.revision="${VCS_REF}"

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    useradd --system --no-create-home --uid 10001 litgraph

COPY --from=builder /build/target/release/litigate /usr/local/bin/litigate
# `litigate` compiles its library from lit/ and cases/ under the working
# directory (e.g. `litigate solve lit/cofc`, and `litigate mcp` resources).
WORKDIR /srv/litgraph
COPY lit lit
COPY cases cases
# The library is read-only to the runtime user: verify into /tmp.
ENV LITIGATE_CACHE=/tmp/litigate

USER litgraph

ENTRYPOINT ["litigate"]
CMD ["describe"]

# `docker run --rm -i <image> mcp` serves the stdio MCP server.
