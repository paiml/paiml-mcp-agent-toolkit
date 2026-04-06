// WASM validation implementations - included from wasm.rs
// NO use imports or #! inner attributes allowed (shares parent module scope)

impl WasmValidator {
    /// Creates a new WASM validator
    #[must_use]
    pub fn new() -> Self {
        Self {
            validation_errors: Vec::new(),
            security_warnings: Vec::new(),
        }
    }

    /// Validates WASM module for correctness (complexity ≤10)
    pub fn validate_wasm_module(&mut self, wasm_bytes: &[u8]) -> Result<bool, String> {
        debug_assert!(!wasm_bytes.is_empty(), "wasm_bytes must not be empty");
        if wasm_bytes.len() < 8 {
            self.validation_errors.push("Module too short".to_string());
            return Ok(false);
        }

        if &wasm_bytes[0..4] != b"\0asm" {
            self.validation_errors
                .push("Invalid magic number".to_string());
            return Ok(false);
        }

        let parser = Parser::new(0);
        for payload in parser.parse_all(wasm_bytes) {
            if payload.is_err() {
                self.validation_errors
                    .push("Parse error in module".to_string());
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Performs security analysis on WASM module (complexity ≤10)
    pub fn analyze_security(&mut self, wasm_bytes: &[u8]) -> Result<Vec<String>, String> {
        debug_assert!(!wasm_bytes.is_empty(), "wasm_bytes must not be empty");
        let mut warnings = Vec::new();

        if wasm_bytes.len() > 1024 * 1024 {
            warnings.push("Large WASM module may consume excessive memory".to_string());
        }

        let parser = Parser::new(0);
        for payload in parser.parse_all(wasm_bytes) {
            if let Ok(Payload::ImportSection(_)) = payload {
                warnings.push("Module imports external functions".to_string());
            }
        }

        self.security_warnings = warnings.clone();
        Ok(warnings)
    }

    /// Gets validation errors
    #[must_use]
    pub fn get_validation_errors(&self) -> &[String] {
        &self.validation_errors
    }

    /// Gets security warnings
    #[must_use]
    pub fn get_security_warnings(&self) -> &[String] {
        &self.security_warnings
    }
}
