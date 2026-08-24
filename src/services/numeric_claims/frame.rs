//! Stage 1 — measurement framing: which numerals are claims about this
//! repository, now.
//!
//! Most numbers in a tree are operands. R1 reads 3.0% of pmat's numeric
//! literals and R2 reads 1.6%; the other ~95% are array bounds, ports, HTTP
//! statuses, powers of two and loop counters, and reading them is how a
//! statistical check drowns. A numeral survives Stage 1 only if it is
//! *presented as a fact about this repository, now*:
//!
//! ```text
//! count   a repo-artifact noun within two words after it   "78 workspace crates"
//! assert  an assertive marker nearby                       "Total: 657 preconditions"
//! ```
//!
//! and only if the line is not one of the shapes where two files legitimately
//! disagree:
//!
//! ```text
//! anti-frame  should must target goal require "at least" threshold budget estimat
//!             -- policy is not measurement. Two targets that differ are two
//!                teams' intentions, not two answers to one question.
//! duration    "takes 30 minutes"          -- a wall-clock cost, not a count
//! table row   "| tests | 1904 |"          -- a data table, read column-wise
//! heading     "### 3.1 Coverage"          -- the numeral is an outline position
//! range       "(0.0 - 1.0)"               -- domain bounds
//! past state  "Before: 570", "reduced from" -- a record, correctly stale
//! code        "// let n = compute(3);"    -- commented-out source
//! ```
//!
//! ## Dates and versions are masked POSITIONALLY, never line-wide
//!
//! This is not a refinement, it is the difference between finding the flagship
//! defect and missing it. aprender's README line carries both the crate count
//! and the release version; dropping any line that contains a version silently
//! ate the one true positive the whole check exists for. So date, version,
//! arXiv and citation spans are overwritten in place — same byte length, so
//! every offset downstream still points where it did — and the rest of the line
//! is read normally.
//!
//! Everything here is pure. No file is opened, no clock is read.

use regex::Regex;
use std::sync::LazyLock;

/// Which frame admitted a numeral.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    /// A repo-artifact noun follows within two words: "78 workspace crates".
    Count,
    /// An assertive marker sits nearby: "Total:", "currently", "there are".
    Assert,
}

/// Why a line was dropped before any numeral on it was read.
///
/// Each variant is a shape where two files can differ without either being
/// wrong, which is exactly what R1 must not report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dropped {
    /// A markdown table row: data read column-wise, not a sentence.
    TableRow,
    /// A duration. A wall-clock cost is not a count of anything in the tree.
    Duration,
    /// A record of past state — correctly stale, by construction.
    PastState,
    /// Commented-out source: the numerals are operands.
    Code,
    /// A section heading, where the numeral is an outline position.
    SectionHeading,
    /// A range endpoint or domain bound.
    RangeEndpoint,
    /// Policy, aspiration or budget. Not a measurement.
    AntiFrame,
}

impl Dropped {
    /// A short, stable label for the census.
    pub fn label(self) -> &'static str {
        match self {
            Self::TableRow => "table-row",
            Self::Duration => "duration",
            Self::PastState => "past-state",
            Self::Code => "code",
            Self::SectionHeading => "section-heading",
            Self::RangeEndpoint => "range-endpoint",
            Self::AntiFrame => "anti-frame",
        }
    }
}

/// A numeral presented as a fact about this repository, now.
#[derive(Debug, Clone, PartialEq)]
pub struct FramedNumeral {
    /// Repo-relative path.
    pub file: String,
    /// 1-indexed line number.
    pub line: usize,
    /// The numeral exactly as written, separators and all: `1,291`.
    pub token: String,
    /// Its value.
    pub value: f64,
    /// Which frame admitted it.
    pub frame: Frame,
    /// The trimmed source line, truncated to [`CONTEXT_CAP`] characters.
    pub context: String,
}

/// Longest stored context, in characters. The context becomes the cohort key,
/// so it is bounded to keep one runaway line from becoming its own template.
pub const CONTEXT_CAP: usize = 160;

/// Byte span of one numeral inside a line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumeralSpan {
    /// Inclusive byte offset of the first digit.
    pub start: usize,
    /// Exclusive byte offset just past the last digit.
    pub end: usize,
}

/// Bytes above this are not read as a quantity: a 13-digit number in prose is
/// an identifier, a timestamp or a hash fragment.
const MAX_QUANTITY: f64 = 1e12;

/// Characters that make an adjacent digit part of something else — an
/// identifier, a version, a path, a date. `sha256`, `v0.34.0`, `a/2/b`, `id_42`
/// all die here, which is why nothing downstream needs a special case for them.
fn is_glue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'-' | b'/')
}

/// Every numeral in `s`, in order.
///
/// Recognises the three written forms, in this precedence: comma-grouped
/// (`1,291`), underscore-grouped (`50_000_000`), plain or decimal (`70`,
/// `0.60`). A candidate is rejected outright if it is glued to an identifier
/// character on either side.
/// Advance past a run of token-glue bytes.
///
/// Extracted because the same three-line loop appeared twice in
/// `scan_numerals`, and the duplication put its cognitive complexity at 27
/// against `pmat verify`'s limit of 25 — measured, not estimated.
fn skip_glue(b: &[u8], from: usize) -> usize {
    let mut i = from;
    while i < b.len() && is_glue(b[i]) {
        i += 1;
    }
    i
}

pub fn scan_numerals(s: &str) -> Vec<NumeralSpan> {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        if !b[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        // Mid-token digits (`utf8`, `sha256`) are not numerals: skip the whole
        // glue run rather than restarting inside it.
        if i > 0 && is_glue(b[i - 1]) {
            i = skip_glue(b, i);
            continue;
        }
        match numeral_end(b, i) {
            Some(end) => {
                out.push(NumeralSpan { start: i, end });
                i = end;
            }
            None => i = skip_glue(b, i),
        }
    }
    out
}

/// The end of the numeral starting at `i`, or `None` if no written form ends on
/// a boundary there.
fn numeral_end(b: &[u8], i: usize) -> Option<usize> {
    candidate_ends(b, i)
        .into_iter()
        .find(|&e| e < b.len() && !is_glue(b[e]) || e == b.len())
}

/// End offsets of a comma-grouped numeral (`1,234,567`), longest first.
///
/// A group is exactly `,ddd` NOT followed by a fourth digit — `1,2345` is not
/// grouped notation, and accepting it would let a version string or an ID
/// masquerade as a count.
fn comma_group_ends(b: &[u8], run: usize) -> Vec<usize> {
    let mut p = run;
    let mut groups = Vec::new();
    while p + 4 <= b.len()
        && b[p] == b','
        && b[p + 1..p + 4].iter().all(u8::is_ascii_digit)
        && !(p + 4 < b.len() && b[p + 4].is_ascii_digit())
    {
        p += 4;
        groups.push(p);
    }
    groups.reverse();
    groups
}

/// End offsets of an underscore-grouped numeral (`50_000_000`), longest first.
///
/// Rust's own literal spelling, and the one that makes `.pmat-metrics.toml`'s
/// `binary_max_bytes = 50_000_000` comparable with `binary_size.rs`'s
/// `50 * 1024 * 1024`.
fn underscore_group_ends(b: &[u8], run: usize, digits_end: &dyn Fn(usize) -> usize) -> Vec<usize> {
    let mut p = run;
    let mut groups = Vec::new();
    while p < b.len() && b[p] == b'_' && p + 1 < b.len() && b[p + 1].is_ascii_digit() {
        p = digits_end(p + 1);
        groups.push(p);
    }
    groups.reverse();
    groups
}

/// Candidate end offsets, in the order the written forms are tried.
fn candidate_ends(b: &[u8], i: usize) -> Vec<usize> {
    let digits_end = |from: usize| {
        let mut j = from;
        while j < b.len() && b[j].is_ascii_digit() {
            j += 1;
        }
        j
    };
    let run = digits_end(i);
    let mut ends = Vec::new();

    // comma-grouped: 1..=3 leading digits, then one or more `,ddd`
    if (1..=3).any(|lead| i + lead == run) {
        ends.extend(comma_group_ends(b, run));
    }

    // underscore-grouped: digits, then one or more `_digits`
    ends.extend(underscore_group_ends(b, run, &digits_end));

    // plain or decimal, then the greedy-digit backtrack the alternation implies
    if run < b.len() && b[run] == b'.' && run + 1 < b.len() && b[run + 1].is_ascii_digit() {
        ends.push(digits_end(run + 1));
    }
    ends.extend((i + 1..=run).rev());
    ends
}

/// Numeral tokens in `s`, as written.
pub fn numeral_tokens(s: &str) -> Vec<String> {
    scan_numerals(s)
        .into_iter()
        .map(|sp| s[sp.start..sp.end].to_string())
        .collect()
}

/// Parse a numeral token, discarding the group separators it was written with.
pub fn parse_numeral(tok: &str) -> Option<f64> {
    tok.replace([',', '_'], "").parse::<f64>().ok()
}

static ASSERT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\b(current(?:ly)?|actual(?:ly)?|measured?|as of|today|now|baseline|",
        r"we (?:have|had|now)|there (?:are|were)|contains?|totals?|counts?|reports?|shows?|",
        r"stands? at|sits? at|coverage|passing|passes|found|detected|remaining|achieved)\b",
    ))
    .expect("ASSERT is a compile-time constant pattern")
});

static COUNT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)^\s*\+?\s*(?:[a-z_-]+\s+){0,2}(crates?|tests?|files?|lines?|functions?|",
        r"modules?|commands?|subcommands?|chapters?|contracts?|checks?|rules?|warnings?|errors?|",
        r"violations?|failures?|defects?|issues?|todos?|clones?|tools?|templates?|handlers?|",
        r"structs?|traits?|packages?|dependencies|deps|assertions?|obligations?|gates?|hooks?)\b",
    ))
    .expect("COUNT is a compile-time constant pattern")
});

static ANTI_FRAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\b(should|must|shall|target|goal|aim|plan|will|would|expect|require[sd]?|",
        r"at least|at most|no more than|up to|limit|threshold|budget)\b|\bestimat|",
        r"[<>≤≥]=?\s*$|[<>≤≥]=?\s*\d",
    ))
    .expect("ANTI_FRAME is a compile-time constant pattern")
});

static DURATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(hours?|minutes?|days?|weeks?|months?|years?|seconds?|secs?|ms|hrs?|sprint)\b",
    )
    .expect("DURATION is a compile-time constant pattern")
});

static SECTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^#{1,6}\s*\d+(\.\d+)*\.?\s|^\**\d+(\.\d+)+\.?\s+[A-Z]|^\s*\d+(\.\d+)+\s")
        .expect("SECTION is a compile-time constant pattern")
});

static RANGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\d\s*(?:-|–|to|and|\.\.=?)\s*\d[\d.]*\s*[\)\]]|",
        r"between\s+[\d.]+\s+and\s+[\d.]+|",
        r"\bin\s*[\[\(][\d.]+\s*,\s*[\d.]+[\]\)]|[\[\(][\d.]+\s*(?:-|,|to)\s*[\d.]+[\]\)]",
    ))
    .expect("RANGE is a compile-time constant pattern")
});

static PAST_STATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?i)\b(before|after|previously|was|were|used to|reduced from|down from|up from|",
        r"increased from|historical|legacy|deprecated|old|initial|originally|v\d+\.\d+)\b[:\s]",
    ))
    .expect("PAST_STATE is a compile-time constant pattern")
});

static CODE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"(?:vec!|=>|\{\s*\w+\s*:|\w+\(\s*\d|\)\s*[;,]|::\w+|\|\||&&|or_insert|",
        r"unwrap\(|assert|\bfn\s|\blet\s|\bR\^|\$)",
    ))
    .expect("CODE is a compile-time constant pattern")
});

/// Spans overwritten before any numeral is read: a date, a semantic version, an
/// arXiv id, a bracketed citation, a month-and-day. Their digits are
/// coordinates, not quantities.
static MASKED: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    [
        r"\b(19|20)\d\d[-/]\d{1,2}[-/]\d{1,2}|\d{1,2}[-/]\d{1,2}[-/](19|20)\d\d",
        r"\bv?\d+\.\d+\.\d+",
        r"(?i)arxiv[:\s]*\d{4}\.\d{4,5}",
        r"\[\d{1,3}\]|\(\d{4}[a-z]?\)",
        r"(?i)\b(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]*\.?\s+\d{1,2}\b",
    ]
    .iter()
    .map(|p| Regex::new(p).expect("MASKED entries are compile-time constant patterns"))
    .collect()
});

/// Filler byte for a masked span. Not a digit and not glue, so it neither
/// creates nor joins a numeral.
const MASK_BYTE: u8 = 0;

/// Lines of `text` that can carry a prose claim.
///
/// `*.rs` and `*.sh` contribute **comments only**: a numeric literal in an
/// expression is an operand. Markdown skips fenced code for the same reason.
/// Everything else keeps lines that look like a key or a comment.
pub fn scan_lines<'a>(path: &str, text: &'a str) -> Vec<(usize, &'a str)> {
    let ext = path.rsplit('.').next().unwrap_or("");
    let numbered = text.lines().enumerate().map(|(i, l)| (i + 1, l));
    match ext {
        "md" | "markdown" => {
            let mut fenced = false;
            let mut out = Vec::new();
            for (i, l) in numbered {
                if l.trim_start().starts_with("```") {
                    fenced = !fenced;
                    continue;
                }
                if !fenced {
                    out.push((i, l));
                }
            }
            out
        }
        "rs" | "sh" => numbered
            .filter(|(_, l)| {
                let t = l.trim_start();
                t.starts_with("//") || t.starts_with('*') || t.starts_with("/*")
            })
            .collect(),
        _ => numbered
            .filter(|(_, l)| l.contains('#') || l.contains('=') || l.contains(':'))
            .collect(),
    }
}

/// Is this line one of the shapes where two files legitimately disagree?
///
/// Order is meaningful only for the reason reported, never for the verdict: a
/// line matching two of these is dropped either way.
pub fn structural_drop(line: &str) -> Option<Dropped> {
    if line.trim_start().starts_with('|') {
        return Some(Dropped::TableRow);
    }
    if DURATION.is_match(line) {
        return Some(Dropped::Duration);
    }
    if PAST_STATE.is_match(line) {
        return Some(Dropped::PastState);
    }
    if CODE.is_match(line) {
        return Some(Dropped::Code);
    }
    if SECTION.is_match(line.trim()) {
        return Some(Dropped::SectionHeading);
    }
    if RANGE.is_match(line) {
        return Some(Dropped::RangeEndpoint);
    }
    if ANTI_FRAME.is_match(line) {
        return Some(Dropped::AntiFrame);
    }
    None
}

/// Overwrite date, version, arXiv and citation spans in place.
///
/// Byte length is preserved so every offset computed on the masked string still
/// indexes the original line — that is what makes the masking positional rather
/// than line-wide.
pub fn mask_spans(line: &str) -> String {
    let mut bytes = line.as_bytes().to_vec();
    for rx in MASKED.iter() {
        for m in rx.find_iter(line) {
            bytes[m.start()..m.end()].fill(MASK_BYTE);
        }
    }
    String::from_utf8(bytes).unwrap_or_else(|_| line.to_string())
}

/// Widen `at` down to a character boundary of `s`.
fn floor_boundary(s: &str, at: usize) -> usize {
    let mut i = at.min(s.len());
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

/// Widen `at` up to a character boundary of `s`.
fn ceil_boundary(s: &str, at: usize) -> usize {
    let mut i = at.min(s.len());
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

/// How much of the line either side of a numeral is read for its frame.
const CONTEXT_WINDOW: usize = 80;

/// How much of the trailing context an assertive marker may sit in. Wider than
/// this and the marker is describing something else on the line.
const ASSERT_WINDOW: usize = 40;

/// Framed numerals on one line.
pub fn frame_line(file: &str, lineno: usize, raw: &str) -> Vec<FramedNumeral> {
    // A line with no digit cannot produce a numeral, so none of the seven
    // structural patterns below can change its verdict. Most lines in a source
    // tree are this line, and skipping them here is worth ~4x on the whole
    // scan. The equivalence is exact, not an approximation.
    if !raw.bytes().any(|b| b.is_ascii_digit()) {
        return Vec::new();
    }
    if structural_drop(raw).is_some() {
        return Vec::new();
    }
    let trimmed = raw.trim();
    let context: String = trimmed.chars().take(CONTEXT_CAP).collect();
    let masked = mask_spans(raw);

    scan_numerals(&masked)
        .into_iter()
        .filter_map(|sp| {
            let token = raw.get(sp.start..sp.end)?;
            let value = parse_numeral(token)?;
            if value > MAX_QUANTITY || is_year(value) {
                return None;
            }
            let pre = &raw[floor_boundary(raw, sp.start.saturating_sub(CONTEXT_WINDOW))..sp.start];
            let post = &raw[sp.end..ceil_boundary(raw, sp.end + CONTEXT_WINDOW)];
            let frame = frame_of(pre, post)?;
            Some(FramedNumeral {
                file: file.to_string(),
                line: lineno,
                token: token.to_string(),
                value,
                frame,
                context: context.clone(),
            })
        })
        .collect()
}

/// A year is a coordinate, not a count of anything in the tree.
fn is_year(v: f64) -> bool {
    v.fract() == 0.0 && (1900.0..=2100.0).contains(&v)
}

/// Which frame, if either, admits a numeral with this context around it.
fn frame_of(pre: &str, post: &str) -> Option<Frame> {
    if COUNT.is_match(post) {
        return Some(Frame::Count);
    }
    let near = &post[..ceil_boundary(post, ASSERT_WINDOW)];
    if ASSERT.is_match(pre) || ASSERT.is_match(near) {
        return Some(Frame::Assert);
    }
    None
}

/// Framed numerals in one file.
pub fn frame_file(path: &str, text: &str) -> Vec<FramedNumeral> {
    scan_lines(path, text)
        .into_iter()
        .flat_map(|(i, l)| frame_line(path, i, l))
        .collect()
}
