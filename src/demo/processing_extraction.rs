/// Extract actual analysis results and timings from demo report
#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn extract_analysis_from_demo_report(
    demo_report: &crate::demo::DemoReport,
) -> (
    Option<crate::services::complexity::ComplexityReport>,
    Option<crate::models::dag::DependencyGraph>,
    (u64, u64, u64, u64), // timings: (ast, complexity, dag, churn)
) {
    let mut complexity_result = None;
    let mut dag_result = None;
    let mut timings = (0u64, 0u64, 0u64, 0u64);

    for step in &demo_report.steps {
        process_demo_step(step, &mut complexity_result, &mut dag_result, &mut timings);
    }

    (complexity_result, dag_result, timings)
}

/// Process a single demo step (cognitive complexity <=8)
fn process_demo_step(
    step: &crate::demo::DemoStep,
    complexity_result: &mut Option<crate::services::complexity::ComplexityReport>,
    dag_result: &mut Option<crate::models::dag::DependencyGraph>,
    timings: &mut (u64, u64, u64, u64),
) {
    match step.capability {
        "AST Context Analysis" => process_ast_step(step, timings),
        "Code Complexity Analysis" => process_complexity_step(step, complexity_result, timings),
        "DAG Visualization" => process_dag_step(step, dag_result, timings),
        "Code Churn Analysis" => process_churn_step(step, timings),
        _ => {} // Unknown capability - skip
    }
}

/// Process AST context analysis step (cognitive complexity 1)
fn process_ast_step(step: &crate::demo::DemoStep, timings: &mut (u64, u64, u64, u64)) {
    timings.0 = step.elapsed_ms;
}

/// Process complexity analysis step (cognitive complexity <=6)
fn process_complexity_step(
    step: &crate::demo::DemoStep,
    complexity_result: &mut Option<crate::services::complexity::ComplexityReport>,
    timings: &mut (u64, u64, u64, u64),
) {
    timings.1 = step.elapsed_ms;

    if let Some(result) = &step.response.result {
        if let Some(complexity_report) = extract_complexity_from_result(result) {
            *complexity_result = Some(complexity_report);
        }
    }
}

/// Process DAG visualization step (cognitive complexity <=6)
fn process_dag_step(
    step: &crate::demo::DemoStep,
    dag_result: &mut Option<crate::models::dag::DependencyGraph>,
    timings: &mut (u64, u64, u64, u64),
) {
    timings.2 = step.elapsed_ms;

    if let Some(result) = &step.response.result {
        if let Some(dag) = extract_dag_from_result(result) {
            *dag_result = Some(dag);
        }
    }
}

/// Process code churn analysis step (cognitive complexity 1)
fn process_churn_step(step: &crate::demo::DemoStep, timings: &mut (u64, u64, u64, u64)) {
    timings.3 = step.elapsed_ms;
}

/// Extract complexity report from JSON result (cognitive complexity <=5)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn extract_complexity_from_result(
    result: &serde_json::Value,
) -> Option<crate::services::complexity::ComplexityReport> {
    let complexity_data = serde_json::from_value::<serde_json::Value>(result.clone()).ok()?;
    let report_value = complexity_data.get("report")?;
    serde_json::from_value::<crate::services::complexity::ComplexityReport>(report_value.clone())
        .ok()
}

/// Extract DAG from JSON result (cognitive complexity <=4)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn extract_dag_from_result(
    result: &serde_json::Value,
) -> Option<crate::models::dag::DependencyGraph> {
    let dag_data = serde_json::from_value::<serde_json::Value>(result.clone()).ok()?;
    parse_dag_data(&dag_data)
}

#[allow(dead_code)]
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub(crate) fn parse_dag_data(
    dag_data: &serde_json::Value,
) -> Option<crate::models::dag::DependencyGraph> {
    // Try to extract basic graph structure from the actual response format
    let node_count = dag_data.get("nodes")?.as_u64().unwrap_or(0) as usize;
    let edge_count = dag_data.get("edges")?.as_u64().unwrap_or(0) as usize;

    // Create a minimal graph structure
    if node_count > 0 || edge_count > 0 {
        return Some(crate::models::dag::DependencyGraph {
            nodes: (0..node_count)
                .map(|i| {
                    let node_id = format!("node_{i}");
                    (
                        node_id.clone(),
                        crate::models::dag::NodeInfo {
                            id: node_id,
                            label: format!("Module {i}"),
                            node_type: crate::models::dag::NodeType::Module,
                            file_path: format!("module_{i}.rs"),
                            line_number: 1,
                            complexity: 1,
                            metadata: Default::default(),
                        },
                    )
                })
                .collect(),
            edges: (0..edge_count)
                .map(|i| crate::models::dag::Edge {
                    from: format!("node_{}", i % node_count),
                    to: format!("node_{}", (i + 1) % node_count),
                    edge_type: crate::models::dag::EdgeType::Imports,
                    weight: 1,
                })
                .collect(),
        });
    }
    None
}
