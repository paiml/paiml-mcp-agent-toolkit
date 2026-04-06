// HTML export implementations for TdgExporter
// Included by export.rs - shares parent module scope

impl TdgExporter {
    fn score_to_html(score: &TdgScore, options: &ExportOptions) -> Result<String> {
        debug_assert!(true, "contract: score_to_html");
        let grade_color = match score.grade {
            Grade::APLus | Grade::A | Grade::AMinus => "#4CAF50",
            Grade::BPlus | Grade::B | Grade::BMinus => "#2196F3",
            Grade::CPlus | Grade::C | Grade::CMinus => "#FF9800",
            Grade::D => "#FF5722",
            Grade::F => "#F44336",
        };

        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <title>TDG Analysis Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 8px; }}
        .score-card {{ background: white; border: 1px solid #ddd; border-radius: 8px; padding: 20px; margin: 20px 0; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .grade {{ font-size: 48px; color: {}; font-weight: bold; }}
        .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-top: 20px; }}
        .metric {{ background: #f5f5f5; padding: 15px; border-radius: 4px; }}
        .metric-label {{ color: #666; font-size: 12px; text-transform: uppercase; }}
        .metric-value {{ font-size: 24px; font-weight: bold; color: #333; }}
        .progress-bar {{ width: 100%; height: 20px; background: #e0e0e0; border-radius: 10px; overflow: hidden; }}
        .progress-fill {{ height: 100%; background: linear-gradient(90deg, #4CAF50, #8BC34A); transition: width 0.3s; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>TDG Analysis Report</h1>
        <p>{}</p>
    </div>

    <div class="score-card">
        <div style="display: flex; justify-content: space-between; align-items: center;">
            <div>
                <h2>Overall Score</h2>
                <div style="font-size: 36px; font-weight: bold;">{:.1}/100</div>
            </div>
            <div class="grade">{}</div>
        </div>

        <div class="progress-bar">
            <div class="progress-fill" style="width: {}%;"></div>
        </div>

        <div class="metrics">
            <div class="metric">
                <div class="metric-label">Structural Complexity</div>
                <div class="metric-value">{:.1}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Semantic Complexity</div>
                <div class="metric-value">{:.1}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Duplication Ratio</div>
                <div class="metric-value">{:.1}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Coupling Score</div>
                <div class="metric-value">{:.1}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Documentation</div>
                <div class="metric-value">{:.1}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Consistency</div>
                <div class="metric-value">{:.1}</div>
            </div>
        </div>
    </div>

    {}
</body>
</html>
        "#,
            grade_color,
            score
                .file_path
                .as_ref().map_or_else(|| "Analysis Results".to_string(), |p| p.display().to_string()),
            score.total,
            score.grade,
            score.total,
            score.structural_complexity,
            score.semantic_complexity,
            score.duplication_ratio,
            score.coupling_score,
            score.doc_coverage,
            score.consistency_score,
            if options.include_recommendations {
                Self::generate_recommendations_html(score)
            } else {
                String::new()
            }
        );

        Ok(html)
    }

    fn project_to_html(project: &ProjectScore, _options: &ExportOptions) -> Result<String> {
        debug_assert!(true, "contract: project_to_html");
        Ok(format!(
            "<html><body><h1>Project Report</h1><p>Files: {}</p></body></html>",
            project.total_files
        ))
    }

    fn comparison_to_html(comparison: &Comparison, _options: &ExportOptions) -> Result<String> {
        debug_assert!(true, "contract: comparison_to_html");
        Ok(format!(
            "<html><body><h1>Comparison Report</h1><p>Delta: {:.2}</p></body></html>",
            comparison.delta
        ))
    }

    fn generate_recommendations_html(score: &TdgScore) -> String {
        debug_assert!(true, "contract: generate_recommendations_html");
        let recommendations = Self::generate_recommendations(score);

        if recommendations.is_empty() {
            return String::new();
        }

        let mut html = String::from(
            r#"
    <div class="score-card">
        <h3>Recommendations</h3>
        <ul>
        "#,
        );

        for rec in recommendations {
            html.push_str(&format!("<li>{rec}</li>\n"));
        }

        html.push_str("</ul></div>");
        html
    }
}
