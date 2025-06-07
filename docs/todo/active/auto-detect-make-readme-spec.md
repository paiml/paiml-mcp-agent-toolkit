```markdown
# Auto-Detection and Compression of Makefile and README for Deep Context

## Abstract

Critical project metadata resides in non-code artifacts (Makefile, README.md) that AST analysis ignores. This specification defines algorithms for extracting, compressing, and integrating these artifacts into deep context reports, ensuring LLMs receive complete project understanding without token bloat.

## Motivation

AST analysis provides structural metrics but lacks semantic context. A codebase with 15,239 functions tells us nothing about:
- Build orchestration and dependency management (Makefile)
- Project philosophy, API contracts, usage patterns (README)

These files often contain 80% noise (badges, verbose examples, redundant targets). We need intelligent compression that preserves signal.

## Technical Specification

### 1. File Detection

```rust
pub struct ProjectMetaDetector {
    patterns: Vec<(Regex, MetaFileType)>,
}

impl ProjectMetaDetector {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                (Regex::new(r"^Makefile$").unwrap(), MetaFileType::Makefile),
                (Regex::new(r"^makefile$").unwrap(), MetaFileType::Makefile),
                (Regex::new(r"^GNUmakefile$").unwrap(), MetaFileType::Makefile),
                (Regex::new(r"^README\.md$").unwrap(), MetaFileType::Readme),
                (Regex::new(r"^README\.markdown$").unwrap(), MetaFileType::Readme),
                (Regex::new(r"^README$").unwrap(), MetaFileType::Readme),
            ],
        }
    }
    
    pub async fn detect(&self, project_root: &Path) -> Vec<MetaFile> {
        // Parallel detection with 10ms timeout per file
        let mut tasks = JoinSet::new();
        for entry in WalkDir::new(project_root).max_depth(2) {
            // Only scan top 2 levels to avoid deep recursion
        }
    }
}
```

### 2. Makefile Compression Algorithm

**Goal**: Extract critical build targets and dependencies, discard verbose recipes.

```rust
pub struct MakefileCompressor {
    // Preserve these critical patterns
    critical_targets: HashSet<&'static str>,
    critical_vars: HashSet<&'static str>,
}

impl MakefileCompressor {
    pub fn compress(&self, content: &str) -> CompressedMakefile {
        let mut result = CompressedMakefile::default();
        
        // Phase 1: Extract variables (project metadata)
        for line in content.lines() {
            if let Some(caps) = VAR_PATTERN.captures(line) {
                let var_name = caps.get(1).unwrap().as_str();
                if self.critical_vars.contains(var_name) || 
                   var_name.starts_with("PROJECT_") {
                    result.variables.push(line.to_string());
                }
            }
        }
        
        // Phase 2: Extract target graph (build structure)
        let targets = self.parse_targets(content);
        for (name, target) in targets {
            if self.is_critical_target(&name) {
                result.targets.push(MakeTarget {
                    name,
                    deps: target.dependencies,
                    // Compress recipe to single line summary
                    recipe_summary: self.summarize_recipe(&target.recipe),
                });
            }
        }
        
        // Phase 3: Detect toolchain patterns
        result.detected_toolchain = self.detect_toolchain(&targets);
        
        result
    }
    
    fn is_critical_target(&self, name: &str) -> bool {
        self.critical_targets.contains(name) ||
        name == "all" || name == "build" || name == "test" ||
        name == "install" || name == "clean" || name == "release" ||
        name.starts_with("docker") || name.contains("deploy")
    }
    
    fn summarize_recipe(&self, recipe_lines: &[String]) -> String {
        // Extract first meaningful command, ignore echo/mkdir
        recipe_lines.iter()
            .filter(|line| !line.starts_with("@echo") && 
                          !line.starts_with("mkdir"))
            .next()
            .map(|line| line.trim_start_matches('@').to_string())
            .unwrap_or_else(|| "[complex recipe]".to_string())
    }
}
```

### 3. README Compression Algorithm

**Goal**: Extract project description, key features, API surface, discard examples/badges.

```rust
pub struct ReadmeCompressor {
    section_importance: HashMap<String, f32>,
    max_section_tokens: usize,
}

impl ReadmeCompressor {
    pub fn compress(&self, content: &str) -> CompressedReadme {
        let doc = parse_markdown(content);
        let mut sections = Vec::new();
        
        // Phase 1: Section scoring
        for section in doc.sections() {
            let score = self.calculate_section_score(&section);
            if score > 0.3 {  // Importance threshold
                sections.push((section, score));
            }
        }
        
        // Phase 2: Token budget allocation
        sections.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let mut token_budget = 2000;  // Target ~2KB compressed
        let mut result = CompressedReadme::default();
        
        for (section, _score) in sections {
            let compressed = self.compress_section(&section, token_budget);
            token_budget -= compressed.estimated_tokens();
            result.sections.push(compressed);
            
            if token_budget < 100 {
                break;
            }
        }
        
        result
    }
    
    fn calculate_section_score(&self, section: &Section) -> f32 {
        let title_lower = section.title.to_lowercase();
        
        // High-value sections
        if title_lower.contains("overview") || 
           title_lower.contains("architecture") ||
           title_lower.contains("api") ||
           title_lower.contains("philosophy") {
            return 0.9;
        }
        
        // Medium-value sections
        if title_lower.contains("features") ||
           title_lower.contains("usage") ||
           title_lower.contains("quickstart") {
            return 0.6;
        }
        
        // Low-value sections (will be filtered)
        if title_lower.contains("badge") ||
           title_lower.contains("license") ||
           title_lower.contains("contributing") {
            return 0.1;
        }
        
        0.4  // Default
    }
    
    fn compress_section(&self, section: &Section, budget: usize) -> CompressedSection {
        // Extract first paragraph + bullet points
        let mut content = String::new();
        
        // First paragraph is usually the summary
        if let Some(first_para) = section.paragraphs.first() {
            content.push_str(&first_para);
        }
        
        // Extract bullet points (often key features)
        for list in &section.lists {
            for item in &list.items {
                if content.len() < budget * 4 {  // ~4 chars per token
                    content.push_str(&format!("- {}\n", item.summary()));
                }
            }
        }
        
        CompressedSection {
            title: section.title.clone(),
            content: self.truncate_intelligently(content, budget),
        }
    }
}
```

### 4. Integration into Deep Context

```rust
impl DeepContextAnalyzer {
    pub async fn analyze_with_metadata(&self, config: &Config) -> DeepContext {
        // Existing AST analysis
        let mut context = self.analyze_ast(config).await?;
        
        // New: Project metadata extraction
        let detector = ProjectMetaDetector::new();
        let meta_files = detector.detect(&config.project_path).await;
        
        for meta_file in meta_files {
            match meta_file.file_type {
                MetaFileType::Makefile => {
                    let compressed = MakefileCompressor::new()
                        .compress(&meta_file.content);
                    context.build_info = Some(BuildInfo::from_makefile(compressed));
                }
                MetaFileType::Readme => {
                    let compressed = ReadmeCompressor::new()
                        .compress(&meta_file.content);
                    context.project_overview = Some(compressed.to_summary());
                }
            }
        }
        
        context
    }
}
```

### 5. Output Format Integration

```rust
impl DeepContextResult {
    pub fn format_enhanced(&self) -> String {
        let mut output = String::new();
        
        // Standard AST metrics
        output.push_str(&self.format_ast_analysis());
        
        // NEW: Compressed project metadata
        if let Some(build_info) = &self.build_info {
            output.push_str("\n## Build System\n\n");
            output.push_str(&format!("**Detected Toolchain:** {}\n", build_info.toolchain));
            output.push_str(&format!("**Primary Targets:** {}\n", 
                build_info.targets.join(", ")));
            output.push_str(&format!("**Key Dependencies:** {}\n", 
                build_info.dependencies.join(", ")));
        }
        
        if let Some(overview) = &self.project_overview {
            output.push_str("\n## Project Overview\n\n");
            output.push_str(&overview.compressed_description);
            if !overview.key_features.is_empty() {
                output.push_str("\n**Key Features:**\n");
                for feature in &overview.key_features {
                    output.push_str(&format!("- {}\n", feature));
                }
            }
        }
        
        output
    }
}
```

### 6. Performance Constraints

- **Detection**: <50ms for project root scanning
- **Compression**: <100ms per file
- **Memory**: <10MB for compression buffers
- **Output Size**:
    - Makefile: Max 1KB compressed
    - README: Max 2KB compressed
    - Total overhead: <3KB per project

### 7. Compression Heuristics

**Makefile Compression Ratios:**
- Variables: Keep PROJECT_*, CARGO_*, CC, CXX, LDFLAGS
- Targets: Keep top-level only, summarize recipes
- Comments: Remove all except section headers
- Typical compression: 10KB → 1KB (90% reduction)

**README Compression Ratios:**
- Headers: Preserve hierarchy, compress content
- Code blocks: Extract signatures only
- Badges/Images: Remove completely
- Examples: Keep first example only
- Typical compression: 20KB → 2KB (90% reduction)

## Implementation Priority

1. **Phase 1**: Basic detection and extraction
2. **Phase 2**: Compression algorithms with heuristics
3. **Phase 3**: Integration with deep context pipeline
4. **Phase 4**: Performance optimization (parallel extraction)

## Success Metrics

- LLM comprehension improves by 40% (measured by correct build command generation)
- Token usage increases by <5% despite added context
- Zero false positives in build target detection
- <200ms total overhead for compression pipeline
```