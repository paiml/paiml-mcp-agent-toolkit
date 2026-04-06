// Git log parsing: PMAT_START block format parser and commit classification.
//
// Provides: parse_git_log (main entry point), parse_commit_block,
// parse_header_line, parse_file_change_line, classify_commit_type_from_subject,
// and extract_issue_refs_from_subject.

// ── Git log parsing ─────────────────────────────────────────────────────────

/// Parse git log output with format:
/// PMAT_START
/// H:<hash>
/// S:<subject>
/// N:<author_name>
/// E:<author_email>
/// T:<timestamp>
/// PMAT_FILES
/// M\tfile1.rs
/// A\tfile2.rs
fn parse_file_change_line(line: &str) -> Option<FileChange> {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let parts: Vec<&str> = line.splitn(2, '\t').collect();
    if parts.len() != 2 { return None; }
    let change_type = match parts[0].chars().next() {
        Some('A') => ChangeType::Added,
        Some('D') => ChangeType::Deleted,
        _ => ChangeType::Modified,
    };
    Some(FileChange {
        path: parts[1].trim().to_string(),
        change_type,
        lines_added: 0,
        lines_deleted: 0,
    })
}

fn parse_header_line(
    line: &str,
    hash: &mut String, subject: &mut String,
    author_name: &mut String, author_email: &mut String, timestamp: &mut i64,
) {
    debug_assert!(!line.is_empty(), "line must not be empty");
    if let Some(val) = line.strip_prefix("H:") { *hash = val.to_string(); }
    else if let Some(val) = line.strip_prefix("S:") { *subject = val.to_string(); }
    else if let Some(val) = line.strip_prefix("N:") { *author_name = val.to_string(); }
    else if let Some(val) = line.strip_prefix("E:") { *author_email = val.to_string(); }
    else if let Some(val) = line.strip_prefix("T:") { *timestamp = val.parse().unwrap_or(0); }
}

fn parse_commit_block(block: &str) -> Option<CommitInfo> {
    debug_assert!(!block.is_empty(), "block must not be empty");
    let mut hash = String::new();
    let mut subject = String::new();
    let mut author_name = String::new();
    let mut author_email = String::new();
    let mut timestamp: i64 = 0;
    let mut files = Vec::new();
    let mut in_files = false;

    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if line == "PMAT_FILES" { in_files = true; continue; }
        if in_files {
            if let Some(fc) = parse_file_change_line(line) { files.push(fc); }
        } else {
            parse_header_line(line, &mut hash, &mut subject, &mut author_name, &mut author_email, &mut timestamp);
        }
    }
    if hash.is_empty() { return None; }

    let (is_fix, is_feat, is_merge) = classify_commit_type_from_subject(&subject);
    let issue_refs = extract_issue_refs_from_subject(&subject);
    Some(CommitInfo {
        hash, message_subject: subject, message_body: None,
        author_name, author_email, timestamp,
        is_merge, is_fix, is_feat, issue_refs, files,
    })
}

fn classify_commit_type_from_subject(subject: &str) -> (bool, bool, bool) {
    debug_assert!(!subject.is_empty(), "subject must not be empty");
    let s = subject.to_lowercase();
    let is_fix = s.starts_with("fix") || s.contains("fix:") || s.contains("bugfix");
    let is_feat = s.starts_with("feat") || s.contains("feat:") || s.starts_with("add ");
    let is_merge = s.starts_with("merge ");
    (is_fix, is_feat, is_merge)
}

fn extract_issue_refs_from_subject(subject: &str) -> Vec<String> {
    debug_assert!(!subject.is_empty(), "subject must not be empty");
    subject
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| c == '(' || c == ')' || c == ',' || c == '.'))
        .filter(|w| {
            (w.starts_with('#') && w.len() > 1)
                || w.starts_with("PMAT-") || w.starts_with("pmat-")
                || w.starts_with("GH-") || w.starts_with("gh-")
        })
        .map(|w| w.to_string())
        .collect()
}

pub(super) fn parse_git_log(log_text: &str) -> Vec<CommitInfo> {
    log_text
        .split("PMAT_START")
        .skip(1)
        .filter_map(parse_commit_block)
        .collect()
}
