#!/usr/bin/env bash
# Disposable hosted-CI compilation outputs only; never release/publication proof.
set -euo pipefail
[[ "${GITHUB_ACTIONS:-}" == true ]] || { echo 'build cache belongs to hosted CI' >&2; exit 2; }
[[ $# == 1 && ( "$1" == restore || "$1" == seal ) ]] || exit 2
mode="$1"
if perl scripts/ci/build-cache.pl "$mode"; then exit 0; fi
if [[ "$mode" == seal ]]; then
    echo 'Compilation cache exceeds its budget or contains unsupported files; not saving.' >&2
    exit 1
fi
echo 'Compilation cache absent/corrupt; rebuilding from locked source.' >&2
# Only the declared disposable restore paths are discarded. Preserve admission logs.
rm -rf -- target/debug target/release target/wasm32-unknown-unknown target/qualification
rm -f -- target/ci-build-cache.json
