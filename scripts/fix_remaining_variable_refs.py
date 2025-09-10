#!/usr/bin/env python3
"""
Sprint 92 Final Fix: Fix remaining variable references in property tests
Some files still have `x` instead of `_x` in prop_assert! calls
"""

import os
import re
from pathlib import Path

def fix_variable_refs_in_file(file_path):
    """Fix remaining variable references in property tests"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Fix patterns where we have _x parameter but x reference
        patterns = [
            (r'fn module_consistency_check\(_x in 0u32\.\.1000\) \{\s*\/\/ Module consistency verification\s*prop_assert!\(x < 1001\);',
             r'fn module_consistency_check(_x in 0u32..1000) {\n            // Module consistency verification\n            prop_assert!(_x < 1001);'),
        ]
        
        fixed_content = content
        changed = False
        
        for old_pattern, new_pattern in patterns:
            if re.search(old_pattern, fixed_content, re.MULTILINE):
                fixed_content = re.sub(old_pattern, new_pattern, fixed_content, flags=re.MULTILINE)
                changed = True
        
        # Simpler pattern for just the prop_assert line
        if 'prop_assert!(x < 1001);' in fixed_content and '_x in 0u32..1000' in fixed_content:
            fixed_content = fixed_content.replace('prop_assert!(x < 1001);', 'prop_assert!(_x < 1001);')
            changed = True
        
        if changed:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(fixed_content)
            return True
        return False
        
    except Exception as e:
        print(f"Error fixing {file_path}: {e}")
        return False

def main():
    print("🔧 Sprint 92: Final Variable Reference Fixes")
    print("=" * 45)
    
    # Change to project root
    os.chdir("/home/noah/src/paiml-mcp-agent-toolkit")
    
    # Find all Rust files with the issue
    server_dir = Path("server/src")
    files_fixed = 0
    
    for rust_file in server_dir.rglob("*.rs"):
        try:
            with open(rust_file, 'r', encoding='utf-8') as f:
                content = f.read()
                if '_x in 0u32..1000' in content and 'prop_assert!(x < 1001)' in content:
                    if fix_variable_refs_in_file(rust_file):
                        files_fixed += 1
                        print(f"  ✅ Fixed: {rust_file}")
        except Exception as e:
            continue
    
    print(f"\n📊 Final Fix Results:")
    print(f"  • Files fixed: {files_fixed}")
    
    # Test compilation
    print(f"\n🧪 Testing final compilation...")
    import subprocess
    result = subprocess.run(["cargo", "check", "--package", "pmat"], 
                          capture_output=True, text=True)
    
    if result.returncode == 0:
        print("  ✅ Compilation successful!")
        
        # Count warnings (only real code warnings, not build process)
        warning_lines = [line for line in result.stderr.split('\n') 
                        if 'warning:' in line and 'pmat@' not in line and line.strip()]
        warning_count = len(warning_lines)
        
        if warning_count == 0:
            print("  🏆 PERFECT: ZERO WARNINGS!")
            
            # Test library tests
            print("\n🧪 Testing library tests...")
            test_result = subprocess.run(["cargo", "test", "--package", "pmat", "--lib", "--quiet"], 
                                       capture_output=True, text=True, timeout=30)
            
            if test_result.returncode == 0:
                print("  ✅ ALL LIBRARY TESTS PASS!")
                print("  🏆 ABSOLUTE BUILD STABILITY ACHIEVED!")
                return True
            else:
                print(f"  ❌ Some tests failed:")
                print(test_result.stderr[:500])
        else:
            print(f"  📊 Remaining warnings: {warning_count}")
            for warning in warning_lines[:3]:
                print(f"      {warning}")
    else:
        print(f"  ❌ Compilation failed:")
        print(result.stderr[:500])
    
    return False

if __name__ == "__main__":
    success = main()
    if success:
        print(f"\n🎯 Sprint 92 Status: BUILD STABILITY PERFECTION ACHIEVED!")
    else:
        print(f"\n🎯 Sprint 92 Status: Still working toward perfection...")