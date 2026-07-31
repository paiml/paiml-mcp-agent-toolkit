// Symbol table tests

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_visibility() {
        assert!(matches!(
            detect_visibility("pub fn test()"),
            Visibility::Public
        ));
        assert!(matches!(
            detect_visibility("private fn test()"),
            Visibility::Private
        ));
        assert!(matches!(
            detect_visibility("fn test()"),
            Visibility::Internal
        ));
    }

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("test.rs")));
        assert!(is_source_file(Path::new("test.js")));
        assert!(!is_source_file(Path::new("test.txt")));
    }

    #[test]
    fn test_extract_symbols_simple() {
        let content = "pub fn test_function() {}\nstruct TestStruct {}";
        let symbols = extract_symbols_simple(content, "test.rs").unwrap();
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "test_function");
        assert_eq!(symbols[1].name, "TestStruct");
    }

    /// Regression test for #654: `analyze symbol-table` reported `total_symbols: 0`
    /// for every tree, because clap's empty `Vec<String>` for --include/--exclude was
    /// joined into `Some("")` and `path.contains("")` skipped every path.
    #[tokio::test]
    async fn test_build_symbol_table_counts_symbols_with_no_patterns() {
        let dir = tempfile::tempdir().expect("tempdir");
        tokio::fs::write(
            dir.path().join("lib.rs"),
            "pub fn alpha() {}\nfn beta() {}\npub struct Gamma {}\n",
        )
        .await
        .expect("write fixture");

        // Exactly what the CLI hands over when neither --include nor --exclude is given.
        let table = build_symbol_table(dir.path(), &[], &[])
            .await
            .expect("symbol table");

        assert_eq!(
            table.total_symbols, 3,
            "expected 3 symbols, got {} — empty pattern lists must not filter everything",
            table.total_symbols
        );
        assert_eq!(table.total_symbols, table.symbols.len());
    }

    /// Regression test for #654: output must vary with input — a tree with sources
    /// must not be byte-identical to an empty directory.
    #[tokio::test]
    async fn test_build_symbol_table_differs_from_empty_directory() {
        let empty = tempfile::tempdir().expect("tempdir");
        let populated = tempfile::tempdir().expect("tempdir");
        tokio::fs::write(populated.path().join("m.rs"), "pub fn only_one() {}\n")
            .await
            .expect("write fixture");

        let empty_table = build_symbol_table(empty.path(), &[], &[])
            .await
            .expect("symbol table");
        let populated_table = build_symbol_table(populated.path(), &[], &[])
            .await
            .expect("symbol table");

        assert_eq!(empty_table.total_symbols, 0);
        assert!(populated_table.total_symbols > empty_table.total_symbols);
    }

    /// Regression test for #654: `--include '*.rs'` was substring-matched, so no path
    /// ever matched and the result was again empty.
    #[tokio::test]
    async fn test_build_symbol_table_glob_include_matches() {
        let dir = tempfile::tempdir().expect("tempdir");
        tokio::fs::write(dir.path().join("keep.rs"), "pub fn kept() {}\n")
            .await
            .expect("write fixture");
        tokio::fs::write(dir.path().join("skip.py"), "def skipped():\n    pass\n")
            .await
            .expect("write fixture");

        let table = build_symbol_table(dir.path(), &["*.rs".to_string()], &[])
            .await
            .expect("symbol table");

        assert_eq!(table.total_symbols, 1);
        assert_eq!(table.symbols[0].name, "kept");
    }

    /// Regression test for #654: --exclude must exclude only what it names.
    #[tokio::test]
    async fn test_build_symbol_table_exclude_only_matching() {
        let dir = tempfile::tempdir().expect("tempdir");
        tokio::fs::write(dir.path().join("keep.rs"), "pub fn kept() {}\n")
            .await
            .expect("write fixture");
        tokio::fs::write(dir.path().join("drop.rs"), "pub fn dropped() {}\n")
            .await
            .expect("write fixture");

        let table = build_symbol_table(dir.path(), &[], &["drop".to_string()])
            .await
            .expect("symbol table");

        assert_eq!(table.total_symbols, 1);
        assert_eq!(table.symbols[0].name, "kept");
    }

    /// missing_path_fails: a nonexistent path must fail and name the path.
    #[tokio::test]
    async fn test_symbol_table_missing_path_fails() {
        let result = handle_analyze_symbol_table(
            PathBuf::from("/nonexistent/pmat/symbol/table/path"),
            crate::cli::SymbolTableOutputFormat::Json,
            None,
            None,
            &[],
            &[],
            false,
            false,
            None,
            false,
        )
        .await;

        let err = result.expect_err("missing path must not exit 0").to_string();
        assert!(
            err.contains("/nonexistent/pmat/symbol/table/path"),
            "error must name the path, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_symbol_table_creation() {
        let table = SymbolTable {
            symbols: vec![Symbol {
                name: "test".to_string(),
                kind: SymbolKind::Function,
                file: "test.rs".to_string(),
                line: 1,
                column: 0,
                visibility: Visibility::Public,
                references: vec![],
            }],
            total_symbols: 1,
            unreferenced_symbols: vec!["test".to_string()],
            most_referenced: vec![],
        };

        assert_eq!(table.total_symbols, 1);
        assert_eq!(table.unreferenced_symbols.len(), 1);
    }
}

