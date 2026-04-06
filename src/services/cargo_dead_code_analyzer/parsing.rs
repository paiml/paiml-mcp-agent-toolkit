// Parsing and metrics calculation for CargoDeadCodeAnalyzer
// Included from cargo_dead_code_analyzer.rs - shares parent module scope

impl CargoDeadCodeAnalyzer {
    /// Parse cargo's JSON output for dead code warnings
    fn parse_cargo_warnings(&self, output: &str) -> Result<Vec<(PathBuf, DeadItem)>> {
        debug_assert!(!output.is_empty(), "output must not be empty");
        let mut dead_items = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let json: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue, // Skip non-JSON lines
            };

            // Check if this is a compiler message
            if json["reason"] != "compiler-message" {
                continue;
            }

            let message = &json["message"];

            // Check if this is a dead code warning
            if let Some(code) = message["code"]["code"].as_str() {
                if code == "dead_code" {
                    if let Some(item) = self.extract_dead_item(message) {
                        dead_items.push(item);
                    }
                }
            }
        }

        Ok(dead_items)
    }

    /// Extract dead code item from compiler message
    fn extract_dead_item(&self, message: &Value) -> Option<(PathBuf, DeadItem)> {
        let spans = message["spans"].as_array()?;
        let primary_span = spans
            .iter()
            .find(|s| s["is_primary"].as_bool() == Some(true))?;

        let file_path = PathBuf::from(primary_span["file_name"].as_str()?);
        let line = primary_span["line_start"].as_u64()? as usize;
        let column = primary_span["column_start"].as_u64()? as usize;

        let message_text = message["message"].as_str()?;
        let (name, kind) = self.parse_message(message_text)?;

        Some((
            file_path,
            DeadItem {
                name,
                kind,
                line,
                column,
                message: message_text.to_string(),
            },
        ))
    }

    /// Parse the warning message to extract name and kind
    fn parse_message(&self, message: &str) -> Option<(String, DeadCodeKind)> {
        // Common patterns in dead code messages
        let patterns = [
            ("function `", "` is never used", DeadCodeKind::Function),
            ("method `", "` is never used", DeadCodeKind::Method),
            ("struct `", "` is never constructed", DeadCodeKind::Struct),
            ("enum `", "` is never used", DeadCodeKind::Enum),
            ("variant `", "` is never constructed", DeadCodeKind::Variant),
            ("field `", "` is never read", DeadCodeKind::Field),
            ("constant `", "` is never used", DeadCodeKind::Constant),
            ("static `", "` is never used", DeadCodeKind::Static),
            ("module `", "` is never used", DeadCodeKind::Module),
            ("trait `", "` is never used", DeadCodeKind::Trait),
            ("type alias `", "` is never used", DeadCodeKind::TypeAlias),
        ];

        for (prefix, suffix, kind) in &patterns {
            if let Some(start) = message.find(prefix) {
                let name_start = start + prefix.len();
                if let Some(end) = message[name_start..].find(suffix) {
                    let name = message[name_start..name_start + end].to_string();
                    return Some((name, kind.clone()));
                }
            }
        }

        // Fallback for unknown patterns
        if message.contains("is never") || message.contains("never used") {
            // Try to extract name between backticks
            if let Some(start) = message.find('`') {
                if let Some(end) = message[start + 1..].find('`') {
                    let name = message[start + 1..start + 1 + end].to_string();
                    return Some((name, DeadCodeKind::Other("unknown".to_string())));
                }
            }
        }

        None
    }

    /// Group dead items by file
    fn group_by_file(&self, items: Vec<(PathBuf, DeadItem)>) -> Vec<FileDeadCode> {
        debug_assert!(!items.is_empty(), "items must not be empty");
        let mut file_map: HashMap<PathBuf, Vec<DeadItem>> = HashMap::new();

        for (path, item) in items {
            file_map.entry(path).or_default().push(item);
        }

        file_map
            .into_iter()
            .map(|(file_path, dead_items)| {
                // Calculate file-specific percentage
                let file_dead_percentage = self
                    .calculate_file_percentage(&file_path, &dead_items)
                    .unwrap_or(0.0);

                FileDeadCode {
                    file_path,
                    dead_items,
                    file_dead_percentage,
                }
            })
            .collect()
    }

    /// Calculate dead code percentage for a specific file
    fn calculate_file_percentage(&self, file_path: &Path, dead_items: &[DeadItem]) -> Result<f64> {
        debug_assert!(file_path.exists(), "file_path must exist: {}", file_path.display());
        let full_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            self.project_path.join(file_path)
        };

        if !full_path.exists() {
            return Ok(0.0);
        }

        let content =
            std::fs::read_to_string(&full_path).context("Failed to read file for line counting")?;

        let total_lines = content.lines().count();
        if total_lines == 0 {
            return Ok(0.0);
        }

        // Estimate dead lines (approximate 3-5 lines per item)
        let estimated_dead_lines = dead_items.len() * 4;
        let percentage = (estimated_dead_lines as f64 / total_lines as f64) * 100.0;

        Ok(percentage.min(100.0))
    }

    /// Calculate overall metrics
    async fn calculate_metrics(&self, files: Vec<FileDeadCode>) -> Result<AccurateDeadCodeReport> {
        debug_assert!(!files.is_empty(), "files must not be empty");
        let mut total_lines = 0;
        let mut dead_lines = 0;
        let mut dead_by_type = HashMap::new();
        let total_dead_items = files.iter().map(|f| f.dead_items.len()).sum();

        // Count lines in all Rust files (with max depth limit to prevent hanging)
        for entry in walkdir::WalkDir::new(&self.project_path)
            .max_depth(self.max_depth) // Critical fix: limit traversal depth
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();

            // Skip target directory and non-Rust files
            if path.starts_with(self.project_path.join("target")) {
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    total_lines += content.lines().count();
                }
            }
        }

        // Count dead lines and categorize by type
        for file in &files {
            for item in &file.dead_items {
                let kind_str = dead_code_kind_to_str(&item.kind);
                *dead_by_type.entry(kind_str.to_string()).or_insert(0) += 1;

                // Estimate lines per item type
                let lines = match item.kind {
                    DeadCodeKind::Function | DeadCodeKind::Method => 5,
                    DeadCodeKind::Struct | DeadCodeKind::Enum => 3,
                    _ => 2,
                };
                dead_lines += lines;
            }
        }

        let dead_code_percentage = if total_lines > 0 {
            (dead_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        Ok(AccurateDeadCodeReport {
            files_with_dead_code: files,
            total_dead_items,
            dead_code_percentage,
            total_lines,
            dead_lines,
            dead_by_type,
        })
    }
}

/// Convert DeadCodeKind to string representation
fn dead_code_kind_to_str(kind: &DeadCodeKind) -> &str {
    match kind {
        DeadCodeKind::Function => "function",
        DeadCodeKind::Method => "method",
        DeadCodeKind::Struct => "struct",
        DeadCodeKind::Enum => "enum",
        DeadCodeKind::Variant => "variant",
        DeadCodeKind::Field => "field",
        DeadCodeKind::Constant => "constant",
        DeadCodeKind::Static => "static",
        DeadCodeKind::Module => "module",
        DeadCodeKind::Trait => "trait",
        DeadCodeKind::TypeAlias => "type_alias",
        DeadCodeKind::Suppressed => "suppressed",
        DeadCodeKind::Other(s) => s,
    }
}
