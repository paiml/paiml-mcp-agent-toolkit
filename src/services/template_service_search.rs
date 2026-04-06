// Template listing and search functions.
// Includes list_templates, list_all_resources, search_templates, and relevance scoring helpers.

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn list_templates<T: TemplateServerTrait>(
    server: &T,
    toolchain: Option<&str>,
    category: Option<&str>,
) -> Result<Vec<Arc<TemplateResource>>, TemplateError> {
    let prefix = build_template_prefix(category, toolchain);
    let mut templates = server.list_templates(&prefix).await.map_err(|_| {
        TemplateError::NotFound(format!("Failed to list templates with prefix: {prefix}"))
    })?;

    // Filter by toolchain if specified but category is not
    // (when both are specified, the prefix already handles filtering)
    if let Some(tc) = toolchain {
        if category.is_none() {
            templates.retain(|t| t.toolchain.as_str() == tc);
        }
    }

    // Sort by toolchain priority and version
    templates.sort_by(|a, b| {
        a.toolchain
            .priority()
            .cmp(&b.toolchain.priority())
            .then_with(|| b.semantic_version.cmp(&a.semantic_version))
    });

    Ok(templates)
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn list_all_resources<T: TemplateServerTrait>(
    server: &T,
) -> Result<Vec<Arc<TemplateResource>>, TemplateError> {
    list_templates(server, None, None).await
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub async fn search_templates<T: TemplateServerTrait>(
    server: Arc<T>,
    query: &str,
    toolchain: Option<&str>,
) -> Result<Vec<SearchResult>, TemplateError> {
    let templates = list_templates(server.as_ref(), toolchain, None).await?;
    let query_lower = query.to_lowercase();

    let mut results: Vec<SearchResult> = templates
        .into_iter()
        .filter_map(|template| compute_template_relevance(template, &query_lower))
        .collect();

    results.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .expect("internal error")
    });
    Ok(results)
}

fn compute_template_relevance(
    template: Arc<crate::models::template::TemplateResource>,
    query_lower: &str,
) -> Option<SearchResult> {
    let mut matches = Vec::new();
    let mut relevance = 0.0_f32;

    check_name_match(&template.name, query_lower, &mut matches, &mut relevance);
    check_description_match(
        &template.description,
        query_lower,
        &mut matches,
        &mut relevance,
    );
    check_parameter_matches(
        &template.parameters,
        query_lower,
        &mut matches,
        &mut relevance,
    );

    if matches.is_empty() {
        None
    } else {
        Some(SearchResult {
            template: (*template).clone(),
            relevance,
            matches,
        })
    }
}

fn check_name_match(name: &str, query: &str, matches: &mut Vec<String>, relevance: &mut f32) {
    if name.to_lowercase().contains(query) {
        matches.push(format!("name: {}", name));
        *relevance += if name.to_lowercase() == query {
            10.0
        } else {
            5.0
        };
    }
}

fn check_description_match(
    desc: &str,
    query: &str,
    matches: &mut Vec<String>,
    relevance: &mut f32,
) {
    if desc.to_lowercase().contains(query) {
        matches.push("description".to_string());
        *relevance += 3.0;
    }
}

fn check_parameter_matches(
    params: &[crate::models::template::ParameterSpec],
    query: &str,
    matches: &mut Vec<String>,
    relevance: &mut f32,
) {
    for param in params {
        if param.name.to_lowercase().contains(query) {
            matches.push(format!("parameter: {}", param.name));
            *relevance += 1.0;
        }
    }
}
