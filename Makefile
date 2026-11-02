.PHONY: test lint format format-check typecheck qualite

test:
	cargo test --workspace

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
