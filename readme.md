# 🐝 SwarmSync

> **Distributed container orchestration made portable.**  
> Schedule, run, and manage containerized jobs at scale — now with a unified Rust CLI and TUI inspection tools.

![Version](https://img.shields.io/badge/version-v0.4.0-blue.svg)  
![Rust](https://img.shields.io/badge/Rust-safe--async--engine-orange.svg)  
![CLI](https://img.shields.io/badge/Client-CLI%20(consumer)-brightgreen.svg)  
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

## 🖥 Consumer CLI

The **`consumer`** is a Rust-powered CLI (no front-end framework) that runs in the Core container or standalone. It supports:

- **Job CRUD**: create, list, update, delete
- **Worker CRUD**: register, list, update, delete
- **Log CRUD**: record, query, paginate, update, delete
- **Inspect**: TUI for browsing Jobs or Core modules/logs

Launch it by:

```bash
# inside the core container
consumer main-menu
```

Or containerized:

```bash
docker run --rm -it swarmcore/consumer:latest consumer main-menu
```

---

## 🔍 Inspection Tools

Two TUI inspectors are available:

- **JobInspect**: visualize job / worker / assignment state in a live TUI graph
- **CoreInspect**: browse Core modules, see purpose & recent logs

Both launch from the `consumer` menu under “Inspect.”

---

## 📦 Job & Worker CRUD

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

## 🧭 Summary

SwarmSync offers a full end‑to‑end Rust stack:

1. **Core** orchestrates jobs and persistence
2. **Consumer CLI** submits and retrieves work
3. **TUI Inspectors** surface live state and logs

Everything is containerized for easy deployment and scaling.

---

## 🧪 Running SwarmSync (Local Dev)

### 1. Start Core & DB

```bash
docker compose up core
```

### 2. Enter `consumer` CLI

```bash
# In container or local CLI install
docker exec -it core-app consumer main-menu
```

### 3. Explore!

- Submit jobs, register workers, view logs
- Launch inspectors from `consumer` → Inspect
- No additional front-end setup required

Happy orchestrating! 🎉

