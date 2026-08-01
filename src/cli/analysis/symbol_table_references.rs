// Cross-reference resolution for the symbol table.
//
// Defect #654 (round 2): the previous fix gave every symbol exactly one
// `Reference` — its own `Definition` — and then derived the summary from
// `references.len()`. On the pmat source tree that produced
// `len(references) == 1` for all 16944 symbols, so `unreferenced_symbols`
// listed 16944 of 16944 (fraction 1.0) and `most_referenced` was
// `[["main",1],["main",1],["Cli",1],["Commands",1]]` — populated fields that
// were 100% wrong. Reference sites are now actually resolved from the sources.
//
// Resolution model (deliberately conservative — we only record references we
// can attribute, and we never claim "unreferenced" for a name we failed to
// resolve):
//
//   1. every identifier occurrence outside comments and string literals is a
//      candidate use site, except member accesses (`v.unwrap()`) and
//      re-declarations (`let name = …`); the definition's own occurrence is
//      skipped because it is already `ReferenceKind::Definition`;
//   2. a candidate declaration only counts if the occurrence has the right
//      shape for it — a function must actually be *called* (`name(`), while a
//      type is accepted in any position;
//   3. if the enclosing file declares that name, the use is attributed there
//      (locality wins); otherwise only declarations visible outside their own
//      file are candidates, and the use is attributed only when exactly one
//      fits;
//   4. if several visible declarations share the name (e.g. `new`, declared in
//      hundreds of files) the use is NOT attributed to a guess, and the name is
//      recorded as unresolved so it can never be reported as "unreferenced";
//   5. if no visible declaration exists, the identifier belongs to something we
//      did not extract. Nothing is attributed, and the file-private
//      declarations of that name stay fully measured within their own file.

use std::collections::HashSet;

/// One source file held in memory for both definition extraction and use-site scanning.
pub(crate) struct FileSource {
    pub path: String,
    pub content: String,
}

/// Which comment syntax a file uses. Identifiers inside comments and string
/// literals are not references, so they are masked out before scanning.
#[derive(Clone, Copy, PartialEq, Eq)]
enum CommentSyntax {
    /// `//` line comments and `/* */` block comments (rust, c, c++, java, js, ts, go).
    CStyle,
    /// `#` line comments (python, ruby).
    Hash,
}

fn comment_syntax_for(path: &str) -> CommentSyntax {
    match Path::new(path).extension().and_then(|e| e.to_str()) {
        Some("py" | "rb") => CommentSyntax::Hash,
        _ => CommentSyntax::CStyle,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MaskState {
    Code,
    LineComment,
    BlockComment,
    StringLiteral,
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Blank out comments and string literals, preserving every byte offset so that
/// line/column positions still line up with the definitions we extracted.
fn mask_non_code(content: &str, syntax: CommentSyntax) -> Vec<u8> {
    let bytes = content.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut state = MaskState::Code;
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] == b'\n' {
            out.push(b'\n');
            // An unterminated quote must not desynchronise the rest of the file.
            if state == MaskState::LineComment || state == MaskState::StringLiteral {
                state = MaskState::Code;
            }
            i += 1;
            continue;
        }
        i += match state {
            MaskState::Code => step_code(bytes, i, syntax, &mut state, &mut out),
            MaskState::LineComment => blank(&mut out, 1),
            MaskState::BlockComment => step_block_comment(bytes, i, &mut state, &mut out),
            MaskState::StringLiteral => step_string(bytes, i, &mut state, &mut out),
        };
    }

    out
}

/// Emit `n` blanks for masked-out input; returns the number of bytes consumed.
fn blank(out: &mut Vec<u8>, n: usize) -> usize {
    out.extend(std::iter::repeat_n(b' ', n));
    n
}

fn step_code(
    bytes: &[u8],
    i: usize,
    syntax: CommentSyntax,
    state: &mut MaskState,
    out: &mut Vec<u8>,
) -> usize {
    if let Some((opened, width)) = opening_delimiter(bytes[i], bytes.get(i + 1).copied(), syntax) {
        *state = opened;
        return blank(out, width);
    }
    out.push(bytes[i]);
    1
}

/// The masked region a delimiter opens, and how many bytes the delimiter spans.
fn opening_delimiter(
    b: u8,
    next: Option<u8>,
    syntax: CommentSyntax,
) -> Option<(MaskState, usize)> {
    match (syntax, b, next) {
        (CommentSyntax::CStyle, b'/', Some(b'/')) => Some((MaskState::LineComment, 2)),
        (CommentSyntax::CStyle, b'/', Some(b'*')) => Some((MaskState::BlockComment, 2)),
        (CommentSyntax::Hash, b'#', _) => Some((MaskState::LineComment, 1)),
        (_, b'"', _) => Some((MaskState::StringLiteral, 1)),
        _ => None,
    }
}

fn step_block_comment(bytes: &[u8], i: usize, state: &mut MaskState, out: &mut Vec<u8>) -> usize {
    if bytes[i] == b'*' && bytes.get(i + 1).copied() == Some(b'/') {
        *state = MaskState::Code;
        return blank(out, 2);
    }
    blank(out, 1)
}

fn step_string(bytes: &[u8], i: usize, state: &mut MaskState, out: &mut Vec<u8>) -> usize {
    if bytes[i] == b'\\' && bytes.get(i + 1).is_some_and(|&n| n != b'\n') {
        return blank(out, 2);
    }
    if bytes[i] == b'"' {
        *state = MaskState::Code;
    }
    blank(out, 1)
}

/// Keywords that introduce a *new* binding. The identifier that follows one is a
/// declaration, not a use of a same-named symbol declared elsewhere — `let name`
/// does not reference a `const name` in another file. `impl` is deliberately
/// absent: `impl Config` really is a use of `Config`.
const BINDING_KEYWORDS: [&str; 13] = [
    "fn",
    "struct",
    "enum",
    "class",
    "def",
    "function",
    "interface",
    "trait",
    "mod",
    "let",
    "const",
    "static",
    "type",
];

/// Index just past the last non-whitespace byte before `i`, or `None` at the
/// start of input. Whitespace includes newlines so that method chains written
/// across lines are still seen as `.method`.
fn skip_whitespace_back(masked: &[u8], mut i: usize) -> Option<usize> {
    while i > 0 && masked[i - 1].is_ascii_whitespace() {
        i -= 1;
    }
    (i > 0).then_some(i)
}

/// `x.unwrap()` / `p->field` name a member of a value, not the free function or
/// type of that name. Without this, `unwrap` — declared once somewhere in the
/// tree — collected 19112 "references" from every `.unwrap()` in pmat, and led
/// `most_referenced`.
fn is_member_access(masked: &[u8], start: usize, member_arrow: bool) -> bool {
    let Some(i) = skip_whitespace_back(masked, start) else {
        return false;
    };
    if masked[i - 1] == b'.' {
        // `..Default::default()` is a struct-update base, not a member access.
        return !(i >= 2 && masked[i - 2] == b'.');
    }
    member_arrow && masked[i - 1] == b'>' && i >= 2 && masked[i - 2] == b'-'
}

/// True when the identifier is preceded by a binding keyword, i.e. it is being
/// declared here rather than used.
fn follows_binding_keyword(masked: &[u8], start: usize) -> bool {
    let Some(end) = skip_whitespace_back(masked, start) else {
        return false;
    };
    let mut begin = end;
    while begin > 0 && is_ident_char(masked[begin - 1]) {
        begin -= 1;
    }
    begin != end
        && std::str::from_utf8(&masked[begin..end])
            .is_ok_and(|word| BINDING_KEYWORDS.contains(&word))
}

/// An identifier occurrence: `line` is 1-based and `column` is a byte offset
/// within the line — exactly the coordinate system `Symbol { line, column }` uses.
struct Occurrence {
    line: usize,
    column: usize,
    start: usize,
    end: usize,
    /// `name(` — the only shape that is a *call* of a function.
    is_call: bool,
}

/// Whether the line beginning at `from` is a Rust attribute (`#[…]` / `#![…]`).
///
/// Attribute arguments are not references: `#[derive(Debug)]` made `derive` —
/// which happens to be declared as a `pub fn derive(` in the tree — the single
/// most referenced symbol in pmat with 3028 hits.
fn starts_attribute(masked: &[u8], from: usize) -> bool {
    let mut i = from;
    while i < masked.len() && (masked[i] == b' ' || masked[i] == b'\t') {
        i += 1;
    }
    if masked.get(i) != Some(&b'#') {
        return false;
    }
    let after = masked.get(i + 1);
    after == Some(&b'[') || (after == Some(&b'!') && masked.get(i + 2) == Some(&b'['))
}

/// Whether the next non-blank byte on this line opens an argument list.
fn opens_call(masked: &[u8], end: usize) -> bool {
    let mut i = end;
    while i < masked.len() && (masked[i] == b' ' || masked[i] == b'\t') {
        i += 1;
    }
    masked.get(i) == Some(&b'(')
}

/// Whether an occurrence can plausibly be a use of this declaration.
///
/// A function is referenced by being *called*. Without this, `pub fn path` in
/// `src/cli/colors.rs` collected 9400 "references" from every parameter and
/// local named `path` in the tree, and led `most_referenced` — a measured but
/// meaningless number. Types are accepted in any position, because a type use
/// has no single syntactic shape.
fn is_plausible_use(symbol: &Symbol, occ: &Occurrence) -> bool {
    match symbol.kind {
        SymbolKind::Function | SymbolKind::Method => occ.is_call,
        _ => true,
    }
}

/// Every occurrence of a *known* symbol name in masked source, in source order,
/// excluding member accesses and re-declarations.
fn collect_known_occurrences(
    masked: &[u8],
    known: &DefinitionIndex,
    member_arrow: bool,
) -> Vec<Occurrence> {
    let mut found = Vec::new();
    let mut line = 1usize;
    let mut line_start = 0usize;
    let mut attribute_line = starts_attribute(masked, 0);
    let mut i = 0usize;

    while i < masked.len() {
        let b = masked[i];
        if b == b'\n' {
            line += 1;
            i += 1;
            line_start = i;
            attribute_line = starts_attribute(masked, line_start);
            continue;
        }
        if !is_ident_start(b) {
            i += 1;
            continue;
        }
        let start = i;
        while i < masked.len() && is_ident_char(masked[i]) {
            i += 1;
        }
        if !attribute_line && is_use_candidate(masked, start, i, known, member_arrow) {
            found.push(Occurrence {
                line,
                column: start - line_start,
                start,
                end: i,
                is_call: opens_call(masked, i),
            });
        }
    }

    found
}

/// Whether the identifier spanning `start..end` could name a declared symbol.
fn is_use_candidate(
    masked: &[u8],
    start: usize,
    end: usize,
    known: &DefinitionIndex,
    member_arrow: bool,
) -> bool {
    known.is_known(&masked[start..end])
        && !is_member_access(masked, start, member_arrow)
        && !follows_binding_keyword(masked, start)
}

/// `->` is member access in C and C++ but a return-type arrow in Rust.
fn uses_arrow_member_access(path: &str) -> bool {
    matches!(
        Path::new(path).extension().and_then(|e| e.to_str()),
        Some("c" | "cpp" | "h" | "hpp")
    )
}

/// Indices into the symbol slice, grouped for attribution.
struct DefinitionIndex {
    by_name: HashMap<String, Vec<usize>>,
    /// Only the declarations that are visible outside their own file — the only
    /// ones a use in *another* file could possibly be talking about.
    exported_by_name: HashMap<String, Vec<usize>>,
    by_file: HashMap<String, FileDefinitions>,
}

/// Whether a declaration can be referenced from another file at all. A bare
/// `fn helper` in Rust, `function helper` in JS or `const e = …` are private to
/// their file, so an identically named identifier elsewhere is a different
/// thing — attributing to it made `name` (declared once, non-public) collect
/// 6386 "references" from every unrelated local variable called `name`.
fn is_exported(symbol: &Symbol) -> bool {
    matches!(symbol.visibility, Visibility::Public | Visibility::Protected)
}

#[derive(Default)]
struct FileDefinitions {
    by_name: HashMap<String, Vec<usize>>,
    sites: HashSet<(usize, usize)>,
}

impl DefinitionIndex {
    fn is_known(&self, ident: &[u8]) -> bool {
        std::str::from_utf8(ident).is_ok_and(|name| self.by_name.contains_key(name))
    }
}

fn build_definition_index(symbols: &[Symbol]) -> DefinitionIndex {
    let mut by_name: HashMap<String, Vec<usize>> = HashMap::new();
    let mut exported_by_name: HashMap<String, Vec<usize>> = HashMap::new();
    let mut by_file: HashMap<String, FileDefinitions> = HashMap::new();

    for (idx, symbol) in symbols.iter().enumerate() {
        by_name.entry(symbol.name.clone()).or_default().push(idx);
        if is_exported(symbol) {
            exported_by_name
                .entry(symbol.name.clone())
                .or_default()
                .push(idx);
        }
        let per_file = by_file.entry(symbol.file.clone()).or_default();
        per_file
            .by_name
            .entry(symbol.name.clone())
            .or_default()
            .push(idx);
        per_file.sites.insert((symbol.line, symbol.column));
    }

    DefinitionIndex {
        by_name,
        exported_by_name,
        by_file,
    }
}

/// Resolve use sites for every symbol, appending `ReferenceKind::Usage`
/// references in file order (deterministic). Returns the set of names that had
/// at least one occurrence we could not attribute — those names must never be
/// reported as unreferenced, because we did not actually measure them.
pub(crate) fn resolve_references(files: &[FileSource], symbols: &mut [Symbol]) -> HashSet<String> {
    let index = build_definition_index(symbols);
    let mut unresolved: HashSet<String> = HashSet::new();

    for file in files {
        let masked = mask_non_code(&file.content, comment_syntax_for(&file.path));
        let occurrences =
            collect_known_occurrences(&masked, &index, uses_arrow_member_access(&file.path));
        let local = index.by_file.get(&file.path);
        for occ in occurrences {
            if local.is_some_and(|defs| defs.sites.contains(&(occ.line, occ.column))) {
                continue; // the definition itself; already recorded
            }
            let Ok(name) = std::str::from_utf8(&masked[occ.start..occ.end]) else {
                continue;
            };
            attribute_use(&index, local, symbols, file, &occ, name, &mut unresolved);
        }
    }

    unresolved
}

fn attribute_use(
    index: &DefinitionIndex,
    local: Option<&FileDefinitions>,
    symbols: &mut [Symbol],
    file: &FileSource,
    occ: &Occurrence,
    name: &str,
    unresolved: &mut HashSet<String>,
) {
    let reference = Reference {
        file: file.path.clone(),
        line: occ.line,
        column: occ.column,
        kind: ReferenceKind::Usage,
    };

    // Locality wins: a use in a file that declares the name refers to that
    // declaration — and to nothing further away, even if it does not look like
    // a use of it.
    if let Some(indices) = local.and_then(|defs| defs.by_name.get(name)) {
        for idx in plausible(symbols, indices, occ) {
            symbols[idx].references.push(reference.clone());
        }
        return;
    }

    let Some(candidates) = index.exported_by_name.get(name) else {
        // No declaration of this name is visible outside its own file, so this
        // identifier is something else entirely. Nothing to attribute, and the
        // file-private declarations remain fully measured within their file.
        return;
    };

    let matching = plausible(symbols, candidates, occ);
    match matching.len() {
        0 => {}
        // Exactly one visible declaration fits: safe to attribute.
        1 => symbols[matching[0]].references.push(reference),
        // Several visible declarations share this name and none is in this file.
        // We cannot tell which one is meant, so we attribute nothing and record
        // the name as unresolved — reporting it as "unreferenced" would be a
        // fabricated measurement.
        _ => {
            unresolved.insert(name.to_string());
        }
    }
}

fn plausible(symbols: &[Symbol], candidates: &[usize], occ: &Occurrence) -> Vec<usize> {
    candidates
        .iter()
        .copied()
        .filter(|&idx| is_plausible_use(&symbols[idx], occ))
        .collect()
}

/// Number of resolved use sites (i.e. references that are not the definition).
pub(crate) fn usage_count(symbol: &Symbol) -> usize {
    symbol
        .references
        .iter()
        .filter(|r| !matches!(r.kind, ReferenceKind::Definition))
        .count()
}
