// Matrix operations: EdgeData weights, SimpleSparseMatrix impl, GraphMatrices conversions
// Included by types.rs - shares parent module scope (no `use` imports)

impl SimpleSparseMatrix {
    /// Create a new sparse matrix with given dimensions
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(nrows: usize, ncols: usize) -> Self {
        Self {
            nrows,
            ncols,
            triplets: Vec::new(),
        }
    }

    /// Add a value at (row, col)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn push(&mut self, row: usize, col: usize, value: f64) {
        self.triplets.push((row, col, value));
    }

    /// Get row values as iterator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn row_values(&self, row: usize) -> impl Iterator<Item = (usize, f64)> + '_ {
        self.triplets
            .iter()
            .filter(move |(r, _, _)| *r == row)
            .map(|(_, c, v)| (*c, *v))
    }

    /// Get all values in a row as Vec
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn row_as_vec(&self, row: usize) -> Vec<(usize, f64)> {
        self.row_values(row).collect()
    }
}

impl EdgeData {
    /// Convert heterogeneous edge types to numeric weights
    /// Complexity: 3 (simple match with arithmetic)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn to_numeric_weight(&self) -> f64 {
        match self {
            EdgeData::Import { weight, .. } => *weight * 2.0, // Imports weighted higher
            EdgeData::FunctionCall { count, .. } => *count as f64,
            EdgeData::TypeDependency { strength, .. } => *strength * 1.5,
            EdgeData::DataFlow { confidence, .. } => *confidence,
            EdgeData::Inheritance { depth } => 3.0 / (*depth as f64 + 1.0),
        }
    }
}

/// Conversion from DependencyGraph to matrix representations
/// Complexity: 8 (loop with matrix operations)
/// Sovereign AI Stack: Uses simple Vec-based sparse matrices
impl From<&DependencyGraph> for GraphMatrices {
    fn from(graph: &DependencyGraph) -> Self {
        let n = graph.node_count();
        let mut adjacency = SimpleSparseMatrix::new(n, n);
        let mut out_degrees = vec![0.0; n];
        let mut edges = Vec::new();

        // Build adjacency matrix with edge weights
        for edge in graph.edge_references() {
            let weight = edge.weight().to_numeric_weight();
            let source = edge.source().0 as usize;
            let target = edge.target().0 as usize;

            if source < n && target < n {
                edges.push((source, target, weight));
                adjacency.push(source, target, weight);
                out_degrees[source] += weight;
            }
        }

        // Create column-stochastic transition matrix
        let transition = Self::normalize_columns(&adjacency, &out_degrees);

        // Compute Laplacian L = D - A
        let laplacian = Self::compute_laplacian(&adjacency);

        GraphMatrices {
            adjacency,
            transition,
            laplacian,
            out_degrees,
            node_count: n,
            edges,
        }
    }
}

impl GraphMatrices {
    /// Normalize columns for stochastic matrix
    /// Complexity: 6 (nested loop with early exit)
    fn normalize_columns(
        adjacency: &SimpleSparseMatrix,
        out_degrees: &[f64],
    ) -> SimpleSparseMatrix {
        let n = adjacency.nrows;
        let mut result = SimpleSparseMatrix::new(n, n);

        for i in 0..n {
            if out_degrees[i] > 0.0 {
                for (col, value) in adjacency.row_values(i) {
                    result.push(i, col, value / out_degrees[i]);
                }
            }
        }

        result
    }

    /// Compute graph Laplacian
    /// Complexity: 6 (simplified matrix operations)
    fn compute_laplacian(adjacency: &SimpleSparseMatrix) -> SimpleSparseMatrix {
        let n = adjacency.nrows;
        let mut result = SimpleSparseMatrix::new(n, n);

        // Compute degree matrix D and build Laplacian L = D - A
        for i in 0..n {
            let row_vals: Vec<_> = adjacency.row_values(i).collect();
            let degree: f64 = row_vals.iter().map(|(_, v)| v).sum();

            // Add diagonal degree
            result.push(i, i, degree);

            // Subtract adjacency values
            for (col, value) in row_vals {
                result.push(i, col, -value);
            }
        }

        result
    }
}
