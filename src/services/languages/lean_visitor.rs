// LeanAstVisitor implementation methods
// Extracts Lean 4 AST items from source code: definitions, theorems, types, instances, axioms

impl LeanAstVisitor {
    /// Creates a new Lean AST visitor
    #[must_use]
    pub fn new(file_path: &Path) -> Self {
        Self {
            items: Vec::new(),
            _file_path: file_path.to_path_buf(),
            namespace: String::new(),
        }
    }

    /// Analyzes Lean 4 source code and extracts AST items (single-pass, complexity <=10)
    pub fn analyze_lean_source(mut self, source: &str) -> Result<Vec<AstItem>, String> {
        if source.trim().is_empty() {
            return Ok(vec![]);
        }

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Track namespace changes inline (single-pass)
            if trimmed.starts_with("namespace ") {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    self.namespace = parts[1].to_string();
                }
                continue;
            }

            self.extract_line(trimmed, line_num);
        }

        Ok(self.items)
    }

    /// Extracts AST items from a single line (complexity <=10)
    fn extract_line(&mut self, trimmed: &str, line_num: usize) {
        // Try definitions first
        if let Some(item) = self.try_extract_definition(trimmed, line_num) {
            self.items.push(item);
            return;
        }
        // Try theorems/lemmas
        if let Some(item) = self.try_extract_theorem(trimmed, line_num) {
            self.items.push(item);
            return;
        }
        // Try types (structure, class, inductive)
        if let Some(item) = self.try_extract_type(trimmed, line_num) {
            self.items.push(item);
            return;
        }
        // Try instances
        if let Some(item) = self.try_extract_instance(trimmed, line_num) {
            self.items.push(item);
            return;
        }
        // Try axioms/opaque
        if let Some(item) = self.try_extract_axiom(trimmed, line_num) {
            self.items.push(item);
        }
    }

    /// Tries to extract a def/abbrev from a line (complexity <=10)
    fn try_extract_definition(&self, trimmed: &str, line_num: usize) -> Option<AstItem> {
        let name = if trimmed.starts_with("def ") {
            Self::extract_name_after_keyword(trimmed, "def")
        } else if trimmed.starts_with("noncomputable def ") {
            Self::extract_name_after_keyword(trimmed, "def")
        } else if trimmed.starts_with("abbrev ") {
            Self::extract_name_after_keyword(trimmed, "abbrev")
        } else if trimmed.starts_with("partial def ") {
            Self::extract_name_after_keyword(trimmed, "def")
        } else if trimmed.starts_with("private def ") {
            Self::extract_name_after_keyword(trimmed, "def")
        } else if trimmed.starts_with("protected def ") {
            Self::extract_name_after_keyword(trimmed, "def")
        } else {
            None
        }?;

        let qualified = self.get_qualified_name(&name);
        let visibility = if trimmed.starts_with("private ") {
            "private"
        } else {
            "public"
        };
        Some(AstItem::Function {
            name: qualified,
            visibility: visibility.to_string(),
            is_async: false,
            line: line_num + 1,
        })
    }

    /// Tries to extract a theorem/lemma from a line (complexity <=10)
    fn try_extract_theorem(&self, trimmed: &str, line_num: usize) -> Option<AstItem> {
        let name = if trimmed.starts_with("theorem ") {
            Self::extract_name_after_keyword(trimmed, "theorem")
        } else if trimmed.starts_with("lemma ") {
            Self::extract_name_after_keyword(trimmed, "lemma")
        } else if trimmed.starts_with("private theorem ") {
            Self::extract_name_after_keyword(trimmed, "theorem")
        } else if trimmed.starts_with("private lemma ") {
            Self::extract_name_after_keyword(trimmed, "lemma")
        } else {
            None
        }?;

        let qualified = self.get_qualified_name(&name);
        Some(AstItem::Function {
            name: qualified,
            visibility: "public".to_string(),
            is_async: false,
            line: line_num + 1,
        })
    }

    /// Tries to extract a structure/class/inductive from a line (complexity <=10)
    fn try_extract_type(&self, trimmed: &str, line_num: usize) -> Option<AstItem> {
        let name = if trimmed.starts_with("structure ") {
            Self::extract_name_after_keyword(trimmed, "structure")
        } else if trimmed.starts_with("class ") {
            Self::extract_name_after_keyword(trimmed, "class")
        } else if trimmed.starts_with("inductive ") {
            Self::extract_name_after_keyword(trimmed, "inductive")
        } else {
            None
        }?;

        let qualified = self.get_qualified_name(&name);
        Some(AstItem::Struct {
            name: qualified,
            visibility: "public".to_string(),
            fields_count: 0,
            derives: vec![],
            line: line_num + 1,
        })
    }

    /// Tries to extract an instance from a line (complexity <=10)
    fn try_extract_instance(&self, trimmed: &str, line_num: usize) -> Option<AstItem> {
        if !trimmed.starts_with("instance ") {
            return None;
        }
        let name = Self::extract_name_after_keyword(trimmed, "instance")?;
        let qualified = self.get_qualified_name(&name);
        Some(AstItem::Function {
            name: qualified,
            visibility: "public".to_string(),
            is_async: false,
            line: line_num + 1,
        })
    }

    /// Tries to extract an axiom/opaque from a line (complexity <=10)
    fn try_extract_axiom(&self, trimmed: &str, line_num: usize) -> Option<AstItem> {
        let name = if trimmed.starts_with("axiom ") {
            Self::extract_name_after_keyword(trimmed, "axiom")
        } else if trimmed.starts_with("opaque ") {
            Self::extract_name_after_keyword(trimmed, "opaque")
        } else {
            None
        }?;

        let qualified = self.get_qualified_name(&name);
        Some(AstItem::Function {
            name: qualified,
            visibility: "public".to_string(),
            is_async: false,
            line: line_num + 1,
        })
    }

    /// Extracts name after a keyword (complexity <=10)
    fn extract_name_after_keyword(line: &str, keyword: &str) -> Option<String> {
        // Find the keyword position and get the word after it
        if let Some(pos) = line.find(keyword) {
            let after = &line[pos + keyword.len()..].trim_start();
            let name = after
                .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '.')
                .next()
                .unwrap_or("");
            if !name.is_empty() {
                Some(name.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Gets qualified name for a symbol (complexity <=10)
    fn get_qualified_name(&self, name: &str) -> String {
        if self.namespace.is_empty() {
            name.to_string()
        } else {
            format!("{}::{}", self.namespace, name)
        }
    }
}
