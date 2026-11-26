#![cfg_attr(coverage_nightly, coverage(off))]
//! `pmat parity` handlers (GH-1947): record, replay, gate over JSONL corpora.

use crate::cli::commands::ParityCommands;
use crate::parity::{self, Case, CaseSpec, Manifest, Violation};
use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

fn read_jsonl<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Vec<T>> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            serde_json::from_str(l).with_context(|| format!("bad JSONL line in {}", path.display()))
        })
        .collect()
}

fn write_jsonl<T: serde::Serialize>(path: &Path, rows: &[T]) -> Result<()> {
    let mut out = String::new();
    for row in rows {
        out.push_str(&serde_json::to_string(row)?);
        out.push('\n');
    }
    std::fs::write(path, out).with_context(|| format!("writing {}", path.display()))
}

/// Dispatch one `pmat parity` subcommand. Any mismatch or gate violation is an error.
/// Dispatch one `pmat parity` subcommand. Any mismatch or gate violation is an error.
pub fn handle_parity(command: ParityCommands) -> Result<()> {
    match command {
        ParityCommands::Record {
            cases,
            corpus,
            command,
        } => record(&cases, &corpus, &command),
        ParityCommands::Replay { corpus, command } => replay(&corpus, &command),
        ParityCommands::Gate {
            corpus,
            manifest,
            source,
        } => gate(&corpus, &manifest, &source),
    }
}

fn record(cases: &Path, corpus: &Path, command: &[String]) -> Result<()> {
    let (bin, base) = split_command(command)?;
    let specs: Vec<CaseSpec> = read_jsonl(cases)?;
    let summary = parity::record_all(bin, base, &specs, &BTreeMap::new())?;
    write_jsonl(corpus, &summary.recorded)?;
    let manifest = Manifest {
        count: summary.recorded.len(),
        hashes: summary.recorded.iter().map(parity::case_hash).collect(),
    };
    let manifest_path = corpus.with_extension("manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    println!(
        "recorded {} flaky {} (manifest {})",
        summary.recorded.len(),
        summary.flaky,
        manifest_path.display()
    );
    Ok(())
}

fn replay(corpus: &Path, command: &[String]) -> Result<()> {
    let (bin, base) = split_command(command)?;
    let cases: Vec<Case> = read_jsonl(corpus)?;
    let report = parity::replay(bin, base, &cases, &BTreeMap::new())?;
    println!("{}", report.line());
    if let Some(diff) = &report.first_diff {
        println!("first difference: {diff}");
    }
    if report.matched != report.total {
        bail!(
            "{} of {} cases differ",
            report.total - report.matched,
            report.total
        );
    }
    Ok(())
}

fn gate(corpus: &Path, manifest: &Path, source: &Path) -> Result<()> {
    let cases: Vec<Case> = read_jsonl(corpus)?;
    let committed: Manifest = serde_json::from_str(
        &std::fs::read_to_string(manifest)
            .with_context(|| format!("reading {}", manifest.display()))?,
    )?;
    let text = std::fs::read_to_string(source)
        .with_context(|| format!("reading {}", source.display()))?;
    let violations = parity::gate(&cases, &text, &committed);
    for v in &violations {
        println!("{}", describe(v));
    }
    if violations.is_empty() {
        println!("gate: pass ({} cases)", cases.len());
        Ok(())
    } else {
        bail!("gate: {} violation(s)", violations.len())
    }
}
fn split_command(command: &[String]) -> Result<(&str, &[String])> {
    match command.split_first() {
        Some((bin, base)) => Ok((bin.as_str(), base)),
        None => bail!("no command given after `--`"),
    }
}

fn describe(v: &Violation) -> String {
    match v {
        Violation::LiteralInSource { argv, len } => {
            format!("cheat: {len}-byte recorded stdout for {argv:?} appears in source")
        }
        Violation::MissingOrEdited { hash } => {
            format!("cheat: recorded case {hash} missing or edited")
        }
        Violation::CountDropped { committed, now } => {
            format!("cheat: case count dropped from {committed} to {now}")
        }
    }
}
