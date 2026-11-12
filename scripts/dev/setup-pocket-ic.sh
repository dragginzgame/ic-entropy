#!/usr/bin/env bash
# Testkit remains the sole downloader/authenticator. Retry only a retained,
# recognized transient curl failure from a new explicit setup attempt.
set -euo pipefail
[[ $# == 2 && -x "$1" && "$2" == /* ]] || exit 2
server="$1" directory="$2"
mkdir -p target/setup-pocket-ic
redact() { perl -pe 's{https?://[^\s<>"\x27]+}{<download-url>}g'; }
for attempt in 1 2 3; do
    logs="$(mktemp -d "$PWD/target/setup-pocket-ic/attempt.XXXXXX")"
    log="$logs/setup.log"
    printf 'PocketIC setup attempt %s/3\n' "$attempt"
    set +e
    "$server" setup --directory "$directory" 2>&1 | redact | tee "$log"
    statuses=("${PIPESTATUS[@]}")
    set -e
    status="${statuses[0]}"
    # Logging errors never turn a setup failure into a successful admission.
    if [[ "${statuses[1]}" != 0 || "${statuses[2]}" != 0 ]]; then
        [[ "$status" != 0 ]] || status=1
        exit "$status"
    fi
    [[ "$status" != 0 ]] || exit 0
    case "$status" in 130|143) exit "$status" ;; esac
    evidence="$(sed -n 's/^PocketIC setup evidence: //p' "$log" | tail -1)"
    case "$evidence" in "$directory"/.setup-v1-*) ;; *) exit "$status" ;; esac
    relative="${evidence#"$directory"/}"
    [[ "$relative" =~ ^\.setup-v1-[A-Za-z0-9_-]+$ ]] || exit "$status"
    download="$evidence/download.stderr"
    [[ -d "$evidence" && ! -L "$evidence" && -f "$download" && ! -L "$download" ]] || exit "$status"
    [[ "$(wc -c < "$download")" -le 65536 ]] || exit "$status"
    if ! redact < "$download" | tee "$log.download.log" >&2; then
        printf 'Could not retain download diagnostics; preserving setup failure.\n' >&2
        exit "$status"
    fi
    if ! LC_ALL=C grep -Eq '^curl: \((5|6|7|18|28|35|52|55|56)\)' "$download"; then exit "$status"; fi
    [[ "$attempt" != 3 ]] || exit "$status"
    printf 'Transient PocketIC download failure; retrying authenticated setup.\n' >&2
    sleep "$((attempt * 5))"
done
