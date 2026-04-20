//! Hooks Install Lifecycle — `pmat hooks install` round-trip
//!
//! Walks a reader through the full install → inspect → uninstall cycle for
//! the pmat git hook suite, inside an ephemeral `tempfile::tempdir()` so the
//! example is safe to run from any checkout without mutating its hooks.
//!
//! Phases:
//!
//!   1. Spin up an empty git repo (`git init -b main`) with a trivial
//!      `Cargo.toml` so `pmat hooks install` has a Rust project to target.
//!   2. Invoke `pmat hooks install` and surface both stdout and exit status.
//!   3. List `.git/hooks/` so the reader can see which hook files were
//!      written and which are executable (the "x" bit signals they are
//!      active — git ignores non-executable hook files).
//!   4. Invoke `pmat hooks uninstall` and re-list `.git/hooks/` to confirm
//!      the directory returns to its pre-install state.
//!
//! Run with: `cargo run --example hooks_install_lifecycle`
//!
//! If `pmat` is not on PATH the walkthrough still prints so the file is
//! useful as documentation. Same pattern as `examples/comply_cb16xx.rs`.

use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("=== PMAT Hooks — Install / Uninstall Lifecycle ===\n");
    print_hook_notes();

    let pmat = option_env!("CARGO_BIN_EXE_pmat").unwrap_or("pmat");
    let tmp = tempfile::tempdir().expect("create tempdir");
    let root = tmp.path();
    println!("Ephemeral repo at: {}\n", root.display());

    // ---- Phase 1: Initialize repo + minimal Rust project ----------------
    println!("--- Phase 1: git init + Cargo.toml ---");
    init_repo(root);
    write_min_project_files(root);
    list_git_hooks(root, "before install");

    // ---- Phase 2: pmat hooks install ------------------------------------
    println!("\n--- Phase 2: pmat hooks install ---");
    if !run_pmat(pmat, root, &["hooks", "install"]) {
        println!(
            "\n(Phase 3/4 skipped — pmat not on PATH. The phase notes above \
             are the primary documentation payload.)"
        );
        return;
    }

    // ---- Phase 3: Inspect .git/hooks/ -----------------------------------
    println!("\n--- Phase 3: inspect .git/hooks/ ---");
    list_git_hooks(root, "after install");

    // ---- Phase 4: pmat hooks uninstall ----------------------------------
    println!("\n--- Phase 4: pmat hooks uninstall ---");
    run_pmat(pmat, root, &["hooks", "uninstall"]);
    list_git_hooks(root, "after uninstall");

    println!("\n=== Lifecycle complete ===");
}

fn print_hook_notes() {
    println!(
        "Git hook lifecycle in pmat:

  install    : writes pre-commit, pre-push, post-commit (etc.) into
               .git/hooks/ and chmod +x them. Idempotent — re-running
               refreshes the payload and keeps the +x bit.
  uninstall  : removes pmat-managed hook files but leaves any hooks you
               authored yourself untouched.

  Under the hood:
    - pre-commit  : runs complexity / format / TDG gates (<30s by design)
    - pre-push    : O(1) artefact check; avoids network or heavy builds
    - post-commit : records TDG/repo-score snapshots under .pmat-metrics/

  Hooks are only active when the file exists AND has the executable bit
  set. `ls -l .git/hooks/` is the quickest way to audit this.
"
    );
}

fn init_repo(root: &Path) {
    let status = Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(root)
        .status()
        .expect("run git init");
    if !status.success() {
        // Older git versions default to master; fall back gracefully.
        let _ = Command::new("git").arg("init").current_dir(root).status();
    }
    // Set a throwaway identity so any future `git commit` in the example
    // (should someone extend it) succeeds without relying on $HOME config.
    let _ = Command::new("git")
        .args(["config", "user.email", "example@pmat.dev"])
        .current_dir(root)
        .status();
    let _ = Command::new("git")
        .args(["config", "user.name", "Example"])
        .current_dir(root)
        .status();
    println!("  git init -b main -> {}", root.display());
}

fn write_min_project_files(root: &Path) {
    let cargo_toml = r#"[package]
name = "hooks-lifecycle-demo"
version = "0.0.1"
edition = "2021"
"#;
    fs::write(root.join("Cargo.toml"), cargo_toml).expect("write Cargo.toml");
    let src = root.join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(src.join("lib.rs"), "pub fn demo() {}\n").expect("write src/lib.rs");
    println!("  Cargo.toml + src/lib.rs written.");
}

fn list_git_hooks(root: &Path, phase: &str) {
    let hooks_dir = root.join(".git").join("hooks");
    println!("  .git/hooks/ ({}):", phase);
    let Ok(entries) = fs::read_dir(&hooks_dir) else {
        println!("    (directory missing — did `git init` fail?)");
        return;
    };
    let mut names: Vec<(String, bool, u64)> = entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            // Skip git's stock `.sample` files — they add noise and are
            // never active. The pmat hooks we're installing drop the .sample
            // suffix, which is the whole point of this listing.
            if name.ends_with(".sample") {
                return None;
            }
            let meta = e.metadata().ok()?;
            let exec = is_executable(&meta);
            Some((name, exec, meta.len()))
        })
        .collect();
    names.sort();
    if names.is_empty() {
        println!("    (no non-sample hooks present)");
        return;
    }
    for (name, exec, size) in names {
        let flag = if exec { "x" } else { "-" };
        println!("    [{flag}] {name:24} {size} bytes");
    }
}

#[cfg(unix)]
fn is_executable(meta: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_meta: &std::fs::Metadata) -> bool {
    // Windows: every file is "executable" for hook purposes as long as it
    // has the right name; git on Windows uses shebangs instead of +x.
    true
}

fn run_pmat(pmat: &str, root: &Path, args: &[&str]) -> bool {
    println!("  $ {pmat} {}", args.join(" "));
    let output = match Command::new(pmat).args(args).current_dir(root).output() {
        Ok(o) => o,
        Err(e) => {
            println!("  (pmat not found on PATH: {e})");
            println!("  Install with `cargo install --path .` from the pmat checkout.");
            return false;
        }
    };
    println!("  exit status: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let preview: Vec<&str> = stdout
        .lines()
        .chain(stderr.lines())
        .filter(|l| !l.trim().is_empty())
        .take(8)
        .collect();
    for line in preview {
        println!("    {line}");
    }
    output.status.success()
}
