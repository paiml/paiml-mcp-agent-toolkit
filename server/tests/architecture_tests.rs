#[cfg(test)]
mod architecture_tests {
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_no_circular_dependencies() {
        let deps = analyze_module_dependencies();
        let cycles = find_cycles(&deps);

        assert!(
            cycles.is_empty(),
            "Circular dependencies detected: {:?}",
            cycles
        );
    }

    #[test]
    fn test_module_boundary_violations() {
        let violations = check_module_boundaries();

        assert!(
            violations.is_empty(),
            "Module boundary violations: {:?}",
            violations
        );
    }

    #[test]
    fn test_layer_access_restrictions() {
        // Define architectural layers
        let layers = vec![
            Layer {
                name: "agents",
                level: 0,
            },
            Layer {
                name: "modules",
                level: 1,
            },
            Layer {
                name: "quality",
                level: 2,
            },
            Layer {
                name: "services",
                level: 3,
            },
        ];

        let violations = check_layer_violations(&layers);

        assert!(
            violations.is_empty(),
            "Layer access violations: {:?}",
            violations
        );
    }

    #[test]
    fn test_dependency_graph_acyclic() {
        let graph = build_dependency_graph();

        assert!(is_dag(&graph), "Dependency graph is not acyclic (DAG)");
    }

    #[test]
    fn test_module_interface_only_dependencies() {
        // Ensure modules only depend on public interfaces
        let deps = analyze_module_dependencies();

        for (module, dependencies) in deps.iter() {
            for dep in dependencies {
                assert!(
                    is_public_interface(dep),
                    "Module {} depends on private implementation of {}",
                    module,
                    dep
                );
            }
        }
    }

    // Property-based test for dependency consistency
    #[test]
    fn proptest_dependency_graph_properties() {
        use proptest::prelude::*;

        proptest!(|(num_modules in 1..20usize)| {
            let graph = generate_test_graph(num_modules);

            // Property 1: No self-loops
            for node in graph.keys() {
                prop_assert!(!graph[node].contains(node));
            }

            // Property 2: Transitive closure is finite
            let closure = transitive_closure(&graph);
            prop_assert!(closure.len() <= num_modules * num_modules);
        });
    }

    struct Layer {
        name: &'static str,
        level: usize,
    }

    fn analyze_module_dependencies() -> HashMap<String, HashSet<String>> {
        let mut deps = HashMap::new();

        // Analyze Rust source files for module dependencies
        let src_dir = Path::new("src");
        analyze_directory(src_dir, &mut deps);

        deps
    }

    fn analyze_directory(dir: &Path, deps: &mut HashMap<String, HashSet<String>>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_dir() {
                    analyze_directory(&path, deps);
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    analyze_rust_file(&path, deps);
                }
            }
        }
    }

    fn analyze_rust_file(path: &Path, deps: &mut HashMap<String, HashSet<String>>) {
        if let Ok(content) = fs::read_to_string(path) {
            let module_name = path
                .strip_prefix("src/")
                .ok()
                .and_then(|p| p.to_str())
                .map(|s| s.replace('/', "::").replace(".rs", ""))
                .unwrap_or_default();

            let mut module_deps = HashSet::new();

            // Find use statements
            for line in content.lines() {
                if line.trim_start().starts_with("use ") {
                    if let Some(dep) = extract_dependency(line) {
                        if !dep.starts_with("std::") && !dep.starts_with("core::") {
                            module_deps.insert(dep);
                        }
                    }
                }
            }

            deps.insert(module_name, module_deps);
        }
    }

    fn extract_dependency(line: &str) -> Option<String> {
        let line = line
            .trim_start()
            .strip_prefix("use ")?
            .trim_end_matches(';');

        // Handle various import patterns
        if let Some(idx) = line.find("::") {
            let module = &line[..idx];
            if module.starts_with("crate::") || module.starts_with("super::") {
                return Some(module.replace("crate::", "").replace("super::", ""));
            }
        }

        None
    }

    fn find_cycles(deps: &HashMap<String, HashSet<String>>) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();

        for node in deps.keys() {
            if !visited.contains(node) {
                dfs_cycles(node, deps, &mut visited, &mut stack, &mut cycles);
            }
        }

        cycles
    }

    fn dfs_cycles(
        node: &str,
        deps: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        if stack.contains(&node.to_string()) {
            // Found a cycle
            let idx = stack.iter().position(|n| n == node).unwrap();
            cycles.push(stack[idx..].to_vec());
            return;
        }

        if visited.contains(node) {
            return;
        }

        visited.insert(node.to_string());
        stack.push(node.to_string());

        if let Some(neighbors) = deps.get(node) {
            for neighbor in neighbors {
                dfs_cycles(neighbor, deps, visited, stack, cycles);
            }
        }

        stack.pop();
    }

    fn check_module_boundaries() -> Vec<String> {
        let mut violations = Vec::new();

        // Check that modules only expose trait interfaces
        let modules = ["analyzer", "transformer", "validator", "orchestrator"];

        for module in &modules {
            let module_path = format!("src/modules/{}.rs", module);
            if let Ok(content) = fs::read_to_string(module_path) {
                // Check for public struct fields (should be private)
                if content.contains("pub struct") && !content.contains("pub trait") {
                    let has_private_fields = content
                        .lines()
                        .any(|line| line.trim().starts_with("pub ") && line.contains(':'));

                    if has_private_fields {
                        violations.push(format!("{} exposes public struct fields", module));
                    }
                }
            }
        }

        violations
    }

    fn check_layer_violations(layers: &[Layer]) -> Vec<String> {
        let mut violations = Vec::new();
        let deps = analyze_module_dependencies();

        for (module, module_deps) in deps.iter() {
            if let Some(module_layer) = find_layer(module, layers) {
                for dep in module_deps {
                    if let Some(dep_layer) = find_layer(dep, layers) {
                        // Higher layers can depend on lower layers, but not vice versa
                        if dep_layer.level < module_layer.level {
                            violations.push(format!(
                                "{} (layer {}) depends on {} (layer {})",
                                module, module_layer.name, dep, dep_layer.name
                            ));
                        }
                    }
                }
            }
        }

        violations
    }

    fn find_layer<'a>(module: &str, layers: &'a [Layer]) -> Option<&'a Layer> {
        layers.iter().find(|layer| module.starts_with(layer.name))
    }

    fn build_dependency_graph() -> HashMap<String, HashSet<String>> {
        analyze_module_dependencies()
    }

    fn is_dag(graph: &HashMap<String, HashSet<String>>) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in graph.keys() {
            if !visited.contains(node) {
                if has_cycle_dfs(node, graph, &mut visited, &mut rec_stack) {
                    return false;
                }
            }
        }

        true
    }

    fn has_cycle_dfs(
        node: &str,
        graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if has_cycle_dfs(neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    fn is_public_interface(module: &str) -> bool {
        // Check if the dependency is on a public trait interface
        module.ends_with("Module") || module.ends_with("Trait") || module.ends_with("Interface")
    }

    fn generate_test_graph(num_modules: usize) -> HashMap<String, HashSet<String>> {
        let mut graph = HashMap::new();

        for i in 0..num_modules {
            let module = format!("module_{}", i);
            let mut deps = HashSet::new();

            // Create a DAG by only allowing forward dependencies
            for j in (i + 1)..num_modules.min(i + 3) {
                deps.insert(format!("module_{}", j));
            }

            graph.insert(module, deps);
        }

        graph
    }

    fn transitive_closure(graph: &HashMap<String, HashSet<String>>) -> HashSet<(String, String)> {
        let mut closure = HashSet::new();

        for (node, neighbors) in graph {
            for neighbor in neighbors {
                closure.insert((node.clone(), neighbor.clone()));
                add_transitive_deps(neighbor, graph, node, &mut closure);
            }
        }

        closure
    }

    fn add_transitive_deps(
        node: &str,
        graph: &HashMap<String, HashSet<String>>,
        source: &str,
        closure: &mut HashSet<(String, String)>,
    ) {
        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                closure.insert((source.to_string(), neighbor.clone()));
                add_transitive_deps(neighbor, graph, source, closure);
            }
        }
    }
}

// Integration test for quality gates
#[test]
fn test_quality_gates_integration() {
    use pmat::quality::gate::QualityGateRunner;
    use std::fs;
    
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.rs");

    // Write clean code
    let clean_code = r#"
        fn calculate_sum(numbers: &[i32]) -> i32 {
            numbers.iter().sum()
        }

        fn main() {
            let nums = vec![1, 2, 3, 4, 5];
            println!("Sum: {}", calculate_sum(&nums));
        }
    "#;

    fs::write(&file_path, clean_code).unwrap();

    let runner = QualityGateRunner::strict();
    let result = runner.validate_module(&file_path);

    assert!(result.is_ok(), "Clean code should pass quality gates");
}
