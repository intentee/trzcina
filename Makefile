CARGO_HOME ?= $(HOME)/.cargo
HEISENBERG_ARGUMENTS = --cargo-home "$(CARGO_HOME)" --sysroot "$(shell rustc --print sysroot)" --workspace "$(CURDIR)"
RUST_SOURCES = $(wildcard */src/*.rs */tests/*.rs)
WORKSPACE_MANIFESTS = Cargo.lock Cargo.toml $(wildcard */Cargo.toml)

target/lcov.info: target/llvm-cov.json
	cargo llvm-cov report --lcov --include-build-script --output-path $@

target/llvm-cov.json: $(RUST_SOURCES) $(WORKSPACE_MANIFESTS) .config/nextest.toml rust-toolchain.toml
	cargo llvm-cov nextest --workspace --lib --bins --tests --examples --json --include-build-script --output-path $@

target/llvm-cov/html/index.html: target/llvm-cov.json
	cargo llvm-cov report --html --include-build-script

.PHONY: clean
clean:
	rm -rf target

.PHONY: clippy
clippy:
	cargo clippy --workspace --all-targets

.PHONY: coverage
coverage: target/lcov.info target/llvm-cov.json
	cargo llvm-cov report --include-build-script
	heisenberg coverage-check $(HEISENBERG_ARGUMENTS) --configuration x86_64-unknown-linux-gnu --coverage-export "$(CURDIR)/target/llvm-cov.json"

.PHONY: coverage-clean
coverage-clean:
	cargo llvm-cov clean --workspace
	rm -rf target/llvm-cov-target
	rm -f target/llvm-cov.json target/lcov.info

.PHONY: coverage-report
coverage-report: target/llvm-cov/html/index.html

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: heisenberg
heisenberg:
	heisenberg check $(HEISENBERG_ARGUMENTS) --configuration x86_64-pc-windows-msvc --configuration x86_64-unknown-linux-gnu

.PHONY: rust-test
rust-test:
	cargo nextest run --workspace

.PHONY: test
test: rust-test
