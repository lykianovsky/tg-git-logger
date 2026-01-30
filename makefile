dev-build:
	cargo build

dev-run:
	cargo run

docker-dev-build:
	docker-compose --env-file .env.local build rust-dev

docker-dev-up:
	docker-compose --env-file .env.local up -d

docker-dev-down:
	docker-compose down

docker-dev-logs:
	docker-compose logs -f rust-dev

prod-build:
	cargo build --release

generate-database-entity:
	sea-orm-cli generate entity --database-url mysql://user:password@localhost/myapp --output-dir ./src/infrastructure/database/mysql/entities

prod-run:
	git pull && make prod-build && ./target/release/tg-bot-logger