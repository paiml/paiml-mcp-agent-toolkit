# QA Validation Protocol: pmat comply v2.3.0

**Target Release:** pmat v2.3.0
**Spec Reference:** `docs/specifications/components/repo-health.md`
**Pass Criteria:** 100% of applicable checks must pass.
**Falsification Logic:** Any finding where `pmat comply` fails to detect a violation (False Negative) or flags valid code (False Positive) is a blocking regression.

---

## I. Critical Code Safety (The "Sovereign Stack" Patterns)

### CB-070: Critical Unwrap Detection
| ID | Test Case | Falsification Strategy (Try to break) | Pass/Fail |
|----|-----------|---------------------------------------|-----------|
| 01 | Basic `.unwrap()` | Create a `fn main() { opt.unwrap(); }`. Must fail. | |
| 02 | Basic `.expect()` | Create `fn main() { res.expect("boom"); }`. Must fail (Warning). | |
| 03 | `.unwrap_err()` | Use `result.unwrap_err()`. Must fail (often missed). | |
| 04 | Unwrap in `println!` | Use `println!("{}", x.unwrap())`. Must fail. | |
| 05 | Unwrap in Closure | Use `let f = || x.unwrap();`. Must fail. | |
| 06 | Unwrap in Match Guard | Use `match x { y if y.unwrap() > 0 => ... }`. Must fail. | |
| 07 | Custom Method Name | Create a struct with a safe method named `unwrap()`. **Must Pass (False Positive Check)**. | |
| 08 | Test Code Exclusion | Put `.unwrap()` inside `#[test] fn t()`. **Must Pass**. | |
| 09 | `unwrap_or` / `unwrap_or_default` | Use `.unwrap_or(0)`. **Must Pass**. | |
| 10 | Unwrap in String | `let s = "don't unwrap() me";`. **Must Pass**. | |

### CB-120 & CB-121: Math & Concurrency Safety (OIP/Tarantula)
| ID | Test Case | Falsification Strategy | Pass/Fail |
|----|-----------|------------------------|-----------|
| 11 | NaN Sort Panic | Code: `vec.sort_by(|a,b| a.partial_cmp(b).unwrap())`. Must fail. | |
| 12 | NaN `total_cmp` | Code: `vec.sort_by(|a,b| a.total_cmp(b))`. **Must Pass**. | |
| 13 | Lock Write Poison | Code: `mutex.write().unwrap()`. Must fail. | |
| 14 | Lock Read Poison | Code: `rwlock.read().unwrap()`. Must fail. | |
| 15 | Lock `unwrap_or_else` | Code: `mutex.lock().unwrap_or_else(...)`. **Must Pass**. | |
| 16 | Chained Lock | Code: `data.lock().unwrap().insert(...)`. Must fail. | |

---

## II. Stub & SATD Detection (CB-050)

| ID | Test Case | Falsification Strategy | Pass/Fail |
|----|-----------|------------------------|-----------|
| 17 | Rust `todo!()` | Standard `todo!()` macro. Must fail. | |
| 18 | Rust `unimplemented!()` | Standard macro. Must fail. | |
| 19 | Explicit Panic | `panic!("not implemented")`. Must fail. | |
| 20 | Empty Function | `fn implementation() {}` (non-trait). Must Warn/Fail. | |
| 21 | Python Stub | `raise NotImplementedError`. Must fail. | |
| 22 | Python Pass | `pass # todo`. Must fail. | |
| 23 | Spacing Attack | `todo ! ( )` (weird whitespace). Must fail. | |
| 24 | Commented Out | `// todo!()`. **Must Pass**. | |
| 25 | String Literal | `let s = "todo!()"`. **Must Pass**. | |
| 26 | Trait Default | `trait T { fn def() {} }`. **Must Pass**. | |

---

## III. GPU & Compute Quality (CB-060)

| ID | Test Case | Falsification Strategy | Pass/Fail |
|----|-----------|------------------------|-----------|
| 27 | PTX Barrier Divergence | PTX: `bra L_exit; ... bar.sync`. Must fail. | |
| 28 | WGSL Barrier Loop | WGSL: `if (id < 5) { workgroupBarrier(); }`. Must fail. | |
| 29 | Unbounded Shared Mem | PTX: `ld.shared` without `setp.lt` check. Must fail. | |
| 30 | Tiled Kernel Bounds | Loop over tiles without boundary check. Must fail. | |
| 31 | Safe Barrier | `bar.sync` before branch. **Must Pass**. | |
| 32 | Non-GPU Project | Run checks on pure CPU repo. **Must Pass/Skip**. | |

---

## IV. Data Integrity & Serde (CB-100, CB-122)

| ID | Test Case | Falsification Strategy | Pass/Fail |
|----|-----------|------------------------|-----------|
| 33 | Serde JSON Unwrap | `serde_json::from_str(s).unwrap()`. Must fail. | |
| 34 | Serde YAML Expect | `serde_yaml::from_str(s).expect("die")`. Must fail. | |
| 35 | Safe Serde | `let res: Result<T> = serde_json::from_str(s);`. **Must Pass**. | |
| 36 | Unsafe Transmute | `unsafe { std::mem::transmute(x) }` in serialize fn. Must fail. | |
| 37 | Asymmetric Serde | Struct has `serialize()` but no `deserialize()`. Must Warn. | |
| 38 | Missing Roundtrip | Project has `.gguf` files but no `test_roundtrip`. Must fail. | |
| 39 | Reshape No Layout | `tensor.reshape(...)` without checking layout. Must fail. | |
| 40 | Quantize Cast | `val as f16` inside quantization logic. Must Warn. | |

---

## V. Testing & Quality Standards (CB-090, CB-123, CB-124, CB-126)

| ID | Test Case | Falsification Strategy | Pass/Fail |
|----|-----------|------------------------|-----------|
| 41 | Sleep in Test | `thread::sleep(...)` inside `#[test]`. Must fail. | |
| 42 | Hardcoded Time | `assert!(elapsed < Duration::from_millis(10))` . Must fail. | |
| 43 | Busy Wait | `while !flag { yield_now() }` in test. Must fail. | |
| 44 | Fixed Port | `bind("127.0.0.1:8080")` in test. Must fail. | |
| 45 | Fixed Temp Path | `File::create("/tmp/test.txt")` in test. Must fail. | |
| 46 | Silent Ignore | `#[ignore]` with no comment/reason. Must fail. | |
| 47 | Documented Ignore | `#[ignore] // GPU required`. **Must Pass**. | |
| 48 | Doc Test Ignore | `/// #[ignore]` in doc comment. **Must Pass**. | |
| 49 | Coverage < 80% | `pmat-quality.toml` threshold set to 58%. Must Warn/Fail. | |
| 50 | Coverage > 80% | Threshold set to 85%. **Must Pass**. | |
| 51 | Slow Test > 5s | Create test that sleeps 6s. Must Warn (Tier 1 violation). | |
| 52 | Slow Test > 60s | Create test that sleeps 61s. Must Error (Tier 2 violation). | |
| 53 | Unbounded Proptest | `ProptestConfig::default()` without cases. Must Warn. | |

---

## VI. Dependency & Platform Management (CB-080, CB-110)

| ID | Test Case | Falsification Strategy | Pass/Fail |
|----|-----------|------------------------|-----------|
| 54 | Path Dep (Prod) | `dep = { path = "../local" }` in `[dependencies]`. Must fail. | |
| 55 | Path Dep (Dev) | `dep = { path = "..." }` in `[dev-dependencies]`. **Must Pass/Warn**. | |
| 56 | Git Dependency | `dep = { git = "..." }`. Must fail (instability). | |
| 57 | Stack Version Drift | `aprender = "0.20"` and `trueno = "0.11"` (incompatible). Must fail. | |
| 58 | Platform Cfg Gap | Code has `#[cfg(windows)]` but CI only tests Linux. Must fail. | |
| 59 | WASM Gap | `wasm-bindgen` in Cargo.toml but no `wasm32` CI job. Must fail. | |
| 60 | Unix Family Gap | `#[cfg(unix)]` used, CI tests Linux but *misses* macOS. Must fail. | |
| 61 | Duplicate Versions | `Cargo.lock` has 2 major versions of `rand`. Warn. | |

---

## VII. Coverage Quality & Gaming (CB-125, CB-127)

| ID | Test Case | Falsification Strategy | Pass/Fail |
|----|-----------|------------------------|-----------|
| 62 | Excessive Exclusions | Add 15 regex patterns to `COVERAGE_EXCLUDE`. Must fail. | |
| 63 | Entry Point Exclusion | Exclude `src/main.rs`. Must fail. | |
| 64 | Broad Exclusion | Exclude `src/core/.*`. Must fail. | |
| 65 | Generated Code | Exclude `target/` or `bindings/`. **Must Pass**. | |
| 66 | Slow Runner | Use `cargo llvm-cov nextest` (profraw explosion). Must Error. | |
| 67 | Missing --lib | Run coverage without `--lib`. Must Warn (includes slow tests). | |
| 68 | Unbounded Coverage | Run coverage without `PROPTEST_CASES` env var. Must Warn. | |

---

## VIII. Dead Code & Entropy (CB-128)

| ID | Test Case | Falsification Strategy | Pass/Fail |
|----|-----------|------------------------|-----------|
| 69 | Dead Private Fn | `fn unused() {}`. Must fail. | |
| 70 | Dead Struct Field | `struct S { dead: i32 }` (never read). Must fail. | |
| 71 | Public Exported | `pub fn api() {}` in `lib.rs`. **Must Pass**. | |
| 72 | Zombie Public | `pub fn internal_api() {}` unused in entire workspace. Must fail. | |
| 73 | Dead Test Helper | `#[cfg(test)] fn unused_helper() {}`. Must Warn. | |
| 74 | Dead Import | `use std::fmt::Display;` (unused). Must fail. | |
| 75 | Allowed Dead Code | `#[allow(dead_code)] fn u() {}`. **Must Pass (Info)**. | |
| 76 | Dead Const | `const X: i32 = 0;`. Must fail. | |
| 77 | Trait Impl | `impl Display for S` (methods unused but required). **Must Pass**. | |
| 78 | Dogfood Check | Run on pmat itself. Must find >0 issues (real world). | |

---

## IX. Suppression Infrastructure (User Control)

| ID | Test Case | Falsification Strategy | Pass/Fail |
|----|-----------|------------------------|-----------|
| 79 | Global Suppression | Add `pattern="**/*"` for CB-050 in `.pmat/suppressions.toml`. Violations must vanish. | |
| 80 | Specific File | Suppress `src/legacy.rs` only. Violations elsewhere must persist. | |
| 81 | Line Range | Suppress lines 10-15. Violation on line 20 must persist. | |
| 82 | Expired Suppression | Set `expires = "2020-01-01"`. Violation must reappear. | |
| 83 | Future Expiry | Set `expires = "2099-01-01"`. Violation must be hidden. | |
| 84 | Missing Reason | Add rule without `reason` field. Config parser must error. | |
| 85 | Invalid TOML | Corrupt the `.toml` file. Tool must error (not panic). | |
| 86 | Inline Comment | `// pmat-ignore: CB-070`. Violation must be hidden. | |
| 87 | Conditional Suppress | Suppress `if os == windows`. Run on Linux. Violation persists. | |
| 88 | Invalid ID | Suppress `CB-999` (non-existent). Warn user. | |

---

## X. System Integration & Resilience

| ID | Test Case | Falsification Strategy | Pass/Fail |
|----|-----------|------------------------|-----------|
| 89 | Binary File | Run on directory with `image.png` containing text "todo!()". **Must Pass**. | |
| 90 | Large Repo | Run on repo with 10,000 files. Must finish < 30s. | |
| 91 | No Permissions | `chmod 000 src/lib.rs`. Tool handles error gracefully. | |
| 92 | Gitignore | Put `todo!()` in `build/` (which is gitignored). **Must Pass**. | |
| 93 | Symlinks | Loop symlink. Tool detects/ignores, doesn't hang. | |
| 94 | Deep Nesting | Directory depth > 50. Tool scans correctly. | |
| 95 | Weird Encodings | File with ISO-8859-1. Tool attempts read or skips, no panic. | |
| 96 | Output JSON | Verify JSON output has `file`, `line`, `pattern_id`. | |
| 97 | Exit Codes | Ensure exit code 1 on violations, 0 on success. | |
| 98 | Idempotency | Run twice. Second run should match first (or faster). | |
| 99 | Help Message | `pmat comply --help` lists new checks (CB-125+). | |
| 100 | Zero-Width Char | `todo!(\u{200B})`. Must fail. | |

---

### End of Protocol
**QA Sign-off:** ___________________
**Date:** ___________________
