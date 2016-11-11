all: test target/release/request_log_analyzer

target/release/request_log_analyzer: src/ musl-deps
	cargo build --release --verbose --target=x86_64-unknown-linux-musl

test:
	cargo test --verbose

musl-deps:
	rustup target add x86_64-unknown-linux-musl

.PHONY: all test musl-deps
