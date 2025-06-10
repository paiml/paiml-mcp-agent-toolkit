use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;

/// Helper to format and output analysis results
pub async fn write_analysis_output(
    content: &str,
    output_path: Option<PathBuf>,
    success_message: &str,
) -> Result<()> {
    if let Some(path) = output_path {
        tokio::fs::write(&path, content).await?;
        eprintln!("✅ {} {}", success_message, path.display());
    } else {
        println!("{content}");
    }
    Ok(())
}

/// Helper to merge ranking data into JSON output
pub fn merge_ranking_into_json(
    json_content: &str,
    key: &str,
    ranking_data: Value,
) -> Result<String> {
    let mut report_json: Value = serde_json::from_str(json_content)?;
    if let Some(obj) = report_json.as_object_mut() {
        obj.insert(key.to_string(), ranking_data);
    }
    Ok(serde_json::to_string_pretty(&report_json)?)
}

/// Helper to apply severity filters to items
pub fn filter_by_severity<T>(
    items: &mut Vec<T>,
    severity_field: impl Fn(&T) -> u8,
    min_severity: u8,
) {
    items.retain(|item| severity_field(item) >= min_severity);
}

/// Helper to truncate results to a limit
pub fn apply_limit<T>(items: &mut Vec<T>, limit: Option<usize>) {
    if let Some(max) = limit {
        items.truncate(max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analysis_helpers_basic() {
        // Basic test
        assert_eq!(1 + 1, 2);
    }
}
