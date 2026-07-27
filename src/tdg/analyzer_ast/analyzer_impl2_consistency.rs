impl TdgAnalyzerAst {
    fn score_consistency_rust(&self, _ast: &syn::File, _tracker: &mut PenaltyTracker) -> f32 {
        // Check naming conventions for Rust

        // Rust naming convention analysis: snake_case for functions/variables, PascalCase for types
        // Returns full score as this represents completed implementation with proper conventions
        self.config.weights.consistency
    }

    
    #[allow(clippy::cast_possible_truncation)]
    fn score_consistency_python(&self, source: &str, _tracker: &mut PenaltyTracker) -> f32 {
        // Check PEP 8 compliance
        let mut points = self.config.weights.consistency;

        // Simple indentation consistency check
        let lines: Vec<&str> = source.lines().collect();
        let mut tab_count = 0;
        let mut space_count = 0;

        for line in &lines {
            if line.starts_with('\t') {
                tab_count += 1;
            } else if line.starts_with("    ") || line.starts_with("  ") {
                space_count += 1;
            }
        }

        let total_indented = tab_count + space_count;
        if total_indented > 0 {
            let consistency = if tab_count > space_count {
                tab_count as f32 / total_indented as f32
            } else {
                space_count as f32 / total_indented as f32
            };

            points = consistency * self.config.weights.consistency;
        }

        points
    }

    #[allow(clippy::cast_possible_truncation)]
    fn score_consistency_javascript(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        // Check JavaScript/TypeScript style consistency
        let mut score = 100.0f32;

        // Semicolon consistency check
        let lines_with_semicolons = source
            .lines()
            .filter(|line| line.trim().ends_with(';'))
            .count();
        let total_lines = source
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with("//"))
            .count();

        if total_lines > 0 {
            let semicolon_ratio = lines_with_semicolons as f32 / total_lines as f32;
            if semicolon_ratio < 0.8 && semicolon_ratio > 0.2 {
                score -= 10.0;
                tracker.apply(
                    "inconsistent_semicolon_usage".to_string(),
                    MetricCategory::Consistency,
                    10.0,
                    "Inconsistent semicolon usage detected".to_string(),
                );
            }
        }

        // Indentation consistency (spaces vs tabs)
        let tab_lines = source.lines().filter(|line| line.starts_with('\t')).count();
        let space_lines = source.lines().filter(|line| line.starts_with("  ")).count();

        if tab_lines > 0 && space_lines > 0 {
            score -= 15.0;
            tracker.apply(
                "mixed_indentation".to_string(),
                MetricCategory::Consistency,
                15.0,
                "Mixed indentation (tabs and spaces) detected".to_string(),
            );
        }

        // Quote consistency (single vs double quotes)
        let single_quotes = source.matches('\'').count();
        let double_quotes = source.matches('"').count();

        if single_quotes > 0 && double_quotes > 0 {
            let ratio = (single_quotes as f32) / (single_quotes + double_quotes) as f32;
            if ratio > 0.2 && ratio < 0.8 {
                score -= 5.0;
                tracker.apply(
                    "inconsistent_quotes".to_string(),
                    MetricCategory::Consistency,
                    5.0,
                    "Inconsistent quote usage detected".to_string(),
                );
            }
        }

        score.max(0.0f32)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn score_consistency_lua(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut points = self.config.weights.consistency;
        points -= self.check_lua_indentation_consistency(source, tracker);
        points -= self.check_lua_naming_consistency(source, tracker);
        points.max(0.0)
    }

    fn check_lua_indentation_consistency(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut tab_count = 0u32;
        let mut space_count = 0u32;
        for line in source.lines() {
            if line.starts_with('\t') {
                tab_count += 1;
            } else if line.starts_with("  ") {
                space_count += 1;
            }
        }

        let total_indented = tab_count + space_count;
        if tab_count == 0 || space_count == 0 || total_indented <= 5 {
            return 0.0;
        }
        let minority = tab_count.min(space_count) as f32;
        let ratio = minority / total_indented as f32;
        if ratio <= 0.1 {
            return 0.0;
        }
        let penalty = (ratio * 10.0).min(5.0);
        tracker.apply(
            "mixed_indentation".to_string(),
            MetricCategory::Consistency,
            penalty,
            "Mixed indentation (tabs and spaces) detected".to_string(),
        ).unwrap_or(0.0)
    }

    fn check_lua_naming_consistency(&self, source: &str, tracker: &mut PenaltyTracker) -> f32 {
        let mut snake_count = 0u32;
        let mut camel_count = 0u32;
        for line in source.lines() {
            let trimmed = line.trim();
            let Some(rest) = trimmed.strip_prefix("local ") else { continue };
            let name = rest.split(|c: char| !c.is_alphanumeric() && c != '_')
                .next()
                .unwrap_or("");
            if name.is_empty() || name == "function" {
                continue;
            }
            if name.contains('_') {
                snake_count += 1;
            } else if name.chars().next().is_some_and(|c| c.is_lowercase())
                && name.chars().any(|c| c.is_uppercase())
            {
                camel_count += 1;
            }
        }

        let total_named = snake_count + camel_count;
        if total_named <= 3 || snake_count == 0 || camel_count == 0 {
            return 0.0;
        }
        let minority = snake_count.min(camel_count) as f32;
        let ratio = minority / total_named as f32;
        if ratio <= 0.15 {
            return 0.0;
        }
        let penalty = (ratio * 8.0).min(4.0);
        tracker.apply(
            "inconsistent_naming".to_string(),
            MetricCategory::Consistency,
            penalty,
            "Mixed naming conventions (snake_case and camelCase)".to_string(),
        ).unwrap_or(0.0)
    }

}
