.PHONY: up down dev test check

RUN_WITH_ENV = @set -a; [ -f .env ] && . ./.env; set +a;

up:
	@bash develop/up.sh

down:
	@bash develop/down.sh

dev:
	$(RUN_WITH_ENV) cargo run

test:
	$(RUN_WITH_ENV) cargo test --all-features

check:
	$(RUN_WITH_ENV) cargo clippy --all-features
