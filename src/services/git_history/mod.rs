// Git History RAG Integration
// Spec: docs/specifications/git-history-rag-integration.md
// Toyota Way: Genchi Genbutsu - Go and see actual git data

#[cfg(feature = "git-lib")]
mod commit_parser;
mod commit_embedder;
mod git_history_index;
mod search_engine;

// Always export from git_history_index (no git2 dependency)
pub use git_history_index::{
    ChangeType, CommitInfo, FileChange, GitHistoryError, GitHistoryIndex, SyncResult,
};
pub use commit_embedder::CommitEmbedder;
pub use search_engine::{GitHistorySearchEngine, GitSearchOptions, GitSearchResult};

// Only export CommitParser when git-lib feature is enabled
#[cfg(feature = "git-lib")]
pub use commit_parser::CommitParser;

#[cfg(test)]
mod tests;
