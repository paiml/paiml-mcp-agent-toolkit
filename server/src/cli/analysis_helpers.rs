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
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_write_analysis_output_to_file() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.txt");

        let result = write_analysis_output(
            "test content",
            Some(output_path.clone()),
            "Analysis saved to",
        )
        .await;

        assert!(result.is_ok());

        let content = tokio::fs::read_to_string(output_path).await.unwrap();
        assert_eq!(content, "test content");
    }

    #[tokio::test]
    async fn test_write_analysis_output_to_stdout() {
        let result = write_analysis_output("test content", None, "Analysis complete").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_write_analysis_output_invalid_path() {
        let invalid_path = PathBuf::from("/invalid/nonexistent/path/file.txt");

        let result =
            write_analysis_output("test content", Some(invalid_path), "Analysis saved to").await;

        assert!(result.is_err());
    }

    #[test]
    fn test_merge_ranking_into_json_valid() {
        let json_content = r#"{"existing": "value"}"#;
        let ranking_data = json!({"rank": 1, "score": 0.95});

        let result = merge_ranking_into_json(json_content, "ranking", ranking_data).unwrap();

        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["existing"], "value");
        assert_eq!(parsed["ranking"]["rank"], 1);
        assert_eq!(parsed["ranking"]["score"], 0.95);
    }

    #[test]
    fn test_merge_ranking_into_json_empty_object() {
        let json_content = r#"{}"#;
        let ranking_data = json!({"test": "data"});

        let result = merge_ranking_into_json(json_content, "new_key", ranking_data).unwrap();

        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["new_key"]["test"], "data");
    }

    #[test]
    fn test_merge_ranking_into_json_invalid_json() {
        let invalid_json = r#"{"invalid": json"#;
        let ranking_data = json!({"test": "data"});

        let result = merge_ranking_into_json(invalid_json, "key", ranking_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_ranking_into_json_array_input() {
        let json_content = r#"["array", "not", "object"]"#;
        let ranking_data = json!({"test": "data"});

        let result = merge_ranking_into_json(json_content, "key", ranking_data).unwrap();

        // Should return original array unchanged since it's not an object
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 3);
    }

    #[derive(Debug, Clone)]
    struct TestItem {
        severity: u8,
        name: String,
    }

    #[test]
    fn test_filter_by_severity_removes_low_severity() {
        let mut items = vec![
            TestItem {
                severity: 1,
                name: "low".to_string(),
            },
            TestItem {
                severity: 5,
                name: "medium".to_string(),
            },
            TestItem {
                severity: 9,
                name: "high".to_string(),
            },
        ];

        filter_by_severity(&mut items, |item| item.severity, 5);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "medium");
        assert_eq!(items[1].name, "high");
    }

    #[test]
    fn test_filter_by_severity_keeps_all() {
        let mut items = vec![
            TestItem {
                severity: 5,
                name: "item1".to_string(),
            },
            TestItem {
                severity: 7,
                name: "item2".to_string(),
            },
        ];

        filter_by_severity(&mut items, |item| item.severity, 3);

        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_filter_by_severity_removes_all() {
        let mut items = vec![
            TestItem {
                severity: 1,
                name: "item1".to_string(),
            },
            TestItem {
                severity: 2,
                name: "item2".to_string(),
            },
        ];

        filter_by_severity(&mut items, |item| item.severity, 5);

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_filter_by_severity_empty_input() {
        let mut items: Vec<TestItem> = vec![];

        filter_by_severity(&mut items, |item| item.severity, 5);

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_apply_limit_with_limit() {
        let mut items = vec![1, 2, 3, 4, 5];

        apply_limit(&mut items, Some(3));

        assert_eq!(items.len(), 3);
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[test]
    fn test_apply_limit_no_limit() {
        let mut items = vec![1, 2, 3, 4, 5];

        apply_limit(&mut items, None);

        assert_eq!(items.len(), 5);
        assert_eq!(items, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_apply_limit_larger_than_size() {
        let mut items = vec![1, 2, 3];

        apply_limit(&mut items, Some(10));

        assert_eq!(items.len(), 3);
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[test]
    fn test_apply_limit_zero() {
        let mut items = vec![1, 2, 3, 4, 5];

        apply_limit(&mut items, Some(0));

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_apply_limit_empty_input() {
        let mut items: Vec<i32> = vec![];

        apply_limit(&mut items, Some(5));

        assert_eq!(items.len(), 0);
    }
}
