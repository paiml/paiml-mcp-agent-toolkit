// correlation_engine_lookups.rs — Bidirectional WASM address/source location lookups
// Included by correlation_engine.rs — no `use` imports or inner attributes allowed

impl CorrelationEngine {
    /// Look up source location for a given WASM address
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn lookup_source_location(
        &self,
        wasm_address: u64,
        line_program_mappings: &[(u64, Location)],
    ) -> Option<Location> {
        line_program_mappings
            .iter()
            .find(|(addr, _)| *addr == wasm_address)
            .map(|(_, loc)| loc.clone())
    }

    /// Look up WASM addresses for a given source line
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn lookup_wasm_addresses(
        &self,
        source_line: u32,
        line_program_mappings: &[(u64, Location)],
    ) -> DeepWasmResult<Vec<u64>> {
        let addresses: Vec<u64> = line_program_mappings
            .iter()
            .filter(|(_, loc)| loc.line == source_line)
            .map(|(addr, _)| *addr)
            .collect();

        Ok(addresses)
    }
}
