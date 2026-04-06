// Node-specific parsing: rules, variables, recipes, comments, includes, conditionals
// Included by parser.rs - shares parent module scope (no `use` imports here)

impl<'src> MakefileParser<'src> {
    fn parse_rule(
        &mut self,
        ast: &mut MakefileAst,
        colon_pos: usize,
        is_double: bool,
    ) -> Result<(), ParseError> {
        let _start_pos = self.cursor;
        let _start_line = self.line;
        let _start_col = self.column;

        // Parse targets
        let targets_str = self.safe_slice(self.cursor, colon_pos);
        let targets = self.parse_targets(targets_str)?;

        // Skip past colon(s)
        let skip_amount = if is_double { 2 } else { 1 };
        // Calculate column advance before updating cursor to avoid underflow
        let column_advance = colon_pos.saturating_sub(self.cursor) + skip_amount;
        self.cursor = (colon_pos + skip_amount).min(self.input.len());
        self.column += column_advance;

        // Parse prerequisites
        let prereqs = self.parse_prerequisites()?;

        // Check if this is a pattern rule
        let is_pattern =
            targets.iter().any(|t| t.contains('%')) || prereqs.iter().any(|p| p.contains('%'));

        // Check if this is a phony rule
        let is_phony = targets.contains(&".PHONY".to_string());

        // Create rule node
        let rule_node = MakefileNode {
            kind: MakefileNodeKind::Rule,
            span: SourceSpan::new(_start_pos, self.cursor, _start_line, _start_col),
            children: Vec::new(),
            data: NodeData::Rule {
                targets: targets.clone(),
                prerequisites: prereqs,
                is_pattern,
                is_phony,
                is_double_colon: is_double,
            },
        };

        let rule_idx = ast.add_node(rule_node);

        // Add target nodes
        for target in targets {
            let target_node = MakefileNode {
                kind: MakefileNodeKind::Target,
                span: SourceSpan::new(_start_pos, self.cursor, _start_line, _start_col),
                children: vec![],
                data: NodeData::Target { name: target },
            };
            ast.add_node(target_node);
        }

        // Parse recipe lines if present
        self.skip_to_next_line();
        while !self.at_end() && self.peek() == Some('\t') {
            self.parse_recipe_line(ast, rule_idx)?;
        }

        Ok(())
    }

    fn parse_variable(
        &mut self,
        ast: &mut MakefileAst,
        op_pos: usize,
        op: AssignmentOp,
    ) -> Result<(), ParseError> {
        let _start_pos = self.cursor;
        let _start_line = self.line;
        let _start_col = self.column;

        // Parse variable name
        let name = self.safe_slice(self.cursor, op_pos).trim().to_string();
        if name.is_empty() {
            return Err(ParseError::InvalidVariable(
                "Empty variable name".to_string(),
            ));
        }

        // Skip past operator
        let op_len = match op {
            AssignmentOp::Deferred => 1,
            AssignmentOp::Immediate
            | AssignmentOp::Conditional
            | AssignmentOp::Append
            | AssignmentOp::Shell => 2,
        };
        self.cursor = (op_pos + op_len).min(self.input.len());

        // Parse value (rest of line)
        let value_start = self.cursor;
        self.skip_to_next_line();
        let value = self.safe_slice(value_start, self.cursor).trim().to_string();

        let var_node = MakefileNode {
            kind: MakefileNodeKind::Variable,
            span: SourceSpan::new(_start_pos, self.cursor, _start_line, _start_col),
            children: Vec::new(),
            data: NodeData::Variable {
                name,
                assignment_op: op,
                value,
            },
        };

        ast.add_node(var_node);
        ast.metadata.variable_count += 1;

        Ok(())
    }

    fn parse_recipe_line(
        &mut self,
        ast: &mut MakefileAst,
        rule_idx: usize,
    ) -> Result<(), ParseError> {
        let _start_pos = self.cursor;
        let _start_line = self.line;
        let _start_col = self.column;

        // Skip tab
        self.advance();

        // Parse prefixes
        let mut prefixes = RecipePrefixes::default();
        while let Some(ch) = self.peek() {
            match ch {
                '@' => {
                    prefixes.silent = true;
                    self.advance();
                }
                '-' => {
                    prefixes.ignore_error = true;
                    self.advance();
                }
                '+' => {
                    prefixes.always_exec = true;
                    self.advance();
                }
                _ => break,
            }
        }

        // Get recipe text
        let text_start = self.cursor;
        self.skip_to_next_line();
        let text = self
            .safe_slice(text_start, self.cursor)
            .trim_end()
            .to_string();

        // Check if we can add to existing recipe
        let last_child_idx = ast
            .nodes
            .get(rule_idx)
            .and_then(|node| node.children.last().copied());

        if let Some(idx) = last_child_idx {
            if let Some(last_child) = ast.nodes.get_mut(idx) {
                if let NodeData::Recipe { lines } = &mut last_child.data {
                    lines.push(RecipeLine { text, prefixes });
                    return Ok(());
                }
            }
        }

        // Create new recipe node
        let recipe_node = MakefileNode {
            kind: MakefileNodeKind::Recipe,
            span: SourceSpan::new(_start_pos, self.cursor, _start_line, _start_col),
            children: Vec::new(),
            data: NodeData::Recipe {
                lines: vec![RecipeLine { text, prefixes }],
            },
        };

        let recipe_idx = ast.add_node(recipe_node);
        if let Some(node) = ast.nodes.get_mut(rule_idx) {
            node.children.push(recipe_idx);
        }
        ast.metadata.recipe_count += 1;

        Ok(())
    }

    fn parse_targets(&self, input: &str) -> Result<Vec<String>, ParseError> {
        debug_assert!(!input.is_empty(), "input must not be empty");
        input
            .split_whitespace()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .into_iter()
            .map(Ok)
            .collect()
    }

    fn parse_prerequisites(&mut self) -> Result<Vec<String>, ParseError> {
        let mut prereqs = Vec::new();

        self.skip_spaces();
        let start = self.cursor;

        // Read until end of line or comment
        while !self.at_end() && self.peek() != Some('\n') && self.peek() != Some('#') {
            self.advance();
        }

        let prereq_str = self.safe_slice(start, self.cursor);
        if !prereq_str.trim().is_empty() {
            prereqs = prereq_str
                .split_whitespace()
                .map(std::string::ToString::to_string)
                .collect();
        }

        Ok(prereqs)
    }

    fn parse_comment(&mut self, ast: &mut MakefileAst) {
        let _start_pos = self.cursor;
        let _start_line = self.line;
        let _start_col = self.column;

        self.skip_to_next_line();

        let comment_node = MakefileNode {
            kind: MakefileNodeKind::Comment,
            span: SourceSpan::new(_start_pos, self.cursor, _start_line, _start_col),
            children: Vec::new(),
            data: NodeData::Text(self.safe_slice(_start_pos, self.cursor).to_string()),
        };

        ast.add_node(comment_node);
    }

    fn parse_include(&mut self, ast: &mut MakefileAst) -> Result<(), ParseError> {
        let _start_pos = self.cursor;
        let _start_line = self.line;
        let _start_col = self.column;

        // Skip "include" or "-include"
        if self.starts_with("-include") {
            self.cursor = (self.cursor + 8).min(self.input.len());
            self.column += 8;
        } else {
            self.cursor = (self.cursor + 7).min(self.input.len());
            self.column += 7;
        }

        self.skip_spaces();

        let files_start = self.cursor;
        self.skip_to_next_line();
        let files = self.safe_slice(files_start, self.cursor).trim().to_string();

        let include_node = MakefileNode {
            kind: MakefileNodeKind::Include,
            span: SourceSpan::new(_start_pos, self.cursor, _start_line, _start_col),
            children: Vec::new(),
            data: NodeData::Text(files),
        };

        ast.add_node(include_node);
        Ok(())
    }

    fn parse_conditional(&mut self, _ast: &mut MakefileAst) -> Result<(), ParseError> {
        // For now, just skip conditional blocks
        self.skip_to_next_line();
        Ok(())
    }
}
