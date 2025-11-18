# Rust Project Score Roadmap
## Toyota Way Execution Plan

**Version**: 1.2 Roadmap (2025-11-18)
**Philosophy**: Kaizen (改善) - Small, continuous improvements
**Status**: v1.1 Production | v1.2 In Development

---

## Version History

### v1.1.0 (2025-11-16) - COMPLETE
**Theme**: Evidence-Based Scoring Foundation

| Sprint | Task | Status | Impact |
|--------|------|--------|--------|
| 1 | Core scoring architecture (6 categories, 106 points) | COMPLETE | Foundation |
| 2 | Tiered Clippy scoring (correctness > suspicious > pedantic) | COMPLETE | -40% false positives |
| 3 | FileCache optimization (22 walks → 1) | COMPLETE | 3x speedup |
| 4 | Parallel scorer execution (Rayon) | COMPLETE | 2-3x speedup |

**Kaizen Rounds Completed**: 4-8 (see spec Section 9)

---

## v1.2.0 Roadmap - Formal Verification & Advanced Testing

**Theme**: Toyota Jidoka (自働化) - Built-in Quality Through Formal Methods
**Target**: Q1 2026
**Evidence**: Certeza Research (Nov 2025)

### Sprint 5: Miri Integration (Jidoka for UB)

**Principle**: Stop the line when undefined behavior is detected.

| Task | Description | Points | Priority |
|------|-------------|--------|----------|
| 5.1 | Detect `unsafe` blocks in project | - | HIGH |
| 5.2 | Run `cargo miri test` on unsafe code | - | HIGH |
| 5.3 | Award points for Miri-passing unsafe code | +3 | MEDIUM |
| 5.4 | Generate recommendations for Miri failures | - | LOW |

**Acceptance Criteria**:
- `unsafe` blocks without Miri validation score 0/3
- `unsafe` blocks passing Miri score 3/3
- Partial credit for documented but untested unsafe

**Implementation**:
```rust
// miri_scorer.rs
pub fn score_miri_compliance(project_path: &Path) -> ScorerResult {
    // 1. Find all unsafe blocks
    // 2. Run cargo miri test
    // 3. Score based on pass rate
}
```

---

### Sprint 6: Kani Formal Verification

**Principle**: Mathematical proof of correctness (Genchi Genbutsu - empirical evidence).

| Task | Description | Points | Priority |
|------|-------------|--------|----------|
| 6.1 | Detect Kani proofs in project | - | HIGH |
| 6.2 | Run `cargo kani` on proof harnesses | - | HIGH |
| 6.3 | Award points for verified code | +5 | MEDIUM |
| 6.4 | Track proof coverage percentage | - | LOW |

**Acceptance Criteria**:
- Projects with Kani proofs: +5 points
- Partial credit based on verification coverage
- Bonus for 100% unsafe code verified

**Evidence**: Certeza research shows formal verification eliminates entire bug classes.

---

### Sprint 7: Property-Based Testing Score (NEW CATEGORY)

**Principle**: Kaizen through generative testing.

| Task | Description | Points | Priority |
|------|-------------|--------|----------|
| 7.1 | Detect proptest/quickcheck usage | - | HIGH |
| 7.2 | Count property tests vs example tests | - | MEDIUM |
| 7.3 | Calculate property test ratio | - | MEDIUM |
| 7.4 | Award points for property coverage | +5 | HIGH |

**New Category**: Testing Excellence gains +5 points (20 → 25)

**Scoring Algorithm**:
```rust
// Property tests are worth 100x example tests (Certeza finding)
let property_equivalent = property_tests * 100;
let total_equivalent = property_equivalent + example_tests;
let ratio = property_equivalent as f64 / total_equivalent as f64;

// Score: 0-5 based on ratio
5.0 * ratio.min(1.0)
```

---

### Sprint 8: Auto-Fix Integration (Poka-Yoke)

**Principle**: Error-proofing through automatic correction.

| Task | Description | Points | Priority |
|------|-------------|--------|----------|
| 8.1 | Integrate `cargo clippy --fix` | - | HIGH |
| 8.2 | Integrate `cargo fmt` auto-fix | - | HIGH |
| 8.3 | Generate fix suggestions for all issues | - | MEDIUM |
| 8.4 | One-click fix command generation | - | LOW |

**User Experience**:
```bash
# After scoring, suggest fixes
pmat rust-project-score --suggest-fixes

# Output:
# Clippy: cargo clippy --fix --all-targets
# Format: cargo fmt
# Audit: cargo audit fix
```

---

### Sprint 9: ScoreVelocity Tracking (Kaizen Metrics)

**Principle**: Measure the rate of improvement, not just the score.

| Task | Description | Points | Priority |
|------|-------------|--------|----------|
| 9.1 | Persist historical scores (SQLite/JSON) | - | HIGH |
| 9.2 | Calculate points-per-day improvement | - | HIGH |
| 9.3 | Project days-to-next-grade | - | MEDIUM |
| 9.4 | Identify most-improved category | - | MEDIUM |

**Implementation** (already in models.rs):
```rust
pub struct ScoreVelocity {
    pub current: f64,
    pub previous: f64,
    pub delta: f64,
    pub delta_percent: f64,
    pub days_elapsed: u64,
    pub points_per_day: f64,       // Kaizen metric
    pub most_improved: Option<String>,
    pub days_to_next_grade: Option<u64>,
}
```

---

### Sprint 10: Quick Mode (<10s)

**Principle**: Respect for People (減らす待ち時間 - Reduce wait time).

| Task | Description | Points | Priority |
|------|-------------|--------|----------|
| 10.1 | Filesystem-only analysis mode | - | HIGH |
| 10.2 | Skip all subprocesses | - | HIGH |
| 10.3 | Return partial score with explanation | - | MEDIUM |
| 10.4 | Add --quick flag to CLI | - | HIGH |

**Use Case**: On-save feedback in IDE/editor integration.

---

## v1.3.0 Future Roadmap

**Theme**: CI/CD Integration & Team Metrics

| Feature | Description | Target |
|---------|-------------|--------|
| GitHub Action | Official pmat-score action | Q2 2026 |
| Badge Generation | README score badges | Q2 2026 |
| Team Dashboard | Multi-repo score tracking | Q3 2026 |
| Trend Analysis | Score regression detection | Q3 2026 |
| PR Quality Gate | Block merge if score drops | Q3 2026 |

---

## Implementation Priorities

### Immediate (Sprint 5-6)
1. **Miri Integration**: Highest impact for Rust safety
2. **Kani Integration**: Formal verification for critical code

### Short-Term (Sprint 7-8)
1. **Property Testing Score**: Evidence-based testing quality
2. **Auto-Fix Integration**: Developer experience improvement

### Medium-Term (Sprint 9-10)
1. **ScoreVelocity**: Kaizen tracking
2. **Quick Mode**: IDE integration support

---

## Dependencies

| Dependency | Purpose | Status |
|------------|---------|--------|
| cargo-miri | Undefined behavior detection | Required |
| cargo-kani | Formal verification | Optional |
| proptest | Property-based testing detection | Detection only |
| quickcheck | Property-based testing detection | Detection only |

---

## Quality Gates

Each sprint must pass:
1. All tests passing (100%)
2. Zero clippy warnings
3. Coverage >85%
4. Documentation complete
5. pmat-book updated

**Toyota Way Validation**: Each feature must embody at least one Toyota Way principle.

---

## Related Documentation

- [Specification](rust-project-score-v1.1-spec.md)
- [Implementation Status](../implementation-status-rust-project-score.md)
- [PMAT Book](https://paiml.github.io/pmat-book/)

---

*Generated: 2025-11-18*
*Toyota Way Compliance: Verified*
