// Operator matching logic for WASM security pattern detection.
// Included from security.rs -- shares parent module scope (no `use` imports here).

impl OperatorMatcher {
    /// Check if operator matches pattern
    fn matches(&self, op: &Operator) -> bool {
        use Operator::{
            Br, BrIf, Call, CallIndirect, I32Add, I32And, I32Const, I32DivS, I32DivU, I32Eq,
            I32Eqz, I32GtS, I32GtU, I32Load, I32LtS, I32LtU, I32Mul, I32Ne, I32Or, I32RemU,
            I32Store, I32Sub, I32Xor, I64Load, I64Store, MemoryGrow, MemorySize,
        };
        use OperatorMatcher as M;

        #[allow(clippy::match_like_matches_macro)]
        match (self, op) {
            (M::I32Const, I32Const { .. }) => true,
            (M::I32Add, I32Add) => true,
            (M::I32Sub, I32Sub) => true,
            (M::I32Mul, I32Mul) => true,
            (M::I32DivS, I32DivS) => true,
            (M::I32DivU, I32DivU) => true,
            (M::I32RemU, I32RemU) => true,
            (M::I32And, I32And) => true,
            (M::I32Or, I32Or) => true,
            (M::I32Xor, I32Xor) => true,
            (M::I32Eqz, I32Eqz) => true,
            (M::I32Eq, I32Eq) => true,
            (M::I32Ne, I32Ne) => true,
            (M::I32LtS, I32LtS) => true,
            (M::I32LtU, I32LtU) => true,
            (M::I32GtS, I32GtS) => true,
            (M::I32GtU, I32GtU) => true,
            (M::I32Load, I32Load { .. }) => true,
            (M::I32Store, I32Store { .. }) => true,
            (M::I64Load, I64Load { .. }) => true,
            (M::I64Store, I64Store { .. }) => true,
            (M::BrIf, BrIf { .. }) => true,
            (M::Br, Br { .. }) => true,
            (M::Call, Call { .. }) => true,
            (M::CallIndirect, CallIndirect { .. }) => true,
            (M::MemoryGrow, MemoryGrow { .. }) => true,
            (M::MemorySize, MemorySize { .. }) => true,
            (M::Any, _) => true,
            _ => false,
        }
    }
}
