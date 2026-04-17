// Fault annotation parsing, JSON parsing, enrichment simulation,
// and workspace coverage merge tests.
// Included via include!() from tests_part2.rs - no use imports allowed.

// ── Fault annotation parsing and filtering logic tests ──────────────

#[test]
fn test_fault_annotation_line_extraction_pattern() {
    let annotation = "BH001: Boundary condition at line 42";
    let line_part = annotation.split("at line ").last().unwrap();
    let line: usize = line_part.parse().unwrap();
    assert_eq!(line, 42);
}

#[test]
fn test_fault_annotation_line_extraction_multi_digit() {
    let annotation = "BH003: Overflow risk at line 1234";
    let line_part = annotation.split("at line ").last().unwrap();
    let line: usize = line_part.parse().unwrap();
    assert_eq!(line, 1234);
}

#[test]
fn test_fault_annotation_line_range_filtering() {
    let func_start: usize = 10;
    let func_loc: u32 = 20;
    let func_end = func_start + func_loc as usize;

    let faults = ["BH001: Before range at line 5".to_string(),
        "BH002: In range start at line 10".to_string(),
        "BH003: In range middle at line 20".to_string(),
        "BH004: In range end at line 30".to_string(),
        "BH005: After range at line 31".to_string(),
        "BH006: Way after at line 100".to_string()];

    let relevant: Vec<_> = faults
        .iter()
        .filter(|f| {
            if let Some(line_part) = f.split("at line ").last() {
                if let Ok(line) = line_part.parse::<usize>() {
                    return line >= func_start && line <= func_end;
                }
            }
            false
        })
        .cloned()
        .collect();

    assert_eq!(relevant.len(), 3);
    assert!(relevant[0].contains("line 10"));
    assert!(relevant[1].contains("line 20"));
    assert!(relevant[2].contains("line 30"));
}

#[test]
fn test_fault_annotation_malformed_line() {
    let faults = ["BH001: No line info".to_string(),
        "BH002: Missing number at line ".to_string(),
        "BH003: Valid at line 15".to_string()];

    let func_start: usize = 10;
    let func_loc: u32 = 20;
    let func_end = func_start + func_loc as usize;

    let relevant: Vec<_> = faults
        .iter()
        .filter(|f| {
            if let Some(line_part) = f.split("at line ").last() {
                if let Ok(line) = line_part.parse::<usize>() {
                    return line >= func_start && line <= func_end;
                }
            }
            false
        })
        .cloned()
        .collect();

    assert_eq!(relevant.len(), 1);
    assert!(relevant[0].contains("line 15"));
}

#[test]
fn test_fault_map_path_normalization() {
    let file = "./src/lib.rs";
    let normalized = file.strip_prefix("./").unwrap_or(file);
    assert_eq!(normalized, "src/lib.rs");

    let file_no_prefix = "src/lib.rs";
    let normalized2 = file_no_prefix.strip_prefix("./").unwrap_or(file_no_prefix);
    assert_eq!(normalized2, "src/lib.rs");
}

#[test]
fn test_fault_finding_json_parsing() {
    let json_str = r#"{
        "findings": [
            {
                "file": "./src/handler.rs",
                "line": 15,
                "title": "Unchecked unwrap",
                "id": "BH001"
            },
            {
                "file": "src/utils.rs",
                "line": 42,
                "title": "Boundary condition",
                "id": "BH002"
            }
        ]
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();
    assert_eq!(findings.len(), 2);

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown fault pattern");
        let id = finding.get("id").and_then(|i| i.as_str()).unwrap_or("BH");

        let normalized_file = file.strip_prefix("./").unwrap_or(file);
        let key = normalized_file.to_string();
        let annotation = format!("{}: {} at line {}", id, title, line);
        fault_map.entry(key).or_default().push(annotation);
    }

    assert_eq!(fault_map.len(), 2);
    assert_eq!(fault_map["src/handler.rs"].len(), 1);
    assert!(fault_map["src/handler.rs"][0].contains("BH001"));
    assert!(fault_map["src/handler.rs"][0].contains("line 15"));
    assert_eq!(fault_map["src/utils.rs"].len(), 1);
    assert!(fault_map["src/utils.rs"][0].contains("BH002"));
}

#[test]
fn test_fault_finding_json_no_findings_key() {
    let json_str = r#"{"status": "ok"}"#;
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").and_then(|f| f.as_array());
    assert!(findings.is_none());
}

#[test]
fn test_fault_finding_json_empty_findings() {
    let json_str = r#"{"findings": []}"#;
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();
    assert!(findings.is_empty());
}

#[test]
fn test_fault_finding_json_missing_fields() {
    let json_str = r#"{
        "findings": [
            {},
            {"file": "a.rs"},
            {"file": "b.rs", "line": 10},
            {"file": "c.rs", "line": 20, "title": "Some fault"}
        ]
    }"#;
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown fault pattern");
        let id = finding.get("id").and_then(|i| i.as_str()).unwrap_or("BH");

        let normalized_file = file.strip_prefix("./").unwrap_or(file);
        let key = normalized_file.to_string();
        let annotation = format!("{}: {} at line {}", id, title, line);
        fault_map.entry(key).or_default().push(annotation);
    }

    assert!(fault_map.contains_key(""));
    let a_annotations = &fault_map["a.rs"];
    assert!(a_annotations[0].contains("BH: Unknown fault pattern at line 0"));
    let c_annotations = &fault_map["c.rs"];
    assert!(c_annotations[0].contains("BH: Some fault at line 20"));
}

#[test]
fn test_fault_finding_json_start_search() {
    let stdout = "WARNING: something\n{\"findings\": []}";
    let json_start = stdout.find('{');
    assert!(json_start.is_some());
    let json_str = &stdout[json_start.unwrap()..];
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert!(parsed.get("findings").is_some());
}

#[test]
fn test_fault_finding_no_json_in_output() {
    let stdout = "No output here";
    let json_start = stdout.find('{');
    assert!(json_start.is_none());
}

#[test]
fn test_fault_enrichment_full_flow_simulation() {
    let mut entry_a = create_test_entry("handler", 5, 2.0);
    entry_a.file_path = "src/handler.rs".to_string();
    entry_a.start_line = 10;
    entry_a.quality.loc = 20;

    let mut entry_b = create_test_entry("validate", 3, 1.0);
    entry_b.file_path = "src/utils.rs".to_string();
    entry_b.start_line = 50;
    entry_b.quality.loc = 10;

    let mut results = [QueryResult::from_entry(&entry_a, 0.9, false),
        QueryResult::from_entry(&entry_b, 0.8, false)];

    let json_str = r#"{
        "findings": [
            {"file": "./src/handler.rs", "line": 15, "title": "Unchecked unwrap", "id": "BH001"},
            {"file": "./src/handler.rs", "line": 25, "title": "In range", "id": "BH002"},
            {"file": "./src/handler.rs", "line": 35, "title": "Out of range", "id": "BH003"},
            {"file": "./src/utils.rs", "line": 55, "title": "Boundary check", "id": "BH004"},
            {"file": "./src/utils.rs", "line": 100, "title": "Way out of range", "id": "BH005"}
        ]
    }"#;

    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let findings = parsed.get("findings").unwrap().as_array().unwrap();

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    for finding in findings {
        let file = finding.get("file").and_then(|f| f.as_str()).unwrap_or("");
        let line = finding.get("line").and_then(|l| l.as_u64()).unwrap_or(0);
        let title = finding
            .get("title")
            .and_then(|t| t.as_str())
            .unwrap_or("Unknown fault pattern");
        let id = finding.get("id").and_then(|i| i.as_str()).unwrap_or("BH");
        let normalized_file = file.strip_prefix("./").unwrap_or(file);
        let key = normalized_file.to_string();
        let annotation = format!("{}: {} at line {}", id, title, line);
        fault_map.entry(key).or_default().push(annotation);
    }

    for result in results.iter_mut() {
        if let Some(faults) = fault_map.get(&result.file_path) {
            let func_start = result.start_line;
            let func_end = result.start_line + result.loc as usize;

            let relevant_faults: Vec<_> = faults
                .iter()
                .filter(|f| {
                    if let Some(line_part) = f.split("at line ").last() {
                        if let Ok(line) = line_part.parse::<usize>() {
                            return line >= func_start && line <= func_end;
                        }
                    }
                    false
                })
                .cloned()
                .collect();

            if !relevant_faults.is_empty() {
                result.fault_annotations = relevant_faults;
            }
        }
    }

    // handler: start=10, loc=20, end=30
    assert_eq!(results[0].fault_annotations.len(), 2);
    assert!(results[0].fault_annotations[0].contains("BH001"));
    assert!(results[0].fault_annotations[1].contains("BH002"));

    // validate: start=50, loc=10, end=60
    assert_eq!(results[1].fault_annotations.len(), 1);
    assert!(results[1].fault_annotations[0].contains("BH004"));
}

#[test]
fn test_fault_enrichment_no_faults_for_file() {
    let mut entry = create_test_entry("orphan_func", 3, 1.0);
    entry.file_path = "src/orphan.rs".to_string();
    entry.start_line = 1;
    entry.quality.loc = 50;

    let mut results = [QueryResult::from_entry(&entry, 0.9, false)];
    let fault_map: HashMap<String, Vec<String>> = HashMap::new();

    for result in results.iter_mut() {
        if let Some(faults) = fault_map.get(&result.file_path) {
            let func_start = result.start_line;
            let func_end = result.start_line + result.loc as usize;
            let relevant: Vec<_> = faults
                .iter()
                .filter(|f| {
                    if let Some(lp) = f.split("at line ").last() {
                        if let Ok(l) = lp.parse::<usize>() {
                            return l >= func_start && l <= func_end;
                        }
                    }
                    false
                })
                .cloned()
                .collect();
            if !relevant.is_empty() {
                result.fault_annotations = relevant;
            }
        }
    }

    assert!(results[0].fault_annotations.is_empty());
}

#[test]
fn test_fault_enrichment_all_faults_out_of_range() {
    let mut entry = create_test_entry("narrow_func", 3, 1.0);
    entry.file_path = "src/narrow.rs".to_string();
    entry.start_line = 100;
    entry.quality.loc = 5;

    let mut results = [QueryResult::from_entry(&entry, 0.9, false)];

    let mut fault_map: HashMap<String, Vec<String>> = HashMap::new();
    fault_map.insert(
        "src/narrow.rs".to_string(),
        vec![
            "BH001: Early fault at line 10".to_string(),
            "BH002: Late fault at line 200".to_string(),
        ],
    );

    for result in results.iter_mut() {
        if let Some(faults) = fault_map.get(&result.file_path) {
            let func_start = result.start_line;
            let func_end = result.start_line + result.loc as usize;
            let relevant: Vec<_> = faults
                .iter()
                .filter(|f| {
                    if let Some(lp) = f.split("at line ").last() {
                        if let Ok(l) = lp.parse::<usize>() {
                            return l >= func_start && l <= func_end;
                        }
                    }
                    false
                })
                .cloned()
                .collect();
            if !relevant.is_empty() {
                result.fault_annotations = relevant;
            }
        }
    }

    assert!(results[0].fault_annotations.is_empty());
}

#[test]
fn test_load_workspace_coverage_merges_siblings() {
    use super::coverage::load_workspace_coverage;

    let tmp1 = tempfile::TempDir::new().unwrap();
    let tmp2 = tempfile::TempDir::new().unwrap();

    let pmat1 = tmp1.path().join(".pmat");
    let pmat2 = tmp2.path().join(".pmat");
    std::fs::create_dir_all(&pmat1).unwrap();
    std::fs::create_dir_all(&pmat2).unwrap();

    let cache1 = serde_json::json!({
        "git_hash": "abc123",
        "coverage_mtime": 0,
        "files": {
            "src/lib.rs": {"1": 5, "2": 0, "3": 10}
        }
    });
    std::fs::write(pmat1.join("coverage-cache.json"), cache1.to_string()).unwrap();

    let cache2 = serde_json::json!({
        "git_hash": "def456",
        "coverage_mtime": 0,
        "files": {
            "src/main.rs": {"1": 1, "5": 3}
        }
    });
    std::fs::write(pmat2.join("coverage-cache.json"), cache2.to_string()).unwrap();

    let siblings = vec![
        (pmat1.join("context.db"), "trueno".to_string()),
        (pmat2.join("context.db"), "realizar".to_string()),
    ];

    let merged = load_workspace_coverage(&siblings);

    assert!(merged.contains_key("trueno/src/lib.rs"));
    assert!(merged.contains_key("realizar/src/main.rs"));
    assert_eq!(merged.len(), 2);

    let trueno_lib = &merged["trueno/src/lib.rs"];
    assert_eq!(trueno_lib.get(&1), Some(&5));
    assert_eq!(trueno_lib.get(&2), Some(&0));
}

#[test]
fn test_load_workspace_coverage_skips_missing() {
    use super::coverage::load_workspace_coverage;

    let tmp = tempfile::TempDir::new().unwrap();
    let pmat = tmp.path().join(".pmat");
    std::fs::create_dir_all(&pmat).unwrap();

    let siblings = vec![(pmat.join("context.db"), "missing_project".to_string())];

    let merged = load_workspace_coverage(&siblings);
    assert!(merged.is_empty());
}
