// Web language mappers: TypeScript, JavaScript
// Included from language_mapper.rs - shares parent module scope

/// TypeScript language mapper
#[derive(Clone)]
pub struct TypeScriptMapper {
    base: BaseLanguageMapper,
}

impl TypeScriptMapper {
    /// Create a new TypeScript mapper
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::TypeScript),
        }
    }

    /// Process TypeScript-specific nodes
    fn process_typescript_specific(&self, nodes: &mut [UnifiedNode]) {
        for node in nodes.iter_mut() {
            // Add TypeScript-specific metadata
            match node.kind {
                NodeKind::Interface => {
                    node.add_metadata("typescript:isInterface", "true");
                }
                NodeKind::Class => {
                    if node.has_modifier("abstract") {
                        node.add_metadata("typescript:isAbstract", "true");
                    }
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl LanguageMapper for TypeScriptMapper {
    fn language(&self) -> Language {
        self.base.language()
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_file(path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        self.base.map_directory(path, recursive).await
    }

    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>> {
        use crate::services::languages::typescript::TypeScriptAstVisitor;

        let visitor = TypeScriptAstVisitor::new(path);
        match visitor.analyze_typescript_source(source) {
            Ok(items) => {
                let mut nodes = self.convert_ast_items(&items, path);
                self.process_typescript_specific(&mut nodes);
                Ok(nodes)
            }
            Err(e) => Err(anyhow!("Failed to analyze TypeScript source: {}", e)),
        }
    }

    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.base.convert_ast_items(items, path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}

/// JavaScript language mapper
#[derive(Clone)]
pub struct JavaScriptMapper {
    base: BaseLanguageMapper,
}

impl JavaScriptMapper {
    /// Create a new JavaScript mapper
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::JavaScript),
        }
    }

    /// Process JavaScript-specific nodes
    fn process_javascript_specific(&self, nodes: &mut [UnifiedNode]) {
        for node in nodes.iter_mut() {
            // Add JavaScript-specific metadata
            match node.kind {
                NodeKind::Class => {
                    node.add_metadata("javascript:isClass", "true");
                }
                NodeKind::Function => {
                    // Check for arrow functions
                    if node.has_modifier("arrow") {
                        node.kind = NodeKind::Lambda;
                    }
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl LanguageMapper for JavaScriptMapper {
    fn language(&self) -> Language {
        self.base.language()
    }

    async fn map_file(&self, path: &Path) -> Result<Vec<UnifiedNode>> {
        self.base.map_file(path).await
    }

    async fn map_directory(&self, path: &Path, recursive: bool) -> Result<Vec<UnifiedNode>> {
        self.base.map_directory(path, recursive).await
    }

    async fn map_source(&self, source: &str, path: &Path) -> Result<Vec<UnifiedNode>> {
        use crate::services::languages::javascript::JavaScriptAstVisitor;

        let visitor = JavaScriptAstVisitor::new(path);
        match visitor.analyze_javascript_source(source) {
            Ok(items) => {
                let mut nodes = self.convert_ast_items(&items, path);
                self.process_javascript_specific(&mut nodes);
                Ok(nodes)
            }
            Err(e) => Err(anyhow!("Failed to analyze JavaScript source: {}", e)),
        }
    }

    fn convert_ast_items(&self, items: &[AstItem], path: &Path) -> Vec<UnifiedNode> {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        self.base.convert_ast_items(items, path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        Box::new(self.clone())
    }
}
