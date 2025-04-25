# 🐝 SwarmSync

> **Distributed container orchestration made portable.**  
> Schedule, run, and manage containerized jobs at scale — now with a unified Rust CLI and TUI inspection tools.

[![Version](https://img.shields.io/badge/version-v0.4.0-blue.svg)]()
[![Rust](https://img.shields.io/badge/Rust-safe--async--engine-orange.svg)]()
[![CLI](<https://img.shields.io/badge/Client-CLI%20(consumer)-brightgreen.svg>)]()
[![Docker](https://img.shields.io/badge/Containerized-yes-blue.svg)]()

---

## 🏁 Purpose

SwarmSync provides a lightweight, extensible platform for scheduling and orchestrating Docker-based workloads. It aims to deliver:

- **Portable deployment** via Docker Compose
- **High-performance** async runtime in Rust
- **Unified interfaces**: HTTP API, CLI, and TUI
- **Robust scheduling**: immediate, delayed, and recurring jobs

---

## 🏗️ Architecture & Components

SwarmSync consists of three primary components. Detailed design docs live under `docs/`.

### 1. Core Module

A containerized Rust service binding together HTTP, engine, and CLI glue.

- **Rocket Server**: exposes REST endpoints for job and worker management
- **Core Engine**: async scheduler, dispatcher, and heartbeat systems
- **CommandDeck**: built-in CLI for administrative tasks and TUI inspection

_See [`docs/overview.md`](docs/overview.md#core-module) for details._

### 2. Consumer CLI

A standalone Rust-based CLI for end users to submit, query, and manage jobs.

- Job submission and status retrieval
- Interactive prompts (Argon2‑secured credentials)
- TUI inspectors for live feedback

_See [`docs/overview.md`](docs/overview.md#consumer-cli) for details._

### 3. Worker Nodes

Stateless executors that poll Core for assignments and run container workloads.

- Docker image pull & execution
- Heartbeat, log streaming, and result upload
- Horizontal scaling on any Docker host

_See [`docs/overview.md`](docs/overview.md#worker-nodes) for details._

---

## 📂 Documentation Layout

All long-form documentation is in `docs/`:

```
docs/
├── overview.md           ← High-level architecture & component summaries
├── modules/              ← Internal subsystem deep dives
└── services/             ← Shared service docs (logger, wiring, auth)
```

Use the conventions defined in [`docs/DOCS_CONVENTIONS.md`](docs/DOCS_CONVENTIONS.md) when adding or editing docs.

---

## ⚙️ Quickstart (Local Development)

### 1. Core Module

```bash
# Enter core folder
eval "$(direnv export bash)" && cd core

# Start dependencies + core service
docker compose up -d

# Run migrations
docker compose exec app diesel migration run

# Launch API & engine
docker compose exec app cargo run --bin core
```

### 2. CommandDeck (Optional)

```bash
docker compose exec app cargo run --bin commanddeck
```

### 3. Consumer CLI

```bash
# Move to consumer
dev cd ../consumer

# Start consumer container
docker compose up -d consumer

# Launch interactive CLI
docker compose exec consumer cargo run
```

---

For detailed instructions, see the [`docs/`](docs/overview.md) folder.
