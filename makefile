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

docker-prod-build:
	docker-compose -f docker-compose.prod.yml --env-file .env.local build rust-app

docker-prod-up:
	docker-compose -f docker-compose.prod.yml --env-file .env.local up -d

docker-prod-down:
	docker-compose -f docker-compose.prod.yml down

docker-prod-logs:
	docker-compose -f docker-compose.prod.yml logs -f rust-app

# ========================
# Utilities
# ========================

generate-database-entity:
	sea-orm-cli generate entity --database-url mysql://user:password@localhost/myapp --output-dir ./src/infrastructure/database/mysql/entities
