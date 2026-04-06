
// =============================================================================
// CB-060: GPU QUALITY DETECTION
// =============================================================================
//
// Per the "Danger Zone" heuristic: We look for patterns that indicate potential
// GPU correctness issues, accepting some false positives in exchange for high
// recall. Target: >90% precision.

/// Pattern to detect PTX branch instructions (predicated jumps)
static PTX_BRANCH_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@%p\d+\s+bra\s+\w+").expect("valid regex"));

/// Pattern to detect PTX barrier sync
static PTX_BARRIER_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"bar\.sync\s+\d+").expect("valid regex"));

/// Pattern to detect PTX shared memory load (destination, then source)
static PTX_SHARED_LOAD_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"ld\.shared\.\w+\s+[^,]+,\s*\[([^\]]+)\]").expect("valid regex"));

/// Pattern to detect PTX shared memory store (address first, then source)
static PTX_SHARED_STORE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"st\.shared\.\w+\s+\[([^\]]+)\],").expect("valid regex"));

/// Pattern to detect PTX predicated shared memory access (safe)
static PTX_PREDICATED_SHARED_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@%p\d+\s+(ld|st)\.shared").expect("valid regex"));

/// Pattern to detect PTX bounds check (setp.lt)
static PTX_BOUNDS_CHECK_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"setp\.lt\.\w+\s+%p\d+").expect("valid regex"));

/// Pattern to detect constant offset shared access (safe)
static PTX_CONSTANT_OFFSET_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(ld|st)\.shared\.\w+\s+[^,]+,\s*\[\w+\s*\+\s*\d+\]").expect("valid regex")
});

/// Pattern to detect WGSL workgroup barrier
static WGSL_BARRIER_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"workgroupBarrier\s*\(\s*\)").expect("valid regex"));

/// Pattern to detect WGSL if statement start
static WGSL_IF_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bif\s*\([^)]+\)\s*\{").expect("valid regex"));

/// Pattern to detect WGSL else block
static WGSL_ELSE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\}\s*else\s*\{").expect("valid regex"));

/// Pattern to detect WGSL for loop with thread-dependent bounds (divergent)
static WGSL_DIVERGENT_LOOP_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"for\s*\([^)]*<\s*(?:local_id|global_id)\.\w+[^)]*\)").expect("valid regex")
});

/// Pattern to detect matrix store without bounds (tiled kernel)
static TILED_STORE_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\bc\s*\[\s*row\s*\*\s*n\s*\+\s*col\s*\]").expect("valid regex"));

/// Pattern to detect proper bounds check (row < m && col < n)
static TILED_BOUNDS_CHECK_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:row|global_id\.y)\s*<\s*(?:m|\w+).*(?:col|global_id\.x)\s*<\s*(?:n|\w+)")
        .expect("valid regex")
});

/// Pattern to detect complex but valid bounds expressions
static TILED_COMPLEX_BOUNDS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\([^)]*row[^)]*\)\s*<\s*\([^)]*m[^)]*\).*col\s*<\s*n").expect("valid regex")
});

/// Pattern to detect PTX early exit pattern before tile loop
static PTX_EARLY_EXIT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"@%p\d+\s+bra\s+exit[\s\S]{0,200}(?:tile|loop|ld\.shared)").expect("valid regex")
});

/// Pattern to detect WGSL tiled kernel store
static WGSL_TILED_STORE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\ba\s*\[\s*(?:global_id|local_id)\.\w+\s*\*").expect("valid regex")
});

/// Detect PTX barrier divergence patterns
/// Returns list of (line_number, pattern_id, description)
///
/// The "Danger Zone" heuristic: A branch before a barrier is dangerous because
/// some threads may exit early, causing the remaining threads to deadlock on
/// the barrier.
pub fn detect_ptx_barrier_divergence_in_str(ptx: &str) -> Vec<(u32, &'static str, String)> {
    debug_assert!(!ptx.is_empty(), "ptx must not be empty");
    let mut violations = Vec::new();
    let lines: Vec<&str> = ptx.lines().collect();

    // First pass: find all branch and barrier locations
    let mut branch_locations: Vec<(usize, &str)> = Vec::new();
    let mut barrier_locations: Vec<usize> = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        // Skip comments
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // Track branch locations
        if let Some(m) = PTX_BRANCH_PATTERN.find(line) {
            branch_locations.push((line_idx, m.as_str()));
        }

        // Track barrier locations
        if PTX_BARRIER_PATTERN.is_match(line) {
            barrier_locations.push(line_idx);
        }
    }

    // "Danger Zone" detection: branch BEFORE barrier (within reasonable distance)
    for (branch_line, branch_text) in &branch_locations {
        for &barrier_line in &barrier_locations {
            // Branch must be BEFORE barrier (danger zone)
            if *branch_line < barrier_line {
                // Check distance - within 20 lines is suspicious
                let distance = barrier_line - *branch_line;
                if distance <= 20 {
                    violations.push((
                        (*branch_line + 1) as u32,
                        "CB-060-A",
                        format!(
                            "Thread divergence before barrier: {} (barrier {} lines later)",
                            branch_text, distance
                        ),
                    ));
                }
            }
        }
    }

    violations
}

/// Detect WGSL barrier divergence patterns
///
/// WGSL workgroupBarrier() inside control flow (if/else) is dangerous because
/// not all threads in the workgroup may execute the barrier.
pub fn detect_wgsl_barrier_divergence_in_str(wgsl: &str) -> Vec<(u32, &'static str, String)> {
    debug_assert!(!wgsl.is_empty(), "wgsl must not be empty");
    let mut violations = Vec::new();
    let lines: Vec<&str> = wgsl.lines().collect();

    // Track control flow state
    let mut if_depth: usize = 0;
    let mut in_divergent_loop = false;
    let mut divergent_loop_depth: usize = 0;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = (line_idx + 1) as u32;
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // Track divergent loops (for with thread-dependent bounds)
        if WGSL_DIVERGENT_LOOP_PATTERN.is_match(line) {
            in_divergent_loop = true;
            divergent_loop_depth = 1 + line
                .matches('{')
                .count()
                .saturating_sub(line.matches('}').count());
        }

        // Track brace depth for divergent loop
        if in_divergent_loop {
            divergent_loop_depth += line.matches('{').count();
            divergent_loop_depth = divergent_loop_depth.saturating_sub(line.matches('}').count());
            if divergent_loop_depth == 0 {
                in_divergent_loop = false;
            }
        }

        // Track if/else depth
        if WGSL_IF_PATTERN.is_match(line) {
            if_depth += 1;
        }
        if WGSL_ELSE_PATTERN.is_match(line) {
            // else maintains if_depth
        }
        // Count closing braces to track depth
        if if_depth > 0 {
            // Simplified: decrement on closing brace
            // This is imprecise but avoids complex parsing
            let opens = line.matches('{').count();
            let closes = line.matches('}').count();
            if closes > opens {
                if_depth = if_depth.saturating_sub(closes - opens);
            }
        }

        // Check for barrier in dangerous context
        if WGSL_BARRIER_PATTERN.is_match(line) {
            if if_depth > 0 {
                violations.push((
                    line_num,
                    "CB-060-D",
                    "workgroupBarrier() in divergent control flow (if/else)".to_string(),
                ));
            } else if in_divergent_loop {
                violations.push((
                    line_num,
                    "CB-060-D",
                    "workgroupBarrier() in divergent loop (thread-dependent bounds)".to_string(),
                ));
            }
        }
    }

    violations
}

/// Detect unbounded shared memory access in PTX
///
/// Shared memory accesses without bounds checks can cause out-of-bounds errors.
/// Safe patterns: predicated access (@%p), constant offset, or preceding setp.lt.
pub fn detect_shared_memory_unbounded_in_str(ptx: &str) -> Vec<(u32, &'static str, String)> {
    debug_assert!(!ptx.is_empty(), "ptx must not be empty");
    let mut violations = Vec::new();
    let lines: Vec<&str> = ptx.lines().collect();

    // Track bounds check coverage (lines covered by a preceding setp.lt)
    let mut bounds_check_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();

    // First pass: identify bounds checks and their coverage
    for (line_idx, line) in lines.iter().enumerate() {
        if PTX_BOUNDS_CHECK_PATTERN.is_match(line) {
            // A bounds check covers the next ~10 lines (heuristic)
            for i in 0..=10 {
                bounds_check_lines.insert(line_idx + i);
            }
        }
    }

    // Second pass: check shared memory accesses
    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = (line_idx + 1) as u32;
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // Check if this line has shared memory access (load or store)
        let has_shared_access =
            PTX_SHARED_LOAD_PATTERN.is_match(line) || PTX_SHARED_STORE_PATTERN.is_match(line);
        if has_shared_access {
            // Check for safety patterns:

            // 1. Predicated access is safe (@%p ld.shared)
            if PTX_PREDICATED_SHARED_PATTERN.is_match(line) {
                continue;
            }

            // 2. Constant offset is safe (shared_mem + 128)
            if PTX_CONSTANT_OFFSET_PATTERN.is_match(line) {
                continue;
            }

            // 3. Covered by preceding bounds check
            if bounds_check_lines.contains(&line_idx) {
                continue;
            }

            // Unsafe: shared access without bounds protection
            violations.push((
                line_num,
                "CB-060-B",
                "Unbounded shared memory access (no bounds check or predicate)".to_string(),
            ));
        }
    }

    violations
}

/// Detect tiled kernels without boundary predicates
///
/// Tiled kernels (GEMM, etc.) must check row < m && col < n before storing
/// to avoid out-of-bounds writes on non-tile-aligned dimensions.
pub fn detect_tiled_kernel_no_bounds_in_str(code: &str) -> Vec<(u32, &'static str, String)> {
    debug_assert!(!code.is_empty(), "code must not be empty");
    let mut violations = Vec::new();
    let lines: Vec<&str> = code.lines().collect();

    // Check for PTX early exit pattern
    if PTX_EARLY_EXIT_PATTERN.is_match(code) {
        // Find the line with the early exit
        for (line_idx, line) in lines.iter().enumerate() {
            if line.contains("@%p") && line.contains("bra") && line.contains("exit") {
                violations.push((
                    (line_idx + 1) as u32,
                    "CB-060-C",
                    "Early exit before tile loop may cause barrier divergence".to_string(),
                ));
                break;
            }
        }
    }

    // Track bounds check state
    let mut has_proper_bounds = false;
    let mut bounds_check_line: Option<usize> = None;

    // Scan for bounds checks and stores
    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Skip comments and string literals
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }
        if trimmed.starts_with('"') || trimmed.contains("= \"") {
            continue;
        }

        // Look for proper bounds checks
        if TILED_BOUNDS_CHECK_PATTERN.is_match(line) || TILED_COMPLEX_BOUNDS_PATTERN.is_match(line)
        {
            has_proper_bounds = true;
            bounds_check_line = Some(line_idx);
        }

        // Look for partial bounds (only row OR col)
        let has_row_check = line.contains("row <") || line.contains("row<");
        let has_col_check = line.contains("col <") || line.contains("col<");
        let has_if = line.contains("if ");
        if has_if && has_row_check && !has_col_check {
            // Partial bounds - only row checked
            violations.push((
                (line_idx + 1) as u32,
                "CB-060-C",
                "Partial bounds check: row checked but not col".to_string(),
            ));
        }
    }

    // Look for tiled stores
    check_tiled_stores(
        &lines,
        has_proper_bounds,
        bounds_check_line,
        &mut violations,
    );

    violations
}

fn check_tiled_stores(
    lines: &[&str],
    has_proper_bounds: bool,
    bounds_check_line: Option<usize>,
    violations: &mut Vec<(u32, &'static str, String)>,
) {
    debug_assert!(!lines.is_empty(), "lines must not be empty");
    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = (line_idx + 1) as u32;
        let trimmed = line.trim();

        // Skip comments and string literals
        if trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }
        if trimmed.starts_with('"')
            || trimmed.contains("= \"")
            || trimmed.starts_with("let kernel_src")
        {
            continue;
        }

        // Check for tiled store pattern
        if TILED_STORE_PATTERN.is_match(line) {
            if !has_proper_bounds {
                violations.push((
                    line_num,
                    "CB-060-C",
                    "Tiled kernel store without bounds check (row < m && col < n)".to_string(),
                ));
            } else if let Some(bounds_line) = bounds_check_line {
                if bounds_line > line_idx {
                    violations.push((
                        line_num,
                        "CB-060-C",
                        "Bounds check after store (must be before)".to_string(),
                    ));
                }
            }
        }

        // Check for WGSL tiled pattern
        if WGSL_TILED_STORE_PATTERN.is_match(line) && !has_proper_bounds {
            violations.push((
                line_num,
                "CB-060-C",
                "WGSL tiled kernel without bounds check".to_string(),
            ));
        }
    }
}
