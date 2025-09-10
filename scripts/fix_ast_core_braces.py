#!/usr/bin/env python3
"""
Systematic fix for all missing braces in ast/core.rs
Sprint 90: Complete property test compilation success
"""

import re
from pathlib import Path

def fix_ast_core_braces():
    file_path = Path("server/src/ast/core.rs")
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    original_content = content
    lines = content.split('\n')
    fixed_lines = []
    
    i = 0
    while i < len(lines):
        line = lines[i]
        
        # Look for function definitions and test patterns that commonly miss braces
        if (re.match(r'\s*(pub\s+)?fn\s+\w+.*\{', line) or 
            re.match(r'\s*#\[test\]\s*$', line)):
            
            # This is a function or test definition
            # Find the next function/test/impl/mod to see if we need to insert a brace
            j = i + 1
            brace_count = line.count('{') - line.count('}')
            
            # Process the function body
            while j < len(lines) and brace_count > 0:
                next_line = lines[j]
                brace_count += next_line.count('{') - next_line.count('}')
                j += 1
            
            # Check if the next line after this function is another function/test without proper closing
            if (j < len(lines) and brace_count == 0 and
                (re.match(r'\s*(pub\s+)?fn\s+\w+', lines[j]) or
                 re.match(r'\s*#\[test\]\s*$', lines[j]) or
                 re.match(r'\s*impl\s+', lines[j]) or
                 re.match(r'\s*mod\s+', lines[j]) or
                 re.match(r'\s*pub\s+enum\s+', lines[j]) or
                 re.match(r'\s*pub\s+struct\s+', lines[j]))):
                
                # Add the current line and the function body
                fixed_lines.append(line)
                for k in range(i + 1, j):
                    fixed_lines.append(lines[k])
                
                # Check if we need to add a closing brace
                last_line = lines[j-1].strip() if j > i+1 else line.strip()
                if not last_line.endswith('}') and not last_line.endswith(';'):
                    fixed_lines.append('    }')
                
                i = j - 1  # Skip to just before the next function
            else:
                fixed_lines.append(line)
        else:
            fixed_lines.append(line)
        
        i += 1
    
    # Join lines back
    fixed_content = '\n'.join(fixed_lines)
    
    # Additional specific fixes for common patterns
    # Fix test functions that are missing closing braces before next test
    fixed_content = re.sub(
        r'(\s+)(prop_assert!\([^)]+\);)\s*\n\s*(#\[test\])',
        r'\1\2\n    }\n\n    \3',
        fixed_content,
        flags=re.MULTILINE
    )
    
    # Fix assert statements before next test
    fixed_content = re.sub(
        r'(\s+)(assert[^;]*;)\s*\n\s*(#\[test\])',
        r'\1\2\n    }\n\n    \3',
        fixed_content,
        flags=re.MULTILINE
    )
    
    # Save if changed
    if fixed_content != original_content:
        with open(file_path, 'w') as f:
            f.write(fixed_content)
        return True
    return False

def main():
    print("🔧 Sprint 90: Fixing ast/core.rs structural braces...")
    
    if fix_ast_core_braces():
        print("✅ Fixed ast/core.rs structural issues")
    else:
        print("ℹ️  No changes needed for ast/core.rs")
    
    # Test compilation
    import os
    print("\n🧪 Testing ast/core.rs compilation...")
    result = os.system("cargo build --package pmat --tests --quiet")
    if result == 0:
        print("✅ ast/core.rs compiles successfully!")
        print("🎯 Sprint 90: AST core fixes COMPLETE")
    else:
        print("❌ ast/core.rs still has compilation issues")
        os.system("cargo build --package pmat --tests 2>&1 | grep -A3 'ast/core.rs' | head -5")

if __name__ == "__main__":
    main()