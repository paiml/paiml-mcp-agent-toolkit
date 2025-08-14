#!/usr/bin/env python3
"""Script to remove dead code functions from deep_context.rs"""

import re
import sys

def find_function_end(lines, start_idx):
    """Find the end of a function by tracking brace balance"""
    brace_count = 0
    in_function = False
    
    for i in range(start_idx, len(lines)):
        line = lines[i]
        # Count braces
        for char in line:
            if char == '{':
                brace_count += 1
                in_function = True
            elif char == '}':
                brace_count -= 1
        
        # Function ends when we return to brace_count 0
        if in_function and brace_count == 0:
            return i
    
    return len(lines) - 1

def remove_dead_code(filename):
    """Remove functions marked with #[allow(dead_code)]"""
    with open(filename, 'r') as f:
        lines = f.readlines()
    
    # Find all dead code markers
    dead_code_indices = []
    for i, line in enumerate(lines):
        if '#[allow(dead_code)]' in line:
            dead_code_indices.append(i)
    
    # Process from end to beginning to maintain indices
    dead_code_indices.reverse()
    
    removed_functions = []
    
    for idx in dead_code_indices:
        # Find function start (usually 1-2 lines after the attribute)
        func_start = idx
        for j in range(idx, min(idx + 5, len(lines))):
            if 'fn ' in lines[j]:
                # Extract function name
                match = re.search(r'fn\s+(\w+)', lines[j])
                if match:
                    func_name = match.group(1)
                    removed_functions.append(func_name)
                func_start = j
                break
        
        # Find function end
        func_end = find_function_end(lines, func_start)
        
        # Remove the function (including any preceding comments/attributes)
        # Look back for comments
        start_remove = idx
        for j in range(idx - 1, max(0, idx - 10), -1):
            if lines[j].strip().startswith('///') or lines[j].strip().startswith('//'):
                start_remove = j
            elif lines[j].strip() == '':
                continue
            else:
                break
        
        # Remove the lines
        del lines[start_remove:func_end + 1]
        
        # Also remove extra blank lines
        while start_remove < len(lines) and lines[start_remove].strip() == '':
            del lines[start_remove]
    
    # Write back
    with open(filename, 'w') as f:
        f.writelines(lines)
    
    return removed_functions

if __name__ == '__main__':
    filename = 'src/services/deep_context.rs'
    removed = remove_dead_code(filename)
    print(f"Removed {len(removed)} dead code functions:")
    for func in removed:
        print(f"  - {func}")