/// Portability rule - checks for GNU Make specific features
pub struct PortabilityRule;

impl Default for PortabilityRule {
    fn default() -> Self {
        Self
    }
}

impl MakefileRule for PortabilityRule {
    fn id(&self) -> &'static str {
        "portability"
    }

    fn default_severity(&self) -> Severity {
        Severity::Info
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Check for GNU-specific assignment operators
        for node in &ast.nodes {
            if let NodeData::Variable { assignment_op, .. } = &node.data {
                match assignment_op {
                    AssignmentOp::Conditional => {
                        violations.push(Violation {
                            rule: self.id().to_string(),
                            severity: self.default_severity(),
                            span: node.span,
                            message: "Conditional assignment (?=) is GNU Make specific".to_string(),
                            fix_hint: Some(
                                "Use ifdef/ifndef for portable conditional assignment".to_string(),
                            ),
                        });
                    }
                    AssignmentOp::Shell => {
                        violations.push(Violation {
                            rule: self.id().to_string(),
                            severity: self.default_severity(),
                            span: node.span,
                            message: "Shell assignment (!=) is GNU Make specific".to_string(),
                            fix_hint: Some(
                                "Use $(shell ...) for portable shell execution".to_string(),
                            ),
                        });
                    }
                    _ => {}
                }
            }
        }

        violations
    }
}
