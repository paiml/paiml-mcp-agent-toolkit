/// Count version entries in changelog content (e.g., [0.1.0], ## 0.1.0)
fn count_version_entries(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            line.contains("[0.")
                || line.contains("[1.")
                || line.contains("[2.")
                || line.contains("## 0.")
                || line.contains("## 1.")
                || line.contains("## 2.")
        })
        .count()
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
