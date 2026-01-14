use anyhow::Result;
use std::collections::{HashMap, HashSet};
use tree_sitter::{Node, Tree};
use crate::tdg::{Language, MetricCategory, PenaltyTracker, TdgConfig};
use crate::tdg::language::{LanguageRules, NamingStyle};
use super::{Scorer, walk_tree, get_node_text};

pub struct ConsistencyAnalyzer;

impl ConsistencyAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    fn check_naming_consistency(&self, root: Node, source: &str, rules: &LanguageRules) -> u32 {
        let mut violations = 0;
        
        walk_tree(root, |node| {
            match node.kind() {
                "identifier" => {
                    if let Some(parent) = node.parent() {
                        let naming_style = match parent.kind() {
                            "function_item" | "function_declaration" | "function_definition" => &rules.function_style,
                            "struct_item" | "class_declaration" | "class_definition" | "type_declaration" => &rules.type_style,
                            "const_item" | "static_item" | "const_declaration" => &rules.constant_style,
                            "parameter" | "variable_declaration" | "let_declaration" => &rules.variable_style,
                            _ => return,
                        };
                        
                        let name = get_node_text(node, source);
                        if !naming_style.matches(name) {
                            violations += 1;
                        }
                    }
                }
                _ => {}
            }
        });
        
        violations
    }
    
    fn check_import_organization(&self, root: Node, source: &str, language: Language) -> u32 {
        let mut issues = 0;
        let mut imports = Vec::new();
        
        walk_tree(root, |node| {
            match node.kind() {
                "use_declaration" | "import_statement" | "import" => {
                    let text = get_node_text(node, source);
                    imports.push((text.to_string(), node.start_byte()));
                }
                _ => {}
            }
        });
        
        if imports.len() < 2 {
            return 0;
        }
        
        match language {
            Language::Rust => {
                issues += self.check_rust_import_order(&imports);
            }
            Language::Python => {
                issues += self.check_python_import_order(&imports);
            }
            Language::JavaScript | Language::TypeScript => {
                issues += self.check_js_import_order(&imports);
            }
            Language::Go => {
                issues += self.check_go_import_order(&imports);
            }
            _ => {}
        }
        
        issues
    }
    
    fn check_rust_import_order(&self, imports: &[(String, usize)]) -> u32 {
        let mut issues = 0;
        let mut prev_category = -1;
        
        for (import, _) in imports {
            let category = if import.starts_with("use std::") {
                0
            } else if import.starts_with("use crate::") {
                2
            } else if import.starts_with("use super::") || import.starts_with("use self::") {
                3
            } else {
                1
            };
            
            if category < prev_category {
                issues += 1;
            }
            prev_category = category;
        }
        
        issues
    }
    
    fn check_python_import_order(&self, imports: &[(String, usize)]) -> u32 {
        let mut issues = 0;
        let mut in_stdlib = true;
        let mut in_third_party = false;
        
        for (import, _) in imports {
            let is_stdlib = self.is_python_stdlib_import(import);
            let is_relative = import.starts_with("from .");
            
            if is_relative && (in_stdlib || in_third_party) {
                issues += 1;
            } else if !is_stdlib && in_stdlib && !in_third_party {
                in_stdlib = false;
                in_third_party = true;
            } else if is_stdlib && (in_third_party || !in_stdlib) {
                issues += 1;
            }
        }
        
        issues
    }
    
    fn is_python_stdlib_import(&self, import: &str) -> bool {
        let stdlib_modules = [
            "os", "sys", "json", "re", "datetime", "collections",
            "itertools", "functools", "math", "random", "urllib",
            "http", "pathlib", "typing", "dataclasses", "enum",
        ];
        
        let module_name = if let Some(from_pos) = import.find("from ") {
            &import[from_pos + 5..].split_whitespace().next().unwrap_or("")
        } else if let Some(import_pos) = import.find("import ") {
            &import[import_pos + 7..].split('.').next().unwrap_or("")
        } else {
            ""
        };
        
        stdlib_modules.contains(&module_name)
    }
    
    fn check_js_import_order(&self, _imports: &[(String, usize)]) -> u32 {
        0
    }
    
    fn check_go_import_order(&self, imports: &[(String, usize)]) -> u32 {
        let mut issues = 0;
        let mut in_stdlib = true;
        
        for (import, _) in imports {
            let is_third_party = import.contains('.') && !import.starts_with("import \"");
            
            if is_third_party && in_stdlib {
                in_stdlib = false;
            } else if !is_third_party && !in_stdlib {
                issues += 1;
            }
        }
        
        issues
    }
    
    fn analyze_pattern_consistency(&self, root: Node, source: &str) -> f32 {
        let patterns = self.extract_patterns(root, source);
        
        let error_consistency = self.error_handling_consistency(&patterns);
        let null_consistency = self.null_check_consistency(&patterns);
        let loop_consistency = self.loop_style_consistency(&patterns);
        let conditional_consistency = self.conditional_style_consistency(&patterns);
        
        let scores = vec![error_consistency, null_consistency, loop_consistency, conditional_consistency];
        scores.iter().sum::<f32>() / scores.len() as f32
    }
    
    fn extract_patterns(&self, root: Node, source: &str) -> CodePatterns {
        let mut patterns = CodePatterns::new();
        
        walk_tree(root, |node| {
            match node.kind() {
                "if_statement" | "if_expression" => {
                    let text = get_node_text(node, source);
                    if text.contains("null") || text.contains("None") || text.contains("nil") {
                        patterns.null_checks.push(text.to_string());
                    }
                    patterns.conditionals.push(text.to_string());
                }
                "match_expression" | "switch_statement" => {
                    patterns.error_handling.push(get_node_text(node, source).to_string());
                }
                "try_expression" | "try_statement" => {
                    patterns.error_handling.push(get_node_text(node, source).to_string());
                }
                "while_statement" | "for_statement" | "while_expression" | "for_expression" => {
                    patterns.loops.push(get_node_text(node, source).to_string());
                }
                "call_expression" => {
                    let text = get_node_text(node, source);
                    if text.contains("unwrap") || text.contains("expect") {
                        patterns.error_handling.push(text.to_string());
                    }
                }
                _ => {}
            }
        });
        
        patterns
    }
    
    fn error_handling_consistency(&self, patterns: &CodePatterns) -> f32 {
        if patterns.error_handling.is_empty() {
            return 1.0;
        }
        
        let unwrap_count = patterns.error_handling.iter()
            .filter(|p| p.contains("unwrap"))
            .count();
        let match_count = patterns.error_handling.iter()
            .filter(|p| p.contains("match") || p.contains("switch"))
            .count();
        let try_count = patterns.error_handling.iter()
            .filter(|p| p.contains("try"))
            .count();
        
        let total = unwrap_count + match_count + try_count;
        if total == 0 {
            return 1.0;
        }
        
        let dominant = unwrap_count.max(match_count).max(try_count);
        dominant as f32 / total as f32
    }
    
    fn null_check_consistency(&self, patterns: &CodePatterns) -> f32 {
        if patterns.null_checks.is_empty() {
            return 1.0;
        }
        
        let explicit_checks = patterns.null_checks.iter()
            .filter(|p| p.contains("== null") || p.contains("is None") || p.contains("== nil"))
            .count();
        let pattern_checks = patterns.null_checks.iter()
            .filter(|p| p.contains("if let") || p.contains("match"))
            .count();
        
        let total = explicit_checks + pattern_checks;
        if total == 0 {
            return 1.0;
        }
        
        let dominant = explicit_checks.max(pattern_checks);
        dominant as f32 / total as f32
    }
    
    fn loop_style_consistency(&self, patterns: &CodePatterns) -> f32 {
        if patterns.loops.is_empty() {
            return 1.0;
        }
        
        let for_loops = patterns.loops.iter()
            .filter(|p| p.starts_with("for"))
            .count();
        let while_loops = patterns.loops.iter()
            .filter(|p| p.starts_with("while"))
            .count();
        let iterator_loops = patterns.loops.iter()
            .filter(|p| p.contains(".iter()") || p.contains(".map(") || p.contains(".filter("))
            .count();
        
        let total = for_loops + while_loops + iterator_loops;
        if total == 0 {
            return 1.0;
        }
        
        let dominant = for_loops.max(while_loops).max(iterator_loops);
        dominant as f32 / total as f32
    }
    
    fn conditional_style_consistency(&self, patterns: &CodePatterns) -> f32 {
        if patterns.conditionals.is_empty() {
            return 1.0;
        }
        
        let if_else = patterns.conditionals.iter()
            .filter(|p| p.contains("if") && p.contains("else"))
            .count();
        let match_patterns = patterns.conditionals.iter()
            .filter(|p| p.contains("match"))
            .count();
        let ternary = patterns.conditionals.iter()
            .filter(|p| p.contains('?') && p.contains(':'))
            .count();
        
        let total = if_else + match_patterns + ternary;
        if total == 0 {
            return 1.0;
        }
        
        let dominant = if_else.max(match_patterns).max(ternary);
        dominant as f32 / total as f32
    }
}

impl Scorer for ConsistencyAnalyzer {
    fn score(&self, tree: &Tree, source: &str, language: Language, config: &TdgConfig, tracker: &mut PenaltyTracker) -> Result<f32> {
        let mut points = config.weights.consistency;
        let root = tree.root_node();
        let rules = LanguageRules::for_language(language);
        
        let naming_violations = self.check_naming_consistency(root, source, &rules);
        let naming_penalty = (naming_violations as f32 * 0.2).min(4.0);
        if naming_penalty > 0.0 {
            if let Some(applied) = tracker.apply(
                format!("naming_violations_{}", naming_violations),
                MetricCategory::Consistency,
                naming_penalty,
                format!("Naming convention violations: {}", naming_violations)
            ) {
                points -= applied;
            }
        }
        
        let import_issues = self.check_import_organization(root, source, language);
        let import_penalty = (import_issues as f32 * 0.3).min(2.0);
        if import_penalty > 0.0 {
            if let Some(applied) = tracker.apply(
                format!("import_issues_{}", import_issues),
                MetricCategory::Consistency,
                import_penalty,
                format!("Import organization issues: {}", import_issues)
            ) {
                points -= applied;
            }
        }
        
        let pattern_score = self.analyze_pattern_consistency(root, source);
        let pattern_penalty = ((1.0 - pattern_score) * 4.0).min(4.0);
        if pattern_penalty > 0.5 {
            if let Some(applied) = tracker.apply(
                format!("pattern_inconsistency_{:.2}", pattern_score),
                MetricCategory::Consistency,
                pattern_penalty,
                format!("Pattern inconsistency: {:.1}% consistent", pattern_score * 100.0)
            ) {
                points -= applied;
            }
        }
        
        Ok(points.max(0.0))
    }
    
    fn category(&self) -> MetricCategory {
        MetricCategory::Consistency
    }
}

#[derive(Debug)]
struct CodePatterns {
    error_handling: Vec<String>,
    null_checks: Vec<String>,
    loops: Vec<String>,
    conditionals: Vec<String>,
}

impl CodePatterns {
    fn new() -> Self {
        Self {
            error_handling: Vec::new(),
            null_checks: Vec::new(),
            loops: Vec::new(),
            conditionals: Vec::new(),
        }
    }
}

impl LanguageRules {
    pub fn for_language(language: Language) -> Self {
        match language {
            Language::Rust => Self::rust_rules(),
            Language::Python => Self::python_rules(),
            Language::JavaScript => Self::javascript_rules(),
            Language::TypeScript => Self::typescript_rules(),
            Language::Go => Self::go_rules(),
            _ => Self::rust_rules(), // Default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tree_sitter::Parser;
    
    fn parse_rust(source: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::language()).unwrap();
        parser.parse(source, None).unwrap()
    }
    
    #[test]
    fn test_naming_consistency() {
        let source = r#"
            pub fn snake_case_function() {}
            pub fn CamelCaseFunction() {}  // Violation
            
            pub struct PascalCaseStruct;
            pub struct snake_case_struct;  // Violation
            
            const SCREAMING_SNAKE: i32 = 42;
            const lowercase_const: i32 = 24;  // Violation
        "#;
        
        let tree = parse_rust(source);
        let analyzer = ConsistencyAnalyzer::new();
        let rules = LanguageRules::rust_rules();
        
        let violations = analyzer.check_naming_consistency(tree.root_node(), source, &rules);
        assert!(violations > 0);
    }
    
    #[test]
    fn test_import_organization() {
        let source = r#"
            use crate::local::Module;
            use std::collections::HashMap;  // Should come first
            use external::crate::Thing;
        "#;
        
        let tree = parse_rust(source);
        let analyzer = ConsistencyAnalyzer::new();
        
        let issues = analyzer.check_import_organization(tree.root_node(), source, Language::Rust);
        assert!(issues > 0);
    }
    
    #[test]
    fn test_pattern_consistency() {
        let source = r#"
            fn inconsistent_patterns() {
                // Mixed error handling
                let result1 = something().unwrap();
                let result2 = match something_else() {
                    Ok(val) => val,
                    Err(_) => return,
                };
                
                // Mixed null checking
                if value.is_some() {
                    // ...
                }
                if other_value == None {
                    // ...
                }
            }
        "#;
        
        let tree = parse_rust(source);
        let analyzer = ConsistencyAnalyzer::new();
        
        let consistency = analyzer.analyze_pattern_consistency(tree.root_node(), source);
        assert!(consistency < 1.0);
    }
}