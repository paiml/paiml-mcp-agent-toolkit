/// Count version entries in changelog content (e.g., [0.1.0], ## 4.2.0)
///
/// Matches lines containing `[N.` or `## N.` where N is any digit sequence,
/// supporting semver major versions beyond 0/1/2.
fn count_version_entries(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Match `## [N.x.y]` or `## N.x.y` patterns (keepachangelog format)
            // Also match `[N.x.y]` standalone (link reference definitions)
            has_version_bracket(trimmed) || has_version_heading(trimmed)
        })
        .count()
}

/// Check if line contains a bracketed version like `[1.2.3]` or `[0.1.0]`
fn has_version_bracket(line: &str) -> bool {
    let bytes = line.as_bytes();
    for i in 0..bytes.len().saturating_sub(2) {
        if bytes[i] == b'[' && bytes[i + 1].is_ascii_digit() {
            // Look for a dot after the digits
            let mut j = i + 2;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j < bytes.len() && bytes[j] == b'.' {
                return true;
            }
        }
    }
    false
}

/// Check if line contains a heading version like `## 0.1.0` (without brackets)
fn has_version_heading(line: &str) -> bool {
    if let Some(rest) = line.strip_prefix("## ") {
        let rest = rest.trim();
        // First char must be a digit, followed eventually by a dot
        let mut chars = rest.chars();
        if let Some(c) = chars.next() {
            if c.is_ascii_digit() {
                for ch in chars {
                    if ch == '.' {
                        return true;
                    }
                    if !ch.is_ascii_digit() {
                        return false;
                    }
                }
            }
        }
    }
    false
}

impl DocumentationScorer {
    /// Score changelog presence (3pts)
    /// Checks for CHANGELOG.md with version history
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available for CHANGELOG.md
    /// **Kaizen Round 5**: Also checks workspace root for monorepo structures
    fn score_changelog(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
        let content = self.read_best_changelog(project_path, cache)?;
        let Some(content) = content else {
            return Ok(0.0);
        };
        let version_count = count_version_entries(&content);
        Ok(Self::changelog_version_score(version_count))
    }

    fn changelog_version_score(version_count: usize) -> f64 {
        match () {
            _ if version_count >= 2 => 3.0,
            _ if version_count >= 1 => 2.0,
            _ => 1.0,
        }
    }

    fn read_best_changelog(
        &self,
        project_path: &Path,
        cache: Option<&FileCache>,
    ) -> ScorerResult<Option<String>> {
        let changelog_path = project_path.join("CHANGELOG.md");
        let ws_changelog = project_path.parent().map(|p| p.join("CHANGELOG.md"));

        let content = self.read_changelog_file(&changelog_path, cache);
        let ws_content = ws_changelog
            .as_ref()
            .and_then(|p| self.read_changelog_file(p, None));

        match (content, ws_content) {
            (Some(c), Some(ws)) => {
                if count_version_entries(&ws) > count_version_entries(&c) {
                    Ok(Some(ws))
                } else {
                    Ok(Some(c))
                }
            }
            (Some(c), None) => Ok(Some(c)),
            (None, Some(ws)) => Ok(Some(ws)),
            (None, None) => Ok(None),
        }
    }

    fn read_changelog_file(
        &self,
        path: &std::path::PathBuf,
        cache: Option<&FileCache>,
    ) -> Option<String> {
        if !path.exists() {
            return None;
        }
        if let Some(cache) = cache {
            return cache.get(path).cloned();
        }
        std::fs::read_to_string(path).ok()
    }
}
