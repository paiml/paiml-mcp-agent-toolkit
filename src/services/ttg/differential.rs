//! Differential acceptance test: the Rust base measures against the Python oracle.
//!
//! This is the acceptance oracle for TTG. It proves that [`super::measure`] and
//! [`super::measure_c_family`] agree **exactly** with the reference
//! implementation the specification was written against — on every definition
//! in the repository index, and on a generated adversarial corpus that covers
//! the lexer rules the real corpus cannot reach.
//!
//! # How to run it
//!
//! Both differentials are `#[ignore]`d: they need `python3` on `PATH`, and the
//! corpus one also needs a built `.pmat/context.db`. Neither is guaranteed in
//! CI, and a differential that silently passes because its oracle was missing
//! is worse than no differential at all — so both **fail** rather than skip
//! when a prerequisite is absent.
//!
//! ```text
//! env -u RUST_MIN_STACK cargo test --lib ttg::differential -- --ignored --nocapture
//! ```
//!
//! The Python oracle is embedded verbatim in `ORACLE_PY`, so the test is
//! self-contained: nothing outside this file has to survive for the next person
//! to re-verify the port. On divergence it prints every disagreeing row with
//! `file:line`, both triples and the source, which is enough to minimise a
//! reproducer by hand.
//!
//! # Why a differential and not a table of expected values
//!
//! A table pins whatever the Rust did on the day it was written, bugs included.
//! The oracle is an independent implementation in another language, so the two
//! agree only when both are right, or when both are wrong in the same way —
//! a far narrower target.
//!
//! # What the repository corpus cannot prove
//!
//! It contains **no genuinely nested block comment**. The 14 definitions whose
//! text matches `/*` inside `/*` are all glob patterns (`"**/…"`) inside string
//! literals, which the lexer never enters comment mode for. Breaking comment
//! nesting in the lexer therefore changes nothing on the real corpus. That
//! rule — and the lifetime-marker rule, and the `:`-does-not-close-a-run rule —
//! are covered only by `fuzz_cases`, which is why both differentials exist.

use super::{measure, measure_c_family};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

// ---------------------------------------------------------------------------
// The oracle
// ---------------------------------------------------------------------------

// The reference tokenizer, verbatim. Treat it as read-only: it is the answer
// key, and editing it to make a test pass is editing the answer.
const ORACLE_PY: &str = r##""""Rust token-tree scanner: reference implementation of the ATG base measures.

Produces, per definition:
  T    token count (comments, doc-comments and attribute spans excluded)
  D    decision points (Campbell-shaped, case-collapsed, boolean-run-collapsed)
  N    max control-flow nesting depth
All three are exactly invariant to line breaking and to comments.
"""
import re

PUNCT = [
 '<<=','>>=','...','..=','::','->','=>','==','!=','<=','>=','&&','||','..',
 '+=','-=','*=','/=','%=','^=','&=','|=','<<','>>','#!',
 '+','-','*','/','%','^','!','&','|','&&','=','<','>','@','_','.',',',';',':','#','$','?','~',
]
OPEN = {'(':')','[':']','{':'}'}
CLOSE = {')','(',']','[','}','{'}
IDENT = re.compile(r'[A-Za-z_][A-Za-z0-9_]*')
NUM = re.compile(r'[0-9][0-9A-Za-z_\.]*')

def lex(src):
    """-> list of (kind, text). kind in {id,lit,p,open,close}"""
    out=[]; i=0; n=len(src)
    while i < n:
        c = src[i]
        if c in ' \t\r\n': i+=1; continue
        # comments (rust block comments nest)
        if c=='/' and i+1<n and src[i+1]=='/':
            j=src.find('\n',i); i = n if j<0 else j+1; continue
        if c=='/' and i+1<n and src[i+1]=='*':
            depth=1; i+=2
            while i<n and depth>0:
                if src.startswith('/*',i): depth+=1; i+=2
                elif src.startswith('*/',i): depth-=1; i+=2
                else: i+=1
            continue
        # raw strings  r"..."  r#"..."#  br#"..."#
        m = re.match(r'(b?r)(#*)"', src[i:])
        if m:
            hashes = m.group(2); i += m.end()
            end = src.find('"'+hashes, i)
            i = n if end<0 else end+1+len(hashes)
            out.append(('lit','"')); continue
        # byte/normal string
        if c=='"' or (c=='b' and i+1<n and src[i+1]=='"'):
            i += 1 if c=='"' else 2
            while i<n:
                if src[i]=='\\': i+=2; continue
                if src[i]=='"': i+=1; break
                i+=1
            out.append(('lit','"')); continue
        # char literal vs lifetime
        if c=="'":
            m2 = re.match(r"'(\\.[^']*|[^\\'])'", src[i:])
            if m2:
                i += m2.end(); out.append(('lit',"'")); continue
            m3 = IDENT.match(src, i+1)
            if m3: i = m3.end(); out.append(('id',"'lt")); continue
            i+=1; continue
        m = IDENT.match(src, i)
        if m:
            out.append(('id', m.group(0))); i = m.end(); continue
        m = NUM.match(src, i)
        if m:
            out.append(('lit','0')); i = m.end(); continue
        if c in OPEN: out.append(('open',c)); i+=1; continue
        if c in (')',']','}'): out.append(('close',c)); i+=1; continue
        for p in PUNCT:
            if src.startswith(p, i):
                out.append(('p',p)); i += len(p); break
        else:
            i+=1
    return out

ENDS_EXPR_ID = {'self','Self','true','false','super','crate'}
CTRL = {'if','while','loop','match'}

def strip_attrs(toks):
    """drop `#[...]` / `#![...]` spans"""
    out=[]; i=0; n=len(toks)
    while i<n:
        if toks[i]==('p','#') or toks[i]==('p','#!'):
            j=i+1
            if j<n and toks[j]==('open','['):
                d=0
                while j<n:
                    if toks[j][0]=='open': d+=1
                    elif toks[j][0]=='close':
                        d-=1
                        if d==0: j+=1; break
                    j+=1
                i=j; continue
        out.append(toks[i]); i+=1
    return out

def ends_expression(t):
    k,x = t
    if k=='lit': return True
    if k=='close': return True
    if k=='id': return x not in ('return','else','in','mut','ref','move','await','as','where','impl','fn','let','const','static','match','if','while','for','loop','yield','break','continue',''"'lt'") or x in ENDS_EXPR_ID
    if k=='p': return x=='?'
    return False

def scan(src, rust=True):
    toks = strip_attrs(lex(src))
    T = len(toks)
    if not rust:
        D = 0
        for k,x in toks:
            if k=='id' and x in ('if','elif','while','for','case','catch','match','loop','switch'): D+=1
            elif k=='p' and x in ('&&','||'): D+=1
        return T, D, 0
    D=0; stack=[]; maxn=0; pending_ctrl=False; prev=None; depth=0
    runop={0:None}          # depth -> operator of the boolean run currently open at that depth
    stmt_head={0:[]}        # depth -> tokens since the last statement separator at that depth
    i=0; n=len(toks)
    while i<n:
        k,x = toks[i]
        if k=='id':
            if x in ('if','while','loop','match'):
                D+=1; pending_ctrl=True
            elif x=='for':
                nxt = toks[i+1] if i+1<n else ('','')
                if nxt!=('p','<') and 'impl' not in stmt_head.get(depth,[]):
                    D+=1; pending_ctrl=True
            elif x=='else':
                # `let PAT = EXPR else { .. }` is a decision; plain `else` is not
                if stmt_head.get(depth,[])[:1]==['let']: D+=1
                pending_ctrl=True
            elif x=='fn':
                pending_ctrl=False
            stmt_head.setdefault(depth,[]).append(x)
        elif k=='p':
            if x in ('&&','||'):
                isbool = not (x=='||' and not (prev and ends_expression(prev)))
                if isbool:
                    if runop.get(depth)!=x:
                        D+=1; runop[depth]=x
                else:
                    runop[depth]=None
            elif x in (';','=>',','):
                runop[depth]=None; stmt_head[depth]=[]
            elif x in ('=','?','.','::','->',':'):
                pass                      # inside one expression: the run survives
            else:
                runop[depth]=None
        elif k=='open':
            depth+=1; runop[depth]=None; stmt_head[depth]=[]
            if x=='{':
                stack.append(pending_ctrl)
                if pending_ctrl: maxn=max(maxn, sum(stack))
                pending_ctrl=False
        elif k=='close':
            runop.pop(depth,None); stmt_head.pop(depth,None); depth=max(0,depth-1)
            if x=='}':
                if stack: stack.pop()
                stmt_head[depth]=[]
                runop[depth]=None
        prev=toks[i]; i+=1
    return T, D, maxn
"##;

// Reads the length-prefixed corpus the Rust side writes and prints one
// `id\tT\tD\tN` row per case. Kept separate from the oracle so the oracle stays
// byte-identical to the reference.
const DRIVER_PY: &str = r#"
import sys, tok

def main():
    blob = open(sys.argv[1], 'rb').read()
    out = open(sys.argv[2], 'w')
    i = 0
    while i < len(blob):
        nl = blob.index(b'\n', i)
        ident, rust, ln = blob[i:nl].decode().split(' ')
        start = nl + 1
        src = blob[start:start + int(ln)].decode('utf-8')
        T, D, N = tok.scan(src, rust=(rust == '1'))
        out.write(f'{ident}\t{T}\t{D}\t{N}\n')
        i = start + int(ln) + 1
    out.close()

main()
"#;

// Dumps every indexed definition into the same length-prefixed corpus format.
// Python's stdlib sqlite3 does the read, so this test needs no database crate
// and compiles under every feature set.
const DUMP_PY: &str = r#"
import sqlite3, sys

def main():
    con = sqlite3.connect('file:' + sys.argv[1] + '?mode=ro', uri=True)
    out = open(sys.argv[2], 'wb')
    meta = open(sys.argv[3], 'w')
    n = 0
    for ident, path, name, line, lang, src in con.execute(
            'select id, file_path, function_name, start_line, language, source'
            ' from functions order by id'):
        raw = src.encode('utf-8')
        out.write(f'{ident} {1 if lang == "Rust" else 0} {len(raw)}\n'.encode())
        out.write(raw)
        out.write(b'\n')
        meta.write(f'{ident}\t{path}\t{name}\t{line}\t{lang}\n')
        n += 1
    out.close()
    meta.close()
    print(n)

main()
"#;

// ---------------------------------------------------------------------------
// Plumbing
// ---------------------------------------------------------------------------

/// One case fed to both implementations: an id, whether to use the Rust
/// decision walk, and the source.
struct Case {
    id: i64,
    rust: bool,
    src: String,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ttg-differential-{tag}"));
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn write(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).expect("write scratch file");
    p
}

/// Run `python3 <script> <args…>` in `dir`, returning stdout.
///
/// A missing interpreter is a hard failure, not a skip: this test exists to
/// catch divergence, and one that quietly passes when it never ran would be a
/// gate that cannot fail.
fn python(dir: &Path, script: &Path, args: &[&str]) -> String {
    let out = Command::new("python3")
        .current_dir(dir)
        .arg(script)
        .args(args)
        .output()
        .expect("python3 must be on PATH to run the TTG differential");
    assert!(
        out.status.success(),
        "oracle failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Write `cases` in the length-prefixed corpus format the driver reads.
fn write_corpus(path: &Path, cases: &[Case]) {
    let f = std::fs::File::create(path).expect("create corpus");
    let mut w = std::io::BufWriter::new(f);
    for c in cases {
        let raw = c.src.as_bytes();
        writeln!(w, "{} {} {}", c.id, u8::from(c.rust), raw.len()).expect("corpus header");
        w.write_all(raw).expect("corpus body");
        w.write_all(b"\n").expect("corpus terminator");
    }
    w.flush().expect("flush corpus");
}

/// `(id, T, D, N)` rows parsed from the oracle's TSV.
fn read_tsv(path: &Path) -> Vec<(i64, u32, u32, u32)> {
    let body = std::fs::read_to_string(path).expect("read oracle tsv");
    body.lines()
        .map(|l| {
            let mut f = l.split('\t');
            let mut next = |what: &str| -> String {
                f.next()
                    .unwrap_or_else(|| unreachable!("{what} column"))
                    .to_owned()
            };
            let id = next("id").parse().expect("id");
            let t = next("T").parse().expect("T");
            let d = next("D").parse().expect("D");
            let n = next("N").parse().expect("N");
            (id, t, d, n)
        })
        .collect()
}

/// Measure every case with the Rust implementation under test.
fn measure_all(cases: &[Case]) -> Vec<(i64, u32, u32, u32)> {
    cases
        .iter()
        .map(|c| {
            let m = if c.rust {
                measure(&c.src)
            } else {
                measure_c_family(&c.src)
            };
            (c.id, m.tokens, m.decisions, m.max_nesting)
        })
        .collect()
}

/// Ask the oracle for its measures of `cases`.
fn oracle_all(tag: &str, cases: &[Case]) -> Vec<(i64, u32, u32, u32)> {
    let dir = scratch(tag);
    write(&dir, "tok.py", ORACLE_PY);
    let driver = write(&dir, "driver.py", DRIVER_PY);
    let corpus = dir.join("corpus.bin");
    write_corpus(&corpus, cases);
    let tsv = dir.join("py.tsv");
    python(
        &dir,
        &driver,
        &[
            corpus.to_str().expect("corpus path"),
            tsv.to_str().expect("tsv path"),
        ],
    );
    read_tsv(&tsv)
}

/// One definition's measures: `(tokens, decisions, max_nesting)`.
type Triple = (u32, u32, u32);

/// A row as the oracle hands it over: the index id plus its three measures.
type OracleRow = (i64, u32, u32, u32);

/// A disagreement: the index id, what Python measured, what Rust measured.
type Disagreement = (i64, Triple, Triple);

/// Compare the two, returning the disagreeing rows.
fn disagreements(py: &[OracleRow], rs: &[OracleRow]) -> Vec<Disagreement> {
    assert_eq!(
        py.len(),
        rs.len(),
        "the two sides did not see the same population"
    );
    let mut out = Vec::new();
    for (p, r) in py.iter().zip(rs.iter()) {
        assert_eq!(p.0, r.0, "row order diverged at id {}", p.0);
        if (p.1, p.2, p.3) != (r.1, r.2, r.3) {
            out.push((p.0, (p.1, p.2, p.3), (r.1, r.2, r.3)));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// The adversarial generator
// ---------------------------------------------------------------------------

// Hand-written minimal cases, one per lexer or walk rule, including the ones
// the repository corpus never exercises. Each of these was checked to move at
// least one mutant; see the module docs.
const HAND: &[&str] = &[
    "",
    " ",
    "\n",
    "//",
    "/*",
    "/*/",
    "\"",
    "'",
    "r#\"",
    "r#",
    "b",
    "b'",
    "\\",
    // genuinely nested block comments — absent from the real corpus
    "/* /* */ */",
    "/* /* */",
    "/* /* /* deep */ */ */",
    "/** /** */ */",
    "/*! /*! */ */",
    "/* a /* b */ c */",
    "/* unterminated",
    // raw strings, hash counting, and keywords hidden inside them
    "r\"a\"#",
    "r##\"x\"##",
    "r#\"a\"#\"#",
    "br##\"q\"##",
    "r\"if a { for b in c {} }\"",
    "r#\"/* nested /* deeper */ still */\"#",
    "b\"if a\"",
    "\"if a && b\"",
    // char literals vs lifetimes
    "'a'",
    "'ab'",
    "'\\''",
    "'\\\\'",
    "'\\n'",
    "'_'",
    "'static",
    "'a",
    "''",
    "'\\u{7f}'",
    "'é'",
    "\"\\\\\"",
    "\"\\\"\"",
    "\"unterminated",
    // numeric lexing: the reference rule swallows dots and suffixes
    "0..10",
    "0.1.2",
    "1u8",
    "0x_",
    "1..=2",
    "3...4",
    // greedy punctuation
    "a<<=b",
    "a>>=b",
    "a#!b",
    "a#b",
    "r#type",
    "x@y",
    "$crate",
    "a~b",
    // attribute spans
    "#[a]",
    "#![a]",
    "#[a",
    "#a",
    "#[a[b]]",
    "#[]",
    "#[doc = \"if a && b\"]",
    // boolean runs
    "if a && b && c {}",
    "if a && b || c {}",
    "if a || b && c {}",
    "if (a && b) && c {}",
    "if a && (b && c) {}",
    "if a && b: c && d {}",
    // `||` as a zero-argument closure head
    "let x = || 1; let y = || 2;",
    "f(|| a, || b)",
    "a || || b",
    // else, let-else, guards
    "let Some(x) = y else { return };",
    "if x {} else {}",
    "if x {} else if y {}",
    "match x { a | b => 1, _ if g => 2, _ => 3 }",
    // for: loop vs HRTB vs impl-for
    "for<'a> fn(&'a u8)",
    "impl<T> Tr for T {}",
    "for x in y {}",
    // unbalanced delimiters — 0.75% of chunker output has them
    "}{",
    ")(",
    "][",
    "}}}}",
    "{{{{",
    "fn f() { if a {} }",
    "matches!(x, Some(_))",
    "x?.y?",
    "a as b",
    "a => b",
    "/* if a && b */",
    "/// if a && b\n",
];

// Fragments the generator concatenates. Chosen to attack greedy punctuation
// matching, literal boundaries, and the walk's statement-head tracking.
const FRAGMENTS: &[&str] = &[
    "<<=",
    ">>=",
    "...",
    "..=",
    "::",
    "->",
    "=>",
    "==",
    "!=",
    "<=",
    ">=",
    "&&",
    "||",
    "..",
    "+=",
    "-=",
    "*=",
    "/=",
    "%=",
    "^=",
    "&=",
    "|=",
    "<<",
    ">>",
    "#!",
    "+",
    "-",
    "*",
    "/",
    "%",
    "^",
    "!",
    "&",
    "|",
    "=",
    "<",
    ">",
    "@",
    "_",
    ".",
    ",",
    ";",
    ":",
    "#",
    "$",
    "?",
    "~",
    "(",
    ")",
    "[",
    "]",
    "{",
    "}",
    "(",
    ")",
    "{",
    "}",
    "if",
    "else",
    "while",
    "loop",
    "match",
    "for",
    "fn",
    "let",
    "impl",
    "in",
    "mut",
    "ref",
    "move",
    "await",
    "as",
    "where",
    "const",
    "static",
    "yield",
    "break",
    "continue",
    "return",
    "self",
    "Self",
    "true",
    "false",
    "super",
    "crate",
    "trait",
    "struct",
    "enum",
    "type",
    "pub",
    "use",
    "dyn",
    "unsafe",
    "foo",
    "bar",
    "x",
    "_z",
    "r",
    "b",
    "br",
    "r#type",
    "0",
    "1",
    "0..10",
    "0.5",
    "1_000u64",
    "0xFFu8",
    "3.",
    "0..=9",
    "\"s\"",
    "\"\"",
    "\"a\\\"b\"",
    "\"/*\"",
    "\"*/\"",
    "\"//\"",
    "\"if x { } else { }\"",
    "b\"bytes\"",
    "b'x'",
    "'a'",
    "'\\n'",
    "'\\u{1F600}'",
    "'a",
    "'static",
    "r\"raw\"",
    "r#\"ra\"w\"#",
    "r##\"a\"#b\"##",
    "br\"x\"",
    "br#\"y\"#",
    "r#\"\"#",
    "// line\n",
    "/// doc\n",
    "//! inner\n",
    "/* block */",
    "/** doc */",
    "/*! inner */",
    "/* /* nested */ */",
    "/* /* /* deep */ */ */",
    "/* unterminated",
    "#[derive(Debug)]",
    "#![allow(dead_code)]",
    "#[cfg(test)]",
    "#[a[b]c]",
    "#[",
    "é",
    "日本語",
    "\"héllo\"",
    "//é\n",
    " ",
    "\t",
    "\n",
    "\r\n",
    "\\",
];

// Templates that build nested, structurally Rust-shaped inputs, so the walk's
// depth stack and statement heads are exercised rather than just the lexer.
const TEMPLATES: &[&str] = &[
    "fn f() { @ }",
    "if @ { }",
    "match x { @ => 1, _ => 2 }",
    "let a = @ else { return; };",
    "for x in @ { }",
    "impl Tr for Ty { @ }",
    "for<@> fn()",
    "while let Some(@) = it.next() { }",
    "|| @",
    "a || @",
    "f(|| @)",
    "x && @ && y || z",
    "closure(|@| @)",
    "loop { @ }",
    "if let Some(@) = o { } else { }",
];

/// A tiny reproducible PRNG. Deliberately not a dependency: the point is that
/// a seed in a failure message regenerates the exact corpus that failed.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // SplitMix64.
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % (n as u64)) as usize
    }

    fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str {
        xs[self.below(xs.len())]
    }
}

fn soup(rng: &mut Rng, n: usize) -> String {
    let mut s = String::new();
    for _ in 0..n {
        s.push_str(rng.pick(FRAGMENTS));
        s.push_str(rng.pick(&["", " ", "\n", "  "]));
    }
    s
}

fn structured(rng: &mut Rng, depth: u32) -> String {
    if depth > 3 || rng.below(10) < 3 {
        let k = 1 + rng.below(6);
        return soup(rng, k);
    }
    let body = structured(rng, depth + 1);
    rng.pick(TEMPLATES).replace('@', &body)
}

/// The adversarial corpus: every hand-written case, then `n` generated pairs.
///
/// Alternate cases go through the C-family walk so both decision rules are
/// differentiated, not just the Rust one.
fn fuzz_cases(seed: u64, n: usize) -> Vec<Case> {
    let mut rng = Rng(seed);
    let mut out: Vec<Case> = HAND
        .iter()
        .map(|s| Case {
            id: 0,
            rust: true,
            src: (*s).to_string(),
        })
        .collect();
    for _ in 0..n {
        let k = 1 + rng.below(40);
        out.push(Case {
            id: 0,
            rust: true,
            src: soup(&mut rng, k),
        });
        out.push(Case {
            id: 0,
            rust: true,
            src: structured(&mut rng, 0),
        });
    }
    for (i, c) in out.iter_mut().enumerate() {
        c.id = i as i64;
        c.rust = i % 2 == 0;
    }
    out
}

// ---------------------------------------------------------------------------
// The differentials
// ---------------------------------------------------------------------------

/// Every definition in `.pmat/context.db`, Rust against the oracle.
///
/// Run with:
///
/// ```text
/// env -u RUST_MIN_STACK cargo test --lib \
///     ttg::differential::rust_measures_match_the_oracle_on_every_indexed_definition \
///     -- --ignored --nocapture
/// ```
#[test]
#[ignore = "needs python3 and a built .pmat/context.db; see the module docs"]
fn rust_measures_match_the_oracle_on_every_indexed_definition() {
    let db = repo_root().join(".pmat/context.db");
    assert!(
        db.exists(),
        "no index at {}. Build one with `pmat query x --limit 1`, \
         then re-run. This test fails rather than skips on purpose: a \
         differential that passes without running is not a gate.",
        db.display()
    );

    let dir = scratch("corpus");
    write(&dir, "tok.py", ORACLE_PY);
    let dump = write(&dir, "dump.py", DUMP_PY);
    let corpus = dir.join("corpus.bin");
    let meta = dir.join("meta.tsv");
    let n: usize = python(
        &dir,
        &dump,
        &[
            db.to_str().expect("db path"),
            corpus.to_str().expect("corpus path"),
            meta.to_str().expect("meta path"),
        ],
    )
    .trim()
    .parse()
    .expect("row count");

    // Read back exactly the bytes the oracle will see, so neither side can be
    // measuring a different population from the other.
    let cases = read_corpus(&corpus);
    assert_eq!(cases.len(), n, "corpus round-trip lost rows");
    assert!(n > 1000, "index looks empty or truncated: {n} rows");

    let driver = write(&dir, "driver.py", DRIVER_PY);
    let tsv = dir.join("py.tsv");
    python(
        &dir,
        &driver,
        &[
            corpus.to_str().expect("corpus path"),
            tsv.to_str().expect("tsv path"),
        ],
    );

    let py = read_tsv(&tsv);
    let rs = measure_all(&cases);
    let bad = disagreements(&py, &rs);

    let names = std::fs::read_to_string(&meta).expect("read meta");
    let names: Vec<&str> = names.lines().collect();
    for (id, p, r) in bad.iter().take(40) {
        let row = names.get(*id as usize - 1).copied().unwrap_or("<unknown>");
        eprintln!("  id={id} {row}\n     oracle(T,D,N)={p:?}  rust={r:?}");
    }
    eprintln!(
        "rows compared {n}, agreeing {} ({:.4}%)",
        n - bad.len(),
        100.0 * (n - bad.len()) as f64 / n as f64
    );
    assert!(
        bad.is_empty(),
        "{} rows disagree with the oracle",
        bad.len()
    );
}

/// The generated adversarial corpus, Rust against the oracle.
///
/// This is the half that covers what the repository cannot: nested block
/// comments, unterminated literals, raw-string hash counting, unbalanced
/// delimiters and greedy punctuation boundaries.
///
/// ```text
/// env -u RUST_MIN_STACK cargo test --lib \
///     ttg::differential::rust_measures_match_the_oracle_on_generated_adversarial_input \
///     -- --ignored --nocapture
/// ```
#[test]
#[ignore = "needs python3 on PATH; see the module docs"]
fn rust_measures_match_the_oracle_on_generated_adversarial_input() {
    let mut total = 0usize;
    let mut failures = Vec::new();
    for seed in 1..=8u64 {
        let cases = fuzz_cases(seed, 4000);
        total += cases.len();
        let py = oracle_all("fuzz", &cases);
        let rs = measure_all(&cases);
        for (id, p, r) in disagreements(&py, &rs) {
            let src = &cases[id as usize].src;
            eprintln!(
                "  seed={seed} case={id} rust_walk={}\n     oracle={p:?} rust={r:?}\n     src={src:?}",
                cases[id as usize].rust
            );
            failures.push(seed);
        }
    }
    eprintln!("adversarial cases compared: {total}");
    assert!(
        failures.is_empty(),
        "{} generated cases disagree with the oracle",
        failures.len()
    );
}

/// Parse the length-prefixed corpus back into cases.
fn read_corpus(path: &Path) -> Vec<Case> {
    let blob = std::fs::read(path).expect("read corpus");
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < blob.len() {
        let nl = i + blob[i..]
            .iter()
            .position(|&c| c == b'\n')
            .unwrap_or_else(|| unreachable!("corpus header has no newline"));
        let hdr = std::str::from_utf8(&blob[i..nl]).expect("header utf8");
        let mut f = hdr.split(' ');
        let mut next = |what: &str| -> String {
            f.next()
                .unwrap_or_else(|| unreachable!("{what} field"))
                .to_owned()
        };
        let id: i64 = next("id").parse().expect("id");
        let rust: u8 = next("rust").parse().expect("rust flag");
        let len: usize = next("len").parse().expect("len");
        let start = nl + 1;
        let src = std::str::from_utf8(&blob[start..start + len]).expect("source utf8");
        out.push(Case {
            id,
            rust: rust == 1,
            src: src.to_owned(),
        });
        i = start + len + 1;
    }
    out
}

// ---------------------------------------------------------------------------
// Always-on regressions
// ---------------------------------------------------------------------------
//
// The two differentials above are `#[ignore]`d, so on a normal `cargo test`
// run nothing here would execute. These pin the specific behaviours the
// mutation battery showed the differentials catch, using no oracle and no
// database, so an ordinary test run still goes red if the port regresses.

#[cfg(test)]
mod always_on {
    use super::super::{measure, measure_c_family};

    /// A run of like boolean operators charges once, however it is wrapped —
    /// the whole reason TTG exists. Mutating `on_bool_op` to charge every
    /// operator moves this and 473 real definitions.
    #[test]
    fn a_boolean_run_charges_once_and_survives_reformatting() {
        // No `if`, so the run is the only thing that can charge: three `&&`
        // tokens, one decision.
        assert_eq!(measure("fn f() { let z = a && b && c && d; }").decisions, 1);
        // The same expression wrapped over five lines. This equality is the
        // whole point of TTG: the incumbent line scanner reports 1 and 3.
        assert_eq!(
            measure("fn f() {\n let z = a\n && b\n && c\n && d;\n}").decisions,
            1
        );
        // Counter-test bounding the over-correction: collapsing is per run,
        // not per function. A `;` closes the run, so two runs charge twice.
        assert_eq!(
            measure("fn f() { let z = a && b; let y = c && d; }").decisions,
            2
        );
        // A change of operator opens a new run: `&&` run, `||` run, plus `if`.
        assert_eq!(measure("fn f() { if a && b || c {} }").decisions, 3);
        // A deeper delimiter closes the run, so the inner run charges again.
        assert_eq!(measure("fn f() { if a && (b && c) {} }").decisions, 3);
        // …and the `if` itself is charged on top of its condition's run.
        assert_eq!(measure("fn f() { if a && b && c && d {} }").decisions, 2);
    }

    /// `match` charges once for the dispatch; arms charge nothing. This is
    /// what takes `classify_command` from `cc = 73` to `D = 1`.
    #[test]
    fn match_charges_once_for_the_dispatch_not_once_per_arm() {
        let m =
            measure(r#"fn f(s: &str) -> u8 { match s { "a" => 1, "b" => 2, "c" => 3, _ => 0 } }"#);
        assert_eq!(m.decisions, 1);
        // A guard is an `if`, and is charged.
        assert_eq!(
            measure("fn f() { match x { a if g => 1, _ => 2 } }").decisions,
            2
        );
        // Pattern alternation is not a decision.
        assert_eq!(
            measure("fn f() { match x { a | b | c => 1 } }").decisions,
            1
        );
    }

    /// Control-flow keywords inside a string literal are invisible: a literal
    /// is exactly one token whatever it contains. This is the
    /// `generate_trigram_index` false positive the incumbent scores `cc = 12`.
    #[test]
    fn keywords_inside_literals_and_comments_do_not_charge() {
        assert_eq!(
            measure(r##"fn f() { let s = r#"if a && b { for c in d {} }"#; }"##).decisions,
            0
        );
        assert_eq!(measure("fn f() { let s = \"if a && b\"; }").decisions, 0);
        assert_eq!(measure("fn f() { /* if a && b */ }").decisions, 0);
        assert_eq!(measure("fn f() { /// if a && b\n }").decisions, 0);
        // …and the string is one token, not its contents.
        assert_eq!(measure(r#""if a && b for c""#).tokens, 1);
    }

    /// Block comments nest. The repository corpus contains no example, so
    /// without this the rule is unverified by anything that runs by default.
    #[test]
    fn block_comments_nest_and_are_untokenised() {
        assert_eq!(measure("/* /* */ */").tokens, 0);
        assert_eq!(measure("/* /* /* deep */ */ */").tokens, 0);
        assert_eq!(measure("/* /* */ */ x").tokens, 1);
        // The naive non-nesting scan would stop at the first `*/` and leave
        // `*/ x` behind as three tokens.
        assert_eq!(measure("/* a /* b */ c */ x").tokens, 1);
    }

    /// Plain `else` is free; `let … else` is a divergence and charges.
    #[test]
    fn only_let_else_charges() {
        assert_eq!(measure("fn f() { if a {} else {} }").decisions, 1);
        assert_eq!(measure("fn f() { if a {} else if b {} }").decisions, 2);
        assert_eq!(
            measure("fn f() { let Some(x) = y else { return; }; }").decisions,
            1
        );
    }

    /// `||` after something that cannot end an expression is a zero-argument
    /// closure, not boolean-or.
    #[test]
    fn a_zero_argument_closure_head_is_not_a_boolean_operator() {
        assert_eq!(measure("fn f() { let x = || 1; }").decisions, 0);
        assert_eq!(measure("fn f() { g(|| a, || b); }").decisions, 0);
        assert_eq!(measure("fn f() { if a || b {} }").decisions, 2);
    }

    /// `for` is a loop unless it is an HRTB or an `impl … for …` header.
    #[test]
    fn for_is_not_charged_in_hrtb_or_impl_headers() {
        assert_eq!(measure("fn f() { for x in y {} }").decisions, 1);
        assert_eq!(measure("fn f(g: for<'a> fn(&'a u8)) {}").decisions, 0);
        assert_eq!(measure("impl<T> Tr for T {}").decisions, 0);
    }

    /// Attribute spans are removed, so documentation and lint control are
    /// untaxed. Leaving them in moves 632 real definitions.
    #[test]
    fn attribute_spans_are_removed() {
        // Attributed and bare read identically: `struct`, `S`, `;`.
        assert_eq!(measure("struct S;").tokens, 3);
        assert_eq!(measure("#[derive(Debug)] struct S;").tokens, 3);
        assert_eq!(
            measure("#[derive(Debug)]\n#[cfg(test)]\nstruct S;").tokens,
            3
        );
        assert_eq!(measure("#[doc = \"if a && b\"] fn f() {}").decisions, 0);
        // A `#` not followed by `[` is kept, which is what lets a raw
        // identifier survive as ordinary tokens.
        assert!(measure("r#type").tokens > 0);
    }

    /// The reference numeric rule swallows dots and suffixes, so `0..10` is
    /// one token. Faithfulness to the oracle beats tidiness; a "corrected"
    /// lexer here disagrees with the oracle on 2,623 real definitions.
    #[test]
    fn the_numeric_rule_swallows_dots_and_suffixes() {
        assert_eq!(measure("0..10").tokens, 1);
        assert_eq!(measure("1_000u64").tokens, 1);
        assert_eq!(measure("0.1.2").tokens, 1);
        // It only starts at a digit, so a leading `.` still lexes as punctuation.
        assert_eq!(measure("..10").tokens, 2);
    }

    /// Unbalanced delimiters must not hang or lose the row: 0.75% of chunker
    /// output has them.
    #[test]
    fn malformed_input_terminates_and_yields_measures() {
        for s in [
            "}{",
            ")(",
            "][",
            "{{{{",
            "}}}}",
            "/* unterminated",
            "\"unterminated",
            "r#\"",
        ] {
            let _ = measure(s);
            let _ = measure_c_family(s);
        }
    }

    /// The C-family walk charges every branching keyword and every
    /// short-circuit operator, with no run collapsing.
    #[test]
    fn the_c_family_walk_does_not_collapse_runs() {
        assert_eq!(measure_c_family("if (a && b && c) {}").decisions, 3);
        assert_eq!(
            measure_c_family("switch (x) { case 1: break; }").decisions,
            2
        );
        // Same input, Rust rules: one `if`, one collapsed run.
        assert_eq!(measure("if a && b && c {}").decisions, 2);
    }
}
