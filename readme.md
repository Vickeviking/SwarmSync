# 🐝 SwarmSync

> **Distributed container orchestration made portable.**  
> Schedule, run, and manage containerized jobs at scale — now with a unified Rust CLI and TUI inspection tools.

![Version](https://img.shields.io/badge/version-v0.4.0-blue.svg)  
![Rust](https://img.shields.io/badge/Rust-safe--async--engine-orange.svg)  
![CLI](<https://img.shields.io/badge/Client-CLI%20(consumer)-brightgreen.svg>)  
![Docker](https://img.shields.io/badge/Containerized-yes-blue.svg)

---

## 📖 Table of Contents

- [🚀 Overview](#-swarmsync-overview)
- [🧠 Core Architecture](#-core-architecture)
- [⚙️ Core Engine Runtime](#-core-engine-runtime)
- [🌐 Rocket API & Auth](#-rocket-api--authentication)
- [🖥 Consumer CLI](#-consumer-cli)
- [🔍 Inspection Tools](#-inspection-tools)
- [📦 Job & Worker CRUD](#-job--worker--log-crud)
- [⏱️ Scheduled Jobs: Hibernator](#-scheduled-execution-with-hibernator)
- [🧱 Worker Nodes](#-worker-nodes)
- [🔐 Security Model](#-security--flexibility)
- [🧭 Summary](#-summary)
- [🧪 Running SwarmSync (Local Dev)](#-running-swarmsync-local-dev)

---

## 🚀 SwarmSync Overview

**SwarmSync** is a modular, high-performance distributed system for orchestrating containerized workloads across clusters. It now provides:

- 🧠 **Async orchestration** in Rust
- 🔁 **Intelligent queuing & scheduling**
- 🎛 **Rust CLI (`consumer`)** for job submission & retrieval
- 🔍 **TUI inspectors** for Jobs and Core modules
- 🐳 **Docker-native execution** and persistence via PostgreSQL + Redis

## 🛠 Built With

- **Rust** + Tokio for high-performance async
- **Rocket** for HTTP API
- **Diesel** ORM + **PostgreSQL** for persistent state
- **Redis** for ephemeral sessions & queue metadata
- **Argon2** for secure password hashing
- **Docker** & **Docker Compose** for containerization
- **tui** + **crossterm** for terminal UIs
- **chrono** for date/time handling
- **anyhow** for ergonomic error handling
- **dialoguer** for interactive CLI prompts

Built for observability, extensibility, and ease of use.

---

## 🧠 Core Architecture

The **Core** backend ties everything together:

- 🚀 `Rocket` HTTP API for integration
- 🧠 Internal async engine built on `tokio`
- 🗃 PostgreSQL (persistent job & log store)
- 🔄 Redis (ephemeral sessions, queue metadata)

Runs fully containerized via Docker Compose, deployable on any Docker host.

---

## ⚙️ Core Engine Runtime

Rust + `tokio` powers a set of decoupled services:

- **`service_channels`** – broadcast/messaging backbone
- **`service_handles`** – lifecycle management of tasks
- **`shared_resource`** – shared `Arc<Mutex<>>` state
- **`pulse_system`** – centralized heartbeat (fast/med/slow)

Modules like `Scheduler`, `Dispatcher`, `Hibernator`, `Harvester`, and `Logger` run concurrently, coordinating via these primitives.

---

## 🌐 Rocket API & Authentication

The `Rocket` HTTP API offers:

- Job submission, state transitions, result retrieval
- Worker registration and heartbeats
- Log querying and stats

**Auth**:

- Token-based sessions in Redis
- User records in PostgreSQL
- Planned PKI challenge/response for CLI

---

## 🖥 CommandDeck

The **`commanddeck`** is a Rust-powered CLI (no front-end framework) that runs in the Core container or standalone. It supports:

- **Job CRUD**: create, list, update, delete
- **Worker CRUD**: register, list, update, delete
- **Log CRUD**: record, query, paginate, update, delete
- **Inspect**: TUI for browsing Jobs or Core modules/logs

### 🔍 Inspection Tools

Two TUI inspectors are available:

- **JobInspect**: visualize job / worker / assignment state in a live TUI graph
- **CoreInspect**: browse Core modules, see purpose & recent logs

Both launch from the `consumer` menu under “Inspect.”

### 📦 Job & Worker CRUD

Through the `consumer` CLI you can:

- Submit new Docker‐based jobs
- List and filter by user, state, schedule
- Assign jobs to workers, track status
- Register and manage worker nodes
- Query and paginate system logs

Fully scriptable and automatable via shell.

---

## ⏱️ Scheduled Execution with Hibernator

The **`Hibernator`** module handles:

- ⏰ **Delayed jobs** — run at specific times
- 🔁 **Recurring jobs** — cron‐style schedules

Schedules are stored in PostgreSQL and triggered by a persistent time-pulse.

---

## 🧱 Worker Nodes

Workers are stateless executors:

- 🐳 Pull & run Docker images
- 🌍 Poll Core for assignments
- 📥 Return output & logs to Core
- 🚀 Scale horizontally on any Docker host

---

## 🔐 Security & Flexibility

- ✅ Redis‐backed token auth
- 🔏 SHA checksums for payload integrity
- 👥 Multi‐user isolation via DB roles
- 🌍 Public or private Core deployments
- 🖥 CLI is stateless and containerized

---

## Consumer

A a Rust-powered CLI (no front-end framework), used for users to log in on a local
or remote device to interact with the core. supports

- Job submission
- Job inspection
- Finnished job retrieval

---

## 🧪 Local Dev Quickstart

Get everything up and running in two stages: Core → Consumer.

1. Spin up the Core

# Move into the core service

cd core

# 1.1 Start containers (Postgres, Redis, Core)

docker compose up -d

# 1.2 Run database migrations

docker compose exec app diesel migration run

# 1.3 Launch the CoreNode API

docker compose exec app cargo run --bin core

# 1.4 (Optional) Launch the CommandDeck CLI

docker compose exec app cargo run --bin commanddeck

# 1.5 Run Core tests

docker compose exec app cargo test 2. Launch the Consumer CLI

# Move into the consumer service

cd ../consumer

# 2.1 Start a long-running consumer container

docker compose up -d consumer

# 2.2 Execute your interactive CLI

docker compose exec consumer cargo run

# 2.3 Run Consumer tests

docker compose exec consumer cargo test
