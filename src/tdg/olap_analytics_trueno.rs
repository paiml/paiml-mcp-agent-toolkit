// TruenoOlapAnalytics constructor and Arrow conversion methods.
// Provides new(), create_schema(), scores_to_arrow(), and arrow_to_scores().

impl TruenoOlapAnalytics {
    /// Create a new trueno-db analytics backend
    ///
    /// # Arguments
    ///
    /// * `path` - Path to Parquet file (optional, can be empty for new storage)
    ///
    /// # Returns
    ///
    /// Initialized analytics backend
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Create new empty storage
    /// let analytics = TruenoOlapAnalytics::new("").await?;
    ///
    /// // Or load existing Parquet file
    /// let analytics = TruenoOlapAnalytics::new("/tmp/tdg_scores.parquet").await?;
    /// ```
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub async fn new(path: &str) -> Result<Self> {
        let storage = if path.is_empty() {
            // Create empty storage
            trueno_db::storage::StorageEngine::new(vec![])
        } else {
            // Load existing Parquet file
            trueno_db::storage::StorageEngine::load_parquet(path)?
        };

        let query_engine = trueno_db::query::QueryEngine::new();
        let executor = trueno_db::query::QueryExecutor::new();

        Ok(Self {
            storage: std::sync::Mutex::new(storage),
            query_engine,
            executor,
        })
    }

    /// Create trueno-db schema for TDG scores
    ///
    /// # Note
    ///
    /// This is a placeholder for future schema creation using trueno-db's API.
    async fn create_schema(_db: &trueno_db::Database) -> Result<()> {
        // Schema creation using trueno-db's Arrow-based schema
        // This is a placeholder - actual implementation depends on trueno-db API

        // CREATE TABLE tdg_scores (
        //     file_path TEXT,
        //     structural_complexity REAL,
        //     semantic_complexity REAL,
        //     duplication_ratio REAL,
        //     coupling_score REAL,
        //     doc_coverage REAL,
        //     consistency_score REAL,
        //     entropy_score REAL,
        //     total REAL,
        //     language TEXT,
        //     confidence REAL
        // )

        Ok(())
    }

    /// Convert TdgScore to Arrow RecordBatch for trueno-db
    fn scores_to_arrow(&self, scores: &[TdgScore]) -> Result<arrow::record_batch::RecordBatch> {
        use arrow::array::{Float32Array, RecordBatch, StringArray};
        use arrow::datatypes::{DataType, Field, Schema};
        use std::sync::Arc;

        // Extract fields into columnar arrays
        let file_paths: Vec<String> = scores
            .iter()
            .map(|s| {
                s.file_path
                    .as_ref()
                    .and_then(|p| p.to_str())
                    .unwrap_or("")
                    .to_string()
            })
            .collect();

        let structural: Vec<f32> = scores.iter().map(|s| s.structural_complexity).collect();
        let semantic: Vec<f32> = scores.iter().map(|s| s.semantic_complexity).collect();
        let duplication: Vec<f32> = scores.iter().map(|s| s.duplication_ratio).collect();
        let coupling: Vec<f32> = scores.iter().map(|s| s.coupling_score).collect();
        let doc: Vec<f32> = scores.iter().map(|s| s.doc_coverage).collect();
        let consistency: Vec<f32> = scores.iter().map(|s| s.consistency_score).collect();
        let entropy: Vec<f32> = scores.iter().map(|s| s.entropy_score).collect();
        let total: Vec<f32> = scores.iter().map(|s| s.total).collect();
        let confidence: Vec<f32> = scores.iter().map(|s| s.confidence).collect();

        let languages: Vec<String> = scores.iter().map(|s| format!("{:?}", s.language)).collect();

        // Create Arrow schema
        let schema = Arc::new(Schema::new(vec![
            Field::new("file_path", DataType::Utf8, false),
            Field::new("structural_complexity", DataType::Float32, false),
            Field::new("semantic_complexity", DataType::Float32, false),
            Field::new("duplication_ratio", DataType::Float32, false),
            Field::new("coupling_score", DataType::Float32, false),
            Field::new("doc_coverage", DataType::Float32, false),
            Field::new("consistency_score", DataType::Float32, false),
            Field::new("entropy_score", DataType::Float32, false),
            Field::new("total", DataType::Float32, false),
            Field::new("confidence", DataType::Float32, false),
            Field::new("language", DataType::Utf8, false),
        ]));

        // Create columnar arrays
        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(StringArray::from(file_paths)),
                Arc::new(Float32Array::from(structural)),
                Arc::new(Float32Array::from(semantic)),
                Arc::new(Float32Array::from(duplication)),
                Arc::new(Float32Array::from(coupling)),
                Arc::new(Float32Array::from(doc)),
                Arc::new(Float32Array::from(consistency)),
                Arc::new(Float32Array::from(entropy)),
                Arc::new(Float32Array::from(total)),
                Arc::new(Float32Array::from(confidence)),
                Arc::new(StringArray::from(languages)),
            ],
        )?;

        Ok(batch)
    }

    /// Convert Arrow RecordBatch to Vec<TdgScore>
    fn arrow_to_scores(&self, batch: arrow::record_batch::RecordBatch) -> Result<Vec<TdgScore>> {
        if batch.num_rows() == 0 {
            return Ok(Vec::new());
        }

        let columns = extract_arrow_columns(&batch)?;
        let mut scores = Vec::with_capacity(batch.num_rows());
        for i in 0..batch.num_rows() {
            scores.push(build_tdg_score_from_row(&columns, i));
        }
        Ok(scores)
    }
}

/// Extracted Arrow columns for building TdgScore records
struct ArrowColumns<'a> {
    file_paths: &'a arrow::array::StringArray,
    structural: &'a arrow::array::Float32Array,
    semantic: &'a arrow::array::Float32Array,
    duplication: &'a arrow::array::Float32Array,
    coupling: &'a arrow::array::Float32Array,
    doc: &'a arrow::array::Float32Array,
    consistency: &'a arrow::array::Float32Array,
    entropy: &'a arrow::array::Float32Array,
    total: &'a arrow::array::Float32Array,
    confidence: &'a arrow::array::Float32Array,
    languages: &'a arrow::array::StringArray,
}

fn downcast_f32<'a>(batch: &'a arrow::record_batch::RecordBatch, col: usize, name: &str) -> Result<&'a arrow::array::Float32Array> {
    batch.column(col).as_any().downcast_ref::<arrow::array::Float32Array>()
        .ok_or_else(|| anyhow::anyhow!("Expected Float32Array for {}", name))
}

fn downcast_string<'a>(batch: &'a arrow::record_batch::RecordBatch, col: usize, name: &str) -> Result<&'a arrow::array::StringArray> {
    batch.column(col).as_any().downcast_ref::<arrow::array::StringArray>()
        .ok_or_else(|| anyhow::anyhow!("Expected StringArray for {}", name))
}

fn extract_arrow_columns(batch: &arrow::record_batch::RecordBatch) -> Result<ArrowColumns<'_>> {
    Ok(ArrowColumns {
        file_paths: downcast_string(batch, 0, "file_path")?,
        structural: downcast_f32(batch, 1, "structural_complexity")?,
        semantic: downcast_f32(batch, 2, "semantic_complexity")?,
        duplication: downcast_f32(batch, 3, "duplication_ratio")?,
        coupling: downcast_f32(batch, 4, "coupling_score")?,
        doc: downcast_f32(batch, 5, "doc_coverage")?,
        consistency: downcast_f32(batch, 6, "consistency_score")?,
        entropy: downcast_f32(batch, 7, "entropy_score")?,
        total: downcast_f32(batch, 8, "total")?,
        confidence: downcast_f32(batch, 9, "confidence")?,
        languages: downcast_string(batch, 10, "language")?,
    })
}

fn parse_language_str(s: &str) -> Language {
    match s {
        "Rust" => Language::Rust,
        "Python" => Language::Python,
        "JavaScript" => Language::JavaScript,
        "TypeScript" => Language::TypeScript,
        "Go" => Language::Go,
        "Java" => Language::Java,
        "Cpp" => Language::Cpp,
        "C" => Language::C,
        "Ruby" => Language::Ruby,
        "Swift" => Language::Swift,
        "Kotlin" => Language::Kotlin,
        "Ruchy" => Language::Ruchy,
        _ => Language::Unknown,
    }
}

fn build_tdg_score_from_row(cols: &ArrowColumns<'_>, i: usize) -> TdgScore {
    let file_path_str = cols.file_paths.value(i);
    let total_score = cols.total.value(i);
    TdgScore {
        file_path: if file_path_str.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(file_path_str))
        },
        structural_complexity: cols.structural.value(i),
        semantic_complexity: cols.semantic.value(i),
        duplication_ratio: cols.duplication.value(i),
        coupling_score: cols.coupling.value(i),
        doc_coverage: cols.doc.value(i),
        consistency_score: cols.consistency.value(i),
        entropy_score: cols.entropy.value(i),
        total: total_score,
        grade: crate::tdg::Grade::from_score(total_score),
        language: parse_language_str(cols.languages.value(i)),
        confidence: cols.confidence.value(i),
        penalties_applied: Vec::new(),
        critical_defects_count: 0,
        has_critical_defects: false,
    }
}
