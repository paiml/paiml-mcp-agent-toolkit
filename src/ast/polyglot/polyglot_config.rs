/// Configuration for polyglot AST analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolyglotConfig {
    /// List of languages to include in analysis
    pub languages: Vec<Language>,

    /// Whether to detect cross-language relationships
    pub detect_relationships: bool,

    /// Maximum depth for relationship analysis (inheritance, implementation, etc.)
    pub relationship_depth: usize,

    /// Whether to include language-specific details
    pub include_language_specific: bool,
}

impl Default for PolyglotConfig {
    fn default() -> Self {
        Self {
            languages: vec![
                Language::Java,
                Language::Kotlin,
                Language::Scala,
                Language::TypeScript,
                Language::JavaScript,
            ],
            detect_relationships: true,
            relationship_depth: 3,
            include_language_specific: true,
        }
    }
}
