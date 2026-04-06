impl RustAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_rust::language())?;
        Ok(Self { parser })
    }
}

impl LanguageAdapter for RustAdapter {
    fn detect(&self, path: &Path) -> bool {
        path.extension().is_some_and(|ext| ext == "rs")
    }

    fn parse(&self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Rust source"))
    }

    fn confidence(&self) -> f32 {
        1.0
    }

    fn language(&self) -> Language {
        Language::Rust
    }

    fn naming_rules(&self) -> LanguageRules {
        LanguageRules::rust_rules()
    }
}

impl PythonAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::language())?;
        Ok(Self { parser })
    }
}

impl LanguageAdapter for PythonAdapter {
    fn detect(&self, path: &Path) -> bool {
        path.extension().is_some_and(|ext| ext == "py")
    }

    fn parse(&self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Python source"))
    }

    fn confidence(&self) -> f32 {
        0.95
    }

    fn language(&self) -> Language {
        Language::Python
    }

    fn naming_rules(&self) -> LanguageRules {
        LanguageRules::python_rules()
    }
}

impl JavaScriptAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_javascript::LANGUAGE.into())?;
        Ok(Self { parser })
    }
}

impl LanguageAdapter for JavaScriptAdapter {
    fn detect(&self, path: &Path) -> bool {
        path.extension().is_some_and(|ext| ext == "js" || ext == "jsx")
    }

    fn parse(&self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse JavaScript source"))
    }

    fn confidence(&self) -> f32 {
        0.90
    }

    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn naming_rules(&self) -> LanguageRules {
        LanguageRules::javascript_rules()
    }
}

impl TypeScriptAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_typescript::language_typescript())?;
        Ok(Self { parser })
    }
}

impl LanguageAdapter for TypeScriptAdapter {
    fn detect(&self, path: &Path) -> bool {
        path.extension().is_some_and(|ext| ext == "ts" || ext == "tsx")
    }

    fn parse(&self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse TypeScript source"))
    }

    fn confidence(&self) -> f32 {
        0.90
    }

    fn language(&self) -> Language {
        Language::TypeScript
    }

    fn naming_rules(&self) -> LanguageRules {
        LanguageRules::typescript_rules()
    }
}

impl GoAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_go::language())?;
        Ok(Self { parser })
    }
}

impl LanguageAdapter for GoAdapter {
    fn detect(&self, path: &Path) -> bool {
        path.extension().is_some_and(|ext| ext == "go")
    }

    fn parse(&self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Go source"))
    }

    fn confidence(&self) -> f32 {
        0.95
    }

    fn language(&self) -> Language {
        Language::Go
    }

    fn naming_rules(&self) -> LanguageRules {
        LanguageRules::go_rules()
    }
}

impl LuaAdapter {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_lua::LANGUAGE.into())?;
        Ok(Self { parser })
    }
}

impl LanguageAdapter for LuaAdapter {
    fn detect(&self, path: &Path) -> bool {
        path.extension().is_some_and(|ext| ext == "lua")
    }

    fn parse(&self, source: &str) -> Result<Tree> {
        self.parser.parse(source, None)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse Lua source"))
    }

    fn confidence(&self) -> f32 {
        0.90
    }

    fn language(&self) -> Language {
        Language::Lua
    }

    fn naming_rules(&self) -> LanguageRules {
        LanguageRules::lua_rules()
    }
}

impl LanguageRegistry {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Result<Self> {
        let mut registry = Self {
            adapters: HashMap::new(),
        };

        registry.register(Language::Rust, Box::new(RustAdapter::new()?))?;
        registry.register(Language::Python, Box::new(PythonAdapter::new()?))?;
        registry.register(Language::JavaScript, Box::new(JavaScriptAdapter::new()?))?;
        registry.register(Language::TypeScript, Box::new(TypeScriptAdapter::new()?))?;
        registry.register(Language::Go, Box::new(GoAdapter::new()?))?;
        registry.register(Language::Lua, Box::new(LuaAdapter::new()?))?;

        Ok(registry)
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn register(&mut self, language: Language, adapter: Box<dyn LanguageAdapter>) -> Result<()> {
        self.adapters.insert(language, adapter);
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_adapter(&self, language: Language) -> Result<&dyn LanguageAdapter> {
        self.adapters
            .get(&language)
            .map(|a| a.as_ref())
            .ok_or_else(|| anyhow::anyhow!("No adapter for language: {}", language))
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn detect_and_get(&self, path: &Path) -> Result<&dyn LanguageAdapter> {
        let language = Language::from_extension(path);
        if language == Language::Unknown {
            return Err(anyhow::anyhow!("Unknown language for file: {}", path.display()));
        }
        self.get_adapter(language)
    }
}
