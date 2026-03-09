# Issue: Integrate Known Defects with TDG Auto-Fail

**Priority**: HIGH
**Sprint**: 2
**Estimate**: 5 story points
**Labels**: enhancement, tdg, known-defects

## Summary

Integrate Known Defects database with TDG analyzer to auto-fail files containing critical defects (e.g., `.unwrap()` in Rust) and display defects in `--explain` output.

## User Story

As a **developer**
I want **TDG to auto-fail files with known critical defects**
So that **production defect patterns are caught before code review**

## Acceptance Criteria

- [ ] TDG analyzer detects known defects using O(1) language lookup
- [ ] Files with CRITICAL defects receive TDG score = 0.0 and Grade F
- [ ] `pmat tdg --explain <file>` shows detected defects with:
  - Defect pattern ID (e.g., RUST-UNWRAP-001)
  - Evidence (Cloudflare incident, CVE, research citation)
  - Fix recommendation with bad/good examples
- [ ] Defect severity badges displayed in TDG reports
- [ ] Test coverage ≥85% for defect detection logic

## Technical Design

### TDG Analyzer Integration

```rust
// server/src/tdg/analyzer.rs
impl TdgAnalyzer {
    fn analyze_file(&self, path: &Path, language: Language) -> TdgResult {
        // 1. Detect known defects using DefectDatabase
        let defects = DEFECT_DATABASE.detect(path, language)?;

        // 2. Auto-fail on critical defects
        if defects.iter().any(|d| d.severity == Severity::Critical) {
            return TdgResult {
                score: 0.0,
                grade: Grade::F,
                defects: defects.clone(),
                auto_failed: true,
                message: format!("Auto-failed due to {} critical defect(s)",
                                 defects.len()),
            };
        }

        // 3. Normal TDG complexity analysis...
        let tdg_score = self.calculate_tdg(path)?;

        TdgResult {
            score: tdg_score.score,
            grade: tdg_score.grade,
            defects,
            auto_failed: false,
            message: String::new(),
        }
    }
}
```

### --explain Output Format

```bash
$ pmat tdg --explain src/main.rs

Function-Level Complexity Breakdown
===================================

main (line 10)
  Complexity: 15
  Cognitive: 18
  TDG Impact: 3.2
  Severity: High

❌ DEFECTS DETECTED (Auto-Fail)
================================

CRITICAL: .unwrap() at line 42
  Pattern: RUST-UNWRAP-001
  Evidence: Cloudflare outage 2025-11-18 (3+ hour network outage)

  thread fl2_worker_thread panicked: called Result::unwrap() on an Err value

  Fix: Use .expect() with descriptive messages or proper error handling

  Bad:  let config = File::open("config.toml").unwrap();

  Good: let config = File::open("config.toml")
                       .expect("Config file must exist and be readable");

  Alt:  let config = File::open("config.toml")
                       .context("Failed to load configuration")?;
```

## Implementation Tasks

1. **DefectDatabase Creation** (1 day)
   - Create `server/src/defects/mod.rs` module
   - Implement per-language defect registry
   - Add Rust defects from spec (RUST-UNWRAP-001, etc.)

2. **TDG Analyzer Integration** (2 days)
   - Add defect detection to `tdg/analyzer.rs`
   - Implement auto-fail logic for critical defects
   - Update TdgResult struct to include defects

3. **TDG Explain Formatters** (1 day)
   - Update `tdg/explain_formatters.rs` to show defects
   - Add defect severity badges
   - Format evidence and recommendations

4. **Testing** (1 day)
   - Unit tests for defect detection (>95% precision)
   - Integration tests for auto-fail behavior
   - E2E tests for --explain output

## Test Plan

### Unit Tests

```rust
#[test]
fn test_rust_unwrap_detected() {
    let code = r#"
        fn main() {
            let x = Some(42).unwrap();
        }
    "#;

    let defects = DEFECT_DATABASE.detect_in_content(code, Language::Rust);

    assert_eq!(defects.len(), 1);
    assert_eq!(defects[0].id, "RUST-UNWRAP-001");
    assert_eq!(defects[0].severity, Severity::Critical);
}

#[test]
fn test_tdg_auto_fail_on_critical_defect() {
    let analyzer = TdgAnalyzer::new();
    let result = analyzer.analyze_file("test.rs", Language::Rust).unwrap();

    assert_eq!(result.score, 0.0);
    assert_eq!(result.grade, Grade::F);
    assert!(result.auto_failed);
    assert_eq!(result.defects.len(), 1);
}
```

### Integration Tests

- File with critical defect → TDG score = 0.0
- File with high defect → TDG score reduced but not auto-failed
- File with no defects → Normal TDG scoring

## Dependencies

- Specification: `docs/specifications/components/language-support.md` ✅
- Issue #2: `pmat analyze defects` command (can be parallel)

## Success Metrics

- Zero tolerance for production defects
- 100% detection rate for known defects in TDG
- <5% false positive rate
- Sub-second defect detection performance

## References

- Cloudflare incident: https://blog.cloudflare.com/2025-01-18-outage
- Spec: `docs/specifications/components/language-support.md`
