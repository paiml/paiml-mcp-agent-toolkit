#![cfg_attr(coverage_nightly, coverage(off))]
//! Handler for `pmat split --auto` — automated file splitting based on line thresholds.
//!
//! Walks the project tree, finds `.rs` files exceeding a configurable line limit,
//! performs AST-aware split-point detection, and generates (or executes) a split plan.

use crate::cli::colors as c;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ── Data structures ──────────────────────────────────────────────────────────

/// A file that exceeds the maximum line threshold.
#[derive(Debug, Clone)]
pub struct OversizedFile {
    pub path: PathBuf,
    pub line_count: usize,
    pub estimated_splits: usize,
}

/// A top-level item detected in a Rust source file.
#[derive(Debug, Clone)]
pub struct TopLevelItem {
    /// The kind of item: "fn", "impl", "struct", "enum", "trait", "mod", "const", "static", "type"
    pub kind: String,
    /// The name extracted after the keyword
    pub name: String,
    /// Start line (1-based)
    pub start_line: usize,
    /// End line (1-based, inclusive)
    pub end_line: usize,
}

/// A cluster of items that should be kept together in a submodule.
#[derive(Debug, Clone)]
pub struct ItemCluster {
    pub items: Vec<TopLevelItem>,
    pub suggested_name: String,
    pub total_lines: usize,
}

/// The plan describing how a file will be split.
#[derive(Debug, Clone)]
pub struct SplitPlan {
    pub source_file: PathBuf,
    pub source_line_count: usize,
    pub use_block: Vec<String>,
    pub attrs: Vec<String>,
    pub clusters: Vec<ItemCluster>,
    pub remainder_items: Vec<TopLevelItem>,
    pub test_block: Option<(usize, usize)>,
}

/// A target file to be written as part of executing the plan.
#[derive(Debug, Clone)]
pub struct SplitTarget {
    pub path: PathBuf,
    pub content: String,
}

// ── Entry point ──────────────────────────────────────────────────────────────

/// Handle the `pmat split --auto` command.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn handle_split_auto(
    path: &Path,
    max_lines: usize,
    file: Option<&Path>,
    dry_run: bool,
    commit: bool,
) -> Result<()> {
    let project_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    println!("{}", c::header("Automated File Splitting"));
    println!(
        "{}: {}",
        c::dim("Project"),
        c::path(&project_path.display().to_string())
    );
    println!(
        "{}: {} lines",
        c::dim("Threshold"),
        c::number(&max_lines.to_string())
    );
    println!();

    // Find oversized files
    let oversized = if let Some(single_file) = file {
        // Single file mode within --auto
        let abs = if single_file.is_absolute() {
            single_file.to_path_buf()
        } else {
            project_path.join(single_file)
        };
        let content = std::fs::read_to_string(&abs)
            .with_context(|| format!("Failed to read {}", abs.display()))?;
        let line_count = content.lines().count();
        if line_count <= max_lines {
            println!(
                "{} {} is {} lines (under threshold of {}). Nothing to split.",
                c::pass(""),
                c::path(&single_file.display().to_string()),
                c::number(&line_count.to_string()),
                c::number(&max_lines.to_string()),
            );
            return Ok(());
        }
        vec![OversizedFile {
            path: abs,
            line_count,
            estimated_splits: estimate_splits(line_count, max_lines),
        }]
    } else {
        find_oversized_files(&project_path, max_lines)?
    };

    if oversized.is_empty() {
        println!(
            "{} No files exceed {} lines. Project is well-structured.",
            c::pass(""),
            c::number(&max_lines.to_string()),
        );
        return Ok(());
    }

    println!(
        "{} Found {} oversized file(s):\n",
        c::warn(""),
        c::number(&oversized.len().to_string()),
    );

    for f in &oversized {
        let rel = f.path.strip_prefix(&project_path).unwrap_or(&f.path);
        println!(
            "  {} {} lines, ~{} split(s)",
            c::path(&rel.display().to_string()),
            c::number(&f.line_count.to_string()),
            c::number(&f.estimated_splits.to_string()),
        );
    }
    println!();

    // Generate and display/execute plans
    for f in &oversized {
        let content = std::fs::read_to_string(&f.path)
            .with_context(|| format!("Failed to read {}", f.path.display()))?;

        // Check for include!() pattern
        if uses_include_pattern(&content) {
            let rel = f.path.strip_prefix(&project_path).unwrap_or(&f.path);
            println!(
                "{} Skipping {} — uses include!() pattern (already split)",
                c::dim("SKIP"),
                c::path(&rel.display().to_string()),
            );
            println!();
            continue;
        }

        let plan = generate_split_plan(&f.path, &content, max_lines);

        if plan.clusters.is_empty() {
            let rel = f.path.strip_prefix(&project_path).unwrap_or(&f.path);
            println!(
                "{} {} — no clear split points found",
                c::dim("SKIP"),
                c::path(&rel.display().to_string()),
            );
            println!();
            continue;
        }

        let rel = f.path.strip_prefix(&project_path).unwrap_or(&f.path);
        print_plan(&plan, rel);

        if !dry_run {
            let targets = build_split_targets(&plan, &content)?;
            execute_split_plan(&plan, &targets, commit)?;

            println!(
                "  {} Created {} submodule file(s)",
                c::pass(""),
                c::number(&targets.len().to_string()),
            );
            for t in &targets {
                let trel = t.path.strip_prefix(&project_path).unwrap_or(&t.path);
                println!("    {}", c::path(&trel.display().to_string()));
            }
            println!();
        }
    }

    if dry_run {
        println!(
            "{}",
            c::dim("Dry run — no files were modified. Use without --dry-run to execute."),
        );
    }

    Ok(())
}

// ── File discovery ───────────────────────────────────────────────────────────

/// Walk the project and find all `.rs` files exceeding `max_lines`.
fn find_oversized_files(project_path: &Path, max_lines: usize) -> Result<Vec<OversizedFile>> {
    let mut results = Vec::new();

    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_entry(|e| !should_skip_dir(e))
    {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();

        if path.extension().map_or(true, |ext| ext != "rs") {
            continue;
        }

        if should_skip_file(path) {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue, // skip unreadable files
        };

        let line_count = content.lines().count();
        if line_count > max_lines {
            results.push(OversizedFile {
                path: path.to_path_buf(),
                line_count,
                estimated_splits: estimate_splits(line_count, max_lines),
            });
        }
    }

    // Sort by line count descending (largest files first)
    results.sort_by_key(|b| std::cmp::Reverse(b.line_count));
    Ok(results)
}

/// Returns true if the directory entry should be skipped during traversal.
fn should_skip_dir(entry: &walkdir::DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    matches!(
        name.as_ref(),
        "target" | ".pmat" | ".git" | "node_modules" | ".cargo"
    )
}

/// Returns true if the file should be skipped.
fn should_skip_file(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    // Skip mod.rs — it's typically a module aggregator
    if file_name == "mod.rs" {
        return true;
    }

    // Skip files in test directories
    let path_str = path.to_string_lossy();
    if path_str.contains("/tests/") || path_str.contains("/benches/") {
        return true;
    }

    // Skip generated files (common patterns)
    if file_name.ends_with(".generated.rs") || file_name.starts_with("generated_") {
        return true;
    }

    false
}

fn estimate_splits(line_count: usize, max_lines: usize) -> usize {
    if max_lines == 0 {
        return 1;
    }
    // Number of splits = ceil(line_count / max_lines) - 1
    // (the original file stays, everything else is extracted)
    let chunks = line_count.div_ceil(max_lines);
    if chunks > 1 {
        chunks - 1
    } else {
        1
    }
}

// ── AST-aware split point detection ──────────────────────────────────────────

/// Returns true if a line starts a top-level Rust item (not indented).
fn is_top_level_item(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim_start();

    // Must start at column 0 (no indentation) or after `pub`/`pub(crate)` etc.
    if line.starts_with(' ') || line.starts_with('\t') {
        return None;
    }

    // Strip visibility prefix
    let after_vis = strip_visibility(trimmed);

    // Check item keywords
    let keywords = [
        "fn ",
        "async fn ",
        "struct ",
        "enum ",
        "trait ",
        "impl ",
        "impl<",
        "mod ",
        "const ",
        "static ",
        "type ",
        "macro_rules! ",
        "union ",
    ];

    for kw in &keywords {
        if let Some(rest) = after_vis.strip_prefix(kw) {
            let kind = kw.split_whitespace().next().unwrap_or(kw.trim());
            // Normalize "async fn" to "fn" and "impl<" to "impl"
            let kind = match kind {
                "async" => "fn",
                "macro_rules!" => "macro",
                _ => kind.trim_end_matches('<'),
            };
            let name = extract_item_name(rest);
            return Some((kind, name));
        }
    }

    None
}

/// Strip `pub`, `pub(crate)`, `pub(super)`, `pub(in ...)` from the start.
fn strip_visibility(s: &str) -> &str {
    if !s.starts_with("pub") {
        return s;
    }
    let after_pub = &s[3..];
    if after_pub.starts_with(' ') {
        return after_pub.trim_start();
    }
    if after_pub.starts_with('(') {
        // Find matching close paren
        if let Some(close) = after_pub.find(')') {
            return after_pub[close + 1..].trim_start();
        }
    }
    s
}

/// Extract the item name from the text after the keyword.
fn extract_item_name(rest: &str) -> &str {
    // Name is the first identifier-like token
    let name = rest
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .next()
        .unwrap_or("unknown");
    if name.is_empty() {
        "unknown"
    } else {
        name
    }
}

/// Detect split points: top-level items with their line ranges.
fn find_split_points(content: &str) -> Vec<TopLevelItem> {
    let lines: Vec<&str> = content.lines().collect();
    let mut items = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut current_item: Option<(String, String, usize)> = None; // (kind, name, start_line)
    let mut in_string = false;
    let mut in_block_comment_depth: i32 = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1; // 1-based
        let depth_before_line = brace_depth;

        // Process character by character for accurate brace counting
        let chars: Vec<char> = line.chars().collect();
        let mut j = 0;
        while j < chars.len() {
            let ch = chars[j];

            // Handle block comments
            if in_block_comment_depth > 0 {
                if ch == '*' && j + 1 < chars.len() && chars[j + 1] == '/' {
                    in_block_comment_depth -= 1;
                    j += 2;
                    continue;
                }
                if ch == '/' && j + 1 < chars.len() && chars[j + 1] == '*' {
                    in_block_comment_depth += 1;
                    j += 2;
                    continue;
                }
                j += 1;
                continue;
            }

            // Detect start of comments
            if ch == '/' && j + 1 < chars.len() {
                if chars[j + 1] == '/' {
                    // Line comment — skip rest of line
                    break;
                }
                if chars[j + 1] == '*' {
                    in_block_comment_depth += 1;
                    j += 2;
                    continue;
                }
            }

            // Handle strings
            if ch == '"' && !in_string {
                // Check for raw string
                if j > 0 && chars[j - 1] == 'r' {
                    // Skip raw string — consume until closing "
                    j += 1;
                    while j < chars.len() && chars[j] != '"' {
                        j += 1;
                    }
                    j += 1;
                    continue;
                }
                in_string = true;
                j += 1;
                continue;
            }
            if ch == '"' && in_string {
                in_string = false;
                j += 1;
                continue;
            }
            if in_string {
                if ch == '\\' {
                    j += 2; // skip escaped char
                    continue;
                }
                j += 1;
                continue;
            }

            // Handle char literals
            if ch == '\'' {
                // Skip char literal
                if j + 2 < chars.len() && chars[j + 2] == '\'' {
                    j += 3;
                    continue;
                }
                if j + 3 < chars.len() && chars[j + 1] == '\\' && chars[j + 3] == '\'' {
                    j += 4;
                    continue;
                }
            }

            // Count braces
            if ch == '{' {
                brace_depth += 1;
            } else if ch == '}' {
                brace_depth -= 1;
            }

            j += 1;
        }

        // Close current item when brace depth returns to the item's start depth.
        // Top-level items close at depth 0, impl-internal methods close at depth 1.
        if current_item.is_some() {
            let is_impl_method = current_item.as_ref().is_some_and(|(k, _, _)| k == "impl");
            let close_depth = if is_impl_method { 1 } else { 0 };
            if brace_depth == close_depth {
                if let Some((kind, name, start)) = current_item.take() {
                    items.push(TopLevelItem {
                        kind,
                        name,
                        start_line: start,
                        end_line: line_num,
                    });
                }
            }
        }

        // Detect new item only when no item is active.
        // At brace_depth 0: top-level items (fn, struct, impl, etc.)
        // At brace_depth 1: methods inside an impl block (indented by 4 spaces)
        if current_item.is_none() && depth_before_line <= 1 {
            let check_line = if depth_before_line == 1 {
                // Inside an impl block — strip standard 4-space indentation
                // so is_top_level_item can detect "pub fn method(..."
                line.strip_prefix("    ").unwrap_or(line)
            } else {
                line
            };
            if let Some((kind, name)) = is_top_level_item(check_line) {
                let attr_start = find_attribute_start(&lines, i);
                let start = attr_start + 1; // 1-based
                let effective_kind = if depth_before_line == 1 {
                    // Methods inside impl are always include!() targets
                    "impl"
                } else {
                    kind
                };
                current_item = Some((effective_kind.to_string(), name.to_string(), start));
            }
        }
    }

    // Close any unclosed item at EOF
    if let Some((kind, name, start)) = current_item.take() {
        items.push(TopLevelItem {
            kind,
            name,
            start_line: start,
            end_line: lines.len(),
        });
    }

    items
}

/// Walk backwards from a line to find the start of preceding doc comments/attributes.
fn find_attribute_start(lines: &[&str], item_line_idx: usize) -> usize {
    let mut start = item_line_idx;
    while start > 0 {
        let prev = lines[start - 1].trim();
        if prev.starts_with("///")
            || prev.starts_with("#[")
            || prev.starts_with("#![")
            || prev.starts_with("//!")
            || prev.is_empty()
        {
            // Empty lines between attributes and items are common — but stop at
            // double blank lines
            if prev.is_empty() && start >= 2 && lines[start - 2].trim().is_empty() {
                break;
            }
            start -= 1;
        } else {
            break;
        }
    }
    start
}

// ── Split plan generation ────────────────────────────────────────────────────

/// Returns true if the content uses `include!()` pattern (file is already split).
fn uses_include_pattern(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("include!(") || trimmed.starts_with("include!(concat!")
    })
}

/// Returns true if a line is a `use` statement.
fn is_use_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("use ")
        || trimmed.starts_with("pub use ")
        || trimmed.starts_with("pub(crate) use ")
}

/// Returns true if a line is a module-level attribute.
fn is_module_attr(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("#![")
}

/// Detect the `#[cfg(test)] mod tests` block boundaries.
fn find_test_block(items: &[TopLevelItem], lines: &[&str]) -> Option<(usize, usize)> {
    for item in items {
        if item.kind == "mod" && item.name == "tests" {
            // Check if any line in the item's range (or just before it) contains #[cfg(test)]
            // The item's start_line may already include the attribute due to find_attribute_start
            let search_start = if item.start_line >= 2 {
                item.start_line - 1 // 1 line before (0-based index)
            } else {
                0
            };
            let search_end = item.end_line.min(lines.len());

            for line_idx in search_start..search_end {
                let trimmed = lines[line_idx].trim();
                if trimmed.contains("cfg(test)") || trimmed.contains("cfg(all(test") {
                    return Some((item.start_line, item.end_line));
                }
                // Stop searching after we reach the mod keyword line
                if trimmed.starts_with("mod ") || trimmed.starts_with("pub mod ") {
                    break;
                }
            }
        }
    }
    None
}

/// Generate a split plan for a single file.
fn generate_split_plan(file_path: &Path, content: &str, max_lines: usize) -> SplitPlan {
    let lines: Vec<&str> = content.lines().collect();
    let items = find_split_points(content);

    // Separate use statements and module attributes
    let mut use_block = Vec::new();
    let mut attrs = Vec::new();
    let mut first_item_line = lines.len();

    if let Some(first) = items.first() {
        first_item_line = first.start_line.saturating_sub(1); // 0-based index
    }

    for (i, line) in lines.iter().enumerate() {
        if i >= first_item_line {
            break;
        }
        if is_module_attr(line) {
            attrs.push(line.to_string());
        } else if is_use_line(line) {
            use_block.push(line.to_string());
        }
    }

    // Find test block
    let test_block = find_test_block(&items, &lines);

    // Filter out test block items from splittable items
    let splittable_items: Vec<TopLevelItem> = items
        .into_iter()
        .filter(|item| {
            if let Some((test_start, test_end)) = test_block {
                // Skip anything overlapping with the test block
                !(item.start_line >= test_start && item.end_line <= test_end)
            } else {
                true
            }
        })
        .collect();

    // Group items into clusters that fit within max_lines
    let clusters = group_items_into_clusters(&splittable_items, max_lines, file_path);

    SplitPlan {
        source_file: file_path.to_path_buf(),
        source_line_count: lines.len(),
        use_block,
        attrs,
        clusters,
        remainder_items: Vec::new(), // Items that stay in the original file
        test_block,
    }
}

/// Group items into clusters that fit within the line budget.
fn group_items_into_clusters(
    items: &[TopLevelItem],
    max_lines: usize,
    file_path: &Path,
) -> Vec<ItemCluster> {
    if items.is_empty() {
        return Vec::new();
    }

    // Strategy: greedily pack items into clusters respecting max_lines.
    // Items of the same "kind" or sharing a common prefix are grouped together.
    let mut clusters: Vec<ItemCluster> = Vec::new();
    let mut current_items: Vec<TopLevelItem> = Vec::new();
    let mut current_lines: usize = 0;

    for item in items {
        let item_lines = item.end_line.saturating_sub(item.start_line) + 1;

        if current_lines + item_lines > max_lines && !current_items.is_empty() {
            // Flush current cluster
            let name = name_submodule(&current_items, file_path);
            clusters.push(ItemCluster {
                total_lines: current_lines,
                suggested_name: name,
                items: std::mem::take(&mut current_items),
            });
            current_lines = 0;
        }

        current_items.push(item.clone());
        current_lines += item_lines;
    }

    // Flush remaining
    if !current_items.is_empty() {
        let name = name_submodule(&current_items, file_path);
        clusters.push(ItemCluster {
            total_lines: current_lines,
            suggested_name: name,
            items: std::mem::take(&mut current_items),
        });
    }

    // Only return clusters if there's more than one (splitting into 1 chunk is pointless)
    if clusters.len() <= 1 {
        return Vec::new();
    }

    clusters
}

/// Generate a descriptive submodule name based on the items it contains.
fn name_submodule(items: &[TopLevelItem], file_path: &Path) -> String {
    let stem = file_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "module".to_string());

    if items.is_empty() {
        return format!("{}_part", stem);
    }

    // Check if all items are the same kind
    let kinds: Vec<&str> = items.iter().map(|i| i.kind.as_str()).collect();
    let all_same_kind = kinds.windows(2).all(|w| w[0] == w[1]);

    if all_same_kind {
        match kinds[0] {
            "struct" | "enum" | "type" | "union" => return format!("{}_types", stem),
            "trait" => return format!("{}_traits", stem),
            "impl" => {
                // Try to get the impl target name
                if items.len() == 1 {
                    return format!("{}_impl_{}", stem, items[0].name.to_lowercase());
                }
                return format!("{}_impls", stem);
            }
            "const" | "static" => return format!("{}_constants", stem),
            "fn" => {
                // Check for common prefixes
                if let Some(prefix) = find_common_prefix(items) {
                    return format!("{}_{}", stem, prefix);
                }
                return format!("{}_helpers", stem);
            }
            _ => {}
        }
    }

    // Mixed kinds — try common prefix
    if let Some(prefix) = find_common_prefix(items) {
        return format!("{}_{}", stem, prefix);
    }

    // Fallback: use ordinal suffix
    format!("{}_part", stem)
}

/// Find a common name prefix among items (returns None if no useful prefix).
fn find_common_prefix(items: &[TopLevelItem]) -> Option<String> {
    if items.len() < 2 {
        return None;
    }

    let names: Vec<&str> = items.iter().map(|i| i.name.as_str()).collect();
    let first = names[0];

    // Try progressively shorter prefixes
    for len in (3..=first.len()).rev() {
        let prefix = &first[..len];
        // Must end at an underscore boundary for readability
        if !prefix.ends_with('_') {
            continue;
        }
        if names.iter().all(|n| n.starts_with(prefix)) {
            let trimmed = prefix.trim_end_matches('_');
            if trimmed.len() >= 3 {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

// ── Plan display ─────────────────────────────────────────────────────────────

fn print_plan(plan: &SplitPlan, rel_path: &Path) {
    println!(
        "{} {}",
        c::label("Split Plan:"),
        c::path(&rel_path.display().to_string()),
    );
    println!(
        "  {}: {}    {}: {}",
        c::dim("Lines"),
        c::number(&plan.source_line_count.to_string()),
        c::dim("Clusters"),
        c::number(&plan.clusters.len().to_string()),
    );

    for (i, cluster) in plan.clusters.iter().enumerate() {
        println!(
            "  {} {} ({} lines, {} items)",
            c::dim(&format!("[{}]", i + 1)),
            c::path(&format!("{}.rs", cluster.suggested_name)),
            c::number(&cluster.total_lines.to_string()),
            c::number(&cluster.items.len().to_string()),
        );
        for item in &cluster.items {
            println!(
                "      {} {} (L{}-L{})",
                c::dim(&item.kind),
                c::label(&item.name),
                item.start_line,
                item.end_line,
            );
        }
    }

    if let Some((start, end)) = plan.test_block {
        println!(
            "  {} test block preserved (L{}-L{})",
            c::dim(""),
            start,
            end,
        );
    }

    println!();
}

// ── Plan execution ───────────────────────────────────────────────────────────

/// Build the actual file contents for each split target.
fn build_split_targets(plan: &SplitPlan, original_content: &str) -> Result<Vec<SplitTarget>> {
    debug_assert!(
        !original_content.is_empty(),
        "original_content must not be empty"
    );
    let lines: Vec<&str> = original_content.lines().collect();
    let mut targets = Vec::new();

    let parent = plan.source_file.parent().unwrap_or_else(|| Path::new("."));

    for cluster in &plan.clusters {
        let mut content = String::new();
        let has_impl = cluster.items.iter().any(|i| i.kind == "impl");

        // Add use block only for mod-style submodules (not include!() clusters).
        // include!() inherits the parent scope — adding `use` would be redundant
        // and can cause "unused import" warnings or shadow issues.
        if !has_impl {
            for use_line in &plan.use_block {
                content.push_str(use_line);
                content.push('\n');
            }
            if !plan.use_block.is_empty() {
                content.push('\n');
            }
        }

        // Add the items
        for item in &cluster.items {
            let start = item.start_line.saturating_sub(1); // to 0-based
            let end = item.end_line.min(lines.len()); // 1-based end, inclusive

            for line_idx in start..end {
                content.push_str(lines[line_idx]);
                content.push('\n');
            }
            content.push('\n');
        }

        let target_path = parent.join(format!("{}.rs", cluster.suggested_name));
        targets.push(SplitTarget {
            path: target_path,
            content,
        });
    }

    Ok(targets)
}

/// Execute the split plan: write submodule files and update the source.
fn execute_split_plan(plan: &SplitPlan, targets: &[SplitTarget], commit: bool) -> Result<()> {
    // Write each target file
    for target in targets {
        std::fs::write(&target.path, &target.content)
            .with_context(|| format!("Failed to write {}", target.path.display()))?;
    }

    // Update the source file: prepend mod declarations
    let original = std::fs::read_to_string(&plan.source_file)?;
    let mut new_content = String::new();

    // Keep module-level attributes
    for attr in &plan.attrs {
        new_content.push_str(attr);
        new_content.push('\n');
    }

    // Add mod or include!() declarations for each new submodule.
    // impl blocks MUST use include!() because they share the parent scope
    // (struct definitions, use imports). mod creates a separate scope which
    // breaks method resolution. Free functions use mod (proper isolation).
    let mut include_clusters = Vec::new();
    for cluster in &plan.clusters {
        let has_impl = cluster.items.iter().any(|i| i.kind == "impl");
        if has_impl {
            include_clusters.push(&cluster.suggested_name);
        } else {
            new_content.push_str(&format!("mod {};\n", cluster.suggested_name));
        }
    }
    new_content.push('\n');

    // Keep use statements
    for use_line in &plan.use_block {
        new_content.push_str(use_line);
        new_content.push('\n');
    }
    if !plan.use_block.is_empty() {
        new_content.push('\n');
    }

    // Keep items that weren't split out (remainder + test block)
    let extracted_ranges: Vec<(usize, usize)> = plan
        .clusters
        .iter()
        .flat_map(|c| c.items.iter().map(|i| (i.start_line, i.end_line)))
        .collect();

    let lines: Vec<&str> = original.lines().collect();
    let mut skip = false;
    let use_end = plan.use_block.len()
        + plan.attrs.len()
        + lines
            .iter()
            .take_while(|l| {
                let t = l.trim();
                t.is_empty()
                    || t.starts_with("use ")
                    || t.starts_with("pub use ")
                    || t.starts_with("pub(crate) use ")
                    || t.starts_with("#![")
                    || t.starts_with("//!")
                    || t.starts_with("//")
            })
            .count();

    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;

        // Skip the preamble (attrs + use) — we already wrote them
        if i < use_end {
            continue;
        }

        // Skip lines belonging to extracted items
        let in_extracted = extracted_ranges
            .iter()
            .any(|&(start, end)| line_num >= start && line_num <= end);

        if in_extracted {
            skip = true;
            continue;
        }

        if skip {
            // Skip blank lines between extracted items
            if line.trim().is_empty() {
                continue;
            }
            skip = false;
        }

        new_content.push_str(line);
        new_content.push('\n');
    }

    // Append include!() for clusters with impl blocks.
    // These must come after remaining items so they share the same scope.
    for name in &include_clusters {
        new_content.push_str(&format!("include!(\"{name}.rs\");\n"));
    }

    std::fs::write(&plan.source_file, &new_content)
        .with_context(|| format!("Failed to update {}", plan.source_file.display()))?;

    // Auto-commit if requested
    if commit {
        let mut files_to_add: Vec<String> = vec![plan.source_file.display().to_string()];
        for target in targets {
            files_to_add.push(target.path.display().to_string());
        }
        let stem = plan
            .source_file
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let add_args: Vec<&str> = files_to_add.iter().map(|s| s.as_str()).collect();
        let _ = std::process::Command::new("git")
            .arg("add")
            .args(&add_args)
            .status();

        let msg = format!(
            "refactor: split {} into {} submodules (pmat split --auto)",
            stem,
            targets.len()
        );
        let _ = std::process::Command::new("git")
            .args(["commit", "-m", &msg])
            .status();
    }

    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_top_level_item_fn() {
        assert_eq!(is_top_level_item("fn hello() {"), Some(("fn", "hello")));
        assert_eq!(is_top_level_item("pub fn world() {"), Some(("fn", "world")));
        assert_eq!(
            is_top_level_item("pub(crate) fn internal() {"),
            Some(("fn", "internal"))
        );
        assert_eq!(
            is_top_level_item("pub async fn do_thing() {"),
            Some(("fn", "do_thing"))
        );
    }

    #[test]
    fn test_is_top_level_item_types() {
        assert_eq!(is_top_level_item("struct Foo {"), Some(("struct", "Foo")));
        assert_eq!(is_top_level_item("pub enum Bar {"), Some(("enum", "Bar")));
        assert_eq!(is_top_level_item("trait Baz {"), Some(("trait", "Baz")));
        assert_eq!(is_top_level_item("impl Foo {"), Some(("impl", "Foo")));
        assert_eq!(is_top_level_item("impl<T> Foo<T> {"), Some(("impl", "T")));
        assert_eq!(
            is_top_level_item("type Alias = u32;"),
            Some(("type", "Alias"))
        );
    }

    #[test]
    fn test_is_top_level_item_indented_is_none() {
        assert_eq!(is_top_level_item("    fn nested() {"), None);
        assert_eq!(is_top_level_item("\tfn nested() {"), None);
        assert_eq!(is_top_level_item("  pub fn nested() {"), None);
    }

    #[test]
    fn test_is_top_level_item_non_items() {
        assert_eq!(is_top_level_item("let x = 1;"), None);
        assert_eq!(is_top_level_item("// fn comment() {"), None);
        assert_eq!(is_top_level_item(""), None);
    }

    #[test]
    fn test_find_split_points_basic() {
        let content = "\
fn alpha() {
    println!(\"a\");
}

fn beta() {
    println!(\"b\");
}

struct Gamma {
    x: i32,
}
";
        let items = find_split_points(content);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].name, "alpha");
        assert_eq!(items[0].kind, "fn");
        assert_eq!(items[1].name, "beta");
        assert_eq!(items[1].kind, "fn");
        assert_eq!(items[2].name, "Gamma");
        assert_eq!(items[2].kind, "struct");
    }

    #[test]
    fn test_group_items_into_clusters() {
        let items = vec![
            TopLevelItem {
                kind: "fn".to_string(),
                name: "a".to_string(),
                start_line: 1,
                end_line: 10,
            },
            TopLevelItem {
                kind: "fn".to_string(),
                name: "b".to_string(),
                start_line: 12,
                end_line: 20,
            },
            TopLevelItem {
                kind: "fn".to_string(),
                name: "c".to_string(),
                start_line: 22,
                end_line: 30,
            },
        ];
        let path = Path::new("example.rs");
        let clusters = group_items_into_clusters(&items, 15, path);
        // With max 15 lines, items a (10 lines) and b (9 lines) won't fit together,
        // so we get multiple clusters
        assert!(
            clusters.len() >= 2,
            "Expected >= 2 clusters, got {}",
            clusters.len()
        );
    }

    #[test]
    fn test_group_items_single_cluster_returns_empty() {
        // All items fit in one cluster -> no split needed -> empty result
        let items = vec![
            TopLevelItem {
                kind: "fn".to_string(),
                name: "a".to_string(),
                start_line: 1,
                end_line: 5,
            },
            TopLevelItem {
                kind: "fn".to_string(),
                name: "b".to_string(),
                start_line: 7,
                end_line: 10,
            },
        ];
        let path = Path::new("example.rs");
        let clusters = group_items_into_clusters(&items, 500, path);
        assert!(
            clusters.is_empty(),
            "Single cluster should return empty (no split needed)"
        );
    }

    #[test]
    fn test_name_submodule_types() {
        let items = vec![
            TopLevelItem {
                kind: "struct".to_string(),
                name: "Foo".to_string(),
                start_line: 1,
                end_line: 5,
            },
            TopLevelItem {
                kind: "struct".to_string(),
                name: "Bar".to_string(),
                start_line: 7,
                end_line: 10,
            },
        ];
        let path = Path::new("handler.rs");
        let name = name_submodule(&items, path);
        assert_eq!(name, "handler_types");
    }

    #[test]
    fn test_name_submodule_helpers() {
        let items = vec![
            TopLevelItem {
                kind: "fn".to_string(),
                name: "do_a".to_string(),
                start_line: 1,
                end_line: 5,
            },
            TopLevelItem {
                kind: "fn".to_string(),
                name: "do_b".to_string(),
                start_line: 7,
                end_line: 10,
            },
        ];
        let path = Path::new("utils.rs");
        let name = name_submodule(&items, path);
        // "do_" is a 3-char prefix after trimming underscore -> "do" is only 2 chars -> too short
        // Falls through to helpers
        assert_eq!(name, "utils_helpers");
    }

    #[test]
    fn test_name_submodule_common_prefix() {
        let items = vec![
            TopLevelItem {
                kind: "fn".to_string(),
                name: "handle_get".to_string(),
                start_line: 1,
                end_line: 5,
            },
            TopLevelItem {
                kind: "fn".to_string(),
                name: "handle_post".to_string(),
                start_line: 7,
                end_line: 10,
            },
        ];
        let path = Path::new("api.rs");
        let name = name_submodule(&items, path);
        assert_eq!(name, "api_handle");
    }

    #[test]
    fn test_skip_include_files() {
        let content_with_include = "use foo::bar;\n\ninclude!(\"helpers.rs\");\n\nfn main() {}\n";
        assert!(uses_include_pattern(content_with_include));

        let content_without = "use foo::bar;\n\nfn main() {}\n";
        assert!(!uses_include_pattern(content_without));
    }

    #[test]
    fn test_skip_test_files() {
        let test_path = Path::new("/project/tests/integration_test.rs");
        assert!(should_skip_file(test_path));

        let bench_path = Path::new("/project/benches/bench_test.rs");
        assert!(should_skip_file(bench_path));

        let mod_path = Path::new("/project/src/mod.rs");
        assert!(should_skip_file(mod_path));
    }

    #[test]
    fn test_skip_small_files() {
        // estimate_splits for a file under threshold should still return 1
        assert_eq!(estimate_splits(100, 500), 1);
        assert_eq!(estimate_splits(500, 500), 1);
        assert_eq!(estimate_splits(999, 500), 1);
        assert_eq!(estimate_splits(1000, 500), 1);
        assert_eq!(estimate_splits(1001, 500), 2);
        assert_eq!(estimate_splits(1500, 500), 2);
    }

    #[test]
    fn test_should_skip_dir() {
        use walkdir::WalkDir;
        // Test with actual walkdir entries by walking a temp dir
        let tmp = std::env::temp_dir().join("pmat_test_skip_dir");
        let _ = std::fs::create_dir_all(tmp.join("target/debug"));
        let _ = std::fs::create_dir_all(tmp.join("src"));

        for e in WalkDir::new(&tmp).max_depth(1).into_iter().flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name == "target" {
                assert!(should_skip_dir(&e));
            }
            if name == "src" {
                assert!(!should_skip_dir(&e));
            }
        }

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_strip_visibility() {
        assert_eq!(strip_visibility("pub fn foo()"), "fn foo()");
        assert_eq!(strip_visibility("pub(crate) fn bar()"), "fn bar()");
        assert_eq!(strip_visibility("pub(super) struct S"), "struct S");
        assert_eq!(strip_visibility("fn baz()"), "fn baz()");
    }

    #[test]
    fn test_extract_item_name() {
        assert_eq!(extract_item_name("foo() {"), "foo");
        assert_eq!(extract_item_name("Bar { x: i32 }"), "Bar");
        assert_eq!(extract_item_name("Baz<T> {"), "Baz");
        assert_eq!(extract_item_name(""), "unknown");
    }

    #[test]
    fn test_is_use_line() {
        assert!(is_use_line("use std::path::Path;"));
        assert!(is_use_line("pub use crate::foo;"));
        assert!(is_use_line("pub(crate) use crate::bar;"));
        assert!(!is_use_line("fn use_something() {"));
        assert!(!is_use_line("// use foo;"));
    }

    #[test]
    fn test_is_module_attr() {
        assert!(is_module_attr(&format!("#![allow({})]", "dead_code")));
        assert!(is_module_attr(
            "#![cfg_attr(coverage_nightly, coverage(off))]"
        ));
        assert!(!is_module_attr("#[test]"));
        assert!(!is_module_attr("fn foo() {}"));
    }

    #[test]
    fn test_find_test_block() {
        let content = "\
fn alpha() {}

#[cfg(test)]
mod tests {
    #[test]
    fn test_alpha() {}
}
";
        let lines: Vec<&str> = content.lines().collect();
        let items = find_split_points(content);
        let test_block = find_test_block(&items, &lines);
        assert!(test_block.is_some());
        let (start, end) = test_block.unwrap();
        assert!(start <= 3); // #[cfg(test)] is line 3
        assert!(end >= 6);
    }

    #[test]
    fn test_estimate_splits_edge_cases() {
        assert_eq!(estimate_splits(0, 500), 1);
        assert_eq!(estimate_splits(500, 0), 1);
        assert_eq!(estimate_splits(2000, 500), 3);
    }

    #[test]
    fn test_skip_generated_files() {
        let gen1 = Path::new("/project/src/schema.generated.rs");
        assert!(should_skip_file(gen1));

        let gen2 = Path::new("/project/src/generated_bindings.rs");
        assert!(should_skip_file(gen2));

        let normal = Path::new("/project/src/handler.rs");
        assert!(!should_skip_file(normal));
    }

    #[test]
    fn test_find_common_prefix() {
        let items = vec![
            TopLevelItem {
                kind: "fn".to_string(),
                name: "parse_json".to_string(),
                start_line: 1,
                end_line: 5,
            },
            TopLevelItem {
                kind: "fn".to_string(),
                name: "parse_xml".to_string(),
                start_line: 7,
                end_line: 10,
            },
        ];
        let prefix = find_common_prefix(&items);
        assert_eq!(prefix, Some("parse".to_string()));
    }

    #[test]
    fn test_find_common_prefix_no_match() {
        let items = vec![
            TopLevelItem {
                kind: "fn".to_string(),
                name: "alpha".to_string(),
                start_line: 1,
                end_line: 5,
            },
            TopLevelItem {
                kind: "fn".to_string(),
                name: "beta".to_string(),
                start_line: 7,
                end_line: 10,
            },
        ];
        let prefix = find_common_prefix(&items);
        assert_eq!(prefix, None);
    }
}
