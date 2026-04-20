//! Query With Rankers — Three flavors of `pmat query`
//!
//! Runs the same semantic prompt ("error handling") through three distinct
//! ranking strategies so readers can see how the ranker dimension shapes
//! `pmat query` output:
//!
//!   (a) Default ranking (TF-IDF / BM25 hybrid) — how `pmat query` answers
//!       when no flags are passed.
//!   (b) `--rank-by pagerank --limit 10` — graph-centrality ranking. Top
//!       results are whatever functions sit at the center of the call
//!       graph, regardless of textual match strength.
//!   (c) `--coverage --min-grade B --limit 5` — quality-gated ranking with
//!       LLVM line coverage enrichment. Useful for finding high-trust
//!       examples that also have production coverage.
//!
//! Run with: `cargo run --example query_with_rankers`
//!
//! No tempdir here — this example intentionally runs against whatever
//! project it is invoked from so readers can see live quality/centrality
//! numbers. If `pmat` isn't on PATH we still print the semantic notes so
//! the file doubles as a cheat-sheet for the flag combinations.

use std::process::Command;

fn main() {
    println!("=== PMAT Query — Ranking Flavors ===\n");
    print_ranker_notes();

    let pmat = option_env!("CARGO_BIN_EXE_pmat").unwrap_or("pmat");
    let prompt = "error handling";

    run_phase(
        "(a) Default ranking (TF-IDF / BM25 hybrid)",
        pmat,
        &[prompt, "--limit", "5"],
        "The dispatcher falls back to TF-IDF scoring when FTS5 is not \
         available, or BM25 when `.pmat/context.db` is present. No quality \
         filter, no coverage enrichment — purely textual relevance.",
    );

    run_phase(
        "(b) PageRank centrality (--rank-by pagerank --limit 10)",
        pmat,
        &[prompt, "--rank-by", "pagerank", "--limit", "10"],
        "Swaps textual scoring for graph centrality. Results are important \
         functions (high PageRank) that still match the query. Expect hub \
         functions like dispatch/parse/error-convert at the top.",
    );

    run_phase(
        "(c) Coverage-enriched + grade gate (--coverage --min-grade B --limit 5)",
        pmat,
        &[prompt, "--coverage", "--min-grade", "B", "--limit", "5"],
        "Adds LLVM line coverage columns (hits/total) and filters to \
         functions with TDG grade B or better. Useful for finding \
         high-trust examples suitable for review or documentation.",
    );

    println!("\n=== Tip ===");
    println!(
        "Combine enrichments for a full audit:
  pmat query \"{prompt}\" --churn --duplicates --entropy --faults -G --limit 10

That single command fuses git-intent ranking with volatility, clone, \
entropy, and fault-pattern signals."
    );
}

fn print_ranker_notes() {
    println!(
        "Ranker dimensions available to `pmat query`:

  textual (default)  : BM25 via FTS5 when .pmat/context.db exists, TF-IDF else.
  --rank-by pagerank : trueno-graph centrality over the call graph.
  --rank-by impact   : (coverage-gap mode) missed_lines * pagerank / complexity.
  --coverage         : append LLVM coverage columns to every row.
  --min-grade G      : filter to rows with TDG grade >= G (A/B/C/D/F).
  --faults           : append fault annotations (unwrap, panic, unsafe, …).
  -G                 : fuse git commit history via RRF (semantic intent).
"
    );
}

fn run_phase(label: &str, pmat: &str, args: &[&str], note: &str) {
    println!("\n--- {label} ---");
    println!("  {note}");
    println!("  $ {pmat} query {}", args.join(" "));

    let mut cmd = Command::new(pmat);
    cmd.arg("query");
    cmd.args(args);
    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            println!("  (pmat not on PATH: {e})");
            println!("  Install with `cargo install --path .` from the pmat checkout.");
            return;
        }
    };

    println!("  exit status: {}", output.status);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Preview first 12 non-empty stdout lines so the example stays short.
    let preview: Vec<&str> = stdout
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(12)
        .collect();
    if preview.is_empty() {
        println!("  (no stdout rows)");
    } else {
        let shown = preview.len();
        println!("  stdout preview (first {shown} non-empty lines):");
        for line in &preview {
            println!("    {line}");
        }
        let total = stdout.lines().filter(|l| !l.trim().is_empty()).count();
        if total > shown {
            println!("    ... ({} more lines)", total - shown);
        }
    }

    // Only surface stderr when there is something meaningful to say — and
    // only the first couple of lines, otherwise warning-verbose CLIs drown
    // the walkthrough.
    let stderr_preview: Vec<&str> = stderr
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(2)
        .collect();
    if !stderr_preview.is_empty() {
        println!("  stderr preview:");
        for line in stderr_preview {
            println!("    {line}");
        }
    }
}
