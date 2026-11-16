#!/usr/bin/env bash
# Explicit acquisition only. Testkit owns asset selection, authentication and
# bounded decompression; the installed driver owns server and instance lifetimes.
set -euo pipefail
umask 077
[[ $# == 2 && -x "$1" && "$2" == /* ]] || exit 2
helper="$1" directory="$2"
selection="$("$helper" --asset)"
url="${selection%%$'\n'*}"
[[ "$url" == https://github.com/dfinity/pocketic/releases/download/* ]] || exit 2
gzip="$(command -v gzip)"
[[ "$gzip" == /* && -x "$gzip" ]] || exit 2
mkdir -p target/setup-pocket-ic
install -d -m 700 "$directory"
logs="$(mktemp -d "$PWD/target/setup-pocket-ic/attempt.XXXXXX")"
redact() { perl -pe 's{https?://[^\s<>"\x27]+}{<download-url>}g'; }
curl --proto '=https' --tlsv1.2 --fail --location --retry 2 --retry-delay 2 \
    "$url" --output "$logs/server.gz" 2>&1 | redact | tee "$logs/download.log"
"$helper" "$logs/server.gz" "$directory" "$gzip" 2>&1 | redact | tee "$logs/prepare.log"
