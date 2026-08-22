//! Token-level lexer for the TTG (Token-Tree Grade) base measures.
//!
//! This is a direct port of the reference implementation's `lex` and
//! `strip_attrs` (spec §2.1). It is deliberately *resilient*, not correct in
//! the rustc sense: it never fails and never aborts on malformed input, because
//! the definitions it scans are chunker output and 0.75% of them have
//! unbalanced delimiters. On anything it does not recognise it advances one
//! byte and emits no token.
//!
//! Three properties are load-bearing and are what make the TTG measures
//! invariant to formatting:
//!
//! - comments (`//`, `///`, `//!`, `/* */` **nested**) produce no tokens;
//! - a string or char literal is exactly **one** token whatever it contains,
//!   so control-flow keywords inside a codegen string are invisible;
//! - attribute spans (`#`/`#!` plus the bracketed group) are removed.

/// What a token is, for the purposes of the decision walk.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TokKind {
    /// An identifier or keyword. Lifetimes are folded to the marker `'lt`.
    Ident,
    /// A string, char or numeric literal. Always exactly one token.
    Lit,
    /// Punctuation, matched greedily against [`PUNCT`].
    Punct,
    /// One of `(`, `[`, `{`.
    Open,
    /// One of `)`, `]`, `}`.
    Close,
}

/// A lexed token: its kind plus the text the decision walk keys on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tok<'a> {
    /// Token class.
    pub kind: TokKind,
    /// For `Ident` the identifier text (or `'lt` for a lifetime); for `Punct`
    /// the operator; for `Open`/`Close` the single delimiter character; for
    /// `Lit` a class marker (`"`, `'` or `0`) and never the literal's content.
    pub text: &'a str,
}

/// Multi-character punctuation, longest-useful-first.
///
/// The **order is load-bearing**: matching walks this list and takes the first
/// entry that is a prefix of the remaining input, so `&&` must precede `&=`
/// and `#!` must precede `#`. Two entries can never match — the second `&&`
/// (the first already shadows it) and `_` (the identifier rule claims it
/// first). They are kept so this table is byte-identical to the reference.
pub const PUNCT: &[&str] = &[
    "<<=", ">>=", "...", "..=", "::", "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "..", "+=",
    "-=", "*=", "/=", "%=", "^=", "&=", "|=", "<<", ">>", "#!", "+", "-", "*", "/", "%", "^", "!",
    "&", "|", "&&", "=", "<", ">", "@", "_", ".", ",", ";", ":", "#", "$", "?", "~",
];

/// Length in bytes of the UTF-8 sequence introduced by `lead`.
///
/// Continuation and invalid bytes report 1, which keeps the scanner advancing
/// on malformed input instead of stalling.
fn char_len(lead: u8) -> usize {
    match lead {
        0x00..=0x7F => 1,
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => 1,
    }
}

struct Lexer<'a> {
    src: &'a str,
    b: &'a [u8],
    n: usize,
    i: usize,
    out: Vec<Tok<'a>>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        let b = src.as_bytes();
        Lexer {
            src,
            b,
            n: b.len(),
            i: 0,
            out: Vec::new(),
        }
    }

    fn push(&mut self, kind: TokKind, text: &'a str) {
        self.out.push(Tok { kind, text });
    }

    fn starts(&self, pat: &[u8]) -> bool {
        self.b[self.i..].starts_with(pat)
    }

    fn at(&self, p: usize) -> Option<u8> {
        self.b.get(p).copied()
    }

    fn run(mut self) -> Vec<Tok<'a>> {
        while self.i < self.n {
            if !self.step() {
                self.i += 1;
            }
        }
        self.out
    }

    fn step(&mut self) -> bool {
        self.skip_ws()
            || self.skip_comment()
            || self.lex_literal()
            || self.lex_word()
            || self.lex_symbol()
    }

    fn lex_literal(&mut self) -> bool {
        self.lex_raw_string() || self.lex_string() || self.lex_char_or_lifetime()
    }

    fn lex_word(&mut self) -> bool {
        self.lex_ident() || self.lex_number()
    }

    fn lex_symbol(&mut self) -> bool {
        self.lex_delim() || self.lex_punct()
    }

    fn skip_ws(&mut self) -> bool {
        let c = self.b[self.i];
        let ws = c == b' ' || c == b'\t' || c == b'\r' || c == b'\n';
        if ws {
            self.i += 1;
        }
        ws
    }

    fn skip_comment(&mut self) -> bool {
        if self.starts(b"//") {
            self.skip_line_comment();
            return true;
        }
        if self.starts(b"/*") {
            self.skip_block_comment();
            return true;
        }
        false
    }

    fn skip_line_comment(&mut self) {
        match self.b[self.i..].iter().position(|&c| c == b'\n') {
            Some(off) => self.i += off + 1,
            None => self.i = self.n,
        }
    }

    /// Rust block comments nest, so this tracks depth rather than scanning for
    /// the first `*/`.
    fn skip_block_comment(&mut self) {
        let mut depth = 1usize;
        self.i += 2;
        while self.i < self.n && depth > 0 {
            if self.starts(b"/*") {
                depth += 1;
                self.i += 2;
            } else if self.starts(b"*/") {
                depth -= 1;
                self.i += 2;
            } else {
                self.i += 1;
            }
        }
    }

    /// `r"…"`, `r#"…"#`, `br#"…"#`. Returns the offset just past the opening
    /// quote and the hash count, or `None` when this is not a raw string.
    fn raw_string_open(&self) -> Option<(usize, usize)> {
        let mut k = self.i;
        if self.at(k) == Some(b'b') {
            k += 1;
        }
        if self.at(k) != Some(b'r') {
            return None;
        }
        k += 1;
        let hash_start = k;
        while self.at(k) == Some(b'#') {
            k += 1;
        }
        if self.at(k) != Some(b'"') {
            return None;
        }
        Some((k + 1, k - hash_start))
    }

    fn lex_raw_string(&mut self) -> bool {
        let Some((body, hashes)) = self.raw_string_open() else {
            return false;
        };
        self.i = body;
        self.i = self.raw_string_end(hashes);
        self.push(TokKind::Lit, "\"");
        true
    }

    fn raw_string_end(&self, hashes: usize) -> usize {
        let mut p = self.i;
        while p < self.n {
            if self.b[p] == b'"' && self.hashes_at(p + 1, hashes) {
                return p + 1 + hashes;
            }
            p += 1;
        }
        self.n
    }

    fn hashes_at(&self, from: usize, count: usize) -> bool {
        from + count <= self.n && self.b[from..from + count].iter().all(|&c| c == b'#')
    }

    fn lex_string(&mut self) -> bool {
        let c = self.b[self.i];
        let byte_str = c == b'b' && self.at(self.i + 1) == Some(b'"');
        if c != b'"' && !byte_str {
            return false;
        }
        self.i += if c == b'"' { 1 } else { 2 };
        self.skip_string_body();
        self.push(TokKind::Lit, "\"");
        true
    }

    fn skip_string_body(&mut self) {
        while self.i < self.n {
            if self.b[self.i] == b'\\' {
                self.i += 2;
                continue;
            }
            if self.b[self.i] == b'"' {
                self.i += 1;
                break;
            }
            self.i += 1;
        }
    }

    fn lex_char_or_lifetime(&mut self) -> bool {
        if self.b[self.i] != b'\'' {
            return false;
        }
        if let Some(end) = self.char_literal_end() {
            self.i = end;
            self.push(TokKind::Lit, "'");
            return true;
        }
        if let Some(end) = ident_end(self.b, self.i + 1) {
            self.i = end;
            self.push(TokKind::Ident, "'lt");
            return true;
        }
        // A stray quote: consume it, emit nothing.
        self.i += 1;
        true
    }

    /// Mirrors the reference regex `'(\\.[^']*|[^\\'])'`.
    fn char_literal_end(&self) -> Option<usize> {
        let p = self.i + 1;
        if self.at(p) == Some(b'\\') {
            return self.escaped_char_end(p);
        }
        self.plain_char_end(p)
    }

    fn escaped_char_end(&self, backslash: usize) -> Option<usize> {
        let c = self.at(backslash + 1)?;
        // `.` in the reference regex does not match a newline.
        if c == b'\n' {
            return None;
        }
        let mut r = backslash + 1 + char_len(c);
        while r < self.n && self.b[r] != b'\'' {
            r += 1;
        }
        if r < self.n {
            Some(r + 1)
        } else {
            None
        }
    }

    fn plain_char_end(&self, p: usize) -> Option<usize> {
        let c = self.at(p)?;
        if c == b'\\' || c == b'\'' {
            return None;
        }
        let q = p + char_len(c);
        if self.at(q) == Some(b'\'') {
            Some(q + 1)
        } else {
            None
        }
    }

    fn lex_ident(&mut self) -> bool {
        let Some(end) = ident_end(self.b, self.i) else {
            return false;
        };
        let text = &self.src[self.i..end];
        self.i = end;
        self.push(TokKind::Ident, text);
        true
    }

    /// The reference number rule is `[0-9][0-9A-Za-z_.]*`, which deliberately
    /// swallows suffixes, separators and dots. `0..10` is therefore ONE token
    /// and `t.0.1` is three. Faithfulness to that beats tidiness here.
    fn lex_number(&mut self) -> bool {
        if !self.b[self.i].is_ascii_digit() {
            return false;
        }
        let mut e = self.i + 1;
        while e < self.n
            && (self.b[e].is_ascii_alphanumeric() || self.b[e] == b'_' || self.b[e] == b'.')
        {
            e += 1;
        }
        self.i = e;
        self.push(TokKind::Lit, "0");
        true
    }

    fn lex_delim(&mut self) -> bool {
        let kind = match self.b[self.i] {
            b'(' | b'[' | b'{' => TokKind::Open,
            b')' | b']' | b'}' => TokKind::Close,
            _ => return false,
        };
        let text = &self.src[self.i..self.i + 1];
        self.i += 1;
        self.push(kind, text);
        true
    }

    fn lex_punct(&mut self) -> bool {
        for p in PUNCT {
            if self.starts(p.as_bytes()) {
                self.i += p.len();
                self.push(TokKind::Punct, p);
                return true;
            }
        }
        false
    }
}

fn ident_end(b: &[u8], from: usize) -> Option<usize> {
    let c = *b.get(from)?;
    if !(c.is_ascii_alphabetic() || c == b'_') {
        return None;
    }
    let mut e = from + 1;
    while e < b.len() && (b[e].is_ascii_alphanumeric() || b[e] == b'_') {
        e += 1;
    }
    Some(e)
}

/// Tokenize `src`. Never fails; unrecognised bytes are skipped silently.
pub fn lex(src: &str) -> Vec<Tok<'_>> {
    Lexer::new(src).run()
}

fn is_attr_start(t: &Tok<'_>) -> bool {
    t.kind == TokKind::Punct && (t.text == "#" || t.text == "#!")
}

fn opens_bracket(t: Option<&Tok<'_>>) -> bool {
    matches!(t, Some(t) if t.kind == TokKind::Open && t.text == "[")
}

/// Index just past the delimiter group that starts at `j`, or the end of the
/// stream when the group never closes.
fn end_of_group(toks: &[Tok<'_>], mut j: usize) -> usize {
    let mut depth: i32 = 0;
    while j < toks.len() {
        match toks[j].kind {
            TokKind::Open => depth += 1,
            TokKind::Close => {
                depth -= 1;
                if depth == 0 {
                    return j + 1;
                }
            }
            _ => {}
        }
        j += 1;
    }
    toks.len()
}

/// Drop `#[…]` and `#![…]` spans. A `#` not followed by `[` is kept, which is
/// what makes a raw identifier (`r#type`) survive as three ordinary tokens.
pub fn strip_attrs<'a>(toks: &[Tok<'a>]) -> Vec<Tok<'a>> {
    let mut out: Vec<Tok<'a>> = Vec::with_capacity(toks.len());
    let mut i = 0;
    while i < toks.len() {
        if is_attr_start(&toks[i]) && opens_bracket(toks.get(i + 1)) {
            i = end_of_group(toks, i + 1);
            continue;
        }
        out.push(toks[i]);
        i += 1;
    }
    out
}
