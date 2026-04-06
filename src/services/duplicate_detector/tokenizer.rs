#![cfg_attr(coverage_nightly, coverage(off))]
//! Universal feature extractor with cross-language tokenization and normalization.

use dashmap::DashMap;

use super::keywords::{C_CPP_KEYWORDS, KOTLIN_KEYWORDS, PYTHON_KEYWORDS, TYPESCRIPT_KEYWORDS};
use super::types::{DuplicateDetectionConfig, Language, Token, TokenKind};

/// Universal feature extractor for cross-language analysis
pub struct UniversalFeatureExtractor {
    pub(super) config: DuplicateDetectionConfig,
    identifier_counter: std::sync::atomic::AtomicU32,
    identifier_map: DashMap<String, String>,
}

impl UniversalFeatureExtractor {
    #[must_use]
    pub fn new(config: DuplicateDetectionConfig) -> Self {
        Self {
            config,
            identifier_counter: std::sync::atomic::AtomicU32::new(0),
            identifier_map: DashMap::new(),
        }
    }

    /// Extract features from source code
    pub fn extract_features(&self, source: &str, lang: Language) -> Vec<Token> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        let tokens = self.tokenize(source, lang);
        self.normalize_tokens(&tokens)
    }

    /// Tokenize source code based on language
    fn tokenize(&self, source: &str, lang: Language) -> Vec<Token> {
        debug_assert!(!source.is_empty(), "source must not be empty");
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
        debug_assert!(!source.is_empty(), "source must not be empty");
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
        debug_assert!(!source.is_empty(), "source must not be empty");
        // Simplified tokenizer - in production would use swc_ecma_parser
        self.tokenize_generic(source, TYPESCRIPT_KEYWORDS)
    }

    /// Basic Python tokenizer
    fn tokenize_python(&self, source: &str) -> Vec<Token> {
        debug_assert!(!source.is_empty(), "source must not be empty");
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
        debug_assert!(!source.is_empty(), "source must not be empty");
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
        debug_assert!(!source.is_empty(), "source must not be empty");
        // Simplified tokenizer for C/C++ - in production would use tree-sitter-c/cpp
        self.tokenize_generic(source, C_CPP_KEYWORDS)
    }

    /// Tokenize Kotlin source code
    fn tokenize_kotlin(&self, source: &str) -> Vec<Token> {
        debug_assert!(!source.is_empty(), "source must not be empty");
        // Simplified tokenizer for Kotlin - in production would use tree-sitter-kotlin
        self.tokenize_generic(source, KOTLIN_KEYWORDS)
    }

    /// Check if string is a Rust keyword
    fn is_rust_keyword(&self, s: &str) -> bool {
        debug_assert!(!s.is_empty(), "s must not be empty");
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
        debug_assert!(!s.is_empty(), "s must not be empty");
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

    /// Normalize tokens for Type-2 clone detection
    pub(super) fn normalize_tokens(&self, tokens: &[Token]) -> Vec<Token> {
        tokens
            .iter()
            .filter_map(|token| match &token.kind {
                TokenKind::Whitespace | TokenKind::Comment if self.config.ignore_comments => None,
                TokenKind::Identifier(name) if self.config.normalize_identifiers => Some(
                    Token::new(TokenKind::Identifier(self.canonicalize_identifier(name))),
                ),
                TokenKind::Literal(_) if self.config.normalize_literals => {
                    Some(Token::new(TokenKind::Literal("LITERAL".to_string())))
                }
                _ => Some(token.clone()),
            })
            .collect()
    }

    /// Canonicalize identifier names
    pub(crate) fn canonicalize_identifier(&self, name: &str) -> String {
        debug_assert!(!name.is_empty(), "name must not be empty");
        if let Some(canonical) = self.identifier_map.get(name) {
            canonical.clone()
        } else {
            let id = self
                .identifier_counter
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let canonical = format!("VAR_{id}");
            self.identifier_map
                .insert(name.to_string(), canonical.clone());
            canonical
        }
    }
}
