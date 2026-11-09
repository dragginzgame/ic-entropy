#!/usr/bin/env bash
set -euo pipefail
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
[[ $# == 1 ]] || exit 2
case "$1" in
    version)
        exec bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml
        ;;
    preflight)
        bash scripts/ci/check-release-source.sh
        make --no-print-directory install-host-tools install-rust-tools release-editor-setup
        cargo fetch --locked
        bash scripts/ci/check-release-changelog.sh "${RELEASE_VERSION:?}" "${RELEASE_PREVIOUS:?}" "${RELEASE_DATE:?}" CHANGELOG.md
        ;;
    verify)
        make --no-print-directory toko-tooling-check
        bash scripts/ci/cargo-release-metadata.sh check-graph ic-entropy
        bash scripts/ci/check-dependency-pins.sh
        make --no-print-directory fmt-check lint test check-wasm
        ;;
    prepare)
        bash scripts/ci/check-release-source.sh --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md
        CARGO_RELEASE_METADATA_EDITOR="$(bash scripts/dev/install-rust-tools.sh --consumer "$PWD" --package ic-build --version 0.73.0 --bin ic-build-release-metadata --profile debug --features cli --check)"
        export CARGO_RELEASE_METADATA_EDITOR
        export CARGO_RELEASE_METADATA_MAX_BYTES=8388608
        exec bash scripts/ci/cargo-release-metadata.sh prepare ic-entropy
        ;;
    check|commit-check)
        if [[ "$1" == commit-check ]]; then
            bash scripts/ci/check-release-source.sh --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md
        fi
        exec bash scripts/ci/cargo-release-metadata.sh check ic-entropy --manifest-order-grouped
        ;;
    *) echo 'expected version|preflight|verify|prepare|check|commit-check' >&2; exit 2 ;;
esac
