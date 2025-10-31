# Bug Report: Incorrect Language Detection (python-uv instead of C++)

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: High
**Component**: Language detection

## Description

When running `pmat context` in the top level of the Ceph project (github.com/ceph/ceph.git), the language detector incorrectly identifies it as a "python-uv" project with 57.2% confidence. The Ceph project is primarily C++ with some Python, not a Python project and definitely not a UV-managed Python project.

The process then hangs indefinitely on "Discovering project structure..." and never completes.

## Steps to Reproduce

```bash
git clone https://github.com/ceph/ceph.git
cd ceph
pmat context
```

## Actual Output

```
🔍 Auto-detecting project language...
✅ Detected: python-uv (confidence: 57.2%)
⠋ Discovering project structure...
^C
```

Process never completes, must be killed with Ctrl+C.

## Expected Behavior

Should detect as C++ project:

```
🔍 Auto-detecting project language...
✅ Detected: cpp (confidence: 85.3%)
⠋ Discovering project structure...
[continues processing...]
```

## Ceph Project Stats

Approximate language breakdown:
- **C++**: ~70-80% of codebase
- **C**: ~10-15%
- **Python**: ~5-10% (tools, tests, admin scripts)
- Other: Shell, CMake, etc.

Key indicators of C++ project:
- `CMakeLists.txt` in root
- Large `src/` directory with `.cc` and `.h` files
- C++ build system (CMake)
- No `pyproject.toml` or `uv.lock` in root

## Analysis

### Why python-uv Detection is Wrong

1. **UV Project Detection**: What makes PMAT think this is a UV project?
   - Does it have a `pyproject.toml` somewhere?
   - Is it detecting Python scripts in subdirectories?
   - Is the confidence algorithm weighted incorrectly?

2. **Low Confidence (57.2%)**: Even the detector isn't sure
   - Should have fallback or ask user when confidence < 70%
   - Should weight primary language (most files) higher

3. **Hanging Issue**: Why does it hang after wrong detection?
   - Likely looking for Python/UV-specific files that don't exist
   - No timeout on discovery phase
   - Infinite loop in project structure detection

## Impact

- **CRITICAL**: Cannot analyze large, important C++ projects
- Wrong language detection leads to wrong analysis strategy
- Hanging process wastes user time and resources
- Blocks PMAT adoption for C++ projects

## Files to Investigate

- `server/src/cli/language_analyzer.rs` - Language detection logic
- Project structure discovery after language detection
- Confidence calculation algorithm
- UV project detection heuristics

## Suggested Fix

### 1. Fix Language Detection Algorithm

```rust
pub fn detect_project_language(path: &Path) -> LanguageDetection {
    let mut scores = HashMap::new();

    // Primary indicators (high weight)
    if path.join("Cargo.toml").exists() {
        scores.insert("rust", 90.0);
    }
    if path.join("CMakeLists.txt").exists() {
        scores.insert("cpp", 85.0);
    }
    if path.join("package.json").exists() {
        scores.insert("javascript", 85.0);
    }

    // Secondary indicators (file count analysis)
    let file_counts = count_files_by_extension(path);
    let total: usize = file_counts.values().sum();

    for (ext, count) in file_counts {
        let percentage = (count as f64 / total as f64) * 100.0;
        match ext.as_str() {
            "rs" => *scores.entry("rust").or_insert(0.0) += percentage,
            "cc" | "cpp" | "cxx" | "h" | "hpp" => {
                *scores.entry("cpp").or_insert(0.0) += percentage
            }
            "py" => *scores.entry("python").or_insert(0.0) += percentage * 0.8,
            _ => {}
        }
    }

    // Only check for UV if Python is detected
    if scores.get("python").unwrap_or(&0.0) > &30.0 {
        if path.join("uv.lock").exists() {
            scores.insert("python-uv", scores["python"] + 10.0);
        }
    }

    // Find highest score
    scores.into_iter()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(lang, conf)| LanguageDetection { language: lang, confidence: conf })
        .unwrap()
}
```

### 2. Add Timeout to Discovery

```rust
use tokio::time::{timeout, Duration};

let discovery = timeout(
    Duration::from_secs(30),
    discover_project_structure(path)
).await??;
```

### 3. Add User Confirmation for Low Confidence

```rust
if detection.confidence < 70.0 {
    eprintln!("⚠️  Low confidence detection: {} ({:.1}%)",
        detection.language, detection.confidence);
    eprintln!("Would you like to specify the language manually? [Y/n]");
    // ... handle user input
}
```

## Test Case

```rust
#[test]
fn test_ceph_project_detected_as_cpp() {
    let ceph_project = create_mock_ceph_structure();
    let detection = detect_project_language(&ceph_project);

    assert_eq!(detection.language, "cpp");
    assert!(detection.confidence > 70.0);
}

#[test]
fn test_discovery_timeout() {
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            timeout(
                Duration::from_secs(5),
                discover_project_structure(&large_project)
            ).await
        });

    assert!(result.is_ok(), "Discovery should complete within timeout");
}
```
