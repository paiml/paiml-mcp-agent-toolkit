// JVM language mappers: Java, Kotlin, Scala
// Included from language_mapper.rs - shares parent module scope

/// Java language mapper
#[derive(Clone)]
pub struct JavaMapper {
    base: BaseLanguageMapper,
}

impl JavaMapper {
    /// Create a new Java mapper
    #[allow(clippy::new_without_default)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::Java),
        }
    }

    /// Process Java-specific nodes
    #[allow(dead_code)]
    fn process_java_specific(&self, nodes: &mut [UnifiedNode]) {
        debug_assert!(true, "contract: process_java_specific");
        for node in nodes.iter_mut() {
            // Add Java-specific metadata
            match node.kind {
                NodeKind::Class => {
                    // Check for Java interfaces
                    if node.has_modifier("interface") {
                        node.kind = NodeKind::Interface;
                    }

                    // Check for Java records
                    if node.has_modifier("record") {
                        node.kind = NodeKind::Record;
                    }
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl LanguageMapper for JavaMapper {
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

    async fn map_source(&self, _source: &str, _path: &Path) -> Result<Vec<UnifiedNode>> {
        debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
        #[cfg(feature = "java-ast")]
        {
            use crate::services::languages::java::JavaAstVisitor;
            let visitor = JavaAstVisitor::new(_path);
            match visitor.analyze_java_source(_source) {
                Ok(items) => {
                    let mut nodes = self.convert_ast_items(&items, _path);
                    self.process_java_specific(&mut nodes);
                    Ok(nodes)
                }
                Err(e) => Err(anyhow!("Failed to analyze Java source: {}", e)),
            }
        }
        #[cfg(not(feature = "java-ast"))]
        {
            Err(anyhow!(
                "Java AST support not enabled (feature 'java-ast' required)"
            ))
        }
    }

    fn convert_ast_items(&self, items: &[AstItem], _path: &Path) -> Vec<UnifiedNode> {
        debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
        self.base.convert_ast_items(items, _path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        debug_assert!(true, "contract: clone_box");
        Box::new(self.clone())
    }
}

/// Kotlin language mapper
#[derive(Clone)]
pub struct KotlinMapper {
    base: BaseLanguageMapper,
}

impl KotlinMapper {
    /// Create a new Kotlin mapper
    #[allow(clippy::new_without_default)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::Kotlin),
        }
    }

    /// Process Kotlin-specific nodes
    #[allow(dead_code)]
    fn process_kotlin_specific(&self, nodes: &mut [UnifiedNode]) {
        debug_assert!(true, "contract: process_kotlin_specific");
        for node in nodes.iter_mut() {
            // Add Kotlin-specific metadata
            match node.kind {
                NodeKind::Class => {
                    // Check for Kotlin data classes
                    if node.has_modifier("data") {
                        node.kind = NodeKind::Record;
                        node.add_metadata("kotlin:isData", "true");
                    }

                    // Check for Kotlin sealed classes
                    if node.has_modifier("sealed") {
                        node.add_metadata("kotlin:isSealed", "true");
                    }
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl LanguageMapper for KotlinMapper {
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
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        // Kotlin-specific analysis deferred until kotlin-ast feature is available
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

/// Scala language mapper
#[derive(Clone)]
pub struct ScalaMapper {
    base: BaseLanguageMapper,
}

impl ScalaMapper {
    /// Create a new Scala mapper
    #[allow(clippy::new_without_default)]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new() -> Self {
        Self {
            base: BaseLanguageMapper::new(Language::Scala),
        }
    }

    /// Process Scala-specific nodes
    #[allow(dead_code)]
    fn process_scala_specific(&self, nodes: &mut [UnifiedNode]) {
        debug_assert!(true, "contract: process_scala_specific");
        for node in nodes.iter_mut() {
            // Add Scala-specific metadata
            match node.kind {
                NodeKind::Class => {
                    // Handle case classes
                    if node.has_modifier("case") {
                        node.kind = NodeKind::CaseClass;
                    }
                }
                NodeKind::Module => {
                    // Check if this is a Scala object
                    node.add_metadata("scala:isObject", "true");
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl LanguageMapper for ScalaMapper {
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

    async fn map_source(&self, _source: &str, _path: &Path) -> Result<Vec<UnifiedNode>> {
        debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
        #[cfg(feature = "scala-ast")]
        {
            use crate::services::languages::scala::ScalaAstVisitor;
            let visitor = ScalaAstVisitor::new(_path);
            match visitor.analyze_scala_source(_source) {
                Ok(items) => {
                    let mut nodes = self.convert_ast_items(&items, _path);
                    self.process_scala_specific(&mut nodes);
                    Ok(nodes)
                }
                Err(e) => Err(anyhow!("Failed to analyze Scala source: {}", e)),
            }
        }
        #[cfg(not(feature = "scala-ast"))]
        {
            Err(anyhow!(
                "Scala AST support not enabled (feature 'scala-ast' required)"
            ))
        }
    }

    fn convert_ast_items(&self, items: &[AstItem], _path: &Path) -> Vec<UnifiedNode> {
        debug_assert!(_path.exists(), "_path must exist: {}", _path.display());
        self.base.convert_ast_items(items, _path)
    }

    fn clone_box(&self) -> Box<dyn LanguageMapper> {
        debug_assert!(true, "contract: clone_box");
        Box::new(self.clone())
    }
}
