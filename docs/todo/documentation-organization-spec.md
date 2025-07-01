# Documentation Organization Specification

## Overview
This specification defines the strict organization rules for documentation in the paiml-mcp-agent-toolkit project, following Zero Tolerance Quality Standards.

## Directory Structure

```
docs/
├── api/                    # API documentation
├── architecture/           # Architecture documents and diagrams
├── archive/               # Historical documents organized by date
│   └── YYYY/             # Year
│       └── MM/           # Month
│           └── category/ # Category subdirectory
├── bugs/                  # Bug tracking
│   ├── active/           # Active bugs being worked on
│   └── archived/         # Fixed and validated bugs
├── features/             # Feature documentation
├── guides/               # User and developer guides
├── integrations/         # Integration documentation
├── policies/             # Project policies and standards
├── templates/            # Document templates
└── todo/                 # Specifications and tasks
    ├── active/           # Active specs being implemented
    └── archive/          # Implemented and validated specs
```

## Rules

### 1. **Root Directory Rules**
- Only `README.md`, `RELEASE_NOTES.md`, and `CLAUDE.md` allowed in project root
- All other markdown files must be in appropriate subdirectories
- No temporary or work-in-progress files in root

### 2. **Docs Root Rules**
- No loose markdown files in `docs/` root
- All documentation must be in appropriate subdirectories
- Only directories allowed in docs root, no files

### 3. **Bug Tracking**
- New bugs go in `docs/bugs/active/`
- When fixed and validated, move to `docs/bugs/archived/` (note: "archived" not "archive")
- Include resolution notes and commit references
- No files directly in `docs/bugs/`

### 4. **Specification Management**
- New specs go in `docs/todo/active/`
- When implemented and validated, move to `docs/todo/archive/`
- Include implementation notes and test references
- No files directly in `docs/todo/`

### 5. **Archive Organization**
- Use `docs/archive/YYYY/MM/category/` structure
- Categories: kaizen, system-status, refactor, etc.
- No duplicate content between archive and active directories

### 6. **Validation Requirements**
- Bugs: Fixed in code, tests pass, documented resolution
- Specs: Fully implemented, tests pass, matches requirements
- Features: Current and accurate documentation
- No outdated or duplicate content

### 7. **Maintenance Commands**
```bash
# Regenerate Rust documentation
make docs

# Validate root directory structure
find . -maxdepth 1 -name "*.md" | grep -v -E "(README|RELEASE_NOTES|CLAUDE)\.md"
# Should return no results

# Validate docs root structure
find docs -maxdepth 1 -name "*.md"
# Should return no results

# Find misplaced bug files
find docs/bugs -maxdepth 1 -name "*.md"
# Should return no results

# Find misplaced todo files
find docs/todo -maxdepth 1 -name "*.md"
# Should return no results

# Check for duplicate directories
find docs -type d -name "kaizen" | wc -l
# Should return 1

# Use refactor docs to clean up
pmat refactor docs --dry-run
```