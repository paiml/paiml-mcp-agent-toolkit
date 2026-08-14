// Python rule set for the Known-Defects database.
//
// Named by docs/specifications/components/language-support.md ("Known Defects
// Database ... Python: bare `except:`, mutable default args, `eval()`") and
// never implemented, so `analyze defects` walked past every `.py` file in a
// project and reported it clean.

impl PythonDefectDetector {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            // `eval(`/`exec(` whose first argument is not a quoted literal.
            // `eval("2 + 2")` is a constant; `eval(request.body)` is CWE-95.
            dynamic_eval_re: Regex::new(r"\b(?:eval|exec)\s*\(\s*[^'\x22\s)]")
                .expect("internal error"),
            // A default argument that is a fresh mutable container. Evaluated
            // once, at def time, and then shared by every call.
            mutable_default_re: Regex::new(r"=\s*(?:\[\]|\{\}|set\(\)|list\(\)|dict\(\))")
                .expect("internal error"),
        }
    }

    /// Detect Python defects in `content`.
    pub fn detect(&self, content: &str, file_path: &Path) -> Vec<DefectPattern> {
        if support_scope::is_support_file(file_path) {
            return Vec::new();
        }

        [
            self.bare_except(content, file_path),
            self.mutable_defaults(content, file_path),
            self.dynamic_eval(content, file_path),
            self.assert_as_validation(content, file_path),
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    /// Lines of `content` that are not comments, with their 0-based index.
    fn code_lines(content: &str) -> impl Iterator<Item = (usize, &str)> {
        content
            .lines()
            .enumerate()
            .map(|(index, line)| (index, line.trim()))
            .filter(|(_, trimmed)| !trimmed.is_empty() && !trimmed.starts_with('#'))
    }

    /// `except:` with no exception type — swallows `SystemExit` and
    /// `KeyboardInterrupt` along with the error the author meant to handle.
    fn bare_except(&self, content: &str, file_path: &Path) -> Option<DefectPattern> {
        let instances = Self::code_lines(content)
            .filter(|(_, trimmed)| {
                trimmed
                    .strip_prefix("except")
                    .is_some_and(|rest| rest.trim() == ":")
            })
            .map(|(index, trimmed)| instance_at(file_path, index, trimmed))
            .collect();

        pattern_from(
            "PY-EXCEPT-001",
            "Bare except clause",
            Severity::High,
            "Catch the exception you can handle (`except ValueError:`), or `except Exception:` \
             if you truly mean every error but Ctrl-C and process exit",
            "try:\n    load()\nexcept:\n    pass",
            "try:\n    load()\nexcept OSError as err:\n    log.warning(\"load failed: %s\", err)",
            "CWE-396 Declaration of Catch-All Exception Handler; a bare except also catches \
             KeyboardInterrupt and SystemExit, so the process stops responding to Ctrl-C",
            instances,
        )
    }

    /// A mutable default argument is created once and shared across calls, so
    /// state leaks between unrelated callers.
    fn mutable_defaults(&self, content: &str, file_path: &Path) -> Option<DefectPattern> {
        let instances = Self::code_lines(content)
            .filter(|(_, trimmed)| {
                (trimmed.starts_with("def ") || trimmed.starts_with("async def "))
                    && self.mutable_default_re.is_match(trimmed)
            })
            .map(|(index, trimmed)| instance_at(file_path, index, trimmed))
            .collect();

        pattern_from(
            "PY-MUTDEF-001",
            "Mutable default argument",
            Severity::High,
            "Default to None and build the container inside the function body",
            "def append(item, into=[]):\n    into.append(item)",
            "def append(item, into=None):\n    into = [] if into is None else into\n    into.append(item)",
            "The default object is evaluated once at definition time and shared by every call \
             (CPython reference: Function definitions, 'Default parameter values')",
            instances,
        )
    }

    /// `eval`/`exec` on anything but a literal is arbitrary code execution.
    fn dynamic_eval(&self, content: &str, file_path: &Path) -> Option<DefectPattern> {
        let instances = Self::code_lines(content)
            .filter(|(_, trimmed)| self.dynamic_eval_re.is_match(trimmed))
            .map(|(index, trimmed)| instance_at(file_path, index, trimmed))
            .collect();

        pattern_from(
            "PY-EVAL-001",
            "Dynamic eval/exec of non-literal input",
            Severity::Critical,
            "Parse the value instead: ast.literal_eval for data, json.loads for JSON, an \
             explicit dispatch table for behaviour",
            "value = eval(request.args['expr'])",
            "value = ast.literal_eval(request.args['expr'])",
            "CWE-95 Eval Injection: the argument is executed with the caller's privileges",
            instances,
        )
    }

    /// `assert` is removed by `python -O`, so an assert that validates input
    /// is a check that vanishes in the configuration production uses.
    fn assert_as_validation(&self, content: &str, file_path: &Path) -> Option<DefectPattern> {
        let instances = Self::code_lines(content)
            .filter(|(_, trimmed)| trimmed.starts_with("assert "))
            .map(|(index, trimmed)| instance_at(file_path, index, trimmed))
            .collect();

        pattern_from(
            "PY-ASSERT-001",
            "assert used for runtime validation",
            Severity::Medium,
            "Raise an explicit exception; keep assert for invariants you are willing to lose",
            "assert user.is_admin, \"forbidden\"",
            "if not user.is_admin:\n    raise PermissionError(\"forbidden\")",
            "CWE-617: `python -O` strips every assert statement, so the check does not run in \
             an optimised deployment",
            instances,
        )
    }
}

impl Default for PythonDefectDetector {
    fn default() -> Self {
        Self::new()
    }
}
