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

    /// The exact fixture from #654 (round 2): `helper_one` is called from both
    /// `main()` and `helper_two()`. Before this fix every symbol carried exactly
    /// one reference — its own `Definition` — so `helper_one` had `references`
    /// of length 1 and was listed in `unreferenced_symbols`.
    const CALL_GRAPH_FIXTURE: &str = concat!(
        "fn helper_one() {}\n",
        "fn helper_two() {\n",
        "    helper_one();\n",
        "}\n",
        "fn never_called() {}\n",
        "fn main() {\n",
        "    helper_one();\n",
        "    helper_two();\n",
        "}\n",
    );

    async fn call_graph_table() -> SymbolTable {
        let dir = tempfile::tempdir().expect("tempdir");
        tokio::fs::write(dir.path().join("main.rs"), CALL_GRAPH_FIXTURE)
            .await
            .expect("write fixture");
        build_symbol_table(dir.path(), &[], &[])
            .await
            .expect("symbol table")
    }

    fn symbol<'a>(table: &'a SymbolTable, name: &str) -> &'a Symbol {
        table
            .symbols
            .iter()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("{name} must be extracted"))
    }

    /// Regression test for #654 (round 2): call sites must actually be resolved.
    #[tokio::test]
    async fn test_call_sites_are_resolved_as_references() {
        let table = call_graph_table().await;

        assert_eq!(
            usage_count(symbol(&table, "helper_one")),
            2,
            "helper_one is called from main() and helper_two(); got {:?}",
            symbol(&table, "helper_one").references
        );
        assert_eq!(usage_count(symbol(&table, "helper_two")), 1);
        assert_eq!(usage_count(symbol(&table, "never_called")), 0);
    }

    /// Regression test for #654 (round 2): `unreferenced_symbols` used to list
    /// every symbol in the tree (16944 of 16944, fraction 1.0).
    #[tokio::test]
    async fn test_unreferenced_lists_only_genuinely_unreferenced() {
        let table = call_graph_table().await;

        assert!(
            !table.unreferenced_symbols.contains(&"helper_one".to_string()),
            "helper_one has two call sites, got {:?}",
            table.unreferenced_symbols
        );
        assert!(
            table
                .unreferenced_symbols
                .contains(&"never_called".to_string()),
            "never_called has no call site, got {:?}",
            table.unreferenced_symbols
        );
        assert!(
            table.unreferenced_symbols.len() < table.total_symbols,
            "every symbol reported unreferenced: {:?}",
            table.unreferenced_symbols
        );
    }

    /// Regression test for #654 (round 2): `most_referenced` was
    /// `[["main",1],["main",1],["Cli",1],["Commands",1]]` — every entry the
    /// symbol's own definition, and names repeated once per declaration.
    #[tokio::test]
    async fn test_most_referenced_counts_use_sites_not_definitions() {
        let table = call_graph_table().await;

        assert_eq!(
            table.most_referenced,
            vec![("helper_one".to_string(), 2), ("helper_two".to_string(), 1)],
            "expected use-site counts, got {:?}",
            table.most_referenced
        );
    }

    /// A name mentioned only in a comment or a string literal is not a reference.
    #[tokio::test]
    async fn test_comments_and_strings_are_not_references() {
        let dir = tempfile::tempdir().expect("tempdir");
        tokio::fs::write(
            dir.path().join("m.rs"),
            concat!(
                "fn mentioned_only_in_prose() {}\n",
                "// mentioned_only_in_prose is discussed here\n",
                "/* and mentioned_only_in_prose here too */\n",
                "fn main() {\n",
                "    let s = \"mentioned_only_in_prose\";\n",
                "}\n",
            ),
        )
        .await
        .expect("write fixture");

        let table = build_symbol_table(dir.path(), &[], &[])
            .await
            .expect("symbol table");

        assert_eq!(usage_count(symbol(&table, "mentioned_only_in_prose")), 0);
        assert!(table
            .unreferenced_symbols
            .contains(&"mentioned_only_in_prose".to_string()));
    }

    /// `x.unwrap()` names a member of a value, not the free function `unwrap`
    /// declared elsewhere in the tree. Without this rule `unwrap` topped
    /// `most_referenced` on the pmat sources with 19112 fabricated references,
    /// and `let name` counted as a use of a `const name` in another file.
    #[tokio::test]
    async fn test_member_access_and_redeclaration_are_not_references() {
        let dir = tempfile::tempdir().expect("tempdir");
        tokio::fs::write(
            dir.path().join("m.rs"),
            concat!(
                "fn unwrap() {}\n",
                "fn caller() {\n",
                "    let v: Option<u8> = None;\n",
                "    v.unwrap();\n",
                "    let unwrap = 1;\n",
                "}\n",
            ),
        )
        .await
        .expect("write fixture");

        let table = build_symbol_table(dir.path(), &[], &[])
            .await
            .expect("symbol table");

        assert_eq!(
            usage_count(symbol(&table, "unwrap")),
            0,
            "`.unwrap()` and `let unwrap` are not references to `fn unwrap`: {:?}",
            symbol(&table, "unwrap").references
        );
    }

    /// Rust attribute arguments are not call sites: `#[derive(Debug)]` made a
    /// `pub fn derive(` elsewhere in the tree the most referenced symbol in pmat.
    #[tokio::test]
    async fn test_attribute_arguments_are_not_references() {
        let dir = tempfile::tempdir().expect("tempdir");
        tokio::fs::write(
            dir.path().join("m.rs"),
            concat!(
                "pub fn derive() {}\n",
                "#[derive(Debug)]\n",
                "pub struct Held {}\n",
            ),
        )
        .await
        .expect("write fixture");

        let table = build_symbol_table(dir.path(), &[], &[])
            .await
            .expect("symbol table");

        assert_eq!(
            usage_count(symbol(&table, "derive")),
            0,
            "`#[derive(..)]` is not a call of `fn derive`: {:?}",
            symbol(&table, "derive").references
        );
    }

    /// An identifier exported from several files and used from a third file
    /// cannot be attributed, so it must not be claimed "unreferenced" either.
    #[tokio::test]
    async fn test_ambiguous_names_are_not_claimed_unreferenced() {
        let dir = tempfile::tempdir().expect("tempdir");
        tokio::fs::write(dir.path().join("a.rs"), "pub fn shared() {}\n")
            .await
            .expect("write fixture");
        tokio::fs::write(dir.path().join("b.rs"), "pub fn shared() {}\n")
            .await
            .expect("write fixture");
        tokio::fs::write(
            dir.path().join("c.rs"),
            "pub fn caller() {\n    shared();\n}\n",
        )
        .await
        .expect("write fixture");

        let table = build_symbol_table(dir.path(), &[], &[])
            .await
            .expect("symbol table");

        assert!(
            !table.unreferenced_symbols.contains(&"shared".to_string()),
            "`shared` is used but unattributable; claiming it unreferenced is a fabricated measurement: {:?}",
            table.unreferenced_symbols
        );
    }

    /// A use in another file resolves only to a declaration that is visible
    /// there. A file-private `fn` cannot be what an identically named
    /// identifier in a different file refers to.
    #[tokio::test]
    async fn test_cross_file_uses_resolve_only_to_exported_declarations() {
        let dir = tempfile::tempdir().expect("tempdir");
        tokio::fs::write(
            dir.path().join("lib.rs"),
            "pub fn exported_helper() {}\nfn private_helper() {}\n",
        )
        .await
        .expect("write fixture");
        tokio::fs::write(
            dir.path().join("user.rs"),
            "pub fn use_them() {\n    exported_helper();\n    private_helper();\n}\n",
        )
        .await
        .expect("write fixture");

        let table = build_symbol_table(dir.path(), &[], &[])
            .await
            .expect("symbol table");

        assert_eq!(usage_count(symbol(&table, "exported_helper")), 1);
        assert_eq!(
            usage_count(symbol(&table, "private_helper")),
            0,
            "a file-private declaration is not what another file's identifier names"
        );
    }

    /// Regression test for #654 (round 2): `--filter` shrank `symbols` but left
    /// `total_symbols` at the pre-filter count — 2 above a 1-element list.
    #[tokio::test]
    async fn test_filter_updates_total_symbols() {
        let dir = tempfile::tempdir().expect("tempdir");
        tokio::fs::write(
            dir.path().join("m.rs"),
            "pub struct OnlyStruct {}\npub fn only_fn() {}\n",
        )
        .await
        .expect("write fixture");

        let table = build_symbol_table(dir.path(), &[], &[])
            .await
            .expect("symbol table");
        assert_eq!(table.total_symbols, 2);

        let filtered = apply_filters(table, Some(crate::cli::SymbolTypeFilter::Functions), None)
            .expect("filter");

        assert_eq!(
            filtered.total_symbols,
            filtered.symbols.len(),
            "the header count must match the list it heads"
        );
        assert_eq!(filtered.total_symbols, 1);
        assert!(
            !filtered
                .unreferenced_symbols
                .contains(&"OnlyStruct".to_string()),
            "a filtered-out symbol must not survive in the summary: {:?}",
            filtered.unreferenced_symbols
        );
    }

    /// Identical input must produce identical output. Run it more than once —
    /// `HashMap` iteration order and `read_dir` order both vary between runs.
    #[tokio::test]
    async fn test_symbol_table_output_is_deterministic() {
        let dir = tempfile::tempdir().expect("tempdir");
        for (name, body) in [
            ("alpha.rs", "pub fn a_one() {}\npub struct AType {}\n"),
            ("beta.rs", "pub fn b_one() {}\npub enum BEnum {}\n"),
            ("gamma.rs", "pub fn g_one() {\n    a_one();\n}\n"),
        ] {
            tokio::fs::write(dir.path().join(name), body)
                .await
                .expect("write fixture");
        }

        let mut renders = Vec::new();
        for _ in 0..5 {
            let table = build_symbol_table(dir.path(), &[], &[])
                .await
                .expect("symbol table");
            let json = format_output(
                table,
                crate::cli::SymbolTableOutputFormat::Json,
                true,
                false,
            )
            .expect("json");
            let table = build_symbol_table(dir.path(), &[], &[])
                .await
                .expect("symbol table");
            let human = format_output(
                table,
                crate::cli::SymbolTableOutputFormat::Summary,
                true,
                true,
            )
            .expect("human");
            renders.push((json, human));
        }

        for (i, render) in renders.iter().enumerate().skip(1) {
            assert_eq!(
                render.0, renders[0].0,
                "JSON render {i} differs from render 0"
            );
            assert_eq!(
                render.1, renders[0].1,
                "human render {i} differs from render 0"
            );
        }
    }

    /// `--show-references` was bound to `_show_references` and discarded, so the
    /// declared flag changed nothing in the output.
    #[tokio::test]
    async fn test_show_references_changes_output() {
        let table = call_graph_table().await;
        let without = format_output(
            call_graph_table().await,
            crate::cli::SymbolTableOutputFormat::Summary,
            false,
            false,
        )
        .expect("render");
        let with = format_output(
            table,
            crate::cli::SymbolTableOutputFormat::Summary,
            false,
            true,
        )
        .expect("render");

        assert_ne!(
            without, with,
            "--show-references must change what is rendered"
        );
        assert!(with.contains("used at: main.rs:3"), "got: {with}");
    }

    /// `--help` advertises `detailed` as "Detailed output with all symbols",
    /// but it rendered byte-identically to `summary`: 10 symbols per group and
    /// no reference sites. A declared format must produce that format.
    #[tokio::test]
    async fn test_detailed_format_differs_from_summary() {
        let dir = tempfile::tempdir().expect("tempdir");
        let many: String = (0..SYMBOLS_PER_GROUP_IN_SUMMARY + 4)
            .map(|i| format!("pub fn f{i}() {{}}\n"))
            .collect();
        tokio::fs::write(dir.path().join("m.rs"), many)
            .await
            .expect("write fixture");

        let build = || build_symbol_table(dir.path(), &[], &[]);
        let summary = format_output(
            build().await.expect("symbol table"),
            crate::cli::SymbolTableOutputFormat::Summary,
            false,
            false,
        )
        .expect("render");
        let detailed = format_output(
            build().await.expect("symbol table"),
            crate::cli::SymbolTableOutputFormat::Detailed,
            false,
            false,
        )
        .expect("render");

        assert_ne!(summary, detailed, "detailed must not be a copy of summary");
        assert!(summary.contains("and 4 more"), "got: {summary}");
        assert!(
            !detailed.contains("and 4 more") && detailed.contains("f13"),
            "detailed must list all symbols, got: {detailed}"
        );
    }

    /// Regression test for #654: `analyze symbol-table -q` collided with the
    /// global `-q/--quiet`, tripping clap's "short option names must be unique"
    /// debug assertion and aborting the command with exit 101 in any debug build.
    #[test]
    fn test_symbol_table_command_parses_in_a_debug_build() {
        // Building pmat's clap tree needs more than the 2 MiB default test stack.
        let outcome = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                use clap::Parser;
                // `-q` here is the *global* quiet flag; `--query` is the search
                // query. Before the fix both claimed `-q` on this subcommand, so
                // clap's uniqueness assertion aborted the process (exit 101)
                // before any argument was even parsed.
                let cli = crate::cli::Cli::try_parse_from([
                    "pmat",
                    "analyze",
                    "symbol-table",
                    "-q",
                    "--query",
                    "needle",
                ])
                .expect("analyze symbol-table must parse");
                assert!(cli.quiet, "-q must still reach the global quiet flag");
            })
            .expect("spawn")
            .join();

        assert!(
            outcome.is_ok(),
            "`pmat analyze symbol-table` must not abort while clap builds the command"
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

