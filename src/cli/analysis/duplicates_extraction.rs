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
        crate::cli::DuplicateType::Fuzzy => {
            extract_fuzzy_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
        }
        // Type-3 (gapped) subsumes Type-2, so it runs the renamed extractor AND
        // the gap-tolerant one. #935: it used to run ONLY the renamed extractor,
        // so `--detection-type gapped` and `--detection-type renamed` executed
        // identical code and no amount of "gapped" in the help text made the
        // hash tolerant of an inserted statement.
        crate::cli::DuplicateType::Gapped => {
            extract_fuzzy_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
            extract_gapped_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
        }
        // `All` must be a superset of every sub-mode, never a subset. Every
        // extractor runs and their blocks are unioned; the hashes cannot
        // collide across modes because each pass prefixes its own digest
        // (`f` fuzzy, `g` gapped, bare hex exact), and identical content
        // legitimately matching under more than one is a genuine duplicate
        // either way.
        crate::cli::DuplicateType::All => {
            extract_exact_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
            extract_fuzzy_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
            extract_gapped_blocks(&mut blocks, lines, &file_str, min_lines, max_tokens);
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

/// Extract exact match blocks using a window over SUBSTANTIVE lines.
///
/// The window slides over code, not over the file. It used to slide over raw
/// lines and hash whatever `normalize_block` left behind — and
/// `normalize_block` DELETES comments and blank lines, so a window covering
/// nothing but comments normalised to the empty string. Every such window in
/// every file of the project hashed to `30406ea523c53def`, the hash of `""`,
/// and landed in one bucket: two files whose sorted lines shared not a single
/// line (`comm -12` empty) were reported as `exact_duplicates: 1`, `similarity:
/// 1.0`, `tokens: 0`, "Duplication percentage: 62.5%". A detector that calls
/// two unrelated files two-thirds duplicated points the wrong way, and
/// `--duplicates` is what this project uses to hunt its own copy-paste.
///
/// The floor is not new policy: the MinHash engine
/// (`services::duplicate_detector`) has always required
/// `tokens.len() >= config.min_tokens` before it will build a fragment. This
/// pass was the one clone finder with an upper bound on tokens (`--max-tokens`)
/// and no lower bound at all. Windowing over substantive lines gives it one and
/// makes `--min-lines` mean what its help text says: every block carries
/// `min_lines` lines of actual code as evidence.
fn extract_exact_blocks(
    blocks: &mut Vec<(String, String, usize, usize, String)>,
    lines: &[&str],
    file_str: &str,
    min_lines: usize,
    max_tokens: usize,
) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // A block of zero lines is not a block. `windows(0)` panics, and the old
    // code answered `--min-lines 0` with a stream of empty-content blocks.
    let min_lines = min_lines.max(1);

    let substantive = substantive_lines(lines);
    for window in substantive.windows(min_lines) {
        let content = window
            .iter()
            .map(|(_, line)| *line)
            .collect::<Vec<_>>()
            .join("\n");

        if count_tokens(&content) <= max_tokens {
            let mut hasher = DefaultHasher::new();
            content.hash(&mut hasher);
            let hash = format!("{:x}", hasher.finish());

            let start = window[0].0;
            let end = window[window.len() - 1].0;
            blocks.push((hash, file_str.to_string(), start, end, content));
        }
    }
}

/// Walk the structural blocks of a file once and hand each qualifying one to
/// `emit` as `(normalised content, start line, end line)`.
///
/// ONE walk, shared by the Type-2 (`extract_fuzzy_blocks`) and Type-3
/// (`extract_gapped_blocks`) passes. Both used to need the identical clamping,
/// floor and span arithmetic below; a second copy of that arithmetic is a second
/// place for the "range end index out of range" panic and the empty-block hole
/// to come back.
fn for_each_structural_block(
    lines: &[&str],
    min_lines: usize,
    max_tokens: usize,
    mut emit: impl FnMut(&str, usize, usize),
) {
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

                // Same floor as the exact pass: the block must carry
                // `min_lines` lines of real code. `is_block_start` is textual
                // (`contains("fn ")`, "ends with `{`"), so a run of comments
                // such as `// build the fn {` opens a "block" whose normalised
                // content is empty — and every empty block in the project
                // hashes alike. Reporting the block's SUBSTANTIVE span rather
                // than its raw span also stops leading and trailing comment
                // lines from being counted as duplicated code.
                let substantive = substantive_lines(block_lines);
                if substantive.len() >= min_lines && count_tokens(&content) <= max_tokens {
                    let start_line = substantive[0].0 + i;
                    let end_line = substantive[substantive.len() - 1].0 + i;
                    emit(&content, start_line, end_line);
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
}

/// Extract fuzzy match blocks based on code structure (Type-2: renamed).
fn extract_fuzzy_blocks(
    blocks: &mut Vec<(String, String, usize, usize, String)>,
    lines: &[&str],
    file_str: &str,
    min_lines: usize,
    max_tokens: usize,
) {
    for_each_structural_block(lines, min_lines, max_tokens, |content, start, end| {
        // Hash the identifier-normalised token stream, not the source text.
        // Hashing the text made this extractor a second, slower exact matcher:
        // three structurally identical functions differ in their signature line,
        // so they hashed differently and fuzzy/gapped/renamed reported 0 blocks
        // over provably duplicated code.
        let hash = format!("f{}", hash_of(&normalize_identifiers(content)));
        blocks.push((hash, file_str.to_string(), start, end, content.to_string()));
    });
}

/// Extract Type-3 (gapped) blocks: the same body with straight-line statements
/// ADDED OR REMOVED.
///
/// #935: `--detection-type gapped` and `--detection-type renamed` ran the same
/// extractor. Two declared modes executing identical code is a flag that does
/// not do what it says, and `gapped` in particular claimed the one tolerance
/// nothing in the hash-bucketing pass provided: hashing an
/// identifier-normalised token stream (the Type-2 rule) is exact matching — one
/// extra `let` line changes the hash and the copy is not found.
///
/// The gapped bucket hashes the block's SKELETON: its identifier-normalised
/// lines with straight-line statements elided (see [`gapped_skeleton`]). Two
/// copies of one body that differ by inserted or deleted simple statements
/// produce the same skeleton and bucket together, which is exactly the Type-3
/// definition. Type-3 subsumes Type-2, so `gapped` runs this pass IN ADDITION TO
/// the Type-2 one and can never report less than `renamed` does.
///
/// Over-grouping is what makes a loose skeleton worthless, so a block only gets
/// a gapped hash when all three hold:
/// 1. eliding actually removed a line — otherwise the skeleton is just the
///    Type-2 hash under another name and the group would be double counted;
/// 2. the skeleton carries at least two CONTROL-FLOW lines, so the near-universal
///    `fn v ( v ) { }` shape cannot bucket every one-statement function together;
/// 3. the skeleton is at least `--min-lines` lines long — the same floor the
///    user already set for what counts as a block.
fn extract_gapped_blocks(
    blocks: &mut Vec<(String, String, usize, usize, String)>,
    lines: &[&str],
    file_str: &str,
    min_lines: usize,
    max_tokens: usize,
) {
    for_each_structural_block(lines, min_lines, max_tokens, |content, start, end| {
        if let Some(skeleton) = gapped_skeleton(content, min_lines) {
            let hash = format!("g{}", hash_of(&skeleton));
            blocks.push((hash, file_str.to_string(), start, end, content.to_string()));
        }
    });
}

/// The stable 64-bit digest every extractor buckets on.
fn hash_of(text: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Words that carry CONTROL FLOW, as opposed to structure in general.
///
/// A skeleton made only of braces and a signature describes almost every
/// function ever written; one made of branches and loops describes a shape.
const CONTROL_FLOW_KEYWORDS: &[&str] = &[
    "case", "catch", "do", "elif", "else", "finally", "for", "if", "loop", "match", "switch",
    "try", "when", "while",
];

/// The gap-tolerant signature of a block, or `None` when the block has no
/// distinctive skeleton (see [`extract_gapped_blocks`] for the three guards).
fn gapped_skeleton(content: &str, min_lines: usize) -> Option<String> {
    let normalized: Vec<String> = content.lines().map(normalize_identifiers).collect();
    let kept: Vec<&str> = normalized
        .iter()
        .map(String::as_str)
        .filter(|line| carries_structure(line))
        .collect();

    // (1) Nothing was elided: this block has no straight-line statements to be
    // tolerant ABOUT, so its skeleton is its Type-2 signature and the fuzzy pass
    // has already bucketed it.
    if kept.len() == normalized.len() {
        return None;
    }
    // (3) and (2): long enough to be a shape, and branching enough to be a
    // distinctive one.
    if kept.len() < min_lines.max(1) {
        return None;
    }
    if kept
        .iter()
        .filter(|line| carries_control_flow(line))
        .count()
        < 2
    {
        return None;
    }

    Some(kept.join("\n"))
}

/// Does this identifier-normalised line carry block structure — a brace or a
/// control-flow word — rather than being a straight-line statement?
fn carries_structure(normalized_line: &str) -> bool {
    normalized_line.contains('{')
        || normalized_line.contains('}')
        || carries_control_flow(normalized_line)
}

/// Does this identifier-normalised line branch or loop?
///
/// `normalize_identifiers` emits whitespace-separated words and keeps the
/// structural keywords verbatim, so whole-word matching is exact here — no
/// substring test that would find `if` inside `notify`.
fn carries_control_flow(normalized_line: &str) -> bool {
    normalized_line
        .split_whitespace()
        .any(|word| CONTROL_FLOW_KEYWORDS.contains(&word))
}

/// Cross-language keywords that carry a block's structure. Everything else that
/// looks like a word is a name, and names are exactly what a Type-2 clone
/// changes.
const STRUCTURAL_KEYWORDS: &[&str] = &[
    "and",
    "as",
    "async",
    "await",
    "bool",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "def",
    "delete",
    "do",
    "elif",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "final",
    "finally",
    "float",
    "fn",
    "for",
    "from",
    "function",
    "if",
    "impl",
    "import",
    "in",
    "int",
    "interface",
    "is",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "new",
    "nil",
    "none",
    "not",
    "null",
    "or",
    "pass",
    "private",
    "protected",
    "public",
    "pub",
    "raise",
    "ref",
    "return",
    "self",
    "static",
    "std",
    "str",
    "string",
    "struct",
    "super",
    "switch",
    "this",
    "throw",
    "trait",
    "true",
    "try",
    "type",
    "typeof",
    "use",
    "var",
    "void",
    "where",
    "while",
    "with",
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

/// Does this (already trimmed) line carry code?
///
/// THE rule for what this analysis treats as content. `normalize_block` deletes
/// everything else, so anything built out of non-substantive lines alone is
/// built out of nothing — which is how comment-only windows all came to hash to
/// the empty string and cluster into a single bogus "exact duplicate".
/// Every caller that decides whether something is a block asks this one
/// function; a second copy of the predicate is a second chance to disagree.
fn is_substantive_line(trimmed: &str) -> bool {
    !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with('#')
}

/// The substantive lines of `lines`, each paired with its 1-based index WITHIN
/// `lines`, so a block can report the span of the code it actually contains.
fn substantive_lines<'a>(lines: &[&'a str]) -> Vec<(usize, &'a str)> {
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| (i + 1, line.trim()))
        .filter(|(_, line)| is_substantive_line(line))
        .collect()
}

/// Normalize code block (remove whitespace variations)
fn normalize_block(lines: &[&str]) -> String {
    substantive_lines(lines)
        .into_iter()
        .map(|(_, line)| line)
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

/// The `--threshold` default. A value that differs from it was typed by the
/// user, and the user is owed an answer about what it did.
const DOCUMENTED_THRESHOLD_DEFAULT: f32 = 0.85;

/// Say once, on stderr, that `--threshold` cannot move any block in or out
/// UNDER THIS DETECTION TYPE.
///
/// Hash bucketing (this file) is all-or-nothing: two blocks are duplicates iff
/// they hash identically under the detection type's normalisation, so no cut-off
/// in 0.0..=1.0 can move a block into or out of a group — the similarity each
/// group reports is measured AFTER the fact, never used to select it.
/// `--threshold` was therefore bound as `_threshold`
/// and dropped: 0.01, 0.5 and 0.99 printed the same "Duplication percentage"
/// over the same tree with no indication that the number had been ignored.
///
/// `gapped`, `fuzzy` and `all` now also run the near-miss (Type-3) pass in
/// `find_structural_similarities`, where the threshold IS the similarity
/// cut-off, so under those types the value acts and nothing is warned about.
/// Under `exact` (Type-1), `renamed` (Type-2) and `semantic` (unimplemented)
/// there is still no similarity comparison for a cut-off to control.
///
/// Returns whether the value is inert, so the behaviour is testable.
fn warn_threshold_has_no_effect(
    threshold: f32,
    detection_type: &crate::cli::DuplicateType,
) -> bool {
    if near_miss_enabled(detection_type) {
        return false;
    }
    if (threshold - DOCUMENTED_THRESHOLD_DEFAULT).abs() < 1e-6 {
        return false;
    }

    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        eprintln!(
            "warning: --threshold {threshold} was ignored: --detection-type {detection_type} \
matches blocks by hash — a block is in a group or it is not — so no similarity cut-off changes \
the result. Use --detection-type gapped, fuzzy or all, where the threshold is the near-miss \
similarity cut-off."
        );
    });
    true
}

/// Classify one hash-bucketed group and measure its similarity.
///
/// Hash bucketing proves the members agree under the detection type's
/// normalisation. That is Type-1 when the normalisation was "drop comments and
/// blank lines" and only Type-2 when it was "drop identifier names" — and the
/// extractor that produced the group is NOT the answer, because a fuzzy-hashed
/// group of byte-identical bodies is a genuine exact clone. Comparing the
/// group's text settles it.
///
/// The returned similarity is measured the same way: exactly 1.0 when every
/// member is identical, otherwise the mean line-level Jaccard similarity of each
/// member against the first site. `similarity: 1.0` used to be written for every
/// group, so a renamed clone differing on 8 of 39 lines reported perfect
/// identity.
fn classify_hash_group(texts: &[&str]) -> (CloneType, f32) {
    let Some((first, rest)) = texts.split_first() else {
        return (CloneType::Exact, 1.0);
    };

    if rest.iter().all(|t| t == first) {
        return (CloneType::Exact, 1.0);
    }

    let total: f32 = rest.iter().map(|t| line_jaccard(first, t)).sum();
    #[allow(clippy::cast_precision_loss)]
    let mean = total / rest.len() as f32;

    // Type-2 vs Type-3, decided by the CONTENT and not by which extractor
    // produced the bucket: members that agree once identifiers are normalised
    // away differ only in names (Type-2, renamed); members that still differ
    // afterwards differ in their STATEMENTS (Type-3, near-miss). Before the
    // gapped pass existed no bucket could reach the second case, so every
    // non-identical group was labelled "renamed" — which would have mislabelled
    // every gapped group the moment one appeared.
    let first_shape = normalize_identifiers(first);
    if !rest.iter().all(|t| normalize_identifiers(t) == first_shape) {
        return (CloneType::NearMiss, mean.min(1.0 - f32::EPSILON));
    }

    // A measured mean can round to 1.0 for texts that are not identical; the
    // identity test above is the ONLY thing allowed to report 1.0, so that
    // `exact_duplicates` and `similarity >= 1.0` can never disagree.
    (CloneType::Renamed, mean.min(1.0 - f32::EPSILON))
}

/// Jaccard similarity of two blocks over their distinct lines.
fn line_jaccard(a: &str, b: &str) -> f32 {
    use std::collections::HashSet;
    let left: HashSet<&str> = a.lines().collect();
    let right: HashSet<&str> = b.lines().collect();
    let union = left.union(&right).count();
    if union == 0 {
        return 0.0;
    }
    let intersection = left.intersection(&right).count();
    #[allow(clippy::cast_precision_loss)]
    let sim = intersection as f32 / union as f32;
    sim
}

/// Find duplicate blocks from all blocks.
///
/// Takes no threshold: hash bucketing has no similarity cut-off to apply. The
/// near-miss pass (`find_structural_similarities`) is where `--threshold` acts.
fn find_duplicate_blocks(
    all_blocks: Vec<(String, String, usize, usize, String)>,
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

            // MEASURED, not assumed: a group survives hash bucketing because its
            // members agree under the detection type's normalisation, and the
            // fuzzy hash normalises identifiers away — so a group can be either
            // Type-1 or Type-2 and only its text can say which.
            let texts: Vec<&str> = locations.iter().map(|(_, _, _, c)| c.as_str()).collect();
            let (clone_type, similarity) = classify_hash_group(&texts);

            // A gapped bucket (`g` prefix) only earns its place when the members
            // really do differ in their STATEMENTS. If they agree once
            // identifiers are normalised away they are a Type-2 group, and the
            // Type-2 pass — which `gapped` and `all` both run — has already
            // reported them: keeping this copy listed the same pair of sites
            // twice (measured on `src/services/complexity`: 10 blocks under
            // `gapped` against 9 under `fuzzy`, over the identical 278
            // duplicate lines).
            if hash.starts_with('g') && clone_type != CloneType::NearMiss {
                continue;
            }

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
                similarity,
                clone_type,
            });
        }
    }

    sort_duplicate_blocks(&mut duplicates);

    duplicates
}

/// Put duplicate blocks in a total order.
///
/// DETERMINISM (round-3 sweep): `find_duplicate_blocks` groups in a `HashMap`,
/// so the vector is built in a per-process random order, and `sort_by_key` is
/// stable — every block of the same `lines` therefore kept that random order.
/// `analyze duplicates --format json` on a fixed two-file fixture produced 5
/// DIFFERENT md5 sums over 5 runs, with the same 14 block hashes merely
/// reordered. The (file, start_line, hash) suffix is a total order over blocks:
/// `locations` is sorted by (file, start) before this runs, and no two blocks
/// share a hash.
///
/// Shared with the near-miss pass, whose blocks are appended to the same vector
/// and must be interleaved in the same fixed order.
fn sort_duplicate_blocks(duplicates: &mut [DuplicateBlock]) {
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
        assert!(!should_process_file(
            path,
            &Some("tests".to_string()),
            &None
        ));
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

        let duplicates = find_duplicate_blocks(blocks);
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
            let duplicates = find_duplicate_blocks(blocks_for(kind.clone()));
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
        let duplicates = find_duplicate_blocks(blocks_for(DuplicateType::Exact));
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

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod threshold_tests {
    use super::*;

    use crate::cli::DuplicateType;

    /// `--threshold` was bound as `_threshold` and dropped: 0.01, 0.5 and 0.99
    /// printed the same duplication percentage with nothing said about the
    /// number being ignored. Hash bucketing cannot honour a similarity cut-off,
    /// so under the detection types that only bucket, a value that cannot act
    /// must still be reported as inert.
    #[test]
    fn a_threshold_that_cannot_act_is_disclosed() {
        for kind in [
            DuplicateType::Exact,
            DuplicateType::Renamed,
            DuplicateType::Semantic,
        ] {
            for typed in [0.0_f32, 0.01, 0.5, 0.99, 1.0] {
                assert!(
                    warn_threshold_has_no_effect(typed, &kind),
                    "--threshold {typed} changes nothing under {kind:?} and must say so"
                );
            }
        }
    }

    /// Under the near-miss types the threshold IS the similarity cut-off the
    /// Type-3 search uses, so warning that it was ignored would be false.
    #[test]
    fn a_threshold_that_acts_is_not_disclaimed() {
        for kind in [
            DuplicateType::Gapped,
            DuplicateType::Fuzzy,
            DuplicateType::All,
        ] {
            for typed in [0.0_f32, 0.5, 0.99] {
                assert!(
                    !warn_threshold_has_no_effect(typed, &kind),
                    "--threshold {typed} controls the near-miss cut-off under {kind:?}"
                );
            }
        }
    }

    /// The default was not typed by anyone; it must not print a warning on
    /// every ordinary run.
    #[test]
    fn the_default_threshold_is_quiet() {
        assert!(!warn_threshold_has_no_effect(
            DOCUMENTED_THRESHOLD_DEFAULT,
            &DuplicateType::Exact
        ));
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod gapped_clone_tests {
    //! #935: `--detection-type gapped` and `--detection-type renamed` ran the
    //! SAME extractor, so the mode that promises tolerance for added or removed
    //! statements matched nothing an exact identifier-normalised hash would not
    //! already match. Two declared modes executing identical code is a flag that
    //! does not do what its help text says.

    use super::*;
    use crate::cli::DuplicateType;
    use std::path::Path;

    /// Two copies of one body. `beta` renames every identifier AND inserts a
    /// straight-line statement (`let scale = 2;`), which is the textbook Type-3
    /// (gapped) clone: same shape, different statements.
    const GAPPED_SOURCE: &str = "\
fn alpha(items: &[u32]) -> u32 {
    let mut total = 0;
    for item in items {
        if *item > 10 {
            total += item;
        } else {
            total -= item;
        }
    }
    total
}
fn beta(values: &[u32]) -> u32 {
    let mut acc = 0;
    let scale = 2;
    for value in values {
        if *value > 10 {
            acc += value;
        } else {
            acc -= value;
        }
    }
    acc
}
";

    fn blocks_for(kind: DuplicateType) -> Vec<(String, String, usize, usize, String)> {
        let lines: Vec<&str> = GAPPED_SOURCE.lines().collect();
        extract_blocks(&lines, Path::new("dup.rs"), 5, 1000, kind)
    }

    /// THE #935 REGRESSION. On the old code `gapped` and `renamed` produced
    /// byte-identical block lists, so this assertion could not hold for any
    /// input whatsoever.
    #[test]
    fn gapped_extracts_something_renamed_does_not() {
        let renamed: Vec<String> = blocks_for(DuplicateType::Renamed)
            .into_iter()
            .map(|(hash, ..)| hash)
            .collect();
        let gapped: Vec<String> = blocks_for(DuplicateType::Gapped)
            .into_iter()
            .map(|(hash, ..)| hash)
            .collect();

        assert!(
            gapped.iter().any(|h| !renamed.contains(h)),
            "gapped must run a pass renamed does not; renamed={renamed:?} gapped={gapped:?}"
        );
        // Type-3 subsumes Type-2: gapped can never report LESS than renamed.
        for hash in &renamed {
            assert!(gapped.contains(hash), "gapped lost the renamed hash {hash}");
        }
    }

    /// The distinction is not cosmetic: the inserted statement means the two
    /// bodies do NOT share a Type-2 hash, so only a gap-tolerant bucket can pair
    /// them.
    #[test]
    fn only_gapped_pairs_bodies_separated_by_an_inserted_statement() {
        let renamed = find_duplicate_blocks(blocks_for(DuplicateType::Renamed));
        assert!(
            renamed.is_empty(),
            "one inserted statement defeats a Type-2 hash — that zero is a measurement: {renamed:?}"
        );

        let gapped = find_duplicate_blocks(blocks_for(DuplicateType::Gapped));
        let paired: Vec<_> = gapped.iter().filter(|d| d.locations.len() >= 2).collect();
        assert!(
            !paired.is_empty(),
            "gapped must pair two copies that differ by an inserted statement: {gapped:?}"
        );
        assert_eq!(
            paired[0].clone_type,
            CloneType::NearMiss,
            "statements differ, not merely names, so the class is Type-3"
        );
        assert!(
            paired[0].similarity < 1.0,
            "a measured near-miss is never reported as identical"
        );
    }

    /// A gapped bucket that turns out to be a Type-2 group is NOT a second
    /// duplicate: the Type-2 pass already reported it. Measured on
    /// `src/services/complexity`, keeping it listed 10 blocks under `gapped`
    /// against 9 under `fuzzy` over the identical 278 duplicate lines.
    #[test]
    fn gapped_does_not_double_count_a_type_2_group() {
        // Two copies with renamed identifiers and NO inserted statement: a
        // Type-2 group, which both the Type-2 and the gapped bucket can form.
        const TYPE_2_ONLY: &str = "\
fn alpha(items: &[u32]) -> u32 {
    let mut total = 0;
    for item in items {
        if *item > 10 {
            total += item;
        } else {
            total -= item;
        }
    }
    total
}
fn beta(values: &[u32]) -> u32 {
    let mut acc = 0;
    for value in values {
        if *value > 10 {
            acc += value;
        } else {
            acc -= value;
        }
    }
    acc
}
";
        let blocks = |kind: DuplicateType| {
            let lines: Vec<&str> = TYPE_2_ONLY.lines().collect();
            find_duplicate_blocks(extract_blocks(&lines, Path::new("dup.rs"), 5, 1000, kind))
        };

        let renamed = blocks(DuplicateType::Renamed);
        assert!(!renamed.is_empty(), "fixture must be a Type-2 clone");
        assert_eq!(
            renamed.len(),
            blocks(DuplicateType::Gapped).len(),
            "gapped must not report a Type-2 group a second time"
        );
    }

    /// `all` stays a superset of every sub-mode.
    #[test]
    fn all_is_a_superset_of_gapped() {
        let gapped: Vec<String> = blocks_for(DuplicateType::Gapped)
            .into_iter()
            .map(|(hash, ..)| hash)
            .collect();
        let all: Vec<String> = blocks_for(DuplicateType::All)
            .into_iter()
            .map(|(hash, ..)| hash)
            .collect();
        for hash in &gapped {
            assert!(all.contains(hash), "`all` dropped the gapped hash {hash}");
        }
    }

    /// The guards that stop the skeleton from grouping unrelated code: a body
    /// with no branching has no distinctive shape, and a body with nothing to
    /// elide is already covered by the Type-2 hash.
    #[test]
    fn skeleton_refuses_shapes_that_would_over_group() {
        // No control flow at all: `fn v ( v ) { }` describes almost every
        // function ever written.
        let straight_line = "fn v(x: usize) -> usize {\nlet a = x + 1;\nlet b = a * 2;\nb\n}";
        assert!(gapped_skeleton(straight_line, 5).is_none());

        // Nothing elided: every line is structural, so the skeleton would be the
        // Type-2 signature under another name.
        let all_structural = "for a {\nif b {\n} else {\n}\n}";
        assert!(gapped_skeleton(all_structural, 5).is_none());
    }

    /// Whole-word matching: `notify` must not read as `if`.
    #[test]
    fn control_flow_detection_is_word_exact() {
        assert!(carries_control_flow(&normalize_identifiers("if x {")));
        assert!(!carries_control_flow(&normalize_identifiers("notify(x);")));
        assert!(!carries_control_flow(&normalize_identifiers(
            "let iffy = 1;"
        )));
    }
}
