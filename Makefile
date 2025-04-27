# Top-level Makefile for SwarmSync

# Default target: show help
.PHONY: help
help:
	@echo "SwarmSync Makefile - available targets:"
	@awk '/^[a-zA-Z\-\_]+:/{print "  " $$1}' $(MAKEFILE_LIST)

# Build Docker images for core and consumer
.PHONY: build
build:
	docker compose build core consumer

# Run services
.PHONY: up-core up-consumer up-all down
up-core:
	@echo "Starting Core module (and dependencies)..."
	docker compose up -d core db_backup  # core depends on postgres & redis; include backup
up-consumer:
	@echo "Starting Consumer CLI module..."
	docker compose up -d consumer
up-all:
	@echo "Starting Core + Consumer + dependencies..."
	docker compose up -d core db_backup consumer
down:
	@echo "Stopping all SwarmSync containers..."
	docker compose down

# Database migrations (run Diesel migrations inside core container)
.PHONY: migrate
migrate: up-core   # ensure core (and DB) is up
	@echo "Running database migrations..."
	docker compose exec core diesel migration run

# Manual database backup and restore
.PHONY: db-save db-load
db-save: 
	@echo "Saving database snapshot to backups/manual_backup.sql..."
	# Dump the 'app_db' database to a SQL file
	docker compose exec -T postgres pg_dump -U postgres app_db > backups/manual_backup.sql
	@echo "Database saved to backups/manual_backup.sql"
db-load: 
	@echo "Restoring database from backups/manual_backup.sql..."
	# Restore the database from the SQL dump
	cat backups/manual_backup.sql | docker compose exec -T postgres psql -U postgres -d app_db
	@echo "Database restored from backups/manual_backup.sql"

# Run tests in both modules
.PHONY: test
test: 
	@echo "Running cargo test in core/ and consumer/..."
	docker compose exec core cargo test
	docker compose exec consumer cargo test

# Run/attach to the main binaries (for convenience)
.PHONY: run-core run-consumer run-commanddeck
run-core: up-core migrate
	@echo "Launching Core service (API & engine)..."
	docker compose exec core cargo run --bin core
run-consumer: up-consumer 
	@echo "Launching Consumer CLI (interactive)..."
	docker compose exec consumer cargo run
run-commanddeck: up-core 
	@echo "Launching CommandDeck TUI (core admin CLI)..."
	docker compose exec core cargo run --bin commanddeck
