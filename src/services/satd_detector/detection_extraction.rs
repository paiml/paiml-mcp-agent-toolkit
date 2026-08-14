// SATD extraction: constructors, comment scanning, and context hashing.
//
// The comment scanner is the fix for #944: a comment was only recognised when
// it STARTED the line, so `pub fn a() -> i32 { 1 } // TODO: trailing one` was
// invisible and a 407-line file with 43 markers reported 3. It is also half the
// fix for #925: the scanner knows which leaders a file's language actually has
// and where its string literals are, so pmat's own `--help` text (a `#`-prefixed
// line inside a Rust raw string) is no longer read as a Python comment.

impl Default for SATDDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SATDDetector {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self::with_classifier(DebtClassifier::new())
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// New strict.
    pub fn new_strict() -> Self {
        Self::with_classifier(DebtClassifier::new_strict())
    }

    /// Extended mode: detects euphemisms like placeholder, stub, "for now"
    /// See issue #149
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new_extended() -> Self {
        Self::with_classifier(DebtClassifier::new_extended())
    }

    fn with_classifier(debt_classifier: DebtClassifier) -> Self {
        Self { debt_classifier }
    }

    /// Extract technical debt from source code content
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn extract_from_content(
        &self,
        content: &str,
        file_path: &Path,
    ) -> Result<Vec<TechnicalDebt>, TemplateError> {
        let mut debts = Vec::new();

        // Analyzer-owned, generated and vendored files are out of scope
        // wholesale; the per-line heuristics that used to guess at this are
        // gone (#925).
        if self.should_exclude_file(file_path) {
            return Ok(debts);
        }

        let mut test_tracker = TestBlockTracker::new(self.is_rust_file(file_path));
        let mut scanner = CommentScanner::for_path(file_path);

        for (line_num, line) in content.lines().enumerate() {
            if line.len() > MAX_LINE_LEN {
                return Err(TemplateError::ValidationError {
                    parameter: "line".to_string(),
                    reason: "Line too long for comment extraction (>10000 chars)".to_string(),
                });
            }

            test_tracker.update_from_line(line.trim());
            let comment = scanner.scan_line(line);

            if test_tracker.is_in_test_block() {
                continue;
            }
            if let Some(debt) = self.debt_of(comment, file_path, line_num as u32 + 1) {
                debts.push(debt);
            }
        }

        self.sort_debts(&mut debts);
        Ok(debts)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) fn is_rust_file(&self, file_path: &Path) -> bool {
        file_path.extension().and_then(|s| s.to_str()) == Some("rs")
    }

    fn sort_debts(&self, debts: &mut [TechnicalDebt]) {
        debts.sort_by_key(|d| (d.file.clone(), d.line, d.column));
    }

    /// The debt admitted by one scanned comment, if any.
    fn debt_of(
        &self,
        comment: Option<CommentSpan>,
        file_path: &Path,
        line_num: u32,
    ) -> Option<TechnicalDebt> {
        let comment = comment?;
        let (category, severity) = self.debt_classifier.classify_comment(&comment.text)?;

        // Basic context (could be enhanced with actual AST analysis)
        let context = AstContext {
            node_type: AstNodeType::Regular,
            parent_function: "unknown".to_string(),
            complexity: 1,
            siblings_count: 0,
            nesting_depth: 0,
            surrounding_statements: vec![],
        };

        Some(TechnicalDebt {
            category,
            severity: self.debt_classifier.adjust_severity(severity, &context),
            context_hash: self.hash_context(file_path, line_num, &comment.text),
            text: comment.text,
            file: file_path.to_path_buf(),
            line: line_num,
            column: comment.column,
        })
    }

    /// Generate context hash for debt identity tracking
    fn hash_context(&self, file_path: &Path, line_num: u32, content: &str) -> [u8; 16] {
        let mut hasher = Hasher::new();

        // Hash structural elements for stability across refactorings
        hasher.update(file_path.to_string_lossy().as_bytes());
        hasher.update(&line_num.to_le_bytes());
        hasher.update(content.as_bytes());

        let hash = hasher.finalize();
        hash.as_bytes()[..16].try_into().expect("internal error")
    }
}

/// Longest line the scanner will look at. A longer line is minified or
/// generated, not commented source.
const MAX_LINE_LEN: usize = 10_000;

/// One comment found on one line: where it starts (1-based column) and its text
/// with the leader stripped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommentSpan {
    pub(crate) column: u32,
    pub(crate) text: String,
}

/// Which comment leaders a file's language actually has.
///
/// #925: the extractor used to try `//`, `#`, `/*` and `<!--` on every file,
/// so a `#`-prefixed line inside a Rust raw string — pmat's own `--help` block
/// in `cli/commands/cli_struct.rs` — was read as a shell comment and reported
/// as `Medium / Design` debt. `#` in Rust is an attribute, never a comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CommentSyntax {
    /// `//` line comments and `/* … */` block comments.
    slash: bool,
    /// `#` line comments.
    hash: bool,
    /// `<!-- … -->` comments.
    html: bool,
    /// `'…'` delimits a string (shell/python) rather than a char literal.
    quote_is_string: bool,
}

impl CommentSyntax {
    /// The syntax of the language `path` is written in. An unknown extension
    /// gets the permissive union of the line-comment styles, since guessing
    /// wrong there can only be a missed comment or an unclassified one — never
    /// a wrong verdict, because classification still requires a marker.
    pub(crate) fn for_path(path: &Path) -> Self {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        match extension.as_str() {
            // C-family and friends: `//`, `/* */`, char literals in single quotes.
            "rs" | "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh" | "cs" | "java" | "js"
            | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "go" | "kt" | "kts" | "swift" | "scala"
            | "dart" | "proto" | "sol" | "zig" | "groovy" | "m" | "mm" | "css" | "scss"
            | "less" | "v" | "sv" => Self {
                slash: true,
                hash: false,
                html: false,
                quote_is_string: false,
            },
            // PHP takes both `//` and `#`.
            "php" => Self {
                slash: true,
                hash: true,
                html: false,
                quote_is_string: true,
            },
            // Hash-comment languages and configuration formats.
            "py" | "rb" | "sh" | "bash" | "zsh" | "fish" | "pl" | "pm" | "r" | "jl" | "nim"
            | "cr" | "ex" | "exs" | "yaml" | "yml" | "toml" | "ini" | "cfg" | "conf" | "tf"
            | "mk" | "cmake" | "ps1" | "awk" | "gradle" => Self {
                slash: false,
                hash: true,
                html: false,
                quote_is_string: true,
            },
            // Markup: only `<!-- -->` is a comment. A leading `#` is a heading.
            "html" | "htm" | "xml" | "xhtml" | "svg" | "md" | "markdown" | "vue" | "svelte" => {
                Self {
                    slash: false,
                    hash: false,
                    html: true,
                    quote_is_string: true,
                }
            }
            // Functional languages whose block/line comments are not modelled
            // here; `--` is not treated as a comment because `i--` is not one.
            "hs" | "elm" | "ml" | "clj" | "lua" | "sql" => Self {
                slash: false,
                hash: false,
                html: false,
                quote_is_string: true,
            },
            _ => Self {
                slash: true,
                hash: true,
                html: false,
                quote_is_string: true,
            },
        }
    }
}

/// Finds the comment on each line of ONE file, carrying the multi-line state
/// (block comments, raw strings) that a per-line regex cannot have.
pub(crate) struct CommentScanner {
    syntax: CommentSyntax,
    /// Inside a `/* … */` that opened on an earlier line.
    in_block_comment: bool,
    /// The block comment that is open is a doc comment (`/** … */`).
    block_is_doc: bool,
    /// Column the open block comment's `/*` sat at, on the line it opened.
    block_start: Option<usize>,
    /// Inside a raw string that opened on an earlier line; holds its closing
    /// delimiter (`"`, `"#`, `"##`, …). Test fixtures embedded in `.rs` files
    /// live here, and their `// TODO` lines are string data, not comments.
    raw_string_close: Option<String>,
}

impl CommentScanner {
    pub(crate) fn for_path(path: &Path) -> Self {
        Self::new(CommentSyntax::for_path(path))
    }

    pub(crate) fn new(syntax: CommentSyntax) -> Self {
        Self {
            syntax,
            in_block_comment: false,
            block_is_doc: false,
            block_start: None,
            raw_string_close: None,
        }
    }

    /// The first comment on `line`, at any column, or `None`.
    ///
    /// Doc comments (`///`, `//!`, `/** … */`) deliberately yield `None`: the
    /// long-standing policy is that documentation is not debt.
    pub(crate) fn scan_line(&mut self, line: &str) -> Option<CommentSpan> {
        let mut found: Option<CommentSpan> = None;
        let mut idx = 0usize;

        while idx < line.len() {
            if let Some(close) = self.raw_string_close.take() {
                match line[idx..].find(close.as_str()) {
                    Some(pos) => idx += pos + close.len(),
                    None => {
                        self.raw_string_close = Some(close);
                        break;
                    }
                }
                continue;
            }

            if self.in_block_comment {
                let (chunk, next) = match line[idx..].find("*/") {
                    Some(pos) => (&line[idx..idx + pos], Some(idx + pos + 2)),
                    None => (&line[idx..], None),
                };
                if !self.block_is_doc {
                    let column = self.block_start.unwrap_or(idx);
                    record(&mut found, column, block_comment_text(chunk));
                }
                match next {
                    Some(next) => {
                        self.in_block_comment = false;
                        self.block_is_doc = false;
                        self.block_start = None;
                        idx = next;
                        continue;
                    }
                    None => {
                        // Continuation lines start at their own column.
                        self.block_start = None;
                        break;
                    }
                }
            }

            let rest = &line[idx..];

            if let Some(consumed) = self.consume_string(line, idx) {
                idx += consumed;
                continue;
            }

            if self.syntax.slash && rest.starts_with("//") {
                // A line comment owns the rest of the line.
                if !rest.starts_with("///") && !rest.starts_with("//!") {
                    record(&mut found, idx, rest[2..].trim().to_string());
                }
                break;
            }

            if self.syntax.slash && rest.starts_with("/*") {
                self.in_block_comment = true;
                self.block_is_doc = rest.starts_with("/**") && !rest.starts_with("/**/");
                self.block_start = Some(idx);
                idx += 2;
                continue;
            }

            if self.syntax.hash && rest.starts_with('#') {
                record(&mut found, idx, rest[1..].trim().to_string());
                break;
            }

            if self.syntax.html && rest.starts_with("<!--") {
                let body = &rest[4..];
                let text = match body.find("-->") {
                    Some(pos) => &body[..pos],
                    None => body,
                };
                record(&mut found, idx, text.trim().to_string());
                break;
            }

            idx += next_char_len(rest);
        }

        found
    }

    /// If a string literal starts at `idx`, how many bytes of `line` it covers
    /// (to the end of the line when it is unterminated). `None` when `idx` is
    /// not the start of one.
    fn consume_string(&mut self, line: &str, idx: usize) -> Option<usize> {
        let rest = &line[idx..];

        // Rust raw string: r"…", r#"…"#, r##"…"##
        if rest.starts_with('r') && !preceded_by_ident_char(line, idx) {
            let hashes = rest[1..].len() - rest[1..].trim_start_matches('#').len();
            if rest[1 + hashes..].starts_with('"') {
                let close = format!("\"{}", "#".repeat(hashes));
                let body_at = 1 + hashes + 1;
                return Some(match rest[body_at..].find(close.as_str()) {
                    Some(pos) => body_at + pos + close.len(),
                    None => {
                        self.raw_string_close = Some(close);
                        rest.len()
                    }
                });
            }
        }

        if rest.starts_with('"') {
            return Some(quoted_len(rest, '"'));
        }

        if rest.starts_with('\'') {
            if self.syntax.quote_is_string {
                return Some(quoted_len(rest, '\''));
            }
            // A char literal — but `'a` in `&'a str` is a lifetime, which has
            // no closing quote.
            let len = quoted_len(rest, '\'');
            if rest[..len].ends_with('\'') && len > 1 {
                return Some(len);
            }
            return Some(1);
        }

        None
    }
}

/// Bytes covered by a `quote`-delimited literal starting at the front of
/// `rest`, honouring backslash escapes. An unterminated literal covers the
/// whole slice.
fn quoted_len(rest: &str, quote: char) -> usize {
    let mut chars = rest.char_indices();
    chars.next(); // opening quote
    let mut escaped = false;
    for (offset, ch) in chars {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return offset + ch.len_utf8();
        }
    }
    rest.len()
}

fn preceded_by_ident_char(line: &str, idx: usize) -> bool {
    line[..idx]
        .chars()
        .next_back()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
}

/// A block comment's continuation lines conventionally start with `*`.
fn block_comment_text(chunk: &str) -> String {
    chunk
        .trim()
        .trim_start_matches('*')
        .trim_start()
        .trim_end()
        .to_string()
}

fn record(found: &mut Option<CommentSpan>, idx: usize, text: String) {
    if found.is_none() && !text.is_empty() {
        *found = Some(CommentSpan {
            column: idx as u32 + 1,
            text,
        });
    }
}

fn next_char_len(rest: &str) -> usize {
    rest.chars().next().map_or(1, char::len_utf8)
}
