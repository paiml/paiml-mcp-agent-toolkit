#![cfg_attr(coverage_nightly, coverage(off))]
//! Ruchy complexity analyzer: AST-based cyclomatic, cognitive, and Halstead metrics.

use std::collections::{HashMap, HashSet};

use crate::services::complexity::{
    ComplexityMetrics, FileComplexityMetrics, FunctionComplexity, HalsteadMetrics,
};

use super::types::{
    ActorInfo, MessageFlow, DeadlockWarning, RuchyAst, RuchyActorAnalysis, RuchyDeadCode,
    RuchyImport, RuchyToken, RuchyType,
};

/// Ruchy complexity analyzer
pub struct RuchyComplexityAnalyzer {
    pub(super) current_complexity: ComplexityMetrics,
    pub(super) nesting_level: u8,
    pub(super) functions: Vec<FunctionComplexity>,
    pub(super) classes: Vec<crate::services::complexity::ClassComplexity>,
    // Halstead metrics tracking
    pub(super) operators: HashSet<String>,
    pub(super) operands: HashSet<String>,
    pub(super) operator_count: u32,
    pub(super) operand_count: u32,
    // Dead code tracking
    pub(super) defined_functions: HashSet<String>,
    pub(super) called_functions: HashSet<String>,
    pub(super) defined_variables: HashSet<String>,
    pub(super) used_variables: HashSet<String>,
    // Type inference tracking
    #[allow(dead_code)]
    pub(super) type_environment: HashMap<String, RuchyType>,
    // Import/dependency tracking
    pub(super) imports: Vec<RuchyImport>,
    pub(super) exports: HashSet<String>,
    // Actor analysis tracking
    pub(super) actors: Vec<ActorInfo>,
    pub(super) current_actor: Option<String>,
    pub(super) message_flows: Vec<MessageFlow>,
    pub(super) _spawn_calls: Vec<(String, String, u32)>, // (spawner, spawned, line)
}

impl Default for RuchyComplexityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl RuchyComplexityAnalyzer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            current_complexity: ComplexityMetrics::default(),
            nesting_level: 0,
            functions: Vec::new(),
            classes: Vec::new(),
            operators: HashSet::new(),
            operands: HashSet::new(),
            operator_count: 0,
            operand_count: 0,
            defined_functions: HashSet::new(),
            called_functions: HashSet::new(),
            defined_variables: HashSet::<String>::new(),
            used_variables: HashSet::new(),
            type_environment: HashMap::new(),
            imports: Vec::new(),
            exports: HashSet::new(),
            actors: Vec::new(),
            current_actor: None,
            message_flows: Vec::new(),
            _spawn_calls: Vec::new(),
        }
    }

    /// Reset Halstead tracking for a new function
    pub(super) fn reset_halstead(&mut self) {
        self.operators.clear();
        self.operands.clear();
        self.operator_count = 0;
        self.operand_count = 0;
    }

    /// Track an operator for Halstead metrics
    pub(super) fn track_operator(&mut self, op: &str) {
        self.operators.insert(op.to_string());
        self.operator_count += 1;
    }

    /// Track an operand for Halstead metrics
    pub(super) fn track_operand(&mut self, operand: &str) {
        self.operands.insert(operand.to_string());
        self.operand_count += 1;
    }

    /// Calculate Halstead metrics for current function
    pub(super) fn calculate_halstead(&self) -> HalsteadMetrics {
        let operators_unique = self.operators.len() as u32;
        let operands_unique = self.operands.len() as u32;
        let operators_total = self.operator_count;
        let operands_total = self.operand_count;

        let n = f64::from(operators_unique + operands_unique);
        let n_total = f64::from(operators_total + operands_total);

        let volume = if n > 0.0 { n_total * n.log2() } else { 0.0 };
        let difficulty = if operands_unique > 0 {
            (f64::from(operators_unique) / 2.0)
                * (f64::from(operands_total) / f64::from(operands_unique))
        } else {
            0.0
        };
        let effort = volume * difficulty;
        let time = effort / 18.0; // Stroud number
        let bugs = volume / 3000.0; // Industry average

        HalsteadMetrics {
            operators_unique,
            operands_unique,
            operators_total,
            operands_total,
            volume,
            difficulty,
            effort,
            time,
            bugs,
        }
    }

    /// Get dead code analysis results
    #[must_use]
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
    pub fn get_imports(&self) -> &[RuchyImport] {
        &self.imports
    }

    /// Get exported items
    #[must_use]
    pub fn get_exports(&self) -> Vec<String> {
        self.exports.iter().cloned().collect()
    }

    /// Analyze pattern complexity for match expressions
    fn analyze_pattern_complexity(&mut self, pattern: &RuchyAst) {
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
