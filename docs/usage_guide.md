# SwarmSync Usage Guide

This guide walks you through setting up and using the unified SwarmSync development environment, powered by a single `docker-compose.yml` and a convenient `Makefile`.

---

## Prerequisites

- Docker & Docker Compose installed on your machine (Ubuntu 20.04+ recommended)
- GNU Make
- Git (to clone this repo)

---

## 1. Clean Slate (Optional)

If you have old Docker artifacts from previous setups, run:

```bash
docker compose down --volumes --remove-orphans
docker system prune --all --volumes --force
```

---

## 2. Build Everything

Compile the Rust code and build Docker images for both Core and Consumer:

```bash
make build
```

---

## 3. Launch Core Module

The Core service includes Rocket HTTP API, scheduler engine, Postgres, Redis, and automatic DB backups.

1. **Start Core + dependencies:**

   ```bash
   make up-core
   ```

2. **Run database migrations:**

   ```bash
   make migrate
   ```

3. **Launch the Core server:**

   ```bash
   make run-core
   ```

4. (Optional) **Open the CommandDeck TUI:**

   ```bash
   make run-commanddeck
   ```

- Core API is exposed at `http://localhost:8000`
- Automatic DB snapshots are saved every 10 minutes in `backups/`

---

## 4. Launch Consumer CLI

Use the standalone Consumer container to submit jobs and query status:

1. **Start Consumer container:**

   ```bash
   make up-consumer
   ```

2. **Run the interactive CLI:**

   ```bash
   make run-consumer
   ```

- By default, the CLI connects to the local Core at `core:8000` on the Compose network.
- If you point to a remote Core, configure `CORE_API_URL` in `consumer/consumer_config.json`.

---

## 5. Bring Up Both Modules

To run Core (with DB/Redis) and Consumer together:

```bash
make up-all
```

---

## 6. Stop Everything

```bash
make down
```

- This stops all containers but **preserves** your database volume and backups.
- To fully wipe volumes, run `docker compose down -v`.

---

## 7. Database Snapshots

- **Manual backup:**

  ```bash
  make db-save
  ```

  → Creates `backups/manual_backup.sql`

- **Manual restore:**

  ```bash
  make db-load
  ```

  → Loads `backups/manual_backup.sql` into `app_db`

- **Automatic backups:**  
  Every 10 minutes via the `db_backup` service; SQL dumps appear in `backups/`.

---

## 8. Code Quality & Testing

- **Lint & format:**

  ```bash
  make clippy   # enforce Rust lints
  make fmt      # auto-format code
  ```

- **Security audit:**

  ```bash
  make audit   # check deps for vulnerabilities
  ```

- **Run tests:**
  ```bash
  make test
  ```

---

## 9. Tips & Best Practices

- Volume names are auto-prefixed by Compose, so multiple clones on one machine won’t conflict.
- You can re-run `make run-core` or `make run-consumer` as you iterate on code—live mounts pick up changes.
- Add any extra `Makefile` targets for profiling, fuzzing, or custom tasks as needed.

---
