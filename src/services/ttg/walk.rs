//! The single token walk that produces `T`, `D` and `N` (spec §2.1).
//!
//! `T` is just the token count after attribute stripping. `D` and `N` come
//! from one left-to-right pass with a delimiter-nesting stack. The walk never
//! sees a newline, so both are exactly invariant to line breaking.

use super::lexer::{Tok, TokKind};

/// Identifiers that cannot end an expression, so a following `||` must be a
/// zero-argument closure rather than boolean-or.
///
/// The last entry is `'lt'` **with a trailing quote**, while the lexer emits
/// lifetimes as `'lt` without one. The two therefore never compare equal and a
/// lifetime *does* count as ending an expression. That is the reference
/// implementation's behaviour, reproduced deliberately; see the port notes.
const NOT_EXPR_END: &[&str] = &[
    "return", "else", "in", "mut", "ref", "move", "await", "as", "where", "impl", "fn", "let",
    "const", "static", "match", "if", "while", "for", "loop", "yield", "break", "continue", "'lt'",
];

/// Can this token close an expression? Used only to tell boolean-or from a
/// closure head.
fn ends_expression(t: &Tok<'_>) -> bool {
    match t.kind {
        TokKind::Lit | TokKind::Close => true,
        TokKind::Ident => !NOT_EXPR_END.contains(&t.text),
        TokKind::Punct => t.text == "?",
        TokKind::Open => false,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RunOp {
    And,
    Or,
}

/// The identifiers seen at one delimiter depth since the last statement
/// separator. Only two questions are ever asked of it, so only those are kept.
#[derive(Clone, Copy, Default)]
struct StmtHead {
    len: u32,
    first_is_let: bool,
    has_impl: bool,
}

impl StmtHead {
    fn push(&mut self, ident: &str) {
        if self.len == 0 {
            self.first_is_let = ident == "let";
        }
        if ident == "impl" {
            self.has_impl = true;
        }
        self.len += 1;
    }

    fn starts_with_let(&self) -> bool {
        self.len > 0 && self.first_is_let
    }

    fn clear(&mut self) {
        *self = StmtHead::default();
    }
}

struct Walk {
    d: u32,
    max_nesting: u32,
    ctrl_open: u32,
    /// One entry per open `{`: whether a control-flow keyword introduced it.
    stack: Vec<bool>,
    pending_ctrl: bool,
    depth: usize,
    /// Per depth: the boolean operator whose run is currently open there.
    runop: Vec<Option<RunOp>>,
    /// Per depth: the current statement head.
    heads: Vec<StmtHead>,
}

impl Walk {
    fn new() -> Self {
        Walk {
            d: 0,
            max_nesting: 0,
            ctrl_open: 0,
            stack: Vec::new(),
            pending_ctrl: false,
            depth: 0,
            runop: vec![None],
            heads: vec![StmtHead::default()],
        }
    }

    fn grow(&mut self) {
        while self.runop.len() <= self.depth {
            self.runop.push(None);
            self.heads.push(StmtHead::default());
        }
    }

    fn set_runop(&mut self, op: Option<RunOp>) {
        self.runop[self.depth] = op;
    }

    fn head(&mut self) -> &mut StmtHead {
        &mut self.heads[self.depth]
    }

    fn on_ident(&mut self, x: &str, next: Option<&Tok<'_>>) {
        match x {
            "if" | "while" | "loop" | "match" => {
                self.d += 1;
                self.pending_ctrl = true;
            }
            "for" => self.on_for(next),
            "else" => self.on_else(),
            "fn" => self.pending_ctrl = false,
            _ => {}
        }
        self.head().push(x);
    }

    /// `for` is a loop unless it is the HRTB `for<'a>` or the `for` of an
    /// `impl Trait for Type` header.
    fn on_for(&mut self, next: Option<&Tok<'_>>) {
        let hrtb = matches!(next, Some(t) if t.kind == TokKind::Punct && t.text == "<");
        if !hrtb && !self.heads[self.depth].has_impl {
            self.d += 1;
            self.pending_ctrl = true;
        }
    }

    /// Plain `else` is free; `let PAT = EXPR else { … }` is a divergence and
    /// charges 1.
    fn on_else(&mut self) {
        if self.heads[self.depth].starts_with_let() {
            self.d += 1;
        }
        self.pending_ctrl = true;
    }

    fn on_punct(&mut self, x: &str, prev: Option<&Tok<'_>>) {
        match x {
            "&&" => self.on_bool_op(RunOp::And),
            "||" => self.on_or(prev),
            ";" | "=>" | "," => {
                self.set_runop(None);
                self.head().clear();
            }
            // Still inside one expression: an open boolean run survives.
            "=" | "?" | "." | "::" | "->" | ":" => {}
            _ => self.set_runop(None),
        }
    }

    fn on_or(&mut self, prev: Option<&Tok<'_>>) {
        if prev.is_some_and(ends_expression) {
            self.on_bool_op(RunOp::Or);
        } else {
            // A zero-argument closure head. Charges nothing, and closes any
            // run that was open.
            self.set_runop(None);
        }
    }

    /// Only the operator that OPENS a run charges; the rest of the run is free.
    fn on_bool_op(&mut self, op: RunOp) {
        if self.runop[self.depth] != Some(op) {
            self.d += 1;
            self.set_runop(Some(op));
        }
    }

    fn on_open(&mut self, x: &str) {
        self.depth += 1;
        self.grow();
        self.set_runop(None);
        self.head().clear();
        if x != "{" {
            return;
        }
        self.stack.push(self.pending_ctrl);
        if self.pending_ctrl {
            self.ctrl_open += 1;
            self.max_nesting = self.max_nesting.max(self.ctrl_open);
        }
        self.pending_ctrl = false;
    }

    fn on_close(&mut self, x: &str) {
        self.set_runop(None);
        self.head().clear();
        self.depth = self.depth.saturating_sub(1);
        if x != "}" {
            return;
        }
        if let Some(was_ctrl) = self.stack.pop() {
            if was_ctrl {
                self.ctrl_open = self.ctrl_open.saturating_sub(1);
            }
        }
        self.head().clear();
        self.set_runop(None);
    }
}

/// Rust decision walk. Returns `(D, N)`.
pub(super) fn walk_rust(toks: &[Tok<'_>]) -> (u32, u32) {
    let mut w = Walk::new();
    let mut prev: Option<&Tok<'_>> = None;
    for (i, t) in toks.iter().enumerate() {
        match t.kind {
            TokKind::Ident => w.on_ident(t.text, toks.get(i + 1)),
            TokKind::Punct => w.on_punct(t.text, prev),
            TokKind::Open => w.on_open(t.text),
            TokKind::Close => w.on_close(t.text),
            TokKind::Lit => {}
        }
        prev = Some(t);
    }
    (w.d, w.max_nesting)
}

const C_DECISION_WORDS: &[&str] = &[
    "if", "elif", "while", "for", "case", "catch", "match", "loop", "switch",
];

fn is_c_decision(t: &Tok<'_>) -> bool {
    match t.kind {
        TokKind::Ident => C_DECISION_WORDS.contains(&t.text),
        TokKind::Punct => t.text == "&&" || t.text == "||",
        _ => false,
    }
}

/// C-family decision count: every branching keyword and every short-circuit
/// operator, with no run collapsing and no nesting. Returns `D`.
pub(super) fn walk_c_family(toks: &[Tok<'_>]) -> u32 {
    toks.iter().filter(|t| is_c_decision(t)).count() as u32
}
