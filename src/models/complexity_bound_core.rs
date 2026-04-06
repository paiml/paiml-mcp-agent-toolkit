// Implementations for core complexity types: ComplexityBound, CacheComplexity,
// RecurrenceRelation, and their Default/Display trait implementations.

impl ComplexityBound {
    /// Create a new complexity bound
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(class: BigOClass, coefficient: u16, input_var: InputVariable) -> Self {
        Self {
            class,
            coefficient,
            input_var,
            confidence: 50,
            flags: ComplexityFlags::new().with(ComplexityFlags::WORST_CASE),
            _padding: [0; 2],
        }
    }

    /// Create a constant time bound O(1)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::{ComplexityBound, BigOClass};
    ///
    /// let bound = ComplexityBound::constant();
    /// assert_eq!(bound.class, BigOClass::Constant);
    /// assert_eq!(bound.confidence, 100);
    /// assert_eq!(bound.notation(), "O(1)");
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn constant() -> Self {
        Self::new(BigOClass::Constant, 1, InputVariable::N)
            .with_confidence(100)
            .with_flags(ComplexityFlags::PROVEN)
    }

    /// Create a linear time bound O(n)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::{ComplexityBound, BigOClass};
    ///
    /// let bound = ComplexityBound::linear();
    /// assert_eq!(bound.class, BigOClass::Linear);
    /// assert_eq!(bound.notation(), "O(n)");
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn linear() -> Self {
        Self::new(BigOClass::Linear, 1, InputVariable::N)
    }

    /// Create a quadratic time bound O(n^2)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::{ComplexityBound, BigOClass};
    ///
    /// let bound = ComplexityBound::quadratic();
    /// assert_eq!(bound.class, BigOClass::Quadratic);
    /// assert_eq!(bound.notation(), "O(n²)");
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn quadratic() -> Self {
        Self::new(BigOClass::Quadratic, 1, InputVariable::N)
    }

    /// Create a logarithmic time bound O(log n)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::{ComplexityBound, BigOClass};
    ///
    /// let bound = ComplexityBound::logarithmic();
    /// assert_eq!(bound.class, BigOClass::Logarithmic);
    /// assert_eq!(bound.notation(), "O(log n)");
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn logarithmic() -> Self {
        Self::new(BigOClass::Logarithmic, 1, InputVariable::N)
    }

    /// Create a linearithmic time bound O(n log n)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::{ComplexityBound, BigOClass};
    ///
    /// let bound = ComplexityBound::linearithmic();
    /// assert_eq!(bound.class, BigOClass::Linearithmic);
    /// assert_eq!(bound.notation(), "O(n log n)");
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn linearithmic() -> Self {
        Self::new(BigOClass::Linearithmic, 1, InputVariable::N)
    }

    /// Create a polynomial bound with given exponent
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::{ComplexityBound, BigOClass};
    ///
    /// let constant = ComplexityBound::polynomial(0, 5);
    /// assert_eq!(constant.class, BigOClass::Constant);
    ///
    /// let cubic = ComplexityBound::polynomial(3, 2);
    /// assert_eq!(cubic.class, BigOClass::Cubic);
    /// assert_eq!(cubic.coefficient, 2);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn polynomial(exponent: u32, coefficient: u16) -> Self {
        let class = match exponent {
            0 => BigOClass::Constant,
            1 => BigOClass::Linear,
            2 => BigOClass::Quadratic,
            3 => BigOClass::Cubic,
            _ => BigOClass::Unknown,
        };
        Self::new(class, coefficient, InputVariable::N)
    }

    /// Create a polynomial-logarithmic bound
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn polynomial_log(degree: u32, log_power: u32) -> Self {
        match (degree, log_power) {
            (1, 1) => Self::linearithmic(),
            _ => Self::unknown(), // More complex cases need custom representation
        }
    }

    /// Create an unknown complexity bound
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::{ComplexityBound, BigOClass};
    ///
    /// let unknown = ComplexityBound::unknown();
    /// assert_eq!(unknown.class, BigOClass::Unknown);
    /// assert_eq!(unknown.confidence, 0);
    /// assert_eq!(unknown.notation(), "O(?)");
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn unknown() -> Self {
        Self::new(BigOClass::Unknown, 0, InputVariable::N).with_confidence(0)
    }

    /// Set confidence level (0-100)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::ComplexityBound;
    ///
    /// let bound = ComplexityBound::linear()
    ///     .with_confidence(85);
    /// assert_eq!(bound.confidence, 85);
    ///
    /// // Values over 100 are clamped
    /// let clamped = ComplexityBound::linear()
    ///     .with_confidence(150);
    /// assert_eq!(clamped.confidence, 100);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.confidence = confidence.min(100);
        self
    }

    /// Add flags to the bound
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::{ComplexityBound, ComplexityFlags};
    ///
    /// let bound = ComplexityBound::linear()
    ///     .with_flags(ComplexityFlags::PROVEN | ComplexityFlags::TIGHT_BOUND);
    /// assert!(bound.flags.has(ComplexityFlags::PROVEN));
    /// assert!(bound.flags.has(ComplexityFlags::TIGHT_BOUND));
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_flags(mut self, flags: u8) -> Self {
        self.flags = self.flags.with(flags);
        self
    }

    /// Get notation string for this bound
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::{ComplexityBound, BigOClass, InputVariable};
    ///
    /// let simple = ComplexityBound::linear();
    /// assert_eq!(simple.notation(), "O(n)");
    ///
    /// let complex = ComplexityBound::new(BigOClass::Linear, 5, InputVariable::N);
    /// assert_eq!(complex.notation(), "5·O(n)");
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn notation(&self) -> String {
        if self.coefficient <= 1 {
            format!("{}", self.class)
        } else {
            format!("{}·{}", self.coefficient, self.class)
        }
    }

    /// Estimate operations for given input size
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::ComplexityBound;
    ///
    /// let linear = ComplexityBound::linear();
    /// assert_eq!(linear.estimate_operations(100.0), 100.0);
    ///
    /// let quadratic = ComplexityBound::quadratic();
    /// assert_eq!(quadratic.estimate_operations(10.0), 100.0);
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn estimate_operations(&self, n: f64) -> f64 {
        f64::from(self.coefficient) * self.class.growth_factor(n)
    }

    /// Check if this bound is better than another
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pmat::models::complexity_bound::{ComplexityBound, BigOClass, InputVariable};
    ///
    /// let linear = ComplexityBound::linear();
    /// let quadratic = ComplexityBound::quadratic();
    /// assert!(linear.is_better_than(&quadratic));
    ///
    /// // Same class but different coefficients
    /// let fast = ComplexityBound::new(BigOClass::Linear, 2, InputVariable::N);
    /// let slow = ComplexityBound::new(BigOClass::Linear, 5, InputVariable::N);
    /// assert!(fast.is_better_than(&slow));
    /// ```
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_better_than(&self, other: &Self) -> bool {
        if self.class == other.class {
            self.coefficient < other.coefficient
        } else {
            self.class.is_better_than(&other.class)
        }
    }
}

impl Default for ComplexityBound {
    fn default() -> Self {
        Self::unknown()
    }
}

impl fmt::Display for ComplexityBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}% confidence)", self.notation(), self.confidence)
    }
}

impl CacheComplexity {
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(hit_ratio: u8, miss_penalty: u8, working_set: BigOClass) -> Self {
        Self {
            hit_ratio: hit_ratio.min(100),
            miss_penalty,
            working_set,
            flags: 0,
            _padding: [0; 4],
        }
    }
}

impl Default for CacheComplexity {
    fn default() -> Self {
        Self::new(0, 1, BigOClass::Unknown)
    }
}

impl RecurrenceRelation {
    /// Attempt to solve using the Master Theorem
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn solve_master_theorem(&self) -> Option<ComplexityBound> {
        // Check if recurrence fits Master Theorem form: T(n) = aT(n/b) + f(n)
        if self.recursive_calls.len() != 1 {
            return None;
        }

        let call = &self.recursive_calls[0];
        if call.size_reduction != 0 || call.division_factor <= 1 {
            return None;
        }

        let a = call.count;
        let b = call.division_factor;
        let work = &self.work_per_call;

        // Apply Master Theorem based on work complexity class
        match work.class {
            BigOClass::Constant => {
                // T(n) = aT(n/b) + O(1)
                let log_b_a = f64::from(a).log(f64::from(b));
                Some(ComplexityBound::polynomial(log_b_a.ceil() as u32, 1))
            }
            BigOClass::Linear => {
                // T(n) = aT(n/b) + O(n)
                let log_b_a = f64::from(a).log(f64::from(b));
                if (log_b_a - 1.0).abs() < 0.01 {
                    // Case 2: a = b
                    Some(ComplexityBound::linearithmic())
                } else if log_b_a < 1.0 {
                    // Case 3: a < b
                    Some(ComplexityBound::linear())
                } else {
                    // Case 1: a > b
                    Some(ComplexityBound::polynomial(log_b_a.ceil() as u32, 1))
                }
            }
            _ => None,
        }
    }
}
