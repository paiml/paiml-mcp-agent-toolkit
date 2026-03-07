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
fn match_git_path(line: &str, files: &std::collections::HashSet<&String>) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Exact match
    if files.contains(&trimmed.to_string()) {
        return Some(trimmed.to_string());
    }
    // Handle path migrations (e.g., server/src/foo.rs -> src/foo.rs)
    let normalized = trimmed.strip_prefix("server/").unwrap_or(trimmed);
    if files.contains(&normalized.to_string()) {
        return Some(normalized.to_string());
    }
    None
}

/// Get commit counts per file from git log
#[cfg_attr(coverage_nightly, coverage(off))] // Integration: requires git process
pub(super) fn get_file_commit_counts<'a>(
    project_root: &std::path::Path,
    files: impl Iterator<Item = &'a String>,
) -> HashMap<String, u32> {
    let files: std::collections::HashSet<_> = files.collect();
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

    let mut result = HashMap::new();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = match_git_path(line, &files) {
            *result.entry(path).or_insert(0) += 1;
        }
    }
    result
}

/// Detect code clones by normalized source hash
#[allow(clippy::cast_possible_truncation)]
pub(super) fn detect_code_clones(functions: &[FunctionEntry]) -> HashMap<usize, u32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut result = HashMap::new();
    let mut hash_to_indices: HashMap<u64, Vec<usize>> = HashMap::new();

    for (i, func) in functions.iter().enumerate() {
        // Normalize source: remove whitespace, lowercase identifiers
        let normalized = normalize_source(&func.source);

        let mut hasher = DefaultHasher::new();
        normalized.hash(&mut hasher);
        let hash = hasher.finish();

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

/// Normalize source code for clone detection
pub(super) fn normalize_source(source: &str) -> String {
    source
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
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
        "GGML_ASSERT", "GGML_ABORT", "TORCH_CHECK", "TORCH_INTERNAL_ASSERT",
        "AT_ASSERT", "CUDA_CHECK", "CHECK_CUDA", "CUBLAS_CHECK",
    ];
    // Dispatch macros — indicate type-generic dispatch complexity
    const DISPATCH_MACROS: &[&str] = &[
        "AT_DISPATCH_ALL_TYPES", "AT_DISPATCH_FLOATING_TYPES",
        "AT_DISPATCH_INTEGRAL_TYPES", "AT_DISPATCH_COMPLEX_TYPES",
        "GGML_DISPATCH_BOOL", "CUDA_DISPATCH",
    ];
    // Logging macros
    const LOG_MACROS: &[&str] = &[
        "GGML_LOG_INFO", "GGML_LOG_WARN", "GGML_LOG_ERROR",
        "TORCH_WARN", "TORCH_LOG",
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
    // Only analyze functions containing inline PTX
    if !source.contains("asm(") && !source.contains("asm volatile") {
        return;
    }

    // PTX_MISSING_BARRIER: st.shared followed by ld.shared without bar.sync
    // Simplified heuristic: shared memory write + read with no barrier between
    let has_shared_store = source.contains("st.shared") || source.contains("__shared__");
    let has_shared_load = source.contains("ld.shared");
    let has_barrier = source.contains("bar.sync") || source.contains("__syncthreads");
    if has_shared_store && has_shared_load && !has_barrier {
        faults.push("PTX_MISSING_BARRIER".to_string());
    }

    // PTX_BARRIER_DIV: Branch/if before bar.sync (thread divergence deadlock risk)
    // Look for pattern: if/branch followed by barrier in asm block
    if has_barrier {
        let lines: Vec<&str> = source.lines().collect();
        let mut in_branch = false;
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("if ") || trimmed.starts_with("if(")
                || trimmed.contains("@!%p") || trimmed.contains("@%p") {
                in_branch = true;
            }
            if in_branch && (trimmed.contains("bar.sync") || trimmed.contains("__syncthreads")) {
                faults.push("PTX_BARRIER_DIV".to_string());
                break;
            }
            if trimmed == "}" || trimmed.starts_with("else") {
                in_branch = false;
            }
        }
    }

    // PTX_REG_SPILL: High register usage indicator
    // Look for many register declarations in inline PTX
    let reg_count = source.matches("\"=r\"").count() + source.matches("\"+r\"").count();
    if reg_count > 8 {
        faults.push("PTX_HIGH_REGS".to_string());
    }

    // PTX_SHARED_U64: 64-bit register for shared memory address (should be 32-bit)
    // Pattern: shared memory accessed via 64-bit register ("l" constraint with shared)
    if source.contains("st.shared") || source.contains("ld.shared") {
        // Check for 64-bit addressing of shared memory (cvta.shared or "l" constraint)
        if source.contains("cvta.shared") || source.contains("cvta.to.shared") {
            faults.push("PTX_SHARED_U64".to_string());
        }
    }

    // PTX_EARLY_EXIT: Thread exits before reaching barrier (PARITY-114)
    // Pattern: return/ret before __syncthreads or bar.sync
    if has_barrier {
        let lines: Vec<&str> = source.lines().collect();
        let mut seen_early_return = false;
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("return") && trimmed.contains(';') {
                seen_early_return = true;
            }
            if seen_early_return && (trimmed.contains("bar.sync") || trimmed.contains("__syncthreads")) {
                faults.push("PTX_EARLY_EXIT".to_string());
                break;
            }
        }
    }

    // PTX_REG_SPILL: Register spills to local memory
    // Pattern: .local declarations in inline PTX indicate spills
    if source.contains(".local") && (source.contains("st.local") || source.contains("ld.local")) {
        faults.push("PTX_REG_SPILL".to_string());
    }

    // PTX_PRED_OVERFLOW: >8 predicate registers (hardware limit)
    // Pattern: excessive predicate register declarations (%p0..%p9+)
    let pred_count = (0..16).filter(|i| source.contains(&format!("%p{i}"))).count();
    if pred_count > 8 {
        faults.push("PTX_PRED_OVERFLOW".to_string());
    }

    // PTX_EMPTY_LOOP: Loop body with no computation
    // Pattern: for/while loop with only barrier or empty body in CUDA context
    if source.contains("__global__") || source.contains("__device__") {
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if (trimmed.starts_with("for") || trimmed.starts_with("while"))
                && i + 1 < lines.len()
            {
                let next = lines[i + 1].trim();
                if next == "{}" || next == "{ }" || next == ";" {
                    faults.push("PTX_EMPTY_LOOP".to_string());
                    break;
                }
            }
        }
    }

    // PTX_REDUNDANT_MOV: Redundant register move chains
    // Pattern: mov instruction where source == destination register
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.contains("mov.") {
            // Parse: mov.type %dest, %src; — detect %rN, %rN patterns
            if let Some(args) = trimmed.split_whitespace().nth(1) {
                let parts: Vec<&str> = args.split(',').map(str::trim).collect();
                if parts.len() == 2 {
                    let dest = parts[0].trim_end_matches(';');
                    let src = parts[1].trim_end_matches(';');
                    if dest == src && dest.starts_with('%') {
                        faults.push("PTX_REDUNDANT_MOV".to_string());
                        break;
                    }
                }
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
pub(crate) fn compute_name_frequency(
    name_index: &HashMap<String, Vec<usize>>,
    total: usize,
) -> HashMap<String, f32> {
    if total == 0 {
        return HashMap::new();
    }
    name_index
        .iter()
        .map(|(name, indices)| (name.clone(), indices.len() as f32 / total as f32))
        .collect()
}
