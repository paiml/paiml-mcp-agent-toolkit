//! DAG (Directed Acyclic Graph) engine for workflow execution
//!
//! Provides dependency analysis and topological sorting for workflow steps.

use super::*;
use std::collections::{HashMap, HashSet, VecDeque};

/// DAG engine for analyzing workflow dependencies
pub struct DagEngine {
    nodes: HashMap<String, DagNode>,
    edges: HashMap<String, Vec<String>>, // step_id -> dependent step_ids
}

/// Node in the workflow DAG
#[derive(Debug, Clone)]
pub struct DagNode {
    pub id: String,
    pub step: WorkflowStep,
    pub dependencies: Vec<String>,
}

/// Result of DAG analysis
#[derive(Debug, Clone)]
pub struct DagAnalysis {
    pub has_cycles: bool,
    pub cycles: Vec<Vec<String>>,
    pub execution_order: Vec<Vec<String>>, // Levels that can be executed in parallel
    pub critical_path: Vec<String>,
    pub max_parallelism: usize,
}

impl DagEngine {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Build DAG from workflow
    pub fn from_workflow(workflow: &Workflow) -> Result<Self, WorkflowError> {
        let mut engine = Self::new();

        for step in &workflow.steps {
            engine.add_step(step.clone());
        }

        // Analyze dependencies
        engine.extract_dependencies()?;

        Ok(engine)
    }

    /// Add a step to the DAG
    pub fn add_step(&mut self, step: WorkflowStep) {
        let node = DagNode {
            id: step.id.clone(),
            step: step.clone(),
            dependencies: Vec::new(),
        };

        self.nodes.insert(step.id.clone(), node);
        self.edges.insert(step.id, Vec::new());
    }

    /// Add dependency between steps
    pub fn add_dependency(&mut self, from: String, to: String) -> Result<(), WorkflowError> {
        if !self.nodes.contains_key(&from) {
            return Err(WorkflowError::InvalidDefinition(format!("Step not found: {}", from)));
        }

        if !self.nodes.contains_key(&to) {
            return Err(WorkflowError::InvalidDefinition(format!("Step not found: {}", to)));
        }

        // Add edge
        self.edges.entry(from.clone()).or_default().push(to.clone());

        // Add to node dependencies
        if let Some(node) = self.nodes.get_mut(&to) {
            node.dependencies.push(from);
        }

        Ok(())
    }

    /// Detect cycles in the DAG
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                self.dfs_cycle_detection(
                    node_id,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn dfs_cycle_detection(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = self.edges.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_cycle_detection(neighbor, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|n| n == neighbor).unwrap();
                    let cycle = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    /// Perform topological sort to get execution order
    pub fn topological_sort(&self) -> Result<Vec<Vec<String>>, WorkflowError> {
        // Check for cycles first
        let cycles = self.detect_cycles();
        if !cycles.is_empty() {
            return Err(WorkflowError::InvalidDefinition(format!(
                "Workflow contains cycles: {:?}",
                cycles
            )));
        }

        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut execution_order = Vec::new();

        // Calculate in-degrees
        for node_id in self.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
        }

        for edges in self.edges.values() {
            for to in edges {
                *in_degree.get_mut(to).unwrap() += 1;
            }
        }

        // Find all nodes with in-degree 0
        let mut queue: VecDeque<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        while !queue.is_empty() {
            let mut level = Vec::new();

            // Process all nodes at this level (can be executed in parallel)
            let level_size = queue.len();
            for _ in 0..level_size {
                if let Some(node) = queue.pop_front() {
                    level.push(node.clone());

                    // Reduce in-degree of neighbors
                    if let Some(neighbors) = self.edges.get(&node) {
                        for neighbor in neighbors {
                            let degree = in_degree.get_mut(neighbor).unwrap();
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push_back(neighbor.clone());
                            }
                        }
                    }
                }
            }

            if !level.is_empty() {
                execution_order.push(level);
            }
        }

        // Check if all nodes were processed
        if execution_order.iter().flatten().count() != self.nodes.len() {
            return Err(WorkflowError::InvalidDefinition(
                "Workflow contains unreachable nodes".to_string(),
            ));
        }

        Ok(execution_order)
    }

    /// Analyze the DAG
    pub fn analyze(&self) -> Result<DagAnalysis, WorkflowError> {
        let cycles = self.detect_cycles();
        let has_cycles = !cycles.is_empty();

        let execution_order = if has_cycles {
            Vec::new()
        } else {
            self.topological_sort()?
        };

        let max_parallelism = execution_order.iter().map(|level| level.len()).max().unwrap_or(0);

        let critical_path = self.find_critical_path();

        Ok(DagAnalysis {
            has_cycles,
            cycles,
            execution_order,
            critical_path,
            max_parallelism,
        })
    }

    fn find_critical_path(&self) -> Vec<String> {
        // Simple implementation: longest path through the DAG
        // In production, would use actual execution time estimates
        let mut longest_path = Vec::new();

        for start_node in self.nodes.keys() {
            let path = self.dfs_longest_path(start_node, &mut HashSet::new());
            if path.len() > longest_path.len() {
                longest_path = path;
            }
        }

        longest_path
    }

    fn dfs_longest_path(&self, node: &str, visited: &mut HashSet<String>) -> Vec<String> {
        if visited.contains(node) {
            return Vec::new();
        }

        visited.insert(node.to_string());

        let mut longest = vec![node.to_string()];

        if let Some(neighbors) = self.edges.get(node) {
            for neighbor in neighbors {
                let mut path = self.dfs_longest_path(neighbor, visited);
                path.insert(0, node.to_string());
                if path.len() > longest.len() {
                    longest = path;
                }
            }
        }

        visited.remove(node);
        longest
    }

    fn extract_dependencies(&mut self) -> Result<(), WorkflowError> {
        // Extract dependencies from step conditions and references
        // For now, sequential steps have implicit dependencies
        let node_ids: Vec<String> = self.nodes.keys().cloned().collect();

        for i in 0..node_ids.len().saturating_sub(1) {
            self.add_dependency(node_ids[i].clone(), node_ids[i + 1].clone())?;
        }

        Ok(())
    }
}

impl Default for DagEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_simple_step(id: &str) -> WorkflowStep {
        WorkflowStep {
            id: id.to_string(),
            name: id.to_string(),
            step_type: StepType::Wait {
                duration: Duration::from_secs(1),
            },
            condition: None,
            retry: None,
            timeout: None,
            on_error: None,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_dag_creation() {
        let engine = DagEngine::new();
        assert_eq!(engine.nodes.len(), 0);
    }

    #[test]
    fn test_add_step() {
        let mut engine = DagEngine::new();
        let step = create_simple_step("step1");

        engine.add_step(step);
        assert_eq!(engine.nodes.len(), 1);
        assert!(engine.nodes.contains_key("step1"));
    }

    #[test]
    fn test_add_dependency() {
        let mut engine = DagEngine::new();
        engine.add_step(create_simple_step("step1"));
        engine.add_step(create_simple_step("step2"));

        engine.add_dependency("step1".to_string(), "step2".to_string()).unwrap();

        assert_eq!(engine.edges.get("step1").unwrap().len(), 1);
    }

    #[test]
    fn test_cycle_detection_no_cycles() {
        let mut engine = DagEngine::new();
        engine.add_step(create_simple_step("step1"));
        engine.add_step(create_simple_step("step2"));
        engine.add_step(create_simple_step("step3"));

        engine.add_dependency("step1".to_string(), "step2".to_string()).unwrap();
        engine.add_dependency("step2".to_string(), "step3".to_string()).unwrap();

        let cycles = engine.detect_cycles();
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_cycle_detection_with_cycle() {
        let mut engine = DagEngine::new();
        engine.add_step(create_simple_step("step1"));
        engine.add_step(create_simple_step("step2"));
        engine.add_step(create_simple_step("step3"));

        engine.add_dependency("step1".to_string(), "step2".to_string()).unwrap();
        engine.add_dependency("step2".to_string(), "step3".to_string()).unwrap();
        engine.add_dependency("step3".to_string(), "step1".to_string()).unwrap();

        let cycles = engine.detect_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_topological_sort() {
        let mut engine = DagEngine::new();
        engine.add_step(create_simple_step("step1"));
        engine.add_step(create_simple_step("step2"));
        engine.add_step(create_simple_step("step3"));

        engine.add_dependency("step1".to_string(), "step2".to_string()).unwrap();
        engine.add_dependency("step1".to_string(), "step3".to_string()).unwrap();

        let order = engine.topological_sort().unwrap();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0], vec!["step1"]);
        assert_eq!(order[1].len(), 2); // step2 and step3 can run in parallel
    }

    #[test]
    fn test_analyze() {
        let mut engine = DagEngine::new();
        engine.add_step(create_simple_step("step1"));
        engine.add_step(create_simple_step("step2"));
        engine.add_step(create_simple_step("step3"));

        engine.add_dependency("step1".to_string(), "step2".to_string()).unwrap();
        engine.add_dependency("step1".to_string(), "step3".to_string()).unwrap();

        let analysis = engine.analyze().unwrap();
        assert!(!analysis.has_cycles);
        assert_eq!(analysis.max_parallelism, 2);
    }
}
