#![cfg_attr(coverage_nightly, coverage(off))]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(
    Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum Grade {
    APLus,
    A,
    AMinus,
    BPlus,
    B,
    BMinus,
    CPlus,
    #[default]
    C,
    CMinus,
    D,
    F,
}

impl Grade {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 95.0 => Grade::APLus,
            s if s >= 90.0 => Grade::A,
            s if s >= 85.0 => Grade::AMinus,
            s if s >= 80.0 => Grade::BPlus,
            s if s >= 75.0 => Grade::B,
            s if s >= 70.0 => Grade::BMinus,
            s if s >= 65.0 => Grade::CPlus,
            s if s >= 60.0 => Grade::C,
            s if s >= 55.0 => Grade::CMinus,
            s if s >= 50.0 => Grade::D,
            _ => Grade::F,
        }
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::APLus => write!(f, "A+"),
            Grade::A => write!(f, "A"),
            Grade::AMinus => write!(f, "A-"),
            Grade::BPlus => write!(f, "B+"),
            Grade::B => write!(f, "B"),
            Grade::BMinus => write!(f, "B-"),
            Grade::CPlus => write!(f, "C+"),
            Grade::C => write!(f, "C"),
            Grade::CMinus => write!(f, "C-"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MetricCategory {
    StructuralComplexity,
    SemanticComplexity,
    Duplication,
    Coupling,
    Documentation,
    Consistency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PenaltyAttribution {
    pub source_metric: MetricCategory,
    pub amount: f32,
    pub applied_to: HashSet<MetricCategory>,
    pub issue: String,
}
