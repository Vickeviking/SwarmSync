###############################################################################
# Makefile with help and interactive menu
###############################################################################

# Detect current Makefile
FILE := $(firstword $(MAKEFILE_LIST))

.PHONY: build up-core up-commanddeck up-consumer up-all down migrate db-save db-load db_store docker-clean \
        test run-core run-consumer run-worker-tui run-worker-tui-rebuild build-worker rebuild-worker run-commanddeck help menu

# Build everything                         ## build: build core, commanddeck, consumer, and swarm-worker
build: ## Build core, commanddeck, consumer, and swarm-worker
	docker compose build core consumer swarm-worker

# Start Core (and DB backup)               ## up-core: start core and db_backup
up-core: ## Start Core service and DB backup
	@echo "Starting Core + DB backup..."
	docker compose up -d core db_backup


# Start Consumer CLI                       ## up-consumer: start consumer CLI
up-consumer: ## Start Consumer CLI
	@echo "Starting Consumer CLI..."
	docker compose up -d consumer

# Start Swarm Worker container             ## up-worker: start Swarm Worker service
up-worker: ## Start Swarm Worker service
	@echo "Starting Swarm Worker..."
	docker compose up -d swarm-worker

# Start everything                         ## up-all: start Core, CommandDeck, Consumer, Worker, and dependencies
up-all: ## Start all services
	@echo "Starting Core, Consumer, Worker, and dependencies..."
	docker compose up -d core db_backup consumer swarm-worker

# Stop all containers                      ## down: stop all SwarmSync containers
down: ## Stop all SwarmSync containers
	@echo "Stopping all SwarmSync containers..."
	@docker compose down

# Migrate DB                                ## migrate: run database migrations
migrate: up-core ## Run database migrations (requires core up)
	@echo "Running database migrations..."
	@docker compose exec -T core diesel migration run

# Save DB to SQL dump                       ## db-save: dump DB to backups/manual_backup.sql
db-save: ## Dump DB to backups/manual_backup.sql
	@echo "Dumping DB to backups/manual_backup.sql..."
	@docker compose exec -T postgres pg_dump -U postgres app_db > backups/manual_backup.sql

# Alias for db-save                         ## db_store: alias for db-save
db_store: db-save

# Load DB from SQL dump                     ## db-load: restore DB from backups/manual_backup.sql
db-load: ## Restore DB from backups/manual_backup.sql
	@echo "Restoring DB from backups/manual_backup.sql..."
	@cat backups/manual_backup.sql | docker compose exec -T postgres psql -U postgres -d app_db

# Run tests                                 ## test: run tests in Core and Consumer
test: ## Run tests in Core and Consumer
	@echo "Running tests in Core and Consumer..."
	@docker compose exec core cargo test
	@docker compose exec consumer cargo test

# Convenience run targets                   ## run-core: run core service
run-core: up-core migrate ## Launch Core service
	@echo "Launching Core service..."
	@docker compose exec -T core cargo run --bin core-api

run-commanddeck: up-core migrate ## Launch Core service
	@echo "Launching Core service..."
	@docker compose exec core cargo run --bin commanddeck

# Launch Consumer CLI                        ## run-consumer: run consumer CLI
run-consumer: up-consumer ## Launch Consumer CLI
	@echo "Launching Consumer CLI..."
	@docker compose exec consumer cargo run

# Rebuild (no cache) and restart worker only ## rebuild-worker: rebuild swarm-worker & restart
rebuild-worker: ## Rebuild swarm-worker & restart it
	docker compose build --no-cache swarm-worker
	docker compose up -d --no-deps swarm-worker

# Build only the worker image                ## build-worker: build just swarm-worker
build-worker: ## Build just swarm-worker
	docker compose build swarm-worker

# Launch Swarm Worker TUI                    ## run-worker-tui: launch Swarm Worker TUI
run-worker-tui: up-worker ## Launch the TUI (with backtraces enabled)
	@echo "Launching Swarm Worker TUI…"
	@docker compose exec \
	  -e RUST_BACKTRACE=1 \
	  swarm-worker \
	  swarm-worker-tui
# Docker-clean                             ## docker-clean: snapshot DB, stop & prune Docker
docker-clean: db_store down ## Snapshot DB, stop services & prune Docker
	@echo "Pruning Docker system (containers, images, volumes...)"
	@docker system prune -af --volumes

# Help                                      ## help: show help message
help: ## Show help for all Makefile targets
	@echo "Usage: make [target]"
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z0-9_-]+:.*?##' $(FILE) \
		| sort \
		| awk -F ':.*?## ' '{printf "  %-20s %s\n", $$1, $$2}'

# Interactive menu                          ## menu: choose a target interactively
menu: help ## Interactive menu to select a target
	@echo ""
	@echo "Select a target to run:"
	@grep -E '^[a-zA-Z0-9_-]+:.*?##' $(FILE) \
		| sort \
		| awk -F ':.*?## ' '{print NR ") " $$1 " - " $$2}' \
		| tee /dev/tty \
		| { read -p "Enter number: " num </dev/tty; \
		    echo ""; \
		    cmd=$$(grep -E '^[a-zA-Z0-9_-]+:.*?##' $(FILE) \
		           | sort \
		           | sed -n "$$num p" \
		           | awk -F ':.*?## ' '{print $$1}'); \
		    echo "Running $$cmd..."; \
		    make $$cmd; \
		  }





