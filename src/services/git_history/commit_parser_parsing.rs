// Git Commit Parser - Parsing methods
// Extracted from commit_parser.rs for maintainability
// Contains: CommitParser impl with all parsing and extraction logic

impl CommitParser {
    /// Open a repository at the given path
    pub fn open(path: &Path) -> Result<Self> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let repo = Repository::discover(path)
            .with_context(|| format!("Failed to open git repository at {:?}", path))?;
        Ok(Self { repo })
    }

    /// Parse all commits, optionally since a given commit hash
    pub fn parse_commits(
        &self,
        since: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(Sort::TIME | Sort::REVERSE)?;
        revwalk.push_head()?;

        // If we have a 'since' commit, hide all ancestors
        if let Some(since_hash) = since {
            if let Ok(oid) = git2::Oid::from_str(since_hash) {
                revwalk.hide(oid)?;
            }
        }

        let mut commits = Vec::new();
        let mut count = 0;

        for oid_result in revwalk {
            if let Some(max) = limit {
                if count >= max {
                    break;
                }
            }

            let oid = oid_result?;
            let commit = self.repo.find_commit(oid)?;

            if let Some(info) = self.parse_commit(&commit)? {
                commits.push(info);
                count += 1;
            }
        }

        Ok(commits)
    }

    /// Parse a single commit into CommitInfo
    fn parse_commit(&self, commit: &Commit) -> Result<Option<CommitInfo>> {
        let hash = commit.id().to_string();

        // Parse message
        let message = commit.message().unwrap_or("");
        let (subject, body) = Self::split_message(message);

        // Extract metadata
        let author = commit.author();
        let author_name = author.name().unwrap_or("Unknown").to_string();
        let author_email = author.email().unwrap_or("").to_string();
        let timestamp = commit.time().seconds();

        // Check if merge commit
        let is_merge = commit.parent_count() > 1;

        // Detect conventional commit types
        let is_fix = Self::is_fix_commit(&subject);
        let is_feat = Self::is_feat_commit(&subject);

        // Extract issue references
        let issue_refs = Self::extract_issue_refs(&subject, body.as_deref().unwrap_or(""));

        // Get file changes
        let files = self.get_file_changes(commit)?;

        Ok(Some(CommitInfo {
            hash,
            message_subject: subject,
            message_body: body,
            author_name,
            author_email,
            timestamp,
            is_merge,
            is_fix,
            is_feat,
            issue_refs,
            files,
        }))
    }

    /// Split commit message into subject and body
    fn split_message(message: &str) -> (String, Option<String>) {
        let lines: Vec<&str> = message.lines().collect();

        if lines.is_empty() {
            return (String::new(), None);
        }

        let subject = lines[0].trim().to_string();

        // Body starts after the first blank line
        let body_start = lines.iter().skip(1).position(|l| l.trim().is_empty());

        let body = if let Some(start) = body_start {
            let body_lines: Vec<&str> = lines.iter().skip(start + 2).copied().collect();
            if body_lines.is_empty() {
                None
            } else {
                Some(body_lines.join("\n").trim().to_string())
            }
        } else {
            None
        };

        (subject, body)
    }

    /// Check if commit is a fix (conventional commit or keyword)
    fn is_fix_commit(subject: &str) -> bool {
        debug_assert!(!subject.is_empty(), "subject must not be empty");
        let lower = subject.to_lowercase();
        lower.starts_with("fix:")
            || lower.starts_with("fix(")
            || lower.starts_with("bugfix:")
            || lower.starts_with("hotfix:")
            || lower.contains("fix ")
            || lower.contains("fixed ")
            || lower.contains("fixes ")
    }

    /// Check if commit is a feature (conventional commit)
    fn is_feat_commit(subject: &str) -> bool {
        debug_assert!(!subject.is_empty(), "subject must not be empty");
        let lower = subject.to_lowercase();
        lower.starts_with("feat:") || lower.starts_with("feat(") || lower.starts_with("feature:")
    }

    /// Extract issue references from commit message
    fn extract_issue_refs(subject: &str, body: &str) -> Vec<String> {
        debug_assert!(!subject.is_empty(), "subject must not be empty");
        debug_assert!(!body.is_empty(), "body must not be empty");
        let mut refs = Vec::new();
        let full_text = format!("{} {}", subject, body);

        // GitHub-style: #123
        let github_re = regex::Regex::new(r"#(\d+)").expect("valid regex");
        for cap in github_re.captures_iter(&full_text) {
            refs.push(format!("#{}", &cap[1]));
        }

        // JIRA-style: PROJ-123
        let jira_re = regex::Regex::new(r"([A-Z]+-\d+)").expect("valid regex");
        for cap in jira_re.captures_iter(&full_text) {
            let issue = cap[1].to_string();
            if !refs.contains(&issue) {
                refs.push(issue);
            }
        }

        refs
    }

    /// Get files changed in a commit with diff stats
    fn get_file_changes(&self, commit: &Commit) -> Result<Vec<FileChange>> {
        let mut changes = Vec::new();

        let tree = commit.tree()?;
        let parent_tree = if commit.parent_count() > 0 {
            Some(commit.parent(0)?.tree()?)
        } else {
            None
        };

        let mut diff_opts = DiffOptions::new();
        let diff =
            self.repo
                .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))?;

        diff.foreach(
            &mut |delta, _progress| {
                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let change_type = match delta.status() {
                    git2::Delta::Added => ChangeType::Added,
                    git2::Delta::Deleted => ChangeType::Deleted,
                    git2::Delta::Renamed => ChangeType::Renamed,
                    _ => ChangeType::Modified,
                };

                changes.push(FileChange {
                    path,
                    change_type,
                    lines_added: 0,
                    lines_deleted: 0,
                });

                true
            },
            None,
            None,
            None,
        )?;

        // Get line stats
        let stats = diff.stats()?;
        let _insertions = stats.insertions();
        let _deletions = stats.deletions();

        Ok(changes)
    }

    /// Get the most recent commit hash
    pub fn head_commit_hash(&self) -> Result<String> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }
}
