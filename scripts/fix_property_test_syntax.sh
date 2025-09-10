#!/bin/bash

# Fix syntax issues in property test files after bulk upgrade
# Sprint 89: Clean up formatting issues from automated upgrades

echo "🔧 Fixing property test syntax issues..."

# Find all Rust files with potential duplicate closing braces
for file in $(find server/src -name "*.rs" -type f); do
    # Check if file has the problematic pattern
    if grep -q "}\s*}\s*}$" "$file" 2>/dev/null; then
        echo "Fixing: $file"
        
        # Create temp file with fix
        sed -E '
            # Fix the pattern where test is outside proptest! macro
            /^[[:space:]]*#\[test\]/,/^[[:space:]]*}[[:space:]]*$/ {
                # If we see a closing brace followed by another closing brace
                /^[[:space:]]*}[[:space:]]*$/ {
                    N
                    # Check if next line is also a closing brace
                    /}\n[[:space:]]*}/ {
                        # Delete the duplicate
                        s/}\n[[:space:]]*}/}/
                    }
                }
            }
        ' "$file" > "$file.tmp"
        
        # Only replace if different
        if ! diff -q "$file" "$file.tmp" > /dev/null; then
            mv "$file.tmp" "$file"
            echo "  ✅ Fixed duplicate braces"
        else
            rm "$file.tmp"
        fi
    fi
    
    # Fix tests that are outside proptest! macro
    if grep -A2 "fn valid_.*_input()" "$file" | grep -q "#\[test\]"; then
        echo "Fixing test placement in: $file"
        
        # This is complex, so use a more targeted approach
        perl -i -pe '
            # Track if we are inside proptest! macro
            if (/proptest!\s*{/) { $in_proptest = 1; }
            if ($in_proptest && /^\s*}\s*$/) { 
                # Look ahead to see if there is a test after the closing brace
                $next_line = <>;
                if ($next_line && $next_line =~ /fn valid_/) {
                    # We found the helper function, check for orphaned test
                    $helper = $next_line;
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
        ' "$file" 2>/dev/null || true
    fi
done

echo "✅ Syntax fixes complete"