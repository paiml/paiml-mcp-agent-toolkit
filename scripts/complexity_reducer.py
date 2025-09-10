#!/usr/bin/env python3

"""
Automated Complexity Reduction Script
Sprint 88 Follow-up: Systematically reduce cyclomatic and cognitive complexity

This script identifies high-complexity functions and provides automated
refactoring suggestions using the Extract Method pattern (Toyota Way).
"""

import os
import re
import sys
import json
import subprocess
import argparse
from pathlib import Path
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass
from collections import defaultdict

@dataclass
class ComplexityViolation:
    """Represents a function with high complexity"""
    file_path: Path
    function_name: str
    line_number: int
    cyclomatic: int
    cognitive: int
    suggestions: List[str]
    priority: int  # 1-5, with 1 being highest

class ComplexityReducer:
    """Automated complexity reduction using Extract Method pattern"""
    
    def __init__(self, base_path: str = "server"):
        self.base_path = Path(base_path)
        self.cyclomatic_threshold = 20
        self.cognitive_threshold = 15
        self.target_complexity = 10  # A+ standard
        
    def find_complex_functions(self) -> List[ComplexityViolation]:
        """Use pmat to find complex functions"""
        try:
            # Run pmat analyze complexity
            result = subprocess.run(
                ["./target/debug/pmat", "analyze", "complexity", 
                 "--project-path", str(self.base_path),
                 "--format", "json",
                 "--max-cyclomatic", str(self.cyclomatic_threshold),
                 "--max-cognitive", str(self.cognitive_threshold)],
                capture_output=True,
                text=True,
                cwd="."
            )
            
            if result.returncode != 0:
                print(f"Warning: pmat analyze failed: {result.stderr}")
                return []
            
            # Parse JSON output
            try:
                data = json.loads(result.stdout)
                return self.parse_complexity_data(data)
            except json.JSONDecodeError:
                # Try line-by-line parsing for streaming JSON
                violations = []
                for line in result.stdout.split('\n'):
                    if line.strip() and line.startswith('{'):
                        try:
                            item = json.loads(line)
                            if 'cyclomatic' in item:
                                violations.append(self.create_violation(item))
                        except:
                            continue
                return violations
                
        except Exception as e:
            print(f"Error running complexity analysis: {e}")
            return []
    
    def parse_complexity_data(self, data: Dict) -> List[ComplexityViolation]:
        """Parse pmat complexity analysis output"""
        violations = []
        
        # Handle different possible output formats
        if isinstance(data, dict):
            if 'violations' in data:
                items = data['violations']
            elif 'functions' in data:
                items = data['functions']
            elif 'results' in data:
                items = data['results']
            else:
                items = [data]
        elif isinstance(data, list):
            items = data
        else:
            return []
        
        for item in items:
            violation = self.create_violation(item)
            if violation:
                violations.append(violation)
        
        # Sort by priority (highest complexity first)
        violations.sort(key=lambda v: (v.priority, -(v.cyclomatic + v.cognitive)))
        return violations
    
    def create_violation(self, item: Dict) -> Optional[ComplexityViolation]:
        """Create a violation from parsed data"""
        try:
            cyclomatic = item.get('cyclomatic', 0)
            cognitive = item.get('cognitive', 0)
            
            # Skip if below thresholds
            if cyclomatic <= self.cyclomatic_threshold and cognitive <= self.cognitive_threshold:
                return None
            
            # Determine priority based on severity
            if cyclomatic > 30 or cognitive > 25:
                priority = 1  # Critical
            elif cyclomatic > 25 or cognitive > 20:
                priority = 2  # High
            elif cyclomatic > 20 or cognitive > 15:
                priority = 3  # Medium
            else:
                priority = 4  # Low
            
            return ComplexityViolation(
                file_path=Path(item.get('file', 'unknown')),
                function_name=item.get('function', item.get('name', 'unknown')),
                line_number=item.get('line', 0),
                cyclomatic=cyclomatic,
                cognitive=cognitive,
                suggestions=self.generate_suggestions(cyclomatic, cognitive),
                priority=priority
            )
        except Exception as e:
            print(f"Error creating violation: {e}")
            return None
    
    def generate_suggestions(self, cyclomatic: int, cognitive: int) -> List[str]:
        """Generate refactoring suggestions based on complexity"""
        suggestions = []
        
        if cyclomatic > 25:
            suggestions.append("CRITICAL: Extract Method - Split into 3+ smaller functions")
            suggestions.append("Consider using Strategy pattern for complex conditionals")
            suggestions.append("Replace nested if-else with early returns")
        elif cyclomatic > 20:
            suggestions.append("HIGH: Extract Method - Split into 2-3 smaller functions")
            suggestions.append("Simplify conditional logic with guard clauses")
            suggestions.append("Consider extracting validation logic")
        elif cyclomatic > 15:
            suggestions.append("MEDIUM: Extract helper functions for repeated logic")
            suggestions.append("Use match/switch instead of if-else chains")
        
        if cognitive > 20:
            suggestions.append("Reduce nesting levels (max 3 recommended)")
            suggestions.append("Extract complex expressions to named variables")
            suggestions.append("Split compound conditions into separate functions")
        elif cognitive > 15:
            suggestions.append("Simplify boolean expressions")
            suggestions.append("Extract inline logic to helper functions")
        
        # Add Toyota Way principle
        suggestions.append("Toyota Way: Apply Kaizen - incremental improvement")
        
        return suggestions
    
    def generate_extract_method_template(self, violation: ComplexityViolation) -> str:
        """Generate Extract Method refactoring template"""
        template = f"""
// REFACTORING TEMPLATE for {violation.function_name}
// Current Complexity: Cyclomatic={violation.cyclomatic}, Cognitive={violation.cognitive}
// Target: ≤10 for both metrics

// Step 1: Identify logical blocks in the function
// Step 2: Extract each block into a separate function
// Step 3: Apply single responsibility principle

// Example refactoring pattern:
impl {violation.function_name.split('::')[0] if '::' in violation.function_name else 'Module'} {{
    
    // Original complex function (simplified)
    pub fn {violation.function_name.split('::')[-1] if '::' in violation.function_name else violation.function_name}(&self) -> Result<()> {{
        self.validate_input()?;
        let processed = self.process_data()?;
        self.generate_output(processed)
    }}
    
    // Extracted methods (complexity ≤10 each)
    fn validate_input(&self) -> Result<()> {{
        // Validation logic here
        Ok(())
    }}
    
    fn process_data(&self) -> Result<ProcessedData> {{
        // Processing logic here
        Ok(ProcessedData::default())
    }}
    
    fn generate_output(&self, data: ProcessedData) -> Result<()> {{
        // Output generation here
        Ok(())
    }}
}}
"""
        return template
    
    def apply_auto_refactor(self, violation: ComplexityViolation, dry_run: bool = True) -> bool:
        """Apply automated refactoring using pmat refactor"""
        if dry_run:
            print(f"[DRY RUN] Would refactor: {violation.file_path}::{violation.function_name}")
            return True
        
        try:
            # Use pmat refactor auto
            result = subprocess.run(
                ["./target/debug/pmat", "refactor", "auto",
                 "--file", str(violation.file_path),
                 "--function", violation.function_name],
                capture_output=True,
                text=True,
                cwd="."
            )
            
            if result.returncode == 0:
                print(f"✅ Refactored: {violation.function_name}")
                return True
            else:
                print(f"❌ Failed to refactor: {violation.function_name}")
                print(f"   Error: {result.stderr}")
                return False
                
        except Exception as e:
            print(f"Error applying refactoring: {e}")
            return False
    
    def generate_report(self, violations: List[ComplexityViolation], format: str = "terminal"):
        """Generate complexity reduction report"""
        if format == "json":
            self.generate_json_report(violations)
        elif format == "markdown":
            self.generate_markdown_report(violations)
        else:
            self.generate_terminal_report(violations)
    
    def generate_terminal_report(self, violations: List[ComplexityViolation]):
        """Generate terminal-formatted report"""
        print("\n" + "="*70)
        print("COMPLEXITY REDUCTION REPORT")
        print("="*70)
        print(f"\nFound {len(violations)} high-complexity functions\n")
        
        if not violations:
            print("✅ No complexity violations found!")
            return
        
        # Group by priority
        by_priority = defaultdict(list)
        for v in violations:
            by_priority[v.priority].append(v)
        
        # Summary
        print("SUMMARY BY PRIORITY:")
        print("-" * 40)
        for priority in sorted(by_priority.keys()):
            count = len(by_priority[priority])
            label = ["", "CRITICAL", "HIGH", "MEDIUM", "LOW", "INFO"][priority]
            print(f"  {label:8} ({priority}): {count} functions")
        
        print(f"\nTOP 10 COMPLEX FUNCTIONS:")
        print("-" * 70)
        print(f"{'Function':<30} {'File':<20} {'Cyclo':<8} {'Cogn':<8} {'Priority'}")
        print("-" * 70)
        
        for v in violations[:10]:
            file_name = v.file_path.name if v.file_path != Path('unknown') else 'unknown'
            func_name = v.function_name[:29] if len(v.function_name) > 29 else v.function_name
            print(f"{func_name:<30} {file_name:<20} {v.cyclomatic:<8} {v.cognitive:<8} P{v.priority}")
        
        if violations:
            print(f"\nREFACTORING SUGGESTIONS FOR TOP VIOLATION:")
            print("-" * 70)
            top = violations[0]
            print(f"Function: {top.function_name}")
            print(f"Location: {top.file_path}:{top.line_number}")
            print(f"Complexity: Cyclomatic={top.cyclomatic}, Cognitive={top.cognitive}")
            print("\nSuggestions:")
            for i, suggestion in enumerate(top.suggestions, 1):
                print(f"  {i}. {suggestion}")
    
    def generate_markdown_report(self, violations: List[ComplexityViolation]):
        """Generate markdown-formatted report"""
        print("# Complexity Reduction Report\n")
        print(f"**Total Violations**: {len(violations)}\n")
        print(f"**Date**: {__import__('datetime').datetime.now().isoformat()}\n")
        
        if not violations:
            print("✅ **No complexity violations found!**")
            return
        
        # Summary table
        print("## Summary by Priority\n")
        print("| Priority | Level | Count | Action Required |")
        print("|----------|-------|-------|-----------------|")
        
        by_priority = defaultdict(list)
        for v in violations:
            by_priority[v.priority].append(v)
        
        for priority in sorted(by_priority.keys()):
            count = len(by_priority[priority])
            label = ["", "CRITICAL", "HIGH", "MEDIUM", "LOW", "INFO"][priority]
            action = ["", "Immediate", "This Sprint", "Next Sprint", "When Possible", "Monitor"][priority]
            print(f"| P{priority} | {label} | {count} | {action} |")
        
        # Top violations
        print("\n## Top 10 Complex Functions\n")
        print("| Function | File | Cyclomatic | Cognitive | Priority |")
        print("|----------|------|------------|-----------|----------|")
        
        for v in violations[:10]:
            file_name = v.file_path.name if v.file_path != Path('unknown') else 'unknown'
            print(f"| `{v.function_name}` | {file_name} | {v.cyclomatic} | {v.cognitive} | P{v.priority} |")
        
        # Detailed refactoring plan
        print("\n## Refactoring Plan\n")
        
        for i, v in enumerate(violations[:5], 1):
            print(f"### {i}. {v.function_name}\n")
            print(f"- **File**: `{v.file_path}`")
            print(f"- **Line**: {v.line_number}")
            print(f"- **Complexity**: Cyclomatic={v.cyclomatic}, Cognitive={v.cognitive}")
            print(f"- **Priority**: P{v.priority}")
            print("\n**Suggestions**:")
            for suggestion in v.suggestions:
                print(f"- {suggestion}")
            print()
    
    def generate_json_report(self, violations: List[ComplexityViolation]):
        """Generate JSON-formatted report"""
        report = {
            "timestamp": __import__('datetime').datetime.now().isoformat(),
            "total_violations": len(violations),
            "thresholds": {
                "cyclomatic": self.cyclomatic_threshold,
                "cognitive": self.cognitive_threshold,
                "target": self.target_complexity
            },
            "violations": []
        }
        
        for v in violations:
            report["violations"].append({
                "file": str(v.file_path),
                "function": v.function_name,
                "line": v.line_number,
                "cyclomatic": v.cyclomatic,
                "cognitive": v.cognitive,
                "priority": v.priority,
                "suggestions": v.suggestions
            })
        
        print(json.dumps(report, indent=2))

def main():
    parser = argparse.ArgumentParser(
        description="Automated complexity reduction using Extract Method pattern"
    )
    parser.add_argument(
        "--path",
        default="server",
        help="Base path for analysis"
    )
    parser.add_argument(
        "--format",
        choices=["terminal", "markdown", "json"],
        default="terminal",
        help="Output format for report"
    )
    parser.add_argument(
        "--max-cyclomatic",
        type=int,
        default=20,
        help="Maximum allowed cyclomatic complexity"
    )
    parser.add_argument(
        "--max-cognitive",
        type=int,
        default=15,
        help="Maximum allowed cognitive complexity"
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Apply automated refactoring (use with caution)"
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=0,
        help="Limit number of functions to process"
    )
    parser.add_argument(
        "--template",
        action="store_true",
        help="Generate refactoring templates"
    )
    
    args = parser.parse_args()
    
    reducer = ComplexityReducer(args.path)
    reducer.cyclomatic_threshold = args.max_cyclomatic
    reducer.cognitive_threshold = args.max_cognitive
    
    # Find complex functions
    violations = reducer.find_complex_functions()
    
    if not violations:
        print("No complexity violations found! ✅")
        return
    
    # Apply limit if specified
    if args.limit > 0:
        violations = violations[:args.limit]
    
    # Generate report
    reducer.generate_report(violations, args.format)
    
    # Generate templates if requested
    if args.template and violations:
        print("\n" + "="*70)
        print("REFACTORING TEMPLATE")
        print("="*70)
        template = reducer.generate_extract_method_template(violations[0])
        print(template)
    
    # Apply refactoring if requested
    if args.apply:
        print("\n" + "="*70)
        print("APPLYING AUTOMATED REFACTORING")
        print("="*70)
        
        success_count = 0
        for v in violations:
            if reducer.apply_auto_refactor(v, dry_run=False):
                success_count += 1
        
        print(f"\n✅ Successfully refactored {success_count}/{len(violations)} functions")

if __name__ == "__main__":
    main()