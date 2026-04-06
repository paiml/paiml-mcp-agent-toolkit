// --- Stub types for extract when tree-sitter is disabled ---

/// Rich file-level extraction result with imports, test boundaries, and per-item visibility.
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileExtract {
    pub file: String,
    pub language: String,
    pub imports: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfg_test_line: Option<usize>,
    pub items: Vec<ExtractedItem>,
}

/// A single extracted item with visibility metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractedItem {
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub start_line: usize,
    pub end_line: usize,
    pub lines: usize,
    pub visibility: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ExtractedItem>,
}

/// Stub: Extract rich file details (requires tree-sitter feature).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
pub fn extract_file_details(
    path: &str,
    _source: &str,
    language: Language,
) -> Result<FileExtract, String> {
    Ok(FileExtract {
        file: path.to_string(),
        language: language.as_str().to_string(),
        imports: Vec::new(),
        cfg_test_line: None,
        items: Vec::new(),
    })
}
