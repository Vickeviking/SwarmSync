# Build everything
.PHONY: build
build:
	docker compose build core consumer swarm-worker swarm-worker-tui

# Start Core (and DB backup)
.PHONY: up-core
up-core:
	@echo "Starting Core + DB backup..."
	docker compose up -d core db_backup

# Start Consumer CLI
.PHONY: up-consumer
up-consumer:
	@echo "Starting Consumer CLI..."
	docker compose up -d consumer

# Start Worker
.PHONY: up-worker
up-worker:
	@echo "Starting Swarm Worker (headless)..."
	docker compose up -d swarm-worker

# Run Worker-TUI interactively
.PHONY: up-worker-tui
up-worker-tui:
	@echo "Launching Swarm Worker TUI..."
	docker compose run --rm swarm-worker-tui

# Start everything
.PHONY: up-all
up-all:
	@echo "Starting Core, Consumer, Worker, and dependencies..."
	docker compose up -d core db_backup consumer swarm-worker

# Stop all containers
.PHONY: down
down:
	@echo "Stopping all SwarmSync containers..."
	docker compose down

# Migrate DB
.PHONY: migrate
migrate: up-core
	@echo "Running database migrations..."
	docker compose exec core diesel migration run

# Save/restore DB
.PHONY: db-save db-load
db-save:
	@echo "Dumping DB to backups/manual_backup.sql..."
	docker compose exec -T postgres pg_dump -U postgres app_db > backups/manual_backup.sql
db-load:
	@echo "Restoring DB from backups/manual_backup.sql..."
	cat backups/manual_backup.sql | docker compose exec -T postgres psql -U postgres -d app_db

# Test both Core and Consumer
.PHONY: test
test:
	@echo "Running tests in Core and Consumer..."
	docker compose exec core cargo test
	docker compose exec consumer cargo test

# Convenience run targets
.PHONY: run-core run-consumer run-worker run-worker-tui run-commanddeck
run-core: up-core migrate
	@echo "Launching Core service..."
	docker compose exec core cargo run --bin core

run-consumer: up-consumer
	@echo "Launching Consumer CLI..."
	docker compose exec consumer cargo run

run-worker: up-core up-worker
	@echo "Launching Swarm Worker (headless)..."
	docker compose exec swarm-worker swarm-worker --config /worker/worker_config.json

run-worker-tui: up-core up-worker
	@echo "Launching Swarm Worker TUI..."
	docker compose exec swarm-worker-tui swarm-worker-tui --config /tui/worker_config.json

run-commanddeck: up-core
	@echo "Launching CommandDeck TUI..."
	docker compose exec core cargo run --bin commanddeck

