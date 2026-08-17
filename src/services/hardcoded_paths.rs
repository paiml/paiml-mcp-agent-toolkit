//! Find machine-specific absolute paths baked into source.
//!
//! # Why this exists
//!
//! aprender shipped binaries containing `/home/noah/...`. They worked perfectly
//! on the machine that built them and were inert everywhere else. Nothing in the
//! quality gates saw it: the code compiled, the tests passed (on that machine),
//! clippy was clean, and the path was just a string literal.
//!
//! That is the failure shape this module exists for — a value that is correct on
//! exactly one host, embedded where it cannot be configured.
//!
//! # What counts as machine-specific
//!
//! A path is machine-specific when it names a location that is not guaranteed to
//! exist on another machine:
//!
//! - `/home/<user>/…`, `/Users/<user>/…`, `C:\Users\<user>\…` — someone's home
//! - `/nix/store/<hash>-…` — a build-host store path
//! - `~/.cargo/registry/…`, `/root/…` — build-host toolchain state
//!
//! And a path is NOT machine-specific merely by being absolute. `/usr/bin/env`,
//! `/etc/hosts`, `/proc/self/status`, `/dev/null` exist on every Linux host by
//! specification. Flagging those would bury the real finding in noise — the same
//! way SATD detection reached a 92% false-positive rate (#925) by matching on
//! phrases instead of on the thing it actually cared about.
//!
//! So the rule here is narrow on purpose: **a literal is flagged only when it
//! names a specific user, a specific store hash, or a specific build root.**
//! Being absolute is not enough.
//!
//! # The report states what it did not measure
//!
//! Every count carries its denominator, and files that could not be read are
//! reported rather than dropped. A scan that walked nothing must not be
//! indistinguishable from a scan that found nothing (#1015).

use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// How a machine-specific path was classified.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PathKind {
    /// `/home/<user>/…`, `/Users/<user>/…`, `C:\Users\<user>\…`
    UserHome,
    /// `/nix/store/<hash>-…` — reproducible only on the build host's store
    NixStore,
    /// `/root/…`, `<…>/.cargo/registry/…`, `<…>/.rustup/toolchains/…`
    BuildHost,
}

impl PathKind {
    pub fn as_str(self) -> &'static str {
        match self {
            PathKind::UserHome => "user-home",
            PathKind::NixStore => "nix-store",
            PathKind::BuildHost => "build-host",
        }
    }

    /// Why this location is not portable, in one clause.
    pub fn reason(self) -> &'static str {
        match self {
            PathKind::UserHome => "names a specific user's home directory",
            PathKind::NixStore => "names a specific nix store path",
            PathKind::BuildHost => "names build-host toolchain state",
        }
    }
}

/// Where a finding sits in the crate, which decides how much it matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Site {
    /// Reached by a shipped binary or library — the aprender case.
    Shipped,
    /// Test code. Still wrong (it pins the suite to one machine) but it cannot
    /// reach a user.
    Test,
    /// Documentation and comments. Misleading, not broken. (Cargo `examples/`
    /// are NOT here — they compile to binaries and count as shipped.)
    Doc,
}

impl Site {
    pub fn as_str(self) -> &'static str {
        match self {
            Site::Shipped => "shipped",
            Site::Test => "test",
            Site::Doc => "doc",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub kind: PathKind,
    pub site: Site,
    /// The offending path, truncated for display.
    pub path: String,
}

/// Files the scan could not read, and why. Kept so a partial scan cannot be
/// reported as a complete one.
#[derive(Debug, Clone, Serialize, Default)]
pub struct Skipped {
    pub unreadable: Vec<String>,
    pub not_utf8: Vec<String>,
}

impl Skipped {
    pub fn total(&self) -> usize {
        self.unreadable.len() + self.not_utf8.len()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub findings: Vec<Finding>,
    /// Denominator: files actually opened and scanned.
    pub files_scanned: usize,
    /// Denominator: string literals actually examined.
    pub literals_scanned: usize,
    pub skipped: Skipped,
}

impl Report {
    pub fn shipped(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.site == Site::Shipped)
            .count()
    }

    pub fn by_kind(&self) -> BTreeMap<&'static str, usize> {
        let mut m = BTreeMap::new();
        for f in &self.findings {
            *m.entry(f.kind.as_str()).or_insert(0) += 1;
        }
        m
    }

    /// A one-line verdict that always carries its denominator, and always says
    /// so when the scan was partial.
    pub fn summary(&self) -> String {
        let mut s = format!(
            "{} machine-specific path(s) in {} literal(s) across {} file(s); {} in shipped code",
            self.findings.len(),
            self.literals_scanned,
            self.files_scanned,
            self.shipped(),
        );
        if self.skipped.total() > 0 {
            // A FLOOR, not a total: unread files may hold more.
            s.push_str(&format!(
                " — FLOOR ONLY: {} file(s) could not be read ({} unreadable, {} not UTF-8)",
                self.skipped.total(),
                self.skipped.unreadable.len(),
                self.skipped.not_utf8.len(),
            ));
        }
        s
    }
}

/// Classify a candidate path. `None` means portable — do not flag.
///
/// This is the whole false-positive story, so it is deliberately conservative:
/// each arm requires a *specific* identifier (a username, a store hash, a
/// toolchain root), never merely a leading slash.
fn classify(path: &str) -> Option<PathKind> {
    // `/home/<user>/` and `/Users/<user>/` — the trailing slash matters. Bare
    // `/home` or `/Users` is the mount point, which is portable and meaningless
    // on its own; it is the *named user under it* that pins the machine.
    for prefix in ["/home/", "/Users/"] {
        if let Some(rest) = path.strip_prefix(prefix) {
            let user = rest.split('/').next().unwrap_or("");
            if is_username(user) {
                return Some(PathKind::UserHome);
            }
            // A placeholder home is portable all the way down, so stop here
            // rather than fall through to the build-host rules below —
            // `/home/user/.cargo/registry/…` is a documentation example, not
            // this machine's crate cache.
            return None;
        }
    }
    // Windows: `C:\Users\<user>\` in any drive letter, either slash direction.
    if let Some(rest) = windows_users_suffix(path) {
        let user = rest.split(['/', '\\']).next().unwrap_or("");
        if is_username(user) {
            return Some(PathKind::UserHome);
        }
    }
    if path
        .strip_prefix("/nix/store/")
        .is_some_and(|r| !r.is_empty())
    {
        return Some(PathKind::NixStore);
    }
    // A bare `/root/` with nothing under it is a prefix, not a location. pmat's
    // own hook checker lists `["/home/", "/Users/", "/root/"]` as the patterns
    // it searches for; flagging a detector's own pattern table is the noise that
    // makes a detector unusable.
    if path.strip_prefix("/root/").is_some_and(|r| !r.is_empty()) {
        return Some(PathKind::BuildHost);
    }
    // Toolchain state, wherever it is rooted. The host root is what makes it
    // machine-specific, so there must be something before `/.cargo/registry/`:
    // `/home/x/.cargo/registry/` is a real location, while the bare
    // `"/.cargo/registry/"` in `file_discovery.rs` is an ignore-glob fragment.
    for marker in ["/.cargo/registry/", "/.rustup/toolchains/"] {
        if path.starts_with('/') && path.find(marker).is_some_and(|at| at > 0) {
            return Some(PathKind::BuildHost);
        }
    }
    None
}

/// `C:\Users\<user>` / `c:/users/<user>` — returns what follows `Users`, if the
/// path has that shape. Both slash directions and either case, because Windows
/// paths reach source in every combination.
fn windows_users_suffix(path: &str) -> Option<&str> {
    let bytes = path.as_bytes();
    // Drive letter, colon, separator: `C:\` or `c:/`.
    if bytes.len() < 3 || !bytes[0].is_ascii_alphabetic() || bytes[1] != b':' {
        return None;
    }
    let rest = path[2..]
        .strip_prefix('\\')
        .or_else(|| path[2..].strip_prefix('/'))?;
    // `Users` then a separator.
    let after = rest.get(..5).filter(|h| h.eq_ignore_ascii_case("Users"))?;
    let tail = &rest[after.len()..];
    tail.strip_prefix('\\').or_else(|| tail.strip_prefix('/'))
}

/// Does this look like a username rather than a system directory?
///
/// The point of the check is to avoid flagging portable paths that merely live
/// under `/home` or `/Users` conceptually. In practice the discriminator is
/// simple: a real segment that is not a placeholder.
fn is_username(seg: &str) -> bool {
    if seg.is_empty() || seg.len() > 32 {
        return false;
    }
    // `/home/.cache/data` — a dotfile directly under the home mount is not a
    // user account.
    if seg.starts_with('.') {
        return false;
    }
    // Placeholders in documentation and templates are not machine-specific:
    // `/home/user`, `/home/$USER`, `/home/{user}`, `/home/<name>`.
    if seg.starts_with('$') || seg.starts_with('{') || seg.starts_with('<') || seg.starts_with('%')
    {
        return false;
    }
    const PLACEHOLDERS: &[&str] = &[
        "user",
        "username",
        "youruser",
        "your-user",
        "me",
        "someone",
        "USER",
        "USERNAME",
        "runner", // GitHub-hosted runners: portable by construction
    ];
    if PLACEHOLDERS.iter().any(|p| p.eq_ignore_ascii_case(seg)) {
        return false;
    }
    seg.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
}

/// Extract candidate path substrings from one line, with the byte offset of
/// each so callers could map back if needed.
///
/// This scans raw text rather than parsing Rust. That is a deliberate tradeoff:
/// the detector must work on `.sh`, `.py`, `.toml` and `.yaml` too — aprender's
/// leak was in a binary built from Rust, but the same class shows up in every
/// script the repo ships. Scanning text is the only representation all of them
/// share.
/// May an absolute path begin right after this byte?
///
/// This one predicate carries most of the false-positive load, because the
/// thing that makes a path absolute is not the leading `/` — it is the leading
/// `/` *at the start of the path*. Three very common constructs put a `/` after
/// something else and are entirely portable:
///
/// ```text
///   $HOME/.cargo/registry      -> `/` follows `E`   (shell variable)
///   $(HOME)/.cargo/registry    -> `/` follows `)`   (make variable)
///   https://example.com/home/x -> `/` follows `m`   (URL path)
/// ```
///
/// An earlier revision special-cased only the URL form by testing for a
/// preceding `/`, which let both variable forms through — `$(HOME)/.cargo/…` in
/// this repo's own Makefile was reported as a build-host leak. Requiring a real
/// boundary handles all three with one rule, and cannot be defeated by a form
/// nobody thought of: anything that is not a separator simply is not a start.
fn is_boundary(prev: u8) -> bool {
    matches!(
        prev,
        b' ' | b'\t' | b'"' | b'\'' | b'`' | b'(' | b'[' | b'{' | b'=' | b',' | b':' | b'<' | b'|'
    )
}

fn candidates(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let starts_path = bytes[i] == b'/'
            || (i + 2 < bytes.len()
                && bytes[i].is_ascii_alphabetic()
                && bytes[i + 1] == b':'
                && (bytes[i + 2] == b'\\' || bytes[i + 2] == b'/'));
        if !starts_path || (i > 0 && !is_boundary(bytes[i - 1])) {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() {
            let c = bytes[i];
            // Terminate on whitespace, quotes, and shell/format metacharacters.
            if c.is_ascii_whitespace()
                || matches!(
                    c,
                    b'"' | b'\'' | b'`' | b')' | b']' | b'}' | b',' | b';' | b'|' | b'>' | b'<'
                )
            {
                break;
            }
            i += 1;
        }
        if i > start {
            out.push(&line[start..i]);
        }
    }
    out
}

/// Which extensions carry code or config we care about.
fn is_scannable(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "sh" | "bash" | "py" | "toml" | "yaml" | "yml" | "json" | "mk" | "ts" | "js")
    ) || path.file_name().and_then(|f| f.to_str()) == Some("Makefile")
}

/// Where in the crate a file sits. Path-based, because a file's role is a
/// property of the build graph, not of its contents.
fn site_of(rel: &str) -> Site {
    let p = rel.replace('\\', "/");
    let name = p.rsplit('/').next().unwrap_or(&p);
    // Test *files*, by every convention in this tree: `foo_test.rs`,
    // `foo_tests.rs`, `coverage_tests_unit.rs` (a `#[cfg(test)] mod` declared in
    // the parent, which a per-file scan cannot see), and the TypeScript
    // `x.test.ts` / `x.spec.ts` forms.
    // Note `_test_` is deliberately absent: `scripts/fix_property_test_warnings.py`
    // is a script that edits tests, not a test, and matching that substring
    // hid it from the shipped tier.
    if name == "tests.rs"
        || name == "test.rs"
        || name.starts_with("test_")
        || name.contains("_tests_")
        || name.contains(".test.")
        || name.contains(".spec.")
        || name.ends_with("_test.rs")
        || name.ends_with("_tests.rs")
    {
        return Site::Test;
    }
    if p.starts_with("tests/")
        || p.contains("/tests/")
        || p.starts_with("benches/")
        || p.contains("/fixtures/")
        || p.starts_with("fixtures/")
    {
        return Site::Test;
    }
    // `examples/` is deliberately NOT here. A cargo example is a compiled
    // binary that `cargo run --example` executes, so a path baked into one
    // fails for every user who is not the author — which is the aprender defect
    // itself, not a documentation nit. An earlier revision mapped a top-level
    // `examples/` to Doc while `crates/*/examples/` fell through to Shipped;
    // the same file was graded two ways depending on workspace layout.
    if p.starts_with("docs/") || p.ends_with(".md") || p.starts_with("book/") {
        return Site::Doc;
    }
    Site::Shipped
}

/// Is this line a comment? Comment hits are documentation-grade, not defects,
/// so they are downgraded rather than dropped — a `/home/noah` in a comment is
/// still a sign someone developed against one machine.
fn is_comment(line: &str, ext: Option<&str>) -> bool {
    let t = line.trim_start();
    match ext {
        Some("rs" | "ts" | "js") => t.starts_with("//") || t.starts_with('*'),
        Some("sh" | "bash" | "py" | "toml" | "yaml" | "yml" | "mk") => t.starts_with('#'),
        _ => false,
    }
}

/// Tracks whether the current line sits inside a `#[cfg(test)]` module.
///
/// Rust's convention is an inline `#[cfg(test)] mod tests { … }` at the foot of
/// the file it tests, so the majority of a crate's test code lives in files
/// whose *path* says nothing about testing. Classifying by path alone put every
/// one of those fixtures in `shipped`: pmat's own scan reported synthetic
/// fixtures like `/home/u/proj/src/b.rs` as production leaks, which is precisely
/// the noise that trains people to ignore a detector.
///
/// Brace counting rather than a "once seen, always test" flag, because a file
/// may declare a test module in the middle and continue with real code after
/// it.
///
/// Braces inside string literals and comments are excluded, and that is not a
/// nicety — a naive count got this wrong on pmat's own tree. In
/// `src/tdg/formatters/ungraded.rs` a test asserts on the rustc message
/// ``"…unexpected end of input while looking for `}`"``. That lone `}` in a
/// string closed the module 30 lines early, so the fixture path on the next
/// line was reported as a production leak. One stray brace in one string was
/// enough to produce a false finding, which is why the counter reads Rust
/// lexically rather than by `matches('{').count()`.
/// Count `{` and `}` that are real Rust syntax, skipping string literals, char
/// literals and line comments. Raw strings (`r"…"`, `r#"…"#`) are handled;
/// literals spanning multiple lines are not, because this reads line by line —
/// that residual is why [`CfgTestTracker`] treats an early close as "leave the
/// test region" (report as shipped) rather than the reverse.
fn braces_outside_literals(line: &str) -> (i32, i32) {
    let b = line.as_bytes();
    let (mut opens, mut closes) = (0i32, 0i32);
    let mut i = 0usize;
    while i < b.len() {
        match b[i] {
            b'/' if i + 1 < b.len() && b[i + 1] == b'/' => break, // line comment
            b'{' => opens += 1,
            b'}' => closes += 1,
            b'\'' => {
                // Char literal or a lifetime (`'a`). Only a closing quote within
                // a few bytes makes it a literal; lifetimes have none.
                i += 1;
                let start = i;
                while i < b.len() && i - start < 4 {
                    if b[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if b[i] == b'\'' {
                        break;
                    }
                    i += 1;
                }
                if i >= b.len() || b[i] != b'\'' {
                    i = start; // a lifetime; rewind and keep scanning normally
                    continue;
                }
            }
            b'r' if i + 1 < b.len() && (b[i + 1] == b'"' || b[i + 1] == b'#') => {
                // Raw string: r"…" or r#"…"# with any number of hashes.
                let mut hashes = 0usize;
                let mut j = i + 1;
                while j < b.len() && b[j] == b'#' {
                    hashes += 1;
                    j += 1;
                }
                if j >= b.len() || b[j] != b'"' {
                    i += 1;
                    continue;
                }
                j += 1;
                // Scan to the terminator: `"` followed by `hashes` `#`s.
                while j < b.len() {
                    if b[j] == b'"' && b[j + 1..].iter().take(hashes).all(|c| *c == b'#') {
                        j += 1 + hashes;
                        break;
                    }
                    j += 1;
                }
                i = j;
                continue;
            }
            b'"' => {
                i += 1;
                while i < b.len() {
                    if b[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if b[i] == b'"' {
                        break;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    (opens, closes)
}

#[derive(Default)]
struct CfgTestTracker {
    armed: bool,
    depth: i32,
}

impl CfgTestTracker {
    /// Returns whether this line is inside a `#[cfg(test)]` module.
    fn feed(&mut self, line: &str, ext: Option<&str>) -> bool {
        if ext != Some("rs") {
            return false;
        }
        let t = line.trim();
        if self.depth == 0 {
            if t.starts_with("#[cfg(test)]") || t.starts_with("#[cfg(all(test") {
                self.armed = true;
                return false;
            }
            if self.armed {
                // The `mod … {` that the attribute applies to. Attributes may
                // stack, so tolerate intervening attribute lines.
                if t.starts_with("#[") || t.is_empty() {
                    return false;
                }
                self.armed = false;
                if t.contains(" mod ") || t.starts_with("mod ") {
                    self.depth = i32::from(t.contains('{'));
                    return self.depth > 0;
                }
                return false;
            }
            return false;
        }
        let (opens, closes) = braces_outside_literals(line);
        self.depth += opens - closes;
        if self.depth <= 0 {
            self.depth = 0;
            return true; // the closing line is still part of the module
        }
        true
    }
}

/// Scan one file's text. Split out from [`analyze`] so it can be tested without
/// touching the filesystem — the fixture-and-code-share-an-author trap is
/// avoided instead by [`analyze`]'s own tests running over a real tree.
pub fn scan_text(rel: &str, text: &str) -> (Vec<Finding>, usize) {
    let ext = Path::new(rel).extension().and_then(|e| e.to_str());
    let base_site = site_of(rel);
    let mut findings = Vec::new();
    let mut literals = 0usize;
    let mut cfg_test = CfgTestTracker::default();

    for (idx, line) in text.lines().enumerate() {
        let in_test = cfg_test.feed(line, ext);
        let comment = is_comment(line, ext);
        for cand in candidates(line) {
            literals += 1;
            let Some(kind) = classify(cand) else { continue };
            let site = if comment {
                Site::Doc
            } else if in_test {
                Site::Test
            } else {
                base_site
            };
            findings.push(Finding {
                file: rel.to_string(),
                line: idx + 1,
                kind,
                site,
                path: truncate(cand, 120),
            });
        }
    }
    (findings, literals)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let head: String = s.chars().take(max).collect();
    format!("{head}…")
}

/// List the repo's tracked files via git, so the scan matches what is actually
/// shipped rather than whatever is lying in the working tree.
pub fn tracked_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z"])
        .output()?;
    if !out.status.success() {
        anyhow::bail!(
            "git ls-files failed in {} — cannot enumerate tracked files, so the scan would \
             have no denominator",
            root.display()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .filter(|p| is_scannable(p))
        .collect())
}

/// Scan a repository.
pub fn analyze(root: &Path, files: &[PathBuf]) -> Report {
    let mut findings = Vec::new();
    let mut skipped = Skipped::default();
    let mut files_scanned = 0usize;
    let mut literals_scanned = 0usize;

    for rel in files {
        let abs = root.join(rel);
        let rel_s = rel.to_string_lossy().to_string();
        let bytes = match std::fs::read(&abs) {
            Ok(b) => b,
            Err(_) => {
                skipped.unreadable.push(rel_s);
                continue;
            }
        };
        let text = match String::from_utf8(bytes) {
            Ok(t) => t,
            Err(_) => {
                skipped.not_utf8.push(rel_s);
                continue;
            }
        };
        files_scanned += 1;
        let (mut f, lits) = scan_text(&rel_s, &text);
        literals_scanned += lits;
        findings.append(&mut f);
    }

    findings.sort_by(|a, b| {
        a.site
            .cmp(&b.site)
            .then(a.file.cmp(&b.file))
            .then(a.line.cmp(&b.line))
    });

    Report {
        findings,
        files_scanned,
        literals_scanned,
        skipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_named_home_is_machine_specific() {
        assert_eq!(classify("/home/noah/src/x"), Some(PathKind::UserHome));
        assert_eq!(classify("/Users/alice/Code"), Some(PathKind::UserHome));
        assert_eq!(
            classify("C:\\Users\\bob\\AppData"),
            Some(PathKind::UserHome)
        );
    }

    /// The false-positive floor. Every one of these exists on any Linux host by
    /// specification; flagging them would drown the real finding. This is the
    /// #925 lesson applied up front rather than after a 92% FP rate.
    #[test]
    fn ordinary_system_paths_are_not_flagged() {
        for p in [
            "/usr/bin/env",
            "/etc/hosts",
            "/proc/self/status",
            "/dev/null",
            "/var/log/syslog",
            "/opt/tool/bin",
            "/tmp",
            "/tmp/scratch",
            "/bin/sh",
            "/",
            "/home",
            "/Users",
        ] {
            assert_eq!(classify(p), None, "{p} must not be flagged");
        }
    }

    /// Documentation placeholders name no machine. `/home/user` in a README is
    /// instruction, not leakage.
    #[test]
    fn placeholders_are_not_machine_specific() {
        for p in [
            "/home/user/project",
            "/home/$USER/project",
            "/home/{user}/project",
            "/home/<name>/project",
            "/home/username/x",
            "/home/runner/work/repo", // GitHub-hosted runner: same on every run
        ] {
            assert_eq!(classify(p), None, "{p} must not be flagged");
        }
    }

    #[test]
    fn build_host_state_is_machine_specific() {
        assert_eq!(classify("/root/.config/x"), Some(PathKind::BuildHost));
        assert_eq!(
            classify("/home/x/.cargo/registry/src/index/foo"),
            // user-home wins: it is the more specific and more actionable class
            Some(PathKind::UserHome)
        );
        assert_eq!(
            classify("/opt/ci/.rustup/toolchains/stable/bin"),
            Some(PathKind::BuildHost)
        );
        assert_eq!(
            classify("/nix/store/abc123-rustc-1.90/bin/rustc"),
            Some(PathKind::NixStore)
        );
        // Relative toolchain config is ordinary project config.
        assert_eq!(classify(".cargo/registry/x"), None);
    }

    /// A URL that happens to contain `/home/` is not a filesystem path. Without
    /// this, every docs link to a user page would be a finding.
    #[test]
    fn urls_are_not_paths() {
        let (f, _) = scan_text("src/a.rs", r#"let u = "https://example.com/home/noah/x";"#);
        assert!(f.is_empty(), "URL was flagged as a path: {f:?}");
    }

    #[test]
    fn shipped_code_is_ranked_above_tests_and_docs() {
        let (shipped, _) = scan_text("src/main.rs", r#"let p = "/home/noah/data";"#);
        let (test, _) = scan_text("tests/it.rs", r#"let p = "/home/noah/data";"#);
        let (doc, _) = scan_text("docs/guide.md", "see /home/noah/data");
        assert_eq!(shipped[0].site, Site::Shipped);
        assert_eq!(test[0].site, Site::Test);
        assert_eq!(doc[0].site, Site::Doc);
    }

    /// A comment is a weaker signal than an executable literal, but not zero.
    #[test]
    fn a_comment_hit_is_downgraded_not_dropped() {
        let (f, _) = scan_text("src/a.rs", "// built against /home/noah/src/x");
        assert_eq!(f.len(), 1, "comment hit was dropped entirely");
        assert_eq!(f[0].site, Site::Doc);
    }

    /// The property that makes the whole report trustworthy: a scan that read
    /// nothing must not read as a clean scan. This is #1015 as an assertion.
    #[test]
    fn an_unread_file_downgrades_the_claim_to_a_floor() {
        let clean = Report {
            findings: vec![],
            files_scanned: 10,
            literals_scanned: 100,
            skipped: Skipped::default(),
        };
        assert!(!clean.summary().contains("FLOOR"));

        let partial = Report {
            findings: vec![],
            files_scanned: 10,
            literals_scanned: 100,
            skipped: Skipped {
                unreadable: vec!["a.rs".into()],
                not_utf8: vec![],
            },
        };
        assert!(
            partial.summary().contains("FLOOR ONLY"),
            "a partial scan reported as complete: {}",
            partial.summary()
        );
    }

    /// Every summary carries its denominator. A bare "0 findings" is the defect
    /// this detector exists to catch, so it must not be the thing it emits.
    #[test]
    fn the_summary_always_carries_its_denominator() {
        let r = Report {
            findings: vec![],
            files_scanned: 42,
            literals_scanned: 7,
            skipped: Skipped::default(),
        };
        let s = r.summary();
        assert!(s.contains("42 file(s)"), "{s}");
        assert!(s.contains("7 literal(s)"), "{s}");
    }

    /// A path built from a variable is portable, whatever it looks like after
    /// the variable is stripped. This repo's own Makefile has
    /// `$(HOME)/.cargo/registry/cache/*`, which the first revision reported as a
    /// build-host leak.
    #[test]
    fn variable_rooted_paths_are_portable() {
        for line in [
            "\trm -rf $(HOME)/.cargo/registry/cache/*",
            "rm -rf $HOME/.cargo/registry",
            "CACHE=${HOME}/.cargo/registry",
            "let p = format!(\"{home}/.cargo/registry\");",
        ] {
            let (f, _) = scan_text("Makefile", line);
            assert!(
                f.is_empty(),
                "variable-rooted path flagged: {f:?} in {line}"
            );
        }
    }

    /// Rust's test code overwhelmingly lives in an inline `#[cfg(test)] mod`
    /// inside the file it tests, so a path-only classifier calls every fixture
    /// production code. pmat's own scan did exactly that.
    #[test]
    fn an_inline_cfg_test_module_is_test_site_not_shipped() {
        let src = r#"
pub fn real() -> &'static str { "/home/noah/prod" }

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        let fixture = "/home/noah/fixture";
        assert!(fixture.len() > 0);
    }
}
"#;
        let (f, _) = scan_text("src/lib.rs", src);
        assert_eq!(f.len(), 2, "expected both paths found: {f:?}");
        let prod = f.iter().find(|x| x.path.contains("prod")).expect("prod");
        let fixture = f
            .iter()
            .find(|x| x.path.contains("fixture"))
            .expect("fixture");
        assert_eq!(prod.site, Site::Shipped, "production literal misclassified");
        assert_eq!(
            fixture.site,
            Site::Test,
            "an inline #[cfg(test)] fixture was reported as shipped code"
        );
    }

    /// The tracker must not swallow the rest of the file. Code after the test
    /// module closes is production code again.
    #[test]
    fn code_after_a_test_module_is_shipped_again() {
        let src = r#"
#[cfg(test)]
mod tests {
    fn t() { let _ = "/home/noah/fixture"; }
}

pub fn later() -> &'static str { "/home/noah/prod" }
"#;
        let (f, _) = scan_text("src/lib.rs", src);
        let prod = f.iter().find(|x| x.path.contains("prod")).expect("prod");
        assert_eq!(
            prod.site,
            Site::Shipped,
            "the test module swallowed the code that followed it"
        );
    }

    /// A brace inside a string literal must not close the test module. Taken
    /// verbatim from `src/tdg/formatters/ungraded.rs`, where this exact line
    /// ended the `#[cfg(test)] mod` 30 lines early and turned the fixture that
    /// followed into a reported production leak.
    #[test]
    fn a_brace_inside_a_string_does_not_close_the_test_module() {
        let src = "\
#[cfg(test)]
mod tests {
    fn a() {
        let msg = \"unexpected end of input while looking for `}`\";
        let _ = msg;
    }
    fn b() {
        let p = \"/home/noah/fixture\";
        let _ = p;
    }
}
";
        let (f, _) = scan_text("src/lib.rs", src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(
            f[0].site,
            Site::Test,
            "a `}}` in a string closed the test module early"
        );
    }

    #[test]
    fn braces_in_literals_are_not_counted() {
        assert_eq!(braces_outside_literals("fn a() {"), (1, 0));
        assert_eq!(braces_outside_literals("let s = \"}\";"), (0, 0));
        assert_eq!(braces_outside_literals("let s = r#\"{{{\"#;"), (0, 0));
        assert_eq!(braces_outside_literals("let c = '}';"), (0, 0));
        assert_eq!(braces_outside_literals("} // trailing { comment"), (0, 1));
        // A lifetime is not a char literal — the brace after it still counts.
        assert_eq!(braces_outside_literals("impl<'a> T<'a> for U {"), (1, 0));
    }

    /// pmat's own hook checker lists the prefixes it searches for. A detector
    /// that flags another detector's pattern table is unusable.
    #[test]
    fn bare_prefixes_and_glob_fragments_are_not_locations() {
        for p in ["/root/", "/nix/store/", "/.cargo/registry/", "/.rustup/"] {
            assert_eq!(classify(p), None, "{p} is a prefix, not a location");
        }
        // …but a real location under them still is one.
        assert_eq!(classify("/root/.config"), Some(PathKind::BuildHost));
        assert_eq!(classify("/nix/store/abc-x/bin"), Some(PathKind::NixStore));
    }

    #[test]
    fn test_files_are_recognised_by_every_convention_in_this_tree() {
        for f in [
            "src/a/coverage_tests_unit.rs",
            "src/a/tests.rs",
            "src/a/foo_tests.rs",
            "src/a/foo_test.rs",
            "src/a/test_helpers.rs",
            "scripts/lib/install-utils.test.ts",
            "web/app.spec.ts",
        ] {
            assert_eq!(site_of(f), Site::Test, "{f} was not recognised as a test");
        }
        assert_eq!(site_of("src/services/file_discovery.rs"), Site::Shipped);
        // A script that EDITS tests is not a test.
        assert_eq!(
            site_of("scripts/fix_property_test_warnings.py"),
            Site::Shipped
        );
        // "latest.rs" contains "test" but is not a test file.
        assert_eq!(site_of("src/latest.rs"), Site::Shipped);
    }

    /// A cargo example is a binary, and it is graded the same wherever the
    /// workspace puts it.
    #[test]
    fn cargo_examples_are_shipped_binaries_at_any_depth() {
        assert_eq!(site_of("examples/bench.rs"), Site::Shipped);
        assert_eq!(
            site_of("crates/aprender-serve/examples/bench_qwen.rs"),
            Site::Shipped
        );
        assert_eq!(site_of("docs/guide.md"), Site::Doc);
    }

    #[test]
    fn a_dotfile_under_the_home_mount_is_not_a_user() {
        assert_eq!(classify("/home/.cache/data"), None);
    }

    /// A placeholder home stays portable all the way down. Without the early
    /// return, `/home/user/.cargo/registry/…` — a documentation example — fell
    /// through to the build-host rule and was reported as this machine's crate
    /// cache.
    #[test]
    fn a_placeholder_home_is_portable_all_the_way_down() {
        assert_eq!(
            classify("/home/user/.cargo/registry/src/crates.io-abc/dep-1.0.0/src/lib.rs"),
            None
        );
        // A real user under the same shape is still a finding.
        assert_eq!(
            classify("/home/noah/.cargo/registry/src/x"),
            Some(PathKind::UserHome)
        );
    }

    #[test]
    fn multiple_paths_on_one_line_are_all_found() {
        let (f, _) = scan_text(
            "src/a.rs",
            r#"copy("/home/alice/a", "/home/bob/b", "/usr/share/c");"#,
        );
        assert_eq!(f.len(), 2, "expected two user homes, got {f:?}");
    }
}
