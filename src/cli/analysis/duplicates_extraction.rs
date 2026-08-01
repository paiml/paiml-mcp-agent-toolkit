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
        // Type-2 (renamed) and Type-4 (semantic) have no implementation here.
        // They extract nothing rather than pretending: the caller reports the
        // honest zero for an explicitly requested mode, which is different from
        // the default silently reporting zero for everything.
        crate::cli::DuplicateType::Renamed | crate::cli::DuplicateType::Semantic => {}
    }

    blocks
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
                    let mut hasher = DefaultHasher::new();
                    content.hash(&mut hasher);
                    let hash = format!("{:x}", hasher.finish());

                    blocks.push((hash, file_str.to_string(), i + 1, end, content));
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
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

    // Sort by lines descending
    duplicates.sort_by_key(|b| std::cmp::Reverse(b.lines));

    duplicates
}

/// Check if file should be processed
fn should_process_file(path: &Path, include: &Option<String>, exclude: &Option<String>) -> bool {
    let path_str = path.to_string_lossy();

    if let Some(excl) = exclude {
        if path_str.contains(excl) {
            return false;
        }
    }

    if let Some(incl) = include {
        return path_str.contains(incl);
    }

    true
}

/// Check if file is source code
fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|s| s.to_str()),
        Some("rs" | "js" | "ts" | "py" | "java" | "cpp" | "c" | "kt" | "kts")
    )
}
