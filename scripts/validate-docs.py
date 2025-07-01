#!/usr/bin/env python3
"""
Documentation Validation Script for PAIML MCP Agent Toolkit

Validates documentation structure, checks for broken links, 
verifies document headers, and detects stale TODO items.
"""

import os
import re
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Tuple, Dict, Set

# Documentation root directory
DOCS_DIR = Path(__file__).parent.parent / "docs"

# Validation configuration
MAX_TODO_AGE_DAYS = 90
REQUIRED_HEADER_FIELDS = ["Status", "Type", "Created", "Updated"]
VALID_STATUSES = ["Draft", "Active", "Archived", "Deprecated", "TODO"]
VALID_TYPES = ["Guide", "Reference", "Decision", "Specification", "Index", "TODO"]

# File patterns to check
MARKDOWN_PATTERN = "*.md"

# Directories to exclude from validation
EXCLUDE_DIRS = {"archive", "templates"}

class DocumentationValidator:
    def __init__(self, docs_dir: Path):
        self.docs_dir = docs_dir
        self.errors: List[str] = []
        self.warnings: List[str] = []
        self.all_files: Set[Path] = set()
        self.linked_files: Set[Path] = set()
        
    def validate(self) -> bool:
        """Run all validation checks"""
        print(f"Validating documentation in: {self.docs_dir}")
        print("-" * 60)
        
        # Collect all markdown files
        self._collect_markdown_files()
        
        # Run validation checks
        self._check_document_headers()
        self._check_todo_staleness()
        self._check_internal_links()
        self._check_orphaned_documents()
        self._check_file_naming()
        
        # Report results
        self._report_results()
        
        return len(self.errors) == 0
    
    def _collect_markdown_files(self):
        """Collect all markdown files in docs directory"""
        for md_file in self.docs_dir.rglob(MARKDOWN_PATTERN):
            # Skip excluded directories
            if any(exclude in md_file.parts for exclude in EXCLUDE_DIRS):
                continue
            self.all_files.add(md_file)
    
    def _check_document_headers(self):
        """Validate document headers"""
        print("Checking document headers...")
        
        for file_path in self.all_files:
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                    
                # Extract header section (between title and first ----)
                header_match = re.search(r'^#[^#].*?\n(.*?)^---', content, re.MULTILINE | re.DOTALL)
                if not header_match:
                    self.warnings.append(f"{file_path.relative_to(self.docs_dir)}: Missing document header")
                    continue
                
                header = header_match.group(1)
                
                # Check required fields
                for field in REQUIRED_HEADER_FIELDS:
                    if f"**{field}**:" not in header:
                        self.warnings.append(f"{file_path.relative_to(self.docs_dir)}: Missing header field '{field}'")
                
                # Validate Status field
                status_match = re.search(r'\*\*Status\*\*:\s*(\w+)', header)
                if status_match and status_match.group(1) not in VALID_STATUSES:
                    self.errors.append(f"{file_path.relative_to(self.docs_dir)}: Invalid status '{status_match.group(1)}'")
                
                # Validate Type field
                type_match = re.search(r'\*\*Type\*\*:\s*(\w+)', header)
                if type_match and type_match.group(1) not in VALID_TYPES:
                    self.errors.append(f"{file_path.relative_to(self.docs_dir)}: Invalid type '{type_match.group(1)}'")
                    
            except Exception as e:
                self.errors.append(f"{file_path.relative_to(self.docs_dir)}: Error reading file: {e}")
    
    def _check_todo_staleness(self):
        """Check for stale TODO items"""
        print("Checking TODO staleness...")
        
        todo_dir = self.docs_dir / "todo"
        if not todo_dir.exists():
            return
        
        cutoff_date = datetime.now() - timedelta(days=MAX_TODO_AGE_DAYS)
        
        for todo_file in todo_dir.rglob(MARKDOWN_PATTERN):
            if "archive" in todo_file.parts:
                continue
                
            # Check file modification time
            mtime = datetime.fromtimestamp(todo_file.stat().st_mtime)
            if mtime < cutoff_date:
                age_days = (datetime.now() - mtime).days
                self.warnings.append(f"Stale TODO: {todo_file.relative_to(self.docs_dir)} ({age_days} days old)")
            
            # Check Created date in header
            try:
                with open(todo_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                    created_match = re.search(r'\*\*Created\*\*:\s*(\d{4}-\d{2}-\d{2})', content)
                    if created_match:
                        created_date = datetime.strptime(created_match.group(1), '%Y-%m-%d')
                        if created_date < cutoff_date:
                            age_days = (datetime.now() - created_date).days
                            self.warnings.append(f"Stale TODO by creation date: {todo_file.relative_to(self.docs_dir)} ({age_days} days old)")
            except Exception:
                pass
    
    def _check_internal_links(self):
        """Check for broken internal links"""
        print("Checking internal links...")
        
        for file_path in self.all_files:
            try:
                with open(file_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # Find all markdown links
                links = re.findall(r'\[([^\]]+)\]\(([^)]+)\)', content)
                
                for link_text, link_target in links:
                    # Skip external links
                    if link_target.startswith(('http://', 'https://', 'mailto:', '#')):
                        continue
                    
                    # Resolve relative link
                    if link_target.endswith('.md'):
                        target_path = (file_path.parent / link_target).resolve()
                        
                        # Track linked files
                        if target_path.is_relative_to(self.docs_dir):
                            self.linked_files.add(target_path)
                        
                        # Check if target exists
                        if not target_path.exists():
                            self.errors.append(f"{file_path.relative_to(self.docs_dir)}: Broken link to '{link_target}'")
                            
            except Exception as e:
                self.errors.append(f"{file_path.relative_to(self.docs_dir)}: Error checking links: {e}")
    
    def _check_orphaned_documents(self):
        """Check for documents not linked from anywhere"""
        print("Checking for orphaned documents...")
        
        # Add README.md files to linked set (entry points)
        for readme in self.docs_dir.rglob("README.md"):
            self.linked_files.add(readme)
        
        # Find orphaned files
        orphaned = self.all_files - self.linked_files
        
        for orphan in orphaned:
            # Skip files in archive
            if "archive" not in orphan.parts:
                self.warnings.append(f"Orphaned document: {orphan.relative_to(self.docs_dir)}")
    
    def _check_file_naming(self):
        """Check file naming conventions"""
        print("Checking file naming conventions...")
        
        for file_path in self.all_files:
            filename = file_path.stem
            
            # Check for lowercase with hyphens
            if not re.match(r'^[a-z0-9-]+$', filename):
                # Allow UPPERCASE files like README, CLAUDE, etc.
                if not re.match(r'^[A-Z_]+$', filename):
                    self.warnings.append(f"Non-standard filename: {file_path.name}")
            
            # Check for special characters
            if any(char in filename for char in ['_', '.', ' ', '@', '!', '#', '$', '%', '^', '&', '*']):
                # Allow underscore in all-caps files
                if not (re.match(r'^[A-Z_]+$', filename) and '_' in filename):
                    self.warnings.append(f"Special characters in filename: {file_path.name}")
    
    def _report_results(self):
        """Report validation results"""
        print("\n" + "=" * 60)
        print("VALIDATION RESULTS")
        print("=" * 60)
        
        print(f"\nTotal files checked: {len(self.all_files)}")
        print(f"Errors found: {len(self.errors)}")
        print(f"Warnings found: {len(self.warnings)}")
        
        if self.errors:
            print("\nERRORS:")
            for error in sorted(self.errors):
                print(f"  ❌ {error}")
        
        if self.warnings:
            print("\nWARNINGS:")
            for warning in sorted(self.warnings):
                print(f"  ⚠️  {warning}")
        
        if not self.errors and not self.warnings:
            print("\n✅ All documentation validation checks passed!")
        
        print("\nSUMMARY:")
        print(f"  - Document coverage: {len(self.linked_files)}/{len(self.all_files)} files linked")
        print(f"  - TODO items checked: {len([f for f in self.all_files if 'todo' in f.parts])}")
        print(f"  - Validation status: {'PASSED' if not self.errors else 'FAILED'}")

def main():
    """Main entry point"""
    validator = DocumentationValidator(DOCS_DIR)
    success = validator.validate()
    
    # Exit with appropriate code
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main()