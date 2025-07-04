//! Symbol table analysis - extracts and analyzes symbols from code

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub visibility: Visibility,
    pub references: Vec<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    Class,
    Method,
    Variable,
    Constant,
    Type,
    Interface,
    Enum,
    Module,
    Property,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub kind: ReferenceKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReferenceKind {
    Definition,
    Usage,
    Import,
    Export,
}

#[derive(Debug, Serialize)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
    pub total_symbols: usize,
    pub unreferenced_symbols: Vec<String>,
    pub most_referenced: Vec<(String, usize)>,
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_analyze_symbol_table(
    project_path: PathBuf,
    format: crate::cli::SymbolTableOutputFormat,
    filter: Option<crate::cli::SymbolTypeFilter>,
    query: Option<String>,
    include: Option<String>,
    exclude: Option<String>,
    show_unreferenced: bool,
    show_references: bool,
    output: Option<PathBuf>,
    _perf: bool,
) -> Result<()> {
    eprintln!("🔍 Building symbol table for project...");

    // Build the symbol table
    let table = build_symbol_table(&project_path, &include, &exclude).await?;

    // Apply filters
    let filtered = apply_filters(table, filter, query)?;

    // Format output
    let content = format_output(filtered, format, show_unreferenced, show_references)?;

    // Write output
    if let Some(output_path) = output {
        tokio::fs::write(&output_path, &content).await?;
        eprintln!("✅ Symbol table written to: {}", output_path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

// Build symbol table from project files
async fn build_symbol_table(
    project_path: &Path,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<SymbolTable> {
    let mut symbols = Vec::new();

    // Get all relevant files
    let files = collect_files(project_path, include, exclude).await?;

    // Extract symbols from each file
    for file in files {
        let file_symbols = extract_symbols_from_file(&file).await?;
        symbols.extend(file_symbols);
    }

    // Find unreferenced symbols
    let unreferenced = find_unreferenced_symbols(&symbols);

    // Find most referenced symbols
    let most_referenced = find_most_referenced(&symbols);

    Ok(SymbolTable {
        total_symbols: symbols.len(),
        symbols,
        unreferenced_symbols: unreferenced,
        most_referenced,
    })
}

// Collect files based on include/exclude patterns
async fn collect_files(
    project_path: &Path,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    collect_files_recursive(project_path, &mut files, include, exclude).await?;

    Ok(files)
}

// Recursively collect files
async fn collect_files_recursive(
    dir: &Path,
    files: &mut Vec<PathBuf>,
    include: &Option<String>,
    exclude: &Option<String>,
) -> Result<()> {
    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let path_str = path.to_string_lossy();

        // Skip excluded paths
        if let Some(excl) = exclude {
            if path_str.contains(excl) {
                continue;
            }
        }

        if path.is_dir() {
            // Skip hidden and vendor directories
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !name.starts_with('.') && name != "node_modules" && name != "target" {
                Box::pin(collect_files_recursive(&path, files, include, exclude)).await?;
            }
        } else if is_source_file(&path) {
            // Check include pattern
            if let Some(incl) = include {
                if path_str.contains(incl) {
                    files.push(path);
                }
            } else {
                files.push(path);
            }
        }
    }

    Ok(())
}

// Check if file is a source file
fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs")
            | Some("js")
            | Some("ts")
            | Some("py")
            | Some("java")
            | Some("cpp")
            | Some("c")
            | Some("h")
            | Some("hpp")
            | Some("go")
            | Some("rb")
    )
}

// Extract symbols from a single file
async fn extract_symbols_from_file(file_path: &Path) -> Result<Vec<Symbol>> {
    let content = tokio::fs::read_to_string(file_path).await?;
    let file_str = file_path.to_string_lossy().to_string();

    // Use simple regex-based extraction for now
    extract_symbols_simple(&content, &file_str)
}

// Simple symbol extraction using regex
fn extract_symbols_simple(content: &str, file: &str) -> Result<Vec<Symbol>> {
    use regex::Regex;

    let mut symbols = Vec::new();

    // Function patterns for different languages
    let patterns = vec![
        (
            Regex::new(r"(?m)^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)")?,
            SymbolKind::Function,
        ),
        (Regex::new(r"(?m)^class\s+(\w+)")?, SymbolKind::Class),
        (
            Regex::new(r"(?m)^(?:export\s+)?(?:async\s+)?function\s+(\w+)")?,
            SymbolKind::Function,
        ),
        (Regex::new(r"(?m)^def\s+(\w+)")?, SymbolKind::Function),
        (Regex::new(r"(?m)^const\s+(\w+)\s*=")?, SymbolKind::Constant),
        (
            Regex::new(r"(?m)^(?:pub\s+)?struct\s+(\w+)")?,
            SymbolKind::Type,
        ),
        (
            Regex::new(r"(?m)^(?:pub\s+)?enum\s+(\w+)")?,
            SymbolKind::Enum,
        ),
        (
            Regex::new(r"(?m)^interface\s+(\w+)")?,
            SymbolKind::Interface,
        ),
    ];

    for (line_no, line) in content.lines().enumerate() {
        for (pattern, kind) in &patterns {
            if let Some(captures) = pattern.captures(line) {
                if let Some(name) = captures.get(1) {
                    symbols.push(Symbol {
                        name: name.as_str().to_string(),
                        kind: kind.clone(),
                        file: file.to_string(),
                        line: line_no + 1,
                        column: name.start(),
                        visibility: detect_visibility(line),
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

// Detect visibility from line content
fn detect_visibility(line: &str) -> Visibility {
    if line.contains("pub ") || line.contains("export ") {
        Visibility::Public
    } else if line.contains("private ") {
        Visibility::Private
    } else if line.contains("protected ") {
        Visibility::Protected
    } else {
        Visibility::Internal
    }
}

// Find unreferenced symbols
fn find_unreferenced_symbols(symbols: &[Symbol]) -> Vec<String> {
    symbols
        .iter()
        .filter(|s| s.references.len() <= 1)
        .map(|s| s.name.clone())
        .collect()
}

// Find most referenced symbols
fn find_most_referenced(symbols: &[Symbol]) -> Vec<(String, usize)> {
    let mut refs: Vec<_> = symbols
        .iter()
        .map(|s| (s.name.clone(), s.references.len()))
        .collect();

    refs.sort_by(|a, b| b.1.cmp(&a.1));
    refs.truncate(10);
    refs
}

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

    Ok(table)
}

/// Format symbol table output based on format type
///
/// # Examples
///
/// ```
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
pub fn format_output(
    table: SymbolTable,
    format: crate::cli::SymbolTableOutputFormat,
    show_unreferenced: bool,
    _show_references: bool,
) -> Result<String> {
    use std::fmt::Write;

    match format {
        crate::cli::SymbolTableOutputFormat::Json => Ok(serde_json::to_string_pretty(&table)?),
        crate::cli::SymbolTableOutputFormat::Human
        | crate::cli::SymbolTableOutputFormat::Summary
        | crate::cli::SymbolTableOutputFormat::Detailed => {
            let mut output = String::new();
            writeln!(&mut output, "# Symbol Table Analysis\n")?;
            writeln!(&mut output, "Total symbols: {}", table.total_symbols)?;
            writeln!(&mut output, "\n## Symbols by Type\n")?;

            // Group by type
            let mut by_type: HashMap<SymbolKind, Vec<&Symbol>> = HashMap::new();
            for symbol in &table.symbols {
                by_type.entry(symbol.kind.clone()).or_default().push(symbol);
            }

            for (kind, syms) in by_type {
                writeln!(&mut output, "### {:?} ({})", kind, syms.len())?;
                for sym in syms.iter().take(10) {
                    writeln!(&mut output, "  - {} ({}:{})", sym.name, sym.file, sym.line)?;
                }
                if syms.len() > 10 {
                    writeln!(&mut output, "  ... and {} more", syms.len() - 10)?;
                }
                writeln!(&mut output)?;
            }

            if show_unreferenced && !table.unreferenced_symbols.is_empty() {
                writeln!(&mut output, "## Unreferenced Symbols\n")?;
                for name in &table.unreferenced_symbols {
                    writeln!(&mut output, "  - {}", name)?;
                }
            }

            if !table.most_referenced.is_empty() {
                writeln!(&mut output, "\n## Most Referenced Symbols\n")?;
                for (name, count) in &table.most_referenced {
                    writeln!(&mut output, "  - {}: {} references", name, count)?;
                }
            }

            // Show top files by symbol count
            if !table.symbols.is_empty() {
                writeln!(&mut output, "\n## Top Files by Symbol Count\n")?;
                
                // Count symbols per file
                let mut file_counts: HashMap<&str, usize> = HashMap::new();
                for symbol in &table.symbols {
                    *file_counts.entry(&symbol.file).or_insert(0) += 1;
                }
                
                // Sort files by symbol count
                let mut sorted_files: Vec<_> = file_counts.into_iter().collect();
                sorted_files.sort_by(|a, b| b.1.cmp(&a.1));
                
                // Display top 10 files
                for (i, (file_path, count)) in sorted_files.iter().take(10).enumerate() {
                    let filename = Path::new(file_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(file_path);
                    writeln!(
                        &mut output,
                        "{}. `{}` - {} symbols",
                        i + 1,
                        filename,
                        count
                    )?;
                }
            }

            Ok(output)
        }
        crate::cli::SymbolTableOutputFormat::Csv => {
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
    }
}

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
