# 🐝 SwarmSync

> **Distributed container orchestration made portable.**  
> Schedule, run, and manage containerized jobs at scale — all with modular services and a plug-and-play AdminPanel.

![Version](https://img.shields.io/badge/version-v0.3.0-blue.svg)  
![Rust](https://img.shields.io/badge/Rust-safe--async--engine-orange.svg)  
![Vue](https://img.shields.io/badge/AdminPanel-Vue3%20+%20Vite-green.svg)  
![Docker](https://img.shields.io/badge/Containerized-yes-blue.svg)

---

## 📖 Table of Contents

- [🚀 Overview](#-swarmsync-overview)
- [🧠 Core Architecture](#-core-architecture)
- [⚙️ Core Engine Runtime](#-core-engine-runtime)
- [🌐 Rocket API & Auth](#-rocket-api--authentication)
- [🖥 AdminPanel](#-adminpanel-client-interface)
- [📦 Job Upload Workflow](#-consumer-workflow-via-adminpanel)
- [⏱️ Scheduled Jobs: Hibernator](#-scheduled-execution-with-hibernator)
- [🧱 Worker Nodes](#-worker-nodes)
- [🔐 Security Model](#-security--flexibility)
- [🧭 Summary](#-summary)
- [🧪 Running SwarmSync (Local Dev)](#-running-swarmsync-local-dev)

---

## 🚀 SwarmSync Overview

**SwarmSync** is a modular, high-performance distributed system for orchestrating containerized workloads across local or remote clusters. It offers:

- 🧠 **Async task orchestration** in Rust
- 🔁 **Intelligent queuing & scheduling**
- 🔐 **Token-based authentication**
- 🖥️ **Web-based AdminPanel** for live control
- 🐳 **Docker-native job execution**
- 🔄 **Delayed & recurring tasks** via persistent schedules

Built for observability, extensibility, and collaboration.

---

## 🧠 Core Architecture

The [**Core**](https://www.notion.so/Core-1d014b38fe39801abf0afd74f96d4f35?pvs=21) is the heart of SwarmSync. It is composed of:

- 🚀 `Rocket` HTTP API for external interaction
- 🧠 Internal async engine for orchestration logic
- 🧩 Redis for ephemeral state (sessions, tokens)
- 🗃 PostgreSQL + Diesel ORM for persistence

Core runs fully containerized (via Docker Compose) and is deployable on any Docker-compatible host.

---

## ⚙️ Core Engine Runtime

The engine is written in Rust using `tokio`. Key systems:

- **`service_channels`** – Async broadcast/messaging backbone
- **`service_handles`** – Lifecycle tracking of tasks & services
- **`shared_resource`** – Global `Arc<Mutex<>>` state sharing
- **`pulse_system`** – Central heartbeat/timing pulse (fast/med/slow)

Internal modules like `Scheduler`, `Dispatcher`, `Hibernator`, `Harvester`, and `Logger` are all decoupled services coordinated through these systems.

---

## 🌐 Rocket API & Authentication

The `Rocket`-based HTTP API provides:

- Job submission, scheduling, result fetching
- Admin login and session issuance
- Live system and queue status

**Auth**:

- Redis-based session tokens for ephemeral auth
- PostgreSQL for user records, roles, and metadata
- Public key challenge/response (planned)

---

## 🖥 AdminPanel (Client Interface)

The AdminPanel is a standalone Vue 3 + Vite web app with Vuetify. It provides:

- 🧠 Real-time dashboard: Core health, worker telemetry
- 📦 Job uploads: Docker metadata, compose setups
- 🔄 Result tracking: Pull archived output as `.txt`
- 🔐 Token-based login with persistent sessions (planned)
- ⚙️ Config via browser storage or optional JSON

Not containerized — launch with `npm run dev` and access via browser at `http://localhost:5173`.

---

## 📦 Consumer Workflow (via AdminPanel)

1. 🛠 Admin sets up a Core instance on a remote or local node.
2. 🔐 Logs into Core via AdminPanel.
3. 📤 Uploads job metadata (Docker image URL, flags, etc).
4. 🧪 Core schedules and dispatches job to available Workers.
5. 📥 Admin monitors job and downloads results from Archive.

No shared LAN or complex network setup required — only public Core access.

---

## ⏱️ Scheduled Execution with Hibernator

The `Hibernator` module handles:

- ⏰ **Delayed jobs** — run at a set time
- 🔁 **Recurring jobs** — e.g., daily, weekly
- 💾 Stored in PostgreSQL for persistence

Admins define schedules via AdminPanel. Triggers are handled by internal time-pulse logic with full control.

---

## 🧱 Worker Nodes

Workers are platform-agnostic executors:

- 🐳 Docker containers that poll Core for jobs
- 🌍 Run locally or remotely
- ⬇️ Fetch public Docker images directly
- 🧪 Execute isolated containers
- 📤 Return output to Core

Lightweight and stateless by design — any machine that supports Docker can become a Worker.

---

## 🔐 Security & Flexibility

- ✅ Token-based auth via Redis
- 🔏 SHA checksums for verifying job payloads
- 👥 Multi-admin support with PostgreSQL roles
- 🌍 Public or private Core deployments
- 📡 AdminPanel is stateless and browser-based

---

## 🧭 Summary

SwarmSync delivers a modular, async-first system for distributed container execution. With its decoupled design, AdminPanel-based UX, and Docker-native pipeline, it enables scalable workload orchestration in everything from dev environments to production-grade multi-node clusters.

---

## 🧪 Running SwarmSync (Local Dev)

To test SwarmSync locally during development, follow these steps:

### 1. Start the Core container

Spin up the Core backend using Docker Compose:

```bash
docker compose up core
```

This launches:

- 🧠 Core (Rust backend)
- 🗃 PostgreSQL (persistent job DB)
- 🔄 Redis (ephemeral token/session store)
  The Core API becomes available at http://localhost:8000 once Rocket is fully booted.

---

### 2. Launch the AdminPanel

Navigate into the frontend directory and start the local dev server:

```bash
cd adminpanel
npm install         # First time only
npm run dev
```

---

### 3. Connect AdminPanel to Core

If Core is on a remote host:

Go to Settings → Core Address
Enter your Core’s URL (e.g., http://192.168.1.42:8000)
Save and reconnect
