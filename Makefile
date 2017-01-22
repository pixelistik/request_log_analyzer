all: test target/release/request_log_analyzer

target/release/request_log_analyzer: src/
	cargo build --release --verbose

test:
	cargo test --verbose

test-no-run:
	cargo test --no-run

perf: src/test/random-small.log src/test/random-big.log
	src/test/perf_test

target/perf/%.csv: target/release/archive/%
	mkdir -p target/perf
	src/test/perf_test_binary $< src/test/request.log.2016-04-06-Pub2-fixed > $@

target/release/archive/%:
	git checkout $(shell basename $@)
	cargo build --release
	cp target/release/request_log_analyzer $@

coverage: test-no-run
# Example file name: request_log_analyzer-751ef51155a898c3
# We are interested in the test binaries with a hash postfix
# In this case, we simply look for the dash that separates the postfix...
	$(eval LATEST_TEST_BINARY := $(shell ls target/debug/*-* -t1 | head -1))
# http://sunjay.ca/2016/07/25/rust-code-coverage
	kcov --exclude-pattern=/.cargo,/usr/lib --verify target/cov $(LATEST_TEST_BINARY)

src/test/random-small.log:
	python src/test/generate_random_log.py 1000 > src/test/random-small.log

src/test/random-big.log:
	python src/test/generate_random_log.py 600000 > src/test/random-big.log

musl-deps:
	rustup target add x86_64-unknown-linux-musl

musl: target/x86_64-unknown-linux-musl/release/request_log_analyzer_portable_musl

target/x86_64-unknown-linux-musl/release/request_log_analyzer: musl-deps test src/
	cargo build --release --verbose --target=x86_64-unknown-linux-musl

target/x86_64-unknown-linux-musl/release/request_log_analyzer_portable_musl: target/x86_64-unknown-linux-musl/release/request_log_analyzer
	cp target/x86_64-unknown-linux-musl/release/request_log_analyzer target/x86_64-unknown-linux-musl/release/request_log_analyzer_portable_musl

.PHONY: all test coverage test-no-run perf musl-deps musl
