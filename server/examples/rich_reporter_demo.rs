//! PMAT-REPORT-V1: Rich Reporter Demo
//!
//! Demonstrates the universal rich reporting framework with:
//! - K-Means clustering of defects
//! - PageRank centrality analysis
//! - Louvain community detection
//! - Anomaly detection (Z-score based)
//! - Time series trend analysis
//! - ASCII visualization (progress bars, sparklines, tables)
//!
//! Run with: cargo run --example rich_reporter_demo

use pmat::services::rich_reporter::{
    Finding, FixSuggestion, OutputFormat, ReportConfig, RichReporter, Severity, SourceLocation,
};
use std::path::PathBuf;

fn main() {
    println!("PMAT-REPORT-V1: Rich Reporter Demo");
    println!("{}", "═".repeat(50));
    println!();

    // Create reporter with default config
    let config = ReportConfig {
        format: OutputFormat::Text,
        k_clusters: 3,
        anomaly_threshold: 0.7,
        ..Default::default()
    };

    let mut reporter = RichReporter::new(config)
        .with_title("Code Quality Analysis")
        .with_project("demo-project");

    // Add sample findings
    println!("Adding sample findings...");

    // Cluster 1: Type system issues
    for i in 0..5 {
        reporter.add_finding(Finding {
            id: format!("TYPE-{:03}", i + 1),
            category: "TypeMismatch".to_string(),
            severity: Severity::High,
            location: SourceLocation {
                file: PathBuf::from(format!("src/parser/ast_{}.rs", i)),
                line: 42 + i * 10,
                column: 1,
                scope: Some(format!("parse_expr_{}", i)),
            },
            message: format!("Expected `String`, found `&str` in function {}", i),
            confidence: 0.95,
            cluster_id: None,
            pagerank: None,
            community: None,
            anomaly_score: None,
            fix_suggestion: Some(FixSuggestion {
                description: "Add `.to_string()` call".to_string(),
                confidence: 0.9,
                auto_fixable: true,
                effort_minutes: Some(5),
            }),
        });
    }

    // Cluster 2: Borrow checker issues
    for i in 0..4 {
        reporter.add_finding(Finding {
            id: format!("BORROW-{:03}", i + 1),
            category: "BorrowCheck".to_string(),
            severity: Severity::Critical,
            location: SourceLocation {
                file: PathBuf::from(format!("src/runtime/vm_{}.rs", i)),
                line: 156 + i * 20,
                column: 1,
                scope: Some(format!("execute_{}", i)),
            },
            message: format!("Cannot borrow `data` as mutable in closure {}", i),
            confidence: 0.92,
            cluster_id: None,
            pagerank: None,
            community: None,
            anomaly_score: None,
            fix_suggestion: Some(FixSuggestion {
                description: "Use `RefCell` for interior mutability".to_string(),
                confidence: 0.75,
                auto_fixable: false,
                effort_minutes: Some(30),
            }),
        });
    }

    // Cluster 3: Documentation gaps
    for i in 0..3 {
        reporter.add_finding(Finding {
            id: format!("DOC-{:03}", i + 1),
            category: "DocumentationGap".to_string(),
            severity: Severity::Low,
            location: SourceLocation {
                file: PathBuf::from(format!("src/api/handler_{}.rs", i)),
                line: 10 + i * 5,
                column: 1,
                scope: Some(format!("handle_{}", i)),
            },
            message: format!("Missing documentation for public function {}", i),
            confidence: 0.80,
            cluster_id: None,
            pagerank: None,
            community: None,
            anomaly_score: None,
            fix_suggestion: None,
        });
    }

    // Add an anomalous finding (very different from others)
    reporter.add_finding(Finding {
        id: "ANOMALY-001".to_string(),
        category: "SecurityFlaw".to_string(),
        severity: Severity::Critical,
        location: SourceLocation {
            file: PathBuf::from("src/legacy/compat.rs"),
            line: 9999, // Unusual line number
            column: 1,
            scope: Some("process_untrusted_input".to_string()),
        },
        message: "SQL injection vulnerability detected".to_string(),
        confidence: 0.99,
        cluster_id: None,
        pagerank: None,
        community: None,
        anomaly_score: None,
        fix_suggestion: Some(FixSuggestion {
            description: "Use parameterized queries".to_string(),
            confidence: 0.95,
            auto_fixable: false,
            effort_minutes: Some(60),
        }),
    });

    // Add file dependencies for PageRank analysis
    reporter.add_dependency("src/parser/ast_0.rs", "src/runtime/vm_0.rs");
    reporter.add_dependency("src/parser/ast_1.rs", "src/runtime/vm_0.rs");
    reporter.add_dependency("src/parser/ast_2.rs", "src/runtime/vm_1.rs");
    reporter.add_dependency("src/runtime/vm_0.rs", "src/api/handler_0.rs");
    reporter.add_dependency("src/runtime/vm_1.rs", "src/api/handler_0.rs");
    reporter.add_dependency("src/api/handler_0.rs", "src/legacy/compat.rs");

    // Add metric history for trend analysis
    let coverage_data: Vec<(i64, f64)> = (0..10)
        .map(|i| (i as i64 * 86400, 70.0 + i as f64 * 2.5))
        .collect();
    reporter.add_metric_history("coverage", coverage_data);

    let defect_data: Vec<(i64, f64)> = (0..10)
        .map(|i| (i as i64 * 86400, 50.0 - i as f64 * 3.0))
        .collect();
    reporter.add_metric_history("defects", defect_data);

    let complexity_data: Vec<(i64, f64)> = (0..10)
        .map(|i| (i as i64 * 86400, 15.0 + (i as f64 * 0.5).sin() * 2.0))
        .collect();
    reporter.add_metric_history("complexity", complexity_data);

    // Set quality score
    reporter.set_quality_score(72.5);

    // Add summary metrics
    reporter.add_summary("Files Analyzed", "127");
    reporter.add_summary("Total LOC", "15,234");
    reporter.add_summary("Test Coverage", "78.3%");

    println!("Running data science analysis...");
    println!();

    // Run analysis
    reporter.analyze();

    // Render text report
    println!("{}", "─".repeat(50));
    println!();
    let text_output = reporter.render_text();
    println!("{}", text_output);

    // Also show JSON output summary
    println!();
    println!("JSON Output (truncated):");
    println!("{}", "─".repeat(50));
    let json_output = reporter.render_json();
    // Show first 500 chars
    println!(
        "{}...",
        json_output.chars().take(500).collect::<String>()
    );

    // Show markdown output
    println!();
    println!("Markdown Output:");
    println!("{}", "─".repeat(50));
    let md_output = reporter.render_markdown();
    println!("{}", md_output);

    println!();
    println!("Demo completed successfully!");
    println!();
    println!("Key features demonstrated:");
    println!("  - K-Means clustering grouped {} findings into clusters", reporter.report().findings.len());
    println!("  - PageRank identified central files in dependency graph");
    println!("  - Louvain detected {} code communities", reporter.report().communities.len());
    println!("  - Anomaly detection found {} outliers", reporter.report().anomalies.len());
    println!("  - Trend analysis tracked {} metrics", reporter.report().trends.len());
    println!("  - Generated {} recommendations", reporter.report().recommendations.len());
}
