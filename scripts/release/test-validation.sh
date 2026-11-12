#!/usr/bin/env bash
# Consumer dispatch and disposable cache recovery; no release/publication effects.
set -euo pipefail
root="$PWD"
mkdir -p target
fixture="$(mktemp -d "$root/target/validation-caller.XXXXXX")"
mkdir -p "$fixture/scripts/release" "$fixture/scripts/ci" "$fixture/.tools/host/bin"
cp scripts/release/adapter.sh scripts/release/native-stage.sh scripts/release/stage-makeflags.pl "$fixture/scripts/release/"
cp scripts/ci/build-cache.sh scripts/ci/build-cache.pl "$fixture/scripts/ci/"
cat > "$fixture/.tools/host/bin/make" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> calls
exit "${FIXTURE_STATUS:-0}"
EOF
chmod +x "$fixture/.tools/host/bin/make"
cat > "$fixture/scripts/ci/read-cargo-workspace-version.sh" <<'EOF'
printf '0.1.10\n'
EOF
for script in check-release-source cargo-release-metadata check-dependency-pins; do
    printf '#!/usr/bin/env bash\nexit 0\n' > "$fixture/scripts/ci/$script.sh"
done
printf '#!/usr/bin/env bash\nexit 0\n' > "$fixture/scripts/release/metadata.sh"
(
    cd "$fixture"
    perl scripts/release/stage-makeflags.pl ' --no-print-directory -- RELEASE_KIND=patch RELEASE_VERSION=0.1.11 CARGO_PUBLISH_ALL_FEATURES=1'
    if perl scripts/release/stage-makeflags.pl ' -j2 --jobserver-auth=3,4'; then exit 1; fi
    if perl scripts/release/stage-makeflags.pl ' -- RUSTFLAGS=-Cdebuginfo=0'; then exit 1; fi
    if perl scripts/release/stage-makeflags.pl '"unterminated'; then exit 1; fi
    # A preliminary version cannot satisfy final-version native validation.
    RELEASE_VERSION=0.1.11 bash scripts/release/adapter.sh verify > preliminary.log
    ! test -e calls || ! rg 'lint test' calls
    # Publication and committed-source checks both retain a native obligation.
    RELEASE_VERSION=0.1.11 RUSTFLAGS=-Cdebuginfo=0 bash scripts/release/adapter.sh publication-verify > publication.log
    rg -q 'lint test check-wasm qualification-check test-installed' calls
    rm calls
    RUSTFLAGS=-Cdebuginfo=0 bash scripts/release/adapter.sh committed-check > committed.log
    rg -q 'lint test check-wasm qualification-check test-installed' calls
    # An unsupported environment must execute ordinarily, preserving failure.
    set +e
    FIXTURE_STATUS=23 RUSTFLAGS=-Cdebuginfo=0 bash scripts/release/native-stage.sh > failed.log
    status=$?
    set -e
    [[ "$status" == 23 ]]
    export GITHUB_ACTIONS=true
    bash scripts/ci/build-cache.sh restore > cold.log 2>&1
    mkdir -p target/debug
    printf 'object' > target/debug/object
    bash scripts/ci/build-cache.sh seal
    bash scripts/ci/build-cache.sh restore
    printf 'damaged' > target/debug/object
    bash scripts/ci/build-cache.sh restore > corrupt.log 2>&1
    [[ ! -e target/debug && ! -e target/ci-build-cache.json ]]
    mkdir -p target/debug
    ln -s "$fixture/calls" target/debug/unsafe
    if bash scripts/ci/build-cache.sh seal > unsafe.log 2>&1; then exit 1; fi
)
printf 'Consumer dispatch, failed validation, cold cache, warm cache and corrupt-cache recovery passed: %s\n' "$fixture"
