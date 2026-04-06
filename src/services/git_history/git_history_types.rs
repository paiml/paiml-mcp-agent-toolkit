/// Type of file change in a commit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl ChangeType {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeType::Added => "A",
            ChangeType::Modified => "M",
            ChangeType::Deleted => "D",
            ChangeType::Renamed => "R",
        }
    }
}

/// Information about a file changed in a commit
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub lines_added: u32,
    pub lines_deleted: u32,
}

/// Parsed commit information (index-compatible version)
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub message_subject: String,
    pub message_body: Option<String>,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
    pub is_merge: bool,
    pub is_fix: bool,
    pub is_feat: bool,
    pub issue_refs: Vec<String>,
    pub files: Vec<FileChange>,
}

impl CommitInfo {
    /// Full commit message (subject + body)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn full_message(&self) -> String {
        match &self.message_body {
            Some(body) if !body.is_empty() => format!("{}\n\n{}", self.message_subject, body),
            _ => self.message_subject.clone(),
        }
    }

    /// Check if this is a meaningful commit for indexing
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_indexable(&self) -> bool {
        if self.is_merge && self.message_subject.starts_with("Merge ") {
            return false;
        }
        self.message_subject.len() >= 10
    }
}

/// Result of incremental sync operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Number of commits added to index
    pub commits_added: usize,
    /// Number of commits skipped (non-indexable or already present)
    pub commits_skipped: usize,
    /// Hash of the last indexed commit
    pub last_commit: Option<String>,
}

/// Error type for git history operations
#[derive(Debug, thiserror::Error)]
pub enum GitHistoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Commit not found: {0}")]
    CommitNotFound(String),

    #[error("Index corrupted: {0}")]
    IndexCorrupted(String),
}

/// Git history index using SQLite
/// Stores commit messages with embeddings for semantic search
pub struct GitHistoryIndex {
    pub(crate) conn: Connection,
}
