impl IncrementalCoverageAnalyzer {
    pub fn new(_db_path: &Path) -> Result<Self> {
        debug_assert!(_db_path.exists(), "_db_path must exist: {}", _db_path.display());
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
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        let content = tokio::fs::read(path).await?;
        let hash = blake3::hash(&content);
        Ok(*hash.as_bytes())
    }
}
