# --- builder stage ---
FROM rust:1.70 AS builder
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY swarm-worker-common swarm-worker-common
COPY swarm-worker-tui   swarm-worker-tui

WORKDIR /usr/src/app/swarm-worker-tui
RUN cargo build --release --bin swarm-worker-tui

# --- runtime stage ---
FROM debian:bullseye-slim
RUN apt-get update \
  && apt-get install -y ca-certificates \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/swarm-worker-tui/target/release/swarm-worker-tui /usr/local/bin/swarm-worker-tui

COPY swarm-worker-common/worker_config.json /etc/swarm/worker_config.json

ENTRYPOINT ["swarm-worker-tui", "--config", "/etc/swarm/worker_config.json"]
