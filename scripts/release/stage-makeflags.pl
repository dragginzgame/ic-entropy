#!/usr/bin/env perl
# Accept only Make's display flags and release/publication routing assignments.
# Parallelism and all build-variable overrides retain ordinary execution.
use strict;
use warnings;
use Text::ParseWords qw(shellwords);
@ARGV==1 or exit 2;
my $assignments=0;
my @words=shellwords($ARGV[0]);
exit 1 if $ARGV[0] =~ /\S/ && !@words;
for my $word (@words) {
    if ($word eq '--') { $assignments=1; next; }
    next if !$assignments && ($word eq '--no-print-directory' || $word eq 'w');
    next if $assignments && $word =~ /\A(?:RELEASE_[A-Z_]+|CARGO_PUBLISH_[A-Z_]+)=/;
    exit 1;
}
