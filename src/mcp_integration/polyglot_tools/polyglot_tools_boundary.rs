// LanguageBoundaryTool McpTool implementation and boundary pattern analysis
// Included from mod.rs — no `use` imports or `#!` attributes allowed

#[async_trait]
impl McpTool for LanguageBoundaryTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "analyze_language_boundaries".to_string(),
            description: "Detects language boundaries and interoperability points in a project"
                .to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to directory to analyze"
                    },
                    "source_language": {
                        "type": "string",
                        "description": "Source language to analyze boundaries from (optional)"
                    },
                    "target_language": {
                        "type": "string",
                        "description": "Target language to analyze boundaries to (optional)"
                    },
                    "max_depth": {
                        "type": "number",
                        "default": 3,
                        "description": "Maximum directory recursion depth"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        let (path, max_depth, source_language, target_language) = parse_boundary_params(&params)?;
        let languages = resolve_languages(source_language, target_language);
        let all_nodes = collect_language_nodes(&path, max_depth, &languages).await;

        let mut dependencies = CrossLanguageDependencies::new();
        dependencies.add_nodes(all_nodes.clone());
        dependencies.detect_all();

        let filtered_deps = filter_dependencies(
            dependencies.get_dependencies(),
            source_language,
            target_language,
        );

        let mut result = json!({
            "status": "completed",
            "path": path.display().to_string(),
            "languages_analyzed": languages.iter().map(|l| l.name()).collect::<Vec<_>>(),
            "summary": {
                "total_boundaries": filtered_deps.len(),
                "source_language": source_language.map(|l| l.name()),
                "target_language": target_language.map(|l| l.name()),
            }
        });

        result["boundaries"] = build_boundaries_json(&filtered_deps, &all_nodes);
        result["boundary_types"] = build_boundary_stats(&filtered_deps);
        result["patterns"] = analyze_boundary_patterns(filtered_deps, &all_nodes);

        Ok(result)
    }
}

fn parse_language_param(value: &Value, key: &str) -> Option<Language> {
    debug_assert!(!key.is_empty(), "key must not be empty");
    value[key].as_str().and_then(|l| match l.to_lowercase().as_str() {
        "java" => Some(Language::Java),
        "kotlin" => Some(Language::Kotlin),
        "scala" => Some(Language::Scala),
        "typescript" => Some(Language::TypeScript),
        "javascript" => Some(Language::JavaScript),
        _ => None,
    })
}

fn parse_boundary_params(params: &Value) -> Result<(PathBuf, usize, Option<Language>, Option<Language>), McpError> {
    let path_str = params["path"].as_str().ok_or_else(|| McpError {
        code: crate::mcp_integration::error_codes::INVALID_PARAMS,
        message: "Missing path parameter".to_string(),
        data: None,
    })?;
    let path = PathBuf::from(path_str);
    let max_depth = params["max_depth"].as_u64().unwrap_or(3) as usize;

    if let Err(e) = PolyglotPathValidator::validate_directory_path(&path) {
        return Err(McpError {
            code: crate::mcp_integration::error_codes::INVALID_PARAMS,
            message: format!("Invalid directory path: {}", e),
            data: Some(json!({
                "path": path.display().to_string(),
                "suggestion": "Please provide a valid directory path"
            })),
        });
    }

    let source_language = parse_language_param(params, "source_language");
    let target_language = parse_language_param(params, "target_language");
    Ok((path, max_depth, source_language, target_language))
}

fn resolve_languages(source: Option<Language>, target: Option<Language>) -> Vec<Language> {
    if source.is_none() && target.is_none() {
        vec![Language::Java, Language::Kotlin, Language::Scala, Language::TypeScript, Language::JavaScript]
    } else {
        let mut langs = Vec::new();
        if let Some(lang) = source { langs.push(lang); }
        if let Some(lang) = target {
            if !langs.contains(&lang) { langs.push(lang); }
        }
        langs
    }
}

async fn collect_language_nodes(path: &Path, max_depth: usize, languages: &[Language]) -> Vec<UnifiedNode> {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let mut all_nodes = Vec::new();
    for language in languages {
        match LanguageMapperFactory::create(*language) {
            Ok(mapper) => match mapper.map_directory(path, max_depth > 0).await {
                Ok(nodes) => all_nodes.extend(nodes),
                Err(e) => tracing::warn!("Error mapping {:?} files: {}", language, e),
            },
            Err(e) => tracing::warn!("Error creating mapper for {:?}: {}", language, e),
        }
    }
    all_nodes
}

fn filter_dependencies<'a>(
    deps: &'a [crate::ast::polyglot::cross_language_dependencies::CrossLanguageDependency],
    source: Option<Language>,
    target: Option<Language>,
) -> Vec<&'a crate::ast::polyglot::cross_language_dependencies::CrossLanguageDependency> {
    deps.iter()
        .filter(|dep| {
            source.map(|l| dep.source_language == l).unwrap_or(true)
                && target.map(|l| dep.target_language == l).unwrap_or(true)
        })
        .collect()
}

fn node_to_json(nodes: &[UnifiedNode], id: &str) -> Value {
    debug_assert!(!id.is_empty(), "id must not be empty");
    debug_assert!(!nodes.is_empty(), "nodes must not be empty");
    nodes.iter().find(|n| n.id == id).map(|n| {
        json!({"id": n.id, "name": n.name, "fqn": n.fqn, "kind": n.kind.as_str(), "file": n.file_path.display().to_string()})
    }).unwrap_or_else(|| json!({"id": id}))
}

fn build_boundaries_json(
    filtered_deps: &[&crate::ast::polyglot::cross_language_dependencies::CrossLanguageDependency],
    all_nodes: &[UnifiedNode],
) -> Value {
    let boundaries: Vec<Value> = filtered_deps.iter().map(|dep| {
        json!({
            "boundary_type": format!("{:?}", dep.kind),
            "source": {"language": dep.source_language.name(), "node": node_to_json(all_nodes, &dep.source_id)},
            "target": {"language": dep.target_language.name(), "node": node_to_json(all_nodes, &dep.target_id)},
            "confidence": dep.confidence
        })
    }).collect();
    json!(boundaries)
}

fn build_boundary_stats(
    filtered_deps: &[&crate::ast::polyglot::cross_language_dependencies::CrossLanguageDependency],
) -> Value {
    let mut grouped: HashMap<String, Vec<_>> = HashMap::new();
    for dep in filtered_deps {
        grouped.entry(format!("{:?}", dep.kind)).or_default().push(*dep);
    }
    let mut stats = json!({});
    for (kind, deps) in &grouped {
        stats[kind] = json!({
            "count": deps.len(),
            "languages": deps.iter().map(|d| format!("{} → {}", d.source_language.name(), d.target_language.name())).collect::<HashSet<_>>()
        });
    }
    stats
}

fn recommendations_for_pair(a: &str, b: &str) -> Value {
    debug_assert!(!a.is_empty(), "a must not be empty");
    debug_assert!(!b.is_empty(), "b must not be empty");
    match (a, b) {
        ("Java", "Kotlin") | ("Kotlin", "Java") => json!([
            "Use Kotlin's @JvmName annotation to control Java-visible names",
            "Leverage Kotlin extension functions for Java interoperability",
            "Use Kotlin's nullable types consistently with Java's @Nullable",
            "Consider avoiding Kotlin-specific features at boundaries (coroutines, delegation)"
        ]),
        ("Java", "Scala") | ("Scala", "Java") => json!([
            "Prefer Java interfaces at language boundaries",
            "Be careful with Scala's implicit conversions at Java boundaries",
            "Avoid using Scala's case classes as Java API",
            "Use Java collections when sharing data between Java and Scala"
        ]),
        ("TypeScript", "JavaScript") | ("JavaScript", "TypeScript") => json!([
            "Use TypeScript declaration files (.d.ts) for JavaScript modules",
            "Add JSDoc comments to JavaScript for TypeScript type inference",
            "Consider migrating to pure TypeScript gradually",
            "Use ES modules format for better interoperability"
        ]),
        ("Java", "TypeScript") | ("TypeScript", "Java") => json!([
            "Use consistent naming conventions across both languages",
            "Define API contracts with OpenAPI/Swagger for REST interfaces",
            "Consider type-safe approaches like GraphQL or gRPC",
            "Enforce model consistency with shared schemas"
        ]),
        _ => json!([
            "Define clear API contracts between languages",
            "Use consistent naming conventions",
            "Minimize direct cross-language dependencies",
            "Consider using an interface language (API specs, proto files, etc.)"
        ]),
    }
}

fn analyze_boundary_patterns(
    deps: Vec<&crate::ast::polyglot::cross_language_dependencies::CrossLanguageDependency>,
    _nodes: &[UnifiedNode],
) -> Value {
    let mut language_pairs: HashMap<String, Vec<_>> = HashMap::new();
    for dep in &deps {
        let key = format!("{}-{}", dep.source_language.name(), dep.target_language.name());
        language_pairs.entry(key).or_default().push(*dep);
    }

    let patterns: Vec<Value> = language_pairs.into_iter().map(|(pair, deps)| {
        let mut pattern = json!({"language_pair": pair, "count": deps.len()});
        let parts: Vec<&str> = pair.split('-').collect();
        if parts.len() == 2 {
            pattern["recommendations"] = recommendations_for_pair(parts[0], parts[1]);
        }
        pattern
    }).collect();

    json!(patterns)
}
