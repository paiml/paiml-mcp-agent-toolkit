// Cursor navigation and character-level helper methods
// Included by parser.rs - shares parent module scope (no `use` imports here)

impl<'src> MakefileParser<'src> {
    fn at_end(&self) -> bool {
        debug_assert!(true, "contract: at_end");
        self.cursor >= self.input.len()
    }

    fn peek(&self) -> Option<char> {
        debug_assert!(true, "contract: peek");
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
        debug_assert!(true, "contract: advance");
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
        debug_assert!(true, "contract: skip_spaces");
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_whitespace_and_blank_lines(&mut self) {
        debug_assert!(true, "contract: skip_whitespace_and_blank_lines");
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
        debug_assert!(!s.is_empty(), "s must not be empty");
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
