all: test target/release/request_log_analyzer

target/release/request_log_analyzer: src/
	cargo build --release --verbose

test:
	cargo test --verbose

.PHONY: all test
