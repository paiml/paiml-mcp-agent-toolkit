use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

/// Incremental Coverage Analysis with Persistent State
/// Optimized for CI/CD environments with minimal overhead
pub struct IncrementalCoverageAnalyzer {
    coverage_cache: Arc<DashMap<Vec<u8>, Vec<u8>>>, // Simple in-memory cache for now
    ast_cache: Arc<DashMap<FileId, (u64, AstNode)>>,
    call_graph: Arc<CallGraph>,
    semaphore: Arc<Semaphore>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct FileId {
    pub path: PathBuf,
    pub hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub functions: Vec<FunctionInfo>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub complexity: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageUpdate {
    pub file_coverage: HashMap<FileId, FileCoverage>,
    pub aggregate_coverage: AggregateCoverage,
    pub delta_coverage: DeltaCoverage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    pub line_coverage: f64,
    pub branch_coverage: f64,
    pub function_coverage: f64,
    pub covered_lines: Vec<usize>,
    pub total_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateCoverage {
    pub line_percentage: f64,
    pub branch_percentage: f64,
    pub function_percentage: f64,
    pub total_files: usize,
    pub covered_files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaCoverage {
    pub new_lines_covered: usize,
    pub new_lines_total: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    pub modified_files: Vec<FileId>,
    pub added_files: Vec<FileId>,
    pub deleted_files: Vec<FileId>,
}

pub struct CallGraph {
    #[allow(dead_code)]
    edges: DashMap<String, HashSet<String>>,
    reverse_edges: DashMap<String, HashSet<String>>,
}

impl IncrementalCoverageAnalyzer {
    pub fn new(_db_path: &Path) -> Result<Self> {
        Ok(Self {
            coverage_cache: Arc::new(DashMap::new()),
            ast_cache: Arc::new(DashMap::new()),
            call_graph: Arc::new(CallGraph::new()),
            semaphore: Arc::new(Semaphore::new(num_cpus::get())),
        })
    }

    pub async fn analyze_changes(&self, changeset: &ChangeSet) -> Result<CoverageUpdate> {
        let affected_files = self.compute_affected_files(changeset).await?;

        // Parallel analysis with bounded concurrency
        let mut handles = Vec::new();

        for file_id in affected_files {
            let analyzer = self.clone();
            let handle = tokio::spawn(async move {
                let _permit = analyzer.semaphore.acquire().await?;
                analyzer.analyze_file_coverage(&file_id).await
            });
            handles.push(handle);
        }

        let mut file_coverage = HashMap::new();
        for handle in handles {
            let (file_id, coverage) = handle.await??;
            file_coverage.insert(file_id, coverage);
        }

        let aggregate = self.calculate_aggregate_coverage(&file_coverage)?;
        let delta = self.calculate_delta_coverage(changeset, &file_coverage)?;

        Ok(CoverageUpdate {
            file_coverage,
            aggregate_coverage: aggregate,
            delta_coverage: delta,
        })
    }

    async fn compute_affected_files(&self, changeset: &ChangeSet) -> Result<Vec<FileId>> {
        let mut affected = HashSet::new();

        // Direct changes
        affected.extend(changeset.modified_files.iter().cloned());
        affected.extend(changeset.added_files.iter().cloned());

        // Transitive dependencies via call graph
        for file in &changeset.modified_files {
            let dependents = self.call_graph.get_dependents(&file.path.to_string_lossy());
            for dep in dependents {
                let path = PathBuf::from(dep);
                let hash = self.compute_file_hash(&path).await?;
                affected.insert(FileId { path, hash });
            }
        }

        Ok(affected.into_iter().collect())
    }

    async fn analyze_file_coverage(&self, file_id: &FileId) -> Result<(FileId, FileCoverage)> {
        // Check cache first
        if let Some(cached) = self.load_cached_coverage(file_id)? {
            return Ok((file_id.clone(), cached));
        }

        // Parse file and extract coverage data
        let ast = self.parse_file(&file_id.path).await?;
        let coverage = self.compute_coverage(&ast).await?;

        // Store in cache
        self.store_coverage(file_id, &coverage)?;

        Ok((file_id.clone(), coverage))
    }

    async fn parse_file(&self, path: &Path) -> Result<AstNode> {
        let content = tokio::fs::read_to_string(path).await?;
        let hash = blake3::hash(content.as_bytes());

        // Check AST cache
        let file_id = FileId {
            path: path.to_path_buf(),
            hash: *hash.as_bytes(),
        };

        if let Some(cached) = self.ast_cache.get(&file_id) {
            return Ok(cached.1.clone());
        }

        // Parse based on file extension
        let ast = match path.extension().and_then(|s| s.to_str()) {
            Some("rs") => self.parse_rust_file(&content)?,
            Some("ts") | Some("tsx") | Some("js") | Some("jsx") => {
                self.parse_typescript_file(&content)?
            }
            Some("py") => self.parse_python_file(&content)?,
            _ => AstNode {
                functions: vec![],
                dependencies: vec![],
            },
        };

        self.ast_cache.insert(file_id, (0, ast.clone()));
        Ok(ast)
    }

    fn parse_rust_file(&self, content: &str) -> Result<AstNode> {
        // Simplified parsing - in production would use syn
        let mut functions = Vec::new();
        let mut dependencies = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("fn ") {
                if let Some(name) = line.split_whitespace().nth(1) {
                    functions.push(FunctionInfo {
                        name: name.trim_end_matches('(').to_string(),
                        start_line: line_num + 1,
                        end_line: line_num + 10, // Simplified
                        complexity: 1,
                    });
                }
            }

            if line.trim().starts_with("use ") {
                if let Some(dep) = line.split_whitespace().nth(1) {
                    dependencies.push(dep.trim_end_matches(';').to_string());
                }
            }
        }

        Ok(AstNode {
            functions,
            dependencies,
        })
    }

    fn parse_typescript_file(&self, content: &str) -> Result<AstNode> {
        // Simplified parsing
        let mut functions = Vec::new();
        let mut dependencies = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.contains("function ") || line.contains("const ") && line.contains("=>") {
                if let Some(name) = extract_function_name(line) {
                    functions.push(FunctionInfo {
                        name,
                        start_line: line_num + 1,
                        end_line: line_num + 10,
                        complexity: 1,
                    });
                }
            }

            if line.trim().starts_with("import ") {
                dependencies.push(line.to_string());
            }
        }

        Ok(AstNode {
            functions,
            dependencies,
        })
    }

    fn parse_python_file(&self, content: &str) -> Result<AstNode> {
        // Simplified parsing
        let mut functions = Vec::new();
        let mut dependencies = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            if line.trim().starts_with("def ") {
                if let Some(name) = line.split_whitespace().nth(1) {
                    functions.push(FunctionInfo {
                        name: name.trim_end_matches('(').trim_end_matches(':').to_string(),
                        start_line: line_num + 1,
                        end_line: line_num + 10,
                        complexity: 1,
                    });
                }
            }

            if line.trim().starts_with("import ") || line.trim().starts_with("from ") {
                dependencies.push(line.to_string());
            }
        }

        Ok(AstNode {
            functions,
            dependencies,
        })
    }

    async fn compute_coverage(&self, ast: &AstNode) -> Result<FileCoverage> {
        // In production, would integrate with llvm-cov or similar
        // For now, return mock data
        let total_lines = ast
            .functions
            .iter()
            .map(|f| f.end_line - f.start_line + 1)
            .sum::<usize>()
            .max(100);

        let covered_lines = (0..total_lines)
            .filter(|i| i % 3 != 0) // Mock: 2/3 lines covered
            .collect::<Vec<_>>();

        Ok(FileCoverage {
            line_coverage: covered_lines.len() as f64 / total_lines as f64 * 100.0,
            branch_coverage: 75.0,   // Mock
            function_coverage: 80.0, // Mock
            covered_lines,
            total_lines,
        })
    }

    pub async fn compute_file_hash(&self, path: &Path) -> Result<[u8; 32]> {
        let content = tokio::fs::read(path).await?;
        let hash = blake3::hash(&content);
        Ok(*hash.as_bytes())
    }

    fn load_cached_coverage(&self, file_id: &FileId) -> Result<Option<FileCoverage>> {
        let key = self.coverage_key(file_id);

        match self.coverage_cache.get(&key) {
            Some(data) => {
                let coverage: FileCoverage = bincode::deserialize(&data)?;
                Ok(Some(coverage))
            }
            None => Ok(None),
        }
    }

    fn store_coverage(&self, file_id: &FileId, coverage: &FileCoverage) -> Result<()> {
        let key = self.coverage_key(file_id);
        let data = bincode::serialize(coverage)?;
        self.coverage_cache.insert(key, data);
        Ok(())
    }

    fn coverage_key(&self, file_id: &FileId) -> Vec<u8> {
        let mut key = b"coverage:".to_vec();
        key.extend_from_slice(&file_id.hash);
        key
    }

    fn calculate_aggregate_coverage(
        &self,
        file_coverage: &HashMap<FileId, FileCoverage>,
    ) -> Result<AggregateCoverage> {
        let total_files = file_coverage.len();
        let covered_files = file_coverage
            .values()
            .filter(|c| c.line_coverage > 0.0)
            .count();

        let total_lines: usize = file_coverage.values().map(|c| c.total_lines).sum();

        let covered_lines: usize = file_coverage.values().map(|c| c.covered_lines.len()).sum();

        Ok(AggregateCoverage {
            line_percentage: if total_lines > 0 {
                covered_lines as f64 / total_lines as f64 * 100.0
            } else {
                0.0
            },
            branch_percentage: file_coverage
                .values()
                .map(|c| c.branch_coverage)
                .sum::<f64>()
                / total_files as f64,
            function_percentage: file_coverage
                .values()
                .map(|c| c.function_coverage)
                .sum::<f64>()
                / total_files as f64,
            total_files,
            covered_files,
        })
    }

    fn calculate_delta_coverage(
        &self,
        changeset: &ChangeSet,
        file_coverage: &HashMap<FileId, FileCoverage>,
    ) -> Result<DeltaCoverage> {
        let mut new_lines_total = 0;
        let mut new_lines_covered = 0;

        for file_id in &changeset.modified_files {
            if let Some(coverage) = file_coverage.get(file_id) {
                // In production, would diff to find actual new lines
                // For now, assume 10% of lines are new
                let new_lines = coverage.total_lines / 10;
                new_lines_total += new_lines;
                new_lines_covered += (new_lines as f64 * coverage.line_coverage / 100.0) as usize;
            }
        }

        Ok(DeltaCoverage {
            new_lines_covered,
            new_lines_total,
            percentage: if new_lines_total > 0 {
                new_lines_covered as f64 / new_lines_total as f64 * 100.0
            } else {
                100.0
            },
        })
    }
}

impl Clone for IncrementalCoverageAnalyzer {
    fn clone(&self) -> Self {
        Self {
            coverage_cache: self.coverage_cache.clone(),
            ast_cache: self.ast_cache.clone(),
            call_graph: self.call_graph.clone(),
            semaphore: self.semaphore.clone(),
        }
    }
}

impl CallGraph {
    fn new() -> Self {
        Self {
            edges: DashMap::new(),
            reverse_edges: DashMap::new(),
        }
    }

    fn get_dependents(&self, module: &str) -> Vec<String> {
        self.reverse_edges
            .get(module)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }
}

fn extract_function_name(line: &str) -> Option<String> {
    // Simplified function name extraction
    if let Some(pos) = line.find("function ") {
        let start = pos + 9;
        if let Some(end) = line[start..].find('(') {
            return Some(line[start..start + end].trim().to_string());
        }
    }

    if let Some(pos) = line.find("const ") {
        let start = pos + 6;
        if let Some(eq) = line[start..].find(" =") {
            return Some(line[start..start + eq].trim().to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_incremental_coverage() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("coverage.db");

        let analyzer = IncrementalCoverageAnalyzer::new(&db_path).unwrap();

        let test_file = temp_dir.path().join("test.rs");
        tokio::fs::write(
            &test_file,
            r#"
fn main() {
    println!("Hello, world!");
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
        )
        .await
        .unwrap();

        let hash = analyzer.compute_file_hash(&test_file).await.unwrap();
        let file_id = FileId {
            path: test_file.clone(),
            hash,
        };

        let changeset = ChangeSet {
            modified_files: vec![file_id],
            added_files: vec![],
            deleted_files: vec![],
        };

        let update = analyzer.analyze_changes(&changeset).await.unwrap();

        assert!(!update.file_coverage.is_empty());
        assert!(update.aggregate_coverage.line_percentage > 0.0);
    }
}
