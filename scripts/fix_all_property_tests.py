#!/usr/bin/env python3

"""
Fix property test syntax issues after bulk upgrade.
Sprint 89: Systematically fix test placement issues.
"""

import os
import re
from pathlib import Path

def fix_property_test_file(filepath):
    """Fix property test syntax issues in a single file."""
    with open(filepath, 'r') as f:
        content = f.read()
    
    original_content = content
    
    # Pattern to find the misplaced test (test outside proptest! macro)
    # This pattern looks for:
    # 1. End of proptest! macro (})
    # 2. Some whitespace
    # 3. A test function that should be inside
    pattern = r'(fn \w+_never_panics.*?\n\s*}\n\s*)\}\n\n\s*(#\[test\]\s*\n\s*fn module_consistency_check.*?\n\s*}\n\s*)\}'
    
    # Replacement puts the test inside the proptest! macro
    replacement = r'\1\n    \2}'
    
    # Apply the fix
    content = re.sub(pattern, replacement, content, flags=re.DOTALL)
    
    # Another pattern for the simpler case where the test is just misplaced
    pattern2 = r'(        }\n    )\}\n\n        (#\[test\].*?prop_assert.*?        }\n    )\}'
    replacement2 = r'\1\n        \2}'
    content = re.sub(pattern2, replacement2, content, flags=re.DOTALL)
    
    # Fix any remaining double closing braces
    content = re.sub(r'}\n\s*}\n}$', '}\n}', content)
    
    if content != original_content:
        with open(filepath, 'w') as f:
            f.write(content)
        return True
    return False

def main():
    """Fix all property test files."""
    server_src = Path('server/src')
    
    if not server_src.exists():
        print("Error: server/src directory not found")
        return
    
    fixed_count = 0
    checked_count = 0
    
    # Find all Rust files
    for filepath in server_src.rglob('*.rs'):
        if 'target' in str(filepath):
            continue
            
        checked_count += 1
        
        # Check if file has property tests
        with open(filepath, 'r') as f:
            content = f.read()
            
        if 'mod property_tests' in content and 'module_consistency_check' in content:
            if fix_property_test_file(filepath):
                print(f"✅ Fixed: {filepath}")
                fixed_count += 1
    
    print(f"\n📊 Summary:")
    print(f"  Files checked: {checked_count}")
    print(f"  Files fixed: {fixed_count}")
    
    if fixed_count == 0:
        print("  No issues found! ✨")

if __name__ == "__main__":
    main()