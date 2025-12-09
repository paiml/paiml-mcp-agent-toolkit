# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 2.210.x | :white_check_mark: |
| 2.200.x | :white_check_mark: |
| < 2.200 | :x:                |

## Reporting a Vulnerability

**DO NOT** open a public issue for security vulnerabilities.

### Responsible Disclosure

1. Email security concerns to: security@paiml.com
2. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact assessment
   - Any suggested fixes (optional)

### Response Timeline

| Stage | Timeline |
|-------|----------|
| Initial Response | 48 hours |
| Triage | 7 days |
| Fix Development | 30 days |
| Public Disclosure | 90 days (coordinated) |

## Security Measures

### Code Analysis

- **cargo-audit**: Run weekly, blocking on HIGH severity
- **cargo-deny**: License and dependency policy enforcement
- **Clippy**: All warnings treated as errors in CI

### Supply Chain Security

- Dependencies pinned in `Cargo.lock`
- Minimal dependency policy (< 300 direct dependencies)
- No unmaintained dependencies (> 2 years without update)

### Input Validation

- All file paths validated against directory traversal
- User input sanitized before shell execution
- Maximum file size limits enforced (100MB default)

## Security Audit History

| Date | Auditor | Scope | Findings |
|------|---------|-------|----------|
| 2025-01 | Internal | Full codebase | 0 critical, 2 medium (fixed) |

## Falsifiable Security Claims

Per Popper's demarcation criterion, our security claims are testable:

| Claim | Test Method | CI Verification |
|-------|-------------|-----------------|
| No command injection | Fuzz testing with special chars | `cargo fuzz run cli_args` |
| No path traversal | Unit tests with `../` patterns | `cargo test path_validation` |
| Audit clean | `cargo audit` exit code 0 | GitHub Actions weekly |
| Memory safe | No `unsafe` without justification | `cargo clippy -D unsafe_code` |
