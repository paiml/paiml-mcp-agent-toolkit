impl IncrementalCoverageAnalyzer {
    async fn parse_file(&self, path: &Path) -> Result<AstNode> {
        let content = tokio::fs::read_to_string(path).await?;
        let hash = blake3::hash(content.as_bytes());

        // Check AST cache
        let file_id = FileId {
            path: path.to_path_buf(),
            hash: *hash.as_bytes(),
        };

        if let Some(cached) = self.ast_cache.get(&file_id) {
            return Ok(cached.1.clone());
        }

        // Parse based on file extension
        let ast = match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => self.parse_rust_file(&content)?,
            Some("ts" | "tsx" | "js" | "jsx") => self.parse_typescript_file(&content)?,
            Some("py") => self.parse_python_file(&content)?,
            _ => AstNode {
                functions: vec![],
                dependencies: vec![],
            },
        };

        self.ast_cache.insert(file_id, (0, ast.clone()));
        Ok(ast)
    }

    fn parse_rust_file(&self, content: &str) -> Result<AstNode> {
        // Simplified parsing - in production would use syn
        let mut functions = Vec::new();
        let mut dependencies = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("fn ") {
                if let Some(name) = line.split_whitespace().nth(1) {
                    functions.push(FunctionInfo {
                        name: name.trim_end_matches('(').to_string(),
                        start_line: line_num + 1,
                        end_line: line_num + 10, // Simplified
                        complexity: 1,
                    });
                }
            }

            if line.trim().starts_with("use ") {
                if let Some(dep) = line.split_whitespace().nth(1) {
                    dependencies.push(dep.trim_end_matches(';').to_string());
                }
            }
        }

        Ok(AstNode {
            functions,
            dependencies,
        })
    }

    fn parse_typescript_file(&self, content: &str) -> Result<AstNode> {
        // Simplified parsing
        let mut functions = Vec::new();
        let mut dependencies = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("function ") || line.contains("const ") && line.contains("=>") {
                if let Some(name) = extract_function_name(line) {
                    functions.push(FunctionInfo {
                        name,
                        start_line: line_num + 1,
                        end_line: line_num + 10,
                        complexity: 1,
                    });
                }
            }

            if line.trim().starts_with("import ") {
                dependencies.push(line.to_string());
            }
        }

        Ok(AstNode {
            functions,
            dependencies,
        })
    }

    fn parse_python_file(&self, content: &str) -> Result<AstNode> {
        // Simplified parsing
        let mut functions = Vec::new();
        let mut dependencies = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("def ") {
                if let Some(name) = line.split_whitespace().nth(1) {
                    functions.push(FunctionInfo {
                        name: name.trim_end_matches('(').trim_end_matches(':').to_string(),
                        start_line: line_num + 1,
                        end_line: line_num + 10,
                        complexity: 1,
                    });
                }
            }

            if line.trim().starts_with("import ") || line.trim().starts_with("from ") {
                dependencies.push(line.to_string());
            }
        }

        Ok(AstNode {
            functions,
            dependencies,
        })
    }
}

fn extract_function_name(line: &str) -> Option<String> {
    // Simplified function name extraction
    if let Some(pos) = line.find("function ") {
        let start = pos + 9;
        if let Some(end) = line.get(start..).unwrap_or_default().find('(') {
            return Some(
                line.get(start..start + end)
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
            );
        }
    }

    if let Some(pos) = line.find("const ") {
        let start = pos + 6;
        if let Some(eq) = line.get(start..).unwrap_or_default().find(" =") {
            return Some(
                line.get(start..start + eq)
                    .unwrap_or_default()
                    .trim()
                    .to_string(),
            );
        }
    }

    None
}
