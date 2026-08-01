// Symbol table filtering and output formatting

// Apply filters to symbol table
fn apply_filters(
    mut table: SymbolTable,
    filter: Option<crate::cli::SymbolTypeFilter>,
    query: Option<String>,
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

    Ok(rederive_summary(table))
}

/// Recompute the header/summary fields from the symbols that survived filtering.
///
/// Defect #654 (round 2): `--filter` retained only the matching symbols but left
/// `total_symbols` at the pre-filter value — on a fixture with one struct and one
/// function, `--filter functions` reported `total_symbols: 2` above a 1-element
/// `symbols` array, and `unreferenced_symbols` still named the filtered-out
/// struct. The header now counts the list it heads.
fn rederive_summary(mut table: SymbolTable) -> SymbolTable {
    let retained: std::collections::HashSet<String> =
        table.symbols.iter().map(|s| s.name.clone()).collect();

    table.total_symbols = table.symbols.len();
    table
        .unreferenced_symbols
        .retain(|name| retained.contains(name));
    table.most_referenced = find_most_referenced(&table.symbols);
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
/// };
///
/// let output = format_output(table, SymbolTableOutputFormat::Summary, true, false).unwrap();
/// assert!(output.contains("Top Files by Symbol Count"));
/// assert!(output.contains("main.rs"));
/// ```
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn format_output(
    table: SymbolTable,
    format: crate::cli::SymbolTableOutputFormat,
    show_unreferenced: bool,
    show_references: bool,
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

    write_most_referenced(&mut output, &table.most_referenced)?;
    write_top_files_by_count(&mut output, &table.symbols)?;

    Ok(output)
}

/// Write header section (cognitive complexity ≤3)
fn write_header(output: &mut String, total_symbols: usize) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;
    writeln!(
        output,
        "{}{}Symbol Table Analysis{}\n",
        c::BOLD, c::UNDERLINE, c::RESET
    )?;
    writeln!(
        output,
        "  {}Total symbols:{} {}{}{}",
        c::BOLD, c::RESET, c::BOLD_WHITE, total_symbols, c::RESET
    )?;
    writeln!(output, "\n{}Symbols by Type{}\n", c::BOLD, c::RESET)?;
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
        "{}{:?}{} ({}{}{})",
        c::BOLD, kind, c::RESET, c::BOLD_WHITE, syms.len(), c::RESET
    )?;

    for sym in syms.iter().take(opts.max_per_group) {
        writeln!(
            output,
            "  - {}{}{}  {}{}:{}{}",
            c::BOLD_WHITE, sym.name, c::RESET, c::CYAN, sym.file, sym.line, c::RESET
        )?;
        if opts.show_references {
            write_reference_sites(output, sym)?;
        }
    }

    if syms.len() > opts.max_per_group {
        writeln!(
            output,
            "  {}... and {} more{}",
            c::DIM,
            syms.len() - opts.max_per_group,
            c::RESET
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
        .map(|r| format!("{}:{}", extract_filename(&r.file), r.line))
        .collect();

    if sites.is_empty() {
        writeln!(output, "      {}used at: none resolved{}", c::DIM, c::RESET)?;
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
        "      {}used at: {}{}{}",
        c::DIM,
        sites.join(", "),
        suffix,
        c::RESET
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

    writeln!(output, "\n{}Unreferenced Symbols{}\n", c::BOLD, c::RESET)?;
    for name in unreferenced {
        writeln!(output, "  - {}{}{}", c::YELLOW, name, c::RESET)?;
    }

    Ok(())
}

/// Write most referenced symbols section (cognitive complexity ≤5)
fn write_most_referenced(output: &mut String, most_referenced: &[(String, usize)]) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    if most_referenced.is_empty() {
        return Ok(());
    }

    writeln!(output, "\n{}Most Referenced Symbols{}\n", c::BOLD, c::RESET)?;
    for (name, count) in most_referenced {
        writeln!(
            output,
            "  - {}{}{}: {}{}{} references",
            c::BOLD_WHITE, name, c::RESET, c::BOLD_WHITE, count, c::RESET
        )?;
    }

    Ok(())
}

/// Write top files by symbol count (cognitive complexity ≤8)
fn write_top_files_by_count(output: &mut String, symbols: &[Symbol]) -> Result<()> {
    use crate::cli::colors as c;
    use std::fmt::Write;

    if symbols.is_empty() {
        return Ok(());
    }

    writeln!(output, "\n{}Top Files by Symbol Count{}\n", c::BOLD, c::RESET)?;

    let sorted_files = get_sorted_file_counts(symbols);

    for (i, (file_path, count)) in sorted_files.iter().take(10).enumerate() {
        let filename = extract_filename(file_path);
        writeln!(
            output,
            "{}{}. {}{}{} - {}{}{} symbols",
            c::BOLD, i + 1, c::CYAN, filename, c::RESET,
            c::BOLD_WHITE, count, c::RESET
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

/// Extract filename from path (cognitive complexity ≤3)
fn extract_filename(file_path: &str) -> &str {
    Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(file_path)
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
