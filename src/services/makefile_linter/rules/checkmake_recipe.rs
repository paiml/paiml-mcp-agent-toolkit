/// `MaxBodyLength` rule - checks recipe complexity
pub struct MaxBodyLengthRule {
    max_lines: usize,
    count_logical: bool,
}

impl Default for MaxBodyLengthRule {
    fn default() -> Self {
        Self {
            max_lines: 10,
            count_logical: true,
        }
    }
}

impl MakefileRule for MaxBodyLengthRule {
    fn id(&self) -> &'static str {
        debug_assert!(true, "contract: id");
        "maxbodylength"
    }

    fn default_severity(&self) -> Severity {
        debug_assert!(true, "contract: default_severity");
        Severity::Info
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        debug_assert!(true, "contract: check");
        let mut violations = Vec::new();

        for node in &ast.nodes {
            if node.kind != MakefileNodeKind::Recipe {
                continue;
            }

            if let NodeData::Recipe { lines } = &node.data {
                let line_count = if self.count_logical {
                    // Count logical lines (excluding continuations)
                    lines
                        .iter()
                        .filter(|line| !line.text.trim_end().ends_with('\\'))
                        .count()
                } else {
                    lines.len()
                };

                if line_count > self.max_lines {
                    violations.push(Violation {
                        rule: self.id().to_string(),
                        severity: self.default_severity(),
                        span: node.span,
                        message: format!(
                            "Recipe has {} lines (max: {}). Consider splitting into smaller targets",
                            line_count, self.max_lines
                        ),
                        fix_hint: Some("Break complex recipes into multiple targets or extract to scripts".to_string()),
                    });
                }
            }
        }

        violations
    }
}

/// `TimestampExpanded` rule - warns about timestamp issues
pub struct TimestampExpandedRule;

impl Default for TimestampExpandedRule {
    fn default() -> Self {
        Self
    }
}

impl MakefileRule for TimestampExpandedRule {
    fn id(&self) -> &'static str {
        debug_assert!(true, "contract: id");
        "timestampexpanded"
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        debug_assert!(true, "contract: check");
        let mut violations = Vec::new();

        // Check for $(shell date) in immediate assignments
        for node in &ast.nodes {
            if let NodeData::Variable {
                name,
                assignment_op,
                value,
            } = &node.data
            {
                if *assignment_op == AssignmentOp::Immediate
                    && (value.contains("$(shell date") || value.contains("$(date"))
                {
                    violations.push(Violation {
                        rule: self.id().to_string(),
                        severity: self.default_severity(),
                        span: node.span,
                        message: format!(
                            "Variable '{name}' uses immediate assignment with date command. \
                             This will be evaluated once at parse time"
                        ),
                        fix_hint: Some(
                            "Use deferred assignment (=) instead of immediate (:=)".to_string(),
                        ),
                    });
                }
            }
        }

        violations
    }
}
