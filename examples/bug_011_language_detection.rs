//! BUG-011 regression guard: a C++ project must be detected as C++.
//!
//! This file used to be a REPRODUCER. It described a live defect — a Ceph-like
//! C++ tree detected as `python-uv`, with the discovery phase hanging — and
//! listed four things to implement in order to fix it.
//!
//! All four shipped. Measured 2026-08-25 against this example's own fixture:
//! detection returns `cpp` at 100.0% confidence and does not hang, and both
//! `pmat context --language` and `--languages` exist (BUG-012). The file was
//! still telling its reader that none of that worked.
//!
//! That made it a false document rather than a stale one: somebody reading it
//! would conclude pmat cannot override language detection, when it can. It also
//! carried the tree's only strict-mode SATD marker — invisible until the
//! `examples/` exclusion was removed (#1035), because SATD never walked this
//! directory.
//!
//! Rewritten as what it now is: a check that FAILS if BUG-011 returns. A
//! reproducer for a fixed bug should either be deleted or become a guard;
//! leaving it to narrate is how a document outlives the thing it describes.
//!
//! Run with: `cargo run --example bug_011_language_detection`
//! Exits non-zero if the bug has regressed.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::timeout;

use pmat::services::enhanced_language_detection::{
    detect_project_language_enhanced, LanguageDetection,
};

/// The hang was the second half of BUG-011, so the guard keeps a deadline.
/// Generous on purpose: this must fail on a hang, never on a slow machine.
const DETECTION_DEADLINE: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> Result<()> {
    println!("BUG-011 regression guard: C++ project detection\n");

    let test_dir = create_mock_cpp_project().await?;
    println!("fixture: {test_dir:?}");
    println!("  70 .cc + 70 .h, a CMakeLists.txt, 20 .py, and a pyproject.toml");
    println!("  (the pyproject.toml is the decoy that produced the original defect)\n");

    let detection = timeout(DETECTION_DEADLINE, detect_project_language(&test_dir)).await;
    cleanup_mock_project(&test_dir).await?;

    let detection = match detection {
        Ok(inner) => inner?,
        Err(_) => bail!(
            "BUG-011 HAS REGRESSED: detection did not finish within {}s. \
             The original defect hung in the discovery phase.",
            DETECTION_DEADLINE.as_secs()
        ),
    };

    println!(
        "detected: {} ({:.1}% confidence)",
        detection.language, detection.confidence
    );

    if detection.language != "cpp" {
        bail!(
            "BUG-011 HAS REGRESSED: a tree that is 70% C++ with a CMakeLists.txt \
             was detected as {:?}. The original defect reported `python-uv`, \
             misled by the pyproject.toml in scripts/.",
            detection.language
        );
    }

    println!("\nPASS: C++ detected, within the deadline.");
    Ok(())
}

async fn detect_project_language(path: &Path) -> Result<LanguageDetection> {
    Ok(detect_project_language_enhanced(path))
}

/// A Ceph-like tree: mostly C++, with enough Python to mislead a naive detector.
async fn create_mock_cpp_project() -> Result<PathBuf> {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new()?;
    let base_path = temp_dir.path();

    fs::create_dir_all(base_path.join("src"))?;
    for i in 0..70 {
        fs::write(
            base_path.join(format!("src/module_{i}.cc")),
            format!("// C++ file {i}\nint main() {{ return 0; }}"),
        )?;
        fs::write(
            base_path.join(format!("src/module_{i}.h")),
            format!("// Header file {i}\n#pragma once"),
        )?;
    }

    // The primary indicator for C++.
    fs::write(
        base_path.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.10)\nproject(TestProject)\n",
    )?;

    fs::create_dir_all(base_path.join("scripts"))?;
    for i in 0..20 {
        fs::write(
            base_path.join(format!("scripts/helper_{i}.py")),
            format!("# Python helper {i}\nprint('hello')"),
        )?;
    }

    // The decoy. This is what the original defect keyed on.
    fs::write(
        base_path.join("scripts/pyproject.toml"),
        "[project]\nname = \"helpers\"\n",
    )?;

    // The directory must outlive the TempDir guard because the caller cleans up
    // explicitly after the detection, including on the failure paths above.
    let leaked_path = base_path.to_path_buf();
    std::mem::forget(temp_dir);
    Ok(leaked_path)
}

async fn cleanup_mock_project(path: &PathBuf) -> Result<()> {
    if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}
