// Core parsing methods for MakefileParser
// Included by parser.rs - shares parent module scope (no `use` imports here)

impl<'src> MakefileParser<'src> {
    #[must_use]
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
}
