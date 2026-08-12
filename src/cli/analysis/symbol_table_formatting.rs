// Symbol table filtering and output formatting

// Apply filters to symbol table
fn apply_filters(
    mut table: SymbolTable,
    filter: Option<crate::cli::SymbolTypeFilter>,
    query: Option<String>,
    top_files: usize,
) -> Result<SymbolTable> {
    // Filter by type
    if let Some(type_filter) = filter {
        table.symbols.retain(|s| match type_filter {
            crate::cli::SymbolTypeFilter::Functions => {
                s.kind == SymbolKind::Function || s.kind == SymbolKind::Method
            }
            crate::cli::SymbolTypeFilter::Classes => s.kind == SymbolKind::Class,
            crate::cli::SymbolTypeFilter::Types => {
                s.kind == SymbolKind::Type
                    || s.kind == SymbolKind::Interface
                    || s.kind == SymbolKind::Enum
            }
            crate::cli::SymbolTypeFilter::Variables => {
                s.kind == SymbolKind::Variable || s.kind == SymbolKind::Constant
            }
            crate::cli::SymbolTypeFilter::Modules => s.kind == SymbolKind::Module,
            crate::cli::SymbolTypeFilter::All => true,
        });
    }

    // Filter by query
    if let Some(q) = query {
        let q_lower = q.to_lowercase();
        table
            .symbols
            .retain(|s| s.name.to_lowercase().contains(&q_lower));
    }

    Ok(rederive_summary(table, top_files))
}

/// Recompute the header/summary fields from the symbols that survived filtering.
///
/// Defect #654 (round 2): `--filter` retained only the matching symbols but left
/// `total_symbols` at the pre-filter value — on a fixture with one struct and one
/// function, `--filter functions` reported `total_symbols: 2` above a 1-element
/// `symbols` array, and `unreferenced_symbols` still named the filtered-out
/// struct. The header now counts the list it heads.
fn rederive_summary(mut table: SymbolTable, top_files: usize) -> SymbolTable {
    let retained: std::collections::HashSet<String> =
        table.symbols.iter().map(|s| s.name.clone()).collect();

    table.total_symbols = table.symbols.len();
    table
        .unreferenced_symbols
        .retain(|name| retained.contains(name));
    let (most_referenced, referenced_symbol_count) =
        find_most_referenced(&table.symbols, top_files);
    table.most_referenced = most_referenced;
    table.referenced_symbol_count = referenced_symbol_count;
    table
}

/// Format symbol table output based on format type
///
/// # Examples
///
/// ```no_run
/// use pmat::cli::analysis::symbol_table::{format_output, SymbolTable, Symbol, SymbolKind, Visibility, Reference, ReferenceKind};
/// use pmat::cli::SymbolTableOutputFormat;
///
/// let table = SymbolTable {
///     symbols: vec![
///         Symbol {
///             name: "test_function".to_string(),
///             kind: SymbolKind::Function,
///             file: "src/main.rs".to_string(),
///             line: 10,
///             column: 4,
///             visibility: Visibility::Public,
///             references: vec![Reference {
///                 file: "src/main.rs".to_string(),
///                 line: 10,
///                 column: 4,
///                 kind: ReferenceKind::Definition,
///             }],
///         },
///         Symbol {
///             name: "TestStruct".to_string(),
///             kind: SymbolKind::Type,
///             file: "src/lib.rs".to_string(),
///             line: 5,
///             column: 0,
///             visibility: Visibility::Public,
///             references: vec![],
///         },
///     ],
///     total_symbols: 2,
///     unreferenced_symbols: vec!["TestStruct".to_string()],
///     most_referenced: vec![("test_function".to_string(), 1)],
///     referenced_symbol_count: 1,
/// };
///
/// let output = format_output(table, SymbolTableOutputFormat::Summary, true, false, 10).unwrap();
/// assert!(output.contains("Top Files by Symbol Count"));
/// assert!(output.contains("main.rs"));
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_output(
    table: SymbolTable,
    format: crate::cli::SymbolTableOutputFormat,
    show_unreferenced: bool,
    show_references: bool,
    top_files: usize,
) -> Result<String> {
    match format {
        crate::cli::SymbolTableOutputFormat::Json => format_json_output(&table),
        crate::cli::SymbolTableOutputFormat::Human
        | crate::cli::SymbolTableOutputFormat::Summary => format_human_output(
            table,
            HumanRender {
                show_unreferenced,
                show_references,
                max_per_group: SYMBOLS_PER_GROUP_IN_SUMMARY,
                top_files,
            },
        ),
        // `--help` advertises detailed as "Detailed output with all symbols";
        // it used to be byte-identical to `summary`, listing 10 per group and
        // no reference sites. It now lists every symbol with where it is used.
        crate::cli::SymbolTableOutputFormat::Detailed => format_human_output(
            table,
            HumanRender {
                show_unreferenced,
                show_references: true,
                max_per_group: usize::MAX,
                top_files,
            },
        ),
        crate::cli::SymbolTableOutputFormat::Csv => format_csv_output(table),
    }
}

/// What the text renderers should include.
struct HumanRender {
    show_unreferenced: bool,
    show_references: bool,
    max_per_group: usize,
    /// `--top-files`; 0 means "all". Applies to both truncated tables.
    top_files: usize,
}

/// Format JSON output (cognitive complexity ≤2)
fn format_json_output(table: &SymbolTable) -> Result<String> {
    Ok(serde_json::to_string_pretty(table)?)
}

/// Format human-readable output (cognitive complexity ≤8)
fn format_human_output(table: SymbolTable, opts: HumanRender) -> Result<String> {
    let mut output = String::new();

    write_header(&mut output, table.total_symbols)?;
    write_symbols_by_type(&mut output, &table.symbols, &opts)?;

    if opts.show_unreferenced {
        write_unreferenced_symbols(&mut output, &table.unreferenced_symbols)?;
    }

    write_most_referenced(
        &mut output,
        &table.most_referenced,
        table.referenced_symbol_count,
    )?;
    write_top_files_by_count(&mut output, &table.symbols, opts.top_files)?;

    Ok(output)
}

/// Write header section (cognitive complexity ≤3)
///
/// Every renderer below goes through `crate::cli::colors`' helper functions
/// rather than the raw `pub const` escape sequences. With the constants,
/// `analyze symbol-table --format summary --color never` emitted 16 lines of
/// raw ESC bytes — identical to `--color always`, and written *into* the file
/// given by `-o` — while sibling commands (`analyze complexity`, `dead-code`)
/// emitted none. The helpers consult `colors_enabled()`; the constants cannot.
fn write_header(output: &mut String, total_symbols: usize) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(output, "{}\n", c::header("Symbol Table Analysis"))?;
    writeln!(
        output,
        "  {} {}",
        c::label("Total symbols:"),
        c::number(&total_symbols.to_string())
    )?;
    writeln!(output, "\n{}\n", c::subheader("Symbols by Type"))?;
    Ok(())
}

/// Write symbols grouped by type (cognitive complexity ≤8)
fn write_symbols_by_type(
    output: &mut String,
    symbols: &[Symbol],
    opts: &HumanRender,
) -> Result<()> {
    for (kind, syms) in group_symbols_by_type(symbols) {
        write_symbol_group(output, &kind, &syms, opts)?;
    }

    Ok(())
}

/// Group symbols by their kind (cognitive complexity ≤4)
///
/// A `BTreeMap` (not a `HashMap`) so the groups come out in `SymbolKind`
/// declaration order: iterating a `HashMap` reordered the sections on every run,
/// which made two runs over an unchanged tree produce different bytes.
fn group_symbols_by_type(symbols: &[Symbol]) -> BTreeMap<SymbolKind, Vec<&Symbol>> {
    let mut by_type: BTreeMap<SymbolKind, Vec<&Symbol>> = BTreeMap::new();
    for symbol in symbols {
        by_type.entry(symbol.kind.clone()).or_default().push(symbol);
    }
    by_type
}

/// Write a single symbol group (cognitive complexity ≤6)
fn write_symbol_group(
    output: &mut String,
    kind: &SymbolKind,
    syms: &[&Symbol],
    opts: &HumanRender,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    writeln!(
        output,
        "{} ({})",
        c::subheader(&format!("{kind:?}")),
        c::number(&syms.len().to_string())
    )?;

    for sym in syms.iter().take(opts.max_per_group) {
        writeln!(
            output,
            "  - {}  {}",
            c::number(&sym.name),
            c::path(&format!("{}:{}", sym.file, sym.line))
        )?;
        if opts.show_references {
            write_reference_sites(output, sym)?;
        }
    }

    if syms.len() > opts.max_per_group {
        writeln!(
            output,
            "  {}",
            c::dim(&format!("... and {} more", syms.len() - opts.max_per_group))
        )?;
    }

    writeln!(output)?;
    Ok(())
}

/// Write the resolved use sites for one symbol.
///
/// `--show-references` used to be bound to `_show_references` and discarded, so
/// the flag changed nothing in the output. It now renders the sites that
/// `resolve_references` actually attributed, and says so when there are none.
fn write_reference_sites(output: &mut String, sym: &Symbol) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    let sites: Vec<String> = sym
        .references
        .iter()
        .filter(|r| !matches!(r.kind, ReferenceKind::Definition))
        .take(REFERENCE_SITES_SHOWN)
        // `mod.rs:13` is not a location a reader can open; the path is.
        .map(|r| {
            format!(
                "{}:{}",
                crate::cli::report_paths::report_path(&r.file),
                r.line
            )
        })
        .collect();

    if sites.is_empty() {
        writeln!(output, "      {}", c::dim("used at: none resolved"))?;
        return Ok(());
    }

    let total = usage_count(sym);
    let suffix = if total > sites.len() {
        format!(" (+{} more)", total - sites.len())
    } else {
        String::new()
    };
    writeln!(
        output,
        "      {}",
        c::dim(&format!("used at: {}{}", sites.join(", "), suffix))
    )?;
    Ok(())
}

/// Write unreferenced symbols section (cognitive complexity ≤5)
fn write_unreferenced_symbols(output: &mut String, unreferenced: &[String]) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    if unreferenced.is_empty() {
        return Ok(());
    }

    writeln!(output, "\n{}\n", c::subheader("Unreferenced Symbols"))?;
    for name in unreferenced {
        writeln!(output, "  - {}", c::colored(c::YELLOW, name))?;
    }

    Ok(())
}

/// Heading for a list that shows only the top `shown` of `total`.
///
/// Naming both numbers is the whole point: `Most Referenced Symbols` used to be
/// a fixed 10 entries with nothing indicating that anything had been left out,
/// which is a cap wearing the shape of a total.
fn truncated_heading(title: &str, shown: usize, total: usize) -> String {
    use crate::cli::colors as c;
    if shown >= total {
        return c::subheader(&format!("{title} ({total})"));
    }
    c::subheader(&format!("{title} (top {shown} of {total})"))
}

/// Write most referenced symbols section (cognitive complexity ≤5)
fn write_most_referenced(
    output: &mut String,
    most_referenced: &[(String, usize)],
    referenced_total: usize,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    if most_referenced.is_empty() {
        return Ok(());
    }

    writeln!(
        output,
        "\n{}\n",
        truncated_heading(
            "Most Referenced Symbols",
            most_referenced.len(),
            referenced_total
        )
    )?;
    for (name, count) in most_referenced {
        writeln!(
            output,
            "  - {}: {} references",
            c::number(name),
            c::number(&count.to_string())
        )?;
    }

    Ok(())
}

/// Write top files by symbol count (cognitive complexity ≤8)
///
/// `top_files` is `--top-files`; 0 means "all". It used to be a hard `take(10)`
/// with the flag discarded, so the table showed the same 10 rows whether the
/// project had 11 files or 3868 and said nothing about the rest.
fn write_top_files_by_count(
    output: &mut String,
    symbols: &[Symbol],
    top_files: usize,
) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    if symbols.is_empty() {
        return Ok(());
    }

    let sorted_files = get_sorted_file_counts(symbols);
    let shown = if top_files == 0 {
        sorted_files.len()
    } else {
        top_files.min(sorted_files.len())
    };

    writeln!(
        output,
        "\n{}\n",
        truncated_heading("Top Files by Symbol Count", shown, sorted_files.len())
    )?;

    for (i, (file_path, count)) in sorted_files.iter().take(shown).enumerate() {
        // Basenames do not identify a file in this tree — three of the ten rows
        // used to read `mod.rs`, `tests.rs`, `types.rs`. Print the path the
        // symbol table is keyed by, which is what `--format json` reports.
        let filename = crate::cli::report_paths::report_path(file_path);
        writeln!(
            output,
            "{}. {} - {} symbols",
            c::subheader(&(i + 1).to_string()),
            c::path(filename),
            c::number(&count.to_string())
        )?;
    }

    Ok(())
}

/// Get file counts sorted by symbol count (cognitive complexity ≤5)
///
/// Ties break on the path, not on `HashMap` iteration order: the previous
/// `sort_by_key(Reverse(count))` left equal-count files in hash order, so the
/// "Top Files" table was reshuffled on every run of the same command.
fn get_sorted_file_counts(symbols: &[Symbol]) -> Vec<(&str, usize)> {
    let mut file_counts: HashMap<&str, usize> = HashMap::new();

    for symbol in symbols {
        *file_counts.entry(&symbol.file).or_insert(0) += 1;
    }

    let mut sorted_files: Vec<_> = file_counts.into_iter().collect();
    sorted_files.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    sorted_files
}


/// Format CSV output (cognitive complexity ≤5)
fn format_csv_output(table: SymbolTable) -> Result<String> {
    use std::fmt::Write;
    let mut output = String::new();

    writeln!(&mut output, "name,kind,file,line,column,visibility")?;

    for sym in table.symbols {
        writeln!(
            &mut output,
            "{},{:?},{},{},{},{:?}",
            sym.name, sym.kind, sym.file, sym.line, sym.column, sym.visibility
        )?;
    }

    Ok(output)
}
