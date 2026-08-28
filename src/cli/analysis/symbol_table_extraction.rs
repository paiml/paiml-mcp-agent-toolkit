// Symbol extraction and file collection logic

#[allow(clippy::too_many_arguments)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub async fn handle_analyze_symbol_table(
    project_path: PathBuf,
    format: crate::cli::SymbolTableOutputFormat,
    filter: Option<crate::cli::SymbolTypeFilter>,
    query: Option<String>,
    include: &[String],
    exclude: &[String],
    show_unreferenced: bool,
    show_references: bool,
    output: Option<PathBuf>,
    _perf: bool,
    top_files: usize,
) -> Result<()> {
    // missing_path_fails: a nonexistent path must exit non-zero naming the path,
    // not walk nothing and report an empty (but plausible-looking) table.
    crate::cli::ensure_analysis_path_exists(&project_path)?;

    crate::status_eprintln!("🔍 Building symbol table for project...");

    // Build the symbol table
    let table = build_symbol_table(&project_path, include, exclude, top_files).await?;

    // #1015: "Total symbols: 0" with exit 0 over a directory holding no source
    // file is the same report a project of empty modules produces. Refused
    // here, after the build, so `build_symbol_table` stays a pure builder and
    // the count it reports is the one the refusal is made on.
    crate::cli::ensure_source_files_were_analyzed(
        "symbol-table",
        &project_path,
        table.files_scanned,
    )?;

    // Apply filters
    let filtered = apply_filters(table, filter, query, top_files)?;

    // Format output
    let content = format_output(
        filtered,
        format,
        show_unreferenced,
        show_references,
        top_files,
    )?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        crate::status_eprintln!("✅ Symbol table written to: {}", output_path.display());
    } else {
        println!("{content}");
    }

    Ok(())
}

// Build symbol table from project files
async fn build_symbol_table(
    project_path: &Path,
    include: &[String],
    exclude: &[String],
    top_files: usize,
) -> Result<SymbolTable> {
    // Get all relevant files. `collect_files` returns them sorted, so every
    // downstream ordering (symbol order, reference order, tie-breaks) is stable.
    let files = collect_files(project_path, include, exclude).await?;

    // Read every file once: definitions come from it, and so do the use sites.
    let mut sources = Vec::with_capacity(files.len());
    for file in files {
        let content = tokio::fs::read_to_string(&file).await?;
        sources.push(FileSource {
            path: file.to_string_lossy().to_string(),
            content,
        });
    }

    let mut symbols = Vec::new();
    for source in &sources {
        symbols.extend(extract_symbols_simple(&source.content, &source.path)?);
    }

    // Defect #654 (round 2): before this, every symbol carried exactly one
    // reference — its own Definition — so `unreferenced_symbols` contained all
    // 16944 of 16944 symbols and `most_referenced` was a list of 1s. Use sites
    // are now resolved from the sources; `unresolved` holds names we could not
    // attribute, which must never be reported as unreferenced.
    let unresolved = resolve_references(&sources, &mut symbols);

    let unreferenced = find_unreferenced_symbols(&symbols, &unresolved);
    let (most_referenced, referenced_symbol_count) = find_most_referenced(&symbols, top_files);

    Ok(SymbolTable {
        total_symbols: symbols.len(),
        symbols,
        unreferenced_symbols: unreferenced,
        most_referenced,
        referenced_symbol_count,
        files_scanned: sources.len(),
    })
}

// Collect files based on include/exclude patterns
async fn collect_files(
    project_path: &Path,
    include: &[String],
    exclude: &[String],
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if project_path.is_file() {
        process_file(project_path.to_path_buf(), &mut files, include)?;
        return Ok(files);
    }

    collect_files_recursive(project_path, &mut files, include, exclude).await?;

    // `read_dir` yields entries in filesystem order, which is not stable across
    // machines (and not guaranteed stable on one). Sorting here is what makes
    // two runs over an unchanged tree byte-identical.
    files.sort();

    Ok(files)
}

// Recursively collect files
async fn collect_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    include: &[String],
    exclude: &[String],
) -> Result<()> {
    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        process_directory_entry(entry, files, include, exclude).await?;
    }

    Ok(())
}

/// Process a single directory entry
async fn process_directory_entry(
    entry: tokio::fs::DirEntry,
    files: &mut Vec<PathBuf>,
    include: &[String],
    exclude: &[String],
) -> Result<()> {
    let path = entry.path();

    if should_skip_path(&path, exclude) {
        return Ok(());
    }

    if path.is_dir() {
        process_directory(&path, files, include, exclude).await
    } else {
        process_file(path, files, include)
    }
}

/// Check if path should be skipped
///
/// Defect #654: this used to take `Option<String>` built with `patterns.join(",")`,
/// so "no --exclude given" arrived as `Some("")` and `path.contains("")` is true for
/// every path — every file and directory was skipped and `total_symbols` was always 0,
/// even for the whole pmat source tree. An empty pattern list now excludes nothing.
fn should_skip_path(path: &Path, exclude: &[String]) -> bool {
    exclude.iter().any(|pattern| matches_pattern(path, pattern))
}

/// Process a directory
async fn process_directory(
    path: &Path,
    files: &mut Vec<PathBuf>,
    include: &[String],
    exclude: &[String],
) -> Result<()> {
    if should_process_directory(path) {
        Box::pin(collect_files_recursive(path, files, include, exclude)).await?;
    }
    Ok(())
}

/// Check if directory should be processed
fn should_process_directory(path: &Path) -> bool {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    !name.starts_with('.') && name != "node_modules" && name != "target"
}

/// Process a file
fn process_file(path: PathBuf, files: &mut Vec<PathBuf>, include: &[String]) -> Result<()> {
    if !is_source_file(&path) {
        return Ok(());
    }

    if should_include_file(&path, include) {
        files.push(path);
    }
    Ok(())
}

/// Check if file should be included (an empty pattern list includes everything)
fn should_include_file(path: &Path, include: &[String]) -> bool {
    include.is_empty() || include.iter().any(|pattern| matches_pattern(path, pattern))
}

/// Match a path against a user-supplied pattern.
///
/// Glob patterns (`*.rs`, `src/**/mod.rs`) are matched against both the full path and
/// the file name; plain patterns fall back to substring matching. Defect #654 also
/// reported `--include '*.rs'` yielding 0 symbols because the old code only ever did
/// `path.contains("*.rs")`, which no real path satisfies.
fn matches_pattern(path: &Path, pattern: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }

    let path_str = path.to_string_lossy();

    if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
        if let Ok(glob) = glob::Pattern::new(pattern) {
            let file_name = path.file_name().map(|n| n.to_string_lossy().to_string());
            return glob.matches(&path_str) || file_name.is_some_and(|name| glob.matches(&name));
        }
    }

    path_str.contains(pattern)
}

// Check if file is a source file
fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" | "h" | "hpp" | "go" | "rb")
    )
}

/// The symbol patterns, compiled ONCE for the process.
///
/// They used to be built inside `extract_symbols_simple`, which runs once per
/// file — so a 1,387-file tree paid for 12 regex compilations 1,387 times.
/// Measured on this repository before the hoist: `analyze symbol-table --path
/// src/services` took 32.2s (debug) for 1,387 files, ~23ms per file, and a
/// controlled experiment showed 82% of the runtime was a constant charged per
/// FILE rather than per byte — 201 near-empty files cost 500ms against 605ms for
/// the same bytes in 202 real ones, while `analyze complexity` over the same two
/// fixtures showed no gap at all (0.94x). Regex compilation was the only
/// difference.
///
/// `expect` rather than `?`: these are compile-time literals, so a failure here
/// is a bug in this file and not a condition a caller can handle. The repo's own
/// note on TDG's false F-grade for `unwrap` on a static regex says to use
/// `expect` with a reason, which is what this is.
static SYMBOL_PATTERNS: std::sync::LazyLock<Vec<(regex::Regex, SymbolKind)>> =
    std::sync::LazyLock::new(|| {
        use regex::Regex;
        vec![
            // `const fn` is a function declaration, not a constant. Without the
            // optional `const\s+` here, `const fn answer()` matched no function
            // pattern at all and `answer` was missing from the table entirely.
            (
                Regex::new(r"(?m)^\s*(?:pub\s+)?(?:const\s+)?(?:async\s+)?fn\s+(\w+)")
                    .expect("static symbol pattern must compile"),
                SymbolKind::Function,
            ),
            (
                Regex::new(r"(?m)^\s*(?:export\s+)?class\s+(\w+)")
                    .expect("static symbol pattern must compile"),
                SymbolKind::Class,
            ),
            (
                Regex::new(r"(?m)^\s*(?:export\s+)?(?:async\s+)?function\s+(\w+)")
                    .expect("static symbol pattern must compile"),
                SymbolKind::Function,
            ),
            (
                Regex::new(r"(?m)^\s*def\s+(\w+)").expect("static symbol pattern must compile"),
                SymbolKind::Function,
            ),
            // `--help` offers `--filter variables` ("Variables and constants") and
            // `--filter modules` ("Modules and namespaces"), but nothing ever
            // produced a `Variable` or a `Module`, so a fixture with `pub const
            // KONST`, `pub static STAT` and `pub mod inner` returned 0 for both —
            // two of the six advertised filter values could not match anything.
            //
            // The old constant pattern was `^const\s+(\w+)\s*=`, which a Rust
            // `pub const KONST: u32 = 1;` fails twice over (the `pub`, and the type
            // annotation before `=`). This one covers both it and JS `const x = …`.
            //
            // Requiring the `:` (Rust type annotation) or `=` (JS initialiser) that
            // must follow a real constant's name is what keeps `const fn answer()`
            // out: the bare `const\s+(\w+)` form captured the `fn` *keyword* as a
            // constant named "fn", which then out-ranked every real identifier in
            // `most_referenced` (54 266 "references" on pmat itself).
            (
                Regex::new(r"(?m)^\s*(?:pub\s+)?const\s+(\w+)\s*[:=]")
                    .expect("static symbol pattern must compile"),
                SymbolKind::Constant,
            ),
            // Same keyword-capture trap as `const fn`: `static mut COUNTER` used to
            // yield a symbol literally named "mut".
            (
                Regex::new(r"(?m)^\s*(?:pub\s+)?static\s+(?:mut\s+)?(\w+)\s*[:=]")
                    .expect("static symbol pattern must compile"),
                SymbolKind::Variable,
            ),
            (
                Regex::new(r"(?m)^\s*(?:pub\s+)?mod\s+(\w+)")
                    .expect("static symbol pattern must compile"),
                SymbolKind::Module,
            ),
            (
                Regex::new(r"(?m)^\s*(?:pub\s+)?struct\s+(\w+)")
                    .expect("static symbol pattern must compile"),
                SymbolKind::Type,
            ),
            (
                Regex::new(r"(?m)^\s*(?:pub\s+)?enum\s+(\w+)")
                    .expect("static symbol pattern must compile"),
                SymbolKind::Enum,
            ),
            (
                Regex::new(r"(?m)^\s*(?:export\s+)?interface\s+(\w+)")
                    .expect("static symbol pattern must compile"),
                SymbolKind::Interface,
            ),
            // #693: `trait` and `type` had no pattern at all, so `pub trait
            // Drawable` and `pub type WidgetAlias = Widget;` were absent from the
            // table entirely — as was every TypeScript `export type`.
            (
                Regex::new(r"(?m)^\s*(?:pub\s+|export\s+)?trait\s+(\w+)")
                    .expect("static symbol pattern must compile"),
                SymbolKind::Interface,
            ),
            (
                Regex::new(r"(?m)^\s*(?:pub\s+|export\s+)?type\s+(\w+)")
                    .expect("static symbol pattern must compile"),
                SymbolKind::Type,
            ),
        ]
    });

// Simple symbol extraction using regex
fn extract_symbols_simple(content: &str, file: &str) -> Result<Vec<Symbol>> {
    let mut symbols = Vec::new();

    // Function patterns for different languages.
    //
    // Issue #693: every pattern used to be anchored at `(?m)^` with no leading
    // whitespace allowed, and they are applied one line at a time below — so
    // `^` could only ever match column 0 and *any indented declaration was
    // invisible by construction*: `impl Widget { pub fn new }`, a trait's own
    // `fn draw`, anything inside an inline `mod inner { … }`. A 12-declaration
    // fixture reported 5. Each anchor is now `^\s*`; `detect_visibility` below
    // takes the text before the name, so a leading indent in that prefix is
    // harmless. `trait` and `type` had no pattern at all, and the `class`
    // pattern had no `export` prefix — which is why `class PlainClass` was
    // found while `export class ExportedClass` right above it was not.
    let patterns = &*SYMBOL_PATTERNS;

    for (line_no, line) in content.lines().enumerate() {
        for (pattern, kind) in patterns {
            if let Some(captures) = pattern.captures(line) {
                if let Some(name) = captures.get(1) {
                    symbols.push(Symbol {
                        name: name.as_str().to_string(),
                        kind: kind.clone(),
                        file: file.to_string(),
                        line: line_no + 1,
                        column: name.start(),
                        // Only the text BEFORE the declared name can be a
                        // modifier of it. Passing the whole line reported the
                        // private `struct PrivType { pub b: u32 }` as Public,
                        // because a public *field* put "pub " somewhere on the
                        // line. (The same struct spread over several lines was
                        // correctly Internal, which is the tell.)
                        visibility: detect_visibility(&line[..name.start()]),
                        references: vec![Reference {
                            file: file.to_string(),
                            line: line_no + 1,
                            column: name.start(),
                            kind: ReferenceKind::Definition,
                        }],
                    });
                }
            }
        }
    }

    Ok(symbols)
}

/// Detect visibility from the modifiers that precede the declared name.
///
/// `prefix` must be the part of the declaration line *before* the symbol's own
/// name — anything after it belongs to the body, not to the declaration.
fn detect_visibility(prefix: &str) -> Visibility {
    if prefix.contains("pub ") || prefix.contains("export ") {
        Visibility::Public
    } else if prefix.contains("private ") {
        Visibility::Private
    } else if prefix.contains("protected ") {
        Visibility::Protected
    } else {
        Visibility::Internal
    }
}

/// Total resolved use sites per symbol name (definitions excluded), so that a
/// name declared in several files is reported once rather than once per file.
fn usage_counts_by_name(symbols: &[Symbol]) -> HashMap<&str, usize> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for symbol in symbols {
        *counts.entry(symbol.name.as_str()).or_insert(0) += usage_count(symbol);
    }
    counts
}

/// Names with zero resolved use sites.
///
/// Defect #654 (round 2): this used to be `references.len() <= 1`, and every
/// symbol had exactly one reference (its own `Definition`), so it returned every
/// symbol in the tree — 16944 of 16944, including `helper_one`, which a fixture
/// proved was called from both `main()` and `helper_two()`. It now counts real
/// use sites, and a name whose uses could not be attributed (see
/// `resolve_references`) is omitted rather than falsely declared unreferenced.
fn find_unreferenced_symbols(symbols: &[Symbol], unresolved: &HashSet<String>) -> Vec<String> {
    let counts = usage_counts_by_name(symbols);
    let mut names: Vec<String> = counts
        .into_iter()
        .filter(|(name, count)| *count == 0 && !unresolved.contains(*name))
        .map(|(name, _)| name.to_string())
        .collect();
    names.sort();
    names
}

/// Top symbol names by resolved use sites, highest first, name-ascending on a
/// tie so the list is identical across runs, plus **how many names there were in
/// total**. Names with no resolved use are omitted — a "most referenced" list of
/// zeros is not a measurement.
///
/// `limit` is `--top-files`; 0 means "all". The list used to be `truncate(10)`
/// with the flag discarded and no total reported, so a project with 11 000
/// referenced names and one with 11 produced the same-shaped 10-entry list.
fn find_most_referenced(symbols: &[Symbol], limit: usize) -> (Vec<(String, usize)>, usize) {
    let mut refs: Vec<(String, usize)> = usage_counts_by_name(symbols)
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(name, count)| (name.to_string(), count))
        .collect();

    refs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let total = refs.len();
    if limit > 0 {
        refs.truncate(limit);
    }
    (refs, total)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod keyword_capture_tests {
    use super::*;

    /// `const fn answer()` used to produce a `Constant` named "fn" (the keyword
    /// captured by the constant regex) and no `answer` at all, because the
    /// function regex had no `const` alternative. Repo-wide that bogus "fn"
    /// topped `most_referenced` with 54 266 textual "references".
    #[test]
    fn const_fn_is_a_function_named_after_the_fn_not_the_keyword() {
        let content = "const fn answer() -> i32 { 42 }\npub fn other() -> i32 { answer() }\n";
        let symbols = extract_symbols_simple(content, "lib.rs").unwrap();

        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(
            !names.contains(&"fn"),
            "the `fn` keyword must never become a symbol: {names:?}"
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "answer" && matches!(s.kind, SymbolKind::Function)),
            "`const fn answer` must be a Function named answer: {names:?}"
        );
    }

    /// Same keyword-capture trap on the `static` pattern.
    #[test]
    fn static_mut_is_named_after_the_variable_not_the_mut_keyword() {
        let content = "pub static mut COUNTER: u32 = 0;\n";
        let symbols = extract_symbols_simple(content, "lib.rs").unwrap();

        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(!names.contains(&"mut"), "captured the keyword: {names:?}");
        assert!(names.contains(&"COUNTER"), "lost the variable: {names:?}");
    }

    /// The constant/variable patterns must still find the ordinary forms.
    #[test]
    fn plain_constants_and_statics_are_still_extracted() {
        let content = "pub const KONST: u32 = 1;\npub static STAT: u32 = 2;\nconst js = 3;\n";
        let symbols = extract_symbols_simple(content, "lib.rs").unwrap();
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"KONST"), "{names:?}");
        assert!(names.contains(&"STAT"), "{names:?}");
        assert!(names.contains(&"js"), "{names:?}");
    }

    /// Issue #693: every pattern was `(?m)^`-anchored and applied per line, so
    /// a declaration that is not at column 0 could never match. This exact
    /// fixture reported 5 of its declarations; the ones below were all silently
    /// absent from a table that still printed `total_symbols` as if it were a
    /// count of what is there.
    #[test]
    fn indented_declarations_are_not_invisible() {
        let content = concat!(
            "pub struct Widget {\n",
            "    pub id: u32,\n",
            "}\n",
            "\n",
            "pub trait Drawable {\n",
            "    fn draw(&self);\n",
            "}\n",
            "\n",
            "impl Widget {\n",
            "    pub fn new() -> Self { Widget { id: 0 } }\n",
            "    fn helper(&self) -> u32 { self.id }\n",
            "}\n",
            "\n",
            "pub type WidgetAlias = Widget;\n",
            "\n",
            "mod inner {\n",
            "    pub fn nested_fn() {}\n",
            "}\n",
            "\n",
            "pub fn top_level() {}\n",
        );
        let symbols = extract_symbols_simple(content, "a.rs").expect("extract");
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();

        for expected in [
            "Widget",
            "Drawable",
            "draw",
            "new",
            "helper",
            "WidgetAlias",
            "inner",
            "nested_fn",
            "top_level",
        ] {
            assert!(
                names.contains(&expected),
                "`{expected}` is declared in the fixture but missing from the table: {names:?}"
            );
        }

        // An indented `pub fn` is public; the indent must not be mistaken for a
        // missing `pub` (or vice versa).
        let new_fn = symbols
            .iter()
            .find(|s| s.name == "new")
            .expect("`new` must be extracted");
        assert!(
            matches!(new_fn.visibility, Visibility::Public),
            "indented `pub fn new` reported as {:?}",
            new_fn.visibility
        );
        let helper = symbols
            .iter()
            .find(|s| s.name == "helper")
            .expect("`helper` must be extracted");
        assert!(
            matches!(helper.visibility, Visibility::Internal),
            "indented private `fn helper` reported as {:?}",
            helper.visibility
        );
    }

    /// Issue #693's tell: `class PlainClass` was found while
    /// `export class ExportedClass` on the line above it was not, because the
    /// class pattern had no `(?:export\s+)?` prefix — so a TypeScript file's
    /// *exported* types were exactly the ones missing.
    #[test]
    fn exported_typescript_declarations_are_extracted() {
        let content = concat!(
            "export class ExportedClass {\n",
            "    method() { return 1; }\n",
            "}\n",
            "class PlainClass {}\n",
            "export function exportedFn() {}\n",
            "export interface Shape {}\n",
            "export type Alias = Shape;\n",
        );
        let symbols = extract_symbols_simple(content, "b.ts").expect("extract");
        let names: Vec<&str> = symbols.iter().map(|s| s.name.as_str()).collect();

        for expected in [
            "ExportedClass",
            "PlainClass",
            "exportedFn",
            "Shape",
            "Alias",
        ] {
            assert!(names.contains(&expected), "missing `{expected}`: {names:?}");
        }
    }
}
