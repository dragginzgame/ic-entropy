.DEFAULT_GOAL := test
CARGO_PUBLISH_PACKAGES := ic-entropy ic-entropy-ic
CARGO_PUBLISH_REGISTRY := crates-io
CARGO_PUBLISH_VERIFY_TARGETS := release-verify
RELEASE_DELIVERY := direct
TOKO_TOOLING_RELEASE := 1
FORMAT_RUST_ARGS := --extra-workspace qualification --extra-manifest-order package,lib,features,dependencies,dev-dependencies,target
export RELEASE_DELIVERY
include make/consumer.mk

.PHONY: test lint check-wasm ci release-verify release-version release-preflight release-prepare-version release-prepared-check release-files release-commit-check release-committed-check release-tagged-check release-push-check release-editor-setup

test:
	cargo test --locked -p ic-entropy
	cargo test --locked -p ic-entropy-ic
	cargo test --locked -p ic-entropy-ic --features buffered
lint:
	cargo clippy --locked -p ic-entropy --all-targets -- -D warnings
	cargo clippy --locked -p ic-entropy-ic --all-targets -- -D warnings
	cargo clippy --locked -p ic-entropy-ic --all-targets --features buffered -- -D warnings
check-wasm:
	cargo check --locked -p ic-entropy --target wasm32-unknown-unknown
	cargo check --locked -p ic-entropy-ic --target wasm32-unknown-unknown
	cargo check --locked -p ic-entropy-ic --features buffered --target wasm32-unknown-unknown
	cargo check --locked -p ic-entropy-ic --features buffered --example consumer_owned --target wasm32-unknown-unknown
ci: toko-tooling-check
	$(MAKE) lint test check-wasm qualification-check test-installed test-release-metadata

release-editor-setup:
	bash scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-build --version 0.73.0 --bin ic-build-release-metadata --profile debug --features cli
release-version:
	@bash scripts/release/adapter.sh version
release-preflight:
	@bash scripts/release/adapter.sh preflight
release-verify:
	@bash scripts/release/adapter.sh verify
release-prepare-version:
	@bash scripts/release/adapter.sh prepare
release-prepared-check release-committed-check release-tagged-check release-push-check:
	@bash scripts/release/adapter.sh check
release-commit-check:
	@bash scripts/release/adapter.sh commit-check
release-files:
	@printf 'Cargo.toml\0Cargo.lock\0CHANGELOG.md\0qualification/Cargo.lock\0'

# Installed qualification stays outside the product dependency graph.
QUALIFICATION_TARGET := $(CURDIR)/target/qualification
.PHONY: install-qualification-tools qualification-check build-entropy-probe test-installed
install-qualification-tools:
	@server="$$(bash scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-testkit --version 0.96.0 --bin ic-testkit-server --profile debug --features cli)" && \
	  "$$server" setup --directory "$(CURDIR)/.tools/pocket-ic"
qualification-check:
	cargo fmt --manifest-path qualification/Cargo.toml --all --check
	cargo clippy --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --all-targets -- -D warnings
	cargo clippy --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --lib --target wasm32-unknown-unknown -- -D warnings
build-entropy-probe:
	cargo build --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --lib --release --target wasm32-unknown-unknown
test-installed: build-entropy-probe
	cargo test --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --test installed --no-run
	@server="$$(bash scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-testkit --version 0.96.0 --bin ic-testkit-server --profile debug --features cli --check)" && \
	  ENTROPY_PROBE_WASM="$(QUALIFICATION_TARGET)/wasm32-unknown-unknown/release/entropy_probe.wasm" \
	  "$$server" run --directory "$(CURDIR)/.tools/pocket-ic" -- \
	  cargo test --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --test installed -- --nocapture

.PHONY: test-release-metadata
test-release-metadata:
	bash scripts/release/test-metadata.sh
