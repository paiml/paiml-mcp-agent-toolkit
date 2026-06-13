/// Check if a function name is too generic for meaningful call graph edges.
///
/// Common method names like `new`, `from`, `clone` appear in thousands of types,
/// creating O(n^2) spurious edges. Excluding them reduces call graph size by ~99%
/// for large repos (e.g., 58GB -> <100MB for 230K-function repos).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn is_generic_callee(name: &str) -> bool {
    matches!(
        name,
        "new" | "from" | "into" | "default" | "clone" | "fmt"
            | "len" | "push" | "pop" | "get" | "set" | "insert" | "remove"
            | "unwrap" | "expect" | "map" | "and_then" | "or_else" | "ok" | "err"
            | "to_string" | "to_owned" | "as_ref" | "as_mut" | "borrow"
            | "iter" | "collect" | "filter" | "fold" | "next"
            | "write" | "read" | "flush" | "close" | "open"
            | "is_empty" | "contains" | "starts_with" | "ends_with"
            | "display" | "debug" | "hash" | "cmp" | "partial_cmp"
            | "serialize" | "deserialize" | "drop"
            | "init" | "run" | "start" | "stop" | "build" | "parse" | "format"
            | "test" | "setup" | "teardown" | "assert" | "verify" | "check"
            // Lua stdlib (prevent noise edges)
            | "require" | "print" | "pairs" | "ipairs" | "type" | "error"
            | "pcall" | "xpcall" | "select" | "rawget" | "rawset" | "rawlen"
            | "tostring" | "tonumber" | "setmetatable" | "getmetatable"
            | "table" | "string" | "math" | "coroutine" | "unpack"
            // C/C++ stdlib and common patterns (prevent noise edges)
            | "main" | "malloc" | "free" | "calloc" | "realloc"
            | "printf" | "fprintf" | "sprintf" | "snprintf"
            | "memcpy" | "memset" | "memmove" | "memcmp"
            | "strlen" | "strcmp" | "strncmp" | "strcpy" | "strcat"
            | "sizeof" | "abort" | "exit"
            | "begin" | "end" | "size" | "empty" | "data" | "clear" | "resize"
            | "front" | "back" | "swap" | "find" | "erase" | "count"
            | "move" | "forward" | "make_shared" | "make_unique"
    )
}

/// Check if a code chunk is a test function or from a test file.
///
/// Used to exclude test code from the index at build time, reducing index size
/// by 25-70% for test-heavy repos.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn is_test_chunk(chunk_name: &str, file_path: &str) -> bool {
    // File-level: skip test directories and test file suffixes
    if file_path.contains("/tests/")
        || file_path.contains("/test/")
        || file_path.starts_with("tests/")
        || file_path.starts_with("test/")
        || file_path.ends_with("_test.rs")
        || file_path.ends_with("_tests.rs")
        || file_path.ends_with("_test.cpp")
        || file_path.ends_with("_test.cc")
        || file_path.ends_with("_test.c")
        || file_path.ends_with("_unittest.cpp")
        || file_path.ends_with("_unittest.cc")
        || file_path.ends_with(".test.ts")
        || file_path.ends_with(".test.js")
        || file_path.ends_with(".spec.ts")
        || file_path.ends_with(".spec.js")
        || file_path.ends_with("_test.py")
        || file_path.ends_with("_test.go")
    {
        return true;
    }
    // Function-level: skip test_ prefixed and TEST_F/TEST/TEST_P (gtest)
    if chunk_name.starts_with("test_")
        || chunk_name.starts_with("TEST_F")
        || chunk_name.starts_with("TEST_P")
        || (chunk_name.starts_with("TEST") && chunk_name.len() > 4 && chunk_name.as_bytes()[4] == b'(')
    {
        return true;
    }
    false
}

/// Build caller/callee graph by matching identifiers in source against function names.
///
/// For each function, extracts identifiers from its source and checks if they match
/// any known function name. If a match is found (and it's not a self-reference),
/// records a call edge. Generic method names (new, clone, etc.) are excluded to
/// prevent O(n^2) edge explosion.
/// Record call edges by iterating identifiers inline (avoids collecting into Vec).
fn record_call_edges_from_source(
    caller_idx: usize,
    source: &str,
    name_index: &HashMap<String, Vec<usize>>,
    calls: &mut HashMap<usize, Vec<usize>>,
    called_by: &mut HashMap<usize, Vec<usize>>,
    seen: &mut std::collections::HashSet<usize>,
) {
    for ident in source.split(|c: char| !c.is_alphanumeric() && c != '_') {
        if ident.len() < 3 || is_keyword(ident) || is_generic_callee(ident) {
            continue;
        }
        let Some(callee_indices) = name_index.get(ident) else {
            continue;
        };
        for &callee_idx in callee_indices {
            if callee_idx != caller_idx && seen.insert(callee_idx) {
                calls
                    .entry(caller_idx)
                    .or_insert_with(|| Vec::with_capacity(8))
                    .push(callee_idx);
                called_by
                    .entry(callee_idx)
                    .or_insert_with(|| Vec::with_capacity(4))
                    .push(caller_idx);
            }
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn build_call_graph(
    functions: &[FunctionEntry],
    name_index: &HashMap<String, Vec<usize>>,
) -> (HashMap<usize, Vec<usize>>, HashMap<usize, Vec<usize>>) {
    let capacity = functions.len() / 2;
    let mut calls: HashMap<usize, Vec<usize>> = HashMap::with_capacity(capacity);
    let mut called_by: HashMap<usize, Vec<usize>> = HashMap::with_capacity(capacity);
    // Reuse seen set across all functions to avoid per-function allocation
    let mut seen: std::collections::HashSet<usize> =
        std::collections::HashSet::with_capacity(64);

    for (caller_idx, func) in functions.iter().enumerate() {
        seen.clear();
        record_call_edges_from_source(
            caller_idx,
            &func.source,
            name_index,
            &mut calls,
            &mut called_by,
            &mut seen,
        );
    }

    (calls, called_by)
}

/// Compute graph metrics (PageRank, centrality) for each function.
///
/// Uses a simplified PageRank algorithm:
/// - Damping factor: 0.85
/// - Iterations: 20 (sufficient for convergence)
/// - Initial score: 1/N for each node
///
/// PageRank represents "importance" - functions that are transitively called
/// by many other functions will have higher scores.
/// Run one iteration of PageRank: distribute scores from callers to callees.
#[allow(clippy::cast_possible_truncation)]
fn pagerank_iteration(
    pagerank: &[f32],
    new_pagerank: &mut [f32],
    calls: &HashMap<usize, Vec<usize>>,
    damping: f32,
    num_functions: usize,
) {
    let teleport = (1.0 - damping) / num_functions as f32;
    new_pagerank.iter_mut().for_each(|s| *s = teleport);

    // Distribute scores along call edges
    for (caller_idx, callees) in calls {
        if !callees.is_empty() {
            let contribution = damping * pagerank[*caller_idx] / callees.len() as f32;
            for &callee_idx in callees {
                if callee_idx < num_functions {
                    new_pagerank[callee_idx] += contribution;
                }
            }
        }
    }

    // Dangling nodes: distribute their rank evenly to all nodes
    let dangling_sum: f32 = (0..num_functions)
        .filter(|idx| calls.get(idx).is_none_or(|c| c.is_empty()))
        .map(|idx| pagerank[idx])
        .sum();
    let dangling_contrib = damping * dangling_sum / num_functions as f32;
    new_pagerank.iter_mut().for_each(|s| *s += dangling_contrib);
}

#[allow(clippy::cast_possible_truncation)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub(crate) fn compute_graph_metrics(
    num_functions: usize,
    calls: &HashMap<usize, Vec<usize>>,
    called_by: &HashMap<usize, Vec<usize>>,
) -> Vec<GraphMetrics> {
    if num_functions == 0 {
        return Vec::new();
    }

    let mut pagerank = vec![1.0 / num_functions as f32; num_functions];
    let mut new_pagerank = vec![0.0; num_functions];

    for _ in 0..20 {
        pagerank_iteration(&pagerank, &mut new_pagerank, calls, 0.85, num_functions);
        std::mem::swap(&mut pagerank, &mut new_pagerank);
    }

    (0..num_functions)
        .map(|idx| {
            let in_degree = called_by.get(&idx).map_or(0, |v| v.len()) as u32;
            let out_degree = calls.get(&idx).map_or(0, |v| v.len()) as u32;
            let centrality =
                (in_degree + out_degree) as f32 / (2.0 * num_functions as f32).max(1.0);
            GraphMetrics {
                pagerank: pagerank[idx],
                centrality,
                in_degree,
                out_degree,
            }
        })
        .collect()
}
