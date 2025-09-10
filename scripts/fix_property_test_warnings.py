#!/usr/bin/env python3
"""
Sprint 92 Property Test Warning Fixes
Systematically remove unused imports and fix unused variables in property test modules
"""

import os
import re
import subprocess
from pathlib import Path

def find_property_test_files():
    """Find all Rust files with property test modules"""
    server_dir = Path("server/src")
    files_with_property_tests = []
    
    for rust_file in server_dir.rglob("*.rs"):
        try:
            with open(rust_file, 'r', encoding='utf-8') as f:
                content = f.read()
                if "#[cfg(test)]\nmod property_tests {" in content:
                    files_with_property_tests.append(rust_file)
        except Exception as e:
            print(f"Error reading {rust_file}: {e}")
            continue
    
    return files_with_property_tests

def fix_unused_imports_in_file(file_path):
    """Remove unused super::* imports from property test modules"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Pattern to match property test modules with unused imports
        pattern = r'(#\[cfg\(test\)\]\nmod property_tests \{\n)    use super::\*;\n(    use proptest::prelude::\*;)'
        
        # Replace with version without unused import
        fixed_content = re.sub(pattern, r'\1\2', content)
        
        # Also fix standalone unused super::* imports
        pattern2 = r'    use super::\*;\n    use proptest::prelude::\*;'
        fixed_content = re.sub(pattern2, '    use proptest::prelude::*;', fixed_content)
        
        if fixed_content != content:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(fixed_content)
            return True
        return False
        
    except Exception as e:
        print(f"Error fixing imports in {file_path}: {e}")
        return False

def fix_unused_variables_in_file(file_path):
    """Fix unused variable warnings by prefixing with underscore"""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Fix unused input variables in property tests
        patterns = [
            (r'fn basic_property_stability\(input in', r'fn basic_property_stability\(_input in'),
            (r'fn module_consistency_check\(x in', r'fn module_consistency_check\(_x in'),
        ]
        
        fixed_content = content
        changed = False
        
        for old_pattern, new_pattern in patterns:
            if re.search(old_pattern, fixed_content):
                fixed_content = re.sub(old_pattern, new_pattern, fixed_content)
                changed = True
        
        if changed:
            with open(file_path, 'w', encoding='utf-8') as f:
                f.write(fixed_content)
            return True
        return False
        
    except Exception as e:
        print(f"Error fixing variables in {file_path}: {e}")
        return False

def main():
    print("🔧 Sprint 92: Fixing Property Test Warnings")
    print("=" * 50)
    
    # Change to project root
    os.chdir("/home/noah/src/paiml-mcp-agent-toolkit")
    
    # Find all property test files
    property_test_files = find_property_test_files()
    print(f"Found {len(property_test_files)} files with property tests")
    
    # Fix unused imports
    import_fixes = 0
    print("\n🗑️  Removing unused imports...")
    for file_path in property_test_files:
        if fix_unused_imports_in_file(file_path):
            import_fixes += 1
            print(f"  ✅ Fixed imports: {file_path}")
    
    # Fix unused variables
    variable_fixes = 0
    print(f"\n🔧 Fixing unused variables...")
    for file_path in property_test_files:
        if fix_unused_variables_in_file(file_path):
            variable_fixes += 1
            print(f"  ✅ Fixed variables: {file_path}")
    
    print(f"\n📊 Sprint 92 Progress:")
    print(f"  • Import fixes: {import_fixes} files")
    print(f"  • Variable fixes: {variable_fixes} files")
    
    # Test compilation
    print(f"\n🧪 Testing compilation...")
    result = subprocess.run(["cargo", "check", "--package", "pmat", "--quiet"], 
                          capture_output=True, text=True)
    
    if result.returncode == 0:
        print("  ✅ Compilation successful!")
        
        # Count remaining warnings
        warn_result = subprocess.run(["cargo", "check", "--package", "pmat"], 
                                   capture_output=True, text=True)
        warning_count = warn_result.stderr.count("warning:")
        print(f"  📊 Remaining warnings: {warning_count}")
        
        if warning_count == 0:
            print("  🏆 ZERO WARNINGS ACHIEVED!")
        
    else:
        print(f"  ❌ Compilation failed:")
        print(result.stderr[:1000])
    
    print(f"\n🎯 Sprint 92 Status: Warning reduction in progress")

if __name__ == "__main__":
    main()