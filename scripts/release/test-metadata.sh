#!/usr/bin/env bash
# Verify this consumer's two-lock projection without staging or live version edits.
set -euo pipefail
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
export CARGO_RELEASE_METADATA_EDITOR
CARGO_RELEASE_METADATA_EDITOR="$(bash scripts/dev/install-rust-tools.sh --consumer "$PWD" --package ic-build --version 0.73.0 --bin ic-build-release-metadata --profile debug --features cli --check)"
export CARGO_RELEASE_METADATA_MAX_BYTES=8388608
export RELEASE_PREVIOUS RELEASE_VERSION RELEASE_DATE=2026-11-09
RELEASE_PREVIOUS="$(bash scripts/ci/read-cargo-workspace-version.sh --stable Cargo.toml)"
RELEASE_VERSION="$(perl scripts/ci/validate-release-inputs.pl next-version "$RELEASE_PREVIOUS" patch)"
mkdir -p target
scratch="$(mktemp -d "$PWD/target/release-metadata-test.XXXXXX")"
complete=false
trap 'if [[ "$complete" == true ]]; then rm -rf -- "$scratch"; else printf "Metadata test retained: %s\n" "$scratch" >&2; fi' EXIT
mkdir -p "$scratch/original/qualification" "$scratch/candidate/qualification"
for path in Cargo.toml Cargo.lock qualification/Cargo.lock; do
    cp "$path" "$scratch/original/$path"
done
printf '# Changelog\n\n## [%s] - Unreleased\n\n### Added\n\n- Fixture.\n' "$RELEASE_VERSION" > "$scratch/original/CHANGELOG.md"
bash scripts/release/metadata.sh render "$scratch/original" "$scratch/candidate"
for path in Cargo.lock qualification/Cargo.lock; do
    yq -p toml -o json '.' "$scratch/original/$path" |
        jq --arg version "$RELEASE_VERSION" '
            (.package[] | select(.source == null and (.name == "ic-entropy" or .name == "ic-entropy-ic")) | .version) = $version
        ' > "$scratch/expected.json"
    yq -p toml -o json '.' "$scratch/candidate/$path" > "$scratch/actual.json"
    jq -e --slurpfile expected "$scratch/expected.json" '. == $expected[0]' "$scratch/actual.json" > /dev/null
done
yq -p toml -o json '.' "$scratch/candidate/Cargo.toml" |
    jq -e --arg version "$RELEASE_VERSION" '
        .workspace.package.version == $version and
        .workspace.dependencies["ic-entropy"].version == $version and
        .workspace.dependencies["ic-entropy-ic"].version == $version
    ' > /dev/null
for path in Cargo.toml Cargo.lock qualification/Cargo.lock; do
    cmp "$path" "$scratch/original/$path"
done
complete=true
echo 'Product versions advance in both locks; private probe and all other selections are preserved.'
