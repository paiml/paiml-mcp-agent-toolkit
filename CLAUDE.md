# CLAUDE.md

## System Architecture Overview

This document serves as the operational guide for the paiml-mcp-agent-toolkit (pmat), a unified protocol implementation supporting CLI, MCP, and HTTP interfaces through a single binary architecture.

**Core Design Principle**: Protocol-agnostic service layer with deterministic behavior across all interfaces.
- Jidoka (自働化): Build quality in through proper error handling and verification (Never use TODO or leave unfinished code)
- Genchi Genbutsu (現地現物): Go and see the actual root causes instead of statistical approximations
- Hansei (反省): Focus on fixing existing broken functionality rather than adding new features
- Kaizen - Continuous Improvement

## Operational Guidelines

### ABSOLUTE RULES
- NEVER work out of the server directory

[Rest of the existing content remains unchanged...]