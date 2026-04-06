#![cfg_attr(coverage_nightly, coverage(off))]
//! Graph metric types, DAG types, and deep context enums

use clap::ValueEnum;
use std::fmt;

/// Graph metric type
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum GraphMetricType {
    /// Degree centrality
    Centrality,
    /// Betweenness centrality
    Betweenness,
    /// Closeness centrality
    Closeness,
    /// `PageRank` scores
    PageRank,
    /// Clustering coefficient
    Clustering,
    /// Connected components analysis
    Components,
    /// All available metrics
    All,
}

impl GraphMetricType {
    /// Get the string representation of the graph metric type
    fn as_str(&self) -> &'static str {
        debug_assert!(true, "contract: as_str");
        match self {
            GraphMetricType::Centrality => "centrality",
            GraphMetricType::Betweenness => "betweenness",
            GraphMetricType::Closeness => "closeness",
            GraphMetricType::PageRank => "pagerank",
            GraphMetricType::Clustering => "clustering",
            GraphMetricType::Components => "components",
            GraphMetricType::All => "all",
        }
    }
}

impl fmt::Display for GraphMetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// DAG generation type
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq, Hash)]
pub enum DagType {
    /// Function call graph
    #[value(name = "call-graph")]
    CallGraph,

    /// Import/dependency graph
    #[value(name = "import-graph")]
    ImportGraph,

    /// Class inheritance hierarchy
    #[value(name = "inheritance")]
    Inheritance,

    /// Complete dependency graph
    #[value(name = "full-dependency")]
    FullDependency,
}

impl fmt::Display for DagType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DagType::CallGraph => write!(f, "call-graph"),
            DagType::ImportGraph => write!(f, "import-graph"),
            DagType::Inheritance => write!(f, "inheritance"),
            DagType::FullDependency => write!(f, "full-dependency"),
        }
    }
}

/// Deep context output format
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextOutputFormat {
    Markdown,
    Json,
    Sarif,
}

impl fmt::Display for DeepContextOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeepContextOutputFormat::Markdown => write!(f, "markdown"),
            DeepContextOutputFormat::Json => write!(f, "json"),
            DeepContextOutputFormat::Sarif => write!(f, "sarif"),
        }
    }
}

/// Deep context DAG type
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextDagType {
    #[value(name = "call-graph")]
    CallGraph,
    #[value(name = "import-graph")]
    ImportGraph,
    #[value(name = "inheritance")]
    Inheritance,
    #[value(name = "full-dependency")]
    FullDependency,
}

impl fmt::Display for DeepContextDagType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeepContextDagType::CallGraph => write!(f, "call-graph"),
            DeepContextDagType::ImportGraph => write!(f, "import-graph"),
            DeepContextDagType::Inheritance => write!(f, "inheritance"),
            DeepContextDagType::FullDependency => write!(f, "full-dependency"),
        }
    }
}

/// Deep context cache strategy
#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum DeepContextCacheStrategy {
    Normal,
    #[value(name = "force-refresh")]
    ForceRefresh,
    Offline,
}

impl fmt::Display for DeepContextCacheStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeepContextCacheStrategy::Normal => write!(f, "normal"),
            DeepContextCacheStrategy::ForceRefresh => write!(f, "force-refresh"),
            DeepContextCacheStrategy::Offline => write!(f, "offline"),
        }
    }
}
