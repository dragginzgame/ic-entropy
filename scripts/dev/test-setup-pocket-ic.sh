#!/usr/bin/env bash
# Simulated Testkit failures qualify caller retry policy; no download mock enters
# production and no checksum/version admission is replaced.
set -euo pipefail
root="$PWD"
mkdir -p target
fixture="$(mktemp -d "$root/target/pocket-ic-setup-tests.XXXXXX")"
mkdir -p "$fixture/bin"
cat > "$fixture/server" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == setup && "$2" == --directory ]]
directory="$3"
mkdir -p "$directory"
count=0
[[ ! -f "$directory/count" ]] || count="$(cat "$directory/count")"
count=$((count + 1)); printf '%s\n' "$count" > "$directory/count"
scenario="${directory##*/}"
case "$scenario:$count" in success:*|recover:2) exit 0 ;; esac
evidence="$directory/.setup-v1-$$-$count"
mkdir -p "$evidence"
printf 'PocketIC setup evidence: %s\n' "$evidence" >&2
case "$scenario" in
    invalid) printf 'checksum mismatch\n' > "$evidence/download.stderr" ;;
    http) printf 'curl: (22) HTTP 404\n' > "$evidence/download.stderr" ;;
    cancel) printf 'curl: (56) interrupted\n' > "$evidence/download.stderr"; exit 130 ;;
    *) printf 'curl: (56) connection reset https://example.invalid/path?secret=test\n' > "$evidence/download.stderr" ;;
esac
exit 17
EOF
cat > "$fixture/bin/sleep" <<'EOF'
#!/usr/bin/env bash
printf '%s\n' "$1" >> sleeps
EOF
chmod +x "$fixture/server" "$fixture/bin/sleep"
(
    cd "$fixture"
    export PATH="$fixture/bin:$PATH"
    for scenario in success recover exhausted invalid http cancel; do
        set +e
        bash "$root/scripts/dev/setup-pocket-ic.sh" "$fixture/server" "$fixture/$scenario" > "$scenario.log" 2>&1
        status=$?
        set -e
        case "$scenario" in
            success) [[ "$status" == 0 && "$(cat "$scenario/count")" == 1 ]] ;;
            recover) [[ "$status" == 0 && "$(cat "$scenario/count")" == 2 ]] ;;
            exhausted) [[ "$status" == 17 && "$(cat "$scenario/count")" == 3 ]] ;;
            invalid|http) [[ "$status" == 17 && "$(cat "$scenario/count")" == 1 ]] ;;
            cancel) [[ "$status" == 130 && "$(cat "$scenario/count")" == 1 ]] ;;
        esac
    done
    [[ "$(cat sleeps)" == $'5\n5\n10' ]]
    if rg 'secret=test' target/setup-pocket-ic; then exit 1; fi
    rg -q '<download-url>' exhausted.log
)
printf 'Setup success, bounded retry, integrity/HTTP refusal, cancellation and redaction passed: %s\n' "$fixture"
