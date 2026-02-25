
#[test]
fn test_new_with_default_config() {
    // Verify creation does not panic and the analyzer is usable
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let result = analyzer.format_as_json(&ctx);
    assert!(result.is_ok());
}

#[test]
fn test_new_with_custom_config() {
    let config = DeepContextConfig {
        period_days: 90,
        parallel: 2,
        ..Default::default()
    };
    let analyzer = DeepContextAnalyzer::new(config);
    // Verify the custom-config analyzer can produce output (config is private)
    let ctx = make_empty_context();
    let json = analyzer.format_as_json(&ctx).unwrap();
    assert!(json.contains("metadata"));
}

// ===========================================================================
// format_as_comprehensive_markdown (async) -- empty context
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_empty_context() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let result = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(result.starts_with("# Deep Context Analysis Report"));
    assert!(result.contains("Quality Scorecard"));
}

#[tokio::test]
async fn test_comprehensive_markdown_contains_header() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("# Deep Context Analysis Report"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- project overview
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_with_project_overview() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.project_overview = Some(make_project_overview());
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("## Project Overview"));
    assert!(md.contains("A test project for analysis."));
    assert!(md.contains("Feature A"));
    assert!(md.contains("Feature B"));
    assert!(md.contains("Microservices architecture"));
}

#[tokio::test]
async fn test_comprehensive_markdown_overview_no_description() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.project_overview = Some(ProjectOverview {
        compressed_description: String::new(),
        key_features: vec!["Only feature".to_string()],
        architecture_summary: None,
        api_summary: None,
    });
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("## Project Overview"));
    assert!(md.contains("Only feature"));
    // Empty description should not produce an extra paragraph
    assert!(!md.contains("\n\n\n\n"));
}

#[tokio::test]
async fn test_comprehensive_markdown_overview_no_features() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.project_overview = Some(ProjectOverview {
        compressed_description: "Desc".to_string(),
        key_features: Vec::new(),
        architecture_summary: None,
        api_summary: None,
    });
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("Desc"));
    assert!(!md.contains("Key Features"));
}

#[tokio::test]
async fn test_comprehensive_markdown_overview_no_architecture() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.project_overview = Some(ProjectOverview {
        compressed_description: "Desc".to_string(),
        key_features: vec!["A".to_string()],
        architecture_summary: None,
        api_summary: None,
    });
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(!md.contains("**Architecture:**"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- build info
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_with_build_info() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.build_info = Some(make_build_info());
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("## Build System"));
    assert!(md.contains("**Detected Toolchain:** Rust"));
    assert!(md.contains("pmat, pmat-cli"));
    assert!(md.contains("serde, tokio"));
    assert!(md.contains("`cargo build --release`"));
}

#[tokio::test]
async fn test_comprehensive_markdown_build_info_no_targets() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.build_info = Some(BuildInfo {
        toolchain: "Python".to_string(),
        targets: Vec::new(),
        dependencies: Vec::new(),
        primary_command: None,
    });
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("**Detected Toolchain:** Python"));
    assert!(!md.contains("Primary Targets"));
    assert!(!md.contains("Key Dependencies"));
    assert!(!md.contains("Build Command"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- quality scorecard
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_quality_scorecard_default() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("Overall Health: 0.0%"));
    assert!(md.contains("Maintainability Index: 0.0%"));
    assert!(md.contains("Refactoring Time: 0.0 hours"));
    assert!(md.contains("Complexity Score: 0.0%"));
}

#[tokio::test]
async fn test_comprehensive_markdown_quality_scorecard_high_health() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.quality_scorecard = make_scorecard(95.0, 88.0, 2.0);
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("Overall Health: 95.0%"));
    assert!(md.contains("Maintainability Index: 88.0%"));
    assert!(md.contains("Refactoring Time: 2.0 hours"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- project structure
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_project_structure() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.file_tree = make_annotated_tree(100, 1_000_000);
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("## Project Structure"));
    assert!(md.contains("Total Files: 100"));
    assert!(md.contains("Total Size: 1000000 bytes"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- analysis results
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_analysis_results_empty() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("## Analysis Results"));
    // No AST, complexity, or churn sub-headings when empty
    assert!(!md.contains("### AST Analysis"));
    assert!(!md.contains("### Complexity Analysis"));
    assert!(!md.contains("### Code Churn"));
}

#[tokio::test]
async fn test_comprehensive_markdown_with_ast_contexts() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.ast_contexts = vec![
        make_enhanced_file_context("a.rs", "Rust"),
        make_enhanced_file_context("b.rs", "Rust"),
        make_enhanced_file_context("c.rs", "Rust"),
    ];
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("### AST Analysis"));
    assert!(md.contains("Files analyzed: 3"));
}

#[tokio::test]
async fn test_comprehensive_markdown_with_complexity_report() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.complexity_report = Some(make_complexity_report());
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("### Complexity Analysis"));
    assert!(md.contains("Total files: 5"));
    assert!(md.contains("Total functions: 20"));
    assert!(md.contains("Median cyclomatic complexity: 4.5"));
}

#[tokio::test]
async fn test_comprehensive_markdown_with_churn_analysis() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.analyses.churn_analysis = Some(make_churn_analysis());
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("### Code Churn"));
    assert!(md.contains("Files analyzed: 1"));
    assert!(md.contains("Total commits: 100"));
}

// ===========================================================================
// format_as_comprehensive_markdown -- recommendations
// ===========================================================================

#[tokio::test]
async fn test_comprehensive_markdown_no_recommendations() {
    let analyzer = make_analyzer();
    let ctx = make_empty_context();
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    // When empty, the recommendations section should NOT be appended
    assert!(!md.contains("## Recommendations"));
}

#[tokio::test]
async fn test_comprehensive_markdown_with_recommendations() {
    let analyzer = make_analyzer();
    let mut ctx = make_empty_context();
    ctx.recommendations = vec![
        make_recommendation("Fix bug", Priority::Critical, Impact::High, vec!["Deploy"]),
        make_recommendation("Add docs", Priority::Low, Impact::Low, vec![]),
    ];
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    assert!(md.contains("## Recommendations"));
    assert!(md.contains("**Fix bug**"));
    assert!(md.contains("Priority: Critical"));
    assert!(md.contains("**Add docs**"));
    assert!(md.contains("Priority: Low"));
}

#[tokio::test]
async fn test_comprehensive_markdown_fully_populated() {
    let analyzer = make_analyzer();
    let ctx = make_populated_context();
    let md = analyzer
        .format_as_comprehensive_markdown(&ctx)
        .await
        .unwrap();
    // All sections should appear
    assert!(md.contains("# Deep Context Analysis Report"));
    assert!(md.contains("## Project Overview"));
    assert!(md.contains("## Build System"));
    assert!(md.contains("## Quality Scorecard"));
    assert!(md.contains("## Project Structure"));
    assert!(md.contains("## Analysis Results"));
    assert!(md.contains("## Recommendations"));
}

// ===========================================================================
// format_as_comprehensive_markdown_legacy -- empty context
// ===========================================================================

