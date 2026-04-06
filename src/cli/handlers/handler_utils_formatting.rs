// Formatting and display utility functions for CLI handlers
// Included by handler_utils.rs - shares parent module scope

/// Format file path for display, truncating long paths
#[must_use]
pub fn format_display_path(path: &std::path::Path, max_len: usize) -> String {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let path_str = path.to_string_lossy();
    if path_str.len() <= max_len {
        path_str.to_string()
    } else {
        format!(
            "...{}",
            path_str
                .get(path_str.len() - max_len + 3..)
                .unwrap_or(&path_str)
        )
    }
}

/// Validate output format string and return canonical form
#[must_use]
pub fn normalize_output_format(format: &str) -> &'static str {
    match format.to_lowercase().as_str() {
        "json" | "j" => "json",
        "markdown" | "md" | "m" => "markdown",
        "text" | "txt" | "t" | "" => "text",
        "yaml" | "yml" | "y" => "yaml",
        "html" | "h" => "html",
        "csv" | "c" => "csv",
        _ => "text",
    }
}

/// Calculate severity level from numeric score
#[must_use]
pub fn score_to_severity(score: f64) -> &'static str {
    match score {
        s if s >= 0.9 => "critical",
        s if s >= 0.7 => "high",
        s if s >= 0.4 => "medium",
        _ => "low",
    }
}

/// Get top N items from a slice, or all items if limit is 0
#[must_use]
pub fn get_top_n<T: Clone>(items: &[T], limit: usize) -> Vec<T> {
    if limit == 0 {
        items.to_vec()
    } else {
        items.iter().take(limit).cloned().collect()
    }
}

/// Write content to file or stdout based on output path
///
/// # Errors
///
/// Returns error if file write fails
pub fn output_to_file_or_stdout(
    output: Option<&std::path::Path>,
    content: &str,
) -> std::io::Result<()> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    if let Some(path) = output {
        std::fs::write(path, content)
    } else {
        println!("{content}");
        Ok(())
    }
}

/// Format a count with appropriate pluralization
#[must_use]
pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{count} {singular}")
    } else {
        format!("{count} {plural}")
    }
}

/// Format a duration in a human-readable way
#[must_use]
pub fn format_duration(duration: std::time::Duration) -> String {
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    if secs >= 60 {
        let mins = secs / 60;
        let remaining_secs = secs % 60;
        format!("{}m {}s", mins, remaining_secs)
    } else if secs > 0 {
        format!("{}.{}s", secs, millis / 100)
    } else {
        format!("{}ms", millis)
    }
}

/// Calculate percentage with bounds (0-100)
#[must_use]
pub fn calculate_percentage(numerator: usize, denominator: usize) -> f64 {
    // Contract: calculate_percentage returns a bounded score
    if denominator == 0 {
        0.0
    } else {
        ((numerator as f64 / denominator as f64) * 100.0).clamp(0.0, 100.0)
    }
}
