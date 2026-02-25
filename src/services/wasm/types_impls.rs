impl std::fmt::Display for Severity {
    ///
    /// Returns an error if the operation fails
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "Low"),
            Severity::Medium => write!(f, "Medium"),
            Severity::High => write!(f, "High"),
            Severity::Critical => write!(f, "Critical"),
        }
    }
}

impl From<u8> for WasmOpcode {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => Self::Unreachable,
            0x01 => Self::Nop,
            0x02 => Self::Block,
            0x03 => Self::Loop,
            0x04 => Self::If,
            0x05 => Self::Else,
            0x0B => Self::End,
            0x0C => Self::Br,
            0x0D => Self::BrIf,
            0x0E => Self::BrTable,
            0x0F => Self::Return,
            0x10 => Self::Call,
            0x11 => Self::CallIndirect,
            0x28 => Self::I32Load,
            0x29 => Self::I64Load,
            0x2A => Self::F32Load,
            0x2B => Self::F64Load,
            0x36 => Self::I32Store,
            0x37 => Self::I64Store,
            0x38 => Self::F32Store,
            0x39 => Self::F64Store,
            0x3F => Self::MemorySize,
            0x40 => Self::MemoryGrow,
            0x41 => Self::I32Const,
            0x42 => Self::I64Const,
            0x43 => Self::F32Const,
            0x44 => Self::F64Const,
            0x20 => Self::LocalGet,
            0x21 => Self::LocalSet,
            0x22 => Self::LocalTee,
            0x23 => Self::GlobalGet,
            0x24 => Self::GlobalSet,
            other => Self::Other(other),
        }
    }
}
