// PolyglotAnalysisTool McpTool implementation
// Included from mod.rs — no `use` imports or `#!` attributes allowed

#[async_trait]
impl McpTool for PolyglotAnalysisTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "analyze_polyglot".to_string(),
            description: "Analyzes cross-language relationships in a project".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to directory to analyze"
                    },
                    "languages": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["java", "kotlin", "scala", "typescript", "javascript"]
                        },
                        "description": "Languages to include (default: all supported)"
                    },
                    "max_depth": {
                        "type": "number",
                        "default": 3,
                        "description": "Maximum directory recursion depth"
                    },
                    "include_graph": {
                        "type": "boolean",
                        "default": true,
                        "description": "Include dependency graph in DOT format"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, params: Value) -> Result<Value, McpError> {
        // Extract parameters
        let path_str = params["path"].as_str().ok_or_else(|| McpError {
            code: crate::mcp_integration::error_codes::INVALID_PARAMS,
            message: "Missing path parameter".to_string(),
            data: None,
        })?;

        let path = PathBuf::from(path_str);
        let max_depth = params["max_depth"].as_u64().unwrap_or(3) as usize;
        let include_graph = params["include_graph"].as_bool().unwrap_or(true);

        // Validate path using the new polyglot path validator
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

        // Parse languages to include
        let languages = if params["languages"].is_array() {
            let langs = params["languages"]
                .as_array()
                .expect("internal error")
                .iter()
                .filter_map(|l| l.as_str())
                .filter_map(|l| match l.to_lowercase().as_str() {
                    "java" => Some(Language::Java),
                    "kotlin" => Some(Language::Kotlin),
                    "scala" => Some(Language::Scala),
                    "typescript" => Some(Language::TypeScript),
                    "javascript" => Some(Language::JavaScript),
                    _ => None,
                })
                .collect::<Vec<_>>();

            if langs.is_empty() {
                vec![
                    Language::Java,
                    Language::Kotlin,
                    Language::Scala,
                    Language::TypeScript,
                    Language::JavaScript,
                ]
            } else {
                langs
            }
        } else {
            // Default to all supported languages
            vec![
                Language::Java,
                Language::Kotlin,
                Language::Scala,
                Language::TypeScript,
                Language::JavaScript,
            ]
        };

        // Analyze the directory for each language
        let mut language_nodes: HashMap<Language, Vec<UnifiedNode>> = HashMap::new();
        let mut all_nodes = Vec::new();

        for language in &languages {
            match LanguageMapperFactory::create(*language) {
                Ok(mapper) => match mapper.map_directory(&path, max_depth > 0).await {
                    Ok(nodes) => {
                        language_nodes.insert(*language, nodes.clone());
                        all_nodes.extend(nodes);
                    }
                    Err(e) => {
                        tracing::warn!("Error mapping {:?} files: {}", language, e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Error creating mapper for {:?}: {}", language, e);
                }
            }
        }

        // Detect cross-language dependencies
        let mut dependencies = CrossLanguageDependencies::new();
        dependencies.add_nodes(all_nodes.clone());
        dependencies.detect_all();

        // Build response
        let mut result = json!({
            "status": "completed",
            "path": path.display().to_string(),
            "languages": languages.iter().map(|l| l.name()).collect::<Vec<_>>(),
            "summary": {
                "total_files": language_nodes.values().map(|nodes| {
                    nodes.iter()
                        .map(|n| n.file_path.clone())
                        .collect::<HashSet<_>>()
                        .len()
                }).sum::<usize>(),
                "total_nodes": all_nodes.len(),
                "nodes_by_language": language_nodes.iter()
                    .map(|(lang, nodes)| (lang.name(), nodes.len()))
                    .collect::<HashMap<_, _>>(),
                "total_cross_language_dependencies": dependencies.get_dependencies().len()
            }
        });

        // Add detailed node counts by type
        let node_counts = get_node_type_counts(&all_nodes);
        result["node_counts"] = json!(node_counts);

        // Add dependency information
        let deps = dependencies.get_dependencies();
        let mut dep_counts: HashMap<String, usize> = HashMap::new();

        for dep in deps {
            let key = format!(
                "{} -> {}",
                dep.source_language.name(),
                dep.target_language.name()
            );
            *dep_counts.entry(key).or_insert(0) += 1;
        }

        result["dependency_counts"] = json!(dep_counts);

        // Add detailed dependencies
        let mut detailed_deps = Vec::new();
        for dep in deps {
            let source_node = all_nodes
                .iter()
                .find(|n| n.id == dep.source_id)
                .map(|n| {
                    json!({
                        "id": n.id,
                        "name": n.name,
                        "fqn": n.fqn,
                        "kind": n.kind.as_str()
                    })
                })
                .unwrap_or_else(|| json!({"id": dep.source_id}));

            let target_node = all_nodes
                .iter()
                .find(|n| n.id == dep.target_id)
                .map(|n| {
                    json!({
                        "id": n.id,
                        "name": n.name,
                        "fqn": n.fqn,
                        "kind": n.kind.as_str()
                    })
                })
                .unwrap_or_else(|| json!({"id": dep.target_id}));

            detailed_deps.push(json!({
                "source": source_node,
                "target": target_node,
                "kind": format!("{:?}", dep.kind),
                "source_language": dep.source_language.name(),
                "target_language": dep.target_language.name(),
                "confidence": dep.confidence
            }));
        }

        result["dependencies"] = json!(detailed_deps);

        // Add graph if requested
        if include_graph {
            result["graph_dot"] = json!(dependencies.to_dot());
        }

        Ok(result)
    }
}
