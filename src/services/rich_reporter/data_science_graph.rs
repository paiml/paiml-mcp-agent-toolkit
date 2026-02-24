impl DataScienceAnalyzer {
    /// Calculate PageRank centrality for findings based on file dependencies
    ///
    /// Higher PageRank = finding is in a more "central" file that many others depend on
    #[allow(clippy::cast_possible_truncation)]
    pub fn calculate_pagerank(&self, findings: &mut [Finding], dependencies: &[(String, String)]) {
        if findings.is_empty() || dependencies.is_empty() {
            return;
        }

        // Build file -> node_id mapping
        let mut file_to_node: HashMap<String, usize> = HashMap::new();
        let mut node_id = 0;

        for finding in findings.iter() {
            let file = finding.location.file.to_string_lossy().to_string();
            if let std::collections::hash_map::Entry::Vacant(e) = file_to_node.entry(file) {
                e.insert(node_id);
                node_id += 1;
            }
        }

        for (from, to) in dependencies {
            if let std::collections::hash_map::Entry::Vacant(e) =
                file_to_node.entry(from.clone())
            {
                e.insert(node_id);
                node_id += 1;
            }
            if let std::collections::hash_map::Entry::Vacant(e) = file_to_node.entry(to.clone()) {
                e.insert(node_id);
                node_id += 1;
            }
        }

        if node_id == 0 {
            return;
        }

        // Build graph using trueno_graph
        let mut graph = trueno_graph::CsrGraph::new();

        // Add edges (nodes are implicitly created)
        for (from, to) in dependencies {
            if let (Some(&from_id), Some(&to_id)) =
                (file_to_node.get(from), file_to_node.get(to))
            {
                // Set node names first
                graph.set_node_name(trueno_graph::NodeId(from_id as u32), from.clone());
                graph.set_node_name(trueno_graph::NodeId(to_id as u32), to.clone());
                // Add edge with weight
                let _ = graph.add_edge(
                    trueno_graph::NodeId(from_id as u32),
                    trueno_graph::NodeId(to_id as u32),
                    1.0,
                );
            }
        }

        // Calculate PageRank
        let scores = match trueno_graph::pagerank(&graph, 20, 1e-6) {
            Ok(s) => s,
            Err(_) => return,
        };

        // Assign PageRank scores to findings
        for finding in findings.iter_mut() {
            let file = finding.location.file.to_string_lossy().to_string();
            if let Some(&node) = file_to_node.get(&file) {
                if node < scores.len() {
                    finding.pagerank = Some(scores[node] as f32);
                }
            }
        }
    }

    /// Detect code communities using Louvain algorithm
    pub fn detect_communities(
        &self,
        findings: &mut [Finding],
        dependencies: &[(String, String)],
    ) -> Vec<CodeCommunity> {
        if findings.is_empty() {
            return Vec::new();
        }

        // Build file -> node_id mapping
        let mut file_to_node: HashMap<String, usize> = HashMap::new();
        let mut node_to_file: HashMap<usize, String> = HashMap::new();
        let mut node_id = 0;

        for finding in findings.iter() {
            let file = finding.location.file.to_string_lossy().to_string();
            if !file_to_node.contains_key(&file) {
                file_to_node.insert(file.clone(), node_id);
                node_to_file.insert(node_id, file);
                node_id += 1;
            }
        }

        // If no dependencies, create simple file-based communities
        if dependencies.is_empty() || node_id == 0 {
            return self.build_file_communities(findings);
        }

        // Add dependency nodes
        for (from, to) in dependencies {
            if !file_to_node.contains_key(from) {
                file_to_node.insert(from.clone(), node_id);
                node_to_file.insert(node_id, from.clone());
                node_id += 1;
            }
            if !file_to_node.contains_key(to) {
                file_to_node.insert(to.clone(), node_id);
                node_to_file.insert(node_id, to.clone());
                node_id += 1;
            }
        }

        // Build edge list for aprender
        let edges: Vec<(usize, usize)> = dependencies
            .iter()
            .filter_map(
                |(from, to)| match (file_to_node.get(from), file_to_node.get(to)) {
                    (Some(&from_id), Some(&to_id)) => Some((from_id, to_id)),
                    _ => None,
                },
            )
            .collect();

        // Create undirected graph from edge list
        let graph = aprender::graph::Graph::from_edges(&edges, false);

        // Run Louvain community detection
        let community_assignments = graph.louvain();

        // Create community name mapping
        let mut node_to_community: HashMap<usize, usize> = HashMap::new();
        for (comm_id, nodes) in community_assignments.iter().enumerate() {
            for &node in nodes {
                node_to_community.insert(node, comm_id);
            }
        }

        // Assign communities to findings
        for finding in findings.iter_mut() {
            let file = finding.location.file.to_string_lossy().to_string();
            if let Some(&node) = file_to_node.get(&file) {
                if let Some(&comm) = node_to_community.get(&node) {
                    finding.community = Some(format!("community_{}", comm));
                }
            }
        }

        // Build community summaries
        community_assignments
            .iter()
            .enumerate()
            .filter(|(_, nodes): &(usize, &Vec<usize>)| !nodes.is_empty())
            .map(|(comm_id, nodes): (usize, &Vec<usize>)| {
                let files: Vec<PathBuf> = nodes
                    .iter()
                    .filter_map(|n| node_to_file.get(n))
                    .map(PathBuf::from)
                    .collect();

                let defect_count = findings
                    .iter()
                    .filter(|f| {
                        f.community
                            .as_ref()
                            .map(|c| c == &format!("community_{}", comm_id))
                            .unwrap_or(false)
                    })
                    .count();

                let primary_issue = findings
                    .iter()
                    .filter(|f| {
                        f.community
                            .as_ref()
                            .map(|c| c == &format!("community_{}", comm_id))
                            .unwrap_or(false)
                    })
                    .fold(HashMap::new(), |mut acc, f| {
                        *acc.entry(f.category.clone()).or_insert(0usize) += 1;
                        acc
                    })
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(cat, _)| cat);

                CodeCommunity {
                    name: format!("community_{}", comm_id),
                    modularity: 0.0, // Would need graph to calculate
                    files,
                    primary_issue,
                    defect_count,
                }
            })
            .collect()
    }

    /// Build file-based communities when no dependencies are available
    fn build_file_communities(&self, findings: &mut [Finding]) -> Vec<CodeCommunity> {
        let mut communities = Vec::new();
        for finding in findings.iter_mut() {
            let file = finding.location.file.to_string_lossy().to_string();
            finding.community = Some(file.clone());
        }

        // Group by file
        let mut file_groups: HashMap<String, Vec<&Finding>> = HashMap::new();
        for finding in findings.iter() {
            let file = finding.location.file.to_string_lossy().to_string();
            file_groups.entry(file).or_default().push(finding);
        }

        for (file, group_findings) in file_groups {
            let mut category_counts: HashMap<&str, usize> = HashMap::new();
            for f in &group_findings {
                *category_counts.entry(&f.category).or_insert(0) += 1;
            }
            let primary = category_counts
                .into_iter()
                .max_by_key(|(_, c)| *c)
                .map(|(cat, _)| cat.to_string());

            communities.push(CodeCommunity {
                name: PathBuf::from(&file)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or(file.clone()),
                modularity: 1.0,
                files: vec![PathBuf::from(&file)],
                primary_issue: primary,
                defect_count: group_findings.len(),
            });
        }

        communities
    }
}
