
// ============================================================================
// Pure functions extracted for testability (Toyota Way: Extract Method)
// ============================================================================

/// Compute reachable functions using fixpoint iteration (pure function).
///
/// Given a set of entry points and a call graph, computes the transitive closure
/// of reachable functions.
///
/// # Arguments
/// * `entry_points` - Set of function names known to be reachable (e.g., `main`, `pub` functions)
/// * `function_calls` - Map from caller to set of callees
///
/// # Returns
/// Set of all reachable function names
#[must_use]
 // Pure function tested in pure_function_tests module
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub(crate) fn compute_reachability(
    entry_points: &HashSet<String>,
    function_calls: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    let mut reachable: HashSet<String> = entry_points.clone();
    let mut changed = true;

    while changed {
        changed = false;
        let current_reachable = reachable.clone();

        for reachable_func in &current_reachable {
            if let Some(callees) = function_calls.get(reachable_func) {
                for callee in callees {
                    if !reachable.contains(callee) {
                        reachable.insert(callee.clone());
                        changed = true;
                    }
                }
            }
        }
    }

    reachable
}

/// Find which function contains a given line number in a file.
///
/// Iterates through all known functions to find which one contains the specified line,
/// using a simple heuristic: the function whose declaration line is closest to (but not
/// after) the given line number.
#[must_use]
fn find_containing_function(
    file_path: &str,
    line_number: usize,
    all_functions: &HashMap<String, (String, u32)>,
) -> Option<String> {
    let mut current_function = None;
    for (qualified_name, (func_file, func_line)) in all_functions {
        if qualified_name.starts_with(file_path)
            && func_file == file_path
            && line_number >= *func_line as usize
        {
            current_function = Some(qualified_name.clone());
        }
    }
    current_function
}

/// Find all function calls in a single line of source code.
///
/// Checks each known function to see if it is called (not defined) on this line,
/// excluding self-calls from the caller.
#[must_use]
fn find_calls_in_line(
    line: &str,
    caller: &str,
    all_functions: &HashMap<String, (String, u32)>,
) -> Vec<String> {
    let mut calls = Vec::new();
    for callee_qualified in all_functions.keys() {
        let callee_name = callee_qualified.split("::").last().unwrap_or("");
        if !callee_name.is_empty()
            && line.contains(&format!("{callee_name}("))
            && !line.contains(&format!("fn {callee_name}"))
            && caller != callee_qualified
        {
            calls.push(callee_qualified.clone());
        }
    }
    calls
}

/// Detect function calls within source code lines (pure function).
///
/// Scans lines of code to detect calls to known functions.
///
/// # Arguments
/// * `file_path` - Path of the file being analyzed (for qualified names)
/// * `lines` - Source code lines
/// * `all_functions` - Map of qualified function names to (file_path, line_number)
///
/// # Returns
/// Map from caller qualified name to set of callee qualified names
#[must_use]
 // Pure function tested in pure_function_tests module
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn detect_function_calls_in_lines(
    file_path: &str,
    lines: &[&str],
    all_functions: &HashMap<String, (String, u32)>,
) -> HashMap<String, HashSet<String>> {
    let mut function_calls: HashMap<String, HashSet<String>> = HashMap::new();

    for (i, line) in lines.iter().enumerate() {
        let line_number = i + 1;

        if let Some(caller) = find_containing_function(file_path, line_number, all_functions) {
            for callee in find_calls_in_line(line, &caller, all_functions) {
                function_calls
                    .entry(caller.clone())
                    .or_default()
                    .insert(callee);
            }
        }
    }

    function_calls
}

/// Classify functions as dead or alive based on reachability (pure function).
///
/// # Arguments
/// * `all_functions` - Map of qualified function names to (file_path, line_number)
/// * `reachable` - Set of reachable function names
///
/// # Returns
/// Vector of dead function items (without cfg-gated filtering)
#[must_use]
 // Pure function tested in pure_function_tests module
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn classify_dead_functions_pure(
    all_functions: &HashMap<String, (String, u32)>,
    reachable: &HashSet<String>,
) -> Vec<(String, String, u32)> {
    let mut dead_functions = Vec::new();

    for (qualified_name, (file_path, line)) in all_functions {
        if !reachable.contains(qualified_name) {
            let function_name = qualified_name.split("::").last().unwrap_or("").to_string();
            dead_functions.push((function_name, file_path.clone(), *line));
        }
    }

    dead_functions
}

/// Collect all functions from project context into a map (pure function).
///
/// # Arguments
/// * `files` - Slice of FileContext from ProjectContext
///
/// # Returns
/// Tuple of (all_functions map, entry_points set)
#[must_use]
 // Pure function reserved for future integration
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn collect_functions_from_context(
    files: &[crate::services::context::FileContext],
) -> (HashMap<String, (String, u32)>, HashSet<String>) {
    use crate::services::context::AstItem;

    let mut all_functions: HashMap<String, (String, u32)> = HashMap::new();
    let mut entry_points: HashSet<String> = HashSet::new();

    for file in files {
        for item in &file.items {
            if let AstItem::Function { name, line, .. } = item {
                let qualified_name = format!("{}::{}", file.path, name);
                all_functions.insert(qualified_name.clone(), (file.path.clone(), *line as u32));

                // Mark main functions and exported functions as entry points
                if name == "main" || name.starts_with("pub ") {
                    entry_points.insert(qualified_name);
                }
            }
        }
    }

    (all_functions, entry_points)
}

/// Calculate dead code percentage (pure function).
#[must_use]
 // Pure function tested in pure_function_tests module
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn calculate_dead_percentage(total_functions: usize, dead_count: usize) -> f32 {
    if total_functions > 0 {
        (dead_count as f32 / total_functions as f32) * 100.0
    } else {
        0.0
    }
}
