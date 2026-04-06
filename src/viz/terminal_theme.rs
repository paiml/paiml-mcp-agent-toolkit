// terminal_theme.rs - Theme colors, node shapes, and config builder methods
// Included by terminal.rs - NO use imports, NO #! inner attributes

impl TerminalTheme {
    /// Get critical node color for this theme
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn critical_color(&self) -> Rgba {
        match self {
            Self::Default => Rgba::new(255, 87, 34, 255), // Deep Orange
            Self::HighContrast => Rgba::new(255, 255, 0, 255), // Yellow
            Self::Light => Rgba::new(244, 67, 54, 255),   // Red
            Self::ColorblindSafe => Rgba::new(230, 159, 0, 255), // Orange (Okabe-Ito)
        }
    }

    /// Get normal node color for this theme
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn normal_color(&self) -> Rgba {
        match self {
            Self::Default => Rgba::new(66, 133, 244, 255), // Blue
            Self::HighContrast => Rgba::new(255, 255, 255, 255), // White
            Self::Light => Rgba::new(33, 150, 243, 255),   // Light Blue
            Self::ColorblindSafe => Rgba::new(0, 114, 178, 255), // Blue (Okabe-Ito)
        }
    }

    /// Get edge color for this theme
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn edge_color(&self) -> Rgba {
        match self {
            Self::Default => Rgba::new(150, 150, 150, 180),
            Self::HighContrast => Rgba::new(200, 200, 200, 255),
            Self::Light => Rgba::new(100, 100, 100, 180),
            Self::ColorblindSafe => Rgba::new(150, 150, 150, 180),
        }
    }

    /// Get background color for this theme
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn background_color(&self) -> Rgba {
        match self {
            Self::Default | Self::HighContrast | Self::ColorblindSafe => Rgba::BLACK,
            Self::Light => Rgba::WHITE,
        }
    }
}

impl NodeShape {
    /// Get the radius multiplier for this shape
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn radius_multiplier(&self) -> f32 {
        match self {
            Self::Circle => 1.0,
            Self::Square => 1.2,
            Self::Diamond => 1.3,
            Self::Triangle => 1.1,
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            theme: TerminalTheme::Default,
            mode: TerminalMode::UnicodeHalfBlock,
            iterations: 100,
            critical_threshold: 0.1,
            max_nodes: 50, // Semantic zooming: limit for readability
            show_labels: true,
        }
    }
}

impl RenderConfig {
    /// Create config with ASCII mode (widest compatibility)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn ascii() -> Self {
        Self {
            mode: TerminalMode::Ascii,
            ..Self::default()
        }
    }

    /// Create config with ANSI true color mode
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn ansi_color() -> Self {
        Self {
            mode: TerminalMode::AnsiTrueColor,
            ..Self::default()
        }
    }

    /// Set theme
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn theme(mut self, theme: TerminalTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Set max nodes (semantic zooming)
    #[must_use]
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn max_nodes(mut self, max: usize) -> Self {
        self.max_nodes = max;
        self
    }
}

impl batuta_common::display::WithDimensions for RenderConfig {
    fn set_dimensions(&mut self, width: u32, height: u32) {
        debug_assert!(width > 0, "width must be positive");
        debug_assert!(height > 0, "height must be positive");
        self.width = width;
        self.height = height;
    }
}
