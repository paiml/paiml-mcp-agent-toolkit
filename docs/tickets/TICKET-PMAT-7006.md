# TICKET-PMAT-7006: MCP Tool Enhancement & Final Polish

**Status**: 🔨 TODO
**Priority**: Medium
**Complexity**: Low (2-3 days)
**Sprint**: Sprint 23
**Created**: 2025-10-07
**Current**: 8/9 TODOs removed, final polish needed

## Objective

Complete production-ready MCP tools with full test coverage, removing the final TODO and adding comprehensive integration tests for TransformTool and ValidateTool.

## Current State

- ✅ 8/9 TODOs removed
- ✅ Basic functionality working
- ❌ Missing integration tests
- ❌ One remaining TODO in QualityGateTool

## Requirements

### 1. Integration Tests (2 days)
- [ ] Add integration tests for TransformTool (6 tests)
  - Transform with valid input
  - Transform with invalid input
  - Transform with edge cases (empty, large files)
  - Transform with various formats (JSON, YAML, TOML)
  - Transform error scenarios
  - Transform performance benchmarks

- [ ] Add integration tests for ValidateTool (6 tests)
  - Validate with valid schema
  - Validate with invalid schema
  - Validate with missing required fields
  - Validate with type mismatches
  - Validate error messages are clear
  - Validate performance benchmarks

### 2. QualityGateTool Enhancement (1 day)
- [ ] Remove final TODO: language-aware analysis
- [ ] Add language parameter support
- [ ] Integrate with language detection system
- [ ] Update quality gate rules per language
- [ ] Update tests for language-aware features
- [ ] Document language-specific quality gates

### 3. OrchestrateTool Implementation (1-2 days) [OPTIONAL - depends on PMAT-7003]
- [ ] Connect to WorkflowExecutor (requires Workflow Executor completion)
- [ ] Orchestrate multi-step workflows
- [ ] Handle workflow failures gracefully
- [ ] Integration tests for workflow orchestration

## Implementation Plan

### Files to Create/Modify
- `server/src/mcp/tools/transform_tool.rs` (extend tests)
- `server/src/mcp/tools/validate_tool.rs` (extend tests)
- `server/src/mcp/tools/quality_gate_tool.rs` (remove TODO, add language support)
- `server/src/mcp/tools/orchestrate_tool.rs` (extend - optional)
- `server/tests/mcp_integration_tests.rs` (extend)

### TransformTool Integration Tests
```rust
#[cfg(test)]
mod transform_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_transform_json_to_yaml() {
        let tool = TransformTool::new();
        let input = r#"{"key": "value"}"#;
        let result = tool.transform(input, Format::Json, Format::Yaml).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("key: value"));
    }

    #[tokio::test]
    async fn test_transform_invalid_json() {
        let tool = TransformTool::new();
        let input = r#"{"key": invalid}"#;
        let result = tool.transform(input, Format::Json, Format::Yaml).await;
        assert!(result.is_err());
        // Verify error message is clear
    }

    #[tokio::test]
    async fn test_transform_empty_input() {
        let tool = TransformTool::new();
        let result = tool.transform("", Format::Json, Format::Yaml).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_transform_large_file() {
        let tool = TransformTool::new();
        let large_json = generate_large_json(10_000); // 10K objects
        let start = Instant::now();
        let result = tool.transform(&large_json, Format::Json, Format::Yaml).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration < Duration::from_secs(5)); // Performance check
    }

    #[tokio::test]
    async fn test_transform_all_format_combinations() {
        let tool = TransformTool::new();
        let formats = [Format::Json, Format::Yaml, Format::Toml];

        for from_fmt in &formats {
            for to_fmt in &formats {
                let input = generate_sample_data(*from_fmt);
                let result = tool.transform(&input, *from_fmt, *to_fmt).await;
                assert!(result.is_ok(), "Failed: {:?} -> {:?}", from_fmt, to_fmt);
            }
        }
    }

    #[tokio::test]
    async fn test_transform_error_messages() {
        let tool = TransformTool::new();
        let invalid_inputs = vec![
            (r#"{"unclosed": "#, "Unclosed JSON object"),
            (r#"key: [unclosed"#, "Invalid YAML syntax"),
            (r#"[invalid toml"#, "Invalid TOML syntax"),
        ];

        for (input, expected_msg) in invalid_inputs {
            let result = tool.transform(input, Format::Json, Format::Yaml).await;
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains(expected_msg));
        }
    }
}
```

### ValidateTool Integration Tests
```rust
#[cfg(test)]
mod validate_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_with_valid_schema() {
        let tool = ValidateTool::new();
        let schema = create_sample_schema();
        let data = create_valid_data();
        let result = tool.validate(&data, &schema).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_valid);
    }

    #[tokio::test]
    async fn test_validate_with_missing_required_field() {
        let tool = ValidateTool::new();
        let schema = create_schema_with_required_field("name");
        let data = json!({"age": 30}); // Missing "name"
        let result = tool.validate(&data, &schema).await;

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.is_valid);
        assert!(validation.errors.iter().any(|e| e.contains("name")));
    }

    #[tokio::test]
    async fn test_validate_with_type_mismatch() {
        let tool = ValidateTool::new();
        let schema = create_schema_with_type("age", "integer");
        let data = json!({"age": "thirty"}); // Should be integer
        let result = tool.validate(&data, &schema).await;

        assert!(result.is_ok());
        let validation = result.unwrap();
        assert!(!validation.is_valid);
        assert!(validation.errors.iter().any(|e| e.contains("type")));
    }

    #[tokio::test]
    async fn test_validate_nested_schema() {
        let tool = ValidateTool::new();
        let schema = create_nested_schema();
        let valid_data = create_nested_valid_data();
        let invalid_data = create_nested_invalid_data();

        let valid_result = tool.validate(&valid_data, &schema).await;
        assert!(valid_result.unwrap().is_valid);

        let invalid_result = tool.validate(&invalid_data, &schema).await;
        assert!(!invalid_result.unwrap().is_valid);
    }

    #[tokio::test]
    async fn test_validate_error_messages_are_clear() {
        let tool = ValidateTool::new();
        let schema = create_complex_schema();
        let invalid_data = create_invalid_data_multiple_errors();
        let result = tool.validate(&invalid_data, &schema).await;

        let validation = result.unwrap();
        assert!(!validation.is_valid);

        // Errors should be specific and actionable
        for error in &validation.errors {
            assert!(error.len() > 10); // Not just "Invalid"
            assert!(error.contains("field") || error.contains("type") || error.contains("value"));
        }
    }

    #[tokio::test]
    async fn test_validate_performance_large_schema() {
        let tool = ValidateTool::new();
        let large_schema = create_schema_with_n_fields(1000);
        let large_data = create_data_for_large_schema(&large_schema);

        let start = Instant::now();
        let result = tool.validate(&large_data, &large_schema).await;
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration < Duration::from_millis(500)); // Should be fast
    }
}
```

### QualityGateTool Language-Aware Enhancement
```rust
pub struct QualityGateTool {
    language_detector: LanguageDetector,
    quality_gates: HashMap<Language, QualityGateConfig>,
}

impl QualityGateTool {
    pub async fn check_quality(
        &self,
        code: &str,
        language: Option<Language>,
    ) -> Result<QualityReport> {
        // Detect language if not provided
        let lang = match language {
            Some(l) => l,
            None => self.language_detector.detect(code)?,
        };

        // Get language-specific quality gates
        let gates = self.quality_gates
            .get(&lang)
            .ok_or_else(|| Error::UnsupportedLanguage(lang))?;

        // Apply language-specific rules
        self.apply_quality_gates(code, lang, gates).await
    }

    fn apply_quality_gates(
        &self,
        code: &str,
        language: Language,
        config: &QualityGateConfig,
    ) -> Result<QualityReport> {
        // Language-specific complexity thresholds
        let max_complexity = match language {
            Language::Rust => 15,
            Language::Python => 10,
            Language::JavaScript => 12,
            Language::TypeScript => 12,
            _ => 10,
        };

        // Language-specific line length
        let max_line_length = match language {
            Language::Rust => 100,
            Language::Python => 79,
            _ => 120,
        };

        // Apply rules...
        Ok(QualityReport {
            passed: true,
            violations: vec![],
            language,
        })
    }
}
```

## Success Criteria

- [ ] All integration tests pass (12+ new tests)
- [ ] TransformTool test coverage > 90%
- [ ] ValidateTool test coverage > 90%
- [ ] QualityGateTool TODO removed
- [ ] Language-aware quality gates working
- [ ] Performance benchmarks meet targets (<500ms per operation)
- [ ] Error messages are clear and actionable
- [ ] Documentation updated
- [ ] All existing tests still pass

## Testing Strategy

1. **Integration Tests**: End-to-end tool usage scenarios
2. **Performance Tests**: Benchmark transform/validate operations
3. **Error Tests**: All error paths covered
4. **Edge Case Tests**: Empty inputs, large files, nested structures
5. **Language Tests**: Language-specific quality gates

## Value Delivered

**Before**: MCP tools functional but missing tests, one TODO remaining
**After**: Production-ready MCP tools with comprehensive tests and full language support
**Impact**: Higher confidence in MCP tool reliability, better error handling
**ROI**: Medium - Completes MCP tool implementation, improves quality

## Estimated Effort

2-3 days

## Notes

- Focus on test coverage and error handling
- Ensure error messages are user-friendly
- Document language-specific quality gate configurations
- Consider adding more format support to TransformTool (XML, CSV)
- OrchestrateTool implementation is optional and depends on PMAT-7003 completion
