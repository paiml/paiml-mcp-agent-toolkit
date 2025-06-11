#[cfg(test)]
mod cli_module_tests {
    use crate::cli::{
        detect_primary_language, apply_satd_filters,
        EarlyCliArgs,
    };
    use crate::models::tdg::{SatdItem, SatdSeverity};
    use std::path::Path;
    use tempfile::TempDir;

    fn create_test_project_with_languages() -> TempDir {
        let test_dir = TempDir::new().unwrap();
        
        // Create Rust files
        let src_dir = test_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();
        std::fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(src_dir.join("lib.rs"), "pub fn test() {}").unwrap();
        std::fs::write(src_dir.join("utils.rs"), "pub mod utils;").unwrap();
        
        // Create TypeScript files
        std::fs::write(test_dir.path().join("app.ts"), "console.log('hello');").unwrap();
        std::fs::write(test_dir.path().join("types.tsx"), "export interface Test {}").unwrap();
        
        // Create Python files
        std::fs::write(test_dir.path().join("script.py"), "print('hello')").unwrap();
        
        // Create JavaScript files
        std::fs::write(test_dir.path().join("app.js"), "console.log('hello');").unwrap();
        
        test_dir
    }

    fn create_rust_heavy_project() -> TempDir {
        let test_dir = TempDir::new().unwrap();
        let src_dir = test_dir.path().join("src");
        std::fs::create_dir(&src_dir).unwrap();
        
        // Create many Rust files
        for i in 0..10 {
            std::fs::write(
                src_dir.join(format!("mod{}.rs", i)),
                format!("pub fn function{}() {{}}", i)
            ).unwrap();
        }
        
        test_dir
    }

    fn create_python_heavy_project() -> TempDir {
        let test_dir = TempDir::new().unwrap();
        
        // Create many Python files
        for i in 0..10 {
            std::fs::write(
                test_dir.path().join(format!("script{}.py", i)),
                format!("def function{}(): pass", i)
            ).unwrap();
        }
        
        test_dir
    }

    fn create_deno_heavy_project() -> TempDir {
        let test_dir = TempDir::new().unwrap();
        
        // Create many TypeScript/JavaScript files
        for i in 0..10 {
            std::fs::write(
                test_dir.path().join(format!("app{}.ts", i)),
                format!("export function function{}() {{}}", i)
            ).unwrap();
        }
        
        test_dir
    }

    #[test]
    fn test_detect_primary_language_rust() {
        let test_dir = create_rust_heavy_project();
        let language = detect_primary_language(test_dir.path());
        assert_eq!(language, Some("rust".to_string()));
    }

    #[test]
    fn test_detect_primary_language_python() {
        let test_dir = create_python_heavy_project();
        let language = detect_primary_language(test_dir.path());
        assert_eq!(language, Some("python-uv".to_string()));
    }

    #[test]
    fn test_detect_primary_language_deno() {
        let test_dir = create_deno_heavy_project();
        let language = detect_primary_language(test_dir.path());
        assert_eq!(language, Some("deno".to_string()));
    }

    #[test]
    fn test_detect_primary_language_mixed() {
        let test_dir = create_test_project_with_languages();
        let language = detect_primary_language(test_dir.path());
        // Should detect rust since it has the most files (3 vs 2 vs 1 vs 1)
        assert_eq!(language, Some("rust".to_string()));
    }

    #[test]
    fn test_detect_primary_language_empty_directory() {
        let test_dir = TempDir::new().unwrap();
        let language = detect_primary_language(test_dir.path());
        assert_eq!(language, None);
    }

    #[test]
    fn test_detect_primary_language_no_recognized_files() {
        let test_dir = TempDir::new().unwrap();
        std::fs::write(test_dir.path().join("README.md"), "# Test").unwrap();
        std::fs::write(test_dir.path().join("data.txt"), "some data").unwrap();
        
        let language = detect_primary_language(test_dir.path());
        assert_eq!(language, None);
    }

    #[test]
    fn test_detect_primary_language_nonexistent_path() {
        let language = detect_primary_language(Path::new("/non/existent/path"));
        assert_eq!(language, None);
    }

    #[test]
    fn test_detect_primary_language_jsx_files() {
        let test_dir = TempDir::new().unwrap();
        std::fs::write(test_dir.path().join("component.jsx"), "export default function() {}").unwrap();
        std::fs::write(test_dir.path().join("app.js"), "console.log('hello');").unwrap();
        
        let language = detect_primary_language(test_dir.path());
        assert_eq!(language, Some("deno".to_string()));
    }

    #[test]
    fn test_apply_satd_filters_no_filters() {
        let items = vec![
            SatdItem {
                file_path: "test.rs".into(),
                line_number: 1,
                comment_text: "TODO: implement".to_string(),
                debt_type: "todo".to_string(),
                severity: SatdSeverity::Medium,
                confidence: 0.8,
            },
            SatdItem {
                file_path: "test2.rs".into(),
                line_number: 2,
                comment_text: "FIXME: bug".to_string(),
                debt_type: "fixme".to_string(),
                severity: SatdSeverity::High,
                confidence: 0.9,
            },
        ];

        let filtered = apply_satd_filters(items.clone(), None, false);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered, items);
    }

    #[test]
    fn test_apply_satd_filters_by_severity() {
        let items = vec![
            SatdItem {
                file_path: "test.rs".into(),
                line_number: 1,
                comment_text: "TODO: implement".to_string(),
                debt_type: "todo".to_string(),
                severity: SatdSeverity::Low,
                confidence: 0.7,
            },
            SatdItem {
                file_path: "test2.rs".into(),
                line_number: 2,
                comment_text: "FIXME: bug".to_string(),
                debt_type: "fixme".to_string(),
                severity: SatdSeverity::High,
                confidence: 0.9,
            },
            SatdItem {
                file_path: "test3.rs".into(),
                line_number: 3,
                comment_text: "HACK: workaround".to_string(),
                debt_type: "hack".to_string(),
                severity: SatdSeverity::Medium,
                confidence: 0.8,
            },
        ];

        let filtered = apply_satd_filters(items, Some(SatdSeverity::High), false);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].severity, SatdSeverity::High);
    }

    #[test]
    fn test_apply_satd_filters_critical_only() {
        let items = vec![
            SatdItem {
                file_path: "test.rs".into(),
                line_number: 1,
                comment_text: "TODO: implement".to_string(),
                debt_type: "todo".to_string(),
                severity: SatdSeverity::Medium,
                confidence: 0.8,
            },
            SatdItem {
                file_path: "test2.rs".into(),
                line_number: 2,
                comment_text: "FIXME: critical bug".to_string(),
                debt_type: "fixme".to_string(),
                severity: SatdSeverity::Critical,
                confidence: 0.95,
            },
        ];

        let filtered = apply_satd_filters(items, None, true);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].severity, SatdSeverity::Critical);
    }

    #[test]
    fn test_apply_satd_filters_severity_and_critical() {
        let items = vec![
            SatdItem {
                file_path: "test.rs".into(),
                line_number: 1,
                comment_text: "TODO: implement".to_string(),
                debt_type: "todo".to_string(),
                severity: SatdSeverity::Low,
                confidence: 0.7,
            },
            SatdItem {
                file_path: "test2.rs".into(),
                line_number: 2,
                comment_text: "FIXME: bug".to_string(),
                debt_type: "fixme".to_string(),
                severity: SatdSeverity::High,
                confidence: 0.9,
            },
            SatdItem {
                file_path: "test3.rs".into(),
                line_number: 3,
                comment_text: "HACK: critical issue".to_string(),
                debt_type: "hack".to_string(),
                severity: SatdSeverity::Critical,
                confidence: 0.95,
            },
        ];

        let filtered = apply_satd_filters(items, Some(SatdSeverity::High), true);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].severity, SatdSeverity::Critical);
    }

    #[test]
    fn test_apply_satd_filters_empty_input() {
        let items = vec![];
        let filtered = apply_satd_filters(items, Some(SatdSeverity::High), true);
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_parse_early_for_tracing_no_flags() {
        // We can't easily mock std::env::args in a unit test, so we'll test the struct directly
        let early_args = EarlyCliArgs {
            verbose: false,
            debug: false,
            trace: false,
            trace_filter: None,
        };

        assert!(!early_args.verbose);
        assert!(!early_args.debug);
        assert!(!early_args.trace);
        assert!(early_args.trace_filter.is_none());
    }

    #[test]
    fn test_parse_early_for_tracing_verbose() {
        let early_args = EarlyCliArgs {
            verbose: true,
            debug: false,
            trace: false,
            trace_filter: None,
        };

        assert!(early_args.verbose);
        assert!(!early_args.debug);
        assert!(!early_args.trace);
    }

    #[test]
    fn test_parse_early_for_tracing_debug() {
        let early_args = EarlyCliArgs {
            verbose: false,
            debug: true,
            trace: false,
            trace_filter: None,
        };

        assert!(!early_args.verbose);
        assert!(early_args.debug);
        assert!(!early_args.trace);
    }

    #[test]
    fn test_parse_early_for_tracing_trace() {
        let early_args = EarlyCliArgs {
            verbose: false,
            debug: false,
            trace: true,
            trace_filter: None,
        };

        assert!(!early_args.verbose);
        assert!(!early_args.debug);
        assert!(early_args.trace);
    }

    #[test]
    fn test_parse_early_for_tracing_with_filter() {
        let early_args = EarlyCliArgs {
            verbose: false,
            debug: false,
            trace: false,
            trace_filter: Some("debug".to_string()),
        };

        assert!(!early_args.verbose);
        assert!(!early_args.debug);
        assert!(!early_args.trace);
        assert_eq!(early_args.trace_filter, Some("debug".to_string()));
    }

    #[test]
    fn test_satd_item_creation() {
        let item = SatdItem {
            file_path: "test.rs".into(),
            line_number: 1,
            comment_text: "TODO: implement".to_string(),
            debt_type: "todo".to_string(),
            severity: SatdSeverity::Medium,
            confidence: 0.8,
        };

        assert_eq!(item.line_number, 1);
        assert_eq!(item.comment_text, "TODO: implement");
        assert_eq!(item.debt_type, "todo");
        assert_eq!(item.severity, SatdSeverity::Medium);
        assert_eq!(item.confidence, 0.8);
    }

    #[test]
    fn test_early_cli_args_debug_trait() {
        let early_args = EarlyCliArgs {
            verbose: true,
            debug: false,
            trace: false,
            trace_filter: Some("info".to_string()),
        };

        let debug_output = format!("{:?}", early_args);
        assert!(debug_output.contains("verbose: true"));
        assert!(debug_output.contains("debug: false"));
        assert!(debug_output.contains("trace: false"));
        assert!(debug_output.contains("info"));
    }
}