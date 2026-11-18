#!/usr/bin/env bash
# Exercise the current prepare_server example and curl caller without downloads.
set -euo pipefail
root="$PWD"
mkdir -p target
fixture="$(mktemp -d "$root/target/pocket-ic-setup-tests.XXXXXX")"
complete=false
trap 'if [[ "$complete" == true ]]; then rm -rf "$fixture"; else printf "PocketIC setup fixture retained: %s\n" "$fixture" >&2; fi' EXIT
mkdir -p "$fixture/bin"
cat > "$fixture/server" <<'HELPER'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$#" == 1 && "$1" == --asset ]]; then
    if [[ "$SCENARIO" == asset-refusal ]]; then
        printf 'https://example.invalid/server\nsha256:fixture\n'
    else printf 'https://github.com/dfinity/pocketic/releases/download/fixture/server.gz\nsha256:fixture\n'; fi
    exit 0
fi
[[ "$#" == 3 && "$1" == "$PWD"/target/setup-pocket-ic/attempt.*/server.gz ]]
[[ -s "$1" && "$2" == "$PWD/prepared" && "$3" == /* && -x "$3" ]]
printf 'prepare\n' >> prepare.calls
case "$SCENARIO" in
    invalid) printf 'archive checksum mismatch https://example.invalid/?secret=test\n' >&2; exit 17 ;;
    prepare-cancel) exit 130 ;;
esac
printf '%s/pocket-ic\n' "$2"
HELPER
cat > "$fixture/bin/curl" <<'CURL'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == *"--proto =https --proto-redir =https"* ]]
[[ "$*" == *"--connect-timeout 15 --max-time 180 --retry 2 --retry-delay 2 --retry-max-time 30"* ]]
output=''
while [[ "$#" -gt 0 ]]; do
    if [[ "$1" == --output ]]; then output="$2"; shift; fi
    shift
done
[[ "$output" == "$PWD"/target/setup-pocket-ic/attempt.*/server.gz ]]
printf 'curl\n' >> download.calls
case "$SCENARIO" in
    http) printf 'curl: (22) HTTP 404\n' >&2; exit 22 ;;
    transient) printf 'curl: (56) connection reset https://example.invalid/?secret=test\n' >&2; exit 56 ;;
    cancel) exit 130 ;;
esac
printf 'fixture archive\n' > "$output"
CURL
chmod +x "$fixture/server" "$fixture/bin/curl"
for scenario in success invalid http transient cancel prepare-cancel asset-refusal; do
    mkdir "$fixture/$scenario"
    status=0
    (cd "$fixture/$scenario" && SCENARIO="$scenario" PATH="$fixture/bin:$PATH" \
        bash "$root/scripts/dev/setup-pocket-ic.sh" "$fixture/server" "$PWD/prepared") \
        > "$fixture/$scenario.log" 2>&1 || status=$?
    case "$scenario" in
        success) [[ "$status" == 0 && -f "$fixture/$scenario/prepare.calls" ]] ;;
        invalid) [[ "$status" == 17 && -f "$fixture/$scenario/prepare.calls" ]] ;;
        prepare-cancel) [[ "$status" == 130 && -f "$fixture/$scenario/prepare.calls" ]] ;;
        http) [[ "$status" == 22 && ! -e "$fixture/$scenario/prepare.calls" ]] ;;
        transient) [[ "$status" == 56 && ! -e "$fixture/$scenario/prepare.calls" ]] ;;
        cancel) [[ "$status" == 130 && ! -e "$fixture/$scenario/prepare.calls" ]] ;;
        asset-refusal) [[ "$status" == 2 && ! -e "$fixture/$scenario/download.calls" ]]; continue ;;
    esac
    [[ "$(wc -l < "$fixture/$scenario/download.calls")" -eq 1 ]]
    logs=("$fixture/$scenario"/target/setup-pocket-ic/attempt.*/download.log)
    [[ "${#logs[@]}" -eq 1 && -f "${logs[0]}" ]]
done
if rg 'secret=test' "$fixture"/invalid/target "$fixture"/transient/target; then exit 1; fi
rg -q '<download-url>' "$fixture/invalid.log"
rg -q '<download-url>' "$fixture/transient.log"
printf 'Current helper contract, bounded curl policy, failure/cancellation status and redacted logs passed.\n'
complete=true
