# Security Audit Report

**Date**: 2025-10-21
**Tool**: `cargo audit`
**Status**: 9 warnings (unmaintained dependencies)
**Critical Vulnerabilities**: 0

## Summary

All detected issues are **warnings** about unmaintained crates, not active security vulnerabilities. The project is safe for production use with awareness of the technical debt.

## Unmaintained Dependencies

### 1. rustpython-parser 0.4.0 (6 related crates)

**Affected crates:**
- `unic-char-property` 0.9.0 (RUSTSEC-2025-0081)
- `unic-char-range` 0.9.0 (RUSTSEC-2025-0075)
- `unic-common` 0.9.0 (RUSTSEC-2025-0080)
- `unic-emoji-char` 0.9.0 (RUSTSEC-2025-0090)
- `unic-ucd-ident` 0.9.0 (RUSTSEC-2025-0100)
- `unic-ucd-version` 0.9.0 (RUSTSEC-2025-0098)

**Status**: Optional dependency (only with `python-ast` feature)
**Mitigation**:
- Consider alternatives: `enderpy_python_parser` or `python-parser`
- Or rely solely on `tree-sitter-python` for Python AST parsing
- Tracked in: `server/Cargo.toml:148`

### 2. sled 0.34.7

**Affected crates:**
- `fxhash` 0.2.1 (RUSTSEC-2025-0057)
- `instant` 0.1.13 (RUSTSEC-2024-0384)

**Status**: ✅ MIGRATION COMPLETE (2025-10-21)
**Mitigation**:
- Migrated to libsql-compatible backend (uses rusqlite for sync API)
- New `LibsqlBackend` implements full StorageBackend trait
- Default backend changed from Sled to Libsql in StorageConfig
- Sled backend deprecated but remains available for compatibility
- Commit: 5b4ac0b4 "Complete sled → libsql migration for TDG storage backend"

### 3. paste 1.0.15

**Affected**: RUSTSEC-2024-0436
**Status**: Widely used macro crate, already at latest version
**Mitigation**:
- Monitor for maintained fork or replacement
- Low risk: macro-only crate with no runtime security impact

### 4. fxhash 0.2.1 (via wasmtime/ruchy)

**Affected**: RUSTSEC-2025-0057
**Status**: Transitive dependency through `wasmtime` → `ruchy`
**Mitigation**:
- Monitor `wasmtime` updates for alternative hash implementations
- Low risk: hashing algorithm, no known exploits

## Risk Assessment

| Severity | Count | Impact |
|----------|-------|--------|
| Critical | 0     | None   |
| High     | 0     | None   |
| Medium   | 0     | None   |
| Low      | 9     | Maintenance burden |

## Action Items

### Immediate (Low Priority)
- [x] Document security audit findings
- [ ] Add `cargo audit` to CI/CD pipeline
- [ ] Set up Dependabot alerts monitoring

### Short-Term (Next Sprint)
- [ ] Evaluate `enderpy_python_parser` as rustpython-parser replacement
- [x] Complete sled → libsql migration (storage_backend.rs) - ✅ DONE (2025-10-21, commit 5b4ac0b4)
- [ ] Monitor paste crate for maintained alternatives

### Long-Term
- [ ] Establish quarterly dependency audit schedule
- [ ] Create dependency update policy
- [ ] Evaluate moving to tree-sitter-only for all language parsing

## GitHub Dependabot Alerts

### Open Alerts (3 total)

**npm vulnerabilities (2 - medium severity):**
1. **vite** (>= 5.2.6, <= 5.4.20) → Fix: Upgrade to 7.1.11
   - Issue: server.fs.deny bypass via backslash on Windows
   - Status: Transitive dependency, investigating source

2. **esbuild** (<= 0.24.2) → Fix: Upgrade to 0.25.0
   - Issue: Any website can send requests to dev server
   - Status: Transitive dependency, investigating source

**Rust vulnerabilities (1 - low severity):**
3. **libsql-sqlite3-parser** (<= 0.13.0) → Fix: No patch available
   - Issue: Crash due to invalid UTF-8 input
   - Status: Transitive via libsql 0.9.24, upgraded to latest
   - Impact: LOW - denial of service only, no data breach risk

## Recent Updates (2025-10-21)

✅ **libsql upgraded**: 0.9 → 0.9.24 (latest version)
- libsql-sqlite3-parser remains at 0.13.0 (no patch available yet)
- Monitoring upstream for security fix

⚠️ **npm vulnerabilities**: vite and esbuild remain open
- Both are transitive dependencies (not direct)
- Medium severity, dev-only impact
- Action: Audit npm dependency tree in next sprint

## Conclusion

✅ **Project is production-ready** - No critical security vulnerabilities
⚠️ **Technical debt identified** - 9 unmaintained dependencies + 3 Dependabot alerts
📋 **Migration path exists** - Clear plan for major unmaintained dependencies
🔄 **Active maintenance** - libsql updated to latest, npm audit scheduled

**Recommendation**: Deploy with confidence. Monitor Dependabot for patches. Schedule npm audit in next sprint.

---

**Audit By**: Claude Code
**Last Update**: 2025-10-21
**Next Audit**: 2026-01-21 (90 days)
