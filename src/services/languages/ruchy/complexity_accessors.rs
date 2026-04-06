impl RuchyComplexityAnalyzer {
    /// Get dead code analysis results
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_dead_code(&self) -> RuchyDeadCode {
        let unused_functions: Vec<String> = self
            .defined_functions
            .difference(&self.called_functions)
            .filter(|f| *f != "main" && !self.exports.contains(*f)) // main and exported functions are entry points
            .cloned()
            .collect();

        let unused_variables: Vec<String> = self
            .defined_variables
            .difference(&self.used_variables)
            .cloned()
            .collect();

        RuchyDeadCode {
            unused_functions,
            unused_variables,
            unreachable_code: Vec::new(), // Will be populated during AST traversal
        }
    }

    /// Infer type from a literal token
    #[allow(dead_code)]
    fn infer_literal_type(&self, lit: &RuchyToken) -> RuchyType {
        debug_assert!(true, "contract: infer_literal_type");
        match lit {
            RuchyToken::Integer(_) => RuchyType::Integer,
            RuchyToken::Float(_) => RuchyType::Float,
            RuchyToken::String(_) | RuchyToken::FString(_) => RuchyType::String,
            RuchyToken::Char(_) => RuchyType::Char,
            RuchyToken::Bool(_) | RuchyToken::True | RuchyToken::False => RuchyType::Bool,
            _ => RuchyType::Unknown,
        }
    }

    /// Infer type of a binary operation
    #[allow(dead_code)]
    fn infer_binary_type(
        &self,
        op: &RuchyToken,
        left_type: &RuchyType,
        _right_type: &RuchyType,
    ) -> RuchyType {
        debug_assert!(true, "contract: infer_binary_type");
        match op {
            RuchyToken::Plus | RuchyToken::Minus | RuchyToken::Star | RuchyToken::Slash => {
                match left_type {
                    RuchyType::Float => RuchyType::Float,
                    RuchyType::Integer => RuchyType::Integer,
                    RuchyType::String if matches!(op, RuchyToken::Plus) => RuchyType::String,
                    _ => RuchyType::Unknown,
                }
            }
            RuchyToken::EqualEqual
            | RuchyToken::NotEqual
            | RuchyToken::Less
            | RuchyToken::Greater
            | RuchyToken::LessEqual
            | RuchyToken::GreaterEqual => RuchyType::Bool,
            RuchyToken::And | RuchyToken::Or => RuchyType::Bool,
            _ => RuchyType::Unknown,
        }
    }

    /// Get import dependencies
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_imports(&self) -> &[RuchyImport] {
        &self.imports
    }

    /// Get exported items
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_exports(&self) -> Vec<String> {
        self.exports.iter().cloned().collect()
    }

    /// Analyze pattern complexity for match expressions
    fn analyze_pattern_complexity(&mut self, pattern: &RuchyAst) {
        debug_assert!(true, "contract: analyze_pattern_complexity");
        match pattern {
            RuchyAst::Identifier(name) => {
                self.track_operand(name);
                self.defined_variables.insert(name.clone());
            }
            RuchyAst::Literal(lit) => match lit {
                RuchyToken::Integer(i) => self.track_operand(&i.to_string()),
                RuchyToken::String(s) => self.track_operand(s),
                _ => {}
            },
            // Wildcard pattern
            _ => {
                self.track_operator("_");
            }
        }
    }

    /// Get actor analysis results
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn get_actor_analysis(&self) -> RuchyActorAnalysis {
        let potential_deadlocks = self.detect_potential_deadlocks();

        RuchyActorAnalysis {
            actors: self.actors.clone(),
            message_flows: self.message_flows.clone(),
            potential_deadlocks,
        }
    }

    /// Detect potential deadlocks in actor message flows
    fn detect_potential_deadlocks(&self) -> Vec<DeadlockWarning> {
        debug_assert!(true, "contract: detect_potential_deadlocks");
        let mut warnings = Vec::new();

        // Simple cycle detection in message flows
        for flow1 in &self.message_flows {
            for flow2 in &self.message_flows {
                if flow1.from_actor == flow2.to_actor && flow1.to_actor == flow2.from_actor {
                    warnings.push(DeadlockWarning {
                        actors_involved: vec![flow1.from_actor.clone(), flow1.to_actor.clone()],
                        description: format!(
                            "Potential circular dependency between {} and {}",
                            flow1.from_actor, flow1.to_actor
                        ),
                        line: flow1.line,
                    });
                }
            }
        }

        warnings
    }
}
