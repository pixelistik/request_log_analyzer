all: test target/release/request_log_analyzer

target/release/request_log_analyzer: src/
	cargo build --release --verbose

test:
	cargo test --verbose

musl-deps:
	rustup target add x86_64-unknown-linux-musl

musl: target/x86_64-unknown-linux-musl/release/request_log_analyzer_portable_musl

target/x86_64-unknown-linux-musl/release/request_log_analyzer: musl-deps test src/
	cargo build --release --verbose --target=x86_64-unknown-linux-musl

target/x86_64-unknown-linux-musl/release/request_log_analyzer_portable_musl: target/x86_64-unknown-linux-musl/release/request_log_analyzer
	cp target/x86_64-unknown-linux-musl/release/request_log_analyzer target/x86_64-unknown-linux-musl/release/request_log_analyzer_portable_musl

.PHONY: all test musl-deps musl
