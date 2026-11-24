#!/usr/bin/env bash
# Run through setup-command.pl so even bootstrap failures retain diagnostics.
set -euo pipefail
make --no-print-directory install-host-tools host-tools-check
export PATH="$PWD/.tools/host/bin:$PATH"
bash vendor/toko-tooling/scripts/dev/prepare-rust-toolchain.sh --toolchain-file rust-toolchain.toml
compiler="$(yq -p toml -o yaml -r '.toolchain.channel' rust-toolchain.toml)"
rustc -vV
cargo -vV
rustc_version="$(rustc --version)"
cargo_version="$(cargo --version)"
test "$(printf '%s\n' "$rustc_version" | cut -d ' ' -f 2)" = "$compiler"
test "$(printf '%s\n' "$cargo_version" | cut -d ' ' -f 2)" = "$compiler"
rustup component list --toolchain "$compiler" --installed
rustup target list --toolchain "$compiler" --installed
# Keep actual checked identities on success; the shared helper sanitizes and
# bounds the retained report just as it does failure excerpts.
printf '%s\n' "$rustc_version" "$cargo_version" |
  perl -I vendor/toko-tooling/scripts/ci -MSetupDiagnostics=excerpt \
    -MFcntl=O_WRONLY,O_CREAT,O_EXCL,O_NOFOLLOW -e '
      sysopen my $out, $ARGV[0], O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW, 0600
        or die "Cannot create compiler identity evidence.\n";
      print {$out} excerpt(\*STDIN) or die "Cannot write compiler identity evidence.\n";
      close $out or die "Cannot close compiler identity evidence.\n";
    ' "$RUNNER_TEMP/early-setup/compiler/identity.log"
