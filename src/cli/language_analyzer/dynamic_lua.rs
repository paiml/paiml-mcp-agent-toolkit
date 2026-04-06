// LuaAnalyzer implementation methods (included from dynamic.rs)

impl LuaAnalyzer {
    // ===== Tree-sitter implementation =====

    #[cfg(feature = "lua-ast")]
    fn extract_functions_treesitter(&self, content: &str) -> Option<Vec<FunctionInfo>> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        use tree_sitter::Parser as TsParser;
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_lua::LANGUAGE.into())
            .ok()?;
        let tree = parser.parse(content, None)?;
        let mut functions = Vec::new();
        Self::collect_functions(&tree.root_node(), content, &mut functions);
        Some(functions)
    }

    #[cfg(feature = "lua-ast")]
    fn collect_functions(node: &tree_sitter::Node, source: &str, out: &mut Vec<FunctionInfo>) {
        debug_assert!(true, "contract: collect_functions");
        match node.kind() {
            "function_declaration" | "function_definition" => {
                let name = Self::ts_function_name(node, source);
                out.push(FunctionInfo {
                    name,
                    line_start: node.start_position().row,
                    line_end: node.end_position().row,
                });
            }
            _ => {}
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::collect_functions(&child, source, out);
        }
    }

    #[cfg(feature = "lua-ast")]
    fn ts_function_name(node: &tree_sitter::Node, source: &str) -> String {
        debug_assert!(true, "contract: ts_function_name");
        if let Some(name_node) = node.child_by_field_name("name") {
            return source[name_node.byte_range()].to_string();
        }
        // Anonymous function -- try parent assignment: local foo = function(...)
        if let Some(parent) = node.parent() {
            if parent.kind() == "assignment_statement" || parent.kind() == "variable_declaration" {
                if let Some(var_node) = parent.child_by_field_name("name") {
                    return source[var_node.byte_range()].to_string();
                }
            }
        }
        format!("<anonymous>:{}", node.start_position().row + 1)
    }

    #[cfg(feature = "lua-ast")]
    #[allow(clippy::cast_possible_truncation)]
    fn estimate_complexity_treesitter(
        &self,
        content: &str,
        function: &FunctionInfo,
    ) -> Option<ComplexityMetrics> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        use tree_sitter::Parser as TsParser;
        let mut parser = TsParser::new();
        parser
            .set_language(&tree_sitter_lua::LANGUAGE.into())
            .ok()?;
        let tree = parser.parse(content, None)?;

        let mut cyc = 1u16;
        let mut cog = 0u16;
        let mut max_nest = 0u8;
        let mut lines = 0u16;

        Self::find_and_analyze_function(
            &tree.root_node(),
            content,
            function.line_start,
            &mut cyc,
            &mut cog,
            &mut max_nest,
            &mut lines,
        );

        if lines == 0 {
            return None;
        }

        Some(ComplexityMetrics {
            cyclomatic: cyc.min(255),
            cognitive: cog.min(255),
            nesting_max: max_nest,
            lines,
            halstead: None,
        })
    }

    #[cfg(feature = "lua-ast")]
    #[allow(clippy::cast_possible_truncation)]
    fn find_and_analyze_function(
        node: &tree_sitter::Node,
        source: &str,
        target_line: usize,
        cyc: &mut u16,
        cog: &mut u16,
        max_nest: &mut u8,
        lines: &mut u16,
    ) {
        debug_assert!(true, "contract: find_and_analyze_function");
        if (node.kind() == "function_declaration" || node.kind() == "function_definition")
            && node.start_position().row == target_line
        {
            *lines = (node.end_position().row - node.start_position().row + 1) as u16;
            Self::walk_complexity(node, source, 0, cyc, cog, max_nest);
            return;
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::find_and_analyze_function(&child, source, target_line, cyc, cog, max_nest, lines);
            if *lines > 0 {
                return;
            }
        }
    }

    #[cfg(feature = "lua-ast")]
    #[allow(clippy::cast_possible_truncation)]
    fn walk_complexity(
        node: &tree_sitter::Node,
        source: &str,
        depth: u8,
        cyc: &mut u16,
        cog: &mut u16,
        max_nest: &mut u8,
    ) {
        debug_assert!(true, "contract: walk_complexity");
        match node.kind() {
            "if_statement" | "for_statement" | "while_statement" | "repeat_statement" => {
                *cyc += 1;
                *cog += 1 + u16::from(depth);
                *max_nest = (*max_nest).max(depth + 1);
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    Self::walk_complexity(&child, source, depth + 1, cyc, cog, max_nest);
                }
                return;
            }
            "elseif_statement" => {
                *cyc += 1;
                *cog += 1 + u16::from(depth);
            }
            "binary_expression" => {
                if let Some(op) = node.child_by_field_name("operator") {
                    let op_text = &source[op.byte_range()];
                    if op_text == "and" || op_text == "or" {
                        *cyc += 1;
                        *cog += 1;
                    }
                }
            }
            _ => {}
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::walk_complexity(&child, source, depth, cyc, cog, max_nest);
        }
    }

    // ===== Heuristic fallback =====

    fn extract_functions_heuristic(&self, content: &str) -> Vec<FunctionInfo> {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let mut functions = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if self.is_function_declaration(trimmed) {
                if let Some(name) = self.extract_function_name(trimmed) {
                    let line_end = self.find_function_end(&lines, line_num);
                    functions.push(FunctionInfo {
                        name,
                        line_start: line_num,
                        line_end,
                    });
                }
            }
        }
        functions
    }

    fn estimate_complexity_heuristic(
        &self,
        content: &str,
        function: &FunctionInfo,
    ) -> ComplexityMetrics {
        debug_assert!(!content.is_empty(), "content must not be empty");
        let lines: Vec<&str> = content.lines().collect();
        let end = function.line_end.min(lines.len() - 1);
        let function_lines = &lines[function.line_start..=end];

        let mut cyclomatic: u16 = 1;
        let mut cognitive: u16 = 0;
        let mut nesting: u8 = 0;
        let mut max_nesting: u8 = 0;

        for line in function_lines {
            let trimmed = line.trim();
            if trimmed.starts_with("if ")
                || trimmed.starts_with("elseif ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("repeat")
            {
                cyclomatic += 1;
                cognitive += 1 + u16::from(nesting);
            }
            if trimmed.contains(" and ") || trimmed.contains(" or ") {
                cyclomatic += 1;
                cognitive += 1;
            }
            if trimmed.starts_with("function ")
                || trimmed.starts_with("local function ")
                || trimmed.starts_with("if ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed.starts_with("do")
                || trimmed.starts_with("repeat")
            {
                nesting += 1;
                max_nesting = max_nesting.max(nesting);
            }
            if trimmed == "end" || trimmed.starts_with("end)") || trimmed.starts_with("end,") {
                nesting = nesting.saturating_sub(1);
            }
            if trimmed.starts_with("until ") {
                nesting = nesting.saturating_sub(1);
            }
        }

        ComplexityMetrics {
            cyclomatic: cyclomatic.min(255),
            cognitive: cognitive.min(255),
            nesting_max: max_nesting,
            lines: function_lines.len() as u16,
            halstead: None,
        }
    }

    fn is_function_declaration(&self, line: &str) -> bool {
        (line.starts_with("function ") || line.starts_with("local function ")) && line.contains('(')
    }

    fn extract_function_name(&self, line: &str) -> Option<String> {
        debug_assert!(!line.is_empty(), "line must not be empty");
        let after = if let Some(rest) = line.strip_prefix("local function ") {
            rest
        } else if let Some(rest) = line.strip_prefix("function ") {
            rest
        } else {
            return None;
        };
        let paren_pos = after.find('(')?;
        let name = after.get(..paren_pos).unwrap_or_default().trim();
        if name.is_empty() {
            return None;
        }
        Some(name.to_string())
    }

    fn find_function_end(&self, lines: &[&str], start: usize) -> usize {
        debug_assert!(true, "contract: find_function_end");
        let mut depth: i32 = 0;
        let mut found_first = false;

        for (i, line) in lines.iter().enumerate().skip(start) {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            if trimmed.starts_with("function ")
                || trimmed.starts_with("local function ")
                || trimmed.starts_with("if ")
                || trimmed.starts_with("for ")
                || trimmed.starts_with("while ")
                || trimmed == "do"
                || trimmed.starts_with("do ")
                || trimmed.starts_with("repeat")
            {
                depth += 1;
                found_first = true;
            }
            if trimmed == "end"
                || trimmed.starts_with("end)")
                || trimmed.starts_with("end,")
                || trimmed.starts_with("end ")
            {
                depth -= 1;
                if found_first && depth <= 0 {
                    return i;
                }
            }
            if trimmed.starts_with("until ") {
                depth -= 1;
                if found_first && depth <= 0 {
                    return i;
                }
            }
        }
        lines.len() - 1
    }
}
