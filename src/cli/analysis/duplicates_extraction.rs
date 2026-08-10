/// Extract code blocks from lines
fn extract_blocks(
    lines: &[&str],
    path: &Path,
    min_lines: usize,
    max_tokens: usize,
    detection_type: crate::cli::DuplicateType,
) -> Vec<(String, String, usize, usize, String)> {
    let mut blocks = Vec::new();
    let file_str = path.to_string_lossy().to_string();

    // Exhaustive on purpose. This match used to end in `_ => {}`, which
    // silently swallowed `All` -- the DOCUMENTED DEFAULT. So the default
    // invocation extracted zero blocks and reported "total_duplicates: 0,
    // duplication_percentage: 0.0" for byte-identical files, while
    // `--detection-type exact` on the same input found 124. Reporting
    // duplicated code as clean is worse than reporting nothing at all.
    //
    // Keeping it exhaustive means a new DuplicateType variant is a compile
    // error here rather than another silent zero.
    match detection_type {
        crate::cli::DuplicateType::Exact => {
            extract_exact_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
        }
        crate::cli::DuplicateType::Fuzzy | crate::cli::DuplicateType::Gapped => {
            extract_fuzzy_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
        }
        // `All` must be a superset of every sub-mode, never a subset. Both
        // extractors run and their blocks are unioned; the hashes cannot
        // collide across modes because `extract_fuzzy_blocks` hashes a
        // structural signature while `extract_exact_blocks` hashes normalised
        // source, and identical content legitimately matching under both is a
        // genuine duplicate either way.
        crate::cli::DuplicateType::All => {
            extract_exact_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
            extract_fuzzy_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
        }
        // Type-2 (renamed). `extract_fuzzy_blocks` hashes an
        // identifier-normalised token stream, which is exactly what makes two
        // blocks that differ only in names hash alike, so renamed shares it.
        // Before, this arm extracted nothing and three byte-identical function
        // bodies with renamed identifiers were reported as "0 blocks / 0.0%
        // duplication" — a clean bill of health for a canonical Type-2 clone.
        crate::cli::DuplicateType::Renamed => {
            extract_fuzzy_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
        }
        // Type-4 (semantic) has no implementation here — detecting behavioural
        // equivalence needs analysis this extractor does not do. It must SAY so:
        // returning an empty block list made `--detection-type semantic` print
        // "0 duplicate blocks / 0.0% duplication" over provably duplicated code,
        // which reads as a measurement rather than a missing feature.
        crate::cli::DuplicateType::Semantic => {
            warn_semantic_unimplemented();
        }
    }

    blocks
}

/// Say once, on stderr, that `--detection-type semantic` measures nothing.
fn warn_semantic_unimplemented() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        eprintln!(
            "warning: --detection-type semantic (Type-4) is not implemented; \
no semantic clones are detected and the reported 0% is not a measurement. \
Use --detection-type all, exact, renamed, fuzzy or gapped."
        );
    });
}

/// Extract exact match blocks using sliding window
fn extract_exact_blocks(
    blocks: &mut Vec<(String, String, usize, usize, String)>,
    lines: &[&str],
    file_str: &str,
    min_lines: usize,
    max_tokens: usize,
) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Sliding window for exact matches
    for i in 0..lines.len().saturating_sub(min_lines) {
        let block_lines = &lines[i..i + min_lines];
        let content = normalize_block(block_lines);

        if count_tokens(&content) <= max_tokens {
            let mut hasher = DefaultHasher::new();
            content.hash(&mut hasher);
            let hash = format!("{:x}", hasher.finish());

            blocks.push((hash, file_str.to_string(), i + 1, i + min_lines, content));
        }
    }
}

/// Extract fuzzy match blocks based on code structure
fn extract_fuzzy_blocks(
    blocks: &mut Vec<(String, String, usize, usize, String)>,
    lines: &[&str],
    file_str: &str,
    min_lines: usize,
    max_tokens: usize,
) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut i = 0;
    while i < lines.len() {
        if is_block_start(lines[i]) {
            // Clamped: `find_block_end` returning None falls back to
            // `min_lines`, and `i + min_lines` can run past the end of the
            // file — `&lines[i..end]` then panics with "range end index 131 out
            // of range for slice of length 130".
            //
            // This was latent: `All` used to fall into a `_ => {}` arm so this
            // extractor was never reached on the DEFAULT detection type. Making
            // `All` a real superset turned a dormant panic into a crash on
            // `pmat analyze duplicates` over any ordinary source tree,
            // including pmat's own (SIGABRT, rc=134).
            let end = (find_block_end(&lines[i..]).unwrap_or(min_lines) + i).min(lines.len());
            if end - i >= min_lines {
                let block_lines = &lines[i..end];
                let content = normalize_block(block_lines);

                if count_tokens(&content) <= max_tokens {
                    // Hash the identifier-normalised token stream, not the
                    // source text. Hashing the text made this extractor a
                    // second, slower exact matcher: three structurally
                    // identical functions differ in their signature line, so
                    // they hashed differently and fuzzy/gapped/renamed reported
                    // 0 blocks over provably duplicated code.
                    let mut hasher = DefaultHasher::new();
                    normalize_identifiers(&content).hash(&mut hasher);
                    let hash = format!("f{:x}", hasher.finish());

                    blocks.push((hash, file_str.to_string(), i + 1, end, content));
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
}

/// Cross-language keywords that carry a block's structure. Everything else that
/// looks like a word is a name, and names are exactly what a Type-2 clone
/// changes.
const STRUCTURAL_KEYWORDS: &[&str] = &[
    "and", "as", "async", "await", "bool", "break", "case", "catch", "class", "const", "continue",
    "def", "delete", "do", "elif", "else", "enum", "export", "extends", "false", "final",
    "finally", "float", "fn", "for", "from", "function", "if", "impl", "import", "in", "int",
    "interface", "is", "let", "loop", "match", "mod", "move", "mut", "new", "nil", "none", "not",
    "null", "or", "pass", "private", "protected", "public", "pub", "raise", "ref", "return",
    "self", "static", "std", "str", "string", "struct", "super", "switch", "this", "throw",
    "trait", "true", "try", "type", "typeof", "use", "var", "void", "where", "while", "with",
    "yield",
];

/// Replace every non-keyword word with a single placeholder, keeping keywords,
/// punctuation and operators.
///
/// Two blocks that differ only in identifier and literal names normalise to the
/// same string, which is the definition of a Type-2 (renamed) clone.
fn normalize_identifiers(content: &str) -> String {
    fn flush(word: &mut String, out: &mut String) {
        if word.is_empty() {
            return;
        }
        if STRUCTURAL_KEYWORDS.contains(&word.as_str()) {
            out.push_str(word);
        } else {
            out.push('v');
        }
        out.push(' ');
        word.clear();
    }

    let mut out = String::with_capacity(content.len());
    let mut word = String::new();
    for ch in content.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            word.push(ch);
        } else {
            flush(&mut word, &mut out);
            if !ch.is_whitespace() {
                out.push(ch);
            }
        }
    }
    flush(&mut word, &mut out);
    out
}

/// Normalize code block (remove whitespace variations)
fn normalize_block(lines: &[&str]) -> String {
    lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with("//") && !line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Count tokens in content
fn count_tokens(content: &str) -> usize {
    content.split_whitespace().count()
}

/// Check if line starts a code block - refactored to reduce complexity
fn is_block_start(line: &str) -> bool {
    let trimmed = line.trim();

    // Check for function/method declarations
    if is_function_declaration(trimmed) {
        return true;
    }

    // Check for class/type declarations
    if is_type_declaration(trimmed) {
        return true;
    }

    // Check for block opening
    if is_block_opening(trimmed) {
        return true;
    }

    false
}

/// Check if line is a function declaration
fn is_function_declaration(line: &str) -> bool {
    line.contains("fn ") || line.contains("function") || line.contains("def ")
}

/// Check if line is a type declaration
fn is_type_declaration(line: &str) -> bool {
    line.contains("class ") || line.contains("struct ") || line.contains("impl ")
}

/// Check if line is a block opening
fn is_block_opening(line: &str) -> bool {
    line.ends_with('{') && !line.starts_with('{')
}

/// Find end of code block
fn find_block_end(lines: &[&str]) -> Option<usize> {
    let mut brace_count = 0;
    let mut in_block = false;

    for (i, line) in lines.iter().enumerate() {
        for ch in line.chars() {
            match ch {
                '{' => {
                    brace_count += 1;
                    in_block = true;
                }
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 && in_block {
                        return Some(i + 1);
                    }
                }
                _ => {}
            }
        }
    }

    None
}

/// Find duplicate blocks from all blocks
fn find_duplicate_blocks(
    all_blocks: Vec<(String, String, usize, usize, String)>,
    _threshold: f32,
) -> Vec<DuplicateBlock> {
    let mut hash_groups: HashMap<String, Vec<(String, usize, usize, String)>> = HashMap::new();

    // Group by hash
    for (hash, file, start, end, content) in all_blocks {
        hash_groups
            .entry(hash)
            .or_default()
            .push((file, start, end, content));
    }

    // Find duplicates
    let mut duplicates = Vec::new();
    for (hash, mut locations) in hash_groups {
        // Collapse OVERLAPPING windows within a file before deciding anything is
        // duplicated. Detection slides a window one line at a time, so a single
        // 5-line function yields windows at 2-6, 3-7 and 4-8 whose normalised
        // text can hash identically. Counting those as three "locations" made a
        // file of four entirely distinct functions report four duplicates at
        // 211.5% duplication. Overlapping windows are the same code, not copies
        // of it.
        //
        // Greedy sweep per file: keep a window, skip every later one that starts
        // before the kept one ends.
        locations.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        let mut kept: Vec<(String, usize, usize, String)> = Vec::new();
        for loc in locations {
            let overlaps_kept = kept
                .last()
                .is_some_and(|last| last.0 == loc.0 && loc.1 <= last.2);
            if !overlaps_kept {
                kept.push(loc);
            }
        }
        let locations = kept;

        // Two or more NON-OVERLAPPING sites is what makes it a duplicate.
        if locations.len() > 1 {
            let lines = locations[0].2 - locations[0].1 + 1;
            let tokens = count_tokens(&locations[0].3);

            let duplicate_locations: Vec<DuplicateLocation> = locations
                .into_iter()
                .map(|(file, start, end, content)| {
                    let preview = content.lines().take(3).collect::<Vec<_>>().join("\n");
                    DuplicateLocation {
                        file,
                        start_line: start,
                        end_line: end,
                        content_preview: if content.lines().count() > 3 {
                            format!("{preview}...")
                        } else {
                            preview
                        },
                    }
                })
                .collect();

            duplicates.push(DuplicateBlock {
                hash,
                locations: duplicate_locations,
                lines,
                tokens,
                similarity: 1.0, // Exact match for now
            });
        }
    }

    // Sort by lines descending.
    //
    // DETERMINISM (round-3 sweep): `hash_groups` is a `HashMap`, so the vector
    // above was built in a per-process random order, and `sort_by_key` is
    // stable — every block of the same `lines` therefore kept that random
    // order. `analyze duplicates --format json` on a fixed two-file fixture
    // produced 5 DIFFERENT md5 sums over 5 runs, with the same 14 block hashes
    // merely reordered. The (file, start_line, hash) suffix is a total order
    // over blocks: `locations` is already sorted by (file, start) above, and no
    // two surviving blocks share a hash.
    duplicates.sort_by(|a, b| {
        b.lines
            .cmp(&a.lines)
            .then_with(|| {
                let a_first = a.locations.first();
                let b_first = b.locations.first();
                match (a_first, b_first) {
                    (Some(x), Some(y)) => (&x.file, x.start_line).cmp(&(&y.file, y.start_line)),
                    (None, Some(_)) => std::cmp::Ordering::Less,
                    (Some(_), None) => std::cmp::Ordering::Greater,
                    (None, None) => std::cmp::Ordering::Equal,
                }
            })
            .then_with(|| a.hash.cmp(&b.hash))
    });

    duplicates
}

/// Check if file should be processed.
///
/// `--help` advertises these as globs ("Include file patterns (e.g.,
/// \"**/*.rs\")"), but both were plain `path_str.contains` substring tests, so
/// the example printed in the help text selected NOTHING: over a directory of
/// .rs files `--include '**/*.rs'` analysed zero files and reported
/// "Duplication percentage: 0.0%" with exit 0, while `--exclude '**/*.rs'`
/// excluded nothing at all. A filter that silently matches no file is reported
/// as a clean project.
///
/// A pattern that carries glob metacharacters is now matched as a glob;
/// anything else keeps the substring behaviour, which is the only form that
/// ever worked (`--include path_validator`) and which callers rely on.
fn should_process_file(path: &Path, include: &Option<String>, exclude: &Option<String>) -> bool {
    let path_str = path.to_string_lossy();

    if let Some(excl) = exclude {
        if matches_file_pattern(&path_str, excl) {
            return false;
        }
    }

    if let Some(incl) = include {
        return matches_file_pattern(&path_str, incl);
    }

    true
}

/// Match one `--include`/`--exclude` pattern against a path.
fn matches_file_pattern(path_str: &str, pattern: &str) -> bool {
    if !pattern.contains(['*', '?', '[', '{']) {
        return path_str.contains(pattern);
    }

    // Matched against the whole path, which is usually absolute, so `*` has to
    // be able to cross a separator — globset's default (`literal_separator`
    // off) is what makes both `*.rs` and the documented `**/*.rs` match
    // `/home/u/proj/src/utils/path_validator.rs`.
    match globset::Glob::new(pattern) {
        Ok(glob) => glob.compile_matcher().is_match(path_str),
        // An unparseable pattern falls back to the historical substring test
        // rather than quietly selecting nothing.
        Err(_) => path_str.contains(pattern),
    }
}

/// Check if file is source code
fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" | "kt" | "kts")
    )
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod include_exclude_pattern_tests {
    use super::*;
    use std::path::Path;

    const SOURCE: &str = "/home/u/proj/src/utils/path_validator.rs";

    /// The pattern printed in `--help` selected zero files: `analyze duplicates
    /// --include '**/*.rs'` over 2902 lines of Rust reported 0 duplicates and
    /// 0.0% duplication with exit 0, and `--exclude '**/*.rs'` dropped nothing.
    #[test]
    fn documented_glob_patterns_match_source_files() {
        for pattern in ["**/*.rs", "*.rs", "**", "*", "**/utils/*.rs"] {
            assert!(
                should_process_file(Path::new(SOURCE), &Some(pattern.to_string()), &None),
                "--include '{pattern}' must select a .rs file"
            );
            assert!(
                !should_process_file(Path::new(SOURCE), &None, &Some(pattern.to_string())),
                "--exclude '{pattern}' must drop a .rs file"
            );
        }
    }

    #[test]
    fn globs_that_do_not_match_still_filter() {
        assert!(!should_process_file(
            Path::new(SOURCE),
            &Some("**/*.py".to_string()),
            &None
        ));
        assert!(should_process_file(
            Path::new(SOURCE),
            &None,
            &Some("**/*.py".to_string())
        ));
    }

    /// Glob support must not remove the only form that ever worked: a bare
    /// substring (`--include path_validator` selected 10 duplicate blocks).
    #[test]
    fn substring_patterns_are_still_honoured() {
        let path = Path::new("src/utils/path_validator.rs");
        assert!(should_process_file(
            path,
            &Some("path_validator".to_string()),
            &None
        ));
        assert!(!should_process_file(path, &Some("tests".to_string()), &None));
        assert!(!should_process_file(path, &None, &Some(".rs".to_string())));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod renamed_clone_tests {
    use super::*;
    use crate::cli::DuplicateType;
    use std::path::Path;

    /// Three copies of one body with every identifier renamed — the canonical
    /// Type-2 clone.
    const RENAMED_SOURCE: &str = "\
fn alpha(input: usize) -> usize {
    let total = input + 1;
    let doubled = total * 2;
    doubled
}
fn beta(value: usize) -> usize {
    let sum = value + 1;
    let twice = sum * 2;
    twice
}
fn gamma(arg: usize) -> usize {
    let acc = arg + 1;
    let scaled = acc * 2;
    scaled
}
";

    fn blocks_for(kind: DuplicateType) -> Vec<(String, String, usize, usize, String)> {
        let lines: Vec<&str> = RENAMED_SOURCE.lines().collect();
        extract_blocks(&lines, Path::new("dup.rs"), 4, 1000, kind)
    }

    /// `--detection-type renamed` extracted nothing at all, so a file of three
    /// renamed copies of one function was reported as 0 blocks / 0.0%
    /// duplication.
    #[test]
    fn renamed_mode_finds_the_renamed_copies() {
        let blocks = blocks_for(DuplicateType::Renamed);
        assert!(!blocks.is_empty(), "renamed mode must extract blocks");

        let duplicates = find_duplicate_blocks(blocks, 0.8);
        assert!(
            duplicates.iter().any(|d| d.locations.len() >= 3),
            "three renamed copies of one body are one duplicate group, got {duplicates:?}"
        );
    }

    /// Gapped and fuzzy are Type-3/Type-2 tolerant by definition; they used to
    /// hash raw text, which made them a slower exact matcher.
    #[test]
    fn gapped_and_fuzzy_find_the_renamed_copies_too() {
        for kind in [DuplicateType::Gapped, DuplicateType::Fuzzy] {
            let duplicates = find_duplicate_blocks(blocks_for(kind.clone()), 0.8);
            assert!(
                duplicates.iter().any(|d| d.locations.len() >= 3),
                "{kind:?} must report the renamed clones"
            );
        }
    }

    /// Exact match is Type-1 only: renamed copies are legitimately not exact
    /// duplicates, and that zero IS a measurement.
    #[test]
    fn exact_mode_does_not_claim_renamed_copies() {
        let duplicates = find_duplicate_blocks(blocks_for(DuplicateType::Exact), 0.8);
        assert!(duplicates.iter().all(|d| d.locations.len() < 3));
    }

    #[test]
    fn identifier_normalisation_erases_names_but_keeps_structure() {
        let a = normalize_identifiers("let total = input + 1;");
        let b = normalize_identifiers("let sum = value + 1;");
        assert_eq!(a, b, "only the names differ");

        let c = normalize_identifiers("let total = input - 1;");
        assert_ne!(a, c, "the operator is structure, not a name");
    }
}
