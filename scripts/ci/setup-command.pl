#!/usr/bin/env perl
# Consumer paths and artifacts; shared tooling owns capture and sanitization.
use strict;
use warnings;
use Cwd qw(abs_path);
use Fcntl qw(O_RDONLY O_NONBLOCK O_NOFOLLOW O_WRONLY O_CREAT O_EXCL S_ISREG);
use FindBin qw($RealBin);
use lib "$RealBin/../../vendor/toko-tooling/scripts/ci";
use SetupDiagnostics qw(run_setup);

my @phases = qw(compiler fetch-product fetch-qualification);
my $temporary = $ENV{RUNNER_TEMP} // '';
$temporary =~ m{\A/} && $temporary !~ /[\x00\r\n]/ && -d $temporary && !-l $temporary
    or die "Expected an existing absolute RUNNER_TEMP directory.\n";
my $root = abs_path($temporary) . '/early-setup';
my $mode = shift(@ARGV) // '';

sub directory {
    -d $_[0] && !-l $_[0] or die "Unsafe setup evidence directory.\n";
}

sub write_record {
    my ($path, $text, $limit) = @_;
    length($text) <= $limit or die "Setup evidence exceeds its bound.\n";
    directory($root);
    my $parent = $path; $parent =~ s{/[^/]+\z}{};
    directory($parent);
    my $temporary = "$path.write-$$";
    sysopen my $out, $temporary, O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW, 0600
        or die "Cannot create setup evidence.\n";
    binmode $out;
    print {$out} $text or die "Cannot write setup evidence.\n";
    close $out or die "Cannot close setup evidence.\n";
    rename $temporary, $path or die "Cannot publish setup evidence.\n";
}

sub read_record {
    my ($path, $limit) = @_;
    sysopen my $in, $path, O_RDONLY | O_NONBLOCK | O_NOFOLLOW
        or die "Missing setup evidence.\n";
    S_ISREG((stat $in)[2]) or die "Nonregular setup evidence.\n";
    my $count = read($in, my $text, $limit + 1);
    defined($count) && $count <= $limit or die "Oversized setup evidence.\n";
    close $in or die "Cannot close setup evidence.\n";
    return $text;
}

if ($mode eq 'report' && !@ARGV) {
    my $valid = eval {
        directory($root);
        my $required = 1;
        for my $phase (@phases) {
            my $path = "$root/$phase";
            if (!-e $path && !-l $path) {
                die "Missing setup phase.\n" if $required;
                next;
            }
            directory($path);
            directory("$path/.attempt");
            my $status = read_record("$path/status.txt", 1024);
            $status =~ /\Aphase=\Q$phase\E\ncommand_status=(\d+|running)\ncollection_status=(passed|skipped|incomplete|failed)\n\z/
                or die "Malformed setup status.\n";
            my ($command, $collection) = ($1, $2);
            print "Early setup $phase: command status $command; collection $collection\n";
            print "Evidence: $path\n", read_record("$path/.attempt/diagnostic.log", 32768);
            $command ne 'running' && ($collection eq 'passed' || $collection eq 'skipped')
                or die "Incomplete setup evidence.\n";
            if ($phase eq 'compiler' && $command eq '0') {
                print read_record("$path/identity.log", 32768);
            }
            $required = $command eq '0';
        }
        1;
    };
    warn "Early setup evidence is missing, incomplete or invalid.\n" unless $valid;
    my $uploaded = ($ENV{ARTIFACT_OUTCOME} // '') eq 'success'
        && ($ENV{ARTIFACT_URL} // '') =~ m{\Ahttps://github[.]com/[^\s]+\z};
    if ($uploaded) {
        my $summary = $ENV{GITHUB_STEP_SUMMARY} // '';
        open my $out, '>>', $summary or die "Cannot write setup artifact link.\n";
        print {$out} "[Setup diagnostics]($ENV{ARTIFACT_URL})\n" or die "Cannot write setup artifact link.\n";
        close $out or die "Cannot close setup summary.\n";
    } else {
        warn "Setup artifact upload failed or produced no link.\n";
    }
    exit($valid && $uploaded ? 0 : 1);
}

grep($_ eq $mode, @phases) && @ARGV or die "Select compiler, fetch-product or fetch-qualification and a command, or report.\n";
umask 077;
if (!-e $root && !-l $root) { mkdir $root, 0700 or die "Cannot create setup evidence root.\n"; }
directory($root);
my $path = "$root/$mode";
mkdir $path, 0700 or die "Setup evidence already exists or cannot be created.\n";
mkdir "$path/.attempt", 0700 or die "Cannot create setup attempt.\n";
my $status_file = "$path/status.txt";
my $log_file = "$path/.attempt/diagnostic.log";
write_record($status_file, "phase=$mode\ncommand_status=running\ncollection_status=incomplete\n", 1024);
write_record($log_file, "[setup command has not completed]\n", 32768);
my $collected = 0;
my $status = run_setup(sub {
    my ($original, $excerpt) = @_;
    directory($path);
    directory("$path/.attempt");
    write_record($log_file, $excerpt, 32768);
    $collected = 1;
    print STDERR "Early setup failed: phase=$mode status=$original evidence=$path\n", $excerpt;
}, @ARGV);
my $collection = !$status ? 'skipped' : $collected ? 'passed' : 'failed';
my $retained = eval {
    directory($path);
    write_record($log_file, "[setup completed successfully]\n", 32768) unless $status;
    write_record($status_file, "phase=$mode\ncommand_status=$status\ncollection_status=$collection\n", 1024);
    1;
};
warn "Setup status retention failed; original command status is $status.\n" unless $retained;
exit($status || ($retained ? 0 : 1));
