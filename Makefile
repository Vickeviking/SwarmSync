###############################################################################
# SwarmSync Makefile — simplified & clarified
###############################################################################

# Variables
DC = docker compose
CORE = core
CONSUMER = consumer
WORKER = swarm-worker
DB = postgres
BACKUP = db_backup

# Detect this file for help target
FILE := $(firstword $(MAKEFILE_LIST))

.PHONY: help menu build up-core up-consumer up-worker up-all down \
        migrate db-save db-load db_store docker-clean \
        run-core run-commanddeck run-consumer run-worker-tui \
        build-worker rebuild-worker test-unit test-integration test-all

#### helper to require Core running (Option A) ####
define require_core
	@running=$$(docker inspect -f '{{.State.Running}}' swarmsync-core 2>/dev/null); \
	if [ "$$running" != "true" ]; then \
	  echo "✖ Core service is not running. Please run 'make up-core'."; \
	  exit 1; \
	fi
endef

#### Build & Up/Down ##########################################################
build: ## Build core, consumer & worker images
	@echo "▶ Building images..."
	$(DC) build $(CORE) $(CONSUMER) $(WORKER)

up-core: ## Start Core service + DB backup
	@echo "▶ Starting Core + DB backup..."
	$(DC) up -d $(CORE) $(BACKUP)

up-consumer: ## Start Consumer CLI container
	@echo "▶ Starting Consumer CLI..."
	$(DC) up -d $(CONSUMER)

up-worker: ## Start Swarm Worker container
	@echo "▶ Starting Worker..."
	$(DC) up -d $(WORKER)

up-all: ## Start Core, DB backup, Consumer & Worker
	@echo "▶ Starting all services..."
	$(DC) up -d $(CORE) $(BACKUP) $(CONSUMER) $(WORKER)

down: ## Stop all SwarmSync containers
	@echo "▶ Stopping all services..."
	@$(DC) down

docker-clean: down ## Snapshot DB, stop & prune Docker system
	@echo "▶ Pruning Docker system…"
	@docker system prune -af --volumes

#### DB tasks ###############################################################
migrate: up-core ## Run DB migrations (needs core up)
	@echo "▶ Applying migrations…"
	@$(DC) exec -T $(CORE) diesel migration run

db-save: ## Dump DB to backups/manual_backup.sql
	@echo "▶ Saving DB snapshot..."
	@$(DC) exec -T $(DB) pg_dump -U postgres app_db > backups/manual_backup.sql

db_store: db-save

db-load: ## Restore DB from backups/manual_backup.sql
	@echo "▶ Restoring DB snapshot..."
	@cat backups/manual_backup.sql | $(DC) exec -T $(DB) psql -U postgres -d app_db

#### Run targets #############################################################
run-core: ## Launch core-api binary
	$(require_core)
	@echo "▶ Running Core API…"
	@$(DC) exec -T $(CORE) cargo run --bin core-api

run-commanddeck: ## Launch commanddeck binary
	$(require_core)
	@$(MAKE) migrate
	@echo "▶ Running CommandDeck…"
	@$(DC) exec $(CORE) cargo run --bin commanddeck

run-consumer: ## Launch consumer CLI
	$(require_core)
	@echo "▶ Running Consumer CLI…"
	@$(DC) exec $(CONSUMER) cargo run

run-worker-tui: ## Launch Swarm Worker TUI
	$(require_core)
	@echo "▶ Running Worker TUI…"
	@$(DC) exec \
	  -e RUST_BACKTRACE=1 \
	  $(WORKER) \
	  swarm-worker-tui

build-worker: ## Build only the swarm-worker image
	@echo "▶ Building worker…"
	$(DC) build $(WORKER)

rebuild-worker: ## Rebuild swarm-worker (no cache) & restart
	@echo "▶ Rebuilding worker…"
	$(DC) build --no-cache $(WORKER)
	$(DC) up -d --no-deps $(WORKER)

#### Testing #################################################################
test-unit: ## Run unit tests in Core, Consumer & Worker
	$(require_core)
	@echo "▶ Core unit tests…"
	@$(DC) exec $(CORE)        cargo test -- --include-ignored
	@echo "▶ Consumer unit tests…"
	@$(DC) exec $(CONSUMER)    cargo test -- --include-ignored
	@echo "▶ Worker unit tests…"
	@$(DC) exec $(WORKER)      cargo test -- --include-ignored

test-integration: up-all migrate ## Run integration tests for every crate (host-side)
	$(require_core)
	@echo "▶ Core-API integration tests…"
	@cd swarm_core/core-api       && cargo test -- --include-ignored

	@echo "▶ CommandDeck integration tests…"
	@cd swarm_core/commanddeck    && cargo test -- --include-ignored

	@echo "▶ Consumer integration tests…"
	@cd swarm_consumer            && cargo -- --include-ignored

	@echo "▶ Swarm-Worker binary integration tests…"
	@cd swarm_worker/swarm-worker         && cargo test -- --include-ignored

	@echo "▶ Swarm-Worker-TUI integration tests…"
	@cd swarm_worker/swarm-worker-tui     && cargo test -- --include-ignored

test-all: test-unit test-integration ## Run all tests

#### Help & Menu #############################################################
help: ## Show this help
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z0-9_-]+:.*?##' $(FILE) \
	  | sort \
	  | awk -F ':.*?## ' '{printf "  %-20s %s\n", $$1, $$2}'

menu: help ## Interactive menu
	@echo ""
	@grep -E '^[a-zA-Z0-9_-]+:.*?##' $(FILE) \
	  | sort \
	  | awk -F ':.*?## ' '{print NR ") " $$1 " — " $$2}' \
	  | tee /dev/tty \
	  | { read -p "Choose a target: " num </dev/tty; \
	      cmd=$$(grep -E '^[a-zA-Z0-9_-]+:.*?##' $(FILE) \
	             | sort \
	             | sed -n "$$num p" \
	             | awk -F ':.*?## ' '{print $$1}'); \
	      echo "▶ Running $$cmd…"; \
	      make $$cmd; }





