// CB-2120: declared committed derived artifacts are fresh (GH-1641).
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// The class: a committed file that a command derives from the tree (a manifest,
// a pin list, a registry) is edited around, or two clean merges leave it wrong,
// and the first thing to notice is a red PR or a red default branch.
//
// A repository declares each such file in `.pmat-derived.toml`:
//
//     [[artifact]]
//     path = "docs/registry.txt"
//     generate = ["scripts/registry.sh", "--print"]   # argv; prints the file
//     timeout_secs = 600                               # optional
//
// `generate` runs repository code, so it runs ONLY when CB-2120 is named in
// `--checks`. A plain `comply check` validates the declaration and skips.

/// The declaration file, at the project root.
const DERIVED_FILE: &str = ".pmat-derived.toml";
const DERIVED_DEFAULT_TIMEOUT_SECS: u64 = 600;
const DERIVED_NAME: &str = "CB-2120: Derived Artifacts Fresh";

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DerivedDecl {
    artifact: Vec<DerivedArtifact>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DerivedArtifact {
    path: String,
    generate: Vec<String>,
    timeout_secs: Option<u64>,
}

fn derived_row(status: CheckStatus, message: String) -> ComplianceCheck {
    let severity = match status {
        CheckStatus::Fail | CheckStatus::Warn => Severity::Warning,
        CheckStatus::Pass | CheckStatus::Skip => Severity::Info,
    };
    ComplianceCheck {
        name: DERIVED_NAME.into(),
        status,
        message,
        severity,
    }
}

/// Parse and validate the declaration. Anything unmeasurable is an error.
fn parse_derived_decl(raw: &str) -> Result<Vec<DerivedArtifact>, String> {
    let decl: DerivedDecl = toml::from_str(raw).map_err(|e| e.to_string())?;
    if decl.artifact.is_empty() {
        return Err("no [[artifact]] is declared".to_string());
    }
    for a in &decl.artifact {
        let relative = !a.path.is_empty()
            && std::path::Path::new(&a.path)
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_)));
        if !relative {
            return Err(format!(
                "path {:?} must be relative to the root, with no `.` or `..`",
                a.path
            ));
        }
        if a.generate.is_empty() {
            return Err(format!("{}: `generate` is empty", a.path));
        }
    }
    Ok(decl.artifact)
}

/// The declared artifacts, or the row that ends the check: Skip when there is no
/// declaration, Fail when it cannot be read. The declaration is read from HEAD,
/// like every other input the verdict depends on.
fn load_derived_decl(project_path: &Path) -> Result<Vec<DerivedArtifact>, ComplianceCheck> {
    let unreadable = |e: String| derived_row(CheckStatus::Fail, format!("{DERIVED_FILE}: {e}"));
    match head_blob(project_path, DERIVED_FILE) {
        Ok(Some(raw)) => match String::from_utf8(raw) {
            Ok(text) => parse_derived_decl(&text).map_err(unreadable),
            Err(e) => Err(unreadable(e.to_string())),
        },
        Ok(None) if project_path.join(DERIVED_FILE).exists() => {
            Err(unreadable("not in HEAD; commit the declaration".to_string()))
        }
        Ok(None) => Err(derived_row(
            CheckStatus::Skip,
            format!("no {DERIVED_FILE} — no derived artifact is declared"),
        )),
        Err(e) => Err(unreadable(e)),
    }
}

/// CB-2120: does every declared derived file equal what its generator prints?
pub(crate) fn check_derived_fresh(project_path: &Path, selected: &[String]) -> ComplianceCheck {
    let artifacts = match load_derived_decl(project_path) {
        Ok(artifacts) => artifacts,
        Err(row) => return row,
    };
    if !selected.iter().any(|s| s.eq_ignore_ascii_case("cb-2120")) {
        return derived_row(
            CheckStatus::Skip,
            format!(
                "{} derived artifact(s) declared; generators run repository code and were \
                 not run. Verify with: pmat comply check --checks CB-2120",
                artifacts.len()
            ),
        );
    }
    let findings: Vec<String> = artifacts
        .iter()
        .filter_map(|a| derived_finding(project_path, a).err())
        .collect();
    if findings.is_empty() {
        derived_row(
            CheckStatus::Pass,
            format!("{} derived artifact(s) match their generators", artifacts.len()),
        )
    } else {
        derived_row(CheckStatus::Fail, findings.join("; "))
    }
}

/// `Ok` when the committed file is what the generator prints from HEAD.
///
/// The verdict is about HEAD, the bytes a push sends. The generator therefore
/// runs in an export of HEAD, never in the working tree, so uncommitted edits to
/// its inputs cannot make a consistent HEAD look stale, nor a stale HEAD look fresh.
fn derived_finding(root: &Path, a: &DerivedArtifact) -> Result<(), String> {
    let path = &a.path;
    let committed = committed_bytes(root, path)?;
    let export = head_export(root, path).map_err(|e| format!("{path}: {e}"))?;
    let artifact = export.cwd.join(path);
    // A link cannot be fingerprinted for a rewrite within one clock tick (no std
    // API sets a link's own mtime), so a symlink artifact is refused outright.
    if stamp(&artifact).is_some_and(|s| s.symlink) {
        return Err(format!("{path}: is a symlink in HEAD; a derived artifact must be a regular file"));
    }
    let before = (read_artifact(&artifact), stamp(&artifact));
    let generated = run_generator(&export.cwd, a, &export.tmp, &export.scratch)
        .map_err(|e| format!("{path}: generator {e}"))?;
    if (read_artifact(&artifact), stamp(&artifact)) != before {
        return Err(format!(
            "{path}: the generator rewrote the artifact; it must print to stdout"
        ));
    }
    if generated == committed {
        return Ok(());
    }
    if read_artifact(&root.join(path)).as_deref() == Some(generated.as_slice()) {
        return Err(format!("{path}: regenerated but not committed"));
    }
    Err(format!(
        "{path}: stale, first difference at line {}; regenerate with `{} > {path}` and commit",
        first_differing_line(&committed, &generated),
        a.generate.join(" ")
    ))
}

/// The bytes of a file, or of a symlink's target, as the export holds them.
fn read_artifact(p: &Path) -> Option<Vec<u8>> {
    match std::fs::symlink_metadata(p) {
        Ok(m) if m.file_type().is_symlink() => std::fs::read_link(p).ok().map(|t| link_bytes(&t)),
        _ => std::fs::read(p).ok(),
    }
}

#[cfg(unix)]
fn link_bytes(target: &Path) -> Vec<u8> {
    use std::os::unix::ffi::OsStrExt;
    target.as_os_str().as_bytes().to_vec()
}

#[cfg(not(unix))]
fn link_bytes(target: &Path) -> Vec<u8> {
    target.to_string_lossy().into_owned().into_bytes()
}

/// What a generator can change about an artifact without changing its bytes:
/// the link itself, size, mtime, and on Unix the inode and ctime. Taken without
/// following symlinks, so a replaced link is a change, not its target's.
#[derive(PartialEq)]
struct Stamp {
    symlink: bool,
    len: u64,
    mtime: Option<std::time::SystemTime>,
    #[cfg(unix)]
    inode: (u64, u64, i64, i64),
}

fn stamp(p: &Path) -> Option<Stamp> {
    let m = std::fs::symlink_metadata(p).ok()?;
    Some(Stamp {
        symlink: m.file_type().is_symlink(),
        len: m.len(),
        mtime: m.modified().ok(),
        #[cfg(unix)]
        inode: {
            use std::os::unix::fs::MetadataExt;
            (m.dev(), m.ino(), m.ctime(), m.ctime_nsec())
        },
    })
}

/// A git command for `root` that ignores repository overrides from the caller's
/// environment: a hook sets `GIT_DIR` and `GIT_INDEX_FILE` for its own repository.
fn git_at(root: &Path) -> std::process::Command {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("-C").arg(root);
    scrub_git_env(&mut cmd);
    cmd
}

fn scrub_git_env(cmd: &mut std::process::Command) {
    for var in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_OBJECT_DIRECTORY", "GIT_COMMON_DIR"] {
        cmd.env_remove(var);
    }
}

fn git_bytes(root: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = git_at(root)
        .args(args)
        .output()
        .map_err(|e| format!("could not run git: {e}"))?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(format!(
            "git {} failed: {}",
            args[0],
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// The bytes HEAD holds at `rel` (relative to `root`), or `None` when HEAD has
/// no such file. Git's own blob bytes: no attribute or eol filter applies.
fn head_blob(root: &Path, rel: &str) -> Result<Option<Vec<u8>>, String> {
    let out = git_at(root)
        .args(["cat-file", "blob", &format!("HEAD:./{rel}")])
        .output()
        .map_err(|e| format!("{rel}: could not run git: {e}"))?;
    Ok(out.status.success().then_some(out.stdout))
}

/// The bytes HEAD holds for `path`: what a push would send.
fn committed_bytes(root: &Path, path: &str) -> Result<Vec<u8>, String> {
    head_blob(root, path)?.ok_or_else(|| format!("{path}: not in HEAD; commit the derived file"))
}

/// A throwaway export of the whole of HEAD. Every blob is written from the object
/// store, so no `.gitattributes`, eol, smudge or working-tree file can change
/// what the generator reads. `cwd` is `root`'s place inside the export.
struct HeadExport {
    _dir: tempfile::TempDir,
    /// The export's own directory: git's ceiling for the generator.
    scratch: std::path::PathBuf,
    /// The generator's TMPDIR: strictly below `scratch`, so `git -C "$TMPDIR"`
    /// stays under the ceiling and cannot climb into the enclosing repository.
    tmp: std::path::PathBuf,
    cwd: std::path::PathBuf,
}

fn head_export(root: &Path, artifact: &str) -> Result<HeadExport, String> {
    // A relative TMPDIR would resolve against `root` in git and inside the repo.
    let base = std::env::temp_dir();
    let base = if base.is_absolute() {
        base
    } else {
        std::env::current_dir()
            .map_err(|e| format!("could not make export dir: {e}"))?
            .join(base)
    };
    let dir = tempfile::Builder::new()
        .prefix("cb2120-")
        .tempdir_in(&base)
        .map_err(|e| format!("could not make export dir: {e}"))?;
    // Canonical, so PWD and TMPDIR carry no `.` or `..` segments.
    let canonical = std::fs::canonicalize(dir.path())
        .map_err(|e| format!("could not make export dir: {e}"))?;
    let tree = canonical.join("tree");
    std::fs::create_dir(&tree).map_err(|e| format!("could not make export dir: {e}"))?;
    // A subproject's prefix ends in `/`: strip it, or `tree.join` keeps it and
    // PWD gains a trailing slash, which breaks `${PWD##*/}` in the generator.
    let mut prefix_bytes = git_bytes(root, &["rev-parse", "--show-prefix"])?;
    while matches!(prefix_bytes.last(), Some(b'\n') | Some(b'/')) {
        prefix_bytes.pop();
    }
    let prefix = path_from_bytes(&prefix_bytes)?;
    let listing = git_bytes(root, &["ls-tree", "-r", "-z", "--full-tree", "HEAD"])?;
    let entries = parse_ls_tree(&listing)?;
    // The artifact is pinned to the epoch so any write by the generator shows.
    let pin = prefix.join(artifact);
    write_blobs(root, &entries, &tree, &pin)?;
    // `join("")` appends a `/`: the repository root must use `tree` itself.
    let cwd = if prefix.as_os_str().is_empty() {
        tree.clone()
    } else {
        tree.join(&prefix)
    };
    std::fs::create_dir_all(&cwd).map_err(|e| format!("could not make export dir: {e}"))?;
    let scratch = canonical;
    // GIT_CEILING_DIRECTORIES is colon-separated: a colon would split it and
    // unbound the ceiling. Fail closed rather than judge outside the export.
    if scratch.to_string_lossy().contains(':') {
        return Err("the temp directory path contains ':'; set TMPDIR to a path without one".into());
    }
    let tmp = scratch.join("tmp");
    std::fs::create_dir(&tmp).map_err(|e| format!("could not make export tmp: {e}"))?;
    Ok(HeadExport { _dir: dir, scratch, tmp, cwd })
}

struct TreeEntry {
    mode: String,
    path: std::path::PathBuf,
    oid: String,
}

/// The blobs of `git ls-tree -r -z`. Submodule commits are not exported. Paths
/// are kept as bytes on Unix: a non-UTF-8 name elsewhere in HEAD is not an error.
fn parse_ls_tree(listing: &[u8]) -> Result<Vec<TreeEntry>, String> {
    let mut entries = Vec::new();
    for record in listing.split(|&b| b == 0).filter(|r| !r.is_empty()) {
        let tab = record
            .iter()
            .position(|&b| b == b'\t')
            .ok_or_else(|| "malformed ls-tree record".to_string())?;
        let meta = std::str::from_utf8(&record[..tab])
            .map_err(|e| format!("malformed ls-tree record: {e}"))?;
        let mut fields = meta.split(' ');
        if let (Some(mode), Some("blob"), Some(oid)) = (fields.next(), fields.next(), fields.next()) {
            entries.push(TreeEntry {
                mode: mode.to_string(),
                path: path_from_bytes(&record[tab + 1..])?,
                oid: oid.to_string(),
            });
        }
    }
    Ok(entries)
}

#[cfg(unix)]
fn path_from_bytes(bytes: &[u8]) -> Result<std::path::PathBuf, String> {
    use std::os::unix::ffi::OsStrExt;
    Ok(std::path::PathBuf::from(std::ffi::OsStr::from_bytes(bytes)))
}

#[cfg(not(unix))]
fn path_from_bytes(bytes: &[u8]) -> Result<std::path::PathBuf, String> {
    std::str::from_utf8(bytes)
        .map(std::path::PathBuf::from)
        .map_err(|e| format!("a path in HEAD is not UTF-8: {e}"))
}

/// Stream every blob through one `git cat-file --batch`, writing each to `tree`.
fn write_blobs(
    root: &Path,
    entries: &[TreeEntry],
    tree: &Path,
    pin: &Path,
) -> Result<(), String> {
    use std::io::Write;
    let mut child = git_at(root)
        .args(["cat-file", "--batch"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not run git: {e}"))?;
    let mut stdin = child.stdin.take().ok_or("no stdin for git cat-file")?;
    let stdout = child.stdout.take().ok_or("no stdout from git cat-file")?;
    let oids: Vec<String> = entries.iter().map(|e| e.oid.clone()).collect();
    let feeder = std::thread::spawn(move || {
        for oid in oids {
            if writeln!(stdin, "{oid}").is_err() {
                break;
            }
        }
    });
    let mut reader = std::io::BufReader::new(stdout);
    let result = read_blobs(&mut reader, entries, tree, pin);
    // On an early error git may still be writing: kill it so the feeder unblocks.
    let _ = child.kill();
    let _ = child.wait();
    let _ = feeder.join();
    result
}

fn read_blobs(
    reader: &mut impl std::io::BufRead,
    entries: &[TreeEntry],
    tree: &Path,
    pin: &Path,
) -> Result<(), String> {
    let io = |e: std::io::Error| format!("could not write export: {e}");
    for entry in entries {
        let mut header = String::new();
        reader.read_line(&mut header).map_err(io)?;
        let size = match header.split_whitespace().collect::<Vec<_>>().as_slice() {
            [_, "blob", size] => size
                .parse::<usize>()
                .map_err(|e| format!("bad cat-file header: {e}"))?,
            _ => {
                return Err(format!(
                    "{}: not a blob in the object store",
                    entry.path.display()
                ))
            }
        };
        let mut content = vec![0u8; size];
        reader.read_exact(&mut content).map_err(io)?;
        let mut newline = [0u8; 1];
        reader.read_exact(&mut newline).map_err(io)?;
        let dest = tree.join(&entry.path);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(io)?;
        }
        write_entry(&dest, &entry.mode, &content, entry.path == pin).map_err(io)?;
    }
    Ok(())
}

/// Write one blob. The artifact (`pin`) is given the epoch mtime, so a
/// generator's write to it is visible as a change. Other files keep their
/// normal mtimes: some generators reject pre-1980 timestamps.
fn write_entry(dest: &Path, mode: &str, content: &[u8], pin: bool) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        if mode == "120000" {
            use std::os::unix::ffi::OsStrExt;
            return std::os::unix::fs::symlink(std::ffi::OsStr::from_bytes(content), dest);
        }
    }
    let mut file = std::fs::File::create(dest)?;
    std::io::Write::write_all(&mut file, content)?;
    if pin {
        file.set_modified(std::time::SystemTime::UNIX_EPOCH)?;
    }
    #[cfg(unix)]
    {
        if mode == "100755" {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))?;
        }
    }
    let _ = mode;
    Ok(())
}

/// Run the generator at the root and return its stdout. Output goes to files, so
/// a large output cannot block the child while the timeout loop waits on it.
fn run_generator(
    root: &Path,
    a: &DerivedArtifact,
    tmp: &Path,
    scratch: &Path,
) -> Result<Vec<u8>, String> {
    use std::io::{Read, Seek};
    let io = |e: std::io::Error| format!("output could not be captured: {e}");
    let mut stdout = tempfile::tempfile().map_err(io)?;
    let mut stderr = tempfile::tempfile().map_err(io)?;
    let (program, args) = a.generate.split_first().ok_or("is empty")?;
    let mut command = std::process::Command::new(program);
    command
        .args(args)
        .current_dir(root)
        .stdin(std::process::Stdio::null())
        .stdout(stdout.try_clone().map_err(io)?)
        .stderr(stderr.try_clone().map_err(io)?);
    scrub_git_env(&mut command);
    // A relative or in-repo TMPDIR would resolve against the child's cwd, and git
    // would find the enclosing repository: the export directory bounds both.
    // PWD is inherited from the caller, usually the live working tree; a generator
    // that reads it must see the export, the directory it actually runs in.
    command
        .env("TMPDIR", tmp)
        .env("PWD", root)
        .env("GIT_CEILING_DIRECTORIES", scratch);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("could not start `{program}`: {e}"))?;
    let limit = std::time::Duration::from_secs(
        a.timeout_secs.unwrap_or(DERIVED_DEFAULT_TIMEOUT_SECS),
    );
    let started = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(io)? {
            break status;
        }
        if started.elapsed() > limit {
            // The child leads its own process group: kill the generator's
            // subprocesses too, so none outlives the timeout.
            // The shell's kill builtin signals the group; the child leads it
            // (process_group above). The external /usr/bin/kill rejects low PGIDs
            // on procps, so the builtin is used. Output is silenced: a child that
            // just exited makes the group gone, which is not an error here.
            #[cfg(unix)]
            {
                let _ = std::process::Command::new("sh")
                    .args(["-c", "kill -s KILL -- \"$1\"", "_", &format!("-{}", child.id())])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status();
            }
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("timed out after {}s", limit.as_secs()));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    if !status.success() {
        let mut text = String::new();
        stderr.rewind().map_err(io)?;
        let _ = stderr.read_to_string(&mut text);
        let last = text.lines().rev().find(|l| !l.trim().is_empty());
        return Err(format!("exited with {status}: {}", last.unwrap_or("no stderr")));
    }
    let mut out = Vec::new();
    stdout.rewind().map_err(io)?;
    stdout.read_to_end(&mut out).map_err(io)?;
    Ok(out)
}

/// 1-based number of the first line that differs.
fn first_differing_line(a: &[u8], b: &[u8]) -> usize {
    let mut left = a.split_inclusive(|&c| c == b'\n');
    let mut right = b.split_inclusive(|&c| c == b'\n');
    let mut line = 1;
    loop {
        match (left.next(), right.next()) {
            (Some(l), Some(r)) if l == r => line += 1,
            _ => return line,
        }
    }
}
