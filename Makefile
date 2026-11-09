.DEFAULT_GOAL := test
CARGO_PUBLISH_PACKAGES := ic-entropy
CARGO_PUBLISH_REGISTRY := crates-io
CARGO_PUBLISH_VERIFY_TARGETS := release-verify
RELEASE_DELIVERY := direct
export RELEASE_DELIVERY
include make/consumer.mk

.PHONY: test lint check-wasm ci release-verify release-version release-preflight release-prepare-version release-prepared-check release-files release-commit-check release-committed-check release-tagged-check release-push-check release-editor-setup

test:
	cargo test --locked -p ic-entropy
lint:
	cargo clippy --locked -p ic-entropy --all-targets -- -D warnings
check-wasm:
	cargo check --locked -p ic-entropy --target wasm32-unknown-unknown
ci: toko-tooling-check
	$(MAKE) test lint check-wasm

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
	@printf 'Cargo.toml\0Cargo.lock\0CHANGELOG.md\0'
