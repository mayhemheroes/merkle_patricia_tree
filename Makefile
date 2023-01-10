.PHONY: build check clippy test bench

build:
	cargo build --release

check:
	cargo check --all-targets

clippy:
	cargo clippy --all-targets -- -D warnings

test:
	cargo test

bench:
	cargo bench
