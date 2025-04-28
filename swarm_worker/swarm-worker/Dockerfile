# --- builder stage ---
FROM rust:1.70 AS builder
WORKDIR /usr/src/app

# Copy just the manifest first for faster rebuilds
COPY Cargo.toml Cargo.lock ./

# Copy common crate and this binary
COPY swarm-worker-common swarm-worker-common
COPY swarm-worker       swarm-worker

WORKDIR /usr/src/app/swarm-worker
RUN cargo build --release --bin swarm-worker

# --- runtime stage ---
FROM debian:bullseye-slim
RUN apt-get update \
  && apt-get install -y ca-certificates curl \
  && rm -rf /var/lib/apt/lists/*

# Copy the built binary
COPY --from=builder /usr/src/app/swarm-worker/target/release/swarm-worker /usr/local/bin/swarm-worker

# Default config (override with bind-mount in Compose)
COPY swarm-worker-common/worker_config.json /etc/swarm/worker_config.json

ENTRYPOINT ["swarm-worker", "--config", "/etc/swarm/worker_config.json"]
