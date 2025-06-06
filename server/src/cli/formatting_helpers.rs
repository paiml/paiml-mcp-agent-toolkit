use crate::models::project_meta::{BuildInfo, ProjectOverview};
use crate::services::deep_context::DeepContext;
use std::fmt::Write;

/// Format executive summary section
pub fn format_executive_summary(context: &DeepContext) -> String {
    let mut output = String::new();

    output.push_str("## Executive Summary\n\n");
    let _ = writeln!(&mut output, "Generated: {}", context.metadata.generated_at);
    let _ = writeln!(&mut output, "Version: {}", context.metadata.tool_version);
    let _ = writeln!(
        &mut output,
        "Analysis Time: {:.2}s",
        context.metadata.analysis_duration.as_secs_f64()
    );
    let _ = writeln!(
        &mut output,
        "Cache Hit Rate: {:.1}%",
        context.metadata.cache_stats.hit_rate * 100.0
    );

    output
}

/// Format quality scorecard section
pub fn format_quality_scorecard(context: &DeepContext) -> String {
    let mut output = String::new();

    output.push_str("\n## Quality Scorecard\n\n");

    let health_emoji = match context.quality_scorecard.overall_health {
        h if h >= 80.0 => "✅",
        h if h >= 60.0 => "⚠️",
        _ => "❌",
    };

    let _ = writeln!(
        &mut output,
        "- **Overall Health**: {} ({:.1}/100)",
        health_emoji, context.quality_scorecard.overall_health
    );
    let _ = writeln!(
        &mut output,
        "- **Maintainability Index**: {:.1}",
        context.quality_scorecard.maintainability_index
    );
    let _ = writeln!(
        &mut output,
        "- **Technical Debt**: {:.1} hours estimated",
        context.quality_scorecard.technical_debt_hours
    );

    output
}

/// Format project overview from README
pub fn format_project_overview(overview: &ProjectOverview) -> String {
    let mut output = String::new();

    output.push_str("### README.md (Documentation)\n\n");

    if !overview.compressed_description.is_empty() {
        output.push_str(&overview.compressed_description);
        output.push_str("\n\n");
    }

    if !overview.key_features.is_empty() {
        output.push_str("**Key Features:**\n");
        for feature in &overview.key_features {
            let _ = writeln!(&mut output, "- {feature}");
        }
        output.push('\n');
    }

    if let Some(arch) = &overview.architecture_summary {
        output.push_str("**Architecture:**\n");
        output.push_str(arch);
        output.push_str("\n\n");
    }

    output
}

/// Format build info from Makefile
pub fn format_build_info(build_info: &BuildInfo) -> String {
    let mut output = String::new();

    output.push_str("### Makefile (Build Configuration)\n\n");
    let _ = writeln!(&mut output, "**Toolchain:** {}", build_info.toolchain);

    if !build_info.targets.is_empty() {
        output.push_str("**Key Targets:**\n");
        for target in &build_info.targets {
            let _ = writeln!(&mut output, "- {target}");
        }
        output.push('\n');
    }

    if !build_info.dependencies.is_empty() {
        output.push_str("**Dependencies:**\n");
        for dep in &build_info.dependencies {
            let _ = writeln!(&mut output, "- {dep}");
        }
        output.push('\n');
    }

    output
}

/// Format defect summary section
pub fn format_defect_summary(context: &DeepContext) -> String {
    let mut output = String::new();

    if context.defect_summary.total_defects > 0 {
        output.push_str("\n## Defect Summary\n\n");
        let _ = writeln!(
            &mut output,
            "- **Total Defects Found**: {}",
            context.defect_summary.total_defects
        );
        let _ = writeln!(
            &mut output,
            "- **Defect Density**: {:.2}",
            context.defect_summary.defect_density
        );

        // Show severity breakdown if available
        if !context.defect_summary.by_severity.is_empty() {
            output.push_str("\n**By Severity:**\n");
            for (severity, count) in &context.defect_summary.by_severity {
                let _ = writeln!(&mut output, "- {severity}: {count}");
            }
        }

        // Show type breakdown if available
        if !context.defect_summary.by_type.is_empty() {
            output.push_str("\n**By Type:**\n");
            for (defect_type, count) in &context.defect_summary.by_type {
                let _ = writeln!(&mut output, "- {defect_type}: {count}");
            }
        }
    }

    output
}

/// Format recommendations section
pub fn format_recommendations(context: &DeepContext) -> String {
    let mut output = String::new();

    if !context.recommendations.is_empty() {
        output.push_str("\n## Recommendations\n\n");

        for (i, rec) in context.recommendations.iter().enumerate() {
            let _ = writeln!(
                &mut output,
                "{}. **{}** (Priority: {:?})",
                i + 1,
                rec.title,
                rec.priority
            );
            let _ = writeln!(&mut output, "   - Impact: {:?}", rec.impact);
            let _ = writeln!(
                &mut output,
                "   - Effort: {:.1} hours",
                rec.estimated_effort.as_secs_f64() / 3600.0
            );
            let _ = writeln!(&mut output, "   - {}", rec.description);
            output.push('\n');
        }
    }

    output
}
