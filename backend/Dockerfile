# Admin Command Center backend image: GritShield (crates.io) + PostgreSQL.
#
# Binds from ACC_BIND/ACC_PORT (see src/main.rs) and serves the JSON REST API
# described in ../api.md. Schema migrations are applied at boot by GritShield's
# built-in DbManager, which scans ./migrations relative to the working dir.
#
# Build context is this backend/ directory:
#   docker build -t acc .
FROM rust:1-bookworm AS builder
RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates libssl3 curl \
  && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/admin-command-center /usr/local/bin/admin-command-center
# The built-in migrator reads ./migrations relative to the working directory.
COPY migrations ./migrations
ENV ACC_BIND=0.0.0.0 \
    ACC_PORT=8000
EXPOSE 8000
HEALTHCHECK --interval=10s --timeout=3s --retries=5 \
    CMD curl -fsS http://127.0.0.1:8000/health >/dev/null || exit 1
CMD ["admin-command-center"]
