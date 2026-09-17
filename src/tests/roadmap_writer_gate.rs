#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-1385 (#1385) — the roadmap WRITER GATE engine: which raw filesystem writes
//! in pmat's own `src/` can land on a path under `docs/roadmaps/`.
//!
//! `pmat work migrate` rewrote `roadmap.yaml` with a bare `std::fs::write`, without
//! the repository lock every `RoadmapService` writer takes. The query that found it,
//! `pmat query --literal "fs::write(roadmap_path"`, reads a VARIABLE NAME: rename the
//! binding and the write disappears from it while still racing every other writer.
//! This engine reads what is written instead of what it is called.
//!
//! It is a taint analysis over the syntax tree of every file the crate compiles
//! outside `cfg(test)`:
//!
//! - SOURCES: a string literal that IS a path naming `docs/roadmaps` or
//!   `roadmap.yaml` (or the bare `roadmaps` segment) — in a `let`, a `const`, a
//!   `format!` template, or a clap attribute defaulting a field. Prose that merely
//!   mentions the roadmap (it contains whitespace) is not a source.
//! - PROPAGATION, never by a variable's name: `let`, assignment, and every pattern
//!   (`if let`, `match`, `for`) — so a renamed binding is the same binding; the
//!   path-preserving methods (`join`, `with_extension`, `parent`, `clone`, …);
//!   `PathBuf::from`/`Path::new`; a path-shaped `format!`; a struct field
//!   initialised or assigned from a tainted value, keyed by its OWNER type; and
//!   ACROSS FUNCTIONS — a tainted argument taints the callee's parameter and a
//!   function returning a tainted value taints its call sites, to a fixpoint.
//! - CALL RESOLUTION: `Type::f` and `Self::f` by owner; a method by the receiver's
//!   type where that is known (`self`, a typed parameter, `let x = Type::ctor(..)`),
//!   otherwise only when exactly one type defines a method of that name and arity;
//!   a free function by the module its qualifier names, or its own file, else every
//!   free function of that name.
//! - SINKS, the serialisation site itself: `fs::write`, `File::create[_new]`,
//!   `fs::rename`/`copy`/`hard_link` (destination), `fs::remove_file`/
//!   `remove_dir_all`, `symlink`, a method `.open(..)` on an option chain that is not
//!   read-only, `NamedTempFile::persist`, and a bare `write(..)` imported from `fs`.
//!
//! What it cannot see, stated rather than implied: a path that reaches a sink only
//! from runtime data (a CLI argument with no roadmap default, an environment
//! variable, a config file); a method call on a receiver whose type is unknown when
//! two or more types define that method; a field read on a receiver whose type is
//! unknown when two or more types declare that field; dispatch through a trait
//! object or a function pointer; an `OpenOptions` value configured in one statement
//! and opened in another; and code outside this crate's `src/`.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use syn::punctuated::Punctuated;
use syn::visit::{self, Visit};
use syn::{Attribute, Block, Expr, FnArg, Item, Member, Pat, Signature, Stmt, Token, UseTree};

/// One raw filesystem write whose destination can be a path under `docs/roadmaps/`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Sink {
    /// Repository-relative path of the file the write is in.
    pub(crate) file: String,
    /// `Type::method` for a method, the bare name for a free function.
    pub(crate) function: String,
    /// Which primitive writes, e.g. `fs::write`.
    pub(crate) kind: &'static str,
}

impl std::fmt::Display for Sink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}: {}", self.file, self.function, self.kind)
    }
}

/// Every tainted sink in `sources` (`(repository-relative path, file text)`).
///
/// # Errors
///
/// Every file that does not parse, one per line. Never skipped: an unread file is
/// not a clean one.
pub(crate) fn tainted_sinks(sources: &[(String, String)]) -> Result<Vec<Sink>, String> {
    let mut parsed = Vec::with_capacity(sources.len());
    let mut unreadable = Vec::new();
    for (path, text) in sources {
        match syn::parse_file(text) {
            Ok(file) => parsed.push((path.as_str(), file)),
            Err(e) => unreadable.push(format!("{path}: {e}")),
        }
    }
    if !unreadable.is_empty() {
        return Err(unreadable.join("\n"));
    }
    let mut collector = Collector::default();
    for (path, file) in &parsed {
        collector.file = path;
        collector.visit_file(file);
    }
    Ok(fixpoint(&collector))
}

/// `true` for a string literal that IS a path naming the roadmap directory or file.
///
/// A literal with whitespace is prose — an error message, help text, a hook
/// template that mentions `docs/roadmaps/roadmap.yaml` — and prose is not where a
/// write lands. Measured on this tree: with prose counted, the messages that name
/// the roadmap tainted the fields they were stored in, and 350 unrelated writes
/// with them.
pub(crate) fn is_source(text: &str) -> bool {
    path_like(text)
        && (text.contains("docs/roadmaps") || text.contains("roadmap.yaml") || text == "roadmaps")
}

/// No whitespace: a path or a path template (`{}/docs/roadmaps/roadmap.yaml`).
fn path_like(text: &str) -> bool {
    !text.is_empty() && !text.chars().any(char::is_whitespace)
}

/// `#[test]`, `#[tokio::test]`, or a `#[cfg(..)]` whose predicate requires `test`.
pub(crate) fn is_test_only(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        if path.segments.last().is_some_and(|s| s.ident == "test") {
            return true;
        }
        path.is_ident("cfg")
            && attr
                .parse_args::<syn::Meta>()
                .is_ok_and(|predicate| requires_test(&predicate))
    })
}

/// Whether a `cfg` predicate can only hold under `cfg(test)`: `test`,
/// `all(.., test, ..)`, or `any(..)` of such predicates. `not(..)` never does.
fn requires_test(predicate: &syn::Meta) -> bool {
    match predicate {
        syn::Meta::Path(path) => path.is_ident("test"),
        syn::Meta::List(list) => {
            let Ok(children) =
                list.parse_args_with(Punctuated::<syn::Meta, Token![,]>::parse_terminated)
            else {
                return false;
            };
            if list.path.is_ident("all") {
                children.iter().any(requires_test)
            } else if list.path.is_ident("any") {
                !children.is_empty() && children.iter().all(requires_test)
            } else {
                false
            }
        }
        syn::Meta::NameValue(_) => false,
    }
}

// ------------------------------------------------------------------ collection

/// One function body to analyse.
struct FnDef<'a> {
    file: &'a str,
    /// The module a qualifier would name: the file stem, or the directory of a
    /// `mod.rs`/`lib.rs`.
    module: String,
    owner: Option<String>,
    name: String,
    has_self: bool,
    /// The identifiers each positional parameter (excluding `self`) binds.
    params: Vec<BTreeSet<String>>,
    /// The type each positional parameter names, where it is a plain path type.
    param_types: Vec<Option<String>>,
    /// The type this function constructs, when it returns `Self` or its owner.
    constructs: Option<String>,
    body: &'a Block,
}

impl FnDef<'_> {
    fn qualified(&self) -> String {
        match &self.owner {
            Some(owner) => format!("{owner}::{}", self.name),
            None => self.name.clone(),
        }
    }
}

#[derive(Default)]
struct Collector<'a> {
    file: &'a str,
    owner: Option<String>,
    fns: Vec<FnDef<'a>>,
    consts: Vec<(String, &'a Expr)>,
    /// `(owner, field)` a clap attribute defaults to a roadmap path. The owner is the
    /// struct, or `Enum::Variant` for a subcommand variant — never the bare field
    /// name: `maintain roadmap --roadmap` defaults to `ROADMAP.md` while another
    /// subcommand's `--roadmap` defaults to `docs/roadmaps/roadmap.yaml`.
    clap_fields: BTreeSet<(String, String)>,
    /// The struct or `Enum::Variant` whose fields are being visited.
    fields_of: Option<String>,
    /// The enum whose variants are being visited.
    enum_of: Option<String>,
    /// Field name -> every struct declaring it.
    field_owners: BTreeMap<String, BTreeSet<String>>,
    /// Per file: local name -> the `std::fs` function it imports.
    fs_imports: BTreeMap<&'a str, BTreeMap<String, String>>,
    /// Function name -> every function of that name.
    by_name: BTreeMap<String, Vec<usize>>,
}

impl<'a> Collector<'a> {
    fn push_fn(&mut self, attrs: &[Attribute], sig: &'a Signature, body: &'a Block) {
        if is_test_only(attrs) {
            return;
        }
        let mut params = Vec::new();
        let mut param_types = Vec::new();
        let mut has_self = false;
        for input in &sig.inputs {
            match input {
                FnArg::Receiver(_) => has_self = true,
                FnArg::Typed(typed) => {
                    let mut names = BTreeSet::new();
                    pattern_idents(&typed.pat, &mut names);
                    params.push(names);
                    param_types.push(type_name(&typed.ty));
                }
            }
        }
        let constructs = match &sig.output {
            syn::ReturnType::Type(_, ty) => self.owner.clone().filter(|owner| {
                let named = type_names(ty);
                named.contains("Self") || named.contains(owner.as_str())
            }),
            syn::ReturnType::Default => None,
        };
        let index = self.fns.len();
        self.by_name
            .entry(sig.ident.to_string())
            .or_default()
            .push(index);
        self.fns.push(FnDef {
            file: self.file,
            module: module_of(self.file),
            owner: self.owner.clone(),
            name: sig.ident.to_string(),
            has_self,
            params,
            param_types,
            constructs,
            body,
        });
    }
}

impl<'a> Visit<'a> for Collector<'a> {
    fn visit_item(&mut self, item: &'a Item) {
        if item_attrs(item).is_some_and(is_test_only) {
            return;
        }
        visit::visit_item(self, item);
    }

    fn visit_item_impl(&mut self, node: &'a syn::ItemImpl) {
        let outer = std::mem::replace(&mut self.owner, type_name(&node.self_ty));
        visit::visit_item_impl(self, node);
        self.owner = outer;
    }

    fn visit_item_fn(&mut self, node: &'a syn::ItemFn) {
        let outer = self.owner.take();
        self.push_fn(&node.attrs, &node.sig, &node.block);
        visit::visit_item_fn(self, node);
        self.owner = outer;
    }

    fn visit_impl_item_fn(&mut self, node: &'a syn::ImplItemFn) {
        if is_test_only(&node.attrs) {
            return;
        }
        self.push_fn(&node.attrs, &node.sig, &node.block);
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'a syn::TraitItemFn) {
        if let Some(body) = &node.default {
            self.push_fn(&node.attrs, &node.sig, body);
        }
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_item_const(&mut self, node: &'a syn::ItemConst) {
        self.consts.push((node.ident.to_string(), &node.expr));
        visit::visit_item_const(self, node);
    }

    fn visit_item_static(&mut self, node: &'a syn::ItemStatic) {
        self.consts.push((node.ident.to_string(), &node.expr));
        visit::visit_item_static(self, node);
    }

    fn visit_item_struct(&mut self, node: &'a syn::ItemStruct) {
        for field in &node.fields {
            if let Some(ident) = &field.ident {
                self.field_owners
                    .entry(ident.to_string())
                    .or_default()
                    .insert(node.ident.to_string());
            }
        }
        let outer = self.fields_of.replace(node.ident.to_string());
        visit::visit_item_struct(self, node);
        self.fields_of = outer;
    }

    fn visit_item_enum(&mut self, node: &'a syn::ItemEnum) {
        let outer = self.enum_of.replace(node.ident.to_string());
        visit::visit_item_enum(self, node);
        self.enum_of = outer;
    }

    fn visit_variant(&mut self, node: &'a syn::Variant) {
        let owner = match &self.enum_of {
            Some(enumeration) => format!("{enumeration}::{}", node.ident),
            None => node.ident.to_string(),
        };
        let outer = self.fields_of.replace(owner);
        visit::visit_variant(self, node);
        self.fields_of = outer;
    }

    fn visit_field(&mut self, node: &'a syn::Field) {
        if let (Some(ident), Some(owner)) = (&node.ident, &self.fields_of) {
            if node.attrs.iter().any(defaults_to_a_roadmap_path) {
                self.clap_fields.insert((owner.clone(), ident.to_string()));
            }
        }
        visit::visit_field(self, node);
    }

    fn visit_item_use(&mut self, node: &'a syn::ItemUse) {
        let mut found = BTreeMap::new();
        fs_use_names(&node.tree, false, &mut found);
        self.fs_imports.entry(self.file).or_default().extend(found);
        visit::visit_item_use(self, node);
    }
}

/// `std::fs::write` imported under any name, as `local name -> fs function`.
fn fs_use_names(tree: &UseTree, under_fs: bool, out: &mut BTreeMap<String, String>) {
    match tree {
        UseTree::Path(path) => fs_use_names(&path.tree, under_fs || path.ident == "fs", out),
        UseTree::Name(name) if under_fs => {
            out.insert(name.ident.to_string(), name.ident.to_string());
        }
        UseTree::Rename(rename) if under_fs => {
            out.insert(rename.rename.to_string(), rename.ident.to_string());
        }
        UseTree::Group(group) => {
            for item in &group.items {
                fs_use_names(item, under_fs, out);
            }
        }
        _ => {}
    }
}

fn item_attrs(item: &Item) -> Option<&[Attribute]> {
    Some(match item {
        Item::Fn(i) => &i.attrs,
        Item::Mod(i) => &i.attrs,
        Item::Impl(i) => &i.attrs,
        Item::Const(i) => &i.attrs,
        Item::Static(i) => &i.attrs,
        Item::Struct(i) => &i.attrs,
        Item::Enum(i) => &i.attrs,
        Item::Trait(i) => &i.attrs,
        Item::Use(i) => &i.attrs,
        Item::Macro(i) => &i.attrs,
        _ => return None,
    })
}

/// A clap/structopt attribute with a string literal that is a roadmap path.
fn defaults_to_a_roadmap_path(attr: &Attribute) -> bool {
    let is_cli = ["arg", "clap", "structopt"]
        .iter()
        .any(|name| attr.path().is_ident(name));
    let syn::Meta::List(list) = &attr.meta else {
        return false;
    };
    is_cli
        && list
            .tokens
            .to_string()
            .split('"')
            .skip(1)
            .step_by(2)
            .any(is_source)
}

fn module_of(file: &str) -> String {
    let path = std::path::Path::new(file);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if matches!(stem, "mod" | "lib") {
        path.parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string()
    } else {
        stem.to_string()
    }
}

/// The last segment of a plain path type, through references.
fn type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
        syn::Type::Reference(r) => type_name(&r.elem),
        syn::Type::Paren(p) => type_name(&p.elem),
        _ => None,
    }
}

/// Every path segment named anywhere in a type (`Result<Option<Self>>` names `Self`).
fn type_names(ty: &syn::Type) -> BTreeSet<String> {
    #[derive(Default)]
    struct Names(BTreeSet<String>);
    impl<'ast> Visit<'ast> for Names {
        fn visit_path_segment(&mut self, node: &'ast syn::PathSegment) {
            self.0.insert(node.ident.to_string());
            visit::visit_path_segment(self, node);
        }
    }
    let mut names = Names::default();
    names.visit_type(ty);
    names.0
}

fn pattern_idents(pat: &Pat, out: &mut BTreeSet<String>) {
    match pat {
        Pat::Ident(p) => {
            out.insert(p.ident.to_string());
            if let Some((_, sub)) = &p.subpat {
                pattern_idents(sub, out);
            }
        }
        Pat::Struct(p) => p.fields.iter().for_each(|f| pattern_idents(&f.pat, out)),
        Pat::TupleStruct(p) => p.elems.iter().for_each(|e| pattern_idents(e, out)),
        Pat::Tuple(p) => p.elems.iter().for_each(|e| pattern_idents(e, out)),
        Pat::Slice(p) => p.elems.iter().for_each(|e| pattern_idents(e, out)),
        Pat::Or(p) => p.cases.iter().for_each(|e| pattern_idents(e, out)),
        Pat::Reference(p) => pattern_idents(&p.pat, out),
        Pat::Type(p) => pattern_idents(&p.pat, out),
        Pat::Paren(p) => pattern_idents(&p.pat, out),
        _ => {}
    }
}

// -------------------------------------------------------------------- fixpoint

#[derive(Default)]
struct Summary {
    consts: BTreeSet<String>,
    /// `(owner type, field name)` known to hold a tainted value.
    fields: BTreeSet<(String, String)>,
    /// `(function index, parameter position)` known to receive a tainted value.
    params: BTreeSet<(usize, usize)>,
    /// Functions that can return a tainted value.
    returns: BTreeSet<usize>,
    sinks: BTreeSet<Sink>,
}

/// A worklist fixpoint: a function is re-analysed only when something it reads
/// changed — one of its own parameters, a field it names, or a function it calls.
fn fixpoint(collector: &Collector<'_>) -> Vec<Sink> {
    let mentions = mentions_index(collector);
    let mut summary = Summary::default();
    loop {
        let before = summary.consts.len();
        for (name, expr) in &collector.consts {
            if FnPass::new(&summary, collector, None).tainted(expr) {
                summary.consts.insert(name.clone());
            }
        }
        if summary.consts.len() == before {
            break;
        }
    }
    let mut queue: VecDeque<usize> = (0..collector.fns.len()).collect();
    let mut queued = vec![true; collector.fns.len()];
    while let Some(index) = queue.pop_front() {
        queued[index] = false;
        let out = analyse(index, collector, &summary);
        for woken in out.merge_into(&mut summary, collector, &mentions) {
            if !queued[woken] {
                queued[woken] = true;
                queue.push_back(woken);
            }
        }
    }
    summary.sinks.into_iter().collect()
}

/// Word -> every function whose body names it as a callee, a method, or a field.
fn mentions_index(collector: &Collector<'_>) -> BTreeMap<String, Vec<usize>> {
    let mut index: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (position, def) in collector.fns.iter().enumerate() {
        let mut words = Mentions::default();
        words.visit_block(def.body);
        for word in words.0 {
            index.entry(word).or_default().push(position);
        }
    }
    index
}

#[derive(Default)]
struct Mentions(BTreeSet<String>);

impl<'ast> Visit<'ast> for Mentions {
    fn visit_item(&mut self, _: &'ast Item) {}

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Expr::Path(path) = &*node.func {
            if let Some(last) = path.path.segments.last() {
                self.0.insert(last.ident.to_string());
            }
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.0.insert(node.method.to_string());
        visit::visit_expr_method_call(self, node);
    }

    fn visit_member(&mut self, node: &'ast Member) {
        if let Member::Named(id) = node {
            self.0.insert(id.to_string());
        }
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Some(args) = macro_args(node) {
            for arg in &args {
                self.visit_expr(arg);
            }
        }
    }
}

/// One function, to its own fixpoint over its locals.
fn analyse(index: usize, collector: &Collector<'_>, summary: &Summary) -> Out {
    let def = &collector.fns[index];
    let mut pass = FnPass::new(summary, collector, Some(index));
    for (position, names) in def.params.iter().enumerate() {
        if summary.params.contains(&(index, position)) {
            pass.locals.extend(names.iter().cloned());
        }
        if let Some(ty) = &def.param_types[position] {
            for name in names {
                pass.types.insert(name.clone(), ty.clone());
            }
        }
    }
    loop {
        let known = pass.locals.len() + pass.types.len();
        pass.out = Out::default();
        pass.visit_block(def.body);
        if block_tail(def.body).is_some_and(|tail| pass.tainted(tail)) {
            pass.out.returns.insert(index);
        }
        if pass.locals.len() + pass.types.len() == known {
            return pass.out;
        }
    }
}

#[derive(Default)]
struct Out {
    fields: BTreeSet<(String, String)>,
    params: BTreeSet<(usize, usize)>,
    returns: BTreeSet<usize>,
    sinks: BTreeSet<Sink>,
}

impl Out {
    /// Fold into `summary`; returns the functions whose inputs just changed.
    fn merge_into(
        self,
        summary: &mut Summary,
        collector: &Collector<'_>,
        mentions: &BTreeMap<String, Vec<usize>>,
    ) -> Vec<usize> {
        let mut woken = Vec::new();
        let mut wake = |word: &str| woken.extend(mentions.get(word).into_iter().flatten().copied());
        for field in self.fields {
            if summary.fields.insert(field.clone()) {
                wake(&field.1);
            }
        }
        for index in self.returns {
            if summary.returns.insert(index) {
                wake(&collector.fns[index].name);
            }
        }
        for (index, position) in self.params {
            if summary.params.insert((index, position)) {
                woken.push(index);
            }
        }
        summary.sinks.extend(self.sinks);
        woken
    }
}

fn block_tail(block: &Block) -> Option<&Expr> {
    match block.stmts.last() {
        Some(Stmt::Expr(expr, None)) => Some(expr),
        _ => None,
    }
}

// ------------------------------------------------------------ one function body

/// Methods whose result is (a view of) their receiver's path.
const PRESERVING: &[&str] = &[
    "as_deref",
    "as_os_str",
    "as_path",
    "as_ref",
    "borrow",
    "canonicalize",
    "clone",
    "cloned",
    "context",
    "display",
    "expect",
    "into",
    "into_os_string",
    "into_owned",
    "join",
    "map_err",
    "ok",
    "ok_or",
    "ok_or_else",
    "parent",
    "to_owned",
    "to_path_buf",
    "to_str",
    "to_string",
    "to_string_lossy",
    "unwrap",
    "unwrap_or",
    "unwrap_or_default",
    "unwrap_or_else",
    "with_context",
    "with_extension",
    "with_file_name",
];

/// Methods whose result can be their ARGUMENT's path: `base.join(roadmap)`,
/// `dir.is_dir().then_some(dir)`, `maybe.unwrap_or(default_roadmap)`.
const CARRYING: &[&str] = &["join", "then_some", "unwrap_or", "or"];

/// Combinators whose result is what their closure returns.
const MAPPING: &[&str] = &[
    "and_then",
    "map",
    "map_or",
    "map_or_else",
    "then",
    "unwrap_or_else",
];

/// Constructors and wrappers whose result carries their argument's path.
const WRAPPING: &[&str] = &["Some", "Ok", "from", "new", "absolute", "canonicalize"];

struct FnPass<'s, 'a> {
    summary: &'s Summary,
    collector: &'s Collector<'a>,
    current: Option<usize>,
    locals: BTreeSet<String>,
    /// Local name -> the type it is known to hold.
    types: BTreeMap<String, String>,
    out: Out,
}

impl<'s, 'a> FnPass<'s, 'a> {
    fn new(summary: &'s Summary, collector: &'s Collector<'a>, current: Option<usize>) -> Self {
        Self {
            summary,
            collector,
            current,
            locals: BTreeSet::new(),
            types: BTreeMap::new(),
            out: Out::default(),
        }
    }

    fn def(&self) -> Option<&'s FnDef<'a>> {
        self.current.map(|index| &self.collector.fns[index])
    }

    fn tainted(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Lit(lit) => matches!(&lit.lit, syn::Lit::Str(s) if is_source(&s.value())),
            Expr::Path(path) => self.path_tainted(&path.path),
            Expr::Reference(e) => self.tainted(&e.expr),
            Expr::Paren(e) => self.tainted(&e.expr),
            Expr::Group(e) => self.tainted(&e.expr),
            Expr::Try(e) => self.tainted(&e.expr),
            Expr::Unary(e) => self.tainted(&e.expr),
            Expr::Cast(e) => self.tainted(&e.expr),
            Expr::Await(e) => self.tainted(&e.base),
            Expr::Field(e) => self.field_tainted(e),
            Expr::MethodCall(call) => self.method_call_tainted(call),
            Expr::Call(call) => self.call_tainted(call),
            Expr::Macro(mac) => self.macro_tainted(&mac.mac),
            Expr::Block(e) => block_tail(&e.block).is_some_and(|t| self.tainted(t)),
            Expr::If(e) => {
                block_tail(&e.then_branch).is_some_and(|t| self.tainted(t))
                    || e.else_branch
                        .as_ref()
                        .is_some_and(|(_, other)| self.tainted(other))
            }
            Expr::Match(e) => e.arms.iter().any(|arm| self.tainted(&arm.body)),
            Expr::Tuple(e) => e.elems.iter().any(|x| self.tainted(x)),
            Expr::Array(e) => e.elems.iter().any(|x| self.tainted(x)),
            _ => false,
        }
    }

    fn path_tainted(&self, path: &syn::Path) -> bool {
        let Some(last) = path.segments.last() else {
            return false;
        };
        let name = last.ident.to_string();
        (path.segments.len() == 1 && self.locals.contains(&name))
            || self.summary.consts.contains(&name)
    }

    fn field_tainted(&self, field: &syn::ExprField) -> bool {
        let Member::Named(id) = &field.member else {
            return false;
        };
        let name = id.to_string();
        self.field_owner(&field.base, &name).is_some_and(|owner| {
            let key = (owner, name);
            self.collector.clap_fields.contains(&key) || self.summary.fields.contains(&key)
        })
    }

    /// The type owning `base.name`: the receiver's type where known, else the one
    /// struct declaring a field of that name.
    fn field_owner(&self, base: &Expr, name: &str) -> Option<String> {
        if let Some(ty) = self.receiver_type(base) {
            return Some(ty);
        }
        let owners = self.collector.field_owners.get(name)?;
        (owners.len() == 1)
            .then(|| owners.iter().next().cloned())
            .flatten()
    }

    /// `self`, a parameter or local of a known type, through `&`, `*` and parens.
    fn receiver_type(&self, receiver: &Expr) -> Option<String> {
        match receiver {
            Expr::Path(path) if path.path.is_ident("self") => self.def()?.owner.clone(),
            Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|ident| self.types.get(&ident.to_string()).cloned()),
            Expr::Reference(r) => self.receiver_type(&r.expr),
            Expr::Unary(u) => self.receiver_type(&u.expr),
            Expr::Paren(p) => self.receiver_type(&p.expr),
            _ => None,
        }
    }

    fn method_call_tainted(&self, call: &syn::ExprMethodCall) -> bool {
        let name = call.method.to_string();
        if PRESERVING.contains(&name.as_str()) && self.tainted(&call.receiver) {
            return true;
        }
        if CARRYING.contains(&name.as_str()) && call.args.iter().any(|a| self.tainted(a)) {
            return true;
        }
        if MAPPING.contains(&name.as_str()) && call.args.iter().any(|a| self.closure_tainted(a)) {
            return true;
        }
        self.method_callees(&call.receiver, &name, call.args.len())
            .into_iter()
            .any(|(index, _)| self.summary.returns.contains(&index))
    }

    /// A closure whose body yields a tainted value — `.map(|name| dir.join(name))`.
    fn closure_tainted(&self, arg: &Expr) -> bool {
        match arg {
            Expr::Closure(closure) => self.tainted(&closure.body),
            _ => false,
        }
    }

    fn call_tainted(&self, call: &syn::ExprCall) -> bool {
        let Expr::Path(func) = &*call.func else {
            return false;
        };
        let Some(last) = func.path.segments.last() else {
            return false;
        };
        let name = last.ident.to_string();
        if WRAPPING.contains(&name.as_str()) && call.args.iter().any(|a| self.tainted(a)) {
            return true;
        }
        self.path_callees(&func.path, call.args.len())
            .into_iter()
            .any(|(index, _)| self.summary.returns.contains(&index))
    }

    /// `format!`/`concat!` build a path only when their literal parts look like one:
    /// `format!("{}/entries", dir)` does, `format!("wrote {}", dir.display())` is prose.
    fn macro_tainted(&self, mac: &syn::Macro) -> bool {
        let is_builder = ["format", "concat"]
            .iter()
            .any(|name| mac.path.is_ident(name));
        let Some(args) = macro_args(mac).filter(|_| is_builder) else {
            return false;
        };
        let prose = args.iter().any(|arg| {
            matches!(arg, Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) if !path_like(&s.value()))
        });
        !prose && args.iter().any(|a| self.tainted(a))
    }

    /// In-crate functions a path call can reach, with the argument shift (1 when a
    /// method is called as `Type::method(receiver, ..)`).
    fn path_callees(&self, path: &syn::Path, arity: usize) -> Vec<(usize, usize)> {
        let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        let Some(name) = segments.last() else {
            return Vec::new();
        };
        let Some(candidates) = self.collector.by_name.get(name) else {
            return Vec::new();
        };
        let qualifier = segments.len().checked_sub(2).map(|i| segments[i].as_str());
        let owner = match qualifier {
            Some("Self") => self.def().and_then(|d| d.owner.clone()),
            Some(q) if q.starts_with(char::is_uppercase) => Some(q.to_string()),
            _ => None,
        };
        let shaped: Vec<(usize, usize)> = candidates
            .iter()
            .filter(|index| self.collector.fns[**index].owner == owner)
            .filter_map(|index| {
                let f = &self.collector.fns[*index];
                if !f.has_self && f.params.len() == arity {
                    Some((*index, 0))
                } else if f.has_self && f.params.len() + 1 == arity {
                    Some((*index, 1))
                } else {
                    None
                }
            })
            .collect();
        if owner.is_some() {
            return shaped;
        }
        self.prefer_module(shaped, qualifier)
    }

    /// Among free functions of one name: those in the module a qualifier names, or,
    /// unqualified, those in this file; every one of them when neither narrows.
    fn prefer_module(
        &self,
        shaped: Vec<(usize, usize)>,
        qualifier: Option<&str>,
    ) -> Vec<(usize, usize)> {
        let narrowed: Vec<(usize, usize)> = shaped
            .iter()
            .copied()
            .filter(|(index, _)| {
                let f = &self.collector.fns[*index];
                match qualifier {
                    Some(module) if !matches!(module, "crate" | "super" | "self") => {
                        f.module == module
                    }
                    _ => self.def().is_some_and(|d| d.file == f.file),
                }
            })
            .collect();
        if narrowed.is_empty() {
            shaped
        } else {
            narrowed
        }
    }

    /// In-crate methods a `receiver.name(..)` call can reach: the receiver type's,
    /// when known; otherwise only an UNAMBIGUOUS name — one type defines it. A
    /// path-preserving name (`join`, `clone`, …) is the standard library's.
    fn method_callees(&self, receiver: &Expr, name: &str, arity: usize) -> Vec<(usize, usize)> {
        if PRESERVING.contains(&name) {
            return Vec::new();
        }
        let Some(candidates) = self.collector.by_name.get(name) else {
            return Vec::new();
        };
        let shaped: Vec<usize> = candidates
            .iter()
            .copied()
            .filter(|index| {
                let f = &self.collector.fns[*index];
                f.has_self && f.params.len() == arity
            })
            .collect();
        let owners: BTreeSet<&Option<String>> = shaped
            .iter()
            .map(|index| &self.collector.fns[*index].owner)
            .collect();
        let keep: Vec<usize> = match self.receiver_type(receiver) {
            Some(ty) => shaped
                .into_iter()
                .filter(|index| self.collector.fns[*index].owner.as_deref() == Some(ty.as_str()))
                .collect(),
            None if owners.len() == 1 => shaped,
            None => Vec::new(),
        };
        keep.into_iter().map(|index| (index, 0)).collect()
    }

    /// The type a `let` initialiser constructs: `Type::ctor(..)`, `Self::ctor(..)`,
    /// or `Type { .. }`.
    fn constructed_type(&self, init: &Expr) -> Option<String> {
        match init {
            Expr::Call(call) => {
                let Expr::Path(func) = &*call.func else {
                    return None;
                };
                self.path_callees(&func.path, call.args.len())
                    .into_iter()
                    .find_map(|(index, _)| self.collector.fns[index].constructs.clone())
            }
            Expr::Struct(s) => s.path.segments.last().map(|seg| seg.ident.to_string()),
            Expr::Try(t) => self.constructed_type(&t.expr),
            Expr::MethodCall(m) if PRESERVING.contains(&m.method.to_string().as_str()) => {
                self.constructed_type(&m.receiver)
            }
            _ => None,
        }
    }

    fn bind(&mut self, pat: &Pat) {
        pattern_idents(pat, &mut self.locals);
    }

    /// Whether the struct or variant a pattern names defaults `field` to a roadmap
    /// path: `Enum::Variant` exactly, or — for a bare `Variant { .. }` brought in by
    /// a glob import — any enum's variant of that name.
    fn clap_default(&self, segments: &[String], field: &str) -> bool {
        let (exact, suffix) = match segments {
            [.., enumeration, variant] => (format!("{enumeration}::{variant}"), None),
            [single] => (single.clone(), Some(format!("::{single}"))),
            [] => return false,
        };
        self.collector.clap_fields.iter().any(|(owner, name)| {
            name == field
                && (owner == &exact || suffix.as_deref().is_some_and(|s| owner.ends_with(s)))
        })
    }

    /// Struct patterns bind a clap field defaulted to a roadmap path whatever the
    /// scrutinee, e.g. a subcommand destructured as `Spec::Sync { roadmap_path, .. }`.
    fn bind_fields(&mut self, pat: &Pat) {
        match pat {
            Pat::Struct(p) => {
                let segments: Vec<String> = p
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                for field in &p.fields {
                    match &field.member {
                        Member::Named(id) if self.clap_default(&segments, &id.to_string()) => {
                            self.bind(&field.pat);
                        }
                        _ => self.bind_fields(&field.pat),
                    }
                }
            }
            Pat::TupleStruct(p) => p.elems.iter().for_each(|e| self.bind_fields(e)),
            Pat::Tuple(p) => p.elems.iter().for_each(|e| self.bind_fields(e)),
            Pat::Or(p) => p.cases.iter().for_each(|e| self.bind_fields(e)),
            Pat::Reference(p) => self.bind_fields(&p.pat),
            Pat::Type(p) => self.bind_fields(&p.pat),
            Pat::Ident(p) => {
                if let Some((_, sub)) = &p.subpat {
                    self.bind_fields(sub);
                }
            }
            _ => {}
        }
    }

    fn record_sink(&mut self, kind: &'static str) {
        let Some(def) = self.def() else {
            return;
        };
        self.out.sinks.insert(Sink {
            file: def.file.to_string(),
            function: def.qualified(),
            kind,
        });
    }

    fn record_params(&mut self, callees: Vec<(usize, usize)>, args: &Punctuated<Expr, Token![,]>) {
        for (position, arg) in args.iter().enumerate() {
            if !self.tainted(arg) {
                continue;
            }
            for (index, shift) in &callees {
                if let Some(param) = position.checked_sub(*shift) {
                    self.out.params.insert((*index, param));
                }
            }
        }
    }

    fn call_sink(&self, path: &syn::Path) -> Option<(&'static str, usize)> {
        let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        match segments.as_slice() {
            [single] => {
                let imports = self.collector.fs_imports.get(self.def()?.file)?;
                sink_kind("fs", imports.get(single)?)
            }
            [.., previous, last] => sink_kind(previous, last),
            [] => None,
        }
    }
}

fn sink_kind(previous: &str, last: &str) -> Option<(&'static str, usize)> {
    Some(match (previous, last) {
        ("fs", "write") => ("fs::write", 0),
        ("File", "create") => ("File::create", 0),
        ("File", "create_new") => ("File::create_new", 0),
        ("fs", "rename") => ("fs::rename", 1),
        ("fs", "copy") => ("fs::copy", 1),
        ("fs", "hard_link") => ("fs::hard_link", 1),
        ("fs", "remove_file") => ("fs::remove_file", 0),
        ("fs", "remove_dir_all") => ("fs::remove_dir_all", 0),
        (_, "symlink" | "symlink_file") => ("symlink", 1),
        _ => return None,
    })
}

/// `.open(path)` on an option chain that is not read-only, or `.persist(path)`.
fn method_sink(call: &syn::ExprMethodCall) -> Option<(&'static str, usize)> {
    match call.method.to_string().as_str() {
        "open" if !read_only_chain(&call.receiver) => Some(("OpenOptions::open", 0)),
        "persist" | "persist_noclobber" => Some(("NamedTempFile::persist", 0)),
        _ => None,
    }
}

/// An option chain that sets `read` and none of the options that write.
fn read_only_chain(receiver: &Expr) -> bool {
    let mut reads = false;
    let mut current = receiver;
    loop {
        match current {
            Expr::MethodCall(call) => {
                match call.method.to_string().as_str() {
                    "write" | "append" | "create" | "create_new" | "truncate" => return false,
                    "read" => reads = true,
                    _ => {}
                }
                current = &call.receiver;
            }
            Expr::Reference(r) => current = &r.expr,
            Expr::Paren(p) => current = &p.expr,
            _ => return reads,
        }
    }
}

fn macro_args(mac: &syn::Macro) -> Option<Punctuated<Expr, Token![,]>> {
    mac.parse_body_with(Punctuated::<Expr, Token![,]>::parse_terminated)
        .ok()
}

impl<'ast> Visit<'ast> for FnPass<'_, '_> {
    fn visit_item(&mut self, _: &'ast Item) {
        // A nested item is its own function, analysed on its own.
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        if let Some(init) = &node.init {
            if self.tainted(&init.expr) {
                self.bind(&node.pat);
            }
            let declared = match &node.pat {
                Pat::Type(typed) => type_name(&typed.ty),
                _ => None,
            };
            if let Some(ty) = declared.or_else(|| self.constructed_type(&init.expr)) {
                let mut names = BTreeSet::new();
                pattern_idents(&node.pat, &mut names);
                for name in names {
                    self.types.insert(name, ty.clone());
                }
            }
        }
        self.bind_fields(&node.pat);
        visit::visit_local(self, node);
    }

    fn visit_expr_let(&mut self, node: &'ast syn::ExprLet) {
        if self.tainted(&node.expr) {
            self.bind(&node.pat);
        }
        self.bind_fields(&node.pat);
        visit::visit_expr_let(self, node);
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        let tainted = self.tainted(&node.expr);
        for arm in &node.arms {
            if tainted {
                self.bind(&arm.pat);
            }
            self.bind_fields(&arm.pat);
        }
        visit::visit_expr_match(self, node);
    }

    fn visit_expr_for_loop(&mut self, node: &'ast syn::ExprForLoop) {
        if self.tainted(&node.expr) {
            self.bind(&node.pat);
        }
        visit::visit_expr_for_loop(self, node);
    }

    fn visit_expr_assign(&mut self, node: &'ast syn::ExprAssign) {
        if self.tainted(&node.right) {
            match &*node.left {
                Expr::Path(path) => {
                    if let Some(ident) = path.path.get_ident() {
                        self.locals.insert(ident.to_string());
                    }
                }
                Expr::Field(field) => {
                    if let Member::Named(id) = &field.member {
                        let name = id.to_string();
                        if let Some(owner) = self.field_owner(&field.base, &name) {
                            self.out.fields.insert((owner, name));
                        }
                    }
                }
                _ => {}
            }
        }
        visit::visit_expr_assign(self, node);
    }

    fn visit_expr_struct(&mut self, node: &'ast syn::ExprStruct) {
        let named = node.path.segments.last().map(|s| s.ident.to_string());
        let owner = match named.as_deref() {
            Some("Self") => self.def().and_then(|d| d.owner.clone()),
            other => other.map(ToString::to_string),
        };
        if let Some(owner) = owner {
            for field in &node.fields {
                if let Member::Named(id) = &field.member {
                    if self.tainted(&field.expr) {
                        self.out.fields.insert((owner.clone(), id.to_string()));
                    }
                }
            }
        }
        visit::visit_expr_struct(self, node);
    }

    fn visit_expr_return(&mut self, node: &'ast syn::ExprReturn) {
        if let (Some(index), Some(expr)) = (self.current, &node.expr) {
            if self.tainted(expr) {
                self.out.returns.insert(index);
            }
        }
        visit::visit_expr_return(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Expr::Path(func) = &*node.func {
            if let Some((kind, position)) = self.call_sink(&func.path) {
                if node
                    .args
                    .iter()
                    .nth(position)
                    .is_some_and(|a| self.tainted(a))
                {
                    self.record_sink(kind);
                }
            }
            let callees = self.path_callees(&func.path, node.args.len());
            self.record_params(callees, &node.args);
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if let Some((kind, position)) = method_sink(node) {
            if node
                .args
                .iter()
                .nth(position)
                .is_some_and(|a| self.tainted(a))
            {
                self.record_sink(kind);
            }
        }
        let name = node.method.to_string();
        if name == "push" && node.args.iter().any(|a| self.tainted(a)) {
            if let Expr::Path(path) = &*node.receiver {
                if let Some(ident) = path.path.get_ident() {
                    self.locals.insert(ident.to_string());
                }
            }
        }
        let callees = self.method_callees(&node.receiver, &name, node.args.len());
        self.record_params(callees, &node.args);
        visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Some(args) = macro_args(node) {
            for arg in &args {
                self.visit_expr(arg);
            }
        }
    }
}
