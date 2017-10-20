all: test target/release/request_log_analyzer

target/release/request_log_analyzer: src/
	cargo build --release --verbose

test:
	cargo test --verbose

test-no-run:
	RUSTFLAGS='-C link-dead-code' cargo test --no-run

.SECONDARY:

perf: src/test/random-small.log src/test/random-big.log target/perf/v1.2.0.csv target/perf/v1.3.0.csv target/perf/v1.4.1.csv target/perf/v2.0.1.csv target/perf/master.csv
	cat target/perf/*.csv > target/perf/all

target/perf/%.csv: target/release/archive/%
	mkdir -p target/perf
	src/test/perf_test_binary $< src/test/random-small.log > $@
	src/test/perf_test_binary $< src/test/request.log.2016-04-06-Pub2-fixed >> $@
	src/test/perf_test_binary $< src/test/random-big.log >> $@

target/release/archive/%:
	git checkout $(shell basename $@)
	cargo build --release
	git checkout master
	cp target/release/request_log_analyzer $@

coverage: test-no-run
# Example file name: request_log_analyzer-751ef51155a898c3
# We are interested in the test binaries with a hash postfix
# In this case, we simply look for the dash that separates the postfix...
	$(eval LATEST_TEST_BINARY := $(shell ls -t1 `find target/debug -maxdepth 1 -type f -executable -name "*-*"` | head -1))
# http://sunjay.ca/2016/07/25/rust-code-coverage
	kcov --coveralls-id=$(COVERALLS_ID) \
		--exclude-pattern=/.cargo,/usr/lib,tests \
		--exclude-region="cfg(test):LCOV_EXCL_STOP" \
		--verify \
		target/cov $(LATEST_TEST_BINARY)

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

release: committedworkingdir
	sed -i 's/version = ".*"/version = "$(VERSION)"/' Cargo.toml
	cargo test
	git commit --all -m "Bump version for release $(VERSION)"

	git tag "v$(VERSION)" --annotate --message="Release $(VERSION)"

committedworkingdir:
	# Fail if there are uncommitted changes
	git diff-index --quiet HEAD

.PHONY: all test coverage test-no-run perf musl-deps musl release committedworkingdir
