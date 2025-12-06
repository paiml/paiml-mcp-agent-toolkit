//! Dashboard State Management - Elm Architecture
//!
//! Implements unidirectional data flow:
//! Event -> Message -> State Update -> View Re-render

use serde::{Deserialize, Serialize};

/// Main dashboard state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DashboardState {
    /// System metrics (CPU, memory, connections)
    pub metrics: SystemMetrics,
    /// List of code hotspots
    pub hotspots: Vec<Hotspot>,
    /// Currently selected tab
    pub selected_tab: TabId,
    /// Layout configuration
    pub layout: LayoutConfig,
    /// Sort configuration for tables
    pub sort_column: Option<String>,
    pub sort_direction: SortDirection,
    /// Connection status
    connected: bool,
}

impl DashboardState {
    /// Check if connected to backend
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Set connection status
    pub fn set_connected(&mut self, connected: bool) {
        self.connected = connected;
    }

    /// Update state based on message (Elm architecture)
    pub fn update(&mut self, msg: DashboardMessage) -> Command {
        match msg {
            DashboardMessage::MetricsUpdated(metrics) => {
                self.metrics = metrics;
                Command::None
            }
            DashboardMessage::HotspotsUpdated(hotspots) => {
                self.hotspots = hotspots;
                Command::None
            }
            DashboardMessage::TabChanged(tab) => {
                self.selected_tab = tab;
                Command::None
            }
            DashboardMessage::SortChanged(column, direction) => {
                self.sort_column = Some(column);
                self.sort_direction = direction;
                Command::None
            }
            DashboardMessage::RefreshMetrics => {
                Command::Task(Box::new(|| {
                    // Async task to fetch metrics
                    DashboardMessage::MetricsUpdated(SystemMetrics::default())
                }))
            }
            DashboardMessage::Export(format) => {
                Command::Export { format }
            }
            DashboardMessage::ConnectionChanged(connected) => {
                self.connected = connected;
                Command::None
            }
        }
    }
}

/// Messages that can update dashboard state
#[derive(Debug, Clone)]
pub enum DashboardMessage {
    /// System metrics updated from backend
    MetricsUpdated(SystemMetrics),
    /// Hotspots list updated
    HotspotsUpdated(Vec<Hotspot>),
    /// User changed tab
    TabChanged(TabId),
    /// User changed sort order
    SortChanged(String, SortDirection),
    /// Request to refresh metrics
    RefreshMetrics,
    /// Export data in specified format
    Export(ExportFormat),
    /// WebSocket connection changed
    ConnectionChanged(bool),
}

/// Commands represent side effects (async tasks, exports, etc.)
pub enum Command {
    /// No side effect
    None,
    /// Batch of commands
    Batch(Vec<Command>),
    /// Async task that produces a message
    Task(Box<dyn FnOnce() -> DashboardMessage + Send>),
    /// Export data
    Export { format: ExportFormat },
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "Command::None"),
            Self::Batch(cmds) => write!(f, "Command::Batch({} commands)", cmds.len()),
            Self::Task(_) => write!(f, "Command::Task(<function>)"),
            Self::Export { format } => write!(f, "Command::Export {{ format: {:?} }}", format),
        }
    }
}

impl Command {
    /// Check if this is the None variant
    pub fn is_none(&self) -> bool {
        matches!(self, Command::None)
    }
}

/// System metrics from backend
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SystemMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_usage: f64,
    /// Memory usage percentage (0-100)
    pub memory_usage: f64,
    /// Number of active connections
    pub active_connections: u32,
}

/// Code hotspot data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hotspot {
    /// File path
    pub file: String,
    /// Cyclomatic complexity score
    pub complexity: u32,
    /// Git churn (number of changes)
    pub churn: u32,
    /// Combined hotspot score
    pub score: f64,
}

/// Dashboard tab identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TabId {
    #[default]
    Overview,
    Hotspots,
    Performance,
    Dag,
}

/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Number of grid columns
    pub columns: u32,
    /// Gap between grid items (pixels)
    pub gap: u32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            columns: 12,
            gap: 16,
        }
    }
}

/// Sort direction for tables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

/// Export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
}

/// Animation configuration for charts
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Spring stiffness (100-500 typical)
    pub stiffness: f32,
    /// Spring damping (10-50 typical)
    pub damping: f32,
    /// Mass (1.0 typical)
    pub mass: f32,
}

impl AnimationConfig {
    /// Create spring animation config
    pub fn spring(stiffness: f32, damping: f32) -> Self {
        Self {
            stiffness,
            damping,
            mass: 1.0,
        }
    }
}

/// Zoom configuration for diagrams
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ZoomConfig {
    /// Minimum zoom level
    pub min: f32,
    /// Maximum zoom level
    pub max: f32,
}

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Parse color from hex string
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Self { r, g, b, a: 255 })
        } else {
            None
        }
    }

    /// Get relative luminance for contrast calculations
    pub fn luminance(&self) -> f64 {
        fn srgb_to_linear(c: u8) -> f64 {
            let c = c as f64 / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * srgb_to_linear(self.r)
            + 0.7152 * srgb_to_linear(self.g)
            + 0.0722 * srgb_to_linear(self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_default() {
        let state = DashboardState::default();
        assert_eq!(state.layout.columns, 12);
        assert!(!state.is_connected());
    }

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#ff0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_animation_config() {
        let config = AnimationConfig::spring(100.0, 20.0);
        assert_eq!(config.stiffness, 100.0);
        assert_eq!(config.damping, 20.0);
        assert_eq!(config.mass, 1.0);
    }
}
