#!/usr/bin/env perl
# Consumer closure for the fixed native Make roster. No version projection.
use strict;
use warnings;
use Cwd qw(abs_path getcwd);
use File::Find;
use JSON::PP;
my $json=JSON::PP->new->canonical;
sub command {
    open my $in,'-|',@_ or die $!;
    local $/; my $out=<$in>; close $in or die "observation failed: @_\n";
    return $out;
}
my $root=getcwd();
my (%files,%trees,%links,%modes);
sub tree {
    my ($path)=@_;
    my $resolved=abs_path($path) // die "missing input: $path\n";
    $links{$path}=$resolved if $resolved ne $path;
    return if $trees{$resolved}++;
    if (-f $resolved) { $files{$resolved}=1; return; }
    -d $resolved or die "nonregular input: $resolved\n";
    find({no_chdir=>1,wanted=>sub {
        my $name=$File::Find::name;
        if ($resolved eq $root && ($name eq "$root/.git" || $name eq "$root/.tools" || $name eq "$root/target" || $name eq "$root/qualification/target")) {
            $File::Find::prune=1; return;
        }
        if (-l $name) { $File::Find::prune=1; tree($name); return; }
        return if -d $name;
        -f $name or die "nonregular stage input: $name\n";
        $files{$name}=1;
    }},$resolved);
}
# Include untracked/new files as well as tracked source; target is output only.
tree($root);
my $cargo_home=$ENV{CARGO_HOME}//"$ENV{HOME}/.cargo";
for my $dir ($root,"$root/qualification",$cargo_home) {
    for my $file ("$dir/config","$dir/config.toml","$dir/.cargo/config","$dir/.cargo/config.toml") {
        tree($file) if -e $file;
    }
}
# Cargo reads configuration from every ancestor of the invocation directory.
my $ancestor=$root;
while ($ancestor =~ s{/[^/]+\z}{}) {
    for my $file ("$ancestor/.cargo/config","$ancestor/.cargo/config.toml") { tree($file) if -e $file; }
}
for my $manifest ('Cargo.toml','qualification/Cargo.toml') {
    my $metadata=$json->decode(command('cargo','metadata','--locked','--offline','--all-features','--format-version','1','--manifest-path',$manifest));
    for my $package (@{$metadata->{packages}}) {
        my $path=$package->{manifest_path}; $path =~ s{/Cargo.toml\z}{} or die "invalid manifest\n";
        tree($path) unless index($path,"$root/")==0;
    }
}
# Inventory selected executable bundles and receipts, not mutable build caches.
tree("$root/.tools/host");
tree("$root/.tools/rust/common");
for my $selector (glob("$root/.tools/rust/selected-*")) { tree($selector); }
tree("$root/.tools/pocket-ic");
my $sysroot=command('rustc','--print','sysroot'); chomp $sysroot;
tree("$sysroot/bin"); tree("$sysroot/lib");
command('uname','-s') eq "Linux\n" or die "native reuse only qualified for Linux\n";
tree('/usr/include');
tree('/usr/lib/gcc');
for my $tool (qw(bash make cargo rustc rustup cc as ld ar git perl jq yq env uname date stat file sed awk sort sha256sum shasum readlink dirname basename head tail wc find ls tr cut cat cp cmp mv rm mkdir mktemp tee tar gzip ldd)) {
    my ($path)=grep {-f $_ && -x $_} map {"$_/$tool"} split /:/,$ENV{PATH};
    defined $path or die "missing tool $tool\n"; tree($path);
    # Include the dynamic loader and shared libraries used by native executables.
    open my $ldd,'-|','ldd',$path or die "cannot inspect native libraries\n";
    my $dependencies=do { local $/; <$ldd> }; close $ldd;
    for my $library ($dependencies =~ m{(/[^\s()]+)}g) { tree($library) if -f $library; }
}
for my $binary (glob("$root/.tools/host/bin/*"),glob("$root/.tools/rust/common/bin/*"),glob("$root/.tools/rust/selected-*/bin/*"),glob("$root/.tools/pocket-ic/*/pocket-ic")) {
    next unless -f $binary && -x $binary;
    open my $ldd,'-|','ldd',$binary or die $!;
    my $dependencies=do {local $/; <$ldd>}; close $ldd;
    for my $library ($dependencies =~ m{(/[^\s()]+)}g) { tree($library) if -f $library; }
}
for my $path (glob('/lib/*/libc.so.*'),glob('/lib/*/libm.so.*'),glob('/lib/*/libgcc_s.so.*'),glob('/lib/*/libpthread.so.*')) { tree($path); }
tree($_) for values %INC;
for my $file (keys %files) { $modes{$file}=(stat($file))[2] & 07777; }
print $json->encode({schema=>1,files=>[sort keys %files],configuration=>{
    contract=>'ic-entropy-native/1',links=>\%links,modes=>\%modes,
    compiler=>command('rustc','-vV'),cargo=>command('cargo','-vV'),cc=>command('cc','--version'),
}}),"\n";
