/// Collect all functions from project context and identify entry points
fn collect_project_functions(
    project_context: &crate::services::context::ProjectContext,
) -> (
    std::collections::HashMap<String, (String, u32)>,
    std::collections::HashSet<String>,
) {
    use crate::services::context::AstItem;
    use std::collections::{HashMap, HashSet};

    let mut all_functions: HashMap<String, (String, u32)> = HashMap::new();
    let mut entry_points: HashSet<String> = HashSet::new();

    for file in &project_context.files {
        for item in &file.items {
            if let AstItem::Function { name, line, .. } = item {
                let qualified_name = format!("{}::{}", file.path, name);
                all_functions.insert(qualified_name.clone(), (file.path.clone(), *line as u32));

                if name == "main" || name.starts_with("pub ") {
                    entry_points.insert(qualified_name);
                }
            }
        }
    }

    (all_functions, entry_points)
}

/// Detect function calls by reading file content and matching call patterns
fn detect_function_calls(
    project_context: &crate::services::context::ProjectContext,
    all_functions: &std::collections::HashMap<String, (String, u32)>,
) -> std::collections::HashMap<String, std::collections::HashSet<String>> {
    use std::collections::{HashMap, HashSet};

    let mut function_calls: HashMap<String, HashSet<String>> = HashMap::new();

    for file in &project_context.files {
        let Ok(content) = std::fs::read_to_string(&file.path) else {
            continue;
        };
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            let line_number = i + 1;

            let mut current_function = None;
            for (qualified_name, (_, func_line)) in all_functions {
                if qualified_name.starts_with(&file.path) && line_number >= *func_line as usize {
                    current_function = Some(qualified_name.clone());
                }
            }

            if let Some(caller) = current_function {
                for callee_qualified in all_functions.keys() {
                    let callee_name =
                        callee_qualified.split("::").last().expect("internal error");
                    if line.contains(&format!("{callee_name}("))
                        && !line.contains(&format!("fn {callee_name}"))
                        && caller != *callee_qualified
                    {
                        function_calls
                            .entry(caller.clone())
                            .or_default()
                            .insert(callee_qualified.clone());
                    }
                }
            }
        }
    }

    function_calls
}

/// Perform reachability analysis from entry points through call graph
fn compute_reachable_functions(
    entry_points: &std::collections::HashSet<String>,
    function_calls: &std::collections::HashMap<String, std::collections::HashSet<String>>,
) -> std::collections::HashSet<String> {
    let mut reachable = entry_points.clone();
    let mut changed = true;

    while changed {
        changed = false;
        let current_reachable = reachable.clone();

        for reachable_func in &current_reachable {
            if let Some(callees) = function_calls.get(reachable_func) {
                for callee in callees {
                    if reachable.insert(callee.clone()) {
                        changed = true;
                    }
                }
            }
        }
    }

    reachable
}

impl DeadCodeAnalyzer {
    /// Analyze dead code using project context directly
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_project_context(
        &mut self,
        project_context: &crate::services::context::ProjectContext,
    ) -> anyhow::Result<DeadCodeReport> {
        use std::collections::HashMap;

        let (all_functions, entry_points) = collect_project_functions(project_context);
        let function_calls = detect_function_calls(project_context, &all_functions);
        let reachable = compute_reachable_functions(&entry_points, &function_calls);

        let mut dead_functions = Vec::new();
        for (qualified_name, (file_path, line)) in &all_functions {
            if !reachable.contains(qualified_name) {
                if is_cfg_gated(file_path, *line) {
                    continue;
                }
                let function_name = qualified_name
                    .split("::")
                    .last()
                    .expect("internal error")
                    .to_string();
                dead_functions.push(DeadCodeItem {
                    node_key: 0,
                    name: function_name,
                    file_path: file_path.clone(),
                    line_number: *line,
                    dead_type: DeadCodeType::UnusedFunction,
                    confidence: 0.95,
                    reason: "Not reachable from any entry point".to_string(),
                });
            }
        }

        let total_functions = all_functions.len();
        let dead_count = dead_functions.len();
        let percentage_dead = if total_functions > 0 {
            (dead_count as f32 / total_functions as f32) * 100.0
        } else {
            0.0
        };

        Ok(DeadCodeReport {
            dead_functions,
            dead_classes: Vec::new(),
            dead_variables: Vec::new(),
            unreachable_code: Vec::new(),
            summary: DeadCodeSummary {
                total_dead_code_lines: dead_count * 5,
                percentage_dead,
                dead_by_type: HashMap::new(),
                confidence_level: 0.85,
            },
        })
    }
}
