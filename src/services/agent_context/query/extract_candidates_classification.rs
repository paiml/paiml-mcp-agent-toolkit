// Classification functions for extract_candidates module
// Included by extract_candidates.rs — no `use` imports allowed here.

/// Classify a function's source code as PURE or IO.
///
/// Returns the classification string and a list of detected I/O pattern labels.
pub(crate) fn classify_io(source: &str) -> (String, Vec<String>) {
    let mut patterns = Vec::new();
    for (label, markers) in IO_PATTERNS {
        if markers.iter().any(|m| source.contains(m)) {
            patterns.push(label.to_string());
        }
    }
    if patterns.is_empty() {
        ("PURE".to_string(), patterns)
    } else {
        ("IO".to_string(), patterns)
    }
}

/// Classify all results in-place, updating io_classification and io_patterns.
pub(crate) fn classify_all_results(results: &mut [QueryResult]) {
    for r in results.iter_mut() {
        if let Some(ref source) = r.source {
            let (class, patterns) = classify_io(source);
            r.io_classification = class;
            r.io_patterns = patterns;
        } else {
            r.io_classification = "PURE".to_string();
        }
    }
}
