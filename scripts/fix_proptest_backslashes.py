#!/usr/bin/env python3
"""
Sprint 92 Fix: Remove invalid backslashes from proptest function names
The previous script incorrectly added backslashes before underscores
"""

import os
import re
from pathlib import Path

def fix_backslashes_in_file(file_path):
    """Remove backslashes from proptest function parameter names"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Fix the backslash issues
        fixes = [
            (r'fn basic_property_stability\\\(_input in', r'fn basic_property_stability(_input in'),
            (r'fn module_consistency_check\\\(_x in', r'fn module_consistency_check(_x in'),
        ]
        
        fixed_content = content
        changed = False
        
        for old_pattern, new_pattern in fixes:
            if re.search(old_pattern, fixed_content):
                fixed_content = re.sub(old_pattern, new_pattern, fixed_content)
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
    print("🔧 Sprint 92: Fixing Proptest Backslash Syntax Errors")
    print("=" * 55)
    
    # Change to project root
    os.chdir("/home/noah/src/paiml-mcp-agent-toolkit")
    
    # Find all Rust files with backslash issues
    server_dir = Path("server/src")
    files_fixed = 0
    
    for rust_file in server_dir.rglob("*.rs"):
        try:
            with open(rust_file, 'r', encoding='utf-8') as f:
                content = f.read()
                if "fn basic_property_stability\\(_input" in content or "fn module_consistency_check\\(_x" in content:
                    if fix_backslashes_in_file(rust_file):
                        files_fixed += 1
                        print(f"  ✅ Fixed: {rust_file}")
        except Exception as e:
            print(f"  ❌ Error reading {rust_file}: {e}")
            continue
    
    print(f"\n📊 Sprint 92 Backslash Fix Results:")
    print(f"  • Files fixed: {files_fixed}")
    
    # Test compilation
    print(f"\n🧪 Testing compilation...")
    import subprocess
    result = subprocess.run(["cargo", "check", "--package", "pmat", "--quiet"], 
                          capture_output=True, text=True)
    
    if result.returncode == 0:
        print("  ✅ Compilation successful!")
        
        # Check for warnings
        warn_result = subprocess.run(["cargo", "check", "--package", "pmat"], 
                                   capture_output=True, text=True)
        stderr_lines = warn_result.stderr.strip().split('\n')
        warning_lines = [line for line in stderr_lines if 'warning:' in line and 'pmat@' not in line]
        warning_count = len(warning_lines)
        
        print(f"  📊 Code warnings: {warning_count}")
        
        if warning_count == 0:
            print("  🏆 ZERO CODE WARNINGS ACHIEVED!")
        elif warning_count < 10:
            print(f"  📝 Remaining warnings: {warning_count} (good progress)")
            for warning in warning_lines[:5]:  # Show first 5
                print(f"      {warning}")
                
    else:
        print(f"  ❌ Compilation failed:")
        print(result.stderr[:1000])
    
    print(f"\n🎯 Sprint 92 Status: Backslash syntax errors fixed")

if __name__ == "__main__":
    main()