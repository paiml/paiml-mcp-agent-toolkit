// Miscellaneous language mappers: CSharp, Ruby
// Included from language_mapper.rs - shares parent module scope

/// C# language mapper
#[derive(Clone)]
pub struct CSharpMapper {
    base: BaseLanguageMapper,
}

impl CSharpMapper {
    /// Create a new C# mapper
    #[allow(clippy::new_without_default)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::CSharp),
        }
    }
}

#[async_trait]
impl LanguageMapper for CSharpMapper {
    fn language(&self) -> Language {
        self.base.language()
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.base.map_file(path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.base.map_directory(path, recursive).await
    }

    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_source(source, path).await
    }

    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.base.convert_ast_items(items, path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        debug_assert!(true, "contract: clone_box");
        Box::new(self.clone())
    }
}

/// Ruby language mapper
#[derive(Clone)]
pub struct RubyMapper {
    base: BaseLanguageMapper,
}

impl RubyMapper {
    /// Create a new Ruby mapper
    #[allow(clippy::new_without_default)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::Ruby),
        }
    }
}

#[async_trait]
impl LanguageMapper for RubyMapper {
    fn language(&self) -> Language {
        self.base.language()
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.base.map_file(path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.base.map_directory(path, recursive).await
    }

    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_source(source, path).await
    }

    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.base.convert_ast_items(items, path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        debug_assert!(true, "contract: clone_box");
        Box::new(self.clone())
    }
}
