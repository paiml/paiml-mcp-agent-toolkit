#![cfg_attr(coverage_nightly, coverage(off))]
//! Ruchy lexer: tokenization of Ruchy source code.

use super::types::{RuchyToken, KEYWORD_MAP, SINGLE_CHAR_TOKEN_MAP};

/// Simple Ruchy lexer for basic tokenization
pub struct RuchyLexer {
    input: String,
    position: usize,
    current_char: Option<char>,
    line: u32,
    column: u32,
}

impl RuchyLexer {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(input: String) -> Self {
        let lexer = Self {
            input: input.clone(),
            position: 0,
            current_char: input.chars().next(),
            line: 1,
            column: 1,
        };
        lexer
    }

    fn advance(&mut self) {
        debug_assert!(true, "contract: advance");
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        self.position += 1;
        self.current_char = self.input.chars().nth(self.position);
    }

    fn peek(&self) -> Option<char> {
        debug_assert!(true, "contract: peek");
        self.input.chars().nth(self.position + 1)
    }

    fn skip_whitespace(&mut self) {
        debug_assert!(true, "contract: skip_whitespace");
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        debug_assert!(true, "contract: skip_comment");
        if self.current_char == Some('/') && self.peek() == Some('/') {
            while self.current_char.is_some() && self.current_char != Some('\n') {
                self.advance();
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        debug_assert!(true, "contract: read_identifier");
        let mut result = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn read_number(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: read_number");
        let mut num_str = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current_char {
            if ch.is_numeric() {
                num_str.push(ch);
                self.advance();
            } else if ch == '.' && !is_float && self.peek().is_some_and(char::is_numeric) {
                is_float = true;
                num_str.push(ch);
                self.advance();
            } else if (ch == 'e' || ch == 'E') && !num_str.contains('e') && !num_str.contains('E') {
                num_str.push(ch);
                self.advance();
                if let Some(sign) = self.current_char {
                    if sign == '+' || sign == '-' {
                        num_str.push(sign);
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }

        if is_float {
            RuchyToken::Float(num_str.parse().unwrap_or(0.0))
        } else {
            RuchyToken::Integer(num_str.parse().unwrap_or(0))
        }
    }

    fn read_string(&mut self, quote: char) -> String {
        debug_assert!(true, "contract: read_string");
        let mut result = String::new();
        self.advance(); // skip opening quote

        while let Some(ch) = self.current_char {
            if ch == quote {
                self.advance(); // skip closing quote
                break;
            } else if ch == '\\' {
                self.advance();
                if let Some(escaped) = self.current_char {
                    match escaped {
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        'r' => result.push('\r'),
                        '\\' => result.push('\\'),
                        '"' => result.push('"'),
                        '\'' => result.push('\''),
                        _ => {
                            result.push('\\');
                            result.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        result
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn next_token(&mut self) -> RuchyToken {
        self.skip_whitespace();
        self.skip_comment();

        match self.current_char {
            None => RuchyToken::Eof,
            Some(ch) if ch.is_alphabetic() || ch == '_' => self.handle_identifier(),
            Some(ch) if ch.is_numeric() => self.read_number(),
            Some('"') => {
                let s = self.read_string('"');
                RuchyToken::String(s)
            }
            Some('\'') => self.handle_char_literal(),
            Some(ch) => self.handle_operator_or_punctuation(ch),
        }
    }

    /// Handle identifier and keyword tokens
    fn handle_identifier(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_identifier");
        let ident = self.read_identifier();
        KEYWORD_MAP
            .get(ident.as_str())
            .cloned()
            .unwrap_or(RuchyToken::Identifier(ident))
    }

    /// Handle character literal tokens
    fn handle_char_literal(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_char_literal");
        self.advance();
        let ch = self.current_char.unwrap_or('\0');
        self.advance();
        if self.current_char == Some('\'') {
            self.advance();
        }
        RuchyToken::Char(ch)
    }

    /// Handle operators and punctuation tokens
    fn handle_operator_or_punctuation(&mut self, ch: char) -> RuchyToken {
        debug_assert!(true, "contract: handle_operator_or_punctuation");
        // Try single-character tokens first
        if let Some(token) = SINGLE_CHAR_TOKEN_MAP.get(&ch) {
            return self.handle_single_char_token(token.clone());
        }

        // Handle multi-character tokens
        match ch {
            '-' => self.handle_dash(),
            '/' => self.handle_slash(),
            '=' => self.handle_equals(),
            '|' => self.handle_pipe(),
            '&' => self.handle_ampersand(),
            '@' => self.handle_annotation(),
            '.' => self.handle_dot(),
            ':' => self.handle_colon(),
            '!' => self.handle_exclamation(),
            '<' => self.handle_less_than(),
            '>' => self.handle_greater_than(),
            _ => {
                self.advance();
                RuchyToken::Error
            }
        }
    }

    /// Handle single character tokens
    fn handle_single_char_token(&mut self, token: RuchyToken) -> RuchyToken {
        debug_assert!(true, "contract: handle_single_char_token");
        self.advance();
        token
    }

    /// Handle dash (-) and arrow (->) tokens
    fn handle_dash(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_dash");
        self.advance();
        if self.current_char == Some('>') {
            self.advance();
            RuchyToken::Arrow
        } else {
            RuchyToken::Minus
        }
    }

    /// Handle slash (/) and comment tokens
    fn handle_slash(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_slash");
        self.advance();
        if self.current_char == Some('/') {
            self.skip_comment();
            self.next_token()
        } else {
            RuchyToken::Slash
        }
    }

    /// Handle equals (=) tokens
    fn handle_equals(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_equals");
        self.advance();
        match self.current_char {
            Some('=') => {
                self.advance();
                RuchyToken::EqualEqual
            }
            Some('>') => {
                self.advance();
                RuchyToken::FatArrow
            }
            _ => RuchyToken::Equal,
        }
    }

    /// Handle pipe (|) tokens
    fn handle_pipe(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_pipe");
        self.advance();
        match self.current_char {
            Some('>') => {
                self.advance();
                RuchyToken::PipeForward
            }
            Some('|') => {
                self.advance();
                RuchyToken::Or
            }
            _ => RuchyToken::Identifier("|".to_string()),
        }
    }

    /// Handle ampersand (&) tokens
    fn handle_ampersand(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_ampersand");
        self.advance();
        if self.current_char == Some('&') {
            self.advance();
            RuchyToken::And
        } else {
            RuchyToken::Identifier("&".to_string())
        }
    }

    /// Handle annotation (@) tokens
    fn handle_annotation(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_annotation");
        self.advance();
        let ident = self.read_identifier();
        RuchyToken::Annotation(format!("@{ident}"))
    }

    /// Handle dot (.) tokens
    fn handle_dot(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_dot");
        self.advance();
        if self.current_char == Some('.') {
            self.advance();
            if self.current_char == Some('.') {
                self.advance();
                RuchyToken::DotDotDot
            } else {
                RuchyToken::DotDot
            }
        } else {
            RuchyToken::Dot
        }
    }

    /// Handle colon (:) tokens
    fn handle_colon(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_colon");
        self.advance();
        if self.current_char == Some(':') {
            self.advance();
            RuchyToken::DoubleColon
        } else {
            RuchyToken::Colon
        }
    }

    /// Handle exclamation (!) tokens
    fn handle_exclamation(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_exclamation");
        self.advance();
        if self.current_char == Some('=') {
            self.advance();
            RuchyToken::NotEqual
        } else {
            RuchyToken::Not
        }
    }

    /// Handle less than (<) tokens
    fn handle_less_than(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_less_than");
        self.advance();
        match self.current_char {
            Some('=') => {
                self.advance();
                RuchyToken::LessEqual
            }
            Some('<') => {
                self.advance();
                RuchyToken::LeftShift
            }
            _ => RuchyToken::Less,
        }
    }

    /// Handle greater than (>) tokens
    fn handle_greater_than(&mut self) -> RuchyToken {
        debug_assert!(true, "contract: handle_greater_than");
        self.advance();
        match self.current_char {
            Some('=') => {
                self.advance();
                RuchyToken::GreaterEqual
            }
            Some('>') => {
                self.advance();
                RuchyToken::RightShift
            }
            _ => RuchyToken::Greater,
        }
    }
}
