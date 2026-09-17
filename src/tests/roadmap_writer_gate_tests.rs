#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-1385 (#1385) — the roadmap writer gate, run over this crate's own `src/`, and
//! the planted mutants that prove it can fail.
//!
//! The tree test is the gate: every raw filesystem write that can land under
//! `docs/roadmaps/` must be one of [`ALLOWED`], each of which is a method of
//! `RoadmapWriteLock` — a value that exists only while the repository's exclusive
//! roadmap lock is held. So "no write bypasses the lock" is split into two claims,
//! each checked by something that can fail: the type system proves every write
//! through the token holds the lock, and this gate proves nothing writes around it.
//!
//! Every planted mutant below is RED on purpose, including the one the literal
//! query cannot see: the same unlocked write with its binding renamed.
//!
//! `scripts/roadmap-writer-gate.sh` runs this suite and refuses a vacuous run.
//! Registered from `cli/handlers/work_handlers/mod.rs`, beside the PMAT-1363 suites.

use std::path::{Path, PathBuf};

use super::roadmap_writer_gate::{is_source, is_test_only as test_only, tainted_sinks, Sink};

/// The only functions allowed to write a path under `docs/roadmaps/` with a raw
/// filesystem call, and why.
const ALLOWED: &[(&str, &str, &str)] = &[
    (
        "src/services/roadmap_write_lock.rs",
        "RoadmapWriteLock::write",
        "a method of the lock token: the exclusive lock is held for as long as the token exists",
    ),
    (
        "src/services/roadmap_write_lock.rs",
        "RoadmapWriteLock::replace",
        "a method of the lock token: staging file plus rename, under the held lock",
    ),
    (
        "src/services/roadmap_write_lock.rs",
        "RoadmapWriteLock::remove",
        "a method of the lock token: deletes a fragment under the held lock",
    ),
    (
        "src/services/roadmap_write_lock.rs",
        "open_lock_file",
        "opens the LOCK file itself (outside git it is the sibling roadmap.yaml.lock); \
         it writes no roadmap content, and it is what taking the lock means",
    ),
];

fn sinks(files: &[(&str, &str)]) -> Vec<Sink> {
    let owned: Vec<(String, String)> = files
        .iter()
        .map(|(path, text)| ((*path).to_string(), (*text).to_string()))
        .collect();
    tainted_sinks(&owned).expect("fixture parses")
}

fn functions(files: &[(&str, &str)]) -> Vec<String> {
    sinks(files).into_iter().map(|s| s.function).collect()
}

// ------------------------------------------------------------ planted mutants

#[test]
fn roadmap_writer_gate_catches_the_migrate_shape() {
    let src = r#"
        fn handle(project: &std::path::Path) {
            let roadmap_path = project.join("docs/roadmaps/roadmap.yaml");
            write_migration(&roadmap_path, "x");
        }
        fn write_migration(roadmap_path: &std::path::Path, new_content: &str) {
            let backup_path = roadmap_path.with_extension("yaml.bak");
            std::fs::write(&backup_path, "old").ok();
            std::fs::write(roadmap_path, new_content).ok();
        }
    "#;
    let found = sinks(&[("src/a.rs", src)]);
    assert_eq!(
        found.len(),
        1,
        "one function, two sinks of one kind: {found:?}"
    );
    assert_eq!(found[0].function, "write_migration");
    assert_eq!(found[0].kind, "fs::write");
}

#[test]
fn roadmap_writer_gate_a_renamed_binding_is_still_caught() {
    // The mutant `pmat query --literal "fs::write(roadmap_path"` cannot see.
    let src = r#"
        fn handle(project: &std::path::Path) {
            let zz = project.join("docs/roadmaps/roadmap.yaml");
            let qq = zz.clone();
            put(&qq, "x");
        }
        fn put(yy: &std::path::Path, body: &str) {
            std::fs::write(yy, body).ok();
        }
    "#;
    assert_eq!(functions(&[("src/a.rs", src)]), vec!["put".to_string()]);
}

#[test]
fn roadmap_writer_gate_follows_consts_returns_fields_and_clap_defaults() {
    let src = r#"
        pub const DEFAULT_ROADMAP_PATH: &str = "docs/roadmaps/roadmap.yaml";
        struct Svc { target: std::path::PathBuf }
        impl Svc {
            fn new<P: AsRef<std::path::Path>>(p: P) -> Self { Self { target: p.as_ref().to_path_buf() } }
            fn flush(&self) { std::fs::File::create(&self.target).ok(); }
        }
        fn make() -> Svc { Svc::new(DEFAULT_ROADMAP_PATH) }
        fn roadmap_file(root: &std::path::Path) -> std::path::PathBuf { root.join("roadmap.yaml") }
        fn via_return(root: &std::path::Path) {
            std::fs::OpenOptions::new().write(true).open(roadmap_file(root)).ok();
        }
        enum Cmd {
            Sync { #[arg(long, default_value = "docs/roadmaps/roadmap.yaml")] roadmap: std::path::PathBuf },
            Fix { #[arg(long, default_value = "ROADMAP.md")] roadmap: std::path::PathBuf },
        }
        fn dispatch(cmd: Cmd) {
            match cmd { Cmd::Sync { roadmap: out } => { std::fs::rename("staging", &out).ok(); } _ => {} }
        }
        fn markdown(cmd: Cmd) {
            // Same field NAME, different subcommand, default ROADMAP.md: not a roadmap write.
            if let Cmd::Fix { roadmap } = cmd { std::fs::write(&roadmap, "- [x]").ok(); }
        }
        fn fragment_dir(roadmap: &std::path::Path) -> Option<std::path::PathBuf> {
            let dir = roadmap.parent()?.join("entries");
            dir.is_dir().then_some(dir)
        }
        fn fragment(root: &std::path::Path) {
            let Some(entries) = fragment_dir(&root.join(DEFAULT_ROADMAP_PATH)) else { return };
            let target = Some("X-1.yaml").map(|name| entries.join(name)).unwrap_or_default();
            std::fs::remove_file(target).ok();
        }
        fn formatted(root: &str) { std::fs::write(format!("{root}/docs/roadmaps/entries/X-1.yaml"), "").ok(); }
    "#;
    let mut found = functions(&[("src/a.rs", src)]);
    found.sort();
    assert_eq!(
        found,
        vec![
            "Svc::flush",
            "dispatch",
            "formatted",
            "fragment",
            "via_return"
        ]
    );
}

#[test]
fn roadmap_writer_gate_sees_every_sink_kind() {
    let src = r#"
        use std::fs::{self, write as put};
        fn all(root: &std::path::Path) {
            let p = root.join("docs/roadmaps");
            fs::copy("a", p.join("b")).ok();
            fs::hard_link("a", p.join("c")).ok();
            fs::remove_file(p.join("d")).ok();
            fs::remove_dir_all(p.join("e")).ok();
            put(p.join("f"), "").ok();
            std::os::unix::fs::symlink("a", p.join("g")).ok();
            tmp.persist(p.join("h")).ok();
            std::fs::File::create_new(p.join("i")).ok();
        }
    "#;
    let mut kinds: Vec<&str> = sinks(&[("src/a.rs", src)]).iter().map(|s| s.kind).collect();
    kinds.sort_unstable();
    assert_eq!(
        kinds,
        vec![
            "File::create_new",
            "NamedTempFile::persist",
            "fs::copy",
            "fs::hard_link",
            "fs::remove_dir_all",
            "fs::remove_file",
            "fs::write",
            "symlink",
        ]
    );
}

#[test]
fn roadmap_writer_gate_is_silent_on_unrelated_reads_and_tests() {
    let src = r#"
        fn unrelated(root: &std::path::Path) {
            std::fs::write(root.join(".pmat/metrics.json"), "{}").ok();
            let text = std::fs::read_to_string(root.join("docs/roadmaps/roadmap.yaml")).unwrap_or_default();
            std::fs::write(root.join("report.md"), text).ok();
            std::fs::OpenOptions::new().read(true).open(root.join("docs/roadmaps/roadmap.yaml")).ok();
        }
        #[cfg(test)]
        mod tests {
            #[test]
            fn fixture() { std::fs::write("docs/roadmaps/roadmap.yaml", "").ok(); }
        }
    "#;
    assert_eq!(functions(&[("src/a.rs", src)]), Vec::<String>::new());
}

#[test]
fn roadmap_writer_gate_crosses_files() {
    let a = r#"
        pub fn run(project: &std::path::Path) { crate::b::save_text(&project.join("docs/roadmaps/roadmap.yaml")); }
    "#;
    let b = r#"
        pub fn save_text(where_to: &std::path::Path) { std::fs::write(where_to, "").ok(); }
    "#;
    assert_eq!(
        functions(&[("src/a.rs", a), ("src/b.rs", b)]),
        vec!["save_text".to_string()]
    );
}

#[test]
fn roadmap_writer_gate_source_predicate() {
    assert!(is_source("docs/roadmaps/roadmap.yaml"));
    assert!(is_source("roadmap.yaml"));
    assert!(is_source("roadmaps"));
    assert!(!is_source("ROADMAP.md"));
    assert!(!is_source("ROADMAP.yaml"));
    assert!(!is_source("docs/execution/roadmap.md"));
    assert!(is_source("{}/docs/roadmaps/roadmap.yaml"));
    assert!(!is_source(
        "Run `pmat work init` to create docs/roadmaps/roadmap.yaml"
    ));
}

// ------------------------------------------------------------------ the tree

/// Every file this crate compiles OUTSIDE `cfg(test)`, found the way rustc finds
/// them: from the crate roots, through each `mod` declaration (honouring `#[path]`)
/// and each `include!` — at ANY depth, in an inline module, an impl or a function
/// body, not only at a file's top level — skipping every `#[cfg(test)]` item and
/// every file whose inner attributes say `#![cfg(test)]`. File names decide nothing:
/// `work_tests_part1.rs` is test code because of how it is reached.
///
/// An `include!` of `concat!(env!("OUT_DIR"), "/x.rs")` is resolved to the build
/// script's real output and read like any other file. Any other computed path fails
/// the walk: a file the gate cannot locate is not a file it has checked.
fn production_sources(root: &Path) -> Vec<(String, String)> {
    let mut walk = ModuleWalk::default();
    for crate_root in ["src/lib.rs", "src/bin/pmat.rs", "src/bin/pmat-agent.rs"] {
        let file = root.join(crate_root);
        let children = file
            .parent()
            .expect("a crate root has a directory")
            .to_path_buf();
        walk.file(root, &file, &children);
    }
    assert!(
        walk.unreadable.is_empty(),
        "the gate could not read {} compiled file(s):\n  {}",
        walk.unreadable.len(),
        walk.unreadable.join("\n  ")
    );
    walk.sources.into_values().collect()
}

#[derive(Default)]
struct ModuleWalk {
    sources: std::collections::BTreeMap<PathBuf, (String, String)>,
    unreadable: Vec<String>,
}

impl ModuleWalk {
    /// `file`, whose out-of-line child modules live in `children`.
    fn file(&mut self, root: &Path, file: &Path, children: &Path) {
        if self.sources.contains_key(file) {
            return;
        }
        let relative = file
            .strip_prefix(root)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/");
        let parsed = std::fs::read_to_string(file)
            .map_err(|e| e.to_string())
            .and_then(|text| {
                syn::parse_file(&text)
                    .map(|f| (text, f))
                    .map_err(|e| e.to_string())
            });
        let (text, syntax) = match parsed {
            Ok(ok) => ok,
            Err(e) => {
                self.unreadable.push(format!("{relative}: {e}"));
                return;
            }
        };
        if test_only(&syntax.attrs) {
            return;
        }
        self.sources
            .insert(file.to_path_buf(), (relative.clone(), text));
        let here = file
            .parent()
            .expect("a source file has a directory")
            .to_path_buf();
        let mut scan = Scan {
            here: &here,
            children: vec![children.to_path_buf()],
            found: Vec::new(),
        };
        syn::visit::Visit::visit_file(&mut scan, &syntax);
        for found in scan.found {
            match found {
                Found::File { file, children } => self.file(root, &file, &children),
                Found::Unresolved(what) => self
                    .unreadable
                    .push(format!("{relative}: include! of a computed path {what}")),
            }
        }
    }
}

/// What one file declares: files to read next, and includes the walk cannot locate.
enum Found {
    File { file: PathBuf, children: PathBuf },
    Unresolved(String),
}

/// A full visit of one file for `mod` and `include!`, `cfg(test)` pruned.
struct Scan<'p> {
    here: &'p Path,
    /// The directory out-of-line children of the innermost module live in.
    children: Vec<PathBuf>,
    found: Vec<Found>,
}

impl Scan<'_> {
    fn children(&self) -> &Path {
        self.children
            .last()
            .expect("the file's own module is on the stack")
    }

    fn include(&mut self, mac: &syn::Macro) {
        let children = self.children().to_path_buf();
        let found = match include_target(mac) {
            Some(Target::Relative(path)) => Found::File {
                file: self.here.join(path),
                children,
            },
            Some(Target::OutDir(name)) => Found::File {
                file: Path::new(env!("OUT_DIR")).join(name.trim_start_matches('/')),
                children,
            },
            None => Found::Unresolved(format!("{:?}", mac.tokens.to_string())),
        };
        self.found.push(found);
    }
}

impl<'ast> syn::visit::Visit<'ast> for Scan<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let attrs: &[syn::Attribute] = match item {
            syn::Item::Fn(i) => &i.attrs,
            syn::Item::Mod(i) => &i.attrs,
            syn::Item::Impl(i) => &i.attrs,
            syn::Item::Macro(i) => &i.attrs,
            syn::Item::Trait(i) => &i.attrs,
            syn::Item::Const(i) => &i.attrs,
            syn::Item::Static(i) => &i.attrs,
            _ => &[],
        };
        if !test_only(attrs) {
            syn::visit::visit_item(self, item);
        }
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if !test_only(&node.attrs) {
            syn::visit::visit_impl_item_fn(self, node);
        }
    }

    fn visit_impl_item_macro(&mut self, node: &'ast syn::ImplItemMacro) {
        if !test_only(&node.attrs) {
            syn::visit::visit_impl_item_macro(self, node);
        }
    }

    fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
        if !test_only(&node.attrs) {
            syn::visit::visit_stmt_macro(self, node);
        }
    }

    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        let name = module.ident.to_string();
        let explicit = path_attribute(&module.attrs);
        if module.content.is_some() {
            let inner = self.children().join(explicit.unwrap_or(name));
            self.children.push(inner);
            syn::visit::visit_item_mod(self, module);
            self.children.pop();
            return;
        }
        let children = self.children();
        let file = match explicit {
            Some(path) => self.here.join(path),
            None if children.join(format!("{name}.rs")).is_file() => {
                children.join(format!("{name}.rs"))
            }
            None => children.join(&name).join("mod.rs"),
        };
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let grandchildren = if stem == "mod" {
            file.parent().expect("mod.rs has a directory").to_path_buf()
        } else {
            file.with_extension("")
        };
        self.found.push(Found::File {
            file,
            children: grandchildren,
        });
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if mac.path.is_ident("include") {
            self.include(mac);
        }
    }
}

/// Where an `include!` points.
enum Target {
    /// `include!("file.rs")`, relative to the including file.
    Relative(String),
    /// `include!(concat!(env!("OUT_DIR"), "/file.rs"))`: the build script's output.
    OutDir(String),
}

fn include_target(mac: &syn::Macro) -> Option<Target> {
    if let Ok(name) = mac.parse_body::<syn::LitStr>() {
        return Some(Target::Relative(name.value()));
    }
    let syn::Expr::Macro(concat) = mac.parse_body::<syn::Expr>().ok()? else {
        return None;
    };
    if !concat.mac.path.is_ident("concat") {
        return None;
    }
    let parts = concat
        .mac
        .parse_body_with(syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)
        .ok()?;
    match parts.iter().collect::<Vec<_>>().as_slice() {
        [syn::Expr::Macro(env), syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(tail),
            ..
        })] if env.mac.path.is_ident("env")
            && env.mac.parse_body::<syn::LitStr>().ok()?.value() == "OUT_DIR" =>
        {
            Some(Target::OutDir(tail.value()))
        }
        _ => None,
    }
}

fn path_attribute(attrs: &[syn::Attribute]) -> Option<String> {
    attrs.iter().find_map(|attr| match &attr.meta {
        syn::Meta::NameValue(nv) if nv.path.is_ident("path") => match &nv.value {
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) => Some(s.value()),
            _ => None,
        },
        _ => None,
    })
}

fn tree_sinks() -> (usize, Vec<Sink>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sources = production_sources(&root);
    let found = tainted_sinks(&sources).expect("the gate reads every compiled file");
    (sources.len(), found)
}

#[test]
fn roadmap_writer_gate_every_roadmap_write_in_the_tree_goes_through_the_lock_token() {
    let (files, found) = tree_sinks();
    assert!(
        files > 1000,
        "only {files} compiled files read — the module walk is broken"
    );
    let violations: Vec<String> = found
        .iter()
        .filter(|sink| {
            !ALLOWED
                .iter()
                .any(|(file, function, _)| sink.file == *file && sink.function == *function)
        })
        .map(ToString::to_string)
        .collect();
    assert!(
        violations.is_empty(),
        "{} raw write(s) can land under docs/roadmaps/ without the RoadmapWriteLock token \
         (route each through RoadmapService, or a RoadmapWriteLock method):\n  {}",
        violations.len(),
        violations.join("\n  ")
    );
}

#[test]
fn roadmap_writer_gate_every_allowed_writer_is_still_reached() {
    // A stale allow-list entry is a hole waiting for a new function of that name,
    // and an allowed writer the taint no longer reaches means the SOURCES broke:
    // the gate would then pass a tree it cannot see into.
    let (_, found) = tree_sinks();
    for (file, function, _) in ALLOWED {
        assert!(
            found
                .iter()
                .any(|s| s.file == *file && s.function == *function),
            "{file} {function} is allowed but no roadmap write reaches it — the allow-list \
             is stale or the taint sources stopped matching the real writers"
        );
    }
}
