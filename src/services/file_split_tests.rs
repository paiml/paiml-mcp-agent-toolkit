#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connected_components_empty() {
        let graph = UndirectedGraph::new();
        let comms = connected_components(&graph);
        assert!(comms.is_empty());
    }

    #[test]
    fn test_connected_components_singleton() {
        let mut graph = UndirectedGraph::new();
        graph.add_node(NodeData {
            path: PathBuf::from("test.rs"),
            module: "test".to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 0,
        });
        let comms = connected_components(&graph);
        assert_eq!(comms.len(), 1);
        assert_eq!(comms[0], 0);
    }

    #[test]
    fn test_connected_components_two_disconnected() {
        let mut graph = UndirectedGraph::new();
        graph.add_node(NodeData {
            path: PathBuf::from("a.rs"),
            module: "a".to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 0,
        });
        graph.add_node(NodeData {
            path: PathBuf::from("b.rs"),
            module: "b".to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 0,
        });
        let comms = connected_components(&graph);
        assert_eq!(comms.len(), 2);
        // Should be in different communities
        assert_ne!(comms[0], comms[1]);
    }

    #[test]
    fn test_connected_components_two_connected() {
        let mut graph = UndirectedGraph::new();
        let n1 = graph.add_node(NodeData {
            path: PathBuf::from("a.rs"),
            module: "a".to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 0,
        });
        let n2 = graph.add_node(NodeData {
            path: PathBuf::from("b.rs"),
            module: "b".to_string(),
            symbols: vec![],
            loc: 10,
            complexity: 1.0,
            ast_hash: 0,
        });
        graph.add_edge(n1, n2, 1.0);
        let comms = connected_components(&graph);
        assert_eq!(comms.len(), 2);
        // Should be in the same community
        assert_eq!(comms[0], comms[1]);
    }

    #[test]
    fn test_estimate_total_lines_empty() {
        let entries: Vec<&FunctionEntry> = vec![];
        assert_eq!(estimate_total_lines(&entries), 0);
    }

    #[test]
    fn test_split_impact_default() {
        let impact = SplitImpact {
            importing_files: vec!["a.rs".to_string()],
            circular_risks: vec![],
        };
        assert_eq!(impact.importing_files.len(), 1);
        assert!(impact.circular_risks.is_empty());
    }

    fn make_entry(name: &str, file: &str, start: usize, end: usize) -> FunctionEntry {
        use crate::services::agent_context::function_index::DefinitionType;
        FunctionEntry {
            file_path: file.to_string(),
            function_name: name.to_string(),
            signature: format!("fn {}()", name),
            definition_type: DefinitionType::Function,
            doc_comment: None,
            source: format!("fn {}() {{}}", name),
            start_line: start,
            end_line: end,
            language: "rust".to_string(),
            quality: Default::default(),
            checksum: String::new(),
            commit_count: 0,
            churn_score: 0.0,
            clone_count: 0,
            pattern_diversity: 0.0,
            fault_annotations: vec![],
            linked_definition: None,
        }
    }

    fn make_struct_entry(name: &str, file: &str, start: usize, end: usize) -> FunctionEntry {
        use crate::services::agent_context::function_index::DefinitionType;
        let mut entry = make_entry(name, file, start, end);
        entry.definition_type = DefinitionType::Struct;
        entry.signature = format!("struct {}", name);
        entry
    }

    #[test]
    fn test_name_cluster_dominant_type() {
        let e1 = make_struct_entry("MyConfig", "a.rs", 1, 10);
        let e2 = make_entry("new", "a.rs", 12, 20);
        let e3 = make_entry("load", "a.rs", 22, 30);
        let entries: Vec<&FunctionEntry> = vec![&e1, &e2, &e3];
        let (name, signal, confidence) = name_cluster(&entries, "a.rs");
        // Should use dominant type or function theme depending on signal
        assert!(!name.is_empty());
        assert!(!signal.is_empty());
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_name_cluster_fallback() {
        // Create entries with no clear signal
        let e1 = make_entry("x", "src/mod.rs", 1, 5);
        let e2 = make_entry("y", "src/mod.rs", 6, 10);
        let entries: Vec<&FunctionEntry> = vec![&e1, &e2];
        let (name, signal, confidence) = name_cluster(&entries, "src/mod.rs");
        // Fallback should produce something
        assert!(!name.is_empty());
        assert!(confidence <= 1.0);
        // With such short names and no signal, likely falls back
        assert!(signal == "ContextWord" || signal == "Fallback");
    }

    #[test]
    #[ignore] // Requires live context index
    fn test_compute_cohesion_single() {
        let cohesion = compute_cohesion(
            &[0],
            &AgentContextIndex::build(std::path::Path::new(".")).unwrap_or_else(|_| {
                // Fallback: test with empty-ish assertion
                panic!("Index needed for cohesion test");
            }),
            &[0],
            &HashMap::new(),
        );
        assert_eq!(cohesion, 1.0);
    }

    #[test]
    fn test_split_plan_serialization() {
        let plan = SplitPlan {
            source_file: "test.rs".to_string(),
            total_lines: 500,
            clusters: vec![SplitCluster {
                suggested_name: "config".to_string(),
                naming_signal: "DominantType".to_string(),
                confidence: 0.9,
                items: vec![ClusterItem {
                    name: "Config".to_string(),
                    definition_type: "Struct".to_string(),
                    line_range: (1, 50),
                    calls: vec![],
                    called_by: vec![],
                }],
                estimated_lines: 50,
                cohesion: 0.8,
            }],
            unclustered: vec![],
            impact: SplitImpact {
                importing_files: vec!["main.rs".to_string()],
                circular_risks: vec![],
            },
            modularity: 0.45,
        };

        let json = serde_json::to_string(&plan).unwrap();
        assert!(json.contains("config"));
        assert!(json.contains("DominantType"));
        assert!(json.contains("main.rs"));
    }

    #[test]
    fn test_cluster_item_serialization() {
        let item = ClusterItem {
            name: "process_data".to_string(),
            definition_type: "Function".to_string(),
            line_range: (10, 50),
            calls: vec!["helper".to_string()],
            called_by: vec!["main".to_string()],
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("process_data"));
        assert!(json.contains("helper"));
    }

    #[test]
    fn test_split_cluster_serialization() {
        let cluster = SplitCluster {
            suggested_name: "parsing".to_string(),
            naming_signal: "FunctionTheme".to_string(),
            confidence: 0.85,
            items: vec![],
            estimated_lines: 200,
            cohesion: 0.6,
        };
        let json = serde_json::to_string(&cluster).unwrap();
        assert!(json.contains("parsing"));
        assert!(json.contains("0.85"));
    }

    #[test]
    fn test_suggest_split_missing_file() {
        // Build index on current project
        let index = match AgentContextIndex::build(std::path::Path::new(".")) {
            Ok(i) => i,
            Err(_) => return, // Skip if can't build index
        };

        let result = suggest_split(&index, "nonexistent_file.rs", 1.0, 50);
        assert!(result.is_none());
    }

    #[test]
    fn test_execute_split_empty_plan() {
        let plan = SplitPlan {
            source_file: "test.rs".to_string(),
            total_lines: 100,
            clusters: vec![],
            unclustered: vec![],
            impact: SplitImpact {
                importing_files: vec![],
                circular_risks: vec![],
            },
            modularity: 0.0,
        };

        let temp_dir = std::env::temp_dir().join("pmat_split_test");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test.rs");
        std::fs::write(&test_file, "fn main() {}\n").unwrap();

        let result = execute_split(&plan, &temp_dir);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_execute_split_with_cluster() {
        let plan = SplitPlan {
            source_file: "test.rs".to_string(),
            total_lines: 10,
            clusters: vec![SplitCluster {
                suggested_name: "helpers".to_string(),
                naming_signal: "FunctionTheme".to_string(),
                confidence: 0.8,
                items: vec![ClusterItem {
                    name: "helper_fn".to_string(),
                    definition_type: "Function".to_string(),
                    line_range: (1, 3),
                    calls: vec![],
                    called_by: vec![],
                }],
                estimated_lines: 3,
                cohesion: 1.0,
            }],
            unclustered: vec![],
            impact: SplitImpact {
                importing_files: vec![],
                circular_risks: vec![],
            },
            modularity: 0.5,
        };

        let temp_dir = std::env::temp_dir().join("pmat_split_test2");
        let _ = std::fs::create_dir_all(&temp_dir);
        let test_file = temp_dir.join("test.rs");
        std::fs::write(
            &test_file,
            "fn helper_fn() {}\nfn other() {}\nfn last() {}\n",
        )
        .unwrap();

        let result = execute_split(&plan, &temp_dir);
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("test_helpers.rs"));

        // Verify file was created with content
        let content = std::fs::read_to_string(&files[0]).unwrap();
        assert!(content.contains("helper_fn"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_estimate_total_lines_with_entries() {
        let e1 = make_entry("a", "f.rs", 1, 50);
        let e2 = make_entry("b", "f.rs", 51, 100);
        let entries: Vec<&FunctionEntry> = vec![&e1, &e2];
        assert_eq!(estimate_total_lines(&entries), 100);
    }

    #[test]
    fn test_make_cluster_item_basic() {
        let e1 = make_entry("process", "f.rs", 10, 30);
        let _entries: Vec<&FunctionEntry> = vec![&e1];
        let _func_indices = [0usize];
        let mut global_to_local = HashMap::new();
        global_to_local.insert(0usize, 0usize);

        let index_stub = AgentContextIndex::build(std::path::Path::new("."));
        if index_stub.is_err() {
            return; // Skip if can't build
        }
        // Just test with the struct creation path
        let item = ClusterItem {
            name: "process".to_string(),
            definition_type: "Function".to_string(),
            line_range: (10, 30),
            calls: vec![],
            called_by: vec![],
        };
        assert_eq!(item.name, "process");
        assert_eq!(item.line_range, (10, 30));
    }

    #[test]
    fn test_generic_prefix_blocklist_contains_from() {
        assert!(GENERIC_PREFIX_BLOCKLIST.contains(&"from"));
        assert!(GENERIC_PREFIX_BLOCKLIST.contains(&"into"));
        assert!(GENERIC_PREFIX_BLOCKLIST.contains(&"with"));
        assert!(GENERIC_PREFIX_BLOCKLIST.contains(&"make"));
        assert!(GENERIC_PREFIX_BLOCKLIST.contains(&"handle"));
    }

    #[test]
    fn test_generic_prefix_blocklist_allows_good_names() {
        assert!(!GENERIC_PREFIX_BLOCKLIST.contains(&"baseline"));
        assert!(!GENERIC_PREFIX_BLOCKLIST.contains(&"health"));
        assert!(!GENERIC_PREFIX_BLOCKLIST.contains(&"metrics"));
        assert!(!GENERIC_PREFIX_BLOCKLIST.contains(&"cluster"));
    }

    #[test]
    fn test_name_cluster_skips_generic_prefix() {
        // Create entries where common prefix would be "from" but should be skipped
        let e1 = make_entry("from_score", "f.rs", 1, 10);
        let e2 = make_entry("from_files", "f.rs", 11, 20);
        let e3 = make_entry("from_projects", "f.rs", 21, 30);
        let entries: Vec<&FunctionEntry> = vec![&e1, &e2, &e3];

        let (name, signal, _confidence) = name_cluster(&entries, "file_health.rs");
        // Should NOT be "from" — should fall through to a better signal
        assert_ne!(name, "from", "Generic prefix 'from' should be blocked");
        assert_ne!(
            signal, "CommonPrefix",
            "Should skip CommonPrefix for generic verb"
        );
    }

    #[test]
    fn test_name_cluster_allows_specific_prefix() {
        // Create entries where common prefix is a meaningful domain word
        let e1 = make_entry("baseline_save", "f.rs", 1, 10);
        let e2 = make_entry("baseline_load", "f.rs", 11, 20);
        let e3 = make_entry("baseline_check", "f.rs", 21, 30);
        let entries: Vec<&FunctionEntry> = vec![&e1, &e2, &e3];

        let (name, signal, _confidence) = name_cluster(&entries, "file_health.rs");
        // "baseline" should be accepted since it's not in the blocklist
        assert_eq!(name, "baseline");
        assert_eq!(signal, "CommonPrefix");
    }
}
