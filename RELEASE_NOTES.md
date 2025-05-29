# Release Notes for v0.4.7

## Features
- Added comprehensive MCP documentation synchronization tests
  - CLI documentation verification tests ensure docs match implementation
  - MCP tools documentation tests verify all tools are documented
  - Documentation examples validation tests check code snippets
- Added "Available MCP Tools" section to documentation with all 10 tools

## Improvements
- Fixed test output paths for Mermaid diagrams (now in artifacts/mermaid/test_output/)
- Updated .gitignore to exclude generated .mmd files at root
- Improved code organization with proper imports ordering

## Bug Fixes
- Fixed all Rust clippy warnings:
  - Moved regex compilation outside loops
  - Removed needless borrows
  - Replaced vec! with array literals where appropriate
  - Used idiomatic range contains checks
- Fixed E2E test to expect correct number of tools (10 instead of 8)

## Documentation
- Documented all 10 MCP tools with examples:
  - generate_template
  - analyze_complexity
  - analyze_code_churn
  - analyze_dag
  - generate_context
  - get_server_info
  - list_templates
  - scaffold_project
  - search_templates
  - validate_template

## Development
- Integrated documentation sync tests into main build process via Makefile
- Tests run automatically with `make test` to catch documentation drift