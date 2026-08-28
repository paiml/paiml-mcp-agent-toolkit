//! R2 stage 1 — turn bytes into `(quantity, value, annotation)` triples.
//!
//! Three decisions here are load-bearing, and each was measured rather than
//! chosen:
//!
//! 1. **Values are evaluated, not read.** `50_000_000`, `1,234`, `0xFFFF`,
//!    `1e6`, `50 * 1024 * 1024` and `1 << 20` all become a number. Without the
//!    product rule the canonical `binary_size.rs` contradiction is invisible:
//!    read literally, `50 * 1024 * 1024` is the numeral `50`.
//! 2. **Units normalise to an ambiguity set, not a choice.** `MB` is 10^6 to a
//!    disk vendor and 2^20 to a linker, and one repository writes both. A rule
//!    may fire only when *every* reading disagrees, so
//!    `binary_max_bytes = 50_000_000  # 50 MB` correctly stays silent.
//! 3. **TOML and YAML keys are section-qualified.** Unqualified,
//!    `.pmat-ratchet.toml`'s six `[metric.*].baseline` keys collapse into one
//!    cluster that contradicts itself by construction.
//!
//! Nothing in this module opens a path or runs a subprocess.

use regex::Regex;
use std::sync::LazyLock;

use super::CorpusFile;

/// Relative tolerance for "these two numbers are the same number".
pub const TOL: f64 = 0.02;

/// A scalar literal: decimal, hex, grouped, or exponent-form.
pub const NUM: &str = r"(?:0[xX][0-9a-fA-F_]+|\d[\d_,]*\.?\d*(?:[eE][-+]?\d+)?)";

fn rx(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex must compile")
}

/// The physical dimension a quantity is measured in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dim {
    /// Durations, canonical unit milliseconds.
    Time,
    /// Sizes, canonical unit bytes.
    Bytes,
    /// Percentages.
    Pct,
    /// Dimensionless counts.
    Count,
}

impl Dim {
    /// The unit a bare number in this dimension is assumed to carry.
    pub fn canonical_unit(self) -> &'static str {
        match self {
            Dim::Time => "ms",
            Dim::Bytes => "b",
            Dim::Pct => "%",
            Dim::Count => "n",
        }
    }
}

/// Whether a key names a ceiling or a floor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    /// `max_*`, `*_limit`, `threshold`, `budget`, `cap`.
    Max,
    /// `min_*`, `floor`, `least`.
    Min,
}

/// Which grammar a file was read with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    /// `*.toml`
    Toml,
    /// `*.yaml`, `*.yml`, `*.json`
    Yaml,
    /// `*.rs`, `const`/`static` declarations only
    Rust,
    /// `*.md`, bold definitions and table rows outside fences
    Markdown,
}

/// One named number, with everything a rule needs to judge it.
#[derive(Debug, Clone)]
pub struct Mention {
    /// Section-qualified key, e.g. `thresholds.binary_max_bytes`.
    pub key: String,
    /// Sorted, stop-worded key tokens. Used to reject generic cross-references.
    pub norm_key: Vec<String>,
    /// The evaluated value, in the units the site wrote it in.
    pub value: f64,
    /// Every plausible canonical reading of that value. Length > 1 means the
    /// unit was ambiguous.
    pub canon: Vec<f64>,
    /// The dimension inferred from the key suffix, overridden by an explicit unit.
    pub dim: Dim,
    /// Repo-relative path.
    pub file: String,
    /// 1-indexed line.
    pub line: usize,
    /// The source line, trimmed and truncated to 200 characters.
    pub text: String,
    /// Which grammar produced it.
    pub kind: FileKind,
    /// Ceiling, floor, or neither.
    pub polarity: Option<Polarity>,
    /// The same-line trailing comment. The only text that can *assert*.
    pub annot: String,
    /// The contiguous comment block above the declaration. Context, not assertion.
    pub block: String,
}

// ---------------------------------------------------------------- value parsing

/// Parse one literal: `50_000_000`, `1,234`, `0xFFFF`, `1e6`.
pub fn parse_literal(s: &str) -> Option<f64> {
    let cleaned: String = s.chars().filter(|c| *c != '_' && *c != ',').collect();
    let lower = cleaned.to_ascii_lowercase();
    if let Some(hex) = lower.strip_prefix("0x") {
        return u128::from_str_radix(hex, 16).ok().map(|v| v as f64);
    }
    cleaned.parse::<f64>().ok()
}

static TYPE_SUFFIX: LazyLock<Regex> =
    LazyLock::new(|| rx(r"(?i)_?(u8|u16|u32|u64|usize|i32|i64|f32|f64)\b"));
static SHIFT: LazyLock<Regex> = LazyLock::new(|| rx(&format!(r"^\s*({NUM})\s*<<\s*({NUM})\s*$")));
static PRODUCT: LazyLock<Regex> =
    LazyLock::new(|| rx(&format!(r"^\s*({NUM})(\s*[*]\s*{NUM})*\s*$")));
static ANY_NUM: LazyLock<Regex> = LazyLock::new(|| rx(NUM));
static NUM_ANCHORED: LazyLock<Regex> = LazyLock::new(|| rx(&format!("^{NUM}")));

/// The numeric literal at the very start of `s`, if there is one.
pub fn num_prefix(s: &str) -> Option<&str> {
    NUM_ANCHORED.find(s).map(|m| m.as_str())
}

/// Evaluate a right-hand side: a literal, a product of literals, or a shift.
///
/// Deliberately narrow. It accepts exactly the shapes measured to matter and
/// nothing else — it is not a general expression evaluator, and it never
/// executes anything from the corpus.
pub fn parse_value_expr(raw: &str) -> Option<f64> {
    let trimmed = raw.trim().trim_end_matches(';').trim();
    let stripped = TYPE_SUFFIX.replace_all(trimmed, "");
    let e = stripped.trim();
    if let Some(c) = SHIFT.captures(e) {
        let a = parse_literal(c.get(1)?.as_str())?;
        let b = parse_literal(c.get(2)?.as_str())?;
        return Some(a * 2f64.powf(b));
    }
    if PRODUCT.is_match(e) {
        let mut acc = 1.0;
        for part in e.split('*') {
            acc *= parse_literal(part.trim())?;
        }
        return Some(acc);
    }
    None
}

/// Count every numeric literal in a blob, whatever its role.
///
/// The census divides the rules' mention count by this, so the check can never
/// be described as auditing "the numbers in the repo": R2 reads about 1.6% of
/// them, and the output has to say so.
pub fn raw_literal_count(text: &str) -> usize {
    ANY_NUM.find_iter(text).count()
}

// ---------------------------------------------------------------- units

fn time_factor(u: &str) -> Option<f64> {
    Some(match u {
        "ms" | "millisecond" | "milliseconds" => 1.0,
        "s" | "sec" | "secs" | "second" | "seconds" => 1_000.0,
        "m" | "min" | "mins" | "minute" | "minutes" => 60_000.0,
        "h" | "hr" | "hour" | "hours" => 3_600_000.0,
        _ => return None,
    })
}

fn byte_factor(u: &str) -> Option<f64> {
    Some(match u {
        "b" | "byte" | "bytes" => 1.0,
        "kb" => 1_000.0,
        "kib" => 1_024.0,
        "mb" => 1_000_000.0,
        "mib" => 1_048_576.0,
        "gb" => 1_000_000_000.0,
        "gib" => 1_073_741_824.0,
        _ => return None,
    })
}

fn pct_factor(u: &str) -> Option<f64> {
    match u {
        "%" | "pct" | "percent" => Some(1.0),
        _ => None,
    }
}

/// The two readings a SI-or-IEC unit could mean.
pub fn ambiguous_byte_factors(u: &str) -> Option<(f64, f64)> {
    match u {
        "kb" => Some((1_000.0, 1_024.0)),
        "mb" => Some((1_000_000.0, 1_048_576.0)),
        "gb" => Some((1_000_000_000.0, 1_073_741_824.0)),
        _ => None,
    }
}

fn unit_factor(dim: Dim, u: &str) -> Option<f64> {
    match dim {
        Dim::Time => time_factor(u),
        Dim::Bytes => byte_factor(u),
        Dim::Pct => pct_factor(u),
        Dim::Count => None,
    }
}

/// The dimension a unit token belongs to, if it names one at all.
pub fn dim_of_unit(unit: &str) -> Option<Dim> {
    let u = unit.trim().to_ascii_lowercase();
    if ambiguous_byte_factors(&u).is_some() || byte_factor(&u).is_some() {
        return Some(Dim::Bytes);
    }
    if time_factor(&u).is_some() {
        return Some(Dim::Time);
    }
    if pct_factor(&u).is_some() {
        return Some(Dim::Pct);
    }
    None
}

fn sorted_dedup(mut v: Vec<f64>) -> Vec<f64> {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v.dedup_by(|a, b| (*a - *b).abs() < f64::EPSILON);
    v
}

/// Every plausible canonical value of `val unit`, in ascending order.
///
/// More than one entry means the unit was ambiguous, and a rule may then fire
/// only if it disagrees with all of them.
pub fn to_canon(val: f64, unit: &str, dim: Dim) -> Vec<f64> {
    let u = unit.trim().to_ascii_lowercase();
    if dim == Dim::Bytes {
        if let Some((a, b)) = ambiguous_byte_factors(&u) {
            return sorted_dedup(vec![val * a, val * b]);
        }
    }
    match unit_factor(dim, &u) {
        Some(f) => vec![val * f],
        None => vec![val],
    }
}

/// Two numbers agree to within [`TOL`].
pub fn close(a: f64, b: f64) -> bool {
    if (a - b).abs() < f64::EPSILON {
        return true;
    }
    if a == 0.0 || b == 0.0 {
        return (a - b).abs() < 1e-9;
    }
    (a - b).abs() / a.abs().max(b.abs()) <= TOL
}

/// Some reading of `a` agrees with some reading of `b`.
pub fn any_close(a: &[f64], b: &[f64]) -> bool {
    a.iter().any(|x| b.iter().any(|y| close(*x, *y)))
}

/// Every reading of `a` agrees with every reading of `b`.
pub fn all_close(a: &[f64], b: &[f64]) -> bool {
    !a.is_empty() && !b.is_empty() && a.iter().all(|x| b.iter().all(|y| close(*x, *y)))
}

// ---------------------------------------------------------------- keys

static KEY_MS: LazyLock<Regex> = LazyLock::new(|| rx(r"_ms$|_millis$"));
static KEY_SEC: LazyLock<Regex> = LazyLock::new(|| rx(r"_secs?$|_seconds$"));
// `_min(s)?$(?<!_min)` in the prototype: the lookbehind rejects a bare `_min`
// suffix, so only the plural survives. `provability_min` is a score, not a
// duration, and reading it as minutes would put it in the wrong dimension.
static KEY_MINS: LazyLock<Regex> = LazyLock::new(|| rx(r"_mins$"));
static KEY_BYTES: LazyLock<Regex> = LazyLock::new(|| rx(r"_bytes$"));
static KEY_MB: LazyLock<Regex> = LazyLock::new(|| rx(r"_mb$"));
static KEY_KB: LazyLock<Regex> = LazyLock::new(|| rx(r"_kb$"));
static KEY_PCT: LazyLock<Regex> = LazyLock::new(|| rx(r"_pct$|_percent$|_percentage$"));
static KEY_DAYS: LazyLock<Regex> = LazyLock::new(|| rx(r"_days$"));

/// The dimension and implied unit a key's suffix declares.
pub fn key_dim(key: &str) -> (Dim, &'static str) {
    let k = key.to_ascii_lowercase();
    for (re, dim, unit) in [
        (&KEY_MS, Dim::Time, "ms"),
        (&KEY_SEC, Dim::Time, "s"),
        (&KEY_MINS, Dim::Time, "min"),
        (&KEY_BYTES, Dim::Bytes, "b"),
        (&KEY_MB, Dim::Bytes, "mb"),
        (&KEY_KB, Dim::Bytes, "kb"),
        (&KEY_PCT, Dim::Pct, "%"),
        (&KEY_DAYS, Dim::Time, "d"),
    ] {
        if re.is_match(&k) {
            return (dim, unit);
        }
    }
    (Dim::Count, "n")
}

static POLARITY_MAX: LazyLock<Regex> =
    LazyLock::new(|| rx(r"\bmax|_max\b|max_|limit|ceiling|budget|threshold|cap\b"));
static POLARITY_MIN: LazyLock<Regex> =
    LazyLock::new(|| rx(r"\bmin|_min\b|min_|floor|least|minimum"));

/// Whether a key names a ceiling, a floor, or neither.
pub fn key_polarity(key: &str) -> Option<Polarity> {
    let k = key.to_ascii_lowercase();
    if POLARITY_MAX.is_match(&k) {
        return Some(Polarity::Max);
    }
    if POLARITY_MIN.is_match(&k) {
        return Some(Polarity::Min);
    }
    None
}

const STOP_TOKENS: [&str; 15] = [
    "the", "a", "of", "for", "max", "min", "and", "to", "in", "is", "total", "num", "number",
    "count", "n",
];

fn split_camel(k: &str) -> String {
    let chars: Vec<char> = k.chars().collect();
    let mut out = String::with_capacity(k.len() + 4);
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && c.is_ascii_uppercase() {
            let prev = chars[i - 1];
            if prev.is_ascii_lowercase() || prev.is_ascii_digit() {
                out.push(' ');
            }
        }
        out.push(*c);
    }
    out
}

/// Sorted, stop-worded tokens of a key.
///
/// Used to reject a cross-reference whose name is a single generic word: one
/// token like `threshold` identifies no quantity, and resolving it would invent
/// a contradiction out of two unrelated keys.
pub fn norm_key(key: &str) -> Vec<String> {
    let spaced = split_camel(key).to_ascii_lowercase();
    let toks: Vec<String> = spaced
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect();
    let mut kept: Vec<String> = toks
        .iter()
        .filter(|t| !STOP_TOKENS.contains(&t.as_str()))
        .cloned()
        .collect();
    if kept.is_empty() {
        kept = toks;
    }
    kept.sort();
    kept
}

// ---------------------------------------------------------------- comments

/// The same-line trailing comment — the only text that may *assert*.
pub fn comment_of(line: &str, kind: FileKind) -> String {
    match kind {
        FileKind::Rust => match line.find("//") {
            Some(i) => line[i + 2..].trim().to_string(),
            None => String::new(),
        },
        FileKind::Toml | FileKind::Yaml => {
            let mut quoted = false;
            for (i, c) in line.char_indices() {
                match c {
                    '"' => quoted = !quoted,
                    '#' if !quoted => return line[i + 1..].trim().to_string(),
                    _ => {}
                }
            }
            String::new()
        }
        FileKind::Markdown => String::new(),
    }
}

fn comment_marker(kind: FileKind) -> Option<&'static str> {
    match kind {
        FileKind::Rust => Some("//"),
        FileKind::Toml | FileKind::Yaml => Some("#"),
        FileKind::Markdown => None,
    }
}

/// The contiguous comment block immediately above a declaration.
///
/// Context, never assertion: a paragraph explains a number, it does not claim
/// to restate it. `.pmat-metrics.toml` at HEAD is exactly this shape, and
/// reading it as an assertion would re-flag work already done.
pub fn preceding_block(lines: &[&str], decl_index: usize, kind: FileKind) -> String {
    let Some(marker) = comment_marker(kind) else {
        return String::new();
    };
    let mut collected: Vec<String> = Vec::new();
    let mut j = decl_index;
    while j > 0 && collected.len() < 8 {
        j -= 1;
        let t = lines[j].trim();
        if !t.starts_with(marker) {
            break;
        }
        collected.push(
            t.trim_matches(|c| c == '/' || c == '!' || c == '#')
                .trim()
                .to_string(),
        );
    }
    collected.reverse();
    collected.join(" ")
}

// ---------------------------------------------------------------- grammars

static TOML_SECTION: LazyLock<Regex> = LazyLock::new(|| rx(r"^\s*\[\[?([^\]]+)\]\]?\s*$"));
static TOML_KV: LazyLock<Regex> =
    LazyLock::new(|| rx(r"^\s*([A-Za-z_][A-Za-z0-9_.-]*)\s*=\s*([^#\n]+)"));
static YAML_MAP: LazyLock<Regex> =
    LazyLock::new(|| rx(r"^(\s*)([A-Za-z_][A-Za-z0-9_.-]*)\s*:\s*$"));
static YAML_KV: LazyLock<Regex> =
    LazyLock::new(|| rx(r"^\s*([A-Za-z_][A-Za-z0-9_.-]*)\s*:\s*([^#\n]+)"));
static RUST_CONST: LazyLock<Regex> = LazyLock::new(|| {
    rx(
        r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:const|static)\s+(?:mut\s+)?([A-Z][A-Z0-9_]{2,})\s*:\s*[^=]+=\s*([^;]+);",
    )
});
static MD_BOLD: LazyLock<Regex> = LazyLock::new(|| {
    rx(&format!(
        r"\*\*([A-Za-z][A-Za-z0-9 _/-]{{2,40}})\*\*\s*[:=]\s*[~≈]?({NUM})\s*([a-zA-Z%]{{0,8}})"
    ))
});
static MD_ROW: LazyLock<Regex> = LazyLock::new(|| {
    rx(&format!(
        r"^\s*\|\s*`?\*?\*?([A-Za-z][A-Za-z0-9 _./-]{{2,40}}?)\*?\*?`?\s*\|\s*[~≈]?({NUM})\s*([a-zA-Z%]{{0,8}})\s*\|"
    ))
});
static VALUE_UNIT: LazyLock<Regex> =
    LazyLock::new(|| rx(&format!(r"^[~≈]?({NUM})\s*([A-Za-z%]{{0,8}})\s*$")));

/// Which grammar a path is read with, or `None` if R2 ignores it.
pub fn file_kind(path: &str) -> Option<FileKind> {
    let ext = path.rsplit('.').next()?;
    match ext {
        "toml" => Some(FileKind::Toml),
        "yaml" | "yml" | "json" => Some(FileKind::Yaml),
        "rs" => Some(FileKind::Rust),
        "md" => Some(FileKind::Markdown),
        _ => None,
    }
}

/// Mutable per-file scanning position: TOML section, YAML indent path, fence depth.
#[derive(Default)]
struct ScanState {
    section: String,
    indents: Vec<(usize, String)>,
    in_fence: bool,
    /// Everything from the first `#[cfg(test)]` to end of file is TEST DATA.
    ///
    /// The mirror test caught this: R2 reported `extract_tests.rs:145` as a
    /// contradiction against `.pmat-metrics.toml`. That line is a Rust STRING
    /// LITERAL inside a unit test — the fixture for
    /// `rust_contributes_const_and_static_only` — and the per-line `RUST_CONST`
    /// regex cannot tell a `const … ;` inside `let rs = "…"` from a real
    /// declaration.
    ///
    /// So the check reported its own test data as a defect in the codebase,
    /// which is `FALSIFY-NC-010` in the contract and would have taken pmat's
    /// own baseline from 1 to 2 on the very commit that introduced the check.
    ///
    /// Truncating at `#[cfg(test)]` rather than excluding `*_tests.rs` by name:
    /// this repository already draws that exact line, in
    /// `.pmat-ratchet.toml`'s `unwrap_outside_cfg_test` metric. A filename
    /// pattern would miss an inline `mod tests` in a production file, which is
    /// the common shape here.
    past_cfg_test: bool,
}

/// True for the attribute that opens a test module.
///
/// Matches `#[cfg(test)]` and the `cfg_attr` and multi-predicate spellings —
/// `#[cfg(all(test, feature = "x"))]` is used in this crate — while NOT matching
/// a mere mention of the word inside a string or comment, which is what the
/// naive `line.contains("cfg(test)")` would do and would truncate this very
/// file at its own doc comment.
fn is_cfg_test_attr(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with("#[cfg(test)]")
        || t.starts_with("#[cfg(all(test")
        || t.starts_with("#[cfg_attr(test")
}

fn track_toml_section(state: &mut ScanState, line: &str) {
    if let Some(c) = TOML_SECTION.captures(line) {
        if let Some(name) = c.get(1) {
            state.section = format!("{}.", name.as_str().trim());
        }
    }
}

fn track_yaml_section(state: &mut ScanState, line: &str) {
    if let Some(c) = YAML_MAP.captures(line) {
        let depth = c.get(1).map_or(0, |m| m.as_str().len());
        let name = c.get(2).map_or("", |m| m.as_str()).to_string();
        while state.indents.last().is_some_and(|(d, _)| *d >= depth) {
            state.indents.pop();
        }
        state.indents.push((depth, name));
        let path: Vec<&str> = state.indents.iter().map(|(_, n)| n.as_str()).collect();
        state.section = format!("{}.", path.join("."));
    } else if !line.trim().is_empty() && !line.starts_with(' ') {
        state.indents.clear();
        state.section.clear();
    }
}

fn markdown_candidates(line: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for c in MD_BOLD.captures_iter(line) {
        let key = c.get(1).map_or("", |m| m.as_str()).to_string();
        let raw = format!(
            "{} {}",
            c.get(2).map_or("", |m| m.as_str()),
            c.get(3).map_or("", |m| m.as_str())
        );
        out.push((key, raw));
    }
    if let Some(c) = MD_ROW.captures(line) {
        let key = c.get(1).map_or("", |m| m.as_str()).to_string();
        let raw = format!(
            "{} {}",
            c.get(2).map_or("", |m| m.as_str()),
            c.get(3).map_or("", |m| m.as_str())
        );
        out.push((key, raw));
    }
    out
}

fn one_capture_pair(re: &Regex, line: &str) -> Vec<(String, String)> {
    match re.captures(line) {
        Some(c) => vec![(
            c.get(1).map_or("", |m| m.as_str()).to_string(),
            c.get(2).map_or("", |m| m.as_str()).to_string(),
        )],
        None => Vec::new(),
    }
}

fn candidates(kind: FileKind, line: &str, state: &ScanState) -> Vec<(String, String)> {
    match kind {
        FileKind::Toml => one_capture_pair(&TOML_KV, line),
        FileKind::Yaml => one_capture_pair(&YAML_KV, line),
        FileKind::Rust => one_capture_pair(&RUST_CONST, line),
        FileKind::Markdown if !state.in_fence => markdown_candidates(line),
        FileKind::Markdown => Vec::new(),
    }
}

/// Split a raw right-hand side into a value and an optional unit token.
fn value_and_unit(raw: &str) -> Option<(f64, String)> {
    if let Some(c) = VALUE_UNIT.captures(raw) {
        let v = parse_literal(c.get(1)?.as_str())?;
        let u = c.get(2).map_or("", |m| m.as_str()).to_string();
        return Some((v, u));
    }
    parse_value_expr(raw).map(|v| (v, String::new()))
}

struct LineContext<'a> {
    file: &'a str,
    line_no: usize,
    line: &'a str,
    lines: &'a [&'a str],
    kind: FileKind,
    section: &'a str,
}

fn build_mention(ctx: &LineContext<'_>, key: &str, raw: &str) -> Option<Mention> {
    let (value, unit) = value_and_unit(raw.trim())?;
    let (mut dim, key_unit) = key_dim(key);
    if !unit.is_empty() {
        if let Some(d) = dim_of_unit(&unit) {
            dim = d;
        }
    }
    let effective_unit = if unit.is_empty() {
        key_unit
    } else {
        unit.as_str()
    };
    let qualified = match ctx.kind {
        FileKind::Toml | FileKind::Yaml => format!("{}{}", ctx.section, key),
        _ => key.to_string(),
    };
    Some(Mention {
        norm_key: norm_key(&qualified),
        canon: to_canon(value, effective_unit, dim),
        value,
        dim,
        file: ctx.file.to_string(),
        line: ctx.line_no,
        text: truncate(ctx.line.trim(), 200),
        kind: ctx.kind,
        polarity: key_polarity(key),
        annot: comment_of(ctx.line, ctx.kind),
        block: preceding_block(ctx.lines, ctx.line_no - 1, ctx.kind),
        key: qualified,
    })
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// Extract every mention from one file.
///
/// Files longer than 60,000 lines are skipped: at that size the file is
/// machine-managed, and R2's grammars are written for hand-authored config.
pub fn extract_file(path: &str, text: &str) -> Vec<Mention> {
    let Some(kind) = file_kind(path) else {
        return Vec::new();
    };
    let lines: Vec<&str> = text.split('\n').collect();
    if lines.len() > 60_000 {
        return Vec::new();
    }
    let mut state = ScanState::default();
    let mut out = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        match kind {
            FileKind::Toml => track_toml_section(&mut state, line),
            FileKind::Yaml => track_yaml_section(&mut state, line),
            FileKind::Markdown if line.trim_start().starts_with("```") => {
                state.in_fence = !state.in_fence;
            }
            FileKind::Rust if is_cfg_test_attr(line) => {
                state.past_cfg_test = true;
            }
            _ => {}
        }
        // Test data is not a claim about the codebase. See `past_cfg_test`.
        if matches!(kind, FileKind::Rust) && state.past_cfg_test {
            continue;
        }
        let ctx = LineContext {
            file: path,
            line_no: idx + 1,
            line,
            lines: &lines,
            kind,
            section: &state.section,
        };
        for (key, raw) in candidates(kind, line, &state) {
            if let Some(m) = build_mention(&ctx, &key, &raw) {
                out.push(m);
            }
        }
    }
    out
}

/// Extract every mention from a corpus.
pub fn extract_all(files: &[CorpusFile]) -> Vec<Mention> {
    files
        .iter()
        .flat_map(|f| extract_file(&f.path, &f.text))
        .collect()
}
