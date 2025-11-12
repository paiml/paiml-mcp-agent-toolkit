#!/usr/bin/env perl
# Fix tests that are outside proptest! macro
# Sprint 89: Move orphaned tests back inside proptest! blocks

use strict;
use warnings;

my $in_proptest = 0;

while (<>) {
    # Track if we are inside proptest! macro
    if (/proptest!\s*{/) { $in_proptest = 1; }

    if ($in_proptest && /^\s*}\s*$/) {
        # Look ahead to see if there is a test after the closing brace
        my $next_line = <>;
        if ($next_line && $next_line =~ /fn valid_/) {
            # We found the helper function, check for orphaned test
            my $helper = $next_line;
            $next_line = <>;
            if ($next_line && $next_line =~ /#\[test\]/) {
                # Found orphaned test, need to move it inside proptest!
                print;  # Print the closing brace
                print "\n";
                print $helper;  # Print helper function
                # Skip the orphaned test - it should already be inside
                while (<>) {
                    last if /^\s*}\s*$/;
                }
                $in_proptest = 0;
                next;
            } else {
                print;
                print $helper if $helper;
                print $next_line if $next_line;
            }
        } else {
            print;
            print $next_line if $next_line;
        }
        $in_proptest = 0;
    }
}
