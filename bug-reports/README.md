# PMAT Bug Reports - Index

**Generated**: 2025-10-31
**Source**: User feedback from production testing
**Total Bugs**: 12

---

## Critical Severity (2 bugs)

### [#004: Dead Code Analysis Requires Cargo.toml for Non-Rust Projects](./004-dead-code-requires-cargo-toml.md)
**Component**: Dead code analyzer
**Impact**: Dead code analysis completely broken for non-Rust projects

The dead code analyzer assumes all projects are Rust and requires `Cargo.toml`, causing errors on C, C++, Python, and other language projects.

**Example**:
```bash
pmat analyze dead-code --path ./cpython
# Error: Cargo check failed: could not find `Cargo.toml`
```

### [#011: Incorrect Language Detection (python-uv instead of C++)](./011-wrong-language-detection.md)
**Component**: Language detection
**Impact**: Cannot analyze large C++ projects, process hangs indefinitely

Language detector incorrectly identifies Ceph (major C++ project) as "python-uv" with 57.2% confidence, then hangs on "Discovering project structure..."

**Example**:
```bash
cd ceph && pmat context
# Detected: python-uv (confidence: 57.2%)
# [hangs forever]
```

---

## High Severity (0 bugs)

---

## Medium Severity (7 bugs)

### [#001: `pmat embed status` Shows Wrong Error Message](./001-embed-status-wrong-error.md)
**Component**: CLI - embed subcommand

Shows error about invalid `--format` value 'summary' with generic PMAT examples instead of embedding-specific help.

### [#002: `pmat embed sync` Shows Wrong Error Message](./002-embed-sync-wrong-error.md)
**Component**: CLI - embed subcommand

Same error as #001, prevents syncing embeddings for semantic search.

### [#005: Broken Progress Output in `pmat context`](./005-broken-progress-output.md)
**Component**: CLI - context command progress display

Progress output doesn't properly rewrite lines, causing visual corruption and cluttered output.

### [#007: Function Count Always Shows Zero Despite Functions Detected](./007-function-count-always-zero.md)
**Component**: Context generation - function detection

Output always shows `File Complexity: 1 | Functions: 0` even when functions are present and listed in detailed output.

**Example**:
```markdown
### ./src/transcriber/auto_transcriber.rs
File Complexity: 1 | Functions: 0  ❌ Wrong!

- **Function**: `new` [complexity: 3]
- **Function**: `transcribe` [complexity: 8]
- **Function**: `process_batch` [complexity: 4]
```

### [#008: Placeholder Text in Context Report Sections](./008-placeholder-text-in-report.md)
**Component**: Context generation - report sections

Report shows empty placeholder sections like "Key architectural components identified in the codebase" with no actual data.

### [#009: "Copyright" Incorrectly Detected as Function Name](./009-copyright-detected-as-function.md)
**Component**: Function detection - AST parsing

C/C++ analyzer detects copyright headers as functions, polluting function lists with false positives.

**Example**:
```markdown
- **Function**: `Copyright` [complexity: 3] [cognitive: 2]
```

### [#012: Missing Multi-Language Support in `pmat context`](./012-missing-multi-language-support.md)
**Component**: CLI - context command

No way to specify programming language or analyze multiple languages. Relies entirely on auto-detection with no override.

**Impact**: Polyglot projects only get partial analysis, no workaround when detection fails.

---

## Low Severity (3 bugs)

### [#003: `pmat embed` Shows Wrong Examples](./003-embed-wrong-examples.md)
**Component**: CLI - embed subcommand help text

Help text shows generic PMAT examples instead of embedding-specific examples.

### [#006: Incorrect Parallel Analysis Count and Typo](./006-parallel-analysis-count-wrong.md)
**Component**: CLI - context command parallel analysis

Shows "8 parallel analyses" but only 4 run. Also has typo in "analyses" vs "analysis".

### [#010: Warnings Displayed as Errors in File Processing](./010-warnings-shown-as-errors.md)
**Component**: CLI - error/warning display

Warnings about file processing failures are formatted like errors, truncated, and interleaved with progress output.

**Example**:
```
Warning: Error processing file ./pybind/.../gateway_pb2.py: Parameter validation failed: l
```

---

## Bug Report Breakdown by Component

### CLI (7 bugs)
- #001, #002, #003: embed subcommand errors
- #005: Progress output
- #006: Parallel analysis display
- #010: Warning/error formatting
- #012: Multi-language support

### Analysis/Detection (4 bugs)
- #004: Dead code analyzer
- #007: Function count
- #009: Copyright detection
- #011: Language detection

### Report Generation (1 bug)
- #008: Placeholder text

---

## Recommended Priority Order

### Sprint 1: Critical Path (Weeks 1-2)
1. **#011**: Fix language detection + add timeout (blocks C++ projects)
2. **#004**: Add multi-language dead code analysis
3. **#012**: Implement language override flags

### Sprint 2: User Experience (Weeks 3-4)
4. **#007**: Fix function count display
5. **#009**: Filter copyright from function detection
6. **#008**: Fill in placeholder report sections
7. **#005**: Fix progress output formatting

### Sprint 3: Polish (Week 5)
8. **#001, #002**: Fix embed subcommand errors
9. **#003**: Fix embed examples
10. **#006**: Fix parallel analysis count
11. **#010**: Improve warning display

---

## Testing Recommendations

### Test Projects
1. **Rust**: paiml-mcp-agent-toolkit (native language)
2. **C++**: Ceph (github.com/ceph/ceph.git) - large, real-world
3. **Python**: CPython (github.com/python/cpython.git) - large, real-world
4. **Polyglot**: Any monorepo with Rust + Python + TypeScript

### Coverage
- Add integration tests for each bug fix
- Test on real-world projects, not just fixtures
- Include timeout tests for hanging issues
- Test edge cases (empty projects, single-file projects, etc.)

---

## Related Documentation

- **Language Detection**: server/src/cli/language_analyzer.rs
- **Context Generation**: server/src/cli/handlers/context.rs
- **Dead Code Analysis**: server/src/services/dead_code.rs (if exists)
- **Function Detection**: server/src/services/simple_deep_context.rs

---

## Notes

- All bug reports include reproduction steps, expected behavior, and suggested fixes
- Each report includes relevant file paths and test cases
- Reports cross-reference related issues
- Focus on user experience and production-readiness
