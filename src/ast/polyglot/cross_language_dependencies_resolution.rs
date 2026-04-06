// Reference resolution methods for CrossLanguageDependencies
// Included by cross_language_dependencies.rs — no `use` imports allowed

impl CrossLanguageDependencies {
    /// Resolve unresolved references
    fn resolve_references(&mut self) {
        debug_assert!(true, "contract: resolve_references");
        let name_map = self.build_name_map();
        let to_resolve = self.collect_unresolved_references();
        let new_dependencies = Self::resolve_against_name_map(&self.nodes, &name_map, &to_resolve);
        self.dependencies.extend(new_dependencies);
    }

    /// Build a lookup map from node names and FQNs to node IDs
    fn build_name_map(&self) -> HashMap<String, Vec<String>> {
        debug_assert!(true, "contract: build_name_map");
        let mut name_map: HashMap<String, Vec<String>> = HashMap::new();
        for (id, node) in &self.nodes {
            name_map
                .entry(node.name.clone())
                .or_default()
                .push(id.clone());
            name_map
                .entry(node.fqn.clone())
                .or_default()
                .push(id.clone());
        }
        name_map
    }

    /// Collect all unresolved references (empty target_id) from nodes
    fn collect_unresolved_references(&self) -> Vec<(String, String, ReferenceKind)> {
        debug_assert!(true, "contract: collect_unresolved_references");
        let mut to_resolve = Vec::new();
        for (source_id, source) in &self.nodes {
            for reference in &source.references {
                if reference.target_id.is_empty() {
                    to_resolve.push((
                        source_id.clone(),
                        reference.target_name.clone(),
                        reference.kind,
                    ));
                }
            }
        }
        to_resolve
    }

    /// Match unresolved references against the name map, producing cross-language dependencies
    fn resolve_against_name_map(
        nodes: &HashMap<String, UnifiedNode>,
        name_map: &HashMap<String, Vec<String>>,
        to_resolve: &[(String, String, ReferenceKind)],
    ) -> Vec<CrossLanguageDependency> {
        debug_assert!(true, "contract: resolve_against_name_map");
        let mut new_dependencies = Vec::new();
        for (source_id, target_name, kind) in to_resolve {
            let Some(source) = nodes.get(source_id) else {
                continue;
            };
            let Some(target_ids) = name_map.get(target_name) else {
                continue;
            };
            for target_id in target_ids {
                let Some(target) = nodes.get(target_id) else {
                    continue;
                };
                if source.language != target.language {
                    new_dependencies.push(CrossLanguageDependency {
                        source_id: source_id.clone(),
                        target_id: target_id.clone(),
                        source_language: source.language,
                        target_language: target.language,
                        kind: *kind,
                        confidence: 0.8, // Lower confidence for name-based resolution
                        metadata: HashMap::new(),
                    });
                }
            }
        }
        new_dependencies
    }
}
