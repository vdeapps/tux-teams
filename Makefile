.PHONY: run test test-integration lint format format-check typecheck qualite

run:
	cargo run -p tuxteams

test:
	cargo test --workspace

# Tests d'intégration qui exigent un Secret Service de session (D-Bus) ; hors `make test`.
test-integration:
	cargo test -p tuxteams-auth --features integration --test coffre

lint:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings

format:
	cargo fmt --all

format-check:
	cargo fmt --all -- --check

typecheck:
	cargo check --workspace --all-targets

qualite: test lint format-check typecheck
