#!/usr/bin/env bash
# Keep setup evidence separate from scanner findings and downloaded executables.
set -euo pipefail
umask 077
[[ "${RUNNER_TEMP:-}" == /* && -d "$RUNNER_TEMP" && ! -L "$RUNNER_TEMP" ]] || exit 2
case "$RUNNER_TEMP" in *$'\r'*|*$'\n'*) exit 2 ;; esac
evidence="$RUNNER_TEMP/scanner-setup"
mkdir "$evidence"
mkdir "$evidence/.attempt"
installer_status=incomplete collector_status=incomplete
write_status() {
    printf 'tool=gitleaks\nphase=installation\ninstaller_status=%s\ncollector_status=%s\nstep_status=%s\n' \
        "$installer_status" "$collector_status" "$1" > "$evidence/status.txt"
}
write_status running
# shellcheck disable=SC2329 # Invoked by the EXIT trap.
finish() {
    local status=$?
    trap - EXIT
    if ! write_status "$status"; then
        echo 'Scanner setup status retention failed.' >&2
        [[ "$status" != 0 ]] || status=1
    fi
    exit "$status"
}
trap finish EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
set +e
bash vendor/toko-tooling/scripts/ci/install-ci-tool.sh gitleaks \
    --install-dir "$RUNNER_TEMP/entropy-tools" 2>&1 |
    perl scripts/ci/scanner-setup-log.pl capture "$evidence/.attempt/install.log"
statuses=("${PIPESTATUS[@]}")
set -e
installer_status="${statuses[0]}" collector_status="${statuses[1]}"
[[ "$installer_status" == 0 ]] || exit "$installer_status"
exit "$collector_status"
