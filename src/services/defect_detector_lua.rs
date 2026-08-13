impl LuaDefectDetector {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            global_assign_re: Regex::new(r"^([a-zA-Z_]\w*)\s*=").expect("internal error"),
            nil_chain_re: Regex::new(r"\)\s*[.:]\w+").expect("internal error"),
            unchecked_pcall_re: Regex::new(r"^\s*x?pcall\s*\(").expect("internal error"),
            dangerous_api_re: Regex::new(
                r"\b(?:os\.execute|io\.popen|loadstring|setfenv|getfenv|debug\.setlocal)\s*\(",
            )
            .expect("internal error"),
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    /// Detect.
    pub fn detect(&self, content: &str, file_path: &Path) -> Vec<DefectPattern> {
        let mut defects = Vec::new();
        if self.should_exclude_file(file_path) {
            return defects;
        }

        self.detect_implicit_globals(content, file_path, &mut defects);
        self.detect_nil_unsafe(content, file_path, &mut defects);
        self.detect_unchecked_pcall(content, file_path, &mut defects);
        self.detect_dangerous_apis(content, file_path, &mut defects);

        defects
    }

    /// #926: this used to be a private copy of the exclusion rule that matched
    /// `"/tests/"`, `"/test/"` and `"/spec/"` as substrings of the **absolute**
    /// path — the #923 defect, re-committed in a second language. It deferred
    /// to nothing, so a checkout under any directory called `tests/` reported
    /// its whole Lua tree as clean. There is one rule; it lives in
    /// [`support_scope::is_support_file`].
    fn should_exclude_file(&self, file_path: &Path) -> bool {
        support_scope::is_support_file(file_path)
    }

    fn detect_implicit_globals(
        &self,
        content: &str,
        file_path: &Path,
        defects: &mut Vec<DefectPattern>,
    ) {
        let lua_keywords = [
            "if", "then", "else", "elseif", "end", "do", "while", "repeat", "until", "for", "in",
            "function", "return", "break", "goto", "not", "and", "or",
        ];
        let mut instances = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("local ") {
                continue;
            }
            let Some(caps) = self.global_assign_re.captures(trimmed) else {
                continue;
            };
            let name = caps.get(1).map_or("", |m| m.as_str());
            if lua_keywords.contains(&name) || name.starts_with('_') {
                continue;
            }
            instances.push(DefectInstance {
                file: file_path.to_string_lossy().to_string(),
                line: line_num + 1,
                column: 1,
                code_snippet: trimmed.to_string(),
            });
        }
        if !instances.is_empty() {
            let severity = if instances.len() > 10 {
                Severity::Critical
            } else {
                Severity::High
            };
            defects.push(DefectPattern {
                id: "LUA-GLOBAL-001".to_string(),
                name: "Implicit global assignment".to_string(),
                severity,
                fix_recommendation: "Add `local` keyword to variable declarations".to_string(),
                bad_example: "count = 0".to_string(),
                good_example: "local count = 0".to_string(),
                evidence_description:
                    "Global namespace pollution is Lua's #1 defect source (Maidl et al. 2014)"
                        .to_string(),
                evidence_url: None,
                instances,
            });
        }
    }

    fn detect_nil_unsafe(&self, content: &str, file_path: &Path, defects: &mut Vec<DefectPattern>) {
        let mut instances = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            if self.nil_chain_re.is_match(trimmed) {
                instances.push(DefectInstance {
                    file: file_path.to_string_lossy().to_string(),
                    line: line_num + 1,
                    column: 1,
                    code_snippet: trimmed.to_string(),
                });
            }
        }
        if !instances.is_empty() {
            defects.push(DefectPattern {
                id: "LUA-NIL-001".to_string(),
                name: "Nil-unsafe chained access".to_string(),
                severity: Severity::High,
                fix_recommendation:
                    "Store function return in local variable and nil-check before accessing"
                        .to_string(),
                bad_example: "get_player():set_health(100)".to_string(),
                good_example: "local p = get_player()\nif p then p:set_health(100) end".to_string(),
                evidence_description:
                    "Chained access on nil return causes runtime crash (LuaTaint analysis)"
                        .to_string(),
                evidence_url: None,
                instances,
            });
        }
    }

    fn detect_unchecked_pcall(
        &self,
        content: &str,
        file_path: &Path,
        defects: &mut Vec<DefectPattern>,
    ) {
        let mut instances = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            // Uncaptured: pcall( without assignment
            if self.unchecked_pcall_re.is_match(trimmed) && !trimmed.contains('=') {
                instances.push(DefectInstance {
                    file: file_path.to_string_lossy().to_string(),
                    line: line_num + 1,
                    column: 1,
                    code_snippet: trimmed.to_string(),
                });
            }
        }
        if !instances.is_empty() {
            defects.push(DefectPattern {
                id: "LUA-PCALL-001".to_string(),
                name: "Unchecked pcall/xpcall return".to_string(),
                severity: Severity::High,
                fix_recommendation: "Capture and check pcall return: local ok, err = pcall(fn); if not ok then ... end".to_string(),
                bad_example: "pcall(dangerous_function)".to_string(),
                good_example: "local ok, err = pcall(dangerous_function)\nif not ok then error(err) end".to_string(),
                evidence_description: "Swallowed errors hide crashes (FLuaScan, Zhang et al. 2020)".to_string(),
                evidence_url: None,
                instances,
            });
        }
    }

    fn detect_dangerous_apis(
        &self,
        content: &str,
        file_path: &Path,
        defects: &mut Vec<DefectPattern>,
    ) {
        let mut instances = Vec::new();
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("--") {
                continue;
            }
            if self.dangerous_api_re.is_match(trimmed) {
                instances.push(DefectInstance {
                    file: file_path.to_string_lossy().to_string(),
                    line: line_num + 1,
                    column: 1,
                    code_snippet: trimmed.to_string(),
                });
            }
        }
        if !instances.is_empty() {
            defects.push(DefectPattern {
                id: "LUA-DANGER-001".to_string(),
                name: "Dangerous/deprecated API usage".to_string(),
                severity: Severity::High,
                fix_recommendation: "Avoid os.execute/io.popen with user input; use structured APIs instead of loadstring".to_string(),
                bad_example: "os.execute('rm -rf ' .. user_input)".to_string(),
                good_example: "os.execute('make clean')  -- hardcoded commands only".to_string(),
                evidence_description: "Command injection via string concatenation in shell APIs (LuaTaint)".to_string(),
                evidence_url: None,
                instances,
            });
        }
    }
}

impl Default for LuaDefectDetector {
    fn default() -> Self {
        Self::new()
    }
}
