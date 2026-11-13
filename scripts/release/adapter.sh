#!/usr/bin/env bash
set -euo pipefail
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
[[ $# == 1 ]] || exit 2
case "$1" in
    version)
        exec bash vendor/toko-tooling/scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml
        ;;
    preflight)
        bash vendor/toko-tooling/scripts/ci/check-release-source.sh
        make --no-print-directory install-host-tools install-rust-tools release-editor-setup install-qualification-tools
        cargo fetch --locked
        cargo fetch --locked --manifest-path qualification/Cargo.toml
        bash vendor/toko-tooling/scripts/ci/check-release-changelog.sh "${RELEASE_VERSION:?}" "${RELEASE_PREVIOUS:?}" "${RELEASE_DATE:?}" CHANGELOG.md
        ;;
    verify|publication-verify)
        make --no-print-directory toko-tooling-check
        bash vendor/toko-tooling/scripts/ci/cargo-release-metadata.sh check-graph ic-entropy
        bash vendor/toko-tooling/scripts/ci/check-dependency-pins.sh
        make --no-print-directory fmt-check
        current="$(bash vendor/toko-tooling/scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
        if [[ "$1" == verify && -n "${RELEASE_VERSION:-}" && "$current" != "$RELEASE_VERSION" ]]; then
            echo 'Native validation deferred to the committed final-version source.'
        else
            bash scripts/release/native-stage.sh
        fi
        ;;
    prepare)
        bash vendor/toko-tooling/scripts/ci/check-release-source.sh --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md --allow qualification/Cargo.lock
        CARGO_RELEASE_METADATA_EDITOR="$(bash vendor/toko-tooling/scripts/dev/install-rust-tools.sh --consumer "$PWD" --package ic-build --version 0.73.0 --bin ic-build-release-metadata --profile debug --features cli --check)"
        export CARGO_RELEASE_METADATA_EDITOR
        export CARGO_RELEASE_METADATA_MAX_BYTES=8388608
        exec bash scripts/release/metadata.sh prepare
        ;;
    check|commit-check|committed-check)
        if [[ "$1" == commit-check ]]; then
            bash vendor/toko-tooling/scripts/ci/check-release-source.sh --allow Cargo.toml --allow Cargo.lock --allow CHANGELOG.md --allow qualification/Cargo.lock
        fi
        bash vendor/toko-tooling/scripts/ci/cargo-release-metadata.sh check ic-entropy --manifest-order-grouped
        bash scripts/release/metadata.sh selected
        if [[ "$1" == committed-check ]]; then
            bash scripts/release/native-stage.sh
        fi
        ;;
    *) echo 'expected version|preflight|verify|publication-verify|prepare|check|commit-check|committed-check' >&2; exit 2 ;;
esac
