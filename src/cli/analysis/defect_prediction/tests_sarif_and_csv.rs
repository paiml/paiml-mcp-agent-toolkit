    // ==================== Test format_defect_sarif ====================

    #[test]
    fn test_format_defect_sarif_structure() {
        let predictions = create_test_predictions();

        let result = format_defect_sarif(&predictions).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        // Check SARIF schema
        assert_eq!(parsed["version"], "2.1.0");
        assert!(parsed["$schema"].as_str().unwrap().contains("sarif-schema"));

        // Check runs
        let runs = parsed["runs"].as_array().unwrap();
        assert_eq!(runs.len(), 1);

        // Check tool info
        let tool = &runs[0]["tool"]["driver"];
        assert_eq!(tool["name"], "pmat-defect-prediction");
        assert!(tool["version"].is_string());

        // Check results
        let results = runs[0]["results"].as_array().unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_format_defect_sarif_risk_levels() {
        let predictions = create_test_predictions();

        let result = format_defect_sarif(&predictions).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let results = parsed["runs"][0]["results"].as_array().unwrap();

        // High risk should be "error"
        let high_risk = results
            .iter()
            .find(|r| {
                r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str()
                    .unwrap()
                    .contains("high_risk")
            })
            .unwrap();
        assert_eq!(high_risk["level"], "error");

        // Medium risk should be "warning"
        let medium_risk = results
            .iter()
            .find(|r| {
                r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str()
                    .unwrap()
                    .contains("medium_risk")
            })
            .unwrap();
        assert_eq!(medium_risk["level"], "warning");

        // Low risk should be "note"
        let low_risk = results
            .iter()
            .find(|r| {
                r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str()
                    .unwrap()
                    .contains("low_risk")
            })
            .unwrap();
        assert_eq!(low_risk["level"], "note");
    }

    #[test]
    fn test_format_defect_sarif_empty() {
        let predictions: Vec<(String, DefectScore)> = vec![];

        let result = format_defect_sarif(&predictions).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let results = parsed["runs"][0]["results"].as_array().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_format_defect_sarif_properties() {
        let predictions = create_test_predictions();

        let result = format_defect_sarif(&predictions).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let result_item = &parsed["runs"][0]["results"][0];
        let props = &result_item["properties"];

        assert!(props["probability"].is_f64());
        assert!(props["confidence"].is_f64());
        assert!(props["contributing_factors"].is_array());
        assert!(props["recommendations"].is_array());
    }

    #[test]
    fn test_format_defect_sarif_locations() {
        let predictions = create_test_predictions();

        let result = format_defect_sarif(&predictions).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

        let result_item = &parsed["runs"][0]["results"][0];
        let location = &result_item["locations"][0];

        assert!(location["physicalLocation"]["artifactLocation"]["uri"].is_string());
        assert_eq!(
            location["physicalLocation"]["artifactLocation"]["uriBaseId"],
            "%SRCROOT%"
        );
    }

    // ==================== Test format_defect_csv ====================

    #[test]
    fn test_format_defect_csv_header() {
        let predictions = create_test_predictions();

        let result = format_defect_csv(&predictions).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(
            lines[0],
            "file,probability,confidence,risk_level,top_factor,top_factor_weight"
        );
    }

    #[test]
    fn test_format_defect_csv_data_rows() {
        let predictions = create_test_predictions();

        let result = format_defect_csv(&predictions).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 4); // 1 header + 3 data rows

        // Check first data row
        assert!(lines[1].contains("high_risk.rs"));
        assert!(lines[1].contains("0.850"));
    }

    #[test]
    fn test_format_defect_csv_empty_factors() {
        let mut score = create_high_risk_score();
        score.contributing_factors = vec![];
        let predictions = vec![("test.rs".to_string(), score)];

        let result = format_defect_csv(&predictions).unwrap();

        // Should handle missing top factor gracefully
        assert!(result.contains("test.rs"));
        assert!(result.contains("0.000")); // Default weight
    }

    #[test]
    fn test_format_defect_csv_empty_predictions() {
        let predictions: Vec<(String, DefectScore)> = vec![];

        let result = format_defect_csv(&predictions).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 1); // Only header
    }

    #[test]
    fn test_format_defect_csv_with_comma_in_factor() {
        let mut score = create_high_risk_score();
        score.contributing_factors = vec![("factor,with,commas".to_string(), 0.5)];
        let predictions = vec![("test.rs".to_string(), score)];

        let result = format_defect_csv(&predictions).unwrap();

        // CSV should still be parseable
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2);
    }
