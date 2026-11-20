#!/usr/bin/env bash
# Consumer dispatch and disposable cache recovery; no release/publication effects.
set -euo pipefail
root="$PWD"
make_binary="$(command -v make)"
mkdir -p target
fixture="$(mktemp -d "$root/target/validation-caller.XXXXXX")"
complete=false
trap 'if [[ "$complete" == true ]]; then rm -rf -- "$fixture"; else printf "Validation caller fixture retained: %s\n" "$fixture" >&2; fi' EXIT
mkdir -p "$fixture/scripts/release" "$fixture/scripts/ci" \
    "$fixture/vendor/toko-tooling/scripts/ci" "$fixture/.tools/host/bin"
cp scripts/release/adapter.sh scripts/release/native-stage.sh scripts/release/stage-makeflags.pl "$fixture/scripts/release/"
cp scripts/ci/build-cache.sh scripts/ci/build-cache.pl "$fixture/scripts/ci/"
cat > "$fixture/.tools/host/bin/make" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$*" >> calls
printf '%s\n' "${CARGO_NET_OFFLINE-unset}" >> cargo-network
exit "${FIXTURE_STATUS:-0}"
EOF
chmod +x "$fixture/.tools/host/bin/make"
cat > "$fixture/vendor/toko-tooling/scripts/ci/read-cargo-workspace-version.sh" <<'EOF'
printf '0.1.10\n'
EOF
for script in check-release-source cargo-release-metadata; do
    printf '#!/usr/bin/env bash\nexit 0\n' > "$fixture/vendor/toko-tooling/scripts/ci/$script.sh"
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
    ! test -e calls || ! rg 'validate-native' calls
    # Publication and committed-source checks both retain a native obligation.
    RELEASE_VERSION=0.1.11 RUSTFLAGS=-Cdebuginfo=0 bash scripts/release/adapter.sh publication-verify > publication.log
    rg -q 'validate-native' calls
    rm calls
    RUSTFLAGS=-Cdebuginfo=0 bash scripts/release/adapter.sh committed-check > committed.log
    rg -q 'validate-native' calls
    # An unsupported environment must execute ordinarily, preserving failure.
    set +e
    FIXTURE_STATUS=23 RUSTFLAGS=-Cdebuginfo=0 bash scripts/release/native-stage.sh > failed.log
    status=$?
    set -e
    [[ "$status" == 23 ]]
    # Explicit Cargo network settings select the ordinary gate unchanged.
    CARGO_NET_OFFLINE=true bash scripts/release/native-stage.sh > offline.log
    [[ "$(tail -n 1 cargo-network)" == true ]]
    rg -q 'environment override: CARGO_NET_OFFLINE' offline.log
    CARGO_NET_OFFLINE=false bash scripts/release/native-stage.sh > online.log
    [[ "$(tail -n 1 cargo-network)" == false ]]
    rg -q 'environment override: CARGO_NET_OFFLINE' online.log
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

# Exercise the real Make roster under parallel scheduling. Failed early lint
# must prevent tests, Wasm/docs and installed qualification from starting.
cp Makefile "$fixture/Makefile"
cp -R vendor/toko-tooling/make "$fixture/vendor/toko-tooling/make"
cat > "$fixture/.tools/host/bin/cargo" <<'EOF_CARGO'
#!/usr/bin/env bash
printf '%s\n' "$*" >> cargo-calls
if [[ "${FIXTURE_CARGO_FAILURE:-}" == "$1" ]]; then exit 23; fi
EOF_CARGO
chmod +x "$fixture/.tools/host/bin/cargo"
(
    cd "$fixture"
    status=0
    FIXTURE_CARGO_FAILURE=clippy "$make_binary" --no-print-directory -j2 validate-native > lint-failed.log 2>&1 || status=$?
    [[ "$status" == 2 ]]
    [[ "$(wc -l < cargo-calls)" -eq 1 ]]
    rg -q '^clippy --locked -p ic-entropy --all-targets$' cargo-calls
    rm cargo-calls
    "$make_binary" --no-print-directory -j2 validate-native > ordered.log 2>&1
    # The complete lint selection precedes the first test, even with -j2.
    awk '
        $1 == "test" { tests = 1 }
        $1 == "clippy" && $0 !~ /--target wasm32/ && $0 !~ /qualification\/Cargo.toml/ {
            if (tests) exit 1
            lint++
        }
        END { if (lint != 9 || !tests) exit 1 }
    ' cargo-calls
)
complete=true
printf 'Consumer dispatch, Cargo settings, cache recovery and parallel lint ordering passed.\n'
