# Bug Report: Missing Multi-Language Support in `pmat context`

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Medium → 🔴 RED (TDD in progress)
**Component**: CLI - context command
**Status**: RED phase complete (6/6 tests written, all failing)
**Progress**: Sprint 79 Phase 1 - In development

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

**Commit**: 53db52aa
