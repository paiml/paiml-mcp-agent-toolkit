impl AgentContextIndex {
    /// Regex-based scoring: match pattern against source/signature/name
    #[allow(clippy::cast_possible_truncation)]
    fn calculate_regex_scores(
        &self,
        pattern: &str,
        candidates: Option<&[usize]>,
        options: &QueryOptions,
    ) -> Result<Vec<(usize, f32)>, String> {
        debug_assert!(true, "contract: calculate_regex_scores");
        let case_insensitive = match options.case_sensitivity {
            CaseSensitivity::Insensitive => true,
            CaseSensitivity::Sensitive => false,
            CaseSensitivity::Smart => !pattern.chars().any(|c| c.is_uppercase()),
        };

        let re = regex::RegexBuilder::new(pattern)
            .case_insensitive(case_insensitive)
            .build()
            .map_err(|e| format!("Invalid regex pattern: {e}"))?;

        let iter: Box<dyn Iterator<Item = usize>> = match candidates {
            Some(indices) => Box::new(indices.iter().copied()),
            None => Box::new(0..self.functions.len()),
        };

        let mut results = Vec::new();
        for idx in iter {
            if idx >= self.functions.len() {
                continue;
            }
            let func = &self.functions[idx];
            // Count matches across name, signature, source, and file path
            let name_matches = re.find_iter(&func.function_name).count();
            let sig_matches = re.find_iter(&func.signature).count();
            let source_matches = re.find_iter(&func.source).count();
            let path_matches = re.find_iter(&func.file_path).count();
            let total = name_matches + sig_matches + source_matches + path_matches;
            if total > 0 {
                // Score: weight name matches highest, then signature, then path, then source
                let score = (name_matches as f32 * 3.0
                    + sig_matches as f32 * 2.0
                    + path_matches as f32 * 1.5
                    + source_matches as f32)
                    / (1.0 + func.source.len() as f32 / 1000.0);
                results.push((idx, score));
            }
        }

        // Normalize
        let max_score = results.iter().map(|(_, s)| *s).fold(0.0f32, f32::max);
        if max_score > 0.0 {
            for (_, score) in &mut results {
                *score /= max_score;
            }
        }

        Ok(results)
    }

    /// Literal string scoring: exact match against source/signature/name
    #[allow(clippy::cast_possible_truncation)]
    fn calculate_literal_scores(
        &self,
        needle: &str,
        candidates: Option<&[usize]>,
        options: &QueryOptions,
    ) -> Result<Vec<(usize, f32)>, String> {
        debug_assert!(!needle.is_empty(), "needle must not be empty");
        let case_insensitive = match options.case_sensitivity {
            CaseSensitivity::Insensitive => true,
            CaseSensitivity::Sensitive => false,
            CaseSensitivity::Smart => !needle.chars().any(|c| c.is_uppercase()),
        };

        let needle_cmp = if case_insensitive {
            needle.to_lowercase()
        } else {
            needle.to_string()
        };

        let iter: Box<dyn Iterator<Item = usize>> = match candidates {
            Some(indices) => Box::new(indices.iter().copied()),
            None => Box::new(0..self.functions.len()),
        };

        let mut results = Vec::new();
        for idx in iter {
            if idx >= self.functions.len() {
                continue;
            }
            let func = &self.functions[idx];
            let (name, sig, source) = if case_insensitive {
                (
                    func.function_name.to_lowercase(),
                    func.signature.to_lowercase(),
                    func.source.to_lowercase(),
                )
            } else {
                (
                    func.function_name.clone(),
                    func.signature.clone(),
                    func.source.clone(),
                )
            };

            let name_matches = name.matches(&needle_cmp).count();
            let sig_matches = sig.matches(&needle_cmp).count();
            let source_matches = source.matches(&needle_cmp).count();
            let file_path_cmp = if case_insensitive {
                func.file_path.to_lowercase()
            } else {
                func.file_path.clone()
            };
            let path_matches = file_path_cmp.matches(&needle_cmp).count();
            let total = name_matches + sig_matches + source_matches + path_matches;
            if total > 0 {
                let score = (name_matches as f32 * 3.0
                    + sig_matches as f32 * 2.0
                    + path_matches as f32 * 1.5
                    + source_matches as f32)
                    / (1.0 + func.source.len() as f32 / 1000.0);
                results.push((idx, score));
            }
        }

        // Normalize
        let max_score = results.iter().map(|(_, s)| *s).fold(0.0f32, f32::max);
        if max_score > 0.0 {
            for (_, score) in &mut results {
                *score /= max_score;
            }
        }

        Ok(results)
    }
}
