#!/usr/bin/env bash
# Exercise actual inventories without release, publication or cache deletion.
set -euo pipefail
root="$PWD"
mkdir -p target
fixture="$(mktemp -d "$root/target/required-inventory.XXXXXX")"
cache="$fixture/cache"
source="$fixture/source"
mkdir -p "$cache/target/debug/unreadable" "$source/unreadable" "$fixture/bin"
trap 'chmod 700 "$cache/target/debug/unreadable" "$source/unreadable"' EXIT
printf 'cached object\n' > "$cache/target/debug/object"

# A partial inventory must never become an admitted cache receipt.
chmod 000 "$cache/target/debug/unreadable"
if (cd "$cache" && perl "$root/scripts/ci/build-cache.pl" seal) > "$fixture/cache.log" 2>&1; then
    echo 'Cache inventory accepted an unreadable directory.' >&2
    exit 1
fi
rg -q 'required inventory traversal failed:' "$fixture/cache.log"
[[ ! -e "$cache/target/ci-build-cache.json" ]]
chmod 700 "$cache/target/debug/unreadable"
(cd "$cache" && perl "$root/scripts/ci/build-cache.pl" seal)
(cd "$cache" && perl "$root/scripts/ci/build-cache.pl" restore)

# Source observation must refuse before querying dependencies or emitting JSON.
cat > "$fixture/bin/cargo" <<'EOF'
#!/usr/bin/env bash
printf 'called\n' > "$FIXTURE_CARGO_MARKER"
exit 73
EOF
chmod +x "$fixture/bin/cargo"
export FIXTURE_CARGO_MARKER="$fixture/cargo-called"
chmod 000 "$source/unreadable"
if (cd "$source" && PATH="$fixture/bin:$PATH" perl "$root/scripts/release/observe-native.pl") \
    > "$fixture/source.json" 2> "$fixture/source.log"; then
    echo 'Source inventory accepted an unreadable directory.' >&2
    exit 1
fi
rg -q 'required inventory traversal failed:' "$fixture/source.log"
[[ ! -e "$FIXTURE_CARGO_MARKER" && ! -s "$fixture/source.json" ]]
chmod 700 "$source/unreadable"
if (cd "$source" && PATH="$fixture/bin:$PATH" perl "$root/scripts/release/observe-native.pl") \
    > "$fixture/readable.json" 2> "$fixture/readable.log"; then
    echo 'Source fixture unexpectedly completed dependency observation.' >&2
    exit 1
fi
[[ -s "$FIXTURE_CARGO_MARKER" ]]
rg -q 'observation failed: cargo metadata' "$fixture/readable.log"
printf 'Required inventories refused unreadable inputs: %s\n' "$fixture"
