#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // parse_dag_data tests
    // ============================================================

    #[test]
    fn test_parse_dag_data_with_valid_data() {
        let dag_data = serde_json::json!({
            "nodes": 5,
            "edges": 4
        });

        let result = parse_dag_data(&dag_data);
        assert!(result.is_some());

        let dag = result.unwrap();
        assert_eq!(dag.nodes.len(), 5);
        assert_eq!(dag.edges.len(), 4);
    }

    #[test]
    fn test_parse_dag_data_with_zero_nodes() {
        let dag_data = serde_json::json!({
            "nodes": 0,
            "edges": 0
        });

        let result = parse_dag_data(&dag_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_dag_data_missing_nodes() {
        let dag_data = serde_json::json!({
            "edges": 3
        });

        let result = parse_dag_data(&dag_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_dag_data_missing_edges() {
        let dag_data = serde_json::json!({
            "nodes": 3
        });

        let result = parse_dag_data(&dag_data);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_dag_data_node_structure() {
        let dag_data = serde_json::json!({
            "nodes": 3,
            "edges": 2
        });

        let result = parse_dag_data(&dag_data).unwrap();

        // Verify node structure
        for (id, node) in &result.nodes {
            assert!(id.starts_with("node_"));
            assert!(node.label.starts_with("Module "));
            assert!(node.file_path.starts_with("module_"));
            assert_eq!(node.line_number, 1);
            assert_eq!(node.complexity, 1);
        }
    }

    #[test]
    fn test_parse_dag_data_edge_structure() {
        let dag_data = serde_json::json!({
            "nodes": 3,
            "edges": 2
        });

        let result = parse_dag_data(&dag_data).unwrap();

        // Verify edge structure
        for edge in &result.edges {
            assert!(edge.from.starts_with("node_"));
            assert!(edge.to.starts_with("node_"));
            assert_eq!(edge.weight, 1);
        }
    }

    // ============================================================
    // extract_complexity_from_result tests
    // ============================================================

    #[test]
    fn test_extract_complexity_from_result_missing_report() {
        let result = serde_json::json!({
            "status": "ok"
        });

        let complexity = extract_complexity_from_result(&result);
        assert!(complexity.is_none());
    }

    #[test]
    fn test_extract_complexity_from_result_empty_object() {
        let result = serde_json::json!({});
        let complexity = extract_complexity_from_result(&result);
        assert!(complexity.is_none());
    }

    // ============================================================
    // extract_dag_from_result tests
    // ============================================================

    #[test]
    fn test_extract_dag_from_result_valid() {
        let result = serde_json::json!({
            "nodes": 2,
            "edges": 1
        });

        let dag = extract_dag_from_result(&result);
        assert!(dag.is_some());
    }

    #[test]
    fn test_extract_dag_from_result_invalid() {
        let result = serde_json::json!({
            "invalid": "data"
        });

        let dag = extract_dag_from_result(&result);
        assert!(dag.is_none());
    }

    // ============================================================
    // Edge case tests
    // ============================================================

    #[test]
    fn test_parse_dag_data_large_counts() {
        let dag_data = serde_json::json!({
            "nodes": 100,
            "edges": 200
        });

        let result = parse_dag_data(&dag_data).unwrap();
        assert_eq!(result.nodes.len(), 100);
        assert_eq!(result.edges.len(), 200);
    }

    #[test]
    fn test_parse_dag_data_with_one_node() {
        let dag_data = serde_json::json!({
            "nodes": 1,
            "edges": 0
        });

        let result = parse_dag_data(&dag_data);
        assert!(result.is_some());

        let dag = result.unwrap();
        assert_eq!(dag.nodes.len(), 1);
        assert_eq!(dag.edges.len(), 0);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_parse_dag_data_node_edge_relationship(
            node_count in 1u64..50u64,
            edge_count in 0u64..100u64
        ) {
            let dag_data = serde_json::json!({
                "nodes": node_count,
                "edges": edge_count
            });

            let result = parse_dag_data(&dag_data);
            prop_assert!(result.is_some());

            let dag = result.unwrap();
            prop_assert_eq!(dag.nodes.len(), node_count as usize);
            prop_assert_eq!(dag.edges.len(), edge_count as usize);
        }

        #[test]
        fn test_protocol_trace_response_preservation(
            protocol_name in "[a-z]{1,10}",
            key in "[a-z]{1,10}",
            value in "[a-z0-9]{1,20}"
        ) {
            let response = serde_json::json!({ &key: &value });
            let trace = super::super::orchestration::ProtocolTrace {
                protocol_name: protocol_name.clone(),
                response: response.clone(),
            };

            prop_assert_eq!(trace.protocol_name, protocol_name);
            prop_assert_eq!(&trace.response[&key], &value);
        }
    }
}
