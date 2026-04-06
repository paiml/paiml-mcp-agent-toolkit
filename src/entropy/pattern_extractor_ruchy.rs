// Ruchy-specific pattern extraction methods

impl PatternExtractor {
    /// Extract Ruchy actor patterns
    fn extract_ruchy_actor_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!content.is_empty(), "content must not be empty");
        use regex::Regex;

        // Pattern: actor definitions with receive handlers
        let actor_pattern =
            Regex::new(r"(?m)^\s*actor\s+\w+\s*\{").expect("Hardcoded regex pattern must be valid");
        let receive_pattern =
            Regex::new(r"(?m)^\s*receive\s+\w+\(").expect("Hardcoded regex pattern must be valid");

        let actor_matches: Vec<_> = actor_pattern.find_iter(content).collect();
        let receive_matches: Vec<_> = receive_pattern.find_iter(content).collect();

        // Only detect as pattern if we have multiple actors or multiple receive handlers
        if actor_matches.len() > 1 || receive_matches.len() > 2 {
            let pattern_hash = self.hash_pattern(&format!("ruchy_actor_{}", file_path.display()));
            let mut locations = Vec::new();

            for (i, m) in actor_matches.iter().enumerate() {
                let line_num = content.get(..m.start()).unwrap_or_default().lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });

                if i >= 10 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::ControlFlow, // Actor model is control flow pattern
                pattern_hash,
                frequency: actor_matches.len().max(receive_matches.len() / 2),
                locations,
                variation_score: self.calculate_actor_variation_score(
                    &actor_matches,
                    &receive_matches,
                    content,
                ),
                example_code: actor_matches
                    .first()
                    .map(|m| {
                        content
                            .get(m.start()..m.end().min(m.start() + 200))
                            .unwrap_or_default()
                            .to_string()
                    })
                    .unwrap_or_default(),
                estimated_loc: actor_matches.len() * 8 + receive_matches.len() * 4,
            };

            collection.add_pattern(pattern);
        }

        Ok(())
    }

    /// Extract Ruchy pipeline operator patterns
    fn extract_ruchy_pipeline_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!content.is_empty(), "content must not be empty");
        use regex::Regex;

        // Pattern: pipeline operators |>
        let pipeline_pattern =
            Regex::new(r"(?m)\s*\|\>\s*\w+\(").expect("Hardcoded regex pattern must be valid");
        let matches: Vec<_> = pipeline_pattern.find_iter(content).collect();

        if matches.len() > 3 {
            // Need at least 3 pipeline operations to be a pattern
            let pattern_hash =
                self.hash_pattern(&format!("ruchy_pipeline_{}", file_path.display()));
            let mut locations = Vec::new();

            for (i, m) in matches.iter().enumerate() {
                let line_num = content.get(..m.start()).unwrap_or_default().lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });

                if i >= 15 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::DataTransformation, // Pipeline is data transformation
                pattern_hash,
                frequency: matches.len(),
                locations,
                variation_score: self.calculate_pipeline_variation_score(&matches, content),
                example_code: matches
                    .first()
                    .map(|m| {
                        let start = m.start().saturating_sub(20);
                        let end = m.end().min(m.start() + 100);
                        content.get(start..end).unwrap_or_default().to_string()
                    })
                    .unwrap_or_default(),
                estimated_loc: matches.len() * 2, // Each pipeline operation is ~2 lines
            };

            collection.add_pattern(pattern);
        }

        Ok(())
    }

    /// Extract Ruchy message passing patterns
    fn extract_ruchy_message_passing_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!content.is_empty(), "content must not be empty");
        use regex::Regex;

        // Pattern: actor message passing <- and <?
        let send_pattern =
            Regex::new(r"(?m)\w+\s*<-\s*\w+\(").expect("Hardcoded regex pattern must be valid");
        let query_pattern =
            Regex::new(r"(?m)\w+\s*<\?\s*\w+\(").expect("Hardcoded regex pattern must be valid");
        let spawn_pattern =
            Regex::new(r"(?m)spawn\s+\w+\s*\{").expect("Hardcoded regex pattern must be valid");

        let send_matches: Vec<_> = send_pattern.find_iter(content).collect();
        let query_matches: Vec<_> = query_pattern.find_iter(content).collect();
        let spawn_matches: Vec<_> = spawn_pattern.find_iter(content).collect();

        let total_messages = send_matches.len() + query_matches.len();

        if total_messages > 2 || spawn_matches.len() > 1 {
            let pattern_hash =
                self.hash_pattern(&format!("ruchy_messaging_{}", file_path.display()));
            let mut locations = Vec::new();

            for (i, m) in send_matches.iter().chain(query_matches.iter()).enumerate() {
                let line_num = content.get(..m.start()).unwrap_or_default().lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });

                if i >= 10 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::ApiCall, // Message passing is like API calls
                pattern_hash,
                frequency: total_messages.max(spawn_matches.len()),
                locations,
                variation_score: self.calculate_messaging_variation_score(
                    &send_matches,
                    &query_matches,
                    content,
                ),
                example_code: send_matches
                    .first()
                    .or(query_matches.first())
                    .map(|m| {
                        content
                            .get(m.start()..m.end().min(m.start() + 80))
                            .unwrap_or_default()
                            .to_string()
                    })
                    .unwrap_or_default(),
                estimated_loc: total_messages * 2 + spawn_matches.len() * 3,
            };

            collection.add_pattern(pattern);
        }

        Ok(())
    }

    /// Extract Ruchy-specific error handling patterns
    fn extract_ruchy_error_handling_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!content.is_empty(), "content must not be empty");
        use regex::Regex;

        // Pattern: Result<T, E> with match statements (Ruchy style)
        let result_match_pattern = Regex::new(r"(?m)match\s+.*Result\s*<.*>\s*\{")
            .expect("Hardcoded regex pattern must be valid");
        let matches: Vec<_> = result_match_pattern.find_iter(content).collect();

        if matches.len() > 1 {
            let pattern_hash =
                self.hash_pattern(&format!("ruchy_error_handling_{}", file_path.display()));
            let mut locations = Vec::new();

            for (i, m) in matches.iter().enumerate() {
                let line_num = content.get(..m.start()).unwrap_or_default().lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });

                if i >= 8 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::ErrorHandling,
                pattern_hash,
                frequency: matches.len(),
                locations,
                variation_score: self.calculate_variation_score(&matches, content),
                example_code: matches
                    .first()
                    .map(|m| {
                        content
                            .get(m.start()..m.end().min(m.start() + 120))
                            .unwrap_or_default()
                            .to_string()
                    })
                    .unwrap_or_default(),
                estimated_loc: matches.len() * 6, // Error handling typically 6 lines
            };

            collection.add_pattern(pattern);
        }

        Ok(())
    }

    /// Extract Ruchy pattern matching patterns
    fn extract_ruchy_pattern_matching_patterns(
        &self,
        file_path: &Path,
        content: &str,
        collection: &mut PatternCollection,
    ) -> Result<()> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        debug_assert!(!content.is_empty(), "content must not be empty");
        use regex::Regex;

        // Pattern: enum matching with => arrows
        let enum_pattern =
            Regex::new(r"(?m)enum\s+\w+\s*\{").expect("Hardcoded regex pattern must be valid");
        let match_pattern =
            Regex::new(r"(?m)match\s+\w+\s*\{").expect("Hardcoded regex pattern must be valid");
        let arrow_pattern =
            Regex::new(r"(?m)\w+::\w+\s*=>\s*").expect("Hardcoded regex pattern must be valid");

        let enum_matches: Vec<_> = enum_pattern.find_iter(content).collect();
        let match_matches: Vec<_> = match_pattern.find_iter(content).collect();
        let arrow_matches: Vec<_> = arrow_pattern.find_iter(content).collect();

        if match_matches.len() > 1 && arrow_matches.len() > 6 {
            // Multiple matches with many arms
            let pattern_hash =
                self.hash_pattern(&format!("ruchy_pattern_matching_{}", file_path.display()));
            let mut locations = Vec::new();

            for (i, m) in match_matches.iter().enumerate() {
                let line_num = content.get(..m.start()).unwrap_or_default().lines().count() + 1;
                locations.push(Location {
                    file: file_path.to_owned(),
                    line: line_num,
                    column: 1,
                });

                if i >= 8 {
                    break;
                }
            }

            let pattern = AstPattern {
                pattern_type: PatternType::ControlFlow,
                pattern_hash,
                frequency: match_matches.len(),
                locations,
                variation_score: self.calculate_pattern_match_variation_score(
                    &enum_matches,
                    &match_matches,
                    &arrow_matches,
                    content,
                ),
                example_code: match_matches
                    .first()
                    .map(|m| {
                        content
                            .get(m.start()..m.end().min(m.start() + 150))
                            .unwrap_or_default()
                            .to_string()
                    })
                    .unwrap_or_default(),
                estimated_loc: match_matches.len() * 5 + arrow_matches.len(),
            };

            collection.add_pattern(pattern);
        }

        Ok(())
    }

    // Ruchy-specific variation score calculation methods

    fn calculate_actor_variation_score(
        &self,
        actor_matches: &[regex::Match],
        _receive_matches: &[regex::Match],
        content: &str,
    ) -> f64 {
        debug_assert!(!content.is_empty(), "content must not be empty");
        if actor_matches.is_empty() {
            return 0.0;
        }

        // Calculate variation based on different actor names and receive handler patterns
        let mut unique_patterns = std::collections::HashSet::new();

        for m in actor_matches {
            if let Some(actor_line) = content
                .lines()
                .nth(content.get(..m.start()).unwrap_or_default().lines().count())
            {
                unique_patterns.insert(actor_line.trim().to_string());
            }
        }

        let variation = unique_patterns.len() as f64 / actor_matches.len() as f64;
        variation.min(1.0)
    }

    fn calculate_pipeline_variation_score(&self, matches: &[regex::Match], content: &str) -> f64 {
        debug_assert!(!content.is_empty(), "content must not be empty");
        if matches.len() < 2 {
            return 0.0;
        }

        // Calculate variation based on different pipeline operations
        let mut unique_operations = std::collections::HashSet::new();

        for m in matches {
            if let Some(op_text) = content.get(m.start()..m.end()) {
                unique_operations.insert(op_text.trim().to_string());
            }
        }

        let variation = unique_operations.len() as f64 / matches.len() as f64;
        variation.min(1.0)
    }

    fn calculate_messaging_variation_score(
        &self,
        send_matches: &[regex::Match],
        query_matches: &[regex::Match],
        content: &str,
    ) -> f64 {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let total_matches = send_matches.len() + query_matches.len();
        if total_matches < 2 {
            return 0.0;
        }

        let mut unique_patterns = std::collections::HashSet::new();

        for m in send_matches.iter().chain(query_matches.iter()) {
            if let Some(msg_text) = content.get(m.start()..m.end()) {
                unique_patterns.insert(msg_text.trim().to_string());
            }
        }

        let variation = unique_patterns.len() as f64 / total_matches as f64;
        variation.min(1.0)
    }

    fn calculate_pattern_match_variation_score(
        &self,
        enum_matches: &[regex::Match],
        match_matches: &[regex::Match],
        _arrow_matches: &[regex::Match],
        content: &str,
    ) -> f64 {
        debug_assert!(!content.is_empty(), "content must not be empty");
        if match_matches.len() < 2 {
            return 0.0;
        }

        // Higher variation if we have different enum types being matched
        let enum_variation = if enum_matches.len() > 1 {
            0.6 // Different enum types = medium variation
        } else {
            0.3 // Same enum type = low variation
        };

        // Calculate variation based on match statement patterns
        let mut unique_match_patterns = std::collections::HashSet::new();

        for m in match_matches {
            if let Some(match_text) = content.get(m.start()..m.start().saturating_add(50)) {
                unique_match_patterns.insert(match_text.trim().to_string());
            }
        }

        let match_variation = unique_match_patterns.len() as f64 / match_matches.len() as f64;

        ((enum_variation + match_variation) / 2.0).min(1.0)
    }
}
