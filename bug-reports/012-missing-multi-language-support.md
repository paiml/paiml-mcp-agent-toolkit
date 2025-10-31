# Bug Report: Missing Multi-Language Support in `pmat context`

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Medium → **FIXED** ✅
**Component**: CLI - context command
**Status**: GREEN phase complete (15/15 tests passing, 100%)
**Progress**: Sprint 79 Phase 1 - COMPLETE
**Fixed**: 2025-10-31 (Sprint 79)

## Description

`pmat context` does not have a way to specify a programming language or to support multiple languages. It relies entirely on auto-detection, which:
1. Can be wrong (see bug #011)
2. Forces single-language analysis even in polyglot projects
3. Provides no override mechanism when detection fails

## Steps to Reproduce

```bash
cd /path/to/polyglot-project  # e.g., project with Rust + Python + TypeScript
pmat context
# Will analyze only one language, ignoring others
```

## Current Behavior

```
🔍 Auto-detecting project language...
✅ Detected: rust (confidence: 85.0%)
⠋ Discovering project structure...
[analyzes only Rust files]
```

## Expected Behavior

Should support multiple languages in one analysis:

### Option 1: Auto-detect Multiple Languages
```
🔍 Auto-detecting project languages...
✅ Detected: rust (45.2%), python (32.1%), typescript (22.7%)
⠋ Analyzing multi-language project...
[analyzes all three languages]
```

### Option 2: Manual Language Override
```bash
# Specify single language
pmat context --language cpp

# Specify multiple languages
pmat context --languages rust,python,typescript

# Disable auto-detection
pmat context --no-auto-detect --languages cpp,c
```

### Option 3: Configuration File
```toml
# pmat.toml
[project]
languages = ["rust", "python", "typescript"]
primary_language = "rust"
```

## Impact

- **Polyglot projects** only get partial analysis
- **No workaround** when auto-detection fails
- **Manual fixes required** when language detection is wrong
- **Limits PMAT usefulness** for modern multi-language projects

## Use Cases

1. **Monorepo**: Rust backend + TypeScript frontend + Python ML scripts
2. **Embedded**: C/C++ core + Python bindings + Rust tooling
3. **Web app**: TypeScript + Python backend + SQL
4. **Wrong detection**: C++ project detected as Python (see bug #011)

## Files to Investigate

- `server/src/cli/mod.rs` - CLI argument parsing
- `server/src/cli/handlers/context.rs` - Context command handler
- `server/src/cli/language_analyzer.rs` - Language detection
- Multi-language support infrastructure

## Suggested Implementation

### 1. Add CLI Arguments

```rust
#[derive(Parser, Debug)]
pub struct ContextArgs {
    /// Override language detection (e.g., "rust", "cpp", "python")
    #[arg(long)]
    language: Option<String>,

    /// Specify multiple languages (comma-separated: "rust,python,typescript")
    #[arg(long, value_delimiter = ',')]
    languages: Option<Vec<String>>,

    /// Disable automatic language detection
    #[arg(long)]
    no_auto_detect: bool,

    // ... other args
}
```

### 2. Multi-Language Detection

```rust
pub struct MultiLanguageDetection {
    pub languages: Vec<LanguageInfo>,
    pub primary: String,
}

pub struct LanguageInfo {
    pub language: String,
    pub confidence: f64,
    pub file_count: usize,
    pub percentage: f64,
}

pub fn detect_all_languages(path: &Path) -> MultiLanguageDetection {
    let file_counts = analyze_file_extensions(path);
    let total_files: usize = file_counts.values().sum();

    let mut languages = vec![];
    for (ext, count) in file_counts {
        if let Some(lang) = extension_to_language(ext) {
            let percentage = (count as f64 / total_files as f64) * 100.0;
            if percentage > 5.0 {  // Include languages with >5% of files
                languages.push(LanguageInfo {
                    language: lang,
                    confidence: calculate_confidence(&lang, path),
                    file_count: count,
                    percentage,
                });
            }
        }
    }

    languages.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap());

    MultiLanguageDetection {
        primary: languages.first().unwrap().language.clone(),
        languages,
    }
}
```

### 3. Multi-Language Analysis

```rust
pub async fn analyze_multi_language(
    path: &Path,
    languages: &[String],
) -> Result<MultiLanguageContext> {
    let mut contexts = HashMap::new();

    for lang in languages {
        let analyzer = get_language_analyzer(lang)?;
        let ctx = analyzer.analyze(path).await?;
        contexts.insert(lang.clone(), ctx);
    }

    Ok(MultiLanguageContext {
        languages: contexts,
        primary: languages[0].clone(),
    })
}
```

## Examples

```bash
# Auto-detect all languages (new default behavior)
pmat context

# Override to specific language
pmat context --language cpp

# Force multi-language analysis
pmat context --languages rust,python,typescript

# Analyze only Python, disable auto-detect
pmat context --no-auto-detect --language python
```

## Test Cases

```rust
#[test]
fn test_multi_language_detection() {
    let project = create_polyglot_project(); // Rust + Python + TS
    let detection = detect_all_languages(&project);

    assert_eq!(detection.languages.len(), 3);
    assert!(detection.languages.iter().any(|l| l.language == "rust"));
    assert!(detection.languages.iter().any(|l| l.language == "python"));
    assert!(detection.languages.iter().any(|l| l.language == "typescript"));
}

#[test]
fn test_language_override() {
    let args = ContextArgs {
        language: Some("cpp".to_string()),
        ..Default::default()
    };

    let detection = get_effective_languages(&args, &project);
    assert_eq!(detection.languages.len(), 1);
    assert_eq!(detection.languages[0], "cpp");
}
```

## Priority

**Medium-High**: Essential for real-world projects that use multiple languages.

## Related Issues

- Bug #011: Wrong language detection needs manual override capability
- Bug #004: Dead code analyzer needs multi-language support

---

## TDD PROGRESS - RED PHASE COMPLETE 🔴

**Date**: 2025-10-31
**Phase**: RED (Test Definition)
**Status**: ✅ Complete - 6 failing tests written

### RED Phase Tests Created

**File**: `server/tests/bug_012_multi_language_cli_tests.rs` (272 lines)

1. ✅ **test_language_override_single**
   - Tests: `--language python` flag
   - Expected: Analyze only Python files, ignore others
   - Status: 🔴 Failing (not implemented)

2. ✅ **test_languages_override_multiple**
   - Tests: `--languages rust,python` flag
   - Expected: Analyze multiple specified languages
   - Status: 🔴 Failing (not implemented)

3. ✅ **test_language_override_invalid_language**
   - Tests: Error handling for unsupported languages
   - Expected: Helpful error message for `--language fortran`
   - Status: 🔴 Failing (not implemented)

4. ✅ **test_language_override_beats_auto_detection**
   - Tests: Override precedence over auto-detection
   - Expected: `--language c` should override auto-detected C++
   - Status: 🔴 Failing (not implemented)

5. ✅ **test_uses_enhanced_language_detection**
   - Tests: Integration with BUG-011 enhanced detection
   - Expected: Auto-detect using confidence scoring
   - Status: 🔴 Failing (not implemented)

6. ✅ **test_language_name_case_insensitive**
   - Tests: Case-insensitive language names
   - Expected: `python`, `Python`, `PYTHON` all work
   - Status: 🔴 Failing (not implemented)

### Test Results (RED Phase)

```
running 6 tests
test test_language_name_case_insensitive ... FAILED
test test_language_override_beats_auto_detection ... FAILED
test test_language_override_invalid_language ... FAILED
test test_language_override_single ... FAILED
test test_languages_override_multiple ... FAILED
test test_uses_enhanced_language_detection ... FAILED

test result: FAILED. 0 passed; 6 failed; 0 ignored
```

✅ **RED Phase Verification**: All tests fail as expected - ready for GREEN phase

### Next Steps (GREEN Phase)

**Implementation Required:**
1. Add CLI argument fields:
   ```rust
   #[arg(long)]
   language: Option<String>,
   
   #[arg(long, value_delimiter = ',')]
   languages: Option<Vec<String>>,
   ```

2. Implement language override logic in context handler
3. Integrate with `enhanced_language_detection` (BUG-011)
4. Normalize language names (case-insensitive)
5. Validate language support
6. Make all 6 tests pass (GREEN)

**Quality Requirements:**
- All 6 tests must pass (100%)
- Cargo example demonstrating flags
- pmat-book chapter via TDD
- Quality gates: TDG, compilation, tests

**Estimated Effort**: 2-3 hours
**Methodology**: Extreme TDD (RED ✅ → GREEN → REFACTOR → COMMIT)

**Commit (RED)**: 53db52aa

---

## FIXED ✅ (2025-10-31 - Sprint 79 Phase 1)

### Solution Implemented

Created CLI language override functionality with full integration into `pmat context` command:

**Architecture:**
- `LanguageOverride` struct with `language` and `languages` fields
- CLI argument parsing via clap (`--language` and `--languages`)
- Integration with enhanced_language_detection (BUG-011)
- 3-tier priority: single override > multiple override > auto-detection

**Implementation Details:**
- **CLI Arguments**: Added to Commands::Context in server/src/cli/commands.rs
  - `--language <LANG>`: Override to single language
  - `--languages <LANG1>,<LANG2>`: Specify multiple languages (comma-separated)
  - Case-insensitive language names (Python, PYTHON, python all work)

- **Language Override Module**: server/src/services/language_override.rs (262 lines)
  - `get_effective_languages()`: Main API with 3-tier priority
  - `normalize_language_name()`: Case-insensitive normalization
  - `validate_language_support()`: Whitelist-based validation
  - Supported languages: rust, python, javascript, typescript, go, cpp, c, java, kotlin, swift, ruby, php, bash, sh, shell, wasm, wat

- **CLI Integration**: Updated command dispatcher and handler
  - server/src/cli/command_dispatcher.rs - Added language parameters
  - server/src/cli/command_structure.rs - Updated UtilityCommandGroup
  - server/src/cli/handlers/utility_handlers.rs - Applied override logic

**Test Coverage:**
- 6 integration tests (100% passing)
- 9 unit tests in language_override module (100% passing)
- Cargo example: `cargo run --example bug_012_multi_language_cli`

**Files Changed:**
- `server/src/services/language_override.rs` (262 lines, NEW)
- `server/src/services/mod.rs` (+1 line, module registration)
- `server/src/cli/commands.rs` (+10 lines, CLI arguments)
- `server/src/cli/command_dispatcher.rs` (+2 lines, parameter passing)
- `server/src/cli/command_structure.rs` (+4 lines, method signature)
- `server/src/cli/handlers/utility_handlers.rs` (+13 lines, override logic)
- `server/src/unified_protocol/adapters/cli.rs` (+2 lines, pattern match)
- `server/tests/bug_012_multi_language_cli_tests.rs` (317 lines, tests)
- `server/examples/bug_012_multi_language_cli.rs` (205 lines, demo)

**Test Results:**
```
running 6 tests
test test_language_name_case_insensitive ... ok
test test_language_override_beats_auto_detection ... ok
test test_language_override_invalid_language ... ok
test test_language_override_single ... ok
test test_languages_override_multiple ... ok
test test_uses_enhanced_language_detection ... ok

test result: ok. 6 passed; 0 failed
```

**Methodology:** Extreme TDD (RED ✅ → GREEN ✅ → REFACTOR → COMMIT)

**Quality Gates:** ✅ All passing
- Compilation: Clean
- Tests: 15/15 passing (100%) - 6 integration + 9 unit
- TDG: No regressions
- Cargo example: Verified

**Usage Examples:**
```bash
# Override to specific language
pmat context --language python

# Analyze multiple languages
pmat context --languages rust,python,typescript

# Case-insensitive
pmat context --language Python
pmat context --language PYTHON

# Auto-detection fallback (uses BUG-011)
pmat context
```

**User Impact:**
- ✅ Manual language override when auto-detection is wrong
- ✅ Multi-language project support (Rust + Python + TypeScript)
- ✅ Case-insensitive language names for better UX
- ✅ Helpful error messages for unsupported languages
- ✅ Seamless integration with BUG-011 enhanced detection
- ✅ Clear priority: override > auto-detection

**Documentation:**
- Bug report updated with FIXED section
- Cargo example demonstrates all features
- pmat-book chapter: TBD

**Sprint 79 Phase 1 Status:**
- ✅ BUG-011: Enhanced Language Detection (COMPLETE)
- ✅ BUG-004: Multi-Language Dead Code (COMPLETE)
- ✅ BUG-012: CLI Language Override (COMPLETE)

**Commit (GREEN)**: 33c73839
