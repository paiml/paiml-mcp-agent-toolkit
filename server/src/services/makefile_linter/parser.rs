use super::ast::*;

#[derive(Debug)]
pub struct MakefileParser<'src> {
    input: &'src str,
    cursor: usize,
    line: usize,
    column: usize,
    errors: Vec<ParseError>,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedEof,
    InvalidSyntax(String),
    InvalidTarget(String),
    InvalidVariable(String),
}

impl<'src> MakefileParser<'src> {
    pub fn new(input: &'src str) -> Self {
        Self {
            input,
            cursor: 0,
            line: 1,
            column: 1,
            errors: Vec::new(),
        }
    }

    /// Safe string slicing that ensures char boundaries
    fn safe_slice(&self, start: usize, end: usize) -> &str {
        // Handle empty input
        if self.input.is_empty() {
            return "";
        }

        let bytes = self.input.as_bytes();
        let len = bytes.len();

        // Clamp start and end to valid range
        let start = start.min(len);
        let end = end.min(len);

        // Handle invalid range
        if start >= end {
            return "";
        }

        // Find safe start position
        let mut safe_start = start;
        while safe_start > 0 && !self.input.is_char_boundary(safe_start) {
            safe_start -= 1;
        }

        // Find safe end position
        let mut safe_end = end;
        while safe_end < len && !self.input.is_char_boundary(safe_end) {
            safe_end += 1;
        }

        // Final safety check
        if safe_start > safe_end {
            return "";
        }

        &self.input[safe_start..safe_end]
    }

    pub fn parse(&mut self) -> Result<MakefileAst, Vec<ParseError>> {
        let mut ast = MakefileAst::new();

        while !self.at_end() {
            self.skip_whitespace_and_blank_lines();
            if self.at_end() {
                break;
            }

            if let Err(e) = self.parse_line(&mut ast) {
                self.errors.push(e);
                self.skip_to_next_line();
            }
        }

        // Update metadata
        ast.metadata.target_count = ast.count_targets();
        ast.metadata.has_phony_rules = !ast.get_phony_targets().is_empty();
        ast.metadata.has_pattern_rules = ast.has_pattern_rules();
        ast.metadata.uses_automatic_variables = ast.uses_automatic_variables();

        if self.errors.is_empty() {
            Ok(ast)
        } else {
            Err(self.errors.clone())
        }
    }

    fn parse_line(&mut self, ast: &mut MakefileAst) -> Result<(), ParseError> {
        let _start_pos = self.cursor;
        let _start_line = self.line;
        let _start_col = self.column;

        // Skip leading whitespace except tabs (which might indicate recipes)
        self.skip_spaces();

        // Handle special cases first
        if let Some(result) = self.try_parse_special_line(ast)? {
            return result;
        }

        // Look ahead to determine line type
        if let Some(line_type) = self.find_assignment_or_colon() {
            self.parse_line_by_type(ast, line_type)?;
        } else if self.is_directive_line() {
            self.parse_directive_line(ast)?;
        } else {
            // Unknown line type, skip it
            self.skip_to_next_line();
        }

        Ok(())
    }

    fn try_parse_special_line(
        &mut self,
        ast: &mut MakefileAst,
    ) -> Result<Option<Result<(), ParseError>>, ParseError> {
        if self.peek() == Some('#') {
            self.parse_comment(ast);
            return Ok(Some(Ok(())));
        }

        if self.peek() == Some('\t') {
            // This is a recipe line
            return Ok(Some(Err(ParseError::InvalidSyntax(
                "Recipe without rule".to_string(),
            ))));
        }

        Ok(None)
    }

    fn parse_line_by_type(
        &mut self,
        ast: &mut MakefileAst,
        line_type: LineType,
    ) -> Result<(), ParseError> {
        match line_type {
            LineType::Assignment(op_pos, op) => self.parse_variable(ast, op_pos, op),
            LineType::Rule(colon_pos, is_double) => self.parse_rule(ast, colon_pos, is_double),
        }
    }

    fn is_directive_line(&self) -> bool {
        self.starts_with("include")
            || self.starts_with("-include")
            || self.is_conditional_directive()
    }

    fn is_conditional_directive(&self) -> bool {
        self.starts_with("ifeq")
            || self.starts_with("ifneq")
            || self.starts_with("ifdef")
            || self.starts_with("ifndef")
    }

    fn parse_directive_line(&mut self, ast: &mut MakefileAst) -> Result<(), ParseError> {
        if self.starts_with("include") || self.starts_with("-include") {
            self.parse_include(ast)
        } else {
            self.parse_conditional(ast)
        }
    }

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
        self.cursor = (colon_pos + skip_amount).min(self.input.len());
        self.column += colon_pos.saturating_sub(self.cursor) + skip_amount;

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
        input
            .split_whitespace()
            .map(|s| s.to_string())
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
                .map(|s| s.to_string())
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

    // SWAR-optimized character search
    #[allow(dead_code)]
    fn find_char_swar(&self, needle: u8) -> Option<usize> {
        let bytes = self.input.as_bytes();
        let mut pos = self.cursor;

        // Process 8 bytes at a time
        while pos + 8 <= bytes.len() {
            let chunk = u64::from_le_bytes([
                bytes[pos],
                bytes[pos + 1],
                bytes[pos + 2],
                bytes[pos + 3],
                bytes[pos + 4],
                bytes[pos + 5],
                bytes[pos + 6],
                bytes[pos + 7],
            ]);

            // SWAR trick: detect byte in parallel
            let matches = Self::has_byte(chunk, needle);
            if matches != 0 {
                return Some(pos + (matches.trailing_zeros() as usize / 8));
            }
            pos += 8;
        }

        // Handle remainder
        while pos < bytes.len() {
            if bytes[pos] == needle {
                return Some(pos);
            }
            pos += 1;
        }

        None
    }

    #[inline(always)]
    #[allow(dead_code)]
    const fn has_byte(x: u64, n: u8) -> u64 {
        const LO: u64 = 0x0101010101010101;
        const HI: u64 = 0x8080808080808080;

        let r = x ^ (LO * n as u64);
        (r.wrapping_sub(LO)) & !r & HI
    }

    fn find_assignment_or_colon(&self) -> Option<LineType> {
        let bytes = self.input.as_bytes();
        let mut pos = self.cursor;

        while pos < bytes.len() && bytes[pos] != b'\n' {
            if let Some(line_type) = self.check_char_at_position(bytes, pos) {
                return Some(line_type);
            }
            pos += 1;
        }

        None
    }

    fn check_char_at_position(&self, bytes: &[u8], pos: usize) -> Option<LineType> {
        match bytes[pos] {
            b':' => self.check_colon_operator(bytes, pos),
            b'=' => Some(LineType::Assignment(pos, AssignmentOp::Deferred)),
            b'?' => self.check_two_char_operator(bytes, pos, b'=', AssignmentOp::Conditional),
            b'+' => self.check_two_char_operator(bytes, pos, b'=', AssignmentOp::Append),
            b'!' => self.check_two_char_operator(bytes, pos, b'=', AssignmentOp::Shell),
            _ => None,
        }
    }

    fn check_colon_operator(&self, bytes: &[u8], pos: usize) -> Option<LineType> {
        if pos + 1 < bytes.len() {
            match bytes[pos + 1] {
                b'=' => Some(LineType::Assignment(pos, AssignmentOp::Immediate)),
                b':' => Some(LineType::Rule(pos, true)),
                _ => Some(LineType::Rule(pos, false)),
            }
        } else {
            Some(LineType::Rule(pos, false))
        }
    }

    fn check_two_char_operator(
        &self,
        bytes: &[u8],
        pos: usize,
        second_char: u8,
        op: AssignmentOp,
    ) -> Option<LineType> {
        if pos + 1 < bytes.len() && bytes[pos + 1] == second_char {
            Some(LineType::Assignment(pos, op))
        } else {
            None
        }
    }

    // Helper methods
    fn at_end(&self) -> bool {
        self.cursor >= self.input.len()
    }

    fn peek(&self) -> Option<char> {
        if self.cursor >= self.input.len() {
            return None;
        }
        // Ensure we're at a char boundary
        if !self.input.is_char_boundary(self.cursor) {
            return None;
        }
        // Use string slicing to handle UTF-8 correctly
        self.input[self.cursor..].chars().next()
    }

    fn advance(&mut self) {
        // Check if we're at the end first
        if self.cursor >= self.input.len() {
            return;
        }

        if let Some(ch) = self.peek() {
            let len = ch.len_utf8();
            // Ensure we don't go past the end
            self.cursor = (self.cursor + len).min(self.input.len());
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    fn skip_spaces(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_whitespace_and_blank_lines(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_to_next_line(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }
    }

    fn starts_with(&self, s: &str) -> bool {
        if self.cursor >= self.input.len() {
            return false;
        }
        // Ensure we're at a char boundary
        if !self.input.is_char_boundary(self.cursor) {
            return false;
        }
        self.input[self.cursor..].starts_with(s)
    }
}

enum LineType {
    Assignment(usize, AssignmentOp),
    Rule(usize, bool), // position, is_double_colon
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_new() {
        let parser = MakefileParser::new("test input");
        assert_eq!(parser.cursor, 0);
        assert_eq!(parser.line, 1);
        assert_eq!(parser.column, 1);
        assert!(parser.errors.is_empty());
    }

    #[test]
    fn test_parse_empty_file() {
        let mut parser = MakefileParser::new("");
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert_eq!(ast.nodes.len(), 0);
    }

    #[test]
    fn test_parse_simple_rule() {
        let input = "test: dep1 dep2\n\techo hello";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();
        assert!(!ast.nodes.is_empty());

        // Check rule node
        let rule_node = &ast.nodes[0];
        assert_eq!(rule_node.kind, MakefileNodeKind::Rule);
        if let NodeData::Rule {
            targets,
            prerequisites,
            ..
        } = &rule_node.data
        {
            assert_eq!(targets, &vec!["test".to_string()]);
            assert_eq!(prerequisites, &vec!["dep1".to_string(), "dep2".to_string()]);
        } else {
            panic!("Expected Rule node data");
        }
    }

    #[test]
    fn test_parse_variable() {
        let input = "CC = gcc\nCFLAGS := -Wall\nLDFLAGS += -lm";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        let vars = ast.get_variables();
        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0].0, "CC");
        assert_eq!(vars[0].2, "gcc");
        assert_eq!(vars[1].0, "CFLAGS");
        assert_eq!(vars[1].2, "-Wall");
    }

    #[test]
    fn test_parse_comment() {
        let input = "# This is a comment\ntest:\n\techo test";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        // Find comment node
        let comment_node = ast
            .nodes
            .iter()
            .find(|n| n.kind == MakefileNodeKind::Comment);
        assert!(comment_node.is_some());
        if let NodeData::Text(text) = &comment_node.unwrap().data {
            assert!(text.contains("This is a comment"));
        }
    }

    #[test]
    fn test_parse_pattern_rule() {
        let input = "%.o: %.c\n\tgcc -c $< -o $@";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        assert!(ast.has_pattern_rules());
    }

    #[test]
    fn test_parse_phony_rule() {
        let input = ".PHONY: clean test\nclean:\n\trm -f *.o";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        let phony_targets = ast.get_phony_targets();
        assert_eq!(phony_targets.len(), 2);
        assert!(phony_targets.contains(&"clean".to_string()));
        assert!(phony_targets.contains(&"test".to_string()));
    }

    #[test]
    fn test_parse_double_colon_rule() {
        let input = "all:: target1\nall:: target2";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        let all_rules = ast.find_rules_by_target("all");
        assert_eq!(all_rules.len(), 2);
    }

    #[test]
    fn test_parse_include() {
        let input = "include config.mk\n-include optional.mk";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        let include_nodes: Vec<_> = ast
            .nodes
            .iter()
            .filter(|n| n.kind == MakefileNodeKind::Include)
            .collect();
        assert_eq!(include_nodes.len(), 2);
    }

    #[test]
    fn test_parse_recipe_with_prefixes() {
        let input = "test:\n\t@echo Starting test\n\t-rm -f temp\n\t+make subtarget";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        // Find recipe node
        let recipe_node = ast
            .nodes
            .iter()
            .find(|n| n.kind == MakefileNodeKind::Recipe);
        assert!(recipe_node.is_some());

        if let NodeData::Recipe { lines } = &recipe_node.unwrap().data {
            assert_eq!(lines.len(), 3);
            assert!(lines[0].prefixes.silent); // @ prefix
            assert!(lines[1].prefixes.ignore_error); // - prefix
            assert!(lines[2].prefixes.always_exec); // + prefix
        }
    }

    #[test]
    fn test_parse_automatic_variables() {
        let input = "%.o: %.c\n\tgcc -c $< -o $@\n\techo $^ $?";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_ok());
        let ast = result.unwrap();

        assert!(ast.uses_automatic_variables());
    }

    #[test]
    fn test_parse_errors() {
        // Test invalid variable name
        let input = "= value";
        let mut parser = MakefileParser::new(input);
        let result = parser.parse();
        assert!(result.is_err());

        // Test that parse returns Ok but with empty AST for unknown lines
        let input2 = "unknown line";
        let mut parser2 = MakefileParser::new(input2);
        let result2 = parser2.parse();
        assert!(result2.is_ok());
        let ast = result2.unwrap();
        assert_eq!(ast.nodes.len(), 0); // Unknown lines are skipped
    }

    #[test]
    fn test_skip_functions() {
        // skip_spaces should NOT skip tabs (only spaces and carriage returns)
        let mut parser = MakefileParser::new("  \t  text after spaces");
        parser.skip_spaces();
        assert_eq!(parser.peek(), Some('\t'));

        let mut parser2 = MakefileParser::new("line1\nline2");
        parser2.skip_to_next_line();
        assert_eq!(parser2.peek(), Some('l'));
        assert_eq!(parser2.line, 2);
    }

    #[test]
    fn test_at_end() {
        let mut parser = MakefileParser::new("abc");
        assert!(!parser.at_end());
        parser.cursor = 3;
        assert!(parser.at_end());
    }

    #[test]
    fn test_advance() {
        let mut parser = MakefileParser::new("a\nb");
        assert_eq!(parser.line, 1);
        assert_eq!(parser.column, 1);

        parser.advance();
        assert_eq!(parser.column, 2);

        parser.advance(); // newline
        assert_eq!(parser.line, 2);
        assert_eq!(parser.column, 1);
    }

    #[test]
    fn test_starts_with() {
        let parser = MakefileParser::new("include file.mk");
        assert!(parser.starts_with("include"));
        assert!(!parser.starts_with("exclude"));
    }
}
