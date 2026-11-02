.PHONY: up down dev test check

up:
	@bash develop/up.sh

down:
	@bash develop/down.sh

dev:
	@export DATABASE_MASTER_URL="postgres://postgres:postgres@127.0.0.1:5432/auth_db" && \
	 export REDIS_NODE_01="redis://127.0.0.1:6379" && \
	 export PORT=8080 && \
	 export APP_ENV=development && \
	 cargo run

test:
	@cargo test --all-features

check:
	@cargo clippy --all-features
