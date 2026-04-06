// CommunityData - aggregated community metrics for Louvain algorithm
// Included by parallel_louvain.rs - shares parent scope

/// Aggregated community data for efficient calculations.
#[derive(Debug)]
#[allow(dead_code)]
struct CommunityData {
    /// Node to community mapping
    node_to_community: Vec<usize>,
    /// Sum of degrees for each community
    community_degrees: HashMap<usize, f64>,
    /// Internal weight for each community
    community_internal_weight: HashMap<usize, f64>,
}

impl CommunityData {
    /// Build community data from current assignment.
    fn new(communities: &[usize], graph_data: &GraphData) -> Self {
        let node_to_community = communities.to_vec();
        let mut community_degrees: HashMap<usize, f64> = HashMap::new();
        let mut community_internal_weight: HashMap<usize, f64> = HashMap::new();

        // Calculate community degrees
        for (node, &community) in communities.iter().enumerate() {
            *community_degrees.entry(community).or_insert(0.0) += graph_data.degrees[node];
        }

        // Calculate internal weights
        for (&(source, target), &weight) in &graph_data.edge_weights {
            if communities[source] == communities[target] {
                *community_internal_weight
                    .entry(communities[source])
                    .or_insert(0.0) += weight;
            }
        }

        CommunityData {
            node_to_community,
            community_degrees,
            community_internal_weight,
        }
    }
}
