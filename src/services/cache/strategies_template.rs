/// Template cache strategy for embedded template resources.
///
/// Caches template resources that are embedded in the binary. Since templates
/// are immutable, validation always returns true for better performance.
///
/// # Cache Characteristics
///
/// - **TTL**: 10 minutes
/// - **Max Size**: 50 entries
/// - **Key**: Template URI prefixed with "template:"
/// - **Validation**: Always valid (templates don't change)
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::cache::TemplateCacheStrategy;
/// use pmat::services::cache::CacheStrategy;
/// use pmat::models::template::TemplateResource;
///
/// let strategy = TemplateCacheStrategy;
/// let template_uri = "template://makefile/rust.mk".to_string();
///
/// // Generate cache key
/// let key = strategy.cache_key(&template_uri);
/// assert_eq!(key, "template:template://makefile/rust.mk");
///
/// // Create a dummy template resource
/// let template = TemplateResource {
///     uri: template_uri.clone(),
///     name: "test".to_string(),
///     description: "test description".to_string(),
///     toolchain: pmat::models::template::Toolchain::RustCli { cargo_features: vec![] },
///     category: pmat::models::template::TemplateCategory::Makefile,
///     parameters: vec![],
///     s3_object_key: "test".to_string(),
///     content_hash: "test".to_string(),
///     semantic_version: semver::Version::parse("1.0.0").expect("valid semver"),
///     dependency_graph: vec![],
/// };
///
/// // Templates are always valid (embedded, immutable)
/// assert!(strategy.validate(&template_uri, &template));
///
/// // TTL should be 10 minutes
/// assert_eq!(strategy.ttl().expect("has ttl").as_secs(), 600);
/// assert_eq!(strategy.max_size(), 50);
/// ```
impl CacheStrategy for TemplateCacheStrategy {
    type Key = String; // Template URI
    type Value = TemplateResource;

    fn cache_key(&self, uri: &String) -> String {
        format!("template:{uri}")
    }

    fn validate(&self, _uri: &String, _cached: &TemplateResource) -> bool {
        // Templates are embedded and don't change
        true
    }

    fn ttl(&self) -> Option<Duration> {
        Some(Duration::from_secs(600)) // 10 minutes
    }

    fn max_size(&self) -> usize {
        50 // Max 50 templates
    }
}
