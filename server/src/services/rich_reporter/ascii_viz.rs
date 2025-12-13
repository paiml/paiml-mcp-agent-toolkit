//! ASCII Visualization Primitives for PMAT-REPORT-V1
//!
//! Implements Toyota Way Mieruka (Visual Management) through:
//! - Progress bars with thresholds
//! - Box drawing for structured output
//! - Sparklines for trends
//! - Tables for data presentation

use super::types::{Severity, TrendDirection};

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
    pub fn new(width: usize) -> Self {
        ProgressBar {
            width,
            ..Default::default()
        }
    }

    /// Enable color output
    pub fn with_color(mut self) -> Self {
        self.use_color = true;
        self
    }

    /// Render progress bar for a value (0.0 - 1.0)
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
    pub fn render_with_percent(&self, value: f64) -> String {
        let rendered = self.render(value);
        format!("{} {:>3.0}%", rendered, value * 100.0)
    }

    /// Render segmented progress bar with thresholds
    /// Thresholds are pairs of (value, color) for different zones
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
    pub fn render(&self, values: &[u8]) -> String {
        values
            .iter()
            .map(|&v| self.chars[(v.min(7)) as usize])
            .collect()
    }

    /// Render sparkline from raw f64 values (auto-normalize)
    pub fn render_auto(&self, values: &[f64]) -> String {
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
    pub fn render_with_trend(&self, values: &[f64]) -> String {
        let sparkline = self.render_auto(values);
        let direction = Self::detect_trend(values);
        format!("{} {}", sparkline, direction.arrow())
    }

    /// Detect trend direction from values
    fn detect_trend(values: &[f64]) -> TrendDirection {
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

/// Box drawing for structured output
pub struct BoxDrawer {
    /// Top-left corner
    tl: char,
    /// Top-right corner
    tr: char,
    /// Bottom-left corner
    bl: char,
    /// Bottom-right corner
    br: char,
    /// Horizontal line
    h: char,
    /// Vertical line
    v: char,
    /// T-junction left
    t_left: char,
    /// T-junction right
    t_right: char,
    /// T-junction top
    t_top: char,
    /// T-junction bottom
    t_bottom: char,
    /// Cross
    cross: char,
}

impl Default for BoxDrawer {
    fn default() -> Self {
        // Single-line box drawing characters
        BoxDrawer {
            tl: '┌',
            tr: '┐',
            bl: '└',
            br: '┘',
            h: '─',
            v: '│',
            t_left: '├',
            t_right: '┤',
            t_top: '┬',
            t_bottom: '┴',
            cross: '┼',
        }
    }
}

impl BoxDrawer {
    /// Create a double-line box drawer (for emphasis)
    pub fn double() -> Self {
        BoxDrawer {
            tl: '╔',
            tr: '╗',
            bl: '╚',
            br: '╝',
            h: '═',
            v: '║',
            t_left: '╠',
            t_right: '╣',
            t_top: '╦',
            t_bottom: '╩',
            cross: '╬',
        }
    }

    /// Draw a horizontal line
    pub fn horizontal(&self, width: usize) -> String {
        self.h.to_string().repeat(width)
    }

    /// Draw a box around content
    pub fn draw_box(&self, content: &[&str], width: usize) -> String {
        let mut lines = Vec::new();

        // Top border
        lines.push(format!("{}{}{}", self.tl, self.horizontal(width), self.tr));

        // Content lines
        for line in content {
            let padding = width.saturating_sub(line.chars().count());
            lines.push(format!(
                "{} {}{} {}",
                self.v,
                line,
                " ".repeat(padding.saturating_sub(2)),
                self.v
            ));
        }

        // Bottom border
        lines.push(format!("{}{}{}", self.bl, self.horizontal(width), self.br));

        lines.join("\n")
    }

    /// Draw a section header
    pub fn section_header(&self, title: &str, width: usize) -> String {
        let dash_count = width.saturating_sub(title.len() + 2);
        format!(
            "{} {} {}",
            self.horizontal(2),
            title,
            self.horizontal(dash_count)
        )
    }
}

/// ASCII table renderer
pub struct TableRenderer {
    /// Column widths
    widths: Vec<usize>,
    /// Column alignments (true = right, false = left)
    alignments: Vec<bool>,
    /// Use box drawing characters (reserved for future use)
    #[allow(dead_code)]
    use_box_chars: bool,
}

impl TableRenderer {
    /// Create a new table renderer with column widths
    pub fn new(widths: Vec<usize>) -> Self {
        let alignments = vec![false; widths.len()];
        TableRenderer {
            widths,
            alignments,
            use_box_chars: true,
        }
    }

    /// Set column alignment (true = right, false = left)
    pub fn with_alignments(mut self, alignments: Vec<bool>) -> Self {
        self.alignments = alignments;
        self
    }

    /// Render a header row
    pub fn render_header(&self, headers: &[&str]) -> String {
        let box_drawer = BoxDrawer::default();
        let mut lines = Vec::new();

        // Top border
        let top: String = self
            .widths
            .iter()
            .map(|&w| box_drawer.horizontal(w + 2))
            .collect::<Vec<_>>()
            .join(&box_drawer.t_top.to_string());
        lines.push(format!("{}{}{}", box_drawer.tl, top, box_drawer.tr));

        // Header row
        let header_cells: String = headers
            .iter()
            .zip(&self.widths)
            .map(|(h, &w)| {
                let truncated: String = h.chars().take(w).collect();
                let padding = w.saturating_sub(truncated.chars().count());
                format!(" {}{} ", truncated, " ".repeat(padding))
            })
            .collect::<Vec<_>>()
            .join(&box_drawer.v.to_string());
        lines.push(format!("{}{}{}", box_drawer.v, header_cells, box_drawer.v));

        // Separator
        let sep: String = self
            .widths
            .iter()
            .map(|&w| box_drawer.horizontal(w + 2))
            .collect::<Vec<_>>()
            .join(&box_drawer.cross.to_string());
        lines.push(format!(
            "{}{}{}",
            box_drawer.t_left, sep, box_drawer.t_right
        ));

        lines.join("\n")
    }

    /// Render a data row
    pub fn render_row(&self, cells: &[&str]) -> String {
        let box_drawer = BoxDrawer::default();

        let cell_strings: String = cells
            .iter()
            .zip(&self.widths)
            .zip(&self.alignments)
            .map(|((c, &w), &right_align)| {
                let truncated: String = c.chars().take(w).collect();
                let padding = w.saturating_sub(truncated.chars().count());
                if right_align {
                    format!(" {}{} ", " ".repeat(padding), truncated)
                } else {
                    format!(" {}{} ", truncated, " ".repeat(padding))
                }
            })
            .collect::<Vec<_>>()
            .join(&box_drawer.v.to_string());

        format!("{}{}{}", box_drawer.v, cell_strings, box_drawer.v)
    }

    /// Render the table footer
    pub fn render_footer(&self) -> String {
        let box_drawer = BoxDrawer::default();

        let bottom: String = self
            .widths
            .iter()
            .map(|&w| box_drawer.horizontal(w + 2))
            .collect::<Vec<_>>()
            .join(&box_drawer.t_bottom.to_string());

        format!("{}{}{}", box_drawer.bl, bottom, box_drawer.br)
    }
}

/// Tree renderer for hierarchical data
pub struct TreeRenderer;

impl TreeRenderer {
    /// Render a tree item (not last in group)
    pub fn branch(text: &str) -> String {
        format!("├── {}", text)
    }

    /// Render a tree item (last in group)
    pub fn last_branch(text: &str) -> String {
        format!("└── {}", text)
    }

    /// Render a continuation line
    pub fn continuation(text: &str) -> String {
        format!("│   {}", text)
    }

    /// Render empty continuation
    pub fn empty_continuation(text: &str) -> String {
        format!("    {}", text)
    }
}

/// Status indicators
pub struct StatusIndicator;

impl StatusIndicator {
    /// Render a pass indicator
    pub fn pass() -> &'static str {
        "✓"
    }

    /// Render a fail indicator
    pub fn fail() -> &'static str {
        "✗"
    }

    /// Render a warning indicator
    pub fn warning() -> &'static str {
        "⚠"
    }

    /// Render a pending indicator
    pub fn pending() -> &'static str {
        "◷"
    }

    /// Render an info indicator
    pub fn info() -> &'static str {
        "ℹ"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_render() {
        let bar = ProgressBar::new(10);
        assert_eq!(bar.render(0.0), "[░░░░░░░░░░]");
        assert_eq!(bar.render(0.5), "[█████░░░░░]");
        assert_eq!(bar.render(1.0), "[██████████]");
    }

    #[test]
    fn test_progress_bar_clamp() {
        let bar = ProgressBar::new(10);
        assert_eq!(bar.render(-0.5), "[░░░░░░░░░░]");
        assert_eq!(bar.render(1.5), "[██████████]");
    }

    #[test]
    fn test_sparkline_render() {
        let spark = Sparkline::default();
        assert_eq!(spark.render(&[0, 7]), "▁█");
        assert_eq!(spark.render(&[0, 1, 2, 3, 4, 5, 6, 7]), "▁▂▃▄▅▆▇█");
    }

    #[test]
    fn test_sparkline_auto() {
        let spark = Sparkline::default();
        let values = vec![0.0, 50.0, 100.0];
        let result = spark.render_auto(&values);
        assert_eq!(result.chars().count(), 3);
        assert!(result.starts_with('▁'));
        assert!(result.ends_with('█'));
    }

    #[test]
    fn test_sparkline_trend_detection() {
        assert_eq!(
            Sparkline::detect_trend(&[1.0, 2.0, 3.0, 4.0, 5.0]),
            TrendDirection::Degrading
        );
        assert_eq!(
            Sparkline::detect_trend(&[5.0, 4.0, 3.0, 2.0, 1.0]),
            TrendDirection::Improving
        );
        assert_eq!(
            Sparkline::detect_trend(&[1.0, 1.0, 1.0, 1.0]),
            TrendDirection::Stable
        );
    }

    #[test]
    fn test_box_drawer_horizontal() {
        let drawer = BoxDrawer::default();
        assert_eq!(drawer.horizontal(5), "─────");
    }

    #[test]
    fn test_table_renderer_header() {
        let table = TableRenderer::new(vec![10, 8, 6]);
        let header = table.render_header(&["File", "Score", "Status"]);
        assert!(header.contains("File"));
        assert!(header.contains("Score"));
        assert!(header.contains('┌'));
        assert!(header.contains('┐'));
    }

    #[test]
    fn test_status_indicators() {
        assert_eq!(StatusIndicator::pass(), "✓");
        assert_eq!(StatusIndicator::fail(), "✗");
        assert_eq!(StatusIndicator::warning(), "⚠");
    }
}
