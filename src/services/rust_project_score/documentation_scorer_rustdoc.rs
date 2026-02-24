impl DocumentationScorer {
    /// Score rustdoc coverage (7pts)
    /// Checks for public API documentation with examples
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available for src/*.rs
    fn score_rustdoc(&self, project_path: &Path, cache: Option<&FileCache>) -> ScorerResult<f64> {
        let src_path = project_path.join("src");
        if !src_path.exists() {
            return Ok(0.0);
        }

        let mut total_public_items = 0;
        let mut documented_items = 0;

        // Walk src directory
        self.count_documented_items(
            &src_path,
            &mut total_public_items,
            &mut documented_items,
            cache,
        )?;

        if total_public_items == 0 {
            // No public API = moderate score
            return Ok(3.5);
        }

        // Calculate documentation coverage ratio
        let doc_ratio = documented_items as f64 / total_public_items as f64;

        // Tiered scoring based on documentation coverage
        if doc_ratio >= 0.90 {
            Ok(7.0) // >=90% documented
        } else if doc_ratio >= 0.75 {
            Ok(6.0) // >=75% documented
        } else if doc_ratio >= 0.60 {
            Ok(4.0) // >=60% documented
        } else if doc_ratio >= 0.40 {
            Ok(2.0) // >=40% documented
        } else {
            Ok(0.0) // <40% documented
        }
    }

    /// Count documented public items in directory (recursive)
    ///
    /// **Kaizen Round 4**: Cache-aware - uses FileCache if available
    fn count_documented_items(
        &self,
        dir: &Path,
        total: &mut usize,
        documented: &mut usize,
        cache: Option<&FileCache>,
    ) -> ScorerResult<()> {
        if let Some(cache) = cache {
            for (_path, content) in cache.get_rust_files_in_dir(dir) {
                self.analyze_doc_coverage(content, total, documented);
            }
        } else {
            self.count_documented_items_from_fs(dir, total, documented)?;
        }
        Ok(())
    }

    fn count_documented_items_from_fs(
        &self,
        dir: &Path,
        total: &mut usize,
        documented: &mut usize,
    ) -> ScorerResult<()> {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return Ok(());
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.count_documented_items_from_fs(&path, total, documented)?;
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    self.analyze_doc_coverage(&content, total, documented);
                }
            }
        }
        Ok(())
    }

    /// Analyze documentation coverage in Rust source code
    fn analyze_doc_coverage(&self, content: &str, total: &mut usize, documented: &mut usize) {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Check for pub items
            if line.starts_with("pub fn")
                || line.starts_with("pub struct")
                || line.starts_with("pub enum")
                || line.starts_with("pub trait")
            {
                *total += 1;

                // Check if previous lines contain doc comments (/// or //!)
                let mut has_doc_comment = false;
                for j in (0..i).rev().take(10) {
                    let prev_line = lines[j].trim();
                    if prev_line.starts_with("///") || prev_line.starts_with("//!") {
                        has_doc_comment = true;
                        break;
                    }
                    if !prev_line.is_empty()
                        && !prev_line.starts_with("//")
                        && !prev_line.starts_with("#[")
                    {
                        break;
                    }
                }

                if has_doc_comment {
                    *documented += 1;
                }
            }

            i += 1;
        }
    }
}
