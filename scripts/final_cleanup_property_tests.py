#!/usr/bin/env python3
"""
Final cleanup of all property test syntax issues.
Sprint 89: Complete property test compilation fix.
"""

import os
import re
from pathlib import Path

def fix_property_test_syntax(file_path):
    """Fix property test syntax in a file."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        original_content = content
        
        # Pattern 1: Fix #[cfg(test)] fn prop_tests() { to proptest! {
        pattern1 = r'#\[cfg\(test\)\] fn prop_tests\(\) \{'
        if re.search(pattern1, content):
            # Replace the problematic pattern
            content = re.sub(pattern1, 'proptest! {', content)
            
            # Fix function declarations inside
            content = re.sub(
                r'(\s+)// #\[test\]\s*\n(\s+)fn (\w+)\(([^)]*)\) \{',
                r'\1#[test]\n\2fn \3(\4) {',
                content
            )
            
            # Fix assert! to prop_assert!
            content = re.sub(
                r'(\s+)assert!\(([^)]+)\);',
                r'\1prop_assert!(\2);',
                content
            )
            
            # Ensure proper module closing
            # Find the property_tests module
            if 'mod property_tests {' in content:
                # Add closing brace if missing
                lines = content.split('\n')
                in_property_mod = False
                brace_count = 0
                needs_closing = False
                
                for i, line in enumerate(lines):
                    if 'mod property_tests {' in line:
                        in_property_mod = True
                        brace_count = 1
                    elif in_property_mod:
                        brace_count += line.count('{') - line.count('}')
                        if brace_count == 0:
                            in_property_mod = False
                        elif i == len(lines) - 1 and brace_count > 0:
                            needs_closing = True
                
                if needs_closing:
                    content += '\n}'
        
        # Pattern 2: Fix parameter syntax like (input: &str) to (input in ".*")
        content = re.sub(
            r'fn (\w+)\(input: &str\) \{',
            r'fn \1(input in ".*") {',
            content
        )
        
        content = re.sub(
            r'fn (\w+)\(x: u32\) \{',
            r'fn \1(x in 0u32..1000) {',
            content
        )
        
        # Save if changed
        if content != original_content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(content)
            return True
        return False
    
    except Exception as e:
        print(f"❌ Error processing {file_path}: {e}")
        return False

def main():
    print("🔧 Final cleanup: Property test syntax fixes")
    
    # Find all .rs files with potential issues
    fixed_count = 0
    
    for file_path in Path("server/src").rglob("*.rs"):
        if fix_property_test_syntax(file_path):
            print(f"✅ Fixed: {file_path}")
            fixed_count += 1
    
    print(f"📊 Summary: Fixed {fixed_count} files")
    
    # Test compilation
    print("\n🧪 Testing compilation...")
    result = os.system("cargo build --package pmat --tests --quiet")
    if result == 0:
        print("✅ All tests compile successfully!")
        print("🎯 Sprint 89: Property test compilation COMPLETE")
    else:
        print("❌ Some compilation issues remain")
        print("🔍 Checking remaining errors...")
        os.system("cargo build --package pmat --tests 2>&1 | grep 'error:' | head -3")

if __name__ == "__main__":
    main()