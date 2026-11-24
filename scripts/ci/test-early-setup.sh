#!/usr/bin/env bash
# Execute actual workflow callers with local setup/fetch and upload substitutes.
set -euo pipefail
root="$PWD"
export PATH="$root/.tools/host/bin:$PATH"
mkdir -p target
fixture="$(mktemp -d "$root/target/early-setup-fixture.XXXXXX")"
complete=false
trap 'if [[ "$complete" == true ]]; then rm -rf -- "$fixture"; else printf "Early setup fixture retained: %s\n" "$fixture" >&2; fi' EXIT
yq -p yaml -o json '.jobs.core.steps' .github/workflows/ci.yml > "$fixture/steps.json"
jq -e '
  def step($id): map(select(.id == $id)) | if length == 1 then .[0] else error("missing/duplicate step") end;
  (step("setup-diagnostics") | .if == "always()" and
    .with["if-no-files-found"] == "error" and .with["include-hidden-files"] == true and
    .with["retention-days"] == 14 and (.uses | startswith("actions/upload-artifact@"))) and
  (step("setup-report") | .if == "always()" and
    .env.ARTIFACT_OUTCOME == "${{ steps.setup-diagnostics.outcome }}" and
    .env.ARTIFACT_URL == "${{ steps.setup-diagnostics.outputs.artifact-url }}") and
  (step("compiler-setup") | (."continue-on-error" // false) == false) and
  (step("locked-fetch") | (."continue-on-error" // false) == false) and
  (map(.id) | index("compiler-setup") < index("locked-fetch") and
    index("locked-fetch") < index("setup-diagnostics") and
    index("setup-diagnostics") < index("setup-report")) and
  any(.[]; .run == "bash scripts/ci/test-early-setup.sh" and .if == "runner.os == '\''Linux'\''")
' "$fixture/steps.json" > /dev/null
for id in compiler-setup locked-fetch setup-report; do
    bash vendor/toko-tooling/scripts/ci/extract-workflow-step.sh "$fixture/steps.json" "$id" > "$fixture/$id.sh"
done
jq -r '.[] | select(.id == "setup-diagnostics") | .with.path' "$fixture/steps.json" > "$fixture/paths"
repo="$fixture/repo"
mkdir -p "$repo/scripts/ci" "$repo/vendor/toko-tooling/scripts/ci" "$repo/vendor/toko-tooling/scripts/dev" "$repo/.tools/host/bin"
cp scripts/ci/setup-command.pl scripts/ci/prepare-compiler.sh "$repo/scripts/ci/"
cp vendor/toko-tooling/scripts/ci/SetupDiagnostics.pm "$repo/vendor/toko-tooling/scripts/ci/"
cp rust-toolchain.toml "$repo/"
printf '#!/usr/bin/env bash\nexit 0\n' > "$repo/vendor/toko-tooling/scripts/dev/prepare-rust-toolchain.sh"
cat > "$repo/.tools/host/bin/make" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
if [[ "$SCENARIO" == compiler-fails ]]; then
    echo 'host bootstrap failed (forced)' >&2
    exit 19
fi
SH
cat > "$repo/.tools/host/bin/rustc" <<'SH'
#!/usr/bin/env bash
printf 'rustc %s (fixture)\n' "$FIXTURE_COMPILER"
SH
printf '#!/usr/bin/env bash\nexit 0\n' > "$repo/.tools/host/bin/rustup"
cat > "$repo/.tools/host/bin/cargo" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
    --version|-vV) printf 'cargo %s (fixture)\n' "$FIXTURE_COMPILER"; exit 0 ;;
esac
[[ "$1" == fetch && "$2" == --locked ]]
printf '%s|%s\n' "$*" "${CARGO_NET_OFFLINE-unset}" >> "$FIXTURE_CALLS"
status=0
case "$SCENARIO:$*" in
    product-fails:'fetch --locked'|collector-fails:'fetch --locked') status=37 ;;
    qualification-fails:'fetch --locked --manifest-path qualification/Cargo.toml') status=42 ;;
esac
if [[ "$status" != 0 ]]; then
    if [[ "$SCENARIO" == collector-fails ]]; then
        rm "$RUNNER_TEMP/early-setup/fetch-product/.attempt/diagnostic.log"
        mkdir "$RUNNER_TEMP/early-setup/fetch-product/.attempt/diagnostic.log"
    fi
    for ((n=0; n<2000; n++)); do printf 'fetch progress %04d %0490d\n' "$n" 0; done
    printf '%s\n' 'Bearer fixture-private-value' 'https://example.invalid/index?token=fixture-url-value' >&2
    printf '%s\n' '::notice::fixture-command' 'locked fetch failed (forced)' >&2
    exit "$status"
fi
SH
chmod +x "$repo/.tools/host/bin/"*
export FIXTURE_COMPILER
FIXTURE_COMPILER="$(yq -p toml -o yaml -r '.toolchain.channel' rust-toolchain.toml)"
network="${CARGO_NET_OFFLINE-unset}"

# Build a bounded local payload using the workflow's actual upload selection.
artifact() {
    local directory="$1" pattern file relative size total=0 count=0
    local marker='${{ runner.temp }}'
    mkdir "$directory/payload"
    while IFS= read -r pattern; do
        [[ -n "$pattern" ]] || continue
        pattern="${pattern//"$marker"/"$RUNNER_TEMP"}"
        while IFS= read -r file; do
            [[ -f "$file" && ! -L "$file" ]]
            relative="${file#"$RUNNER_TEMP/"}"
            [[ "$relative" != "$file" ]]
            mkdir -p "$directory/payload/${relative%/*}"
            cp "$file" "$directory/payload/$relative"
            size="$(wc -c < "$file")"
            [[ "$size" -le 32768 ]]
            total=$((total + size)); count=$((count + 1))
        done < <(cd "$repo" && compgen -G "$pattern" || true)
    done < "$fixture/paths"
    [[ "$count" -gt 0 && "$total" -le 102400 ]]
    tar -czf "$directory/retained.tgz" -C "$directory/payload" .
    [[ "$(wc -c < "$directory/retained.tgz")" -le 131072 ]]
    [[ ! -e "$directory/payload/private-download.tar" ]]
}

for scenario in compiler-fails product-fails qualification-fails success collector-fails; do
    directory="$fixture/$scenario"
    mkdir -p "$directory/runtime"
    export RUNNER_TEMP="$directory/runtime" SCENARIO="$scenario" FIXTURE_CALLS="$directory/cargo.calls"
    printf 'unselected private archive\n' > "$RUNNER_TEMP/private-download.tar"
    status=0
    (
        cd "$repo"
        export PATH="$repo/.tools/host/bin:$PATH"
        bash --noprofile --norc -e -o pipefail "$fixture/compiler-setup.sh"
    ) > "$directory/command.log" 2>&1 || status=$?
    if [[ "$status" == 0 ]]; then
        (cd "$repo" && PATH="$repo/.tools/host/bin:$PATH" bash --noprofile --norc -e -o pipefail "$fixture/locked-fetch.sh") \
            >> "$directory/command.log" 2>&1 || status=$?
    fi
    phase=fetch-product expected=37
    case "$scenario" in
        compiler-fails) phase=compiler expected=19; [[ ! -e "$FIXTURE_CALLS" ]] ;;
        qualification-fails) phase=fetch-qualification expected=42 ;;
        success) phase=fetch-qualification expected=0 ;;
    esac
    [[ "$status" == "$expected" ]]
    rg -q "^command_status=$expected$" "$RUNNER_TEMP/early-setup/$phase/status.txt"
    if [[ "$scenario" != compiler-fails ]]; then
        awk -F '|' -v expected="$network" '$2 != expected { exit 1 }' "$FIXTURE_CALLS"
    fi
    if [[ "$scenario" == product-fails || "$scenario" == collector-fails ]]; then
        [[ "$(wc -l < "$FIXTURE_CALLS")" -eq 1 ]]
        [[ ! -e "$RUNNER_TEMP/early-setup/fetch-qualification" ]]
    fi
    report=0
    (cd "$repo" && ARTIFACT_OUTCOME=success ARTIFACT_URL=https://github.com/example/artifacts/1 \
        GITHUB_STEP_SUMMARY="$directory/summary" bash --noprofile --norc -e -o pipefail "$fixture/setup-report.sh") \
        > "$directory/report.log" 2>&1 || report=$?
    if [[ "$scenario" == collector-fails ]]; then
        [[ "$report" != 0 ]]
        rg -q '^collection_status=failed$' "$RUNNER_TEMP/early-setup/fetch-product/status.txt"
        rg -q 'original setup status remains 37' "$directory/command.log"
        continue
    fi
    [[ "$report" == 0 ]]
    rg -q 'https://github.com/example/artifacts/1' "$directory/summary"
    if [[ "$scenario" != compiler-fails ]]; then
        rg -Fq "rustc $FIXTURE_COMPILER (fixture)" "$directory/report.log"
        rg -Fq "cargo $FIXTURE_COMPILER (fixture)" "$directory/report.log"
    fi
    artifact "$directory"
    if [[ "$scenario" == product-fails || "$scenario" == qualification-fails ]]; then
        log="$directory/payload/early-setup/$phase/.attempt/diagnostic.log"
        rg -q 'locked fetch failed \(forced\)' "$log"
        rg -q 'earlier output.*omitted' "$log"
        if rg 'fixture-private-value|fixture-url-value|::notice::fixture-command' "$directory/payload" "$directory/report.log"; then exit 1; fi
    fi
    # Neither an upload failure nor missing evidence can be reported as success.
    report=0
    (cd "$repo" && ARTIFACT_OUTCOME=failure ARTIFACT_URL='' GITHUB_STEP_SUMMARY="$directory/summary" \
        bash --noprofile --norc -e -o pipefail "$fixture/setup-report.sh") > "$directory/upload-failed.log" 2>&1 || report=$?
    [[ "$report" != 0 ]]
    rg -q 'upload failed or produced no link' "$directory/upload-failed.log"
    rm "$RUNNER_TEMP/early-setup/$phase/status.txt"
    report=0
    (cd "$repo" && ARTIFACT_OUTCOME=success ARTIFACT_URL=https://github.com/example/artifacts/1 \
        GITHUB_STEP_SUMMARY="$directory/summary" bash --noprofile --norc -e -o pipefail "$fixture/setup-report.sh") \
        > "$directory/missing.log" 2>&1 || report=$?
    [[ "$report" != 0 ]]
    # Removing the entire last phase must not make its required evidence optional.
    rm -rf -- "$RUNNER_TEMP/early-setup/$phase"
    report=0
    (cd "$repo" && ARTIFACT_OUTCOME=success ARTIFACT_URL=https://github.com/example/artifacts/1 \
        GITHUB_STEP_SUMMARY="$directory/summary" bash --noprofile --norc -e -o pipefail "$fixture/setup-report.sh") \
        > "$directory/missing-phase.log" 2>&1 || report=$?
    [[ "$report" != 0 ]]
done
complete=true
printf 'Early compiler/fetch failures retained bounded sanitized artifacts and original statuses; collector, missing-evidence and upload failures refused.\n'
