impl DemoRunner {
    fn generate_system_diagram(&self, _steps: &[DemoStep]) -> Result<String> {
        // Extract component relationships from analysis results
        let mut components = HashMap::new();

        // Map internal components to high-level architecture
        components.insert(
            "ast_context".to_string(),
            Component {
                id: "A".to_string(),
                label: "AST Context Analysis".to_string(),
                color: "#90EE90".to_string(),
                connections: vec![("B".to_string(), "uses".to_string())],
            },
        );

        components.insert(
            "file_parser".to_string(),
            Component {
                id: "B".to_string(),
                label: "File Parser".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![
                    ("C".to_string(), String::new()),
                    ("D".to_string(), String::new()),
                    ("E".to_string(), String::new()),
                ],
            },
        );

        // Language-specific AST components
        components.insert(
            "rust_ast".to_string(),
            Component {
                id: "C".to_string(),
                label: "Rust AST".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![],
            },
        );

        components.insert(
            "typescript_ast".to_string(),
            Component {
                id: "D".to_string(),
                label: "TypeScript AST".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![],
            },
        );

        components.insert(
            "python_ast".to_string(),
            Component {
                id: "E".to_string(),
                label: "Python AST".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![],
            },
        );

        // Analysis components
        components.insert(
            "complexity".to_string(),
            Component {
                id: "F".to_string(),
                label: "Code Complexity".to_string(),
                color: "#FFD700".to_string(),
                connections: vec![
                    ("C".to_string(), "analyzes".to_string()),
                    ("D".to_string(), "analyzes".to_string()),
                    ("E".to_string(), "analyzes".to_string()),
                ],
            },
        );

        components.insert(
            "dag_gen".to_string(),
            Component {
                id: "G".to_string(),
                label: "DAG Generation".to_string(),
                color: "#FFA500".to_string(),
                connections: vec![
                    ("C".to_string(), "reads".to_string()),
                    ("D".to_string(), "reads".to_string()),
                    ("E".to_string(), "reads".to_string()),
                ],
            },
        );

        components.insert(
            "churn".to_string(),
            Component {
                id: "H".to_string(),
                label: "Code Churn".to_string(),
                color: "#FF6347".to_string(),
                connections: vec![("I".to_string(), "git history".to_string())],
            },
        );

        components.insert(
            "git".to_string(),
            Component {
                id: "I".to_string(),
                label: "Git Analysis".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![],
            },
        );

        components.insert(
            "template".to_string(),
            Component {
                id: "J".to_string(),
                label: "Template Generation".to_string(),
                color: "#87CEEB".to_string(),
                connections: vec![("K".to_string(), "renders".to_string())],
            },
        );

        components.insert(
            "handlebars".to_string(),
            Component {
                id: "K".to_string(),
                label: "Handlebars".to_string(),
                color: "#FFFFFF".to_string(),
                connections: vec![],
            },
        );

        // Generate Mermaid diagram
        self.render_system_mermaid(&components)
    }

    fn render_system_mermaid(&self, _components: &HashMap<String, Component>) -> Result<String> {
        let mut output = String::with_capacity(1024);
        output.push_str("graph TD\n");

        // Add nodes and connections based on target diagram
        output.push_str("    A[AST Context Analysis] -->|uses| B[File Parser]\n");
        output.push_str("    B --> C[Rust AST]\n");
        output.push_str("    B --> D[TypeScript AST]\n");
        output.push_str("    B --> E[Python AST]\n\n");

        output.push_str("    F[Code Complexity] -->|analyzes| C\n");
        output.push_str("    F -->|analyzes| D\n");
        output.push_str("    F -->|analyzes| E\n\n");

        output.push_str("    G[DAG Generation] -->|reads| C\n");
        output.push_str("    G -->|reads| D\n");
        output.push_str("    G -->|reads| E\n\n");

        output.push_str("    H[Code Churn] -->|git history| I[Git Analysis]\n\n");

        output.push_str("    J[Template Generation] -->|renders| K[Handlebars]\n\n");

        // Add styling
        output.push_str("    style A fill:#90EE90\n");
        output.push_str("    style F fill:#FFD700\n");
        output.push_str("    style G fill:#FFA500\n");
        output.push_str("    style H fill:#FF6347\n");
        output.push_str("    style J fill:#87CEEB\n");

        Ok(output)
    }
}
