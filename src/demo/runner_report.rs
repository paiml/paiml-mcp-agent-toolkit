impl DemoReport {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Render the content.
    pub fn render(&self, mode: ExecutionMode) -> String {
        match mode {
            ExecutionMode::Cli => self.render_cli(),
            ExecutionMode::Mcp => serde_json::to_string_pretty(self)
                .expect("JSON serialization cannot fail for DemoResult"),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn render_cli(&self) -> String {
        let mut output = String::with_capacity(4096);

        writeln!(&mut output, "\n🎯 PAIML MCP Agent Toolkit Demo Complete")
            .expect("Writing to String buffer cannot fail");
        writeln!(&mut output, "Repository: {}", self.repository)
            .expect("Writing to String buffer cannot fail");
        writeln!(&mut output, "\n📊 Capabilities Demonstrated:\n")
            .expect("Writing to String buffer cannot fail");

        for (idx, step) in self.steps.iter().enumerate() {
            writeln!(
                &mut output,
                "{}. {} ({} ms)",
                idx + 1,
                step.capability,
                step.elapsed_ms
            )
            .expect("Writing to String buffer cannot fail");

            // Extract key metrics from response
            if let Some(result) = &step.response.result {
                self.render_step_highlights(&mut output, step.capability, result);
            }
        }

        writeln!(
            &mut output,
            "\n⏱️  Total execution time: {} ms",
            self.total_time_ms
        )
        .expect("Writing to String buffer cannot fail");

        // Add system diagram if available
        if let Some(ref diagram) = self.system_diagram {
            writeln!(&mut output, "\n🌍 System Architecture:")
                .expect("Writing to String buffer cannot fail");
            writeln!(&mut output, "```mermaid").expect("Writing to String buffer cannot fail");
            writeln!(&mut output, "{diagram}").expect("Writing to String buffer cannot fail");
            writeln!(&mut output, "```").expect("Writing to String buffer cannot fail");
        }

        writeln!(
            &mut output,
            "\n🚀 Get started with PAIML MCP Agent Toolkit:"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(
            &mut output,
            "   - Generate templates: paiml-mcp-agent-toolkit scaffold <toolchain>"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(
            &mut output,
            "   - Analyze complexity: paiml-mcp-agent-toolkit analyze complexity"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(
            &mut output,
            "   - View code churn: paiml-mcp-agent-toolkit analyze churn"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(
            &mut output,
            "   - Create DAGs: paiml-mcp-agent-toolkit analyze dag"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(
            &mut output,
            "   - System architecture: paiml-mcp-agent-toolkit analyze architecture"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(
            &mut output,
            "   - Defect probability: paiml-mcp-agent-toolkit analyze defects"
        )
        .expect("Writing to String buffer cannot fail");
        writeln!(&mut output).expect("Writing to String buffer cannot fail");
        writeln!(
            &mut output,
            "📊 To view Mermaid diagrams: https://mermaid.live"
        )
        .expect("Writing to String buffer cannot fail");

        output
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn format_highlight(capability: &str, result: &Value) -> Option<String> {
        let v: Value = serde_json::from_value(result.clone()).ok()?;
        match capability {
            "Code Complexity Analysis" => {
                let total = v.get("total_functions")?;
                let warnings = v.get("total_warnings")?;
                let errors = v.get("total_errors")?;
                Some(format!("      Functions: {total}, Warnings: {warnings}, Errors: {errors}"))
            }
            "DAG Visualization" => {
                let stats = v.get("stats")?;
                let nodes = stats.get("nodes")?;
                let edges = stats.get("edges")?;
                Some(format!("      Graph size: {nodes} nodes, {edges} edges"))
            }
            "Code Churn Analysis" => {
                let files = v.get("files_analyzed")?;
                let total_churn = v.get("total_churn_score")?;
                Some(format!("      Files analyzed: {files}, Total churn: {total_churn}"))
            }
            "System Architecture Analysis" => {
                let metadata = v.get("metadata")?;
                let nodes = metadata.get("nodes")?;
                let edges = metadata.get("edges")?;
                Some(format!("      Components: {nodes}, Relationships: {edges}"))
            }
            "Defect Probability Analysis" => {
                let high_risk = v.get("high_risk_files")?;
                let avg_prob = v.get("average_probability")?;
                Some(format!(
                    "      High-risk files: {}, Avg probability: {:.2}",
                    high_risk.as_array().map_or(0, std::vec::Vec::len),
                    avg_prob.as_f64().unwrap_or(0.0)
                ))
            }
            _ => None,
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn render_step_highlights(&self, output: &mut String, capability: &str, result: &Value) {
        if let Some(line) = Self::format_highlight(capability, result) {
            writeln!(output, "{line}").expect("Writing to String buffer cannot fail");
        }
    }
}
