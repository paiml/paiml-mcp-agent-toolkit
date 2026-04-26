/// Populate cached annotations for all functions during index build.
/// Computes: git churn, code clones, pattern diversity, fault patterns.
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires git + filesystem for churn/clones
#[allow(clippy::cast_possible_truncation)]
pub(super) fn populate_cached_annotations(
    functions: &mut [FunctionEntry],
    file_index: &HashMap<String, Vec<usize>>,
    project_root: &std::path::Path,
) {
    eprintln!("Computing annotations for {} functions...", functions.len());

    // 1. Git churn: get commit counts per file
    let file_commits = get_file_commit_counts(project_root, file_index.keys());
    let max_commits = file_commits.values().copied().max().unwrap_or(1) as f32;
    eprintln!(
        "  Git churn: {} files with commits (max={})",
        file_commits.len(),
        max_commits as u32
    );

    // 2. Detect duplicate/similar functions (by normalized source hash)
    let clone_groups = detect_code_clones(functions);
    eprintln!("  Clones: {} functions with duplicates", clone_groups.len());

    // 3. Compute pattern diversity per file
    let file_diversity = compute_file_pattern_diversity(functions, file_index);
    eprintln!("  Diversity: {} files analyzed", file_diversity.len());

    // 4. Detect fault patterns in source code
    let fault_patterns = detect_fault_patterns(functions);
    eprintln!("  Faults: {} functions with patterns", fault_patterns.len());

    // Apply annotations to functions
    let mut churn_applied = 0;
    let mut clone_applied = 0;
    let mut diversity_applied = 0;
    let mut fault_applied = 0;

    for (i, func) in functions.iter_mut().enumerate() {
        // Churn data
        if let Some(&commits) = file_commits.get(&func.file_path) {
            func.commit_count = commits;
            func.churn_score = commits as f32 / max_commits;
            churn_applied += 1;
        }

        // Clone count
        if let Some(&count) = clone_groups.get(&i) {
            func.clone_count = count;
            clone_applied += 1;
        }

        // Pattern diversity (from file-level)
        if let Some(&diversity) = file_diversity.get(&func.file_path) {
            func.pattern_diversity = diversity;
            diversity_applied += 1;
        }

        // Fault annotations
        if let Some(faults) = fault_patterns.get(&i) {
            func.fault_annotations = faults.clone();
            fault_applied += 1;
        }
    }

    eprintln!(
        "  Applied: churn={}, clones={}, diversity={}, faults={}",
        churn_applied, clone_applied, diversity_applied, fault_applied
    );
}

/// Match a git log file path against the known file set, handling path migrations.
fn match_git_path<'a>(line: &'a str, files: &std::collections::HashSet<&str>) -> Option<&'a str> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Exact match (no allocation — compare &str directly)
    if files.contains(trimmed) {
        return Some(trimmed);
    }
    // Handle path migrations (e.g., server/src/foo.rs -> src/foo.rs)
    let normalized = trimmed.strip_prefix("server/").unwrap_or(trimmed);
    if files.contains(normalized) {
        return Some(normalized);
    }
    None
}

/// Get commit counts per file from git log
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires git process
pub(super) fn get_file_commit_counts<'a>(
    project_root: &std::path::Path,
    files: impl Iterator<Item = &'a String>,
) -> HashMap<String, u32> {
    let files: std::collections::HashSet<&str> = files.map(String::as_str).collect();
    if files.is_empty() {
        return HashMap::new();
    }

    let output = std::process::Command::new("git")
        .args(["log", "--format=", "--name-only", "--since=1 year ago"])
        .current_dir(project_root)
        .output();

    let Ok(output) = output else {
        return HashMap::new();
    };
    if !output.status.success() {
        return HashMap::new();
    }

    let mut result: HashMap<String, u32> = HashMap::with_capacity(files.len());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = match_git_path(line, &files) {
            *result.entry(path.to_string()).or_insert(0) += 1;
        }
    }
    result
}

/// Detect code clones by normalized source hash
#[allow(clippy::cast_possible_truncation)]
pub(super) fn detect_code_clones(functions: &[FunctionEntry]) -> HashMap<usize, u32> {
    let mut result = HashMap::new();
    let mut hash_to_indices: HashMap<u64, Vec<usize>> = HashMap::with_capacity(functions.len());

    for (i, func) in functions.iter().enumerate() {
        // Hash normalized source inline (avoids allocating intermediate String)
        let hash = normalize_source_hash(&func.source);
        hash_to_indices.entry(hash).or_default().push(i);
    }

    // Mark functions that have clones (more than 1 with same hash)
    for indices in hash_to_indices.values() {
        if indices.len() > 1 {
            let count = indices.len() as u32;
            for &idx in indices {
                result.insert(idx, count);
            }
        }
    }

    result
}

/// Normalize source code for clone detection.
/// Hashes inline instead of building an intermediate String (saves ~2 allocs per function).
pub(super) fn normalize_source_hash(source: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    for c in source.chars() {
        if !c.is_whitespace() {
            for lc in c.to_lowercase() {
                lc.hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

/// Compute pattern diversity per file (unique AST patterns / total patterns)
#[allow(clippy::cast_possible_truncation)]
pub(super) fn compute_file_pattern_diversity(
    functions: &[FunctionEntry],
    file_index: &HashMap<String, Vec<usize>>,
) -> HashMap<String, f32> {
    let mut result = HashMap::new();

    for (file_path, indices) in file_index {
        if indices.is_empty() {
            continue;
        }

        // Count unique patterns in file based on function signatures
        let mut patterns = std::collections::HashSet::new();
        for &idx in indices {
            if let Some(func) = functions.get(idx) {
                // Extract pattern: return type + param count + complexity bucket
                let pattern = format!(
                    "{}:{}:{}",
                    extract_return_type(&func.signature),
                    count_params(&func.signature),
                    func.quality.complexity / 5 // bucket by 5
                );
                patterns.insert(pattern);
            }
        }

        let diversity = patterns.len() as f32 / indices.len() as f32;
        result.insert(file_path.clone(), diversity);
    }

    result
}

/// Extract return type from signature (simplified)
pub(super) fn extract_return_type(sig: &str) -> &str {
    if sig.contains("->") {
        sig.split("->").last().unwrap_or("void").trim()
    } else {
        "void"
    }
}

/// Count parameters in signature
pub(super) fn count_params(sig: &str) -> usize {
    if let Some(start) = sig.find('(') {
        // Find matching ')' AFTER '(' to handle C++ comments like "// 1) out = exp(a - val)"
        if let Some(end) = sig[start..].find(')') {
            let params = &sig[start + 1..start + end];
            if params.trim().is_empty() {
                return 0;
            }
            return params.split(',').count();
        }
    }
    0
}

/// Detect fault patterns in function source
pub(super) fn detect_fault_patterns(functions: &[FunctionEntry]) -> HashMap<usize, Vec<String>> {
    let mut result = HashMap::new();

    let patterns = [
        ("unwrap()", "UNWRAP"),
        ("expect(", "EXPECT"),
        ("panic!", "PANIC"),
        ("unsafe {", "UNSAFE"),
        ("unsafe{", "UNSAFE"),
        (".clone()", "CLONE"),
        ("// TODO", "TODO"),
        ("// FIXME", "FIXME"),
        ("// HACK", "HACK"),
        ("// XXX", "XXX"),
        ("unimplemented!", "UNIMPL"),
        ("todo!", "TODO_MACRO"),
        ("unreachable!", "UNREACHABLE"),
        // CUDA/PTX fault patterns
        ("asm volatile", "INLINE_PTX"),
        ("asm(\"", "INLINE_PTX"),
        ("__syncthreads()", "CUDA_SYNC"),
        ("__shared__", "CUDA_SHMEM"),
        // Cross-language boundary patterns (Phase 9)
        ("extern \"C\"", "EXTERN_C"),
        ("__global__", "CUDA_KERNEL"),
        ("__device__", "CUDA_DEVICE"),
    ];

    for (i, func) in functions.iter().enumerate() {
        let mut faults = Vec::new();
        let src = &func.source;

        for (pattern, label) in &patterns {
            if src.contains(pattern) {
                faults.push(label.to_string());
            }
        }

        // Extract inline PTX instruction mnemonics as searchable tags
        extract_ptx_instruction_tags(src, &mut faults);

        // Phase 6: C/C++ macro classification (built-in known patterns)
        classify_cpp_macros(src, &mut faults);

        // Phase 8.4: Inline PTX defect patterns (barrier/shared memory safety)
        detect_inline_ptx_defects(src, &mut faults);

        if !faults.is_empty() {
            faults.sort();
            faults.dedup();
            result.insert(i, faults);
        }
    }

    result
}

/// Extract PTX instruction mnemonics from inline asm() blocks as searchable tags.
///
/// Parses `asm("instruction.modifier ...")` and `asm volatile("...")` blocks
/// to extract PTX opcode mnemonics like `mma.sync`, `cp.async`, `bar.sync`.
fn extract_ptx_instruction_tags(source: &str, faults: &mut Vec<String>) {
    // Known PTX instruction prefixes to search for in asm() string literals
    const PTX_OPCODES: &[&str] = &[
        "mma.sync",
        "ldmatrix",
        "movmatrix",
        "cp.async",
        "bar.sync",
        "bar.arrive",
        "membar",
        "ld.shared",
        "st.shared",
        "ld.global",
        "st.global",
        "atom.shared",
        "red.shared",
        "shfl.sync",
        "vote.sync",
        "match.sync",
    ];
    for opcode in PTX_OPCODES {
        if source.contains(opcode) {
            faults.push(format!("PTX:{opcode}"));
        }
    }
}

/// Phase 6: Classify known C/C++ macro patterns as fault annotations.
///
/// Detects common macro families (assertions, dispatch, logging) used in
/// ML infrastructure codebases (GGML, PyTorch, CUDA) and emits searchable
/// classification tags like `MACRO:ASSERT`, `MACRO:DISPATCH`, `MACRO:LOG`.
pub(super) fn classify_cpp_macros(source: &str, faults: &mut Vec<String>) {
    // Assertion macros — indicate boundary validation
    const ASSERT_MACROS: &[&str] = &[
        "GGML_ASSERT",
        "GGML_ABORT",
        "TORCH_CHECK",
        "TORCH_INTERNAL_ASSERT",
        "AT_ASSERT",
        "CUDA_CHECK",
        "CHECK_CUDA",
        "CUBLAS_CHECK",
    ];
    // Dispatch macros — indicate type-generic dispatch complexity
    const DISPATCH_MACROS: &[&str] = &[
        "AT_DISPATCH_ALL_TYPES",
        "AT_DISPATCH_FLOATING_TYPES",
        "AT_DISPATCH_INTEGRAL_TYPES",
        "AT_DISPATCH_COMPLEX_TYPES",
        "GGML_DISPATCH_BOOL",
        "CUDA_DISPATCH",
    ];
    // Logging macros
    const LOG_MACROS: &[&str] = &[
        "GGML_LOG_INFO",
        "GGML_LOG_WARN",
        "GGML_LOG_ERROR",
        "TORCH_WARN",
        "TORCH_LOG",
    ];

    let has_assert = ASSERT_MACROS.iter().any(|m| source.contains(m));
    let has_dispatch = DISPATCH_MACROS.iter().any(|m| source.contains(m));
    let has_log = LOG_MACROS.iter().any(|m| source.contains(m));

    if has_assert {
        faults.push("MACRO:ASSERT".to_string());
    }
    if has_dispatch {
        faults.push("MACRO:DISPATCH".to_string());
    }
    if has_log {
        faults.push("MACRO:LOG".to_string());
    }
}

/// Phase 8.4: Detect inline PTX defect patterns in CUDA source.
///
/// Lightweight static analysis for PTX safety issues that can be detected
/// from source-level patterns without full PTX parsing. Based on defect
/// classes from `detection_ptx.rs` (GPUVerify SDV semantics).
pub(super) fn detect_inline_ptx_defects(source: &str, faults: &mut Vec<String>) {
    if !source.contains("asm(") && !source.contains("asm volatile") {
        return;
    }

    let has_shared_store = source.contains("st.shared") || source.contains("__shared__");
    let has_shared_load = source.contains("ld.shared");
    let has_barrier = source.contains("bar.sync") || source.contains("__syncthreads");

    if has_shared_store && has_shared_load && !has_barrier {
        faults.push("PTX_MISSING_BARRIER".to_string());
    }

    if has_barrier {
        detect_ptx_barrier_divergence(source, faults);
        detect_ptx_early_exit(source, faults);
    }

    detect_ptx_register_issues(source, faults);
    detect_ptx_shared_u64(source, faults);
    detect_ptx_local_spills(source, faults);
    detect_ptx_pred_overflow(source, faults);
    detect_ptx_empty_loop(source, faults);
    detect_ptx_redundant_mov(source, faults);
}

fn detect_ptx_barrier_divergence(source: &str, faults: &mut Vec<String>) {
    let mut in_branch = false;
    for line in source.lines() {
        let t = line.trim();
        if t.starts_with("if ") || t.starts_with("if(") || t.contains("@!%p") || t.contains("@%p") {
            in_branch = true;
        }
        if in_branch && (t.contains("bar.sync") || t.contains("__syncthreads")) {
            faults.push("PTX_BARRIER_DIV".to_string());
            return;
        }
        if t == "}" || t.starts_with("else") {
            in_branch = false;
        }
    }
}

fn detect_ptx_early_exit(source: &str, faults: &mut Vec<String>) {
    let mut seen_return = false;
    for line in source.lines() {
        let t = line.trim();
        if t.starts_with("return") && t.contains(';') {
            seen_return = true;
        }
        if seen_return && (t.contains("bar.sync") || t.contains("__syncthreads")) {
            faults.push("PTX_EARLY_EXIT".to_string());
            return;
        }
    }
}

fn detect_ptx_register_issues(source: &str, faults: &mut Vec<String>) {
    let reg_count = source.matches("\"=r\"").count() + source.matches("\"+r\"").count();
    if reg_count > 8 {
        faults.push("PTX_HIGH_REGS".to_string());
    }
}

fn detect_ptx_shared_u64(source: &str, faults: &mut Vec<String>) {
    let has_shared = source.contains("st.shared") || source.contains("ld.shared");
    if has_shared && (source.contains("cvta.shared") || source.contains("cvta.to.shared")) {
        faults.push("PTX_SHARED_U64".to_string());
    }
}

fn detect_ptx_local_spills(source: &str, faults: &mut Vec<String>) {
    if source.contains(".local") && (source.contains("st.local") || source.contains("ld.local")) {
        faults.push("PTX_REG_SPILL".to_string());
    }
}

fn detect_ptx_pred_overflow(source: &str, faults: &mut Vec<String>) {
    let pred_count = (0..16)
        .filter(|i| source.contains(&format!("%p{i}")))
        .count();
    if pred_count > 8 {
        faults.push("PTX_PRED_OVERFLOW".to_string());
    }
}

fn detect_ptx_empty_loop(source: &str, faults: &mut Vec<String>) {
    if !source.contains("__global__") && !source.contains("__device__") {
        return;
    }
    // Use peekable iterator instead of collecting all lines into a Vec
    let mut lines = source.lines().peekable();
    while let Some(line) = lines.next() {
        let t = line.trim();
        if t.starts_with("for") || t.starts_with("while") {
            if let Some(&next) = lines.peek() {
                let next = next.trim();
                if next == "{}" || next == "{ }" || next == ";" {
                    faults.push("PTX_EMPTY_LOOP".to_string());
                    return;
                }
            }
        }
    }
}

fn detect_ptx_redundant_mov(source: &str, faults: &mut Vec<String>) {
    for line in source.lines() {
        let t = line.trim();
        if !t.contains("mov.") {
            continue;
        }
        let Some(args) = t.split_whitespace().nth(1) else {
            continue;
        };
        let parts: Vec<&str> = args.split(',').map(str::trim).collect();
        if parts.len() == 2 {
            let dest = parts[0].trim_end_matches(';');
            let src = parts[1].trim_end_matches(';');
            if dest == src && dest.starts_with('%') {
                faults.push("PTX_REDUNDANT_MOV".to_string());
                return;
            }
        }
    }
}

/// Link C/C++ declarations (prototypes in headers) to their definitions (implementations).
///
/// For each function with a `[decl]` suffix in its name, find the corresponding
/// definition (same name without `[decl]`) and set `linked_definition` to point
/// to the definition's location. Also marks the declaration's `definition_type`
/// as `Declaration`.
pub(super) fn link_declarations_to_definitions(functions: &mut [FunctionEntry]) {
    // Build index: bare_name → Vec<(index, file_path, start_line)> for definitions
    let mut def_index: HashMap<String, Vec<(usize, String, usize)>> = HashMap::new();
    for (i, func) in functions.iter().enumerate() {
        if !func.function_name.ends_with(" [decl]") {
            def_index
                .entry(func.function_name.clone())
                .or_default()
                .push((i, func.file_path.clone(), func.start_line));
        }
    }

    // Link declarations to definitions
    let mut linked = 0;
    for func in functions.iter_mut() {
        if func.function_name.ends_with(" [decl]") {
            let bare_name = func.function_name.trim_end_matches(" [decl]");
            func.definition_type = DefinitionType::Declaration;
            if let Some(defs) = def_index.get(bare_name) {
                // Pick the definition in a different file (header→impl linking)
                let best = defs
                    .iter()
                    .find(|(_, path, _)| path != &func.file_path)
                    .or_else(|| defs.first());
                if let Some((_, path, line)) = best {
                    func.linked_definition = Some(format!("{path}:{line}"));
                    linked += 1;
                }
            }
        }
    }

    if linked > 0 {
        eprintln!("  Decl-def links: {linked} declarations linked to definitions");
    }
}

/// Compute name frequency for generic name demotion.
///
/// Returns a map of function_name -> fraction of total functions with that name.
/// High-frequency names like `new`, `default`, `from` get demoted in search results.
#[allow(clippy::cast_possible_truncation)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub(crate) fn compute_name_frequency(
    name_index: &HashMap<String, Vec<usize>>,
    total: usize,
) -> HashMap<String, f32> {
    if total == 0 {
        return HashMap::new();
    }
    let mut result = HashMap::with_capacity(name_index.len());
    for (name, indices) in name_index {
        result.insert(name.clone(), indices.len() as f32 / total as f32);
    }
    result
}

#[cfg(test)]
mod annotations_tests {
    use super::*;
    use std::collections::HashSet;

    // ── match_git_path ──────────────────────────────────────────────────────

    fn files_set<'a>(paths: &[&'a str]) -> HashSet<&'a str> {
        paths.iter().copied().collect()
    }

    #[test]
    fn test_match_git_path_exact_match() {
        let files = files_set(&["src/lib.rs", "src/main.rs"]);
        assert_eq!(match_git_path("src/lib.rs", &files), Some("src/lib.rs"));
    }

    #[test]
    fn test_match_git_path_strips_server_prefix() {
        let files = files_set(&["src/foo.rs"]);
        // Path migration: server/src/foo.rs → src/foo.rs
        assert_eq!(
            match_git_path("server/src/foo.rs", &files),
            Some("src/foo.rs")
        );
    }

    #[test]
    fn test_match_git_path_no_match_returns_none() {
        let files = files_set(&["src/foo.rs"]);
        assert!(match_git_path("docs/readme.md", &files).is_none());
    }

    #[test]
    fn test_match_git_path_empty_line_returns_none() {
        let files = files_set(&["src/foo.rs"]);
        assert!(match_git_path("", &files).is_none());
        assert!(match_git_path("   ", &files).is_none());
    }

    #[test]
    fn test_match_git_path_trims_input() {
        let files = files_set(&["src/lib.rs"]);
        assert_eq!(match_git_path("  src/lib.rs  ", &files), Some("src/lib.rs"));
    }

    // ── extract_return_type ─────────────────────────────────────────────────

    #[test]
    fn test_extract_return_type_with_arrow() {
        assert_eq!(extract_return_type("fn foo() -> i32"), "i32");
        assert_eq!(extract_return_type("fn bar() -> Result<()>"), "Result<()>");
    }

    #[test]
    fn test_extract_return_type_no_arrow_returns_void() {
        assert_eq!(extract_return_type("fn foo()"), "void");
        assert_eq!(extract_return_type(""), "void");
    }

    #[test]
    fn test_extract_return_type_trims_whitespace() {
        assert_eq!(extract_return_type("fn f() ->   String   "), "String");
    }

    // ── count_params ────────────────────────────────────────────────────────

    #[test]
    fn test_count_params_empty_parens() {
        assert_eq!(count_params("fn foo()"), 0);
        assert_eq!(count_params("fn bar(  )"), 0);
    }

    #[test]
    fn test_count_params_single_param() {
        assert_eq!(count_params("fn foo(x: u32)"), 1);
    }

    #[test]
    fn test_count_params_multiple() {
        assert_eq!(count_params("fn foo(a: u32, b: u32, c: u32)"), 3);
    }

    #[test]
    fn test_count_params_no_paren_returns_zero() {
        assert_eq!(count_params("not a signature"), 0);
    }

    #[test]
    fn test_count_params_handles_close_paren_in_comment() {
        // Pinned: function finds `)` AFTER `(`, so a `// ...)` before `(` is OK
        assert_eq!(count_params("// 1) note\nfn foo(x: u32, y: u32)"), 2);
    }

    // ── normalize_source_hash ───────────────────────────────────────────────

    #[test]
    fn test_normalize_source_hash_whitespace_invariant() {
        let h1 = normalize_source_hash("fn foo() { bar(); }");
        let h2 = normalize_source_hash("fn  foo()   {\n  bar();\n}");
        assert_eq!(h1, h2, "whitespace differences should not affect hash");
    }

    #[test]
    fn test_normalize_source_hash_case_invariant() {
        let h1 = normalize_source_hash("FN FOO");
        let h2 = normalize_source_hash("fn foo");
        assert_eq!(h1, h2, "case differences should not affect hash");
    }

    #[test]
    fn test_normalize_source_hash_different_content_differs() {
        let h1 = normalize_source_hash("fn foo() {}");
        let h2 = normalize_source_hash("fn bar() {}");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_normalize_source_hash_empty() {
        // Empty input still produces a hash
        let _h = normalize_source_hash("");
    }

    // ── extract_ptx_instruction_tags ────────────────────────────────────────

    #[test]
    fn test_extract_ptx_instruction_tags_finds_known_opcodes() {
        let mut faults = Vec::new();
        extract_ptx_instruction_tags("asm(\"mma.sync.aligned.m16n8k16 ...\")", &mut faults);
        assert!(faults.contains(&"PTX:mma.sync".to_string()));
    }

    #[test]
    fn test_extract_ptx_instruction_tags_multiple_opcodes() {
        let mut faults = Vec::new();
        let src = "asm volatile(\"cp.async ...\");\nasm(\"bar.sync 0\");";
        extract_ptx_instruction_tags(src, &mut faults);
        assert!(faults.contains(&"PTX:cp.async".to_string()));
        assert!(faults.contains(&"PTX:bar.sync".to_string()));
    }

    #[test]
    fn test_extract_ptx_instruction_tags_no_match_no_push() {
        let mut faults = Vec::new();
        extract_ptx_instruction_tags("regular C++ code", &mut faults);
        assert!(faults.is_empty());
    }

    // ── classify_cpp_macros ─────────────────────────────────────────────────

    #[test]
    fn test_classify_cpp_macros_assert_family() {
        let mut faults = Vec::new();
        classify_cpp_macros("GGML_ASSERT(x > 0);", &mut faults);
        assert_eq!(faults, vec!["MACRO:ASSERT"]);
    }

    #[test]
    fn test_classify_cpp_macros_dispatch_family() {
        let mut faults = Vec::new();
        classify_cpp_macros("AT_DISPATCH_ALL_TYPES(x, ...)", &mut faults);
        assert_eq!(faults, vec!["MACRO:DISPATCH"]);
    }

    #[test]
    fn test_classify_cpp_macros_log_family() {
        let mut faults = Vec::new();
        classify_cpp_macros("GGML_LOG_INFO(\"hello\")", &mut faults);
        assert_eq!(faults, vec!["MACRO:LOG"]);
    }

    #[test]
    fn test_classify_cpp_macros_torch_check_is_assert() {
        let mut faults = Vec::new();
        classify_cpp_macros("TORCH_CHECK(cond);", &mut faults);
        assert!(faults.contains(&"MACRO:ASSERT".to_string()));
    }

    #[test]
    fn test_classify_cpp_macros_combination() {
        let mut faults = Vec::new();
        classify_cpp_macros("GGML_ASSERT(x); GGML_LOG_WARN(\"y\");", &mut faults);
        assert_eq!(faults.len(), 2);
        assert!(faults.contains(&"MACRO:ASSERT".to_string()));
        assert!(faults.contains(&"MACRO:LOG".to_string()));
    }

    #[test]
    fn test_classify_cpp_macros_no_macros() {
        let mut faults = Vec::new();
        classify_cpp_macros("int x = 1;", &mut faults);
        assert!(faults.is_empty());
    }

    // ── detect_inline_ptx_defects (orchestrator) ────────────────────────────

    #[test]
    fn test_detect_inline_ptx_defects_no_asm_returns_immediately() {
        let mut faults = Vec::new();
        detect_inline_ptx_defects("regular C++", &mut faults);
        assert!(faults.is_empty());
    }

    #[test]
    fn test_detect_inline_ptx_defects_shared_store_load_no_barrier() {
        let mut faults = Vec::new();
        let src = "asm(\"st.shared ...\"); asm(\"ld.shared ...\");";
        detect_inline_ptx_defects(src, &mut faults);
        assert!(faults.contains(&"PTX_MISSING_BARRIER".to_string()));
    }

    #[test]
    fn test_detect_inline_ptx_defects_with_barrier_no_missing() {
        let mut faults = Vec::new();
        let src = "asm(\"st.shared ...\"); asm(\"ld.shared ...\"); asm(\"bar.sync 0\");";
        detect_inline_ptx_defects(src, &mut faults);
        assert!(!faults.contains(&"PTX_MISSING_BARRIER".to_string()));
    }

    // ── detect_ptx_barrier_divergence ───────────────────────────────────────

    #[test]
    fn test_detect_ptx_barrier_divergence_in_if_branch() {
        let mut faults = Vec::new();
        let src = "if (cond) {\nbar.sync 0;\n}\n";
        detect_ptx_barrier_divergence(src, &mut faults);
        assert!(faults.contains(&"PTX_BARRIER_DIV".to_string()));
    }

    #[test]
    fn test_detect_ptx_barrier_divergence_outside_branch_clean() {
        let mut faults = Vec::new();
        let src = "bar.sync 0;\nif (cond) { foo; }\n";
        detect_ptx_barrier_divergence(src, &mut faults);
        assert!(faults.is_empty());
    }

    #[test]
    fn test_detect_ptx_barrier_divergence_predicate_branch() {
        let mut faults = Vec::new();
        let src = "@%p1 bar.sync 0;\n";
        detect_ptx_barrier_divergence(src, &mut faults);
        assert!(faults.contains(&"PTX_BARRIER_DIV".to_string()));
    }

    // ── detect_ptx_early_exit ───────────────────────────────────────────────

    #[test]
    fn test_detect_ptx_early_exit_return_then_barrier() {
        // PIN: trigger requires a *trimmed line that STARTS WITH* "return" (and contains ';').
        // `if (x) return;` does NOT trigger because the line doesn't start with "return".
        let mut faults = Vec::new();
        let src = "return;\nbar.sync 0;\n";
        detect_ptx_early_exit(src, &mut faults);
        assert!(faults.contains(&"PTX_EARLY_EXIT".to_string()));
    }

    #[test]
    fn test_detect_ptx_early_exit_inline_return_does_not_trigger() {
        // PIN: starts_with check means inline returns (e.g. `if (x) return;`) are NOT flagged.
        let mut faults = Vec::new();
        let src = "if (x) return;\nbar.sync 0;\n";
        detect_ptx_early_exit(src, &mut faults);
        assert!(faults.is_empty());
    }

    #[test]
    fn test_detect_ptx_early_exit_no_return_clean() {
        let mut faults = Vec::new();
        let src = "bar.sync 0;\nfoo();\n";
        detect_ptx_early_exit(src, &mut faults);
        assert!(faults.is_empty());
    }

    // ── detect_ptx_register_issues ──────────────────────────────────────────

    #[test]
    fn test_detect_ptx_register_issues_above_threshold() {
        let mut faults = Vec::new();
        // 9 register-output operands → > 8
        let src = "\"=r\" \"=r\" \"=r\" \"=r\" \"=r\" \"=r\" \"=r\" \"=r\" \"=r\"";
        detect_ptx_register_issues(src, &mut faults);
        assert!(faults.contains(&"PTX_HIGH_REGS".to_string()));
    }

    #[test]
    fn test_detect_ptx_register_issues_below_threshold_clean() {
        let mut faults = Vec::new();
        let src = "\"=r\" \"=r\" \"=r\""; // 3 < 8
        detect_ptx_register_issues(src, &mut faults);
        assert!(faults.is_empty());
    }

    #[test]
    fn test_detect_ptx_register_issues_includes_plus_r() {
        let mut faults = Vec::new();
        let src = "\"=r\" \"=r\" \"=r\" \"=r\" \"+r\" \"+r\" \"+r\" \"+r\" \"+r\"";
        detect_ptx_register_issues(src, &mut faults);
        assert!(faults.contains(&"PTX_HIGH_REGS".to_string()));
    }

    // ── detect_ptx_shared_u64 ───────────────────────────────────────────────

    #[test]
    fn test_detect_ptx_shared_u64_with_cvta() {
        let mut faults = Vec::new();
        let src = "st.shared.u64 [...]; cvta.shared ...";
        detect_ptx_shared_u64(src, &mut faults);
        assert!(faults.contains(&"PTX_SHARED_U64".to_string()));
    }

    #[test]
    fn test_detect_ptx_shared_u64_no_shared_clean() {
        let mut faults = Vec::new();
        detect_ptx_shared_u64("cvta.shared without st/ld", &mut faults);
        assert!(faults.is_empty());
    }

    // ── detect_ptx_local_spills ─────────────────────────────────────────────

    #[test]
    fn test_detect_ptx_local_spills_with_st_local() {
        let mut faults = Vec::new();
        detect_ptx_local_spills(".local .u32 spill;\nst.local.u32 ...", &mut faults);
        assert!(faults.contains(&"PTX_REG_SPILL".to_string()));
    }

    #[test]
    fn test_detect_ptx_local_spills_no_local_clean() {
        let mut faults = Vec::new();
        detect_ptx_local_spills("just regular ptx", &mut faults);
        assert!(faults.is_empty());
    }

    // ── detect_ptx_pred_overflow ────────────────────────────────────────────

    #[test]
    fn test_detect_ptx_pred_overflow_above_8() {
        let mut faults = Vec::new();
        let src = "%p0 %p1 %p2 %p3 %p4 %p5 %p6 %p7 %p8";
        detect_ptx_pred_overflow(src, &mut faults);
        assert!(faults.contains(&"PTX_PRED_OVERFLOW".to_string()));
    }

    #[test]
    fn test_detect_ptx_pred_overflow_below_threshold_clean() {
        let mut faults = Vec::new();
        let src = "%p0 %p1 %p2";
        detect_ptx_pred_overflow(src, &mut faults);
        assert!(faults.is_empty());
    }

    // ── detect_ptx_empty_loop ───────────────────────────────────────────────

    #[test]
    fn test_detect_ptx_empty_loop_global_with_empty_for() {
        let mut faults = Vec::new();
        let src = "__global__ void k() {\nfor (...)\n{}\n}";
        detect_ptx_empty_loop(src, &mut faults);
        assert!(faults.contains(&"PTX_EMPTY_LOOP".to_string()));
    }

    #[test]
    fn test_detect_ptx_empty_loop_no_global_clean() {
        let mut faults = Vec::new();
        let src = "void k() {\nfor (...)\n{}\n}"; // not __global__
        detect_ptx_empty_loop(src, &mut faults);
        assert!(faults.is_empty());
    }

    #[test]
    fn test_detect_ptx_empty_loop_with_body_clean() {
        let mut faults = Vec::new();
        let src = "__global__ void k() {\nfor (...)\nfoo();\n}";
        detect_ptx_empty_loop(src, &mut faults);
        assert!(faults.is_empty());
    }

    // ── detect_ptx_redundant_mov ────────────────────────────────────────────

    #[test]
    fn test_detect_ptx_redundant_mov_same_reg() {
        // PIN: parser uses `split_whitespace().nth(1)` so dest+src must be in ONE
        // whitespace-token (no space after comma) — `"%r1,%r1"`. With a space after
        // the comma, args = "%r1," and src parses as empty, so no detection.
        let mut faults = Vec::new();
        let src = "mov.u32 %r1,%r1;";
        detect_ptx_redundant_mov(src, &mut faults);
        assert!(faults.contains(&"PTX_REDUNDANT_MOV".to_string()));
    }

    #[test]
    fn test_detect_ptx_redundant_mov_space_after_comma_does_not_trigger() {
        // PIN: `"%r1, %r1"` (with space) is missed — split_whitespace tokenization
        // grabs only `"%r1,"`, so the duplicate-register check never sees src.
        let mut faults = Vec::new();
        detect_ptx_redundant_mov("mov.u32 %r1, %r1;", &mut faults);
        assert!(faults.is_empty());
    }

    #[test]
    fn test_detect_ptx_redundant_mov_different_reg_clean() {
        let mut faults = Vec::new();
        detect_ptx_redundant_mov("mov.u32 %r1,%r2;", &mut faults);
        assert!(faults.is_empty());
    }

    // ── compute_name_frequency ──────────────────────────────────────────────

    #[test]
    fn test_compute_name_frequency_zero_total_returns_empty() {
        let mut idx = HashMap::new();
        idx.insert("foo".to_string(), vec![0usize, 1]);
        let result = compute_name_frequency(&idx, 0);
        assert!(result.is_empty());
    }

    #[test]
    fn test_compute_name_frequency_basic() {
        let mut idx = HashMap::new();
        idx.insert("new".to_string(), vec![0, 1, 2]);
        idx.insert("rare_name".to_string(), vec![3]);
        let result = compute_name_frequency(&idx, 4);
        assert!((result["new"] - 0.75).abs() < 1e-6);
        assert!((result["rare_name"] - 0.25).abs() < 1e-6);
    }
}
// #[requires(project_path.exists())]
// #[ensures(result.is_ok())]
