// Parity recording verbs (GH-1947): record the original, replay, gate.

/// Parity subcommands: prove a second implementation against recorded behaviour.
#[derive(Subcommand, Debug, Clone)]
pub enum ParityCommands {
    /// Run the ORIGINAL (after `--`) per case, twice, and store the stable cases as JSONL
    Record {
        /// Case specs: JSONL file of {"argv":[..],"stdin":".."} lines
        #[arg(long)]
        cases: PathBuf,
        /// Corpus file to write
        #[arg(long)]
        corpus: PathBuf,
        /// Original command, after `--`
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
    /// Replay a corpus against an implementation (after `--`); prints matched N/M
    Replay {
        /// Corpus file to replay
        #[arg(long)]
        corpus: PathBuf,
        /// Implementation command, after `--`
        #[arg(last = true, required = true)]
        command: Vec<String>,
    },
    /// Gate: refuse on a literal in source, a missing or edited case, or a dropped count
    Gate {
        /// Corpus file
        #[arg(long)]
        corpus: PathBuf,
        /// Committed manifest (JSON with count and hashes)
        #[arg(long)]
        manifest: PathBuf,
        /// Implementation source file to scan for recorded literals
        #[arg(long)]
        source: PathBuf,
    },
}
