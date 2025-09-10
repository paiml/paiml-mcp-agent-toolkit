#!/usr/bin/env python3
"""
Fix missing braces in daemon.rs file.
Sprint 89: Complete daemon.rs syntax fixes.
"""

import re
from pathlib import Path

def fix_daemon_braces():
    file_path = Path("server/src/agent/daemon.rs")
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    original_content = content
    
    # Strategy: Find all function definitions and ensure they have proper closing braces
    # Look for patterns like:
    # pub fn function_name(...) -> ReturnType {
    #   ... code ...
    # (missing })
    # next function/impl
    
    lines = content.split('\n')
    fixed_lines = []
    i = 0
    brace_count = 0
    in_function = False
    function_start_line = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Track braces
        open_braces = line.count('{')
        close_braces = line.count('}')
        brace_count += open_braces - close_braces
        
        # Check if this is a function definition
        if re.match(r'\s*(pub\s+)?(async\s+)?fn\s+\w+', line) and line.endswith('{'):
            in_function = True
            function_start_line = i
            function_brace_count = 1  # The opening brace
        elif in_function:
            function_brace_count += open_braces - close_braces
            if function_brace_count == 0:
                in_function = False
        
        # Check if we need to add a closing brace before the next function/impl
        if (i < len(lines) - 1 and 
            in_function and 
            function_brace_count > 0 and
            (re.match(r'\s*(pub\s+)?(async\s+)?fn\s+\w+', lines[i + 1]) or
             re.match(r'\s*impl\s+', lines[i + 1]) or
             re.match(r'\s*#\[', lines[i + 1]) or
             lines[i + 1].strip() == '}' or
             re.match(r'\s*mod\s+', lines[i + 1]))):
            # Add missing closing braces
            missing_braces = function_brace_count
            fixed_lines.append(line)
            for _ in range(missing_braces):
                fixed_lines.append('    }')
            in_function = False
        else:
            fixed_lines.append(line)
        
        i += 1
    
    # Join lines back
    fixed_content = '\n'.join(fixed_lines)
    
    # Additional cleanup - ensure impl blocks are closed properly
    # Count impl block braces
    impl_pattern = r'impl\s+(?:\w+\s+for\s+)?\w+'
    impl_matches = list(re.finditer(impl_pattern, fixed_content))
    
    # Simple fix: ensure each impl block has proper closing
    if impl_matches:
        lines = fixed_content.split('\n')
        final_lines = []
        impl_brace_count = 0
        
        for line in lines:
            final_lines.append(line)
            if re.match(r'\s*impl\s+', line):
                impl_brace_count = 1
            elif impl_brace_count > 0:
                impl_brace_count += line.count('{') - line.count('}')
        
        # If impl block is not closed, close it
        if impl_brace_count > 0:
            final_lines.append('}')
        
        fixed_content = '\n'.join(final_lines)
    
    # Save if changed
    if fixed_content != original_content:
        with open(file_path, 'w') as f:
            f.write(fixed_content)
        return True
    return False

def main():
    print("🔧 Fixing daemon.rs braces...")
    
    if fix_daemon_braces():
        print("✅ Fixed daemon.rs braces")
    else:
        print("ℹ️  No changes needed for daemon.rs")
    
    # Test compilation
    import os
    print("\n🧪 Testing compilation...")
    result = os.system("cargo build --package pmat --tests --quiet")
    if result == 0:
        print("✅ All tests compile successfully!")
        print("🎯 Sprint 89: Property test compilation COMPLETE")
    else:
        print("❌ Compilation issues remain")
        os.system("cargo build --package pmat --tests 2>&1 | grep -A3 'unclosed delimiter' | head -10")

if __name__ == "__main__":
    main()