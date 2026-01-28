dev-build:
	cargo build

dev-run:
	cargo run

prod-build:
	cargo build --release

prod-run:
	git pull && make prod-build && ./target/release/tg-bot-logger