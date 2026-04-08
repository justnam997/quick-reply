.PHONY: build release fmt lint test check clean

build:
	cargo build

release:
	cargo build --release

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

lint:
	cargo clippy -- -D warnings

test:
	cargo test

check: fmt-check lint test build
	@echo "All checks passed"

clean:
	cargo clean

run:
	cargo run
