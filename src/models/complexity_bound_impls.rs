// Implementations for supporting complexity types: BigOClass, InputVariable,
// ComplexityFlags, and their Display trait implementations.

impl BigOClass {
    /// Get a human-readable notation for the complexity class
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::BigOClass;
    ///
    /// assert_eq!(BigOClass::Constant.notation(), "O(1)");
    /// assert_eq!(BigOClass::Linear.notation(), "O(n)");
    /// assert_eq!(BigOClass::Quadratic.notation(), "O(n²)");
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn notation(&self) -> &'static str {
        match self {
            Self::Constant => "O(1)",
            Self::Logarithmic => "O(log n)",
            Self::Linear => "O(n)",
            Self::Linearithmic => "O(n log n)",
            Self::Quadratic => "O(n²)",
            Self::Cubic => "O(n³)",
            Self::Exponential => "O(2^n)",
            Self::Factorial => "O(n!)",
            Self::Unknown => "O(?)",
        }
    }

    /// Check if this complexity is better than another
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::BigOClass;
    ///
    /// assert!(BigOClass::Constant.is_better_than(&BigOClass::Linear));
    /// assert!(BigOClass::Linear.is_better_than(&BigOClass::Quadratic));
    /// assert!(!BigOClass::Quadratic.is_better_than(&BigOClass::Linear));
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_better_than(&self, other: &Self) -> bool {
        (*self as u8) < (*other as u8)
    }

    /// Get approximate growth factor for small n
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::BigOClass;
    ///
    /// assert_eq!(BigOClass::Constant.growth_factor(100.0), 1.0);
    /// assert_eq!(BigOClass::Linear.growth_factor(100.0), 100.0);
    /// assert_eq!(BigOClass::Quadratic.growth_factor(10.0), 100.0);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn growth_factor(&self, n: f64) -> f64 {
        match self {
            Self::Constant => 1.0,
            Self::Logarithmic => n.log2(),
            Self::Linear => n,
            Self::Linearithmic => n * n.log2(),
            Self::Quadratic => n * n,
            Self::Cubic => n * n * n,
            Self::Exponential => 2.0_f64.powf(n),
            Self::Factorial => {
                // Stirling's approximation for large n
                if n <= 20.0 {
                    (1..=n as u32).map(f64::from).product()
                } else {
                    (2.0 * std::f64::consts::PI * n).sqrt() * (n / std::f64::consts::E).powf(n)
                }
            }
            Self::Unknown => f64::NAN,
        }
    }
}

impl fmt::Display for BigOClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.notation())
    }
}

impl fmt::Display for InputVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::N => write!(f, "n"),
            Self::M => write!(f, "m"),
            Self::K => write!(f, "k"),
            Self::D => write!(f, "d"),
            Self::Custom => write!(f, "x"),
        }
    }
}

impl ComplexityFlags {
    pub const AMORTIZED: u8 = 0b00000001;
    pub const WORST_CASE: u8 = 0b00000010;
    pub const AVERAGE_CASE: u8 = 0b00000100;
    pub const BEST_CASE: u8 = 0b00001000;
    pub const TIGHT_BOUND: u8 = 0b00010000;
    pub const EMPIRICAL: u8 = 0b00100000;
    pub const PROVEN: u8 = 0b01000000;
    pub const RECURSIVE: u8 = 0b10000000;

    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Create a new instance.
    pub fn new() -> Self {
        Self(0)
    }

    /// Adds a flag to the complexity flags
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::ComplexityFlags;
    ///
    /// let flags = ComplexityFlags::new()
    ///     .with(ComplexityFlags::WORST_CASE)
    ///     .with(ComplexityFlags::PROVEN);
    /// assert!(flags.has(ComplexityFlags::WORST_CASE));
    /// assert!(flags.has(ComplexityFlags::PROVEN));
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with(mut self, flag: u8) -> Self {
        self.0 |= flag;
        self
    }

    /// Checks if a flag is set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::ComplexityFlags;
    ///
    /// let flags = ComplexityFlags::new().with(ComplexityFlags::AMORTIZED);
    /// assert!(flags.has(ComplexityFlags::AMORTIZED));
    /// assert!(!flags.has(ComplexityFlags::WORST_CASE));
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn has(&self, flag: u8) -> bool {
        self.0 & flag != 0
    }

    /// Checks if this represents worst-case complexity
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::ComplexityFlags;
    ///
    /// let flags = ComplexityFlags::new()
    ///     .with(ComplexityFlags::WORST_CASE);
    /// assert!(flags.is_worst_case());
    ///
    /// let avg_case = ComplexityFlags::new()
    ///     .with(ComplexityFlags::AVERAGE_CASE);
    /// assert!(!avg_case.is_worst_case());
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_worst_case(&self) -> bool {
        self.has(Self::WORST_CASE)
    }

    /// Checks if this complexity has been formally proven
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::ComplexityFlags;
    ///
    /// let proven = ComplexityFlags::new()
    ///     .with(ComplexityFlags::PROVEN);
    /// assert!(proven.is_proven());
    ///
    /// let empirical = ComplexityFlags::new()
    ///     .with(ComplexityFlags::EMPIRICAL);
    /// assert!(!empirical.is_proven());
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_proven(&self) -> bool {
        self.has(Self::PROVEN)
    }
}
