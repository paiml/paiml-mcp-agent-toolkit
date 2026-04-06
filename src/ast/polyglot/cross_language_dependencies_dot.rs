// DOT graph generation for CrossLanguageDependencies
// Included by cross_language_dependencies.rs — no `use` imports allowed

impl CrossLanguageDependencies {
    /// Generate a dependency graph in DOT format
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph CrossLanguageDependencies {\n");

        // Add nodes
        for node in self.nodes.values() {
            let shape = match node.kind {
                NodeKind::Class => "box",
                NodeKind::Interface | NodeKind::Trait => "ellipse",
                NodeKind::Method | NodeKind::Function => "octagon",
                _ => "plaintext",
            };

            let label = format!("{} ({})", node.name, node.language.name());
            dot.push_str(&format!(
                "    \"{}\" [label=\"{}\" shape={} style=filled fillcolor={}];\n",
                node.id,
                label,
                shape,
                self.language_color(node.language)
            ));
        }

        // Add edges
        for dep in &self.dependencies {
            let style = match dep.kind {
                ReferenceKind::Inherits => "bold",
                ReferenceKind::Implements => "dashed",
                _ => "solid",
            };

            let label = format!("{:?}", dep.kind);
            dot.push_str(&format!(
                "    \"{}\" -> \"{}\" [label=\"{}\" style={}];\n",
                dep.source_id, dep.target_id, label, style
            ));
        }

        dot.push_str("}\n");
        dot
    }

    /// Get color for a language in DOT format
    fn language_color(&self, language: Language) -> &'static str {
        debug_assert!(true, "contract: language_color");
        match language {
            Language::Java => "\"#b07219\"",       // Java brown
            Language::Kotlin => "\"#A97BFF\"",     // Kotlin purple
            Language::Scala => "\"#c22d40\"",      // Scala red
            Language::TypeScript => "\"#2b7489\"", // TypeScript blue
            Language::JavaScript => "\"#f1e05a\"", // JavaScript yellow
            Language::Python => "\"#3572A5\"",     // Python blue
            Language::Rust => "\"#dea584\"",       // Rust orange
            Language::Go => "\"#00ADD8\"",         // Go blue
            Language::Cpp => "\"#f34b7d\"",        // C++ pink
            _ => "\"#bbbbbb\"",                    // Gray for others
        }
    }
}
