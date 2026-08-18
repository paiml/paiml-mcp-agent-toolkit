//! Walk the module graph of one crate target, carrying the `#[cfg]` context.
//!
//! Keyed on the FULL MODULE PATH, never the bare test name. Two tests in this
//! crate — `our_own_failures_stay_internal` and
//! `documented_bounds_are_invalid_params_not_internal_errors` — exist twice,
//! once under `mcp_pmcp` (compiled by default) and once under `mcp_integration`
//! (compiled only behind a feature). A name search finds them in the default
//! `cargo test -- --list` output and concludes they run; one copy never
//! executed. The path is what distinguishes the copies, so the path is the key.

use super::cfg::{self, CfgExpr};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// One `#[test]`/`#[tokio::test]` function and what it takes to compile it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestFn {
    /// `crate::a::b::tests::name` — the key.
    pub path: String,
    /// Repo-relative file the body lives in.
    pub file: String,
    /// Conjunction of every `#[cfg]` between the crate root and the function.
    pub cfg: CfgExpr,
    /// `#[ignore]`: compiled and listed, but not executed.
    pub ignored: bool,
}

/// What the walk found, including what it could not do.
#[derive(Debug, Clone, Default)]
pub struct Collected {
    pub tests: Vec<TestFn>,
    /// `mod x;` whose file was not found. The subtree below it is missing, so
    /// every count here is a floor.
    pub unresolved: Vec<String>,
    /// Files `syn` refused. Their tests are absent from `tests`, so they are a
    /// finding, not a silent zero.
    pub unparsed: Vec<String>,
    pub files: usize,
}

struct Ctx<'a> {
    root: &'a Path,
    seen: BTreeSet<(PathBuf, String)>,
    out: Collected,
    /// The file currently being walked was textually spliced, so a test's body
    /// may live in one of its `include!` targets rather than in the file named.
    spliced: bool,
}

/// Collect every test reachable from `entry` (a crate root such as `src/lib.rs`).
#[must_use]
pub fn collect(project_root: &Path, entry: &Path) -> Collected {
    let mut ctx = Ctx {
        root: project_root,
        seen: BTreeSet::new(),
        out: Collected::default(),
        spliced: false,
    };
    walk_file(&mut ctx, entry, entry, "crate", &CfgExpr::True, true);
    ctx.out.tests.sort();
    ctx.out.tests.dedup();
    ctx.out
}

/// `file` is where the text comes from (and what `include!` is relative to);
/// `owner` is the module-owning file (what `mod x;` is relative to). They differ
/// exactly inside an `include!`d fragment, which this crate uses 2000+ times.
fn walk_file(
    ctx: &mut Ctx,
    file: &Path,
    owner: &Path,
    modpath: &str,
    cfg_ctx: &CfgExpr,
    is_root: bool,
) {
    let key = (
        file.canonicalize().unwrap_or_else(|_| file.to_path_buf()),
        modpath.to_string(),
    );
    if !ctx.seen.insert(key) {
        return;
    }
    let Ok(src) = std::fs::read_to_string(file) else {
        ctx.out
            .unparsed
            .push(format!("{} (unreadable)", rel(ctx, file)));
        return;
    };
    // Some `include!` targets in this crate are FRAGMENTS: `coverage_tests_
    // part1.rs` opens `mod coverage_tests {` and `part4.rs` closes it, so no
    // part is a token stream on its own. `include!` splices text, so splicing
    // the text is the faithful reading; the recursive walk is used wherever it
    // works because it attributes each test to the file its body lives in.
    let (text, spliced) = match splice_needed(file, &src) {
        false => (src, false),
        true => (splice(file, &src, 0), true),
    };
    let parsed = match syn::parse_file(&text) {
        Ok(f) => f,
        Err(e) => {
            ctx.out.unparsed.push(format!("{}: {e}", rel(ctx, file)));
            return;
        }
    };
    ctx.out.files += 1;
    let inner = cfg::of_attrs(&parsed.attrs);
    let cfg_here = cfg_ctx.clone().and(inner);
    ctx.spliced = spliced;
    walk_items(ctx, &parsed.items, file, owner, modpath, &cfg_here, is_root);
    ctx.spliced = false;
}

/// A line that is nothing but `include!("…")`.
fn include_target(line: &str) -> Option<&str> {
    let t = line.trim();
    let rest = t.strip_prefix("include!(")?;
    let rest = rest.strip_suffix(';').unwrap_or(rest).strip_suffix(')')?;
    let inner = rest.trim();
    inner
        .strip_prefix('"')
        .and_then(|r| r.strip_suffix('"'))
        .filter(|p| !p.is_empty())
}

/// True when at least one `include!` target is not a standalone token stream.
fn splice_needed(file: &Path, src: &str) -> bool {
    let Some(dir) = file.parent() else {
        return false;
    };
    src.lines().filter_map(include_target).any(|rel| {
        std::fs::read_to_string(dir.join(rel))
            .map(|t| syn::parse_file(&t).is_err())
            .unwrap_or(false)
    })
}

/// Textual `include!` expansion, the semantics rustc implements.
fn splice(file: &Path, src: &str, depth: usize) -> String {
    if depth > 8 {
        return src.to_string();
    }
    let Some(dir) = file.parent() else {
        return src.to_string();
    };
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        match include_target(line) {
            Some(rel) => {
                let target = dir.join(rel);
                match std::fs::read_to_string(&target) {
                    Ok(t) => out.push_str(&splice(&target, &t, depth + 1)),
                    Err(_) => out.push_str(line),
                }
            }
            None => out.push_str(line),
        }
        out.push('\n');
    }
    out
}

fn walk_items(
    ctx: &mut Ctx,
    items: &[syn::Item],
    file: &Path,
    owner: &Path,
    modpath: &str,
    cfg_ctx: &CfgExpr,
    is_root: bool,
) {
    for item in items {
        match item {
            syn::Item::Fn(f) => record_fn(ctx, f, file, modpath, cfg_ctx),
            syn::Item::Mod(m) => walk_mod(ctx, m, file, owner, modpath, cfg_ctx, is_root),
            syn::Item::Macro(m) => walk_include(ctx, m, file, owner, modpath, cfg_ctx),
            _ => {}
        }
    }
}

fn is_test_attr(a: &syn::Attribute) -> bool {
    a.path()
        .segments
        .last()
        .is_some_and(|s| s.ident == "test" && matches!(a.meta, syn::Meta::Path(_)))
}

fn record_fn(ctx: &mut Ctx, f: &syn::ItemFn, file: &Path, modpath: &str, cfg_ctx: &CfgExpr) {
    if !f.attrs.iter().any(is_test_attr) {
        return;
    }
    let cfg = cfg_ctx.clone().and(cfg::of_attrs(&f.attrs));
    let file = if ctx.spliced {
        format!("{} (or an include! target of it)", rel(ctx, file))
    } else {
        rel(ctx, file)
    };
    ctx.out.tests.push(TestFn {
        path: format!("{modpath}::{}", f.sig.ident),
        file,
        cfg,
        ignored: f.attrs.iter().any(|a| a.path().is_ident("ignore")),
    });
}

fn walk_mod(
    ctx: &mut Ctx,
    m: &syn::ItemMod,
    file: &Path,
    owner: &Path,
    modpath: &str,
    cfg_ctx: &CfgExpr,
    is_root: bool,
) {
    let cfg_mod = cfg_ctx.clone().and(cfg::of_attrs(&m.attrs));
    let child = format!("{modpath}::{}", m.ident);
    if let Some((_, items)) = &m.content {
        walk_items(ctx, items, file, owner, &child, &cfg_mod, is_root);
        return;
    }
    match resolve_mod(owner, &m.ident.to_string(), path_attr(&m.attrs), is_root) {
        Some(p) => walk_file(ctx, &p, &p, &child, &cfg_mod, false),
        None => ctx
            .out
            .unresolved
            .push(format!("{}: mod {};", rel(ctx, owner), m.ident)),
    }
}

fn path_attr(attrs: &[syn::Attribute]) -> Option<String> {
    attrs.iter().find_map(|a| {
        if !a.path().is_ident("path") {
            return None;
        }
        match &a.meta {
            syn::Meta::NameValue(nv) => match &nv.value {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) => Some(s.value()),
                _ => None,
            },
            _ => None,
        }
    })
}

/// `include!("x.rs")` splices items into the CURRENT module. The included file
/// is not a module of its own, so the module path does not grow — which is why
/// `services::analytics_backend::tests::*` is the path of a test whose text
/// lives in `analytics_backend_tests.rs`.
fn walk_include(
    ctx: &mut Ctx,
    m: &syn::ItemMacro,
    file: &Path,
    owner: &Path,
    modpath: &str,
    cfg_ctx: &CfgExpr,
) {
    if !m.mac.path.is_ident("include") || ctx.spliced {
        return;
    }
    let Ok(lit) = m.mac.parse_body::<syn::LitStr>() else {
        ctx.out.unresolved.push(format!(
            "{}: include! with a non-literal path",
            rel(ctx, file)
        ));
        return;
    };
    let Some(dir) = file.parent() else { return };
    let target = dir.join(lit.value());
    if !target.is_file() {
        ctx.out
            .unresolved
            .push(format!("{}: include!(\"{}\")", rel(ctx, file), lit.value()));
        return;
    }
    let cfg_inc = cfg_ctx.clone().and(cfg::of_attrs(&m.attrs));
    walk_file(ctx, &target, owner, modpath, &cfg_inc, false);
}

/// `mod name;` resolves against the owning file's directory for a crate root
/// and for `mod.rs`, and against `dir/<stem>/` otherwise.
fn resolve_mod(
    owner: &Path,
    name: &str,
    path_attr: Option<String>,
    is_root: bool,
) -> Option<PathBuf> {
    let parent = owner.parent()?;
    if let Some(p) = path_attr {
        let c = parent.join(p);
        return c.is_file().then_some(c);
    }
    let stem = owner.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let dir = if is_root || stem == "mod" || stem == "lib" || stem == "main" {
        parent.to_path_buf()
    } else {
        parent.join(stem)
    };
    [
        dir.join(format!("{name}.rs")),
        dir.join(name).join("mod.rs"),
    ]
    .into_iter()
    .find(|c| c.is_file())
}

fn rel(ctx: &Ctx, p: &Path) -> String {
    p.strip_prefix(ctx.root).unwrap_or(p).display().to_string()
}
