#!/usr/bin/env perl
# A bounded setup-only projection; never retain download archives or scan output.
# Keep completed diagnostic lines on disk while setup runs. SetupDiagnostics currently
# returns its excerpt only after completion, so it cannot replace this live log.
use strict;
use warnings;
use Fcntl qw(:DEFAULT S_ISREG);
use IO::Handle;

my ($mode, $path) = @ARGV;
@ARGV == 2 && ($mode eq 'capture' || $mode eq 'report')
    or die "usage: scanner-setup-log.pl capture LOG | report DIRECTORY\n";
my $limit = 64 * 1024;

sub read_regular {
    my ($file, $maximum) = @_;
    sysopen my $in, $file, O_RDONLY | O_NONBLOCK | O_NOFOLLOW
        or die "Scanner setup evidence missing/unreadable: $file\n";
    S_ISREG((stat $in)[2]) or die "Scanner setup evidence is not regular: $file\n";
    my $size = read($in, my $bytes, $maximum + 1);
    defined($size) && $size <= $maximum or die "Scanner setup evidence exceeds its limit: $file\n";
    close $in or die "Scanner setup evidence read failed: $file\n";
    return $bytes;
}

if ($mode eq 'report') {
    -d $path && !-l $path && -d "$path/.attempt" && !-l "$path/.attempt"
        or die "Scanner setup evidence directory missing or unsafe: $path\n";
    my $status = read_regular("$path/status.txt", 1024);
    $status =~ /\Atool=gitleaks\nphase=installation\ninstaller_status=(\d+|incomplete)\ncollector_status=(\d+|incomplete)\nstep_status=(\d+|running)\n\z/
        or die "Scanner setup status is malformed\n";
    my ($collector, $step) = ($2, $3);
    print "Scanner setup evidence: $path\n";
    print "[scanner setup] $_\n" for split /\n/, $status;
    my $log = read_regular("$path/.attempt/install.log", $limit);
    my $excerpt = length($log) > 4096 ? substr($log, -4096) : $log;
    print "[scanner setup] $_\n" for split /\n/, $excerpt;
    print "Excerpt limited to the last 4 KiB; see the retained setup log.\n" if length($log) > 4096;
    $collector eq '0' && $step ne 'running'
        or die "Scanner setup diagnostic collection failed or is incomplete; inspect status.txt\n";
    exit 0;
}

sysopen my $out, $path, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW, 0600
    or die "Cannot retain scanner setup diagnostics: $path\n";
$out->autoflush(1);
my @secrets = sort { length($b) <=> length($a) }
    map { $ENV{$_} } grep {
        /TOKEN|SECRET|PASSWORD|PASSWD|PRIVATE_KEY|CREDENTIAL|ACCESS_KEY|AUTHORIZATION/i
            && length($ENV{$_}) >= 4
    } keys %ENV;
my ($pending, $discarding, $private_key, $written, $truncated, $received) = ('', 0, 0, 0, 0, 0);
sub retain {
    my ($line) = @_;
    if (!$truncated && $written + length($line) <= $limit - 100) {
        print {$out} $line or die "Scanner setup diagnostic write failed\n";
        $written += length($line);
    } else { $truncated = 1; }
}
sub sanitize {
    my ($line) = @_;
    if ($line =~ /-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----/) {
        $private_key = 1;
        retain("<private key omitted>\n");
    }
    if ($private_key) {
        $private_key = 0 if $line =~ /-----END [A-Z0-9 ]*PRIVATE KEY-----/;
        return;
    }
    $line =~ s/\Q$_\E/<redacted>/g for @secrets;
    $line =~ s{https?://[^\s<>"']+}{<url-redacted>}gi;
    $line =~ s{\b(?:gh[pousr]_[A-Za-z0-9_]+|github_pat_[A-Za-z0-9_]+)\b}{<redacted>}g;
    $line =~ s{(\b(?:Proxy-)?Authorization\s*[:=]\s*)[^\r\n]*}{$1<redacted>}gi;
    $line =~ s{("?[\w-]*(?:token|password|passwd|secret|credential|private[_-]?key|access[_-]?key)[\w-]*"?\s*[:=]\s*)(?:"[^"\n]*"|'[^'\n]*'|[^\s,;]+)}{$1<redacted>}gi;
    $line =~ s{([^\x09\x0a\x20-\x7e])}{sprintf('\\x%02x', ord $1)}ge;
    retain(length($line) <= 4096 ? $line : "<oversized diagnostic line omitted>\n");
}
while (1) {
    my $size = sysread(STDIN, my $chunk, 4096);
    defined $size or die "Scanner setup diagnostic read failed\n";
    last unless $size;
    $received += $size;
    $pending .= $chunk;
    while ((my $end = index($pending, "\n")) >= 0) {
        my $line = substr($pending, 0, $end + 1, '');
        if ($discarding || length($line) > 4096) {
            $private_key = 1 if $line =~ /-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----/;
            retain("<oversized diagnostic line omitted>\n");
        }
        else { sanitize($line); }
        $discarding = 0;
    }
    if (length($pending) > 4096) {
        $private_key = 1 if $pending =~ /-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----/;
        $pending = '';
        $discarding = 1;
    }
}
if ($discarding) { retain("<oversized diagnostic line omitted>\n"); }
elsif (length $pending) { sanitize("$pending\n"); }
retain("No installer output was produced.\n") unless $received;
if ($truncated) {
    print {$out} "<log limited to 64 KiB; further output omitted>\n"
        or die "Scanner setup diagnostic write failed\n";
}
close $out or die "Scanner setup diagnostic close failed\n";
