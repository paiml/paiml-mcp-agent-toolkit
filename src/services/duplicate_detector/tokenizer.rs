#![cfg_attr(coverage_nightly, coverage(off))]
//! Universal feature extractor with cross-language tokenization and normalization.

use super::keywords::{C_CPP_KEYWORDS, KOTLIN_KEYWORDS, PYTHON_KEYWORDS, TYPESCRIPT_KEYWORDS};
use super::types::{DuplicateDetectionConfig, Language, Token, TokenKind};

/// Universal feature extractor for cross-language analysis
pub struct UniversalFeatureExtractor {
    pub(super) config: DuplicateDetectionConfig,
}

impl UniversalFeatureExtractor {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new(config: DuplicateDetectionConfig) -> Self {
        Self { config }
    }

    /// Extract features from source code
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn extract_features(&self, source: &str, lang: Language) -> Vec<Token> {
        let tokens = self.tokenize(source, lang);
        self.normalize_tokens(&tokens)
    }

    /// Tokenize source code based on language
    fn tokenize(&self, source: &str, lang: Language) -> Vec<Token> {
        match lang {
            Language::Rust => self.tokenize_rust(source),
            Language::TypeScript | Language::JavaScript => self.tokenize_typescript(source),
            Language::Python => self.tokenize_python(source),
            Language::C | Language::Cpp => self.tokenize_c_style(source),
            Language::Kotlin => self.tokenize_kotlin(source),
        }
    }

    fn handle_whitespace(&self, tokens: &mut Vec<Token>) {
        if !self.config.ignore_comments {
            tokens.push(Token::new(TokenKind::Whitespace));
        }
    }

    fn handle_comment(
        &self,
        chars: &mut std::iter::Peekable<std::str::CharIndices>,
        tokens: &mut Vec<Token>,
    ) {
        if !self.config.ignore_comments {
            while let Some((_, ch)) = chars.peek() {
                if *ch == '\n' {
                    break;
                }
                chars.next();
            }
            tokens.push(Token::new(TokenKind::Comment));
        }
    }

    fn handle_string_literal(
        &self,
        ch: char,
        chars: &mut std::iter::Peekable<std::str::CharIndices>,
        tokens: &mut Vec<Token>,
    ) {
        let mut literal = String::new();
        literal.push(ch);
        while let Some((_, ch)) = chars.next() {
            literal.push(ch);
            if ch == '"' {
                break;
            }
            if ch == '\\' {
                if let Some((_, escaped)) = chars.next() {
                    literal.push(escaped);
                }
            }
        }
        tokens.push(Token::new(TokenKind::Literal(literal)));
    }

    fn handle_number(
        &self,
        ch: char,
        chars: &mut std::iter::Peekable<std::str::CharIndices>,
        tokens: &mut Vec<Token>,
    ) {
        let mut number = String::new();
        number.push(ch);
        while let Some((_, ch)) = chars.peek() {
            if ch.is_ascii_alphanumeric() || *ch == '.' || *ch == '_' {
                number.push(*ch);
                chars.next();
            } else {
                break;
            }
        }
        tokens.push(Token::new(TokenKind::Literal(number)));
    }

    fn handle_identifier(
        &self,
        ch: char,
        chars: &mut std::iter::Peekable<std::str::CharIndices>,
        tokens: &mut Vec<Token>,
    ) {
        let mut ident = String::new();
        ident.push(ch);
        while let Some((_, ch)) = chars.peek() {
            if ch.is_ascii_alphanumeric() || *ch == '_' {
                ident.push(*ch);
                chars.next();
            } else {
                break;
            }
        }

        let token = if self.is_rust_keyword(&ident) {
            Token::new(TokenKind::Keyword(ident))
        } else {
            Token::new(TokenKind::Identifier(ident))
        };
        tokens.push(token);
    }

    fn handle_operator(
        &self,
        ch: char,
        chars: &mut std::iter::Peekable<std::str::CharIndices>,
        tokens: &mut Vec<Token>,
    ) {
        let mut op = String::new();
        op.push(ch);

        if let Some((_, next_ch)) = chars.peek() {
            let two_char = format!("{ch}{next_ch}");
            if self.is_rust_operator(&two_char) {
                op.push(*next_ch);
                chars.next();
            }
        }

        if self.is_rust_operator(&op) {
            tokens.push(Token::new(TokenKind::Operator(op)));
        } else if self.is_delimiter(ch) {
            tokens.push(Token::new(TokenKind::Delimiter(op)));
        }
    }

    fn tokenize_rust(&self, source: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = source.char_indices().peekable();

        while let Some((_, ch)) = chars.next() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => self.handle_whitespace(&mut tokens),
                '/' if chars.peek().map(|(_, c)| *c) == Some('/') => {
                    self.handle_comment(&mut chars, &mut tokens);
                }
                '"' => self.handle_string_literal(ch, &mut chars, &mut tokens),
                ch if ch.is_ascii_digit() => self.handle_number(ch, &mut chars, &mut tokens),
                ch if ch.is_ascii_alphabetic() || ch == '_' => {
                    self.handle_identifier(ch, &mut chars, &mut tokens);
                }
                _ => self.handle_operator(ch, &mut chars, &mut tokens),
            }
        }

        tokens
    }

    /// Basic TypeScript/JavaScript tokenizer
    fn tokenize_typescript(&self, source: &str) -> Vec<Token> {
        // Simplified tokenizer - in production would use swc_ecma_parser
        self.tokenize_generic(source, TYPESCRIPT_KEYWORDS)
    }

    /// Basic Python tokenizer
    fn tokenize_python(&self, source: &str) -> Vec<Token> {
        // Simplified tokenizer - in production would use rustpython_parser
        self.tokenize_generic(source, PYTHON_KEYWORDS)
    }

    /// Classify a character into a token
    fn classify_char(
        ch: char,
        chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
        keywords: &[&str],
        ignore_comments: bool,
    ) -> Option<Token> {
        match ch {
            ' ' | '\t' | '\n' | '\r' if !ignore_comments => Some(Token::new(TokenKind::Whitespace)),
            ' ' | '\t' | '\n' | '\r' => None,
            ch if ch.is_ascii_alphabetic() || ch == '_' => {
                let ident = Self::consume_identifier(ch, chars);
                let kind = if keywords.contains(&ident.as_str()) {
                    TokenKind::Keyword(ident)
                } else {
                    TokenKind::Identifier(ident)
                };
                Some(Token::new(kind))
            }
            ch if ch.is_ascii_digit() => Some(Token::new(TokenKind::Literal(
                Self::consume_number(ch, chars),
            ))),
            _ => Some(Token::new(TokenKind::Operator(ch.to_string()))),
        }
    }

    /// Generic tokenizer for any language
    fn tokenize_generic(&self, source: &str, keywords: &[&str]) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = source.char_indices().peekable();
        while let Some((_, ch)) = chars.next() {
            if let Some(token) =
                Self::classify_char(ch, &mut chars, keywords, self.config.ignore_comments)
            {
                tokens.push(token);
            }
        }
        tokens
    }

    /// Consume an identifier starting with the given character
    fn consume_identifier(
        first: char,
        chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    ) -> String {
        let mut ident = String::new();
        ident.push(first);
        while let Some((_, ch)) = chars.peek() {
            if ch.is_ascii_alphanumeric() || *ch == '_' {
                ident.push(*ch);
                chars.next();
            } else {
                break;
            }
        }
        ident
    }

    /// Consume a number literal starting with the given digit
    fn consume_number(
        first: char,
        chars: &mut std::iter::Peekable<std::str::CharIndices<'_>>,
    ) -> String {
        let mut number = String::new();
        number.push(first);
        while let Some((_, ch)) = chars.peek() {
            if ch.is_ascii_alphanumeric() || *ch == '.' {
                number.push(*ch);
                chars.next();
            } else {
                break;
            }
        }
        number
    }

    /// Tokenize C/C++ source code
    fn tokenize_c_style(&self, source: &str) -> Vec<Token> {
        // Simplified tokenizer for C/C++ - in production would use tree-sitter-c/cpp
        self.tokenize_generic(source, C_CPP_KEYWORDS)
    }

    /// Tokenize Kotlin source code
    fn tokenize_kotlin(&self, source: &str) -> Vec<Token> {
        // Simplified tokenizer for Kotlin - in production would use tree-sitter-kotlin
        self.tokenize_generic(source, KOTLIN_KEYWORDS)
    }

    /// Check if string is a Rust keyword
    fn is_rust_keyword(&self, s: &str) -> bool {
        matches!(
            s,
            "fn" | "let"
                | "mut"
                | "if"
                | "else"
                | "match"
                | "for"
                | "while"
                | "loop"
                | "return"
                | "break"
                | "continue"
                | "struct"
                | "enum"
                | "impl"
                | "trait"
                | "mod"
                | "use"
                | "pub"
                | "crate"
                | "super"
                | "self"
                | "Self"
                | "where"
                | "async"
                | "await"
                | "const"
                | "static"
                | "extern"
                | "unsafe"
        )
    }

    /// Check if string is a Rust operator
    fn is_rust_operator(&self, s: &str) -> bool {
        matches!(
            s,
            "+" | "-"
                | "*"
                | "/"
                | "%"
                | "="
                | "=="
                | "!="
                | "<"
                | ">"
                | "<="
                | ">="
                | "&&"
                | "||"
                | "!"
                | "&"
                | "|"
                | "^"
                | "<<"
                | ">>"
                | "+="
                | "-="
                | "*="
                | "/="
                | "%="
                | "&="
                | "|="
                | "^="
                | "<<="
                | ">>="
                | "?"
                | "::"
                | "->"
                | "=>"
                | ".."
                | "..="
                | "@"
        )
    }

    /// Check if character is a delimiter
    fn is_delimiter(&self, ch: char) -> bool {
        matches!(ch, '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | '.')
    }

    /// Normalize tokens for Type-2 clone detection.
    ///
    /// RENAME-INVARIANCE: the canonical names are numbered per CALL, in order of
    /// first appearance, so two fragments that differ only in their identifiers
    /// normalise to the same token stream — which is the definition of a Type-2
    /// clone and the whole purpose of this pass.
    ///
    /// They used to be numbered from a counter and map that lived on the
    /// extractor for its whole lifetime, i.e. across every file of the project.
    /// The first file's `total` became `VAR_2` and the second file's `sum`
    /// became `VAR_9`, so a renamed copy of a function hashed differently and
    /// its MinHash signature barely overlapped: two 14-line functions differing
    /// only in variable names and one extra statement measured a Jaccard
    /// similarity of 0.10 and no clone group was formed. The engine could
    /// therefore only ever find clones that were identical MODULO LITERALS —
    /// exactly what the cheap hash pass in `analyze duplicates` already found —
    /// while advertising Type-2/Type-3 detection.
    pub(super) fn normalize_tokens(&self, tokens: &[Token]) -> Vec<Token> {
        let mut scope = IdentifierScope::default();
        tokens
            .iter()
            .filter_map(|token| match &token.kind {
                TokenKind::Whitespace | TokenKind::Comment if self.config.ignore_comments => None,
                TokenKind::Identifier(name) if self.config.normalize_identifiers => {
                    Some(Token::new(TokenKind::Identifier(scope.canonicalize(name))))
                }
                TokenKind::Literal(_) if self.config.normalize_literals => {
                    Some(Token::new(TokenKind::Literal("LITERAL".to_string())))
                }
                _ => Some(token.clone()),
            })
            .collect()
    }
}

/// Identifier numbering for ONE normalisation pass.
///
/// Scoped deliberately: see `normalize_tokens`. A scope shared between
/// fragments makes the numbering depend on which other code was tokenized
/// first, which is precisely what breaks rename-invariance.
#[derive(Debug, Default)]
pub struct IdentifierScope {
    names: std::collections::HashMap<String, String>,
}

impl IdentifierScope {
    /// Canonical name for `name` within this scope: `VAR_0` for the first
    /// distinct identifier seen, `VAR_1` for the second, and the same name again
    /// for every later occurrence of the same identifier.
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn canonicalize(&mut self, name: &str) -> String {
        if let Some(canonical) = self.names.get(name) {
            return canonical.clone();
        }
        let canonical = format!("VAR_{}", self.names.len());
        self.names.insert(name.to_string(), canonical.clone());
        canonical
    }
}
