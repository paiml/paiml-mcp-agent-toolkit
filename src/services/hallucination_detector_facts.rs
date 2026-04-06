// CodeFactDatabase implementation: ground truth storage from codebase analysis,
// markdown parsing, function/language/capability lookups.

impl CodeFactDatabase {
    /// Create empty fact database
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            languages: Vec::new(),
            capabilities: Vec::new(),
        }
    }

    /// Load facts from deep context markdown
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_markdown(content: &str) -> Result<Self> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut db = Self::new();

        // Parse functions from "Functions:" sections
        let function_regex = Regex::new(r"(?m)^-\s+([a-zA-Z_][a-zA-Z0-9_]*)\(\)")?;
        for caps in function_regex.captures_iter(content) {
            let function_name = caps.get(1).expect("internal error").as_str().to_string();
            db.functions
                .entry(function_name.clone())
                .or_default()
                .push("".to_string());
        }

        // Parse supported languages from "Supported languages:" sections
        let language_regex = Regex::new(
            r"(?m)^-\s+(Rust|TypeScript|JavaScript|Python|C|C\+\+|Go|Java|Kotlin|Ruby|PHP|Swift|C#|Bash|WASM|Haskell|Elixir|Erlang|OCaml)",
        )?;
        for caps in language_regex.captures_iter(content) {
            let language = caps.get(1).expect("internal error").as_str().to_string();
            if !db.languages.contains(&language) {
                db.languages.push(language);
            }
        }

        Ok(db)
    }

    /// Check if database contains a function
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_function(&self, name: &str) -> bool {
        debug_assert!(!name.is_empty(), "name must not be empty");
        self.functions.contains_key(name)
    }

    /// Check if database supports a language
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_language_support(&self, language: &str) -> bool {
        debug_assert!(!language.is_empty(), "language must not be empty");
        self.languages.iter().any(|l| l == language)
    }

    /// Add capability to database
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn add_capability(&mut self, capability: String) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// Check if database has a capability
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has_capability(&self, capability: &str) -> bool {
        debug_assert!(!capability.is_empty(), "capability must not be empty");
        self.capabilities.iter().any(|c| c == capability)
    }
}

impl Default for CodeFactDatabase {
    fn default() -> Self {
        Self::new()
    }
}
