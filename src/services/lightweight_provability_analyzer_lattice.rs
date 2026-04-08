// Lattice impl blocks for abstract domain types:
// PropertyDomain (top, join, widen, is_equal),
// NullabilityLattice (join), IntervalLattice (widen, is_equal),
// AliasLattice (join), PurityLattice (meet).

impl PropertyDomain {
    /// Create a top (unknown) property domain
    ///
    /// # Examples
    ///
    /// ```
    /// use pmat::services::lightweight_provability_analyzer::PropertyDomain;
    ///
    /// let domain = PropertyDomain::top();
    /// // All properties are initially unknown
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn top() -> Self {
        Self {
            nullability: NullabilityLattice::Top,
            bounds: IntervalLattice {
                lower: None,
                upper: None,
            },
            aliasing: AliasLattice::Top,
            purity: PurityLattice::Top,
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Join parts together.
    pub fn join(&self, other: &Self) -> Self {
        Self {
            nullability: self.nullability.join(&other.nullability),
            bounds: self.bounds.widen(&other.bounds),
            aliasing: self.aliasing.join(&other.aliasing),
            purity: self.purity.meet(&other.purity),
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Widen.
    pub fn widen(&self, other: &Self) -> Self {
        Self {
            nullability: self.nullability.join(&other.nullability),
            bounds: self.bounds.widen(&other.bounds),
            aliasing: self.aliasing.join(&other.aliasing),
            purity: self.purity.meet(&other.purity),
        }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Is equal.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.nullability == other.nullability
            && self.bounds.is_equal(&other.bounds)
            && self.aliasing == other.aliasing
            && self.purity == other.purity
    }
}

impl NullabilityLattice {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Join parts together.
    pub fn join(&self, other: &Self) -> Self {
        use NullabilityLattice::{Bottom, MaybeNull, NotNull, Null, Top};
        match (self, other) {
            (Bottom, x) | (x, Bottom) => x.clone(),
            (Top, _) | (_, Top) => Top,
            (NotNull, NotNull) => NotNull,
            (Null, Null) => Null,
            (NotNull, Null) | (Null, NotNull) => Bottom, // Contradiction
            _ => MaybeNull,
        }
    }
}

impl IntervalLattice {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Widen.
    pub fn widen(&self, other: &Self) -> Self {
        let lower = match (self.lower, other.lower) {
            (Some(a), Some(b)) if a > b => None, // Widening to -inf
            (l, _) => l,
        };

        let upper = match (self.upper, other.upper) {
            (Some(a), Some(b)) if a < b => None, // Widening to +inf
            (u, _) => u,
        };

        Self { lower, upper }
    }

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Is equal.
    pub fn is_equal(&self, other: &Self) -> bool {
        self.lower == other.lower && self.upper == other.upper
    }
}

impl AliasLattice {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Join parts together.
    pub fn join(&self, other: &Self) -> Self {
        use AliasLattice::{Bottom, MayAlias, MustAlias, NoAlias, Top};
        match (self, other) {
            (Bottom, x) | (x, Bottom) => x.clone(),
            (Top, _) | (_, Top) => Top,
            (NoAlias, NoAlias) => NoAlias,
            (MustAlias, MustAlias) => MustAlias,
            _ => MayAlias,
        }
    }
}

impl PurityLattice {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Meet.
    pub fn meet(&self, other: &Self) -> Self {
        use PurityLattice::{Bottom, Pure, ReadOnly, Top, WriteGlobal, WriteLocal};
        match (self, other) {
            (Bottom, _) | (_, Bottom) => Bottom,
            (WriteGlobal, _) | (_, WriteGlobal) => WriteGlobal,
            (WriteLocal, _) | (_, WriteLocal) => WriteLocal,
            (ReadOnly, ReadOnly) => ReadOnly,
            (Pure, Pure) => Pure,
            _ => Top,
        }
    }
}
