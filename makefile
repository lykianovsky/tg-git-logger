# ========================
# Local Dev
# ========================

dev-build:
	cargo build

dev-run:
	cargo run

# ========================
# Docker Dev
# ========================

docker-dev-build:
	docker-compose -f docker-compose.dev.yml --env-file .env.local build rust-dev

docker-dev-up:
	docker-compose -f docker-compose.dev.yml --env-file .env.local up -d

docker-dev-down:
	docker-compose -f docker-compose.dev.yml down

docker-dev-logs:
	docker-compose -f docker-compose.dev.yml logs -f rust-dev

# ========================
# Local Prod
# ========================

prod-build:
	cargo build --release

prod-run:
	git pull && make prod-build && ./target/release/tg-bot-logger

# ========================
# Docker Prod
# ========================

# Сборка Rust-приложения
docker-prod-build:
	docker-compose -f docker-compose.prod.yml --env-file .env.local build rust-app

docker-prod-up:
	docker-compose -f docker-compose.prod.yml --env-file .env.local up -d

docker-prod-down:
	docker-compose -f docker-compose.prod.yml --env-file .env.local down -d

docker-prod-logs:
	docker-compose -f docker-compose.prod.yml --env-file .env.local logs -f rust-app

docker-prod-restart:
	docker-compose -f docker-compose.prod.yml --env-file .env.local up -d --build

# ========================
# Monitoring (Prometheus + Grafana + Loki + Promtail)
# ========================

monitoring-up:
	docker-compose -f docker-compose.monitoring.yml up -d

monitoring-down:
	docker-compose -f docker-compose.monitoring.yml down

monitoring-restart:
	docker-compose -f docker-compose.monitoring.yml up -d --force-recreate

monitoring-logs:
	docker-compose -f docker-compose.monitoring.yml logs -f

monitoring-logs-grafana:
	docker-compose -f docker-compose.monitoring.yml logs -f grafana

monitoring-logs-prometheus:
	docker-compose -f docker-compose.monitoring.yml logs -f prometheus

monitoring-logs-loki:
	docker-compose -f docker-compose.monitoring.yml logs -f loki

# ========================
# Utilities
# ========================

generate-database-entity:
	sea-orm-cli generate entity --database-url mysql://user:password@localhost/myapp --output-dir ./src/infrastructure/database/mysql/entities
