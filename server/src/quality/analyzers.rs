pub use super::complexity::ComplexityAnalyzer;
pub use super::efficiency::EfficiencyAnalyzer;
pub use super::entropy::EntropyCalculator;
pub use super::satd::SatdDetector;

use super::gate::QualityMetrics;

pub trait QualityAnalyzer: Send + Sync {
    fn analyze(&self, ast: &syn::File) -> QualityMetrics;
    fn name(&self) -> &'static str;
}

impl QualityAnalyzer for ComplexityAnalyzer {
    fn analyze(&self, ast: &syn::File) -> QualityMetrics {
        QualityMetrics {
            cyclomatic_complexity: self.calculate_cyclomatic(ast),
            cognitive_complexity: self.calculate_cognitive(ast),
            nesting_depth: 0, // Would need additional implementation
            satd_count: 0,
            entropy: 0.0,
            efficiency: "O(1)".to_string(),
        }
    }

    fn name(&self) -> &'static str {
        "ComplexityAnalyzer"
    }
}

impl QualityAnalyzer for EfficiencyAnalyzer {
    fn analyze(&self, ast: &syn::File) -> QualityMetrics {
        QualityMetrics {
            cyclomatic_complexity: 0,
            cognitive_complexity: 0,
            nesting_depth: 0,
            satd_count: 0,
            entropy: 0.0,
            efficiency: self.analyze(ast),
        }
    }

    fn name(&self) -> &'static str {
        "EfficiencyAnalyzer"
    }
}
