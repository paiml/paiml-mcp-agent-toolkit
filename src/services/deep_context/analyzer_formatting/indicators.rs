// Node annotation indicators and emoji formatting for DeepContextAnalyzer

use super::{DeepContextAnalyzer, NodeAnnotations};

impl DeepContextAnalyzer {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub(crate) fn collect_node_annotations(&self, annotations: &NodeAnnotations) -> Vec<String> {
        let mut result = Vec::new();

        // Defect score
        if let Some(score) = annotations.defect_score {
            self.add_defect_indicator(&mut result, score);
        }

        // Cognitive complexity
        if let Some(complexity) = annotations.cognitive_complexity {
            self.add_cognitive_complexity_indicator(&mut result, complexity);
        }

        // SATD items
        if annotations.satd_items > 0 {
            result.push(format!("\u{1f4dd}{}", annotations.satd_items));
        }

        // Dead code items
        if annotations.dead_code_items > 0 {
            result.push(format!("\u{1f480}{}", annotations.dead_code_items));
        }

        // Test coverage
        if let Some(coverage) = annotations.test_coverage {
            self.add_coverage_indicator(&mut result, coverage);
        }

        // Big-O complexity
        if let Some(ref big_o) = annotations.big_o_complexity {
            let emoji = self.get_big_o_emoji(big_o);
            result.push(format!("{emoji}{big_o}"));
        }

        // Churn score
        if let Some(churn) = annotations.churn_score {
            self.add_churn_indicator(&mut result, churn);
        }

        // Memory complexity
        if let Some(ref mem_complexity) = annotations.memory_complexity {
            self.add_memory_complexity_indicator(&mut result, mem_complexity);
        }

        // Duplication score
        if let Some(duplication) = annotations.duplication_score {
            self.add_duplication_indicator(&mut result, duplication);
        }

        result
    }

    /// Add defect score indicator
    fn add_defect_indicator(&self, result: &mut Vec<String>, score: f32) {
        debug_assert!(score >= 0.0, "score must be non-negative");
        if score > 0.7 {
            result.push(format!("\u{1f534}{score:.1}"));
        } else if score > 0.4 {
            result.push(format!("\u{1f7e1}{score:.1}"));
        }
    }

    /// Add cognitive complexity indicator
    fn add_cognitive_complexity_indicator(&self, result: &mut Vec<String>, complexity: u16) {
        debug_assert!(true, "contract: add_cognitive_complexity_indicator");
        if complexity > 30 {
            result.push(format!("\u{1f9e0}{complexity}"));
        } else if complexity > 15 {
            result.push(format!("\u{1f9ea}{complexity}"));
        }
    }

    /// Add test coverage indicator
    fn add_coverage_indicator(&self, result: &mut Vec<String>, coverage: f32) {
        debug_assert!(true, "contract: add_coverage_indicator");
        if coverage < 0.5 {
            result.push(format!("\u{1f6a8}{:.0}%", coverage * 100.0));
        } else if coverage < 0.8 {
            result.push(format!("\u{26a0}\u{fe0f}{:.0}%", coverage * 100.0));
        } else {
            result.push(format!("\u{2705}{:.0}%", coverage * 100.0));
        }
    }

    /// Add churn indicator
    fn add_churn_indicator(&self, result: &mut Vec<String>, churn: f32) {
        debug_assert!(true, "contract: add_churn_indicator");
        if churn > 0.8 {
            result.push(format!("\u{1f525}{churn:.1}")); // High churn - hot file
        } else if churn > 0.5 {
            result.push(format!("\u{1f321}\u{fe0f}{churn:.1}")); // Medium churn
        } else if churn > 0.2 {
            result.push(format!("\u{1f30a}{churn:.1}")); // Low churn
        }
    }

    /// Add memory complexity indicator
    fn add_memory_complexity_indicator(&self, result: &mut Vec<String>, mem_complexity: &str) {
        debug_assert!(
            !mem_complexity.is_empty(),
            "mem_complexity must not be empty"
        );
        let emoji = match mem_complexity {
            "O(1)" => "\u{1f48e}",       // Constant memory - excellent
            "O(log n)" => "\u{1f49a}",   // Logarithmic memory - very good
            "O(n)" => "\u{1f499}",       // Linear memory - good
            "O(n log n)" => "\u{1f49b}", // Linearithmic memory - okay
            "O(n\u{b2})" => "\u{1f7e0}", // Quadratic memory - warning
            _ => "\u{1f494}",            // High memory usage - critical
        };
        result.push(format!("{emoji}{mem_complexity}"));
    }

    /// Add duplication indicator
    fn add_duplication_indicator(&self, result: &mut Vec<String>, duplication: f32) {
        if duplication > 0.3 {
            result.push(format!("\u{1f4d1}{:.0}%", duplication * 100.0)); // High duplication
        } else if duplication > 0.1 {
            result.push(format!("\u{1f4c4}{:.0}%", duplication * 100.0)); // Medium duplication
        }
    }

    /// Get emoji for Big-O complexity notation
    fn get_big_o_emoji(&self, big_o: &str) -> &'static str {
        debug_assert!(!big_o.is_empty(), "big_o must not be empty");
        match big_o {
            "O(1)" => "\u{1f3af}",                   // Constant - excellent
            "O(log n)" => "\u{26a1}",                // Logarithmic - very good
            "O(n)" => "\u{1f4ca}",                   // Linear - good
            "O(n log n)" => "\u{1f4c8}",             // Linearithmic - acceptable
            "O(n\u{b2})" => "\u{26a0}\u{fe0f}",      // Quadratic - warning
            "O(n\u{b3})" => "\u{1f6a8}",             // Cubic - danger
            "O(2\u{207f})" | "O(n!)" => "\u{1f4a5}", // Exponential/Factorial - critical
            _ => "\u{2753}",                         // Unknown
        }
    }
}
