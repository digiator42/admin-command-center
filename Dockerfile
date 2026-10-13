# Admin Command Center image: GritShield (crates.io) + PostgreSQL.
#
# The service binds from ACC_BIND/ACC_PORT (see src/main.rs) and serves the
# JSON REST API described in api.md. Migrations are embedded at compile time
# via sqlx::migrate!.
#
# Build context is this repo (no sibling path deps needed):
#   docker build -t acc .
FROM rust:1-bookworm AS builder
RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates libssl3 curl \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /src/target/release/admin-command-center /usr/local/bin/admin-command-center
# File-based routes + static assets are read from the working directory at runtime.
COPY src ./src
COPY static ./static
ENV ACC_BIND=0.0.0.0 \
    ACC_PORT=8000
EXPOSE 8000
HEALTHCHECK --interval=10s --timeout=3s --retries=5 \
    CMD curl -fsS http://127.0.0.1:8000/health >/dev/null || exit 1
CMD ["admin-command-center"]
