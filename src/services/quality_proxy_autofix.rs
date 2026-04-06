impl QualityProxyService {
    async fn auto_fix_content(
        &self,
        content: &str,
        file_path: &str,
        extension: &str,
        config: &QualityConfig,
    ) -> Result<(String, Vec<HashMap<String, serde_json::Value>>)> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        debug_assert!(!file_path.is_empty(), "file_path must not be empty");
        if extension != "rs" {
            return Ok((content.to_string(), Vec::new()));
        }

        info!("Applying auto-fix refactoring to {}", file_path);

        // For now, we'll apply basic fixes:
        // 1. Remove SATD comments
        // 2. Add missing documentation
        // 3. Format code

        let mut fixed_content = content.to_string();
        let mut plan = Vec::new();

        // Remove SATD comments
        if !config.allow_satd {
            let satd_patterns = vec![
                r"//\s*TODO:.*\n",
                r"//\s*FIXME:.*\n",
                r"//\s*HACK:.*\n",
                r"//\s*BUG:.*\n",
            ];

            for pattern in satd_patterns {
                let re = regex::Regex::new(pattern)?;
                if re.is_match(&fixed_content) {
                    fixed_content = re.replace_all(&fixed_content, "").to_string();
                    let mut step = HashMap::new();
                    step.insert("action".to_string(), serde_json::json!("remove_satd"));
                    step.insert("pattern".to_string(), serde_json::json!(pattern));
                    plan.push(step);
                }
            }
        }

        // Add basic documentation for public items
        if config.require_docs {
            let lines: Vec<&str> = fixed_content.lines().collect();
            let mut new_lines = Vec::new();

            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("pub fn")
                    || trimmed.starts_with("pub struct")
                    || trimmed.starts_with("pub enum")
                {
                    let prev_has_doc = i > 0 && lines[i - 1].trim().starts_with("///");
                    if !prev_has_doc {
                        let item_name = trimmed
                            .split_whitespace()
                            .nth(2)
                            .unwrap_or("item")
                            .split('(')
                            .next()
                            .unwrap_or("item");

                        new_lines.push(format!("/// {item_name}"));
                        let mut step = HashMap::new();
                        step.insert("action".to_string(), serde_json::json!("add_documentation"));
                        step.insert("item".to_string(), serde_json::json!(item_name));
                        plan.push(step);
                    }
                }
                new_lines.push((*line).to_string());
            }

            if !plan.is_empty() {
                fixed_content = new_lines.join("\n");
            }
        }

        // Format code using rustfmt
        if config.auto_format {
            if let Ok(formatted) = self.format_rust_code(&fixed_content).await {
                if formatted != fixed_content {
                    fixed_content = formatted;
                    let mut step = HashMap::new();
                    step.insert("action".to_string(), serde_json::json!("format_code"));
                    plan.push(step);
                }
            }
        }

        Ok((fixed_content, plan))
    }
}
