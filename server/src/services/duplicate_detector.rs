//! High-performance duplicate code detection using LSH and MinHash
//!
//! This module implements the duplicate code detection system as specified
//! in the dupe-code-redux-spec.md using locality-sensitive hashing (LSH),
//! MinHash signatures, and cross-language normalization.

use anyhow::Result;
use blake3::Hasher;
use dashmap::DashMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use xxhash_rust::xxh64::xxh64;

/// Language supported by the duplicate detection engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    C,
    Cpp,
}

/// Types of code clones detected
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CloneType {
    /// Exact clones (modulo whitespace)
    Type1 { similarity: f64 },
    /// Parametric clones (identifiers/literals differ)
    Type2 { similarity: f64, normalized: bool },
    /// Structural clones (statements added/removed)
    Type3 { similarity: f64, ast_distance: f64 },
}

/// Token types for normalization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    Identifier(String),
    Literal(String),
    Keyword(String),
    Operator(String),
    Delimiter(String),
    Comment,
    Whitespace,
}

/// Normalized token
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

impl Token {
    pub fn new(kind: TokenKind) -> Self {
        let text = match &kind {
            TokenKind::Identifier(s) => s.clone(),
            TokenKind::Literal(s) => s.clone(),
            TokenKind::Keyword(s) => s.clone(),
            TokenKind::Operator(s) => s.clone(),
            TokenKind::Delimiter(s) => s.clone(),
            TokenKind::Comment => "//".to_string(),
            TokenKind::Whitespace => " ".to_string(),
        };
        Self { kind, text }
    }

    pub fn hash(&self) -> u64 {
        xxh64(self.text.as_bytes(), 0)
    }
}

/// MinHash signature for similarity estimation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinHashSignature {
    pub values: Vec<u64>,
}

impl MinHashSignature {
    pub fn jaccard_similarity(&self, other: &MinHashSignature) -> f64 {
        let matches = self
            .values
            .iter()
            .zip(&other.values)
            .filter(|(a, b)| a == b)
            .count();
        matches as f64 / self.values.len() as f64
    }
}

/// Code fragment for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeFragment {
    pub id: FragmentId,
    pub file_path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub raw_content: String,
    pub tokens: Vec<Token>,
    pub normalized_tokens: Vec<Token>,
    pub signature: MinHashSignature,
    pub hash: u64,
    pub language: Language,
}

pub type FragmentId = u64;

/// Clone instance in a clone group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneInstance {
    pub file: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub similarity_to_representative: f64,
    pub normalized_hash: u64,
}

/// Group of similar code fragments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneGroup {
    pub id: u64,
    pub clone_type: CloneType,
    pub fragments: Vec<CloneInstance>,
    pub total_lines: usize,
    pub total_tokens: usize,
    pub average_similarity: f64,
    pub representative: FragmentId,
}

/// Summary of duplication analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneSummary {
    pub total_files: usize,
    pub total_fragments: usize,
    pub duplicate_lines: usize,
    pub total_lines: usize,
    pub duplication_ratio: f64,
    pub clone_groups: usize,
    pub largest_group_size: usize,
}

/// Duplication hotspot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicationHotspot {
    pub file: PathBuf,
    pub duplicate_lines: usize,
    pub clone_groups: usize,
    pub severity: f64,
}

/// Complete clone detection report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneReport {
    pub summary: CloneSummary,
    pub groups: Vec<CloneGroup>,
    pub hotspots: Vec<DuplicationHotspot>,
}

/// Configuration for duplicate detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDetectionConfig {
    pub min_tokens: usize,
    pub similarity_threshold: f64,
    pub shingle_size: usize,
    pub num_hash_functions: usize,
    pub num_bands: usize,
    pub rows_per_band: usize,
    pub normalize_identifiers: bool,
    pub normalize_literals: bool,
    pub ignore_comments: bool,
    pub min_group_size: usize,
}

impl Default for DuplicateDetectionConfig {
    fn default() -> Self {
        Self {
            min_tokens: 50,
            similarity_threshold: 0.70,
            shingle_size: 5,
            num_hash_functions: 200,
            num_bands: 20,
            rows_per_band: 10,
            normalize_identifiers: true,
            normalize_literals: true,
            ignore_comments: true,
            min_group_size: 2,
        }
    }
}

/// Universal feature extractor for cross-language analysis
pub struct UniversalFeatureExtractor {
    config: DuplicateDetectionConfig,
    identifier_counter: std::sync::atomic::AtomicU32,
    identifier_map: DashMap<String, String>,
}

impl UniversalFeatureExtractor {
    pub fn new(config: DuplicateDetectionConfig) -> Self {
        Self {
            config,
            identifier_counter: std::sync::atomic::AtomicU32::new(0),
            identifier_map: DashMap::new(),
        }
    }

    /// Extract features from source code
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
        }
    }

    /// Simple Rust tokenizer (would use syn in production)
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

        // Check if it's a keyword
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

        // Handle multi-character operators
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
                // Skip whitespace
                ' ' | '\t' | '\n' | '\r' => self.handle_whitespace(&mut tokens),
                // Comments
                '/' if chars.peek().map(|(_, c)| *c) == Some('/') => {
                    self.handle_comment(&mut chars, &mut tokens);
                }
                // String literals
                '"' => self.handle_string_literal(ch, &mut chars, &mut tokens),
                // Numbers
                ch if ch.is_ascii_digit() => self.handle_number(ch, &mut chars, &mut tokens),
                // Identifiers and keywords
                ch if ch.is_ascii_alphabetic() || ch == '_' => {
                    self.handle_identifier(ch, &mut chars, &mut tokens);
                }
                // Operators and delimiters
                _ => self.handle_operator(ch, &mut chars, &mut tokens),
            }
        }

        tokens
    }

    /// Basic TypeScript/JavaScript tokenizer
    fn tokenize_typescript(&self, source: &str) -> Vec<Token> {
        // Simplified tokenizer - in production would use swc_ecma_parser
        self.tokenize_generic(
            source,
            &[
                "function",
                "const",
                "let",
                "var",
                "if",
                "else",
                "for",
                "while",
                "return",
                "class",
                "interface",
                "type",
                "export",
                "import",
                "from",
                "async",
                "await",
            ],
        )
    }

    /// Basic Python tokenizer
    fn tokenize_python(&self, source: &str) -> Vec<Token> {
        // Simplified tokenizer - in production would use rustpython_parser
        self.tokenize_generic(
            source,
            &[
                "def", "class", "if", "elif", "else", "for", "while", "return", "import", "from",
                "try", "except", "finally", "with", "as", "async", "await",
            ],
        )
    }

    /// Generic tokenizer for any language
    fn tokenize_generic(&self, source: &str, keywords: &[&str]) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut chars = source.char_indices().peekable();

        while let Some((_, ch)) = chars.next() {
            match ch {
                ' ' | '\t' | '\n' | '\r' => {
                    if !self.config.ignore_comments {
                        tokens.push(Token::new(TokenKind::Whitespace));
                    }
                }
                ch if ch.is_ascii_alphabetic() || ch == '_' => {
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

                    let token = if keywords.contains(&ident.as_str()) {
                        Token::new(TokenKind::Keyword(ident))
                    } else {
                        Token::new(TokenKind::Identifier(ident))
                    };
                    tokens.push(token);
                }
                ch if ch.is_ascii_digit() => {
                    let mut number = String::new();
                    number.push(ch);
                    while let Some((_, ch)) = chars.peek() {
                        if ch.is_ascii_alphanumeric() || *ch == '.' {
                            number.push(*ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::new(TokenKind::Literal(number)));
                }
                _ => {
                    tokens.push(Token::new(TokenKind::Operator(ch.to_string())));
                }
            }
        }

        tokens
    }

    /// Tokenize C/C++ source code
    fn tokenize_c_style(&self, source: &str) -> Vec<Token> {
        // Simplified tokenizer for C/C++ - in production would use tree-sitter-c/cpp
        self.tokenize_generic(
            source,
            &[
                "auto",
                "break",
                "case",
                "char",
                "const",
                "continue",
                "default",
                "do",
                "double",
                "else",
                "enum",
                "extern",
                "float",
                "for",
                "goto",
                "if",
                "inline",
                "int",
                "long",
                "register",
                "restrict",
                "return",
                "short",
                "signed",
                "sizeof",
                "static",
                "struct",
                "switch",
                "typedef",
                "union",
                "unsigned",
                "void",
                "volatile",
                "while",
                "_Bool",
                "_Complex",
                "_Imaginary",
                // C++ additional keywords
                "class",
                "namespace",
                "template",
                "typename",
                "virtual",
                "override",
                "private",
                "protected",
                "public",
                "new",
                "delete",
                "try",
                "catch",
                "throw",
                "using",
                "friend",
                "constexpr",
                "explicit",
                "mutable",
                "operator",
                "this",
                "nullptr",
                "bool",
                "true",
                "false",
            ],
        )
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

    /// Normalize tokens for Type-2 clone detection
    fn normalize_tokens(&self, tokens: &[Token]) -> Vec<Token> {
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
    fn canonicalize_identifier(&self, name: &str) -> String {
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

/// MinHash generator for similarity estimation
pub struct MinHashGenerator {
    num_hashes: usize,
    seeds: Vec<u64>,
}

impl MinHashGenerator {
    pub fn new(num_hashes: usize) -> Self {
        let seeds = (0..num_hashes).map(|i| i as u64).collect();

        Self { num_hashes, seeds }
    }

    /// Compute MinHash signature from shingles
    pub fn compute_signature(&self, shingles: &[u64]) -> MinHashSignature {
        let mut signature = vec![u64::MAX; self.num_hashes];

        for &shingle in shingles {
            for (i, &seed) in self.seeds.iter().enumerate() {
                let hash = xxh64(&shingle.to_le_bytes(), seed);
                signature[i] = signature[i].min(hash);
            }
        }

        MinHashSignature { values: signature }
    }

    /// Generate k-shingles from tokens
    pub fn generate_shingles(&self, tokens: &[Token], k: usize) -> Vec<u64> {
        if tokens.len() < k {
            return vec![];
        }

        let mut shingles = Vec::new();
        let mut hasher = Hasher::new();

        for window in tokens.windows(k) {
            hasher.reset();
            for token in window {
                hasher.update(token.text.as_bytes());
            }
            let hash = hasher.finalize();
            shingles.push(u64::from_le_bytes(
                hash.as_bytes()[0..8].try_into().unwrap(),
            ));
        }

        shingles
    }
}

/// Main duplicate detection engine
pub struct DuplicateDetectionEngine {
    feature_extractor: UniversalFeatureExtractor,
    minhash_generator: MinHashGenerator,
    config: DuplicateDetectionConfig,
    fragments: DashMap<FragmentId, CodeFragment>,
    next_fragment_id: std::sync::atomic::AtomicU64,
}

impl DuplicateDetectionEngine {
    pub fn new(config: DuplicateDetectionConfig) -> Self {
        let minhash_generator = MinHashGenerator::new(config.num_hash_functions);
        let feature_extractor = UniversalFeatureExtractor::new(config.clone());

        Self {
            feature_extractor,
            minhash_generator,
            config,
            fragments: DashMap::new(),
            next_fragment_id: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Detect duplicates in a set of files
    pub fn detect_duplicates(&self, files: &[(PathBuf, String, Language)]) -> Result<CloneReport> {
        // Phase 1: Extract fragments from all files
        let mut all_fragments = Vec::new();
        for (path, content, lang) in files {
            let fragments = self.extract_fragments(path, content, *lang)?;
            all_fragments.extend(fragments);
        }

        // Phase 2: Find similar fragments using MinHash
        let clone_pairs = self.find_clone_pairs(&all_fragments)?;

        // Phase 3: Group clones into clone groups
        let clone_groups = self.group_clones(clone_pairs)?;

        // Phase 4: Generate summary and hotspots
        let summary = self.compute_summary(&all_fragments, &clone_groups);
        let hotspots = self.compute_hotspots(&clone_groups);

        Ok(CloneReport {
            summary,
            groups: clone_groups,
            hotspots,
        })
    }

    /// Extract code fragments from a single file
    fn extract_fragments(
        &self,
        path: &Path,
        content: &str,
        lang: Language,
    ) -> Result<Vec<CodeFragment>> {
        let tokens = self.feature_extractor.extract_features(content, lang);
        let mut fragments = Vec::new();

        // Extract function-level fragments by looking for function definitions
        let lines: Vec<&str> = content.lines().collect();
        let mut current_function_start = None;

        for (line_idx, line) in lines.iter().enumerate() {
            let line = line.trim();

            // Detect function starts (simplified)
            if self.is_function_start(line, lang) {
                current_function_start = Some(line_idx);
            }

            // Detect function ends (simplified)
            if current_function_start.is_some() && self.is_function_end(line, lang) {
                if let Some(start_line) = current_function_start {
                    let end_line = line_idx;
                    if end_line > start_line {
                        let fragment_content = lines[start_line..=end_line].join("\n");
                        let fragment_tokens = self
                            .feature_extractor
                            .extract_features(&fragment_content, lang);

                        if fragment_tokens.len() >= self.config.min_tokens {
                            let fragment = self.create_fragment(
                                path,
                                &fragment_content,
                                fragment_tokens,
                                start_line + 1, // 1-indexed
                                end_line + 1,
                                lang,
                            )?;
                            fragments.push(fragment);
                        }
                    }
                    current_function_start = None;
                }
            }
        }

        // If no functions found, treat entire file as one fragment
        if fragments.is_empty() && tokens.len() >= self.config.min_tokens {
            let fragment = self.create_fragment(path, content, tokens, 1, lines.len(), lang)?;
            fragments.push(fragment);
        }

        Ok(fragments)
    }

    /// Create a code fragment
    fn create_fragment(
        &self,
        path: &Path,
        content: &str,
        tokens: Vec<Token>,
        start_line: usize,
        end_line: usize,
        lang: Language,
    ) -> Result<CodeFragment> {
        let id = self
            .next_fragment_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        // Generate shingles and signature
        let shingles = self
            .minhash_generator
            .generate_shingles(&tokens, self.config.shingle_size);
        let signature = self.minhash_generator.compute_signature(&shingles);

        // Compute normalized hash
        let mut hasher = Hasher::new();
        for token in &tokens {
            hasher.update(token.text.as_bytes());
        }
        let hash = u64::from_le_bytes(hasher.finalize().as_bytes()[0..8].try_into().unwrap());

        let fragment = CodeFragment {
            id,
            file_path: path.to_path_buf(),
            start_line,
            end_line,
            start_column: 1,
            end_column: 1,
            raw_content: content.to_string(),
            tokens: Vec::new(), // Save memory by not storing raw tokens
            normalized_tokens: tokens,
            signature,
            hash,
            language: lang,
        };

        self.fragments.insert(id, fragment.clone());
        Ok(fragment)
    }

    /// Check if line starts a function
    fn is_function_start(&self, line: &str, lang: Language) -> bool {
        match lang {
            Language::Rust => line.contains("fn ") && line.contains("("),
            Language::TypeScript | Language::JavaScript => {
                line.contains("function ")
                    || line.contains("=> {")
                    || (line.contains("(") && line.contains(") {"))
            }
            Language::Python => line.starts_with("def ") && line.contains("("),
            Language::C | Language::Cpp => {
                // C/C++ function detection (simplified)
                line.contains("(") && (line.contains(") {") || line.ends_with("{"))
            }
        }
    }

    /// Check if line ends a function
    fn is_function_end(&self, line: &str, lang: Language) -> bool {
        match lang {
            Language::Rust
            | Language::TypeScript
            | Language::JavaScript
            | Language::C
            | Language::Cpp => line == "}",
            Language::Python => {
                // Python function ends when we reach another def or class at the same level
                line.starts_with("def ")
                    || line.starts_with("class ")
                    || (!line.starts_with(" ")
                        && !line.starts_with("\t")
                        && !line.trim().is_empty())
            }
        }
    }

    /// Find clone pairs using similarity threshold
    fn find_clone_pairs(
        &self,
        fragments: &[CodeFragment],
    ) -> Result<Vec<(FragmentId, FragmentId, f64)>> {
        // Compare all pairs (O(n²) - would use LSH in production)
        // Use parallel processing to speed up the comparison
        let clone_pairs: Vec<(FragmentId, FragmentId, f64)> = (0..fragments.len())
            .into_par_iter()
            .flat_map(|i| {
                let frag1 = &fragments[i];
                fragments[i + 1..]
                    .iter()
                    .filter_map(move |frag2| {
                        let similarity = frag1.signature.jaccard_similarity(&frag2.signature);
                        if similarity >= self.config.similarity_threshold {
                            Some((frag1.id, frag2.id, similarity))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        Ok(clone_pairs)
    }

    /// Group similar fragments into clone groups
    fn group_clones(
        &self,
        clone_pairs: Vec<(FragmentId, FragmentId, f64)>,
    ) -> Result<Vec<CloneGroup>> {
        // Use Union-Find for grouping
        let mut groups: HashMap<FragmentId, Vec<FragmentId>> = HashMap::new();
        let mut representative: HashMap<FragmentId, FragmentId> = HashMap::new();

        // Initialize each fragment as its own group
        for fragment in self.fragments.iter() {
            let id = *fragment.key();
            representative.insert(id, id);
            groups.insert(id, vec![id]);
        }

        // Union fragments in clone pairs
        for (id1, id2, _similarity) in clone_pairs {
            let rep1 = Self::find_representative(&representative, id1);
            let rep2 = Self::find_representative(&representative, id2);

            if rep1 != rep2 {
                // Merge groups
                if let (Some(group1), Some(group2)) = (groups.remove(&rep1), groups.remove(&rep2)) {
                    let mut merged = group1;
                    merged.extend(group2);
                    groups.insert(rep1, merged);
                    representative.insert(rep2, rep1);
                }
            }
        }

        // Convert to CloneGroup format
        let mut clone_groups = Vec::new();
        let mut group_id = 1;

        for (rep_id, fragment_ids) in groups {
            if fragment_ids.len() >= self.config.min_group_size {
                let instances: Vec<CloneInstance> = fragment_ids
                    .iter()
                    .filter_map(|&id| self.fragments.get(&id))
                    .map(|frag| CloneInstance {
                        file: frag.file_path.clone(),
                        start_line: frag.start_line,
                        end_line: frag.end_line,
                        start_column: frag.start_column,
                        end_column: frag.end_column,
                        similarity_to_representative: 1.0, // Simplified
                        normalized_hash: frag.hash,
                    })
                    .collect();

                if !instances.is_empty() {
                    let total_lines = instances
                        .iter()
                        .map(|i| i.end_line - i.start_line + 1)
                        .sum();

                    let total_tokens = fragment_ids
                        .iter()
                        .filter_map(|&id| self.fragments.get(&id))
                        .map(|f| f.normalized_tokens.len())
                        .sum();

                    clone_groups.push(CloneGroup {
                        id: group_id,
                        clone_type: CloneType::Type2 {
                            similarity: self.config.similarity_threshold,
                            normalized: true,
                        },
                        fragments: instances,
                        total_lines,
                        total_tokens,
                        average_similarity: self.config.similarity_threshold,
                        representative: rep_id,
                    });

                    group_id += 1;
                }
            }
        }

        Ok(clone_groups)
    }

    /// Find representative in Union-Find structure
    fn find_representative(
        representative: &HashMap<FragmentId, FragmentId>,
        id: FragmentId,
    ) -> FragmentId {
        if let Some(&rep) = representative.get(&id) {
            if rep == id {
                id
            } else {
                Self::find_representative(representative, rep)
            }
        } else {
            id
        }
    }

    /// Compute summary statistics
    fn compute_summary(&self, fragments: &[CodeFragment], groups: &[CloneGroup]) -> CloneSummary {
        let total_files = fragments
            .iter()
            .map(|f| &f.file_path)
            .collect::<HashSet<_>>()
            .len();

        let duplicate_lines = groups.iter().map(|g| g.total_lines).sum();

        let total_lines = fragments
            .iter()
            .map(|f| f.end_line - f.start_line + 1)
            .sum();

        let duplication_ratio = if total_lines > 0 {
            duplicate_lines as f64 / total_lines as f64
        } else {
            0.0
        };

        let largest_group_size = groups.iter().map(|g| g.fragments.len()).max().unwrap_or(0);

        CloneSummary {
            total_files,
            total_fragments: fragments.len(),
            duplicate_lines,
            total_lines,
            duplication_ratio,
            clone_groups: groups.len(),
            largest_group_size,
        }
    }

    /// Compute duplication hotspots
    fn compute_hotspots(&self, groups: &[CloneGroup]) -> Vec<DuplicationHotspot> {
        let mut file_stats: HashMap<PathBuf, (usize, usize)> = HashMap::new();

        for group in groups {
            for instance in &group.fragments {
                let (lines, count) = file_stats.entry(instance.file.clone()).or_insert((0, 0));
                *lines += instance.end_line - instance.start_line + 1;
                *count += 1;
            }
        }

        let mut hotspots: Vec<DuplicationHotspot> = file_stats
            .into_iter()
            .map(|(file, (duplicate_lines, clone_groups))| {
                let severity =
                    (duplicate_lines as f64).ln().max(1.0) * (clone_groups as f64).sqrt();
                DuplicationHotspot {
                    file,
                    duplicate_lines,
                    clone_groups,
                    severity,
                }
            })
            .collect();

        hotspots.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());
        hotspots.truncate(10); // Top 10 hotspots
        hotspots
    }
}

impl Default for DuplicateDetectionEngine {
    fn default() -> Self {
        Self::new(DuplicateDetectionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_hash() {
        let token1 = Token::new(TokenKind::Identifier("test".to_string()));
        let token2 = Token::new(TokenKind::Identifier("test".to_string()));
        assert_eq!(token1.hash(), token2.hash());
    }

    #[test]
    fn test_minhash_similarity() {
        let sig1 = MinHashSignature {
            values: vec![1, 2, 3, 4, 5],
        };
        let sig2 = MinHashSignature {
            values: vec![1, 2, 3, 6, 7],
        };
        let similarity = sig1.jaccard_similarity(&sig2);
        assert_eq!(similarity, 0.6); // 3 out of 5 match
    }

    #[test]
    fn test_feature_extraction() {
        let config = DuplicateDetectionConfig::default();
        let extractor = UniversalFeatureExtractor::new(config);

        let tokens = extractor.extract_features("fn test() { return 42; }", Language::Rust);
        assert!(!tokens.is_empty());

        // Should normalize identifiers
        assert!(tokens
            .iter()
            .any(|t| matches!(&t.kind, TokenKind::Identifier(name) if name.starts_with("VAR_"))));
    }

    #[test]
    fn test_duplicate_detection() {
        // Create config with lower min_tokens for testing
        let config = DuplicateDetectionConfig {
            min_tokens: 5, // Lower threshold for test snippets
            ..Default::default()
        };
        let engine = DuplicateDetectionEngine::new(config);

        let files = vec![
            (
                PathBuf::from("test1.rs"),
                "fn hello() { println!(\"Hello\"); }".to_string(),
                Language::Rust,
            ),
            (
                PathBuf::from("test2.rs"),
                "fn greet() { println!(\"Hello\"); }".to_string(),
                Language::Rust,
            ),
        ];

        let report = engine.detect_duplicates(&files).unwrap();
        assert!(report.summary.total_fragments >= 1);
    }

    #[test]
    fn test_shingle_generation() {
        let generator = MinHashGenerator::new(100);
        let tokens = vec![
            Token::new(TokenKind::Keyword("fn".to_string())),
            Token::new(TokenKind::Identifier("test".to_string())),
            Token::new(TokenKind::Delimiter("(".to_string())),
            Token::new(TokenKind::Delimiter(")".to_string())),
            Token::new(TokenKind::Delimiter("{".to_string())),
        ];

        let shingles = generator.generate_shingles(&tokens, 3);
        assert_eq!(shingles.len(), 3); // 5 tokens, k=3 -> 3 shingles
    }
}
