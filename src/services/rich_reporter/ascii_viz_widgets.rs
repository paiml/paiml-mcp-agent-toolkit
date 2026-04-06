/// Progress bar renderer
pub struct ProgressBar {
    /// Total width in characters
    width: usize,
    /// Filled character
    fill_char: char,
    /// Empty character
    empty_char: char,
    /// Use color
    use_color: bool,
}

impl Default for ProgressBar {
    fn default() -> Self {
        ProgressBar {
            width: 20,
            fill_char: '█',
            empty_char: '░',
            use_color: false,
        }
    }
}

impl ProgressBar {
    /// Create a new progress bar with specified width
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn new(width: usize) -> Self {
        debug_assert!(width > 0, "width must be positive");
        ProgressBar {
            width,
            ..Default::default()
        }
    }

    /// Enable color output
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn with_color(mut self) -> Self {
        self.use_color = true;
        self
    }

    /// Render progress bar for a value (0.0 - 1.0)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn render(&self, value: f64) -> String {
        let clamped = value.clamp(0.0, 1.0);
        let filled = (clamped * self.width as f64).round() as usize;
        let empty = self.width.saturating_sub(filled);

        format!(
            "[{}{}]",
            self.fill_char.to_string().repeat(filled),
            self.empty_char.to_string().repeat(empty)
        )
    }

    /// Render progress bar with percentage
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn render_with_percent(&self, value: f64) -> String {
        let rendered = self.render(value);
        format!("{} {:>3.0}%", rendered, value * 100.0)
    }

    /// Render segmented progress bar with thresholds
    /// Thresholds are pairs of (value, color) for different zones
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn render_segmented(&self, value: f64, thresholds: &[(f64, Severity)]) -> String {
        let clamped = value.clamp(0.0, 1.0);
        let filled = (clamped * self.width as f64).round() as usize;

        let mut result = String::with_capacity(self.width + 10);
        result.push('[');

        for i in 0..self.width {
            let pos = i as f64 / self.width as f64;
            let char_to_use = if i < filled {
                self.fill_char
            } else {
                self.empty_char
            };

            if self.use_color && i < filled {
                // Find the right color for this position
                let severity = thresholds
                    .iter()
                    .filter(|(t, _)| pos < *t)
                    .map(|(_, s)| s)
                    .next()
                    .unwrap_or(&Severity::Low);

                result.push_str(severity.color_code());
                result.push(char_to_use);
                result.push_str("\x1b[0m");
            } else {
                result.push(char_to_use);
            }
        }

        result.push(']');
        result
    }
}

/// Sparkline renderer for trend visualization
pub struct Sparkline {
    /// Characters for 8 levels (0-7)
    chars: [char; 8],
}

impl Default for Sparkline {
    fn default() -> Self {
        Sparkline {
            chars: ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'],
        }
    }
}

impl Sparkline {
    /// Render sparkline from normalized values (0-7)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn render(&self, values: &[u8]) -> String {
        debug_assert!(!values.is_empty(), "values must not be empty");
        values
            .iter()
            .map(|&v| self.chars[(v.min(7)) as usize])
            .collect()
    }

    /// Render sparkline from raw f64 values (auto-normalize)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn render_auto(&self, values: &[f64]) -> String {
        debug_assert!(!values.is_empty(), "values must not be empty");
        if values.is_empty() {
            return String::new();
        }

        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let range = max - min;

        if range == 0.0 {
            return self.chars[4].to_string().repeat(values.len());
        }

        let normalized: Vec<u8> = values
            .iter()
            .map(|&v| ((v - min) / range * 7.0).round() as u8)
            .collect();

        self.render(&normalized)
    }

    /// Render sparkline with trend indicator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn render_with_trend(&self, values: &[f64]) -> String {
        debug_assert!(!values.is_empty(), "values must not be empty");
        let sparkline = self.render_auto(values);
        let direction = Self::detect_trend(values);
        format!("{} {}", sparkline, direction.arrow())
    }

    /// Detect trend direction from values
    fn detect_trend(values: &[f64]) -> TrendDirection {
        debug_assert!(!values.is_empty(), "values must not be empty");
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        // Simple linear regression slope
        let n = values.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean: f64 = values.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for (i, &y) in values.iter().enumerate() {
            let x = i as f64;
            numerator += (x - x_mean) * (y - y_mean);
            denominator += (x - x_mean) * (x - x_mean);
        }

        if denominator == 0.0 {
            return TrendDirection::Stable;
        }

        let slope = numerator / denominator;

        // Threshold for stability (5% of mean)
        let threshold = y_mean.abs() * 0.05;

        if slope > threshold {
            TrendDirection::Degrading
        } else if slope < -threshold {
            TrendDirection::Improving
        } else {
            TrendDirection::Stable
        }
    }
}

/// Status indicators
pub struct StatusIndicator;

impl StatusIndicator {
    /// Render a pass indicator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn pass() -> &'static str {
        "✓"
    }

    /// Render a fail indicator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn fail() -> &'static str {
        "✗"
    }

    /// Render a warning indicator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn warning() -> &'static str {
        "⚠"
    }

    /// Render a pending indicator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn pending() -> &'static str {
        "◷"
    }

    /// Render an info indicator
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn info() -> &'static str {
        "ℹ"
    }
}
