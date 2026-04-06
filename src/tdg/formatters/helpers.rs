#![cfg_attr(coverage_nightly, coverage(off))]
use super::super::Grade;

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn progress_bar(value: f32, max_value: f32, width: usize) -> String {
    let ratio = (value / max_value).clamp(0.0, 1.0);
    let filled = (ratio * width as f32) as usize;
    let empty = width - filled;

    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn grade_delta(from: Grade, to: Grade) -> String {
    let from_val = grade_to_number(from);
    let to_val = grade_to_number(to);
    let delta = to_val - from_val;

    if delta > 0 {
        format!("↑{delta}")
    } else if delta < 0 {
        format!("↓{}", delta.abs())
    } else {
        "=".to_string()
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn grade_to_number(grade: Grade) -> i32 {
    match grade {
        Grade::APLus => 11,
        Grade::A => 10,
        Grade::AMinus => 9,
        Grade::BPlus => 8,
        Grade::B => 7,
        Grade::BMinus => 6,
        Grade::CPlus => 5,
        Grade::C => 4,
        Grade::CMinus => 3,
        Grade::D => 2,
        Grade::F => 1,
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn format_metric_name(metric: &crate::tdg::MetricCategory) -> &'static str {
    match metric {
        crate::tdg::MetricCategory::StructuralComplexity => "Structural Complexity",
        crate::tdg::MetricCategory::SemanticComplexity => "Semantic Complexity",
        crate::tdg::MetricCategory::Duplication => "Code Duplication",
        crate::tdg::MetricCategory::Coupling => "Coupling",
        crate::tdg::MetricCategory::Documentation => "Documentation",
        crate::tdg::MetricCategory::Consistency => "Consistency",
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn grade_description(grade: Grade) -> &'static str {
    match grade {
        Grade::APLus => {
            "**A+** (95-100): Exceptional code quality. Production-ready with excellent practices."
        }
        Grade::A => "**A** (90-94): High quality code with minor room for improvement.",
        Grade::AMinus => "**A-** (85-89): Good code quality with some areas for enhancement.",
        Grade::BPlus => "**B+** (80-84): Above average code that meets most quality standards.",
        Grade::B => "**B** (75-79): Average code quality with moderate technical debt.",
        Grade::BMinus => "**B-** (70-74): Below average with noticeable issues to address.",
        Grade::CPlus => "**C+** (65-69): Poor code quality requiring significant attention.",
        Grade::C => "**C** (60-64): Very poor quality with major issues throughout.",
        Grade::CMinus => "**C-** (55-59): Problematic code that needs extensive refactoring.",
        Grade::D => "**D** (50-54): Severely problematic code with critical issues.",
        Grade::F => "**F** (0-49): Failing grade indicating code that is not production-ready.",
    }
}
