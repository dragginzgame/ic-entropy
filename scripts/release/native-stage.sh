#!/usr/bin/env bash
# Exact final-version reuse; shared tooling owns proof custody and invalidation.
set -euo pipefail
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
# Nested Make calls repeat these prefixes. Removing exact duplicates preserves
# executable lookup order while release and publication use the same environment.
PATH="$(perl -e 'my %seen; print join ":",grep {!$seen{$_}++} split /:/,$ARGV[0],-1' "$PATH")"
export PATH
ordinary() {
    printf 'Native validation: executing (%s)\n' "$1"
    make --no-print-directory lint test check-wasm doc-check qualification-check test-installed
}
# Unknown compiler/build/simulator selections retain the original ambient gate.
if [[ "$(uname -s)" != Linux ]]; then ordinary 'host not qualified for reuse'; exit; fi
directory="$PWD/qualification"
while :; do
    if [[ -e "$directory/.cargo/config" || -e "$directory/.cargo/config.toml" ]]; then
        ordinary 'Cargo configuration needs separate qualification'; exit
    fi
    [[ "$directory" != / ]] || break
    directory="${directory%/*}"; [[ -n "$directory" ]] || directory=/
done
for config in "${CARGO_HOME:-$HOME/.cargo}/config" "${CARGO_HOME:-$HOME/.cargo}/config.toml"; do
    if [[ -e "$config" ]]; then ordinary 'Cargo home configuration needs separate qualification'; exit; fi
done
for name in $(compgen -e); do
    case "$name" in
        CARGO_*|RUST*|CC*|CXX*|CFLAGS|CPPFLAGS|LDFLAGS|AR|LD*|PKG_CONFIG*|SDKROOT|MAKEFLAGS|MFLAGS|POCKET_IC*|IC_TESTKIT*|TOKO_TOOLING*)
            case "$name" in
                RUSTUP_HOME|CARGO_HOME|CARGO_PUBLISH_*|CARGO_RELEASE_METADATA_*) continue ;;
                MAKEFLAGS|MFLAGS)
                    if perl scripts/release/stage-makeflags.pl "${!name}"; then continue; fi ;;
            esac
            ordinary "environment override: $name"; exit ;;
    esac
done
mkdir -p target/release-validation
environment="$(mktemp "$PWD/target/release-validation/environment.XXXXXX")"
export ENTROPY_STAGE_PATH="$PATH"
perl -MJSON::PP -e '
    my %e=(PATH=>$ENV{ENTROPY_STAGE_PATH},HOME=>$ENV{HOME},LANG=>"C",LC_ALL=>"C",CARGO_NET_OFFLINE=>"true");
    for (qw(CARGO_HOME RUSTUP_HOME)) { $e{$_}=$ENV{$_} if exists $ENV{$_}; }
    print JSON::PP->new->canonical->encode(\%e),"\n";
' > "$environment"
# A failure/refusal from the shared verifier never falls through to a skip path.
perl scripts/ci/run-validation-stage.pl --state target/release-validation/stages \
    --stage native --contract ic-entropy-native/1 --group entropy \
    --source "$(git rev-parse HEAD)" --observe scripts/release/observe-native.pl \
    --environment "$environment" --max-bytes 536870912 -- \
    make --no-print-directory lint test check-wasm doc-check qualification-check test-installed
