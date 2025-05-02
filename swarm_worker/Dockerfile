# Stage 1: Build
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install CA certs and OpenSSL 3 so libssl.so.3 is available
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
  ca-certificates \
  libssl3 \
  && rm -rf /var/lib/apt/lists/*

# Prepare a place for the config
RUN mkdir -p /worker/config

# Bake in the *default* config so that a named volume will initialize from this
COPY swarm-worker-common/worker_config.json /worker/config/worker_config.json

# Copy the binaries
COPY --from=builder /app/target/release/swarm-worker /usr/local/bin/swarm-worker
COPY --from=builder /app/target/release/swarm-worker-tui /usr/local/bin/swarm-worker-tui

# Start the TUI (which will spawn the worker process internally)
ENTRYPOINT ["swarm-worker-tui"]

