// Cursor navigation and character-level helper methods
// Included by parser.rs - shares parent module scope (no `use` imports here)

impl<'src> MakefileParser<'src> {
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

    /// Advance past blank space between top-level lines — but never past a TAB.
    ///
    /// A tab is not decoration in a Makefile, it is the recipe marker. This used
    /// to skip it along with everything else `char::is_whitespace` matches,
    /// which made `try_parse_special_line`'s "Recipe without rule" error
    /// unreachable: by the time it peeked, the tab had already been consumed and
    /// the line fell through to the "unknown line type, skip it" branch. A
    /// Makefile whose first line is `\techo orphan` — rejected by GNU make with
    /// "recipe commences before first target" — parsed clean.
    ///
    /// Recipe lines belonging to a rule are consumed by `parse_rule`, so a tab
    /// still visible at this level is always a recipe with no rule above it.
    ///
    /// A tab with nothing but whitespace after it is NOT a recipe — GNU make
    /// ignores such a line — so it is skipped like any other blank.
    fn skip_whitespace_and_blank_lines(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\t' && self.tab_starts_a_recipe_line() {
                break;
            }
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// At a TAB: does the rest of this line carry a command, or is it blank?
    fn tab_starts_a_recipe_line(&self) -> bool {
        self.input[self.cursor..]
            .chars()
            .take_while(|&c| c != '\n')
            .any(|c| !c.is_whitespace())
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
