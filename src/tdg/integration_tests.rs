#[cfg(test)]
mod tests {
    use super::*;
    use crate::tdg::{TdgAnalyzer, Language, Grade, TdgScore};
    use tempfile::{tempdir, NamedTempFile};
    use std::io::Write;
    use std::fs;
    
    #[tokio::test]
    async fn test_analyze_simple_rust_file() {
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(
            temp_file,
            r#"
/// A simple function
pub fn add(a: i32, b: i32) -> i32 {{
    a + b
}}
"#
        ).unwrap();
        
        let analyzer = TdgAnalyzer::new().unwrap();
        let score = analyzer.analyze_file(temp_file.path()).unwrap();
        
        assert_eq!(score.language, Language::Rust);
        assert!(score.total > 0.0);
        assert!(score.total <= 100.0);
        assert!(score.grade != Grade::F);
        assert!(score.confidence > 0.0);
    }
    
    #[tokio::test]
    async fn test_analyze_complex_file_with_penalties() {
        let mut temp_file = NamedTempFile::with_suffix(".rs").unwrap();
        writeln!(
            temp_file,
            r#"
fn complex_function(x: i32) -> i32 {{
    if x > 0 {{
        if x > 10 {{
            if x > 20 {{
                if x > 30 {{
                    if x > 40 {{
                        return x * 2;
                    }}
                }}
            }}
        }}
    }}
    x
}}

fn duplicate_line() {{
    let a = 1;
    let b = 2;
    let c = 3;
    let a = 1;
    let b = 2;
    let c = 3;
}}
"#
        ).unwrap();
        
        let analyzer = TdgAnalyzer::new().unwrap();
        let score = analyzer.analyze_file(temp_file.path()).unwrap();
        
        assert!(!score.penalties_applied.is_empty(), "Expected penalties for complex code");
        assert!(score.structural_complexity < 25.0, "Expected reduced structural complexity score");
        assert!(score.semantic_complexity < 25.0, "Expected reduced semantic complexity score");
    }
    
    #[tokio::test]
    async fn test_analyze_directory() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path();
        
        // Create multiple files
        let file1 = dir_path.join("simple.rs");
        fs::write(&file1, r#"
/// Simple function
pub fn simple() -> i32 {
    42
}
"#).unwrap();
        
        let file2 = dir_path.join("complex.rs");
        fs::write(&file2, r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            x * 2
        } else {
            x
        }
    } else {
        0
    }
}
"#).unwrap();
        
        let analyzer = TdgAnalyzer::new().unwrap();
        let project_score = analyzer.analyze_project(dir_path).unwrap();
        
        assert_eq!(project_score.total_files, 2);
        assert!(project_score.average_score > 0.0);
        assert!(project_score.files.len() == 2);
        assert!(project_score.language_distribution.contains_key(&Language::Rust));
    }
    
    #[tokio::test]
    async fn test_language_detection() {
        let test_cases = vec![
            ("test.rs", Language::Rust),
            ("test.py", Language::Python),
            ("test.js", Language::JavaScript),
            ("test.ts", Language::TypeScript),
            ("test.go", Language::Go),
            ("test.java", Language::Java),
            ("test.c", Language::C),
            ("test.cpp", Language::Cpp),
            ("test.rb", Language::Ruby),
            ("test.swift", Language::Swift),
            ("test.kt", Language::Kotlin),
            ("test.unknown", Language::Unknown),
        ];
        
        for (filename, expected_lang) in test_cases {
            let detected = Language::from_extension(std::path::Path::new(filename));
            assert_eq!(detected, expected_lang, "Failed for {}", filename);
        }
    }
    
    #[tokio::test]
    async fn test_grade_calculation() {
        let test_cases = vec![
            (95.0, Grade::APlus),
            (92.0, Grade::A),
            (88.0, Grade::AMinus),
            (85.0, Grade::BPlus),
            (82.0, Grade::B),
            (78.0, Grade::BMinus),
            (75.0, Grade::CPlus),
            (72.0, Grade::C),
            (68.0, Grade::CMinus),
            (65.0, Grade::D),
            (50.0, Grade::F),
        ];
        
        for (score, expected_grade) in test_cases {
            let calculated = Grade::from_score(score);
            assert_eq!(calculated, expected_grade, "Failed for score {}", score);
        }
    }
    
    #[tokio::test]
    async fn test_file_comparison() {
        let temp_dir = tempdir().unwrap();
        let dir_path = temp_dir.path();
        
        let file1 = dir_path.join("good.rs");
        fs::write(&file1, r#"
/// Well documented function
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#).unwrap();
        
        let file2 = dir_path.join("bad.rs");
        fs::write(&file2, r#"
fn complex(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            if x > 20 {
                if x > 30 {
                    x * 2
                } else {
                    x
                }
            } else {
                x
            }
        } else {
            x
        }
    } else {
        0
    }
}
"#).unwrap();
        
        let analyzer = TdgAnalyzer::new().unwrap();
        let comparison = analyzer.compare(&file1, &file2).unwrap();
        
        assert!(comparison.score1.total > comparison.score2.total, "Good file should have higher score");
        assert!(!comparison.improvements.is_empty() || !comparison.regressions.is_empty(), "Should have differences");
    }
    
    #[tokio::test]
    async fn test_documentation_scoring() {
        let test_cases = vec![
            (
                Language::Rust,
                r#"
/// This is well documented
/// Multiple lines of documentation
pub fn documented() -> i32 {
    42
}
"#,
                "documented Rust code"
            ),
            (
                Language::Python,
                r#"
def documented():
    """
    This is well documented
    Multiple lines of documentation
    """
    return 42
"#,
                "documented Python code"
            ),
            (
                Language::Rust,
                r#"
pub fn undocumented() -> i32 {
    42
}
"#,
                "undocumented Rust code"
            ),
        ];
        
        let analyzer = TdgAnalyzer::new().unwrap();
        
        for (language, code, description) in test_cases {
            let score = analyzer.analyze_source(code, language, None).unwrap();
            
            if description.contains("documented") && !description.contains("undocumented") {
                assert!(score.doc_coverage > 0.0, "Expected some documentation score for {}", description);
            }
            
            assert!(score.total > 0.0, "Expected positive score for {}", description);
        }
    }
    
    #[tokio::test]
    async fn test_consistency_scoring() {
        let consistent_code = r#"
fn function_one() -> i32 {
    let variable_one = 1;
    let variable_two = 2;
    variable_one + variable_two
}

fn function_two() -> i32 {
    let variable_three = 3;
    let variable_four = 4;
    variable_three + variable_four
}
"#;
        
        let inconsistent_code = r#"
fn function_one() -> i32 {
	let VariableOne = 1;
    let variable_two = 2;
    VariableOne + variable_two
}

fn FunctionTwo() -> i32 {
    let VARIABLE_THREE = 3;
	let variable_four = 4;
    VARIABLE_THREE + variable_four
}
"#;
        
        let analyzer = TdgAnalyzer::new().unwrap();
        
        let consistent_score = analyzer.analyze_source(consistent_code, Language::Rust, None).unwrap();
        let inconsistent_score = analyzer.analyze_source(inconsistent_code, Language::Rust, None).unwrap();
        
        assert!(consistent_score.consistency_score >= inconsistent_score.consistency_score,
               "Consistent code should have better or equal consistency score");
    }
    
    #[tokio::test]
    async fn test_coupling_analysis() {
        let high_coupling = r#"
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::VecDeque;
use std::collections::LinkedList;
use std::collections::BinaryHeap;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::thread;
use std::sync::Mutex;
use std::sync::Arc;
use std::path::Path;
use std::path::PathBuf;
use std::env;
use std::process::Command;
use tokio::fs;
use tokio::net;
use anyhow::Result;

fn high_coupling_function() -> i32 {
    42
}
"#;
        
        let low_coupling = r#"
fn low_coupling_function() -> i32 {
    42
}
"#;
        
        let analyzer = TdgAnalyzer::new().unwrap();
        
        let high_score = analyzer.analyze_source(high_coupling, Language::Rust, None).unwrap();
        let low_score = analyzer.analyze_source(low_coupling, Language::Rust, None).unwrap();
        
        assert!(low_score.coupling_score >= high_score.coupling_score,
               "Low coupling code should have better or equal coupling score");
    }
    
    #[tokio::test]
    async fn test_project_aggregation() {
        use crate::tdg::ProjectScore;
        
        let scores = vec![
            TdgScore {
                language: Language::Rust,
                confidence: 1.0,
                file_path: Some(std::path::PathBuf::from("test1.rs")),
                structural_complexity: 20.0,
                semantic_complexity: 18.0,
                duplication_ratio: 15.0,
                coupling_score: 22.0,
                doc_coverage: 25.0,
                consistency_score: 20.0,
                total: 80.0,
                grade: Grade::B,
                penalties_applied: vec![],
            },
            TdgScore {
                language: Language::Python,
                confidence: 1.0,
                file_path: Some(std::path::PathBuf::from("test2.py")),
                structural_complexity: 25.0,
                semantic_complexity: 20.0,
                duplication_ratio: 20.0,
                coupling_score: 25.0,
                doc_coverage: 20.0,
                consistency_score: 15.0,
                total: 75.0,
                grade: Grade::CPlus,
                penalties_applied: vec![],
            },
        ];
        
        let project = ProjectScore::aggregate(scores);
        
        assert_eq!(project.total_files, 2);
        assert_eq!(project.average_score, 77.5);
        assert_eq!(project.files.len(), 2);
        assert!(project.language_distribution.contains_key(&Language::Rust));
        assert!(project.language_distribution.contains_key(&Language::Python));
        assert_eq!(project.language_distribution[&Language::Rust], 1);
        assert_eq!(project.language_distribution[&Language::Python], 1);
    }
    
    #[tokio::test]
    async fn test_should_skip_directory() {
        let analyzer = TdgAnalyzer::new().unwrap();
        
        let skip_dirs = vec![
            "node_modules", "target", "build", "dist", ".git",
            "__pycache__", ".pytest_cache", "venv", ".venv",
            "vendor", ".idea", ".vscode"
        ];
        
        for dir in skip_dirs {
            let path = std::path::Path::new(dir);
            assert!(analyzer.should_skip_directory(path), "Should skip directory: {}", dir);
        }
        
        let keep_dirs = vec!["src", "tests", "examples", "docs"];
        for dir in keep_dirs {
            let path = std::path::Path::new(dir);
            assert!(!analyzer.should_skip_directory(path), "Should not skip directory: {}", dir);
        }
    }
    
    #[tokio::test]
    async fn test_should_analyze_file() {
        let analyzer = TdgAnalyzer::new().unwrap();
        
        let analyze_files = vec![
            "test.rs", "test.py", "test.js", "test.ts", "test.jsx", "test.tsx",
            "test.go", "test.java", "test.c", "test.h", "test.cpp", "test.cc",
            "test.cxx", "test.hpp", "test.rb", "test.swift", "test.kt", "test.kts"
        ];
        
        for file in analyze_files {
            let path = std::path::Path::new(file);
            assert!(analyzer.should_analyze_file(path), "Should analyze file: {}", file);
        }
        
        let skip_files = vec!["test.txt", "test.md", "test.json", "test.xml", "test"];
        for file in skip_files {
            let path = std::path::Path::new(file);
            assert!(!analyzer.should_analyze_file(path), "Should not analyze file: {}", file);
        }
    }
    
    #[tokio::test]
    async fn test_cyclomatic_complexity_estimation() {
        let simple_code = r#"
fn simple() -> i32 {
    42
}
"#;
        
        let complex_code = r#"
fn complex(x: i32, y: i32) -> i32 {
    if x > 0 {
        if y > 0 {
            for i in 0..x {
                while i < y {
                    match i {
                        0 => return 1,
                        1 => return 2,
                        _ => continue,
                    }
                }
            }
        }
    }
    0
}
"#;
        
        let analyzer = TdgAnalyzer::new().unwrap();
        
        let simple_lines: Vec<&str> = simple_code.lines().collect();
        let complex_lines: Vec<&str> = complex_code.lines().collect();
        
        let simple_complexity = analyzer.estimate_cyclomatic_complexity(&simple_lines);
        let complex_complexity = analyzer.estimate_cyclomatic_complexity(&complex_lines);
        
        assert!(simple_complexity < complex_complexity, 
               "Complex code should have higher cyclomatic complexity");
        assert!(simple_complexity >= 1, "Minimum complexity should be 1");
    }
    
    #[tokio::test]
    async fn test_nesting_depth_estimation() {
        let shallow_code = r#"
fn shallow() {
    let x = 1;
}
"#;
        
        let deep_code = r#"
fn deep() {
    if true {
        if true {
            if true {
                if true {
                    let x = 1;
                }
            }
        }
    }
}
"#;
        
        let analyzer = TdgAnalyzer::new().unwrap();
        
        let shallow_depth = analyzer.estimate_nesting_depth(shallow_code);
        let deep_depth = analyzer.estimate_nesting_depth(deep_code);
        
        assert!(deep_depth > shallow_depth, "Deep code should have higher nesting depth");
    }
    
    #[tokio::test]
    async fn test_duplication_ratio_estimation() {
        let no_duplication = r#"
fn unique_function() -> i32 {
    let unique_variable = 42;
    unique_variable
}
"#;
        
        let with_duplication = r#"
fn function_with_duplication() -> i32 {
    let very_long_variable_name_that_duplicates = 1;
    let another_variable = 2;
    let very_long_variable_name_that_duplicates = 1;
    let another_different_variable = 3;
    very_long_variable_name_that_duplicates
}
"#;
        
        let analyzer = TdgAnalyzer::new().unwrap();
        
        let no_dup_ratio = analyzer.estimate_duplication_ratio(no_duplication);
        let with_dup_ratio = analyzer.estimate_duplication_ratio(with_duplication);
        
        assert!(with_dup_ratio >= no_dup_ratio, 
               "Code with duplication should have higher duplication ratio");
    }
}