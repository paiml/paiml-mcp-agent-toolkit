// Detection methods for CrossLanguageDependencies
// Included by cross_language_dependencies.rs — no `use` imports allowed

impl CrossLanguageDependencies {
    /// Detect dependencies between nodes of two specific language groups
    /// This function avoids borrowing self mutably while iterating
    fn detect_between_language_groups(
        &self,
        node_ids1: &[String],
        lang1: Language,
        node_ids2: &[String],
        lang2: Language,
    ) -> Vec<CrossLanguageDependency> {
        let mut dependencies = Vec::new();

        for id1 in node_ids1 {
            let Some(source) = self.nodes.get(id1) else {
                continue;
            };
            for reference in &source.references {
                self.find_matching_targets(
                    source,
                    reference,
                    node_ids2,
                    lang1,
                    lang2,
                    &mut dependencies,
                );
            }
        }

        dependencies
    }

    /// Check each target node for a reference match and collect dependencies
    fn find_matching_targets(
        &self,
        source: &UnifiedNode,
        reference: &NodeReference,
        node_ids2: &[String],
        lang1: Language,
        lang2: Language,
        dependencies: &mut Vec<CrossLanguageDependency>,
    ) {
        for id2 in node_ids2 {
            let Some(target) = self.nodes.get(id2) else {
                continue;
            };
            if self.is_reference_match(source, reference, target, lang1, lang2) {
                dependencies.push(CrossLanguageDependency {
                    source_id: source.id.clone(),
                    target_id: target.id.clone(),
                    source_language: lang1,
                    target_language: lang2,
                    kind: reference.kind,
                    confidence: 1.0,
                    metadata: HashMap::new(),
                });
            }
        }
    }

    /// Detect dependencies between nodes of two specific languages
    #[allow(dead_code)]
    fn detect_between_languages(
        &mut self,
        nodes1: &[&UnifiedNode],
        lang1: Language,
        nodes2: &[&UnifiedNode],
        lang2: Language,
    ) {
        // For each node in first language
        for &source in nodes1 {
            // For each reference in the node
            for reference in &source.references {
                // For each node in second language
                for &target in nodes2 {
                    // Check if reference matches target
                    if self.is_reference_match(source, reference, target, lang1, lang2) {
                        self.add_dependency(source, target, reference.kind, 1.0);
                    }
                }
            }
        }
    }

    /// Check if a reference from one node matches a target node
    fn is_reference_match(
        &self,
        source: &UnifiedNode,
        reference: &crate::ast::polyglot::unified_node::NodeReference,
        target: &UnifiedNode,
        source_lang: Language,
        target_lang: Language,
    ) -> bool {
        // Direct ID match
        if !reference.target_id.is_empty() && reference.target_id == target.id {
            return true;
        }

        // Name match
        if reference.target_name == target.name || reference.target_name == target.fqn {
            return true;
        }

        // Try to resolve using language-specific resolver
        if let Some(resolver) = self.name_resolvers.get(&source_lang) {
            if resolver.can_resolve(source_lang, target_lang, source, reference, target) {
                return true;
            }
        }

        false
    }

    /// Add a dependency between two nodes
    #[allow(dead_code)]
    fn add_dependency(
        &mut self,
        source: &UnifiedNode,
        target: &UnifiedNode,
        kind: ReferenceKind,
        confidence: f64,
    ) {
        debug_assert!(confidence >= 0.0, "confidence must be non-negative");
        let dependency = CrossLanguageDependency {
            source_id: source.id.clone(),
            target_id: target.id.clone(),
            source_language: source.language,
            target_language: target.language,
            kind,
            confidence,
            metadata: HashMap::new(),
        };

        self.dependencies.push(dependency);
    }
}
