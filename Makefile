.PHONY: fmt clippy test build crds e2e e2e-up e2e-down

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all-features

build:
	cargo build --all-features

crds:
	cargo run -- export-crds > deploy/crds/database.runvibe.dev.yaml

e2e:
	scripts/e2e.sh

e2e-up:
	docker compose -f docker-compose.e2e.yaml up --build

e2e-down:
	docker compose -f docker-compose.e2e.yaml down -v
