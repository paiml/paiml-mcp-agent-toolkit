impl RuchyComplexityAnalyzer {
    /// Analyze a Ruchy AST node for complexity
    fn analyze_node(&mut self, node: &RuchyAst) {
        match node {
            RuchyAst::Function {
                name,
                body,
                line_start,
                line_end,
                ..
            } => {
                self.analyze_function(name, body, *line_start, *line_end);
            }
            RuchyAst::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.analyze_if(condition, then_branch, else_branch.as_deref());
            }
            RuchyAst::While { condition, body } => {
                self.analyze_while(condition, body);
            }
            RuchyAst::For { body, .. } => {
                self.analyze_for(body);
            }
            RuchyAst::Match { expr, arms } => {
                self.analyze_match(expr, arms);
            }
            RuchyAst::BinaryOp { left, op, right } => {
                self.analyze_binary_op(left, op, right);
            }
            RuchyAst::Block { statements } => {
                self.analyze_block(statements);
            }
            RuchyAst::Import {
                module,
                items,
                line,
            } => {
                self.analyze_import(module, items, *line);
            }
            RuchyAst::Export { items, .. } => {
                self.analyze_export(items);
            }
            RuchyAst::Actor {
                name,
                state,
                handlers,
                line_start,
                line_end,
            } => {
                self.analyze_actor(name, state, handlers, *line_start, *line_end);
            }
            _ => {
                // Other nodes don't affect complexity directly
            }
        }
    }

    /// Analyze function complexity
    fn analyze_function(&mut self, name: &str, body: &RuchyAst, line_start: u32, line_end: u32) {
        self.defined_functions.insert(name.to_string());
        self.track_operator("fun");
        self.track_operand(name);

        let prev_complexity = self.current_complexity;
        let prev_nesting = self.nesting_level;

        self.current_complexity = ComplexityMetrics {
            cyclomatic: 1,
            cognitive: 0,
            nesting_max: 0,
            lines: (line_end - line_start) as u16,
            halstead: None,
        };
        self.nesting_level = 0;
        self.reset_halstead();

        self.analyze_node(body);

        let halstead = self.calculate_halstead();
        self.current_complexity.halstead = Some(halstead);

        self.functions.push(FunctionComplexity {
            name: name.to_string(),
            line_start,
            line_end,
            metrics: self.current_complexity,
        });

        self.current_complexity = prev_complexity;
        self.nesting_level = prev_nesting;
    }

    /// Analyze if statement complexity
    fn analyze_if(
        &mut self,
        condition: &RuchyAst,
        then_branch: &RuchyAst,
        else_branch: Option<&RuchyAst>,
    ) {
        self.current_complexity.cyclomatic += 1;
        self.current_complexity.cognitive += 1 + u16::from(self.nesting_level);
        self.track_operator("if");

        self.nesting_level += 1;
        self.current_complexity.nesting_max =
            self.current_complexity.nesting_max.max(self.nesting_level);

        self.analyze_node(condition);
        self.analyze_node(then_branch);
        if let Some(else_br) = else_branch {
            self.current_complexity.cyclomatic += 1;
            self.track_operator("else");
            self.analyze_node(else_br);
        }

        self.nesting_level -= 1;
    }

    /// Analyze while loop complexity
    fn analyze_while(&mut self, condition: &RuchyAst, body: &RuchyAst) {
        self.current_complexity.cyclomatic += 1;
        self.current_complexity.cognitive += 1 + u16::from(self.nesting_level);

        self.nesting_level += 1;
        self.current_complexity.nesting_max =
            self.current_complexity.nesting_max.max(self.nesting_level);

        self.analyze_node(condition);
        self.analyze_node(body);

        self.nesting_level -= 1;
    }

    /// Analyze for loop complexity
    fn analyze_for(&mut self, body: &RuchyAst) {
        self.current_complexity.cyclomatic += 1;
        self.current_complexity.cognitive += 1 + u16::from(self.nesting_level);

        self.nesting_level += 1;
        self.current_complexity.nesting_max =
            self.current_complexity.nesting_max.max(self.nesting_level);

        self.analyze_node(body);

        self.nesting_level -= 1;
    }

    /// Analyze match expression complexity
    fn analyze_match(&mut self, expr: &RuchyAst, arms: &[(RuchyAst, RuchyAst)]) {
        let arm_count = arms.len() as u16;
        self.current_complexity.cyclomatic += arm_count;
        self.current_complexity.cognitive += (arm_count * 2) + u16::from(self.nesting_level);

        self.track_operator("match");

        self.nesting_level += 1;
        self.current_complexity.nesting_max =
            self.current_complexity.nesting_max.max(self.nesting_level);

        self.analyze_node(expr);
        for (pattern, body) in arms {
            self.analyze_pattern_complexity(pattern);
            self.analyze_node(body);
        }

        self.nesting_level -= 1;
    }

    /// Analyze binary operation complexity
    fn analyze_binary_op(&mut self, left: &RuchyAst, op: &RuchyToken, right: &RuchyAst) {
        // Toyota Way Extract Method: Separate concerns for operator processing
        let op_str = Self::get_operator_string(op);
        self.track_operator(op_str);

        // Toyota Way Extract Method: Handle complexity tracking for logical operators
        self.handle_logical_operator_complexity(op);

        // Analyze operands
        self.analyze_node(left);
        self.analyze_node(right);
    }

    /// Toyota Way Extract Method: Get string representation of operator
    /// Single responsibility: operator token to string conversion
    fn get_operator_string(op: &RuchyToken) -> &'static str {
        match op {
            RuchyToken::Plus => "+",
            RuchyToken::Minus => "-",
            RuchyToken::Star => "*",
            RuchyToken::Slash => "/",
            RuchyToken::Percent => "%",
            RuchyToken::EqualEqual => "==",
            RuchyToken::NotEqual => "!=",
            RuchyToken::Less => "<",
            RuchyToken::Greater => ">",
            RuchyToken::LessEqual => "<=",
            RuchyToken::GreaterEqual => ">=",
            RuchyToken::And => "&&",
            RuchyToken::Or => "||",
            RuchyToken::PipeForward => "|>",
            _ => "op",
        }
    }

    /// Toyota Way Extract Method: Handle complexity tracking for logical operators
    /// Single responsibility: complexity increment for short-circuit operators
    fn handle_logical_operator_complexity(&mut self, op: &RuchyToken) {
        if matches!(op, RuchyToken::And | RuchyToken::Or) {
            self.current_complexity.cyclomatic += 1;
            self.current_complexity.cognitive += 1;
        }
    }

    /// Analyze block complexity
    fn analyze_block(&mut self, statements: &[RuchyAst]) {
        for stmt in statements {
            self.analyze_node(stmt);
        }
    }

    /// Analyze import statement
    fn analyze_import(&mut self, module: &str, items: &[String], line: u32) {
        self.imports.push(RuchyImport {
            module: module.to_string(),
            items: items.to_vec(),
            line,
        });
        self.track_operator("import");
        self.track_operand(module);
    }

    /// Analyze export statement
    fn analyze_export(&mut self, items: &[String]) {
        for item in items {
            self.exports.insert(item.clone());
        }
        self.track_operator("export");
    }

    /// Analyze actor complexity
    fn analyze_actor(
        &mut self,
        name: &str,
        state: &[(String, String)],
        handlers: &[RuchyAst],
        line_start: u32,
        line_end: u32,
    ) {
        self.track_operator("actor");
        self.track_operand(name);

        let prev_actor = self.current_actor.clone();
        self.current_actor = Some(name.to_string());

        let mut actor_info = ActorInfo {
            name: name.to_string(),
            state_fields: state.iter().map(|(field, _)| field.clone()).collect(),
            message_handlers: Vec::new(),
            spawned_actors: Vec::new(),
            line_start,
            line_end,
        };

        let mut class_complexity = ComplexityMetrics::default();

        for handler in handlers {
            if let RuchyAst::Function {
                name: handler_name, ..
            } = handler
            {
                actor_info.message_handlers.push(handler_name.clone());
            }
            self.analyze_node(handler);
            if let RuchyAst::Function { .. } = handler {
                if let Some(func) = self.functions.last() {
                    class_complexity.cyclomatic += func.metrics.cyclomatic;
                    class_complexity.cognitive += func.metrics.cognitive;
                    class_complexity.nesting_max =
                        class_complexity.nesting_max.max(func.metrics.nesting_max);
                }
            }
        }

        self.actors.push(actor_info);
        self.classes
            .push(crate::services::complexity::ClassComplexity {
                name: name.to_string(),
                line_start,
                line_end,
                metrics: class_complexity,
                methods: vec![],
            });

        self.current_actor = prev_actor;
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn analyze_program(&mut self, ast: &RuchyAst) -> FileComplexityMetrics {
        if let RuchyAst::Program { items } = ast {
            for item in items {
                self.analyze_node(item);
            }
        } else {
            self.analyze_node(ast);
        }

        // Calculate total file complexity
        let total_complexity = ComplexityMetrics {
            cyclomatic: self
                .functions
                .iter()
                .map(|f| f.metrics.cyclomatic)
                .sum::<u16>()
                .max(1),
            cognitive: self
                .functions
                .iter()
                .map(|f| f.metrics.cognitive)
                .sum::<u16>()
                .max(1),
            nesting_max: self
                .functions
                .iter()
                .map(|f| f.metrics.nesting_max)
                .max()
                .unwrap_or(0),
            lines: self.functions.iter().map(|f| f.metrics.lines).sum::<u16>(),
            halstead: None,
        };

        FileComplexityMetrics {
            path: String::new(), // Will be set by caller
            total_complexity,
            functions: self.functions.clone(),
            classes: self.classes.clone(),
        }
    }
}
