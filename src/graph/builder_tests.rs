// Tests for DependencyGraphBuilder
// Extracted from builder.rs

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_source_file() {
        assert!(DependencyGraphBuilder::is_source_file(Path::new("test.rs")));
        assert!(DependencyGraphBuilder::is_source_file(Path::new("test.py")));
        assert!(DependencyGraphBuilder::is_source_file(Path::new("test.ts")));
        assert!(!DependencyGraphBuilder::is_source_file(Path::new(
            "test.txt"
        )));
        assert!(!DependencyGraphBuilder::is_source_file(Path::new(
            "README.md"
        )));
    }

    #[test]
    fn test_extract_function_names() {
        assert_eq!(
            DependencyGraphBuilder::extract_function_name("pub fn test_func() {"),
            Some("test_func")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_function_name("fn private_func(arg: i32) {"),
            Some("private_func")
        );
    }

    #[test]
    fn test_extract_python_names() {
        assert_eq!(
            DependencyGraphBuilder::extract_python_function_name("def test_func():"),
            Some("test_func")
        );
        assert_eq!(
            DependencyGraphBuilder::extract_python_class_name("class TestClass(BaseClass):"),
            Some("TestClass")
        );
    }

    #[test]
    fn test_builder_creation() {
        let builder = DependencyGraphBuilder::new();
        assert!(builder.symbol_table.is_empty());
        assert_eq!(builder.graph.node_count(), 0);
        assert_eq!(builder.graph.edge_count(), 0);
    }

    /// Test that analyze_file cache hit returns correct node_id
    /// Validates unwrap at line 159-162
    #[test]
    fn test_analyze_file_cache_hit() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}\n").unwrap();

        let mut builder = DependencyGraphBuilder::new();

        // First analysis - should create new node
        let node_id_1 = builder.analyze_file(&test_file).unwrap();
        assert_eq!(builder.graph.node_count(), 1);
        assert!(builder.node_map.contains_key(&test_file));
        assert!(builder.processed_hashes.contains_key(&test_file));

        // Second analysis with same content - should return cached node_id
        let node_id_2 = builder.analyze_file(&test_file).unwrap();
        assert_eq!(node_id_1, node_id_2, "Cache hit should return same node_id");
        assert_eq!(
            builder.graph.node_count(),
            1,
            "Should not create duplicate node"
        );
    }

    /// Test that analyze_file updates node when content changes
    /// Validates unwrap at line 184-187
    #[test]
    fn test_analyze_file_content_change() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}\n").unwrap();

        let mut builder = DependencyGraphBuilder::new();

        // First analysis
        let node_id_1 = builder.analyze_file(&test_file).unwrap();
        let original_loc = builder.graph.node_weight(node_id_1).unwrap().loc;

        // Modify file (different content = different hash)
        fs::write(&test_file, "fn main() {}\nfn helper() {}\n").unwrap();

        // Second analysis - should update existing node
        let node_id_2 = builder.analyze_file(&test_file).unwrap();
        assert_eq!(
            node_id_1, node_id_2,
            "Should reuse same node_id for same path"
        );
        assert_eq!(
            builder.graph.node_count(),
            1,
            "Should still have only 1 node"
        );

        // Verify node was updated (LOC should increase)
        let updated_loc = builder.graph.node_weight(node_id_2).unwrap().loc;
        assert!(
            updated_loc > original_loc,
            "Node should be updated with new LOC"
        );
    }

    /// Test that node_map and processed_hashes stay synchronized
    /// Validates invariant that both maps are updated together (lines 191-195)
    #[test]
    fn test_node_map_hash_map_synchronization() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn test() {}\n").unwrap();

        let mut builder = DependencyGraphBuilder::new();

        // Analyze file
        builder.analyze_file(&test_file).unwrap();

        // Both maps should contain the file
        assert!(
            builder.node_map.contains_key(&test_file),
            "node_map should contain analyzed file"
        );
        assert!(
            builder.processed_hashes.contains_key(&test_file),
            "processed_hashes should contain analyzed file"
        );

        // Both maps should have same size
        assert_eq!(
            builder.node_map.len(),
            builder.processed_hashes.len(),
            "node_map and processed_hashes must stay synchronized"
        );
    }

    /// Test first-time file analysis creates both node and hash entry
    /// Validates initialization path (lines 189-195)
    #[test]
    fn test_first_time_analysis() {
        use std::fs;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("fresh.rs");
        fs::write(&test_file, "pub fn new_function() {}\n").unwrap();

        let mut builder = DependencyGraphBuilder::new();

        // Fresh builder should have empty maps
        assert_eq!(builder.node_map.len(), 0);
        assert_eq!(builder.processed_hashes.len(), 0);

        // First analysis
        let node_id = builder.analyze_file(&test_file).unwrap();

        // Should create entries in both maps
        assert_eq!(builder.node_map.len(), 1);
        assert_eq!(builder.processed_hashes.len(), 1);
        assert!(builder.node_map.contains_key(&test_file));
        assert!(builder.processed_hashes.contains_key(&test_file));

        // Should create node in graph
        assert_eq!(builder.graph.node_count(), 1);
        assert!(builder.graph.node_weight(node_id).is_some());
    }
}
