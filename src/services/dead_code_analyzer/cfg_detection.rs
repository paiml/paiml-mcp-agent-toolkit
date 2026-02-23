//! Conditional compilation (`#[cfg(...)]`) attribute detection.
//!
//! Functions for determining if code is behind conditional compilation gates,
//! which should be excluded from dead code reports.

/// Check if a function is behind conditional compilation attributes.
///
/// Detects functions gated by:
/// - `#[cfg(target_arch = ...)]` and other `#[cfg(...)]` variants
/// - `#[cfg_attr(...)]` conditional attributes
/// - `#[target_feature(enable = "...")]` (SIMD intrinsics)
///
/// Also detects module-level cfg gating by scanning for `#[cfg` on
/// `mod` blocks that enclose the function.
pub(crate) fn is_cfg_gated(file_path: &str, fn_line: u32) -> bool {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let lines: Vec<&str> = content.lines().collect();
    let fn_idx = fn_line.saturating_sub(1) as usize;

    has_cfg_attribute_above(&lines, fn_idx) || is_inside_cfg_module(&lines, fn_idx)
}

/// Check if a line is a conditional compilation attribute.
fn is_conditional_attr(trimmed: &str) -> bool {
    trimmed.starts_with("#[cfg(")
        || trimmed.starts_with("#[cfg_attr(")
        || trimmed.starts_with("#[target_feature(")
}

/// Check if a line is an annotation or preamble (attributes, comments, keywords).
fn is_annotation_or_preamble(trimmed: &str) -> bool {
    trimmed.is_empty()
        || trimmed.starts_with("#[")
        || trimmed.starts_with("///")
        || trimmed.starts_with("//!")
        || trimmed.starts_with("//")
        || trimmed.starts_with("pub ")
        || trimmed.starts_with("pub(")
        || trimmed.starts_with("async ")
        || trimmed.starts_with("unsafe ")
        || trimmed == "{"
        || trimmed == "}"
}

/// Scan up to 10 lines above a function for direct cfg/target_feature attributes.
fn has_cfg_attribute_above(lines: &[&str], fn_idx: usize) -> bool {
    let start = fn_idx.saturating_sub(10);
    for i in start..fn_idx {
        let Some(line) = lines.get(i) else { continue };
        let trimmed = line.trim();
        if is_conditional_attr(trimmed) {
            return true;
        }
        if !is_annotation_or_preamble(trimmed) {
            break;
        }
    }
    false
}

/// Check if a line is inside a module that has a `#[cfg(...)]` attribute.
///
/// Scans from the top of the file, tracking `#[cfg]` attributes on `mod` blocks
/// and brace nesting to determine if the function at `fn_idx` is enclosed in
/// a cfg-gated module.
fn is_inside_cfg_module(lines: &[&str], fn_idx: usize) -> bool {
    let mut brace_depth: i32 = 0;
    let mut cfg_depth: Option<i32> = None;
    let mut pending_cfg = false;

    for (i, line) in lines.iter().enumerate() {
        if i > fn_idx {
            break;
        }
        let trimmed = line.trim();

        if is_conditional_attr(trimmed) {
            pending_cfg = true;
            continue;
        }

        if pending_cfg && is_cfg_mod_start(trimmed) {
            cfg_depth = Some(brace_depth);
        }
        pending_cfg = false;

        brace_depth = update_brace_depth(trimmed, brace_depth, &mut cfg_depth);
    }

    cfg_depth.is_some()
}

/// Check if a trimmed line starts a module block (e.g. `mod foo {` or `pub mod foo {`).
fn is_cfg_mod_start(trimmed: &str) -> bool {
    (trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ")) && trimmed.contains('{')
}

/// Update brace depth by counting `{` and `}` in a line.
/// Clears `cfg_depth` when the cfg-gated module scope is closed.
fn update_brace_depth(trimmed: &str, mut depth: i32, cfg_depth: &mut Option<i32>) -> i32 {
    for ch in trimmed.chars() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if let Some(d) = *cfg_depth {
                    if depth <= d {
                        *cfg_depth = None;
                    }
                }
            }
            _ => {}
        }
    }
    depth
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cfg_gated_direct_cfg() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &test_file,
            "#[cfg(target_arch = \"x86_64\")]\nfn simd_func() {}\n",
        )
        .unwrap();
        assert!(is_cfg_gated(&test_file.to_string_lossy(), 2));
    }

    #[test]
    fn test_is_cfg_gated_target_feature() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &test_file,
            "#[target_feature(enable = \"avx2\")]\nunsafe fn avx2_func() {}\n",
        )
        .unwrap();
        assert!(is_cfg_gated(&test_file.to_string_lossy(), 2));
    }

    #[test]
    fn test_is_cfg_gated_cfg_attr() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &test_file,
            "#[cfg_attr(feature = \"simd\", target_feature(enable = \"sse4.1\"))]\nfn conditional_simd() {}\n",
        )
        .unwrap();
        assert!(is_cfg_gated(&test_file.to_string_lossy(), 2));
    }

    #[test]
    fn test_is_cfg_gated_not_gated() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn normal_func() {}\n").unwrap();
        assert!(!is_cfg_gated(&test_file.to_string_lossy(), 1));
    }

    #[test]
    fn test_is_cfg_gated_module_level() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(
            &test_file,
            "#[cfg(target_arch = \"x86_64\")]\nmod simd {\n    fn inner_func() {}\n}\n",
        )
        .unwrap();
        // inner_func is at line 3, inside a cfg-gated module
        assert!(is_cfg_gated(&test_file.to_string_lossy(), 3));
    }

    #[test]
    fn test_is_inside_cfg_module_basic() {
        let lines: Vec<&str> = vec![
            "#[cfg(target_arch = \"x86_64\")]",
            "mod simd {",
            "    fn a() {}",
            "    fn b() {}",
            "}",
            "fn outside() {}",
        ];
        // fn a() at index 2 is inside cfg mod
        assert!(is_inside_cfg_module(&lines, 2));
        // fn b() at index 3 is inside cfg mod
        assert!(is_inside_cfg_module(&lines, 3));
        // fn outside() at index 5 is not inside cfg mod
        assert!(!is_inside_cfg_module(&lines, 5));
    }

    #[test]
    fn test_is_inside_cfg_module_no_cfg() {
        let lines: Vec<&str> = vec!["mod normal {", "    fn inside() {}", "}"];
        assert!(!is_inside_cfg_module(&lines, 1));
    }
}
