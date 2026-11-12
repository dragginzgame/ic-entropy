#!/usr/bin/env perl
use strict;
use warnings;
use File::Find;
use Digest::SHA;
use JSON::PP;
my $mode=shift // die "mode required\n";
$mode eq 'restore' || $mode eq 'seal' or die "invalid mode\n";
my $json=JSON::PP->new->canonical;
my (%files,$total);
for my $root (qw(target/debug target/release target/wasm32-unknown-unknown target/qualification)) {
    next unless -e $root || -l $root;
    -d $root && !-l $root or die "unsafe build cache root\n";
    find({no_chdir=>1,wanted=>sub {
        my $file=$File::Find::name;
        !-l $file or die "symlink in compilation cache\n";
        return if -d $file;
        -f $file or die "nonregular compilation cache\n";
        $total+=-s $file;
        $total <= 4294967296 or die "compilation cache exceeds 4 GiB\n";
        open my $in,'<:raw',$file or die $!;
        my $digest=Digest::SHA->new(256)->addfile($in)->hexdigest;
        close $in or die $!;
        $files{$file}={sha256=>$digest,mode=>(stat($file))[2]&07777};
    }},$root);
}
my $record=$json->encode({schema=>1,files=>\%files})."\n";
my $path='target/ci-build-cache.json';
if ($mode eq 'seal') {
    keys(%files) or die "no compilation outputs\n";
    open my $out,'>:raw',$path or die $!; print {$out} $record or die $!; close $out or die $!;
} else {
    -f $path && !-l $path && -s $path<=16777216 or die "missing/unsafe compilation receipt\n";
    open my $in,'<:raw',$path or die $!; local $/; my $prior=<$in>; close $in or die $!;
    $prior eq $record or die "compilation cache bytes/modes/inventory changed\n";
}
