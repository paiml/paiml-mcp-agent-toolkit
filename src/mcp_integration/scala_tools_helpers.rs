/// Analyzes a single Scala file
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
async fn analyze_scala_file(
    path: &std::path::Path,
    include_metrics: bool,
    include_ast: bool,
) -> Result<Value> {
    let content = fs::read_to_string(path).await?;
    let visitor = ScalaAstVisitor::new(path);

    match visitor.analyze_scala_source(&content) {
        Ok(items) => {
            let mut result = build_file_summary(path, &items);

            if include_metrics {
                let method_count = count_by_kinds(&items, &["method", "function"]);
                result["metrics"] = build_file_metrics(&items, method_count, &content);
            }

            if include_ast {
                result["items"] = serde_json::to_value(&items)?;
            }

            Ok(result)
        }
        Err(e) => {
            warn!("Failed to parse Scala file {}: {}", path.display(), e);
            Ok(json!({
                "status": "error",
                "path": path.display().to_string(),
                "language": "scala",
                "error": e
            }))
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn count_by_kinds(items: &[crate::services::context::AstItem], kinds: &[&str]) -> usize {
    items
        .iter()
        .filter(|item| {
            let kind = extract_kind(item);
            kinds.contains(&kind.as_str())
        })
        .count()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn count_classes(items: &[crate::services::context::AstItem]) -> usize {
    items
        .iter()
        .filter(|item| {
            if let crate::services::context::AstItem::Struct { derives, .. } = item {
                !derives.contains(&"case".to_string())
            } else {
                false
            }
        })
        .count()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn count_case_classes(items: &[crate::services::context::AstItem]) -> usize {
    items
        .iter()
        .filter(|item| {
            if let crate::services::context::AstItem::Struct { derives, .. } = item {
                derives.contains(&"case".to_string())
            } else {
                false
            }
        })
        .count()
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn build_file_summary(path: &std::path::Path, items: &[crate::services::context::AstItem]) -> Value {
    let package_name = items
        .iter()
        .find(|item| {
            let kind = extract_kind(item);
            kind == "package" || kind == "module"
        })
        .map(extract_name)
        .unwrap_or_else(|| "default".to_string());

    json!({
        "status": "completed",
        "path": path.display().to_string(),
        "language": "scala",
        "summary": {
            "class_count": count_classes(items),
            "trait_count": count_by_kinds(items, &["trait"]),
            "object_count": count_by_kinds(items, &["object", "module"]),
            "case_class_count": count_case_classes(items),
            "method_count": count_by_kinds(items, &["method", "function"]),
            "package": package_name,
            "total_items": items.len()
        }
    })
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn build_file_metrics(
    items: &[crate::services::context::AstItem],
    method_count: usize,
    content: &str,
) -> Value {
    let total_complexity: u32 = items.iter().map(extract_complexity).sum();
    let max_complexity = items.iter().map(extract_complexity).max().unwrap_or(0);
    let avg_complexity = if method_count > 0 {
        (total_complexity as f64) / (method_count as f64)
    } else {
        0.0
    };

    json!({
        "total_complexity": total_complexity,
        "max_complexity": max_complexity,
        "avg_complexity": avg_complexity,
        "functional_percentage": calculate_functional_percentage(items),
        "loc": content.lines().count()
    })
}

struct DirectoryAccumulator {
    total_classes: u64,
    total_traits: u64,
    total_objects: u64,
    total_case_classes: u64,
    total_methods: u64,
    total_complexity: u64,
    max_complexity: u64,
    total_loc: u64,
    weighted_functional_pct: f64,
    total_weight: f64,
}

impl DirectoryAccumulator {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn new() -> Self {
        Self {
            total_classes: 0, total_traits: 0, total_objects: 0,
            total_case_classes: 0, total_methods: 0, total_complexity: 0,
            max_complexity: 0, total_loc: 0, weighted_functional_pct: 0.0,
            total_weight: 0.0,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn accumulate_summary(&mut self, summary: &serde_json::Map<String, Value>) {
        if let Some(v) = summary["class_count"].as_u64() { self.total_classes += v; }
        if let Some(v) = summary["trait_count"].as_u64() { self.total_traits += v; }
        if let Some(v) = summary["object_count"].as_u64() { self.total_objects += v; }
        if let Some(v) = summary["case_class_count"].as_u64() { self.total_case_classes += v; }
        if let Some(v) = summary["method_count"].as_u64() { self.total_methods += v; }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn accumulate_metrics(&mut self, metrics: &serde_json::Map<String, Value>) {
        if let Some(v) = metrics["total_complexity"].as_u64() { self.total_complexity += v; }
        if let Some(v) = metrics["max_complexity"].as_u64() {
            self.max_complexity = std::cmp::max(self.max_complexity, v);
        }
        if let Some(loc) = metrics["loc"].as_u64() {
            self.total_loc += loc;
            if let Some(fp) = metrics["functional_percentage"].as_f64() {
                self.weighted_functional_pct += fp * (loc as f64);
                self.total_weight += loc as f64;
            }
        }
    }
}

/// Analyzes a directory of Scala files recursively
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
async fn analyze_scala_directory(
    path: &std::path::Path,
    max_depth: u64,
    include_metrics: bool,
    include_ast: bool,
) -> Result<Value> {
    let scala_files = find_scala_files(path, max_depth as usize)?;

    if scala_files.is_empty() {
        return Ok(json!({
            "status": "completed",
            "path": path.display().to_string(),
            "language": "scala",
            "summary": { "file_count": 0, "message": "No Scala files found" }
        }));
    }

    let mut acc = DirectoryAccumulator::new();
    let mut file_results = Vec::new();

    for file_path in &scala_files {
        match analyze_scala_file(file_path, include_metrics, false).await {
            Ok(result) => {
                if let Some(summary) = result["summary"].as_object() {
                    acc.accumulate_summary(summary);
                }
                if include_metrics {
                    if let Some(metrics) = result["metrics"].as_object() {
                        acc.accumulate_metrics(metrics);
                    }
                }
                file_results.push(result);
            }
            Err(e) => {
                warn!("Error analyzing Scala file {}: {}", file_path.display(), e);
            }
        }
    }

    let avg_complexity = if acc.total_methods > 0 {
        (acc.total_complexity as f64) / (acc.total_methods as f64)
    } else {
        0.0
    };
    let avg_functional = if acc.total_weight > 0.0 {
        acc.weighted_functional_pct / acc.total_weight
    } else {
        0.0
    };

    let mut result = json!({
        "status": "completed",
        "path": path.display().to_string(),
        "language": "scala",
        "summary": {
            "file_count": scala_files.len(),
            "class_count": acc.total_classes,
            "trait_count": acc.total_traits,
            "object_count": acc.total_objects,
            "case_class_count": acc.total_case_classes,
            "method_count": acc.total_methods,
        }
    });

    if include_metrics {
        result["metrics"] = json!({
            "total_complexity": acc.total_complexity,
            "max_complexity": acc.max_complexity,
            "avg_complexity": avg_complexity,
            "functional_percentage": avg_functional,
            "total_loc": acc.total_loc
        });
    }

    if include_ast {
        result["files"] = serde_json::to_value(&file_results)?;
    }

    Ok(result)
}

/// Helper function to find all Scala files in a directory
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn find_scala_files(path: &std::path::Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    let mut scala_files = Vec::new();

    let walker = walkdir::WalkDir::new(path)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(Result::ok);

    for entry in walker {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == "scala" || ext == "sc")
        {
            scala_files.push(path.to_path_buf());
        }
    }

    Ok(scala_files)
}

/// Helper function to calculate the percentage of functional code patterns vs imperative
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn calculate_functional_percentage(items: &[crate::services::context::AstItem]) -> f64 {
    let mut functional_score = 0.0;
    let mut imperative_score = 0.0;

    for item in items {
        let kind = extract_kind(item);
        let name = extract_name(item);

        match kind.as_str() {
            "struct" if name.starts_with("Case") => functional_score += 1.0,
            "trait" => functional_score += 0.5,
            "module" => functional_score += 0.5,
            "struct" | "class" if !name.starts_with("Case") => imperative_score += 0.5,
            "function" | "method" => imperative_score += 0.3,
            _ => {}
        }
    }

    let total = functional_score + imperative_score;
    if total > 0.0 {
        (functional_score / total) * 100.0
    } else {
        50.0
    }
}
