// Which crate a requested path belongs to.
// Included from cargo_dead_code_analyzer.rs - shares parent module scope.
// NO `use` imports or `#!` inner attributes allowed here.

/// What a `Cargo.toml` DECLARES — which is a different question from whether a
/// `Cargo.toml` is present.
///
/// A manifest holding `[workspace]` and no `[package]` (a *virtual* manifest)
/// declares a workspace and no crate at all: it has no targets, no `[lib]`
/// section and no `src/lib.rs` beside it, because a virtual manifest never
/// does. Reading those absences as evidence about a crate is reading evidence
/// about a crate that was never declared.
pub(crate) enum ManifestKind {
    /// A `[package]` table: this directory IS a crate, named where the manifest
    /// names it.
    Package { name: Option<String> },
    /// A `[workspace]` table and no `[package]`: a workspace, not a crate. Its
    /// `members` are where the crates actually are.
    WorkspaceOnly { members: Vec<String> },
    /// Neither table. Cargo rejects such a file outright ("manifest is missing
    /// either a [package] or a [workspace]"), so it declares no crate either.
    Neither,
}

/// Which crate — if any — governs a requested path.
///
/// Three outcomes, kept distinct because the caller's honest answer differs for
/// each: analyse the package, refuse because a workspace manifest declares no
/// package, refuse because there is no manifest at all. Collapsing the last two
/// into "no crate found" is what let a workspace root be described as "a
/// binary-only crate".
pub(crate) enum CrateRootResolution {
    /// The directory of the `Cargo.toml` whose `[package]` governs the path.
    Package {
        root: PathBuf,
        manifest: PathBuf,
        name: Option<String>,
    },
    /// The walk reached a workspace manifest before it found any package, so
    /// the path is inside the workspace and inside none of its members.
    WorkspaceOnly {
        manifest: PathBuf,
        members: Vec<String>,
    },
    /// No `Cargo.toml` anywhere above the path.
    NoManifest,
}

impl CrateRootResolution {
    /// The crate root, or `None` when no package governs the path.
    pub(crate) fn package_root(&self) -> Option<&Path> {
        match self {
            Self::Package { root, .. } => Some(root),
            Self::WorkspaceOnly { .. } | Self::NoManifest => None,
        }
    }

    /// Why no package governs `path`, in one clause naming the evidence.
    /// `None` when one does.
    ///
    /// Written once and shared by every caller that has to explain itself — the
    /// CLI's refusal, the cargo engine's library verdict and the
    /// multi-language engine's — so the three cannot describe the same tree
    /// three different ways.
    pub(crate) fn no_package_reason(&self, path: &Path) -> Option<String> {
        match self {
            Self::Package { .. } => None,
            Self::WorkspaceOnly { manifest, members } => Some(format!(
                "the nearest Cargo.toml above it, {}, is a WORKSPACE manifest — a \
                 [workspace] table with no [package] — so it declares no crate, no \
                 targets and no library, and {} is inside none of its member packages \
                 ({})",
                manifest.display(),
                path.display(),
                describe_members(members)
            )),
            Self::NoManifest => Some(format!(
                "walking up from {} to the filesystem root found no Cargo.toml at all",
                path.display()
            )),
        }
    }
}

/// The members clause of a workspace, for a message a reader can act on.
fn describe_members(members: &[String]) -> String {
    if members.is_empty() {
        "it lists no members".to_string()
    } else {
        members.join(", ")
    }
}

/// The `Cargo.toml` that governs `path`, and what it declares.
///
/// `cargo check` used to be run in the directory this command was POINTED AT,
/// and every "is this a library" question was asked of that directory too.
/// Point the command at a SUBDIRECTORY of a crate and both went wrong at once:
/// no `Cargo.toml` was found there, so the crate read as binary-only, `--lib`
/// was dropped from the cargo invocation, `cargo check --bins` on a lib-only
/// crate matched no target and compiled NOTHING — and the command published
/// `dead_functions: 0, dead_classes: 0` at exit 0 over a subdirectory holding a
/// dead private function and a never-constructed struct, under a
/// `library_target.detail` asserting the crate "declares no [lib] and there is
/// no src/lib.rs" about a crate whose `src/lib.rs` was two directories up.
///
/// A path inside a crate is not a crate; it is a VIEW of one. Finding the crate
/// is what makes the view measurable — rustc cannot type-check half a crate —
/// so it is found here, once, and the cargo invocation, the library verdict and
/// the scope the report is restricted to are all taken from it.
///
/// The nearest PACKAGE wins, so a package inside a workspace resolves to the
/// package rather than to the workspace root. A workspace manifest is not a
/// package and is not skipped over silently either: the walk STOPS there and
/// says so, because a workspace root is cargo's own boundary — a file under it
/// that no member contains belongs to no package, and a package further up the
/// filesystem does not own it. Walking past the boundary would answer with a
/// crate that does not contain the path; stopping and reporting
/// [`CrateRootResolution::WorkspaceOnly`] lets the caller refuse instead of
/// inventing a verdict, which is exactly the zero this analyzer keeps
/// publishing when it guesses.
pub(crate) fn resolve_crate_root(path: &Path) -> CrateRootResolution {
    let absolute = absolutize(path);
    let start: PathBuf = if absolute.is_file() {
        match absolute.parent() {
            Some(parent) => parent.to_path_buf(),
            None => return CrateRootResolution::NoManifest,
        }
    } else {
        absolute
    };

    for dir in start.ancestors() {
        let manifest = dir.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        match classify_manifest(&manifest) {
            ManifestKind::Package { name } => {
                return CrateRootResolution::Package {
                    root: dir.to_path_buf(),
                    manifest,
                    name,
                }
            }
            ManifestKind::WorkspaceOnly { members } => {
                return CrateRootResolution::WorkspaceOnly { manifest, members }
            }
            // Not a manifest cargo can use at all. It declares no crate and no
            // workspace boundary, so it is no reason to stop looking for one.
            ManifestKind::Neither => continue,
        }
    }
    CrateRootResolution::NoManifest
}

/// The directory holding the `Cargo.toml` whose `[package]` governs `path`, or
/// `None` when no package does.
///
/// The thin form of [`resolve_crate_root`], for the callers that only need the
/// directory. A caller that has to EXPLAIN the `None` — and every user-facing
/// one does — must use [`resolve_crate_root`] instead: "no package here" and
/// "no manifest anywhere" are different facts about the tree.
pub(crate) fn enclosing_crate_root(path: &Path) -> Option<PathBuf> {
    resolve_crate_root(path)
        .package_root()
        .map(Path::to_path_buf)
}

/// What the manifest at `manifest` declares.
///
/// A manifest that cannot be READ is treated as a package: this function only
/// exists to recognise the case where the file is present and demonstrably
/// declares no package, and an unreadable file demonstrates nothing. Cargo
/// fails on it in its own way, which is the pre-existing behaviour.
pub(crate) fn classify_manifest(manifest: &Path) -> ManifestKind {
    let Ok(content) = std::fs::read_to_string(manifest) else {
        return ManifestKind::Package { name: None };
    };
    classify_manifest_content(&content)
}

/// [`classify_manifest`] over the manifest's text, so the rule is testable
/// without a filesystem.
///
/// A hand-rolled table scan rather than a TOML parse: `toml` is an optional
/// dependency of this crate (`standard-deps`), and the sibling
/// `project_has_library` reads the same file the same way. Only table headers
/// and two keys are needed, and both are read from the table they belong to —
/// `[workspace.package]` is a workspace table, not a package one, and reading
/// it as `[package]` would make every modern workspace root a crate again.
pub(crate) fn classify_manifest_content(content: &str) -> ManifestKind {
    if declares_table(content, "package") {
        return ManifestKind::Package {
            name: table_value(content, "package", "name").and_then(|v| quoted_scalar(&v)),
        };
    }
    if declares_table(content, "workspace") {
        return ManifestKind::WorkspaceOnly {
            members: table_value(content, "workspace", "members")
                .map(|v| quoted_strings(&v))
                .unwrap_or_default(),
        };
    }
    ManifestKind::Neither
}

/// Does the manifest declare a table whose FIRST segment is `root`?
///
/// The first segment is the whole rule: `[package.metadata.docs]` is part of a
/// package declaration, and `[workspace.package]` — the version-inheritance
/// table every modern monorepo root carries — is part of a workspace one. Read
/// as "contains the word package" instead, every such root becomes a crate
/// again and takes the contradicted verdict with it.
fn declares_table(content: &str, root: &str) -> bool {
    content
        .lines()
        .filter_map(|line| table_header(line.trim()))
        .any(|header| table_root(&header) == root)
}

/// The raw text of `key` in the top-level table `table`, with an array read
/// whole across however many lines it spans.
fn table_value(content: &str, table: &str, key: &str) -> Option<String> {
    let mut current = String::new();
    let mut lines = content.lines();
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        if let Some(header) = table_header(trimmed) {
            current = header;
            continue;
        }
        let Some((found, value)) = manifest_key_value(trimmed) else {
            continue;
        };
        if current != table || found != key {
            continue;
        }
        // `members = [` opens an array that may close several lines down.
        // Reading only the first line yields the members that happened to fit
        // on it, which in a formatted manifest is none of them.
        let mut block = value.to_string();
        while block.contains('[') && !block.contains(']') {
            match lines.next() {
                Some(next) => {
                    block.push('\n');
                    block.push_str(next);
                }
                None => break,
            }
        }
        return Some(block);
    }
    None
}

/// The table name in a `[header]` / `[[header]]` line, trimmed of the brackets
/// and the whitespace TOML permits inside them. `None` for a comment, a
/// key/value line or anything else that is not a header.
fn table_header(trimmed: &str) -> Option<String> {
    if trimmed.starts_with('#') {
        return None;
    }
    let inner = trimmed.strip_prefix('[')?;
    let inner = inner.strip_prefix('[').unwrap_or(inner);
    let end = inner.find(']')?;
    Some(inner[..end].trim().to_string())
}

/// The first dotted segment of a table name.
fn table_root(header: &str) -> &str {
    header.split('.').next().unwrap_or_default().trim()
}

/// A `key = value` line, split at the first `=`.
fn manifest_key_value(trimmed: &str) -> Option<(&str, &str)> {
    let (key, value) = trimmed.split_once('=')?;
    Some((key.trim(), value.trim()))
}

/// The contents of a quoted TOML scalar, basic or literal.
fn quoted_scalar(value: &str) -> Option<String> {
    for quote in ['"', '\''] {
        if let Some(rest) = value.strip_prefix(quote) {
            if let Some(end) = rest.find(quote) {
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}

/// Every quoted string in `block`, in order — the members of a `members = [...]`
/// array.
fn quoted_strings(block: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = block;
    while let Some(open) = rest.find(['"', '\'']) {
        let quote = rest.as_bytes()[open] as char;
        let after = &rest[open + 1..];
        match after.find(quote) {
            Some(close) => {
                out.push(after[..close].to_string());
                rest = &after[close + 1..];
            }
            None => break,
        }
    }
    out
}

/// `path` as an absolute, symlink-resolved path.
///
/// `canonicalize` is the accurate answer and it fails on a path that is not
/// there, so a missing path falls back to being made absolute against the
/// working directory: a caller asking about a directory that does not exist
/// must still get an answer it can compare, not a panic.
pub(crate) fn absolutize(path: &Path) -> PathBuf {
    if let Ok(canonical) = std::fs::canonicalize(path) {
        return canonical;
    }
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir().map_or_else(|_| path.to_path_buf(), |cwd| cwd.join(path))
}
