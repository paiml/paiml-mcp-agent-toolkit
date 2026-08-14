// TypeScript/JavaScript rule set for the Known-Defects database.
//
// Named by docs/specifications/components/language-support.md ("Known Defects
// Database ... TypeScript: `any` type usage, missing null checks") and never
// implemented. It also supplies the only `Medium` and `Low` rules pmat has:
// before this, no rule set anywhere emitted either severity, so
// `analyze defects --severity medium` and `--severity low` were flags that
// parsed, printed a report, and could not match a finding in any codebase.

impl TypeScriptDefectDetector {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {
            // `: any`, `as any`, `<any>` — the escape hatch out of the type
            // system, wherever it is spelled.
            any_type_re: Regex::new(r"(?::\s*any\b|\bas\s+any\b|<\s*any\s*>)")
                .expect("internal error"),
            // A non-null assertion: `user!.name`, `cache!['k']`, `fn!()`.
            // `!=` cannot match because the character after `!` must be one of
            // `.`, `[`, `(`.
            non_null_assert_re: Regex::new(r"[\w\)\]]!\s*[.\[(]").expect("internal error"),
            // `==` / `!=` that is not `===` / `!==`.
            loose_equality_re: Regex::new(r"(?:^|[^=!<>])(==|!=)([^=]|$)").expect("internal error"),
        }
    }

    /// Detect TypeScript/JavaScript defects in `content`.
    pub fn detect(&self, content: &str, file_path: &Path) -> Vec<DefectPattern> {
        if support_scope::is_support_file(file_path) {
            return Vec::new();
        }

        [
            self.non_null_assertions(content, file_path),
            self.any_types(content, file_path),
            self.loose_equality(content, file_path),
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
            .filter(|(_, trimmed)| !trimmed.is_empty() && !support_scope::is_comment(trimmed))
    }

    /// `x!.y` tells the compiler to stop checking for null on exactly the
    /// expression the author was unsure about.
    fn non_null_assertions(&self, content: &str, file_path: &Path) -> Option<DefectPattern> {
        let instances = Self::code_lines(content)
            .filter(|(_, trimmed)| self.non_null_assert_re.is_match(trimmed))
            .map(|(index, trimmed)| instance_at(file_path, index, trimmed))
            .collect();

        pattern_from(
            "TS-NONNULL-001",
            "Non-null assertion suppresses a null check",
            Severity::High,
            "Narrow the value instead: optional chaining (`user?.name`), an early return, or a \
             type guard",
            "render(users.find(u => u.id === id)!.name)",
            "const user = users.find(u => u.id === id);\nif (!user) return;\nrender(user.name);",
            "`!` erases strictNullChecks for that expression only, so the failure surfaces as a \
             TypeError at runtime instead of a compile error (TypeScript handbook, \
             non-null assertion operator)",
            instances,
        )
    }

    /// `any` opts a value out of every subsequent check, and it spreads.
    fn any_types(&self, content: &str, file_path: &Path) -> Option<DefectPattern> {
        let instances = Self::code_lines(content)
            .filter(|(_, trimmed)| self.any_type_re.is_match(trimmed))
            .map(|(index, trimmed)| instance_at(file_path, index, trimmed))
            .collect();

        pattern_from(
            "TS-ANY-001",
            "Explicit `any` type",
            Severity::Medium,
            "Use `unknown` and narrow it, or write the real shape",
            "function handle(payload: any) { return payload.id; }",
            "function handle(payload: unknown) {\n  if (isRequest(payload)) return payload.id;\n}",
            "`any` is assignable both ways, so one annotation disables checking along every path \
             the value reaches (typescript-eslint `no-explicit-any`)",
            instances,
        )
    }

    /// `==` applies coercion: `0 == ''`, `'0' == false` and `[] == false` are
    /// all true. `== null` is exempt — it is the documented idiom for "null or
    /// undefined" and is what `eqeqeq: ["error", "always", {null: "ignore"}]`
    /// permits.
    fn loose_equality(&self, content: &str, file_path: &Path) -> Option<DefectPattern> {
        let instances = Self::code_lines(content)
            .filter(|(_, trimmed)| self.has_coercing_comparison(trimmed))
            .map(|(index, trimmed)| instance_at(file_path, index, trimmed))
            .collect();

        pattern_from(
            "TS-EQ-001",
            "Coercing equality comparison",
            Severity::Low,
            "Use `===` / `!==`; keep `== null` only when you mean \"null or undefined\"",
            "if (count == '0') { ... }",
            "if (count === 0) { ... }",
            "The abstract equality algorithm coerces operands, so `0 == ''`, `'0' == false` and \
             `[] == false` all hold (ECMA-262 IsLooselyEqual)",
            instances,
        )
    }

    /// True when the line has a `==`/`!=` whose right operand is not
    /// `null`/`undefined`.
    fn has_coercing_comparison(&self, trimmed: &str) -> bool {
        self.loose_equality_re
            .captures_iter(trimmed)
            .any(|caps| match caps.get(2) {
                Some(rest) => {
                    let tail = trimmed[rest.start()..].trim_start();
                    !tail.starts_with("null") && !tail.starts_with("undefined")
                }
                None => true,
            })
    }
}

impl Default for TypeScriptDefectDetector {
    fn default() -> Self {
        Self::new()
    }
}
