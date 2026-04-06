/// `MinPhony` rule - ensures required targets are declared as .PHONY
pub struct MinPhonyRule {
    required_targets: Vec<String>,
    check_exists: bool,
}

impl Default for MinPhonyRule {
    fn default() -> Self {
        Self {
            required_targets: vec!["all".to_string(), "clean".to_string(), "test".to_string()],
            check_exists: true,
        }
    }
}

impl MakefileRule for MinPhonyRule {
    fn id(&self) -> &'static str {
        debug_assert!(true, "contract: id");
        "minphony"
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        debug_assert!(true, "contract: check");
        let mut violations = Vec::new();
        let phony_targets: HashSet<_> = ast.get_phony_targets().into_iter().collect();

        // Collect all defined targets
        let mut defined_targets = HashSet::new();
        for node in &ast.nodes {
            if let NodeData::Rule { targets, .. } = &node.data {
                for target in targets {
                    if !target.starts_with('.') {
                        defined_targets.insert(target.clone());
                    }
                }
            }
        }

        // Check required targets
        for required in &self.required_targets {
            let exists = defined_targets.contains(required);
            let is_phony = phony_targets.contains(required);

            if (!self.check_exists || exists) && !is_phony {
                violations.push(Violation {
                    rule: self.id().to_string(),
                    severity: self.default_severity(),
                    span: SourceSpan::file_level(),
                    message: format!("Target '{required}' should be declared .PHONY"),
                    fix_hint: Some(format!("Add '.PHONY: {required}' to your Makefile")),
                });
            }
        }

        violations
    }
}

/// `PhonyDeclared` rule - warns about targets that should be .PHONY
pub struct PhonyDeclaredRule {
    ignore_suffixes: Vec<String>,
}

impl Default for PhonyDeclaredRule {
    fn default() -> Self {
        Self {
            ignore_suffixes: vec![
                ".o".to_string(),
                ".a".to_string(),
                ".so".to_string(),
                ".exe".to_string(),
                ".ko".to_string(),
                ".mod".to_string(),
            ],
        }
    }
}

impl MakefileRule for PhonyDeclaredRule {
    fn id(&self) -> &'static str {
        debug_assert!(true, "contract: id");
        "phonydeclared"
    }

    fn default_severity(&self) -> Severity {
        debug_assert!(true, "contract: default_severity");
        Severity::Info
    }

    fn check(&self, ast: &MakefileAst) -> Vec<Violation> {
        debug_assert!(true, "contract: check");
        let mut violations = Vec::new();
        let phony_targets: HashSet<_> = ast.get_phony_targets().into_iter().collect();

        for node in &ast.nodes {
            if node.kind != MakefileNodeKind::Rule {
                continue;
            }

            if let NodeData::Rule { targets, .. } = &node.data {
                for target in targets {
                    // Skip special targets
                    if target.starts_with('.') || target.contains('/') || target.contains('%') {
                        continue;
                    }

                    // Skip files with known extensions
                    if self
                        .ignore_suffixes
                        .iter()
                        .any(|suffix| target.ends_with(suffix))
                    {
                        continue;
                    }

                    // Check if declared phony
                    if !phony_targets.contains(target) {
                        violations.push(Violation {
                            rule: self.id().to_string(),
                            severity: self.default_severity(),
                            span: node.span,
                            message: format!(
                                "Target '{target}' should probably be declared .PHONY"
                            ),
                            fix_hint: Some(format!("Add '{target}' to .PHONY declaration")),
                        });
                    }
                }
            }
        }

        violations
    }
}
