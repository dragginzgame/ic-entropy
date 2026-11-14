.DEFAULT_GOAL := test
CARGO_PUBLISH_PACKAGES := ic-entropy ic-entropy-ic
CARGO_PUBLISH_REGISTRY := crates-io
CARGO_PUBLISH_VERIFY_TARGETS := publication-verify
RELEASE_DELIVERY := direct
TOKO_TOOLING_RELEASE := 1
FORMAT_RUST_ARGS := --extra-workspace qualification --extra-manifest-order package,lib,features,dependencies,dev-dependencies,target
LOCAL_TOOL_INSTALL_TARGETS := release-editor-setup qualification-server-setup
LOCAL_TOOL_CHECK_TARGETS := release-editor-check qualification-server-check
export RELEASE_DELIVERY
include vendor/toko-tooling/make/consumer.mk
include vendor/toko-tooling/make/cargo-dependencies.mk

.PHONY: test lint check-wasm ci release-verify release-version release-preflight release-prepare-version release-prepared-check release-files release-commit-check release-committed-check release-tagged-check release-push-check release-editor-setup

test:
	cargo test --locked -p ic-entropy
	cargo test --locked -p ic-entropy-ic
	cargo test --locked -p ic-entropy-ic --features buffered
lint:
	CARGO_BUILD_WARNINGS=deny cargo clippy --locked -p ic-entropy --all-targets
	CARGO_BUILD_WARNINGS=deny cargo clippy --locked -p ic-entropy-ic --all-targets
	CARGO_BUILD_WARNINGS=deny cargo clippy --locked -p ic-entropy-ic --all-targets --features buffered
check-wasm:
	CARGO_BUILD_WARNINGS=deny cargo clippy --locked -p ic-entropy --target wasm32-unknown-unknown
	CARGO_BUILD_WARNINGS=deny cargo clippy --locked -p ic-entropy-ic --target wasm32-unknown-unknown
	CARGO_BUILD_WARNINGS=deny cargo clippy --locked -p ic-entropy-ic --features buffered --target wasm32-unknown-unknown
	CARGO_BUILD_WARNINGS=deny cargo clippy --locked -p ic-entropy-ic --features buffered --example consumer_owned --target wasm32-unknown-unknown
.PHONY: doc-check
doc-check:
	RUSTDOCFLAGS="-D warnings" cargo doc --locked -p ic-entropy -p ic-entropy-ic --no-deps
	RUSTDOCFLAGS="-D warnings" cargo doc --locked -p ic-entropy-ic --features buffered --no-deps
	RUSTDOCFLAGS="-D warnings" cargo doc --locked -p ic-entropy -p ic-entropy-ic --target wasm32-unknown-unknown --no-deps
	RUSTDOCFLAGS="-D warnings" cargo doc --locked -p ic-entropy-ic --features buffered --target wasm32-unknown-unknown --no-deps
ci: toko-tooling-check
	$(MAKE) lint test check-wasm doc-check qualification-check test-installed test-release-metadata

release-editor-setup:
	bash vendor/toko-tooling/scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-build --version 0.73.0 --bin ic-build-release-metadata --profile debug --features cli
release-version:
	@bash scripts/release/adapter.sh version
release-preflight:
	@bash scripts/release/adapter.sh preflight
release-verify:
	@bash scripts/release/adapter.sh verify
.PHONY: publication-verify
publication-verify:
	@bash scripts/release/adapter.sh publication-verify
release-prepare-version:
	@bash scripts/release/adapter.sh prepare
release-prepared-check release-tagged-check release-push-check:
	@bash scripts/release/adapter.sh check
release-committed-check:
	@bash scripts/release/adapter.sh committed-check
release-commit-check:
	@bash scripts/release/adapter.sh commit-check
release-files:
	@printf 'Cargo.toml\0Cargo.lock\0CHANGELOG.md\0qualification/Cargo.lock\0'

# Installed qualification stays outside the product dependency graph.
QUALIFICATION_TARGET := $(CURDIR)/target/qualification
.PHONY: install-qualification-tools qualification-check build-entropy-probe test-installed
install-qualification-tools:
	@server="$$(bash vendor/toko-tooling/scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-testkit --version 0.104.0 --bin ic-testkit-server --profile debug --features cli)" && \
	  bash scripts/dev/setup-pocket-ic.sh "$$server" "$(CURDIR)/.tools/pocket-ic"
qualification-check:
	cargo fmt --manifest-path qualification/Cargo.toml --all --check
	CARGO_BUILD_WARNINGS=deny cargo clippy --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --all-targets
	CARGO_BUILD_WARNINGS=deny cargo clippy --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --lib --target wasm32-unknown-unknown
	RUSTDOCFLAGS="-D warnings" cargo doc --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --no-deps
	RUSTDOCFLAGS="-D warnings" cargo doc --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --target wasm32-unknown-unknown --no-deps
build-entropy-probe:
	cargo build --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --lib --release --target wasm32-unknown-unknown
test-installed: build-entropy-probe
	cargo test --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --test installed --no-run
	@server="$$(bash vendor/toko-tooling/scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-testkit --version 0.104.0 --bin ic-testkit-server --profile debug --features cli --check)" && \
	  ENTROPY_PROBE_WASM="$(QUALIFICATION_TARGET)/wasm32-unknown-unknown/release/entropy_probe.wasm" \
	  "$$server" run --directory "$(CURDIR)/.tools/pocket-ic" -- \
	  cargo test --locked --manifest-path qualification/Cargo.toml --target-dir "$(QUALIFICATION_TARGET)" -p entropy-probe --test installed -- --nocapture

.PHONY: test-release-metadata
test-release-metadata:
	bash scripts/release/test-metadata.sh

.PHONY: release-editor-check qualification-tools-check qualification-server-setup qualification-server-check
qualification-server-setup:
	@bash vendor/toko-tooling/scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-testkit --version 0.104.0 --bin ic-testkit-server --profile debug --features cli
qualification-server-check:
	@bash vendor/toko-tooling/scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-testkit --version 0.104.0 --bin ic-testkit-server --profile debug --features cli --check > /dev/null
release-editor-check:
	@bash vendor/toko-tooling/scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-build --version 0.73.0 --bin ic-build-release-metadata --profile debug --features cli --check > /dev/null
qualification-tools-check:
	@server="$$(bash vendor/toko-tooling/scripts/dev/install-rust-tools.sh --consumer "$(CURDIR)" --package ic-testkit --version 0.104.0 --bin ic-testkit-server --profile debug --features cli --check)" && \
	  "$$server" check --directory "$(CURDIR)/.tools/pocket-ic" > /dev/null
