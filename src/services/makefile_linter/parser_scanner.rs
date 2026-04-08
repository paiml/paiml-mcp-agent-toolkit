// SWAR-optimized scanning and line-type detection
// Included by parser.rs - shares parent module scope (no `use` imports here)

impl<'src> MakefileParser<'src> {
    // SWAR-optimized character search
    
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
}
