#!/usr/bin/env bash
# Offline checks of this consumer's setup capture and failure propagation only.
set -euo pipefail
root="$PWD"
mkdir -p target
fixture="$(mktemp -d "$root/target/scanner-setup-tests.XXXXXX")"
complete=false
trap 'if [[ "$complete" == true ]]; then rm -rf "$fixture"; else printf "Scanner setup fixture retained: %s\n" "$fixture" >&2; fi' EXIT
mkdir -p "$fixture/repo/scripts/ci" "$fixture/repo/vendor/toko-tooling/scripts/ci" "$fixture/bin"
cp scripts/ci/setup-scanner.sh scripts/ci/scanner-setup-log.pl "$fixture/repo/scripts/ci/"
cat > "$fixture/repo/vendor/toko-tooling/scripts/ci/install-ci-tool.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
[[ "$1" == gitleaks && "$2" == --install-dir && "$3" == "$RUNNER_TEMP/entropy-tools" ]]
case "$SCENARIO" in
    success|collector-fails) printf '%s\n' "$3/gitleaks"; exit 0 ;;
    early)
        printf '%s\n' 'curl: (22) The requested URL returned error: 503' >&2
        printf '%s\n' 'https://user:password@example.invalid/path?token=url-secret' >&2
        printf '%s\n' 'Authorization: Bearer header-secret' 'TOKEN=assignment-secret' >&2
        printf '%s\n' 'Bearer bare-bearer-value' 'request failed (bEaReR embedded-bearer-value)' >&2
        printf '%s\n' '{"payload":"json-payload-value"}' "payload='plain-payload-value'" >&2
        printf '%s\n' '{"request_payload":"prefixed-payload-value"}' >&2
        printf '%s\n' "'PaYlOaD': ['nested-payload-value']" >&2
        printf '%s\n' "$FIXTURE_API_KEY" "embedded=$FIXTURE_ACCESS_KEY!" 'api_key=labelled-api-value' >&2
        printf '%s\n' '::notice title=setup::diagnostic message' >&2
        printf '%s\n' 'github_pat_tokenSecret123' "$FIXTURE_SECRET" >&2
        printf '%s\n' '-----BEGIN PRIVATE KEY-----' 'private-material' '-----END PRIVATE KEY-----' >&2
        mkdir -p "$3/.gitleaks-install.supplier"
        printf 'not diagnostic evidence\n' > "$3/.gitleaks-install.supplier/archive.tar.gz"
        exit 22 ;;
    no-output) exit 17 ;;
    original-wins) printf 'underlying setup failure\n' >&2; exit 19 ;;
    large)
        for ((n=0; n<3000; n++)); do printf 'checksum mismatch diagnostic %s\n' "$n"; done
        exit 21 ;;
    long-line)
        printf '%05000d\n' 0
        printf 'checksum mismatch\n'
        exit 23 ;;
esac
exit 2
SH
cat > "$fixture/bin/perl" <<'SH'
#!/usr/bin/env bash
cat > /dev/null
exit 71
SH
chmod +x "$fixture/bin/perl"
run() {
    local scenario="$1" expected="$2" status=0
    mkdir "$fixture/$scenario"
    if [[ "$scenario" == collector-fails || "$scenario" == original-wins ]]; then
        extra_path="$fixture/bin:"
    else extra_path=''; fi
    (cd "$fixture/repo" && RUNNER_TEMP="$fixture/$scenario" SCENARIO="$scenario" \
        FIXTURE_SECRET=environment-secret FIXTURE_API_KEY=api-environment-value \
        FIXTURE_ACCESS_KEY=access-environment-value PATH="$extra_path$PATH" bash scripts/ci/setup-scanner.sh) \
        > "$fixture/$scenario.stdout" 2> "$fixture/$scenario.stderr" || status=$?
    [[ "$status" == "$expected" ]]
    [[ -f "$fixture/$scenario/scanner-setup/status.txt" ]]
    rg -q "^step_status=$expected$" "$fixture/$scenario/scanner-setup/status.txt"
}
run success 0
run early 22
run no-output 17
run collector-fails 71
run original-wins 19
run large 21
run long-line 23

# Short diagnostics must reach disk while the installer is still running.
mkdir "$fixture/live"
mkfifo "$fixture/live/input"
perl scripts/ci/scanner-setup-log.pl capture "$fixture/live/log" < "$fixture/live/input" &
collector=$!
exec 3> "$fixture/live/input"
printf 'partial installer diagnostic\n' >&3
retained=false
for ((n=0; n<20; n++)); do
    if [[ -f "$fixture/live/log" ]] && rg -q 'partial installer diagnostic' "$fixture/live/log"; then
        retained=true
        break
    fi
    perl -e 'select undef, undef, undef, 0.05'
done
exec 3>&-
wait "$collector"
[[ "$retained" == true ]]

evidence="$fixture/early/scanner-setup"
perl scripts/ci/scanner-setup-log.pl report "$evidence" > "$fixture/report"
rg -q 'curl: \(22\).*503' "$fixture/report"
rg -q '^installer_status=22$' "$evidence/status.txt"
rg -q '<url-redacted>' "$evidence/.attempt/install.log"
rg -q '<private key omitted>' "$evidence/.attempt/install.log"
rg -q '<sensitive diagnostic line omitted>' "$evidence/.attempt/install.log"
if rg -i 'url-secret|password@example|header-secret|assignment-secret|tokenSecret123|environment-secret|private-material|bearer-value|payload-value|api-environment-value|access-environment-value|labelled-api-value' "$evidence" "$fixture/report"; then exit 1; fi
rg -q '::notice title=setup::diagnostic message' "$evidence/.attempt/install.log"
rg -q ': :notice title=setup: :diagnostic message' "$fixture/report"
if rg -q '::notice|::diagnostic' "$fixture/report"; then exit 1; fi
[[ "$(find "$evidence" -type f | wc -l)" -eq 2 ]]
[[ -f "$fixture/early/entropy-tools/.gitleaks-install.supplier/archive.tar.gz" ]]
rg -q 'No installer output' "$fixture/no-output/scanner-setup/.attempt/install.log"
rg -q '<log limited to 64 KiB' "$fixture/large/scanner-setup/.attempt/install.log"
[[ "$(wc -c < "$fixture/large/scanner-setup/.attempt/install.log")" -le 65536 ]]
rg -q 'oversized diagnostic line omitted' "$fixture/long-line/scanner-setup/.attempt/install.log"
rg -q 'checksum mismatch' "$fixture/long-line/scanner-setup/.attempt/install.log"
rg -q '^installer_status=0$' "$fixture/collector-fails/scanner-setup/status.txt"
rg -q '^collector_status=71$' "$fixture/original-wins/scanner-setup/status.txt"
if perl scripts/ci/scanner-setup-log.pl report "$fixture/collector-fails/scanner-setup" > "$fixture/collector-report" 2>&1; then exit 1; fi
rg -q 'installer_status=0' "$fixture/collector-report"
rg -q 'collector_status=71' "$fixture/collector-report"

# Execute the workflow's real report command, with artifact effects substituted.
"$root/.tools/host/bin/yq" -r '.jobs.secrets.steps[] | select(.name == "Report scanner setup evidence") | .run' \
    .github/workflows/ci.yml > "$fixture/workflow-report.sh"
RUNNER_TEMP="$fixture/success" ARTIFACT_OUTCOME=success ARTIFACT_URL=https://example.invalid/artifact/123 \
    GITHUB_STEP_SUMMARY="$fixture/summary" bash "$fixture/workflow-report.sh" > "$fixture/workflow-report.log"
rg -q '\[Scanner setup diagnostics\]\(https://example.invalid/artifact/123\)' "$fixture/summary"
for outcome in failure missing-link; do
    upload_outcome=success
    [[ "$outcome" != failure ]] || upload_outcome=failure
    if RUNNER_TEMP="$fixture/success" ARTIFACT_OUTCOME="$upload_outcome" ARTIFACT_URL='' \
        GITHUB_STEP_SUMMARY="$fixture/summary" bash "$fixture/workflow-report.sh" > "$fixture/$outcome.log" 2>&1; then exit 1; fi
    rg -q 'artifact upload failed or produced no link' "$fixture/$outcome.log"
done
rm "$evidence/.attempt/install.log"
if perl scripts/ci/scanner-setup-log.pl report "$evidence" > /dev/null 2>&1; then exit 1; fi
if RUNNER_TEMP="$fixture/early" ARTIFACT_OUTCOME=success ARTIFACT_URL=https://example.invalid/artifact/123 \
    GITHUB_STEP_SUMMARY="$fixture/summary" bash "$fixture/workflow-report.sh" > /dev/null 2>&1; then exit 1; fi
ln -s "$fixture/early/entropy-tools/.gitleaks-install.supplier/archive.tar.gz" "$evidence/.attempt/install.log"
if perl scripts/ci/scanner-setup-log.pl report "$evidence" > /dev/null 2>&1; then exit 1; fi
printf 'Setup status, sanitized hidden evidence, byte bounds, missing evidence, collector/upload failures and artifact links passed.\n'
complete=true
