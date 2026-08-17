# Antigravity Global Rules for PMAT

Welcome to the `paiml-mcp-agent-toolkit` (PMAT) codebase! PMAT is a zero-config AI context generation and code quality toolkit.

## Agent Guidelines

- Use the PMAT CLI via the configured MCP Server for quality checks and analysis.
- Follow the rules defined in `.agents/rules/` for all operations.
- Maintain test coverage above 95% (as detailed in the rules).
- Adhere to the autonomous verify loop protocol for self-correction.

When making code changes, ensure that PMAT quality gates (via the `pmat-quality-feedback.sh` hook) pass. If it fails, fix the code according to the feedback!
