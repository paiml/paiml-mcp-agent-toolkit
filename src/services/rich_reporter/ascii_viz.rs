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

#[cfg(test)]
mod comprehensive_coverage_tests {
    use super::*;

    // ==================== ProgressBar Tests ====================

    #[test]
    fn test_progress_bar_default() {
        let bar = ProgressBar::default();
        assert_eq!(bar.width, 20);
        assert_eq!(bar.fill_char, '█');
        assert_eq!(bar.empty_char, '░');
        assert!(!bar.use_color);
    }

    #[test]
    fn test_progress_bar_with_color() {
        let bar = ProgressBar::new(10).with_color();
        assert!(bar.use_color);
    }

    #[test]
    fn test_progress_bar_render_with_percent() {
        let bar = ProgressBar::new(10);
        let result = bar.render_with_percent(0.75);
        assert!(result.contains("75%"));
        assert!(result.contains('['));
        assert!(result.contains(']'));
    }

    #[test]
    fn test_progress_bar_render_with_percent_zero() {
        let bar = ProgressBar::new(10);
        let result = bar.render_with_percent(0.0);
        assert!(result.contains("0%"));
    }

    #[test]
    fn test_progress_bar_render_with_percent_full() {
        let bar = ProgressBar::new(10);
        let result = bar.render_with_percent(1.0);
        assert!(result.contains("100%"));
    }

    #[test]
    fn test_progress_bar_render_segmented_no_color() {
        let bar = ProgressBar::new(10);
        let thresholds = vec![
            (0.3, Severity::Low),
            (0.6, Severity::Medium),
            (1.0, Severity::Critical),
        ];
        let result = bar.render_segmented(0.5, &thresholds);
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
        // Without color, should just be filled/empty chars
        assert!(result.contains('█'));
        assert!(result.contains('░'));
    }

    #[test]
    fn test_progress_bar_render_segmented_with_color() {
        let bar = ProgressBar::new(10).with_color();
        let thresholds = vec![
            (0.3, Severity::Low),
            (0.6, Severity::Medium),
            (1.0, Severity::Critical),
        ];
        let result = bar.render_segmented(0.5, &thresholds);
        // With color, should contain ANSI codes
        assert!(result.contains("\x1b[0m"));
    }

    #[test]
    fn test_progress_bar_render_segmented_empty() {
        let bar = ProgressBar::new(10);
        let result = bar.render_segmented(0.0, &[]);
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
    }

    #[test]
    fn test_progress_bar_render_segmented_full() {
        let bar = ProgressBar::new(10).with_color();
        let thresholds = vec![(1.0, Severity::High)];
        let result = bar.render_segmented(1.0, &thresholds);
        assert!(result.contains('█'));
    }

    #[test]
    fn test_progress_bar_various_widths() {
        for width in [1, 5, 20, 50, 100] {
            let bar = ProgressBar::new(width);
            let result = bar.render(0.5);
            // Total length = width + 2 (for '[' and ']')
            assert_eq!(result.chars().count(), width + 2);
        }
    }

    #[test]
    fn test_progress_bar_render_small_fractions() {
        let bar = ProgressBar::new(10);
        let result = bar.render(0.05);
        // 5% of 10 = 0.5, rounds to 1
        assert!(result.contains('█'));
    }

    // ==================== Sparkline Tests ====================

    #[test]
    fn test_sparkline_default() {
        let spark = Sparkline::default();
        assert_eq!(spark.chars.len(), 8);
        assert_eq!(spark.chars[0], '▁');
        assert_eq!(spark.chars[7], '█');
    }

    #[test]
    fn test_sparkline_render_empty() {
        let spark = Sparkline::default();
        assert_eq!(spark.render(&[]), "");
    }

    #[test]
    fn test_sparkline_render_clamping() {
        let spark = Sparkline::default();
        // Values > 7 should be clamped to 7
        let result = spark.render(&[10, 15, 255]);
        assert_eq!(result.chars().count(), 3);
        for c in result.chars() {
            assert_eq!(c, '█'); // All should be max
        }
    }

    #[test]
    fn test_sparkline_auto_empty() {
        let spark = Sparkline::default();
        assert_eq!(spark.render_auto(&[]), "");
    }

    #[test]
    fn test_sparkline_auto_single_value() {
        let spark = Sparkline::default();
        let result = spark.render_auto(&[42.0]);
        // Single value means range=0, should use middle char
        assert_eq!(result.chars().count(), 1);
    }

    #[test]
    fn test_sparkline_auto_same_values() {
        let spark = Sparkline::default();
        let result = spark.render_auto(&[5.0, 5.0, 5.0, 5.0]);
        // All same values = range 0, should repeat middle char
        assert_eq!(result.chars().count(), 4);
        // All chars should be the same (index 4 = '▅')
        let chars: Vec<char> = result.chars().collect();
        assert!(chars.iter().all(|&c| c == chars[0]));
    }

    #[test]
    fn test_sparkline_auto_negative_values() {
        let spark = Sparkline::default();
        let result = spark.render_auto(&[-100.0, 0.0, 100.0]);
        assert_eq!(result.chars().count(), 3);
        let chars: Vec<char> = result.chars().collect();
        assert_eq!(chars[0], '▁'); // min
        assert_eq!(chars[2], '█'); // max
    }

    #[test]
    fn test_sparkline_render_with_trend() {
        let spark = Sparkline::default();
        let result = spark.render_with_trend(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!(result.contains('↓')); // Degrading trend arrow (down = bad)
    }

    #[test]
    fn test_sparkline_render_with_trend_improving() {
        let spark = Sparkline::default();
        let result = spark.render_with_trend(&[5.0, 4.0, 3.0, 2.0, 1.0]);
        assert!(result.contains('↑')); // Improving trend arrow (up = good)
    }

    #[test]
    fn test_sparkline_render_with_trend_stable() {
        let spark = Sparkline::default();
        let result = spark.render_with_trend(&[5.0, 5.0, 5.0, 5.0]);
        assert!(result.contains('→')); // Stable trend arrow
    }

    #[test]
    fn test_sparkline_trend_detection_single_value() {
        // Less than 2 values = Stable
        assert_eq!(Sparkline::detect_trend(&[1.0]), TrendDirection::Stable);
        assert_eq!(Sparkline::detect_trend(&[]), TrendDirection::Stable);
    }

    #[test]
    fn test_sparkline_trend_detection_two_values() {
        // Two values - should still work
        assert_eq!(
            Sparkline::detect_trend(&[1.0, 10.0]),
            TrendDirection::Degrading
        );
        assert_eq!(
            Sparkline::detect_trend(&[10.0, 1.0]),
            TrendDirection::Improving
        );
    }

    #[test]
    fn test_sparkline_trend_detection_near_zero_mean() {
        // Values around zero with increasing trend
        // Note: When y_mean is near zero, threshold is also near zero,
        // so any non-zero slope causes Degrading/Improving
        let values = vec![-0.001, 0.0, 0.001];
        let result = Sparkline::detect_trend(&values);
        // Increasing values (even tiny) = Degrading trend
        assert_eq!(result, TrendDirection::Degrading);
    }

    // ==================== BoxDrawer Tests ====================

    #[test]
    fn test_box_drawer_default() {
        let drawer = BoxDrawer::default();
        assert_eq!(drawer.tl, '┌');
        assert_eq!(drawer.tr, '┐');
        assert_eq!(drawer.bl, '└');
        assert_eq!(drawer.br, '┘');
        assert_eq!(drawer.h, '─');
        assert_eq!(drawer.v, '│');
    }

    #[test]
    fn test_box_drawer_double() {
        let drawer = BoxDrawer::double();
        assert_eq!(drawer.tl, '╔');
        assert_eq!(drawer.tr, '╗');
        assert_eq!(drawer.bl, '╚');
        assert_eq!(drawer.br, '╝');
        assert_eq!(drawer.h, '═');
        assert_eq!(drawer.v, '║');
        assert_eq!(drawer.cross, '╬');
    }

    #[test]
    fn test_box_drawer_horizontal_zero() {
        let drawer = BoxDrawer::default();
        assert_eq!(drawer.horizontal(0), "");
    }

    #[test]
    fn test_box_drawer_horizontal_large() {
        let drawer = BoxDrawer::default();
        let result = drawer.horizontal(100);
        assert_eq!(result.chars().count(), 100);
        assert!(result.chars().all(|c| c == '─'));
    }

    #[test]
    fn test_box_drawer_draw_box_empty() {
        let drawer = BoxDrawer::default();
        let result = drawer.draw_box(&[], 10);
        assert!(result.contains('┌'));
        assert!(result.contains('┐'));
        assert!(result.contains('└'));
        assert!(result.contains('┘'));
    }

    #[test]
    fn test_box_drawer_draw_box_single_line() {
        let drawer = BoxDrawer::default();
        let result = drawer.draw_box(&["Hello"], 10);
        assert!(result.contains("Hello"));
        assert!(result.contains('│'));
    }

    #[test]
    fn test_box_drawer_draw_box_multiple_lines() {
        let drawer = BoxDrawer::default();
        let result = drawer.draw_box(&["Line 1", "Line 2", "Line 3"], 15);
        assert!(result.contains("Line 1"));
        assert!(result.contains("Line 2"));
        assert!(result.contains("Line 3"));
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 5); // top + 3 content + bottom
    }

    #[test]
    fn test_box_drawer_draw_box_long_content() {
        let drawer = BoxDrawer::default();
        // Content longer than width
        let result = drawer.draw_box(&["This is a very long line"], 5);
        assert!(result.contains('│'));
    }

    #[test]
    fn test_box_drawer_section_header() {
        let drawer = BoxDrawer::default();
        let result = drawer.section_header("Test Section", 30);
        assert!(result.contains("Test Section"));
        assert!(result.contains('─'));
    }

    #[test]
    fn test_box_drawer_section_header_short_width() {
        let drawer = BoxDrawer::default();
        // Width smaller than title
        let result = drawer.section_header("Very Long Title", 10);
        assert!(result.contains("Very Long Title"));
    }

    #[test]
    fn test_box_drawer_double_draw_box() {
        let drawer = BoxDrawer::double();
        let result = drawer.draw_box(&["Emphasis"], 15);
        assert!(result.contains('╔'));
        assert!(result.contains('╗'));
        assert!(result.contains('║'));
    }

    // ==================== TableRenderer Tests ====================

    #[test]
    fn test_table_renderer_new() {
        let table = TableRenderer::new(vec![10, 20, 30]);
        assert_eq!(table.widths.len(), 3);
        assert_eq!(table.alignments.len(), 3);
        assert!(table.alignments.iter().all(|&a| !a)); // all left-aligned by default
    }

    #[test]
    fn test_table_renderer_with_alignments() {
        let table = TableRenderer::new(vec![10, 10, 10]).with_alignments(vec![false, true, false]);
        assert!(!table.alignments[0]);
        assert!(table.alignments[1]); // right-aligned
        assert!(!table.alignments[2]);
    }

    #[test]
    fn test_table_renderer_render_row() {
        let table = TableRenderer::new(vec![10, 8]);
        let row = table.render_row(&["Cell1", "Cell2"]);
        assert!(row.contains("Cell1"));
        assert!(row.contains("Cell2"));
        assert!(row.contains('│'));
    }

    #[test]
    fn test_table_renderer_render_row_right_aligned() {
        let table = TableRenderer::new(vec![10, 10]).with_alignments(vec![false, true]);
        let row = table.render_row(&["Left", "Right"]);
        // Right-aligned cell should have padding before text
        assert!(row.contains("Right"));
    }

    #[test]
    fn test_table_renderer_render_row_long_content() {
        let table = TableRenderer::new(vec![5, 5]);
        let row = table.render_row(&["TooLongText", "Short"]);
        // Should truncate long content
        assert!(!row.contains("TooLongText"));
        assert!(row.contains("TooLo")); // truncated
    }

    #[test]
    fn test_table_renderer_render_footer() {
        let table = TableRenderer::new(vec![10, 10]);
        let footer = table.render_footer();
        assert!(footer.contains('└'));
        assert!(footer.contains('┘'));
        assert!(footer.contains('┴'));
    }

    #[test]
    fn test_table_renderer_single_column() {
        let table = TableRenderer::new(vec![15]);
        let header = table.render_header(&["Column"]);
        let row = table.render_row(&["Data"]);
        let footer = table.render_footer();
        assert!(header.contains("Column"));
        assert!(row.contains("Data"));
        assert!(footer.contains('─'));
    }

    #[test]
    fn test_table_renderer_many_columns() {
        let table = TableRenderer::new(vec![5, 5, 5, 5, 5]);
        let header = table.render_header(&["A", "B", "C", "D", "E"]);
        let row = table.render_row(&["1", "2", "3", "4", "5"]);
        assert!(header.contains('┬')); // column separators
        assert!(row.contains('│'));
    }

    #[test]
    fn test_table_renderer_empty_cells() {
        let table = TableRenderer::new(vec![10, 10]);
        let row = table.render_row(&["", ""]);
        assert!(row.contains('│'));
    }

    #[test]
    fn test_table_renderer_full_workflow() {
        let table = TableRenderer::new(vec![12, 8, 6]).with_alignments(vec![false, true, false]);
        let header = table.render_header(&["Filename", "Size", "OK"]);
        let row1 = table.render_row(&["main.rs", "1024", "✓"]);
        let row2 = table.render_row(&["lib.rs", "512", "✓"]);
        let footer = table.render_footer();

        let full_table = format!("{}\n{}\n{}\n{}", header, row1, row2, footer);
        assert!(full_table.contains("Filename"));
        assert!(full_table.contains("main.rs"));
        assert!(full_table.contains("lib.rs"));
        assert!(full_table.contains('┌'));
        assert!(full_table.contains('└'));
    }

    // ==================== TreeRenderer Tests ====================

    #[test]
    fn test_tree_renderer_branch() {
        let result = TreeRenderer::branch("item");
        assert_eq!(result, "├── item");
    }

    #[test]
    fn test_tree_renderer_last_branch() {
        let result = TreeRenderer::last_branch("item");
        assert_eq!(result, "└── item");
    }

    #[test]
    fn test_tree_renderer_continuation() {
        let result = TreeRenderer::continuation("child");
        assert_eq!(result, "│   child");
    }

    #[test]
    fn test_tree_renderer_empty_continuation() {
        let result = TreeRenderer::empty_continuation("orphan");
        assert_eq!(result, "    orphan");
    }

    #[test]
    fn test_tree_renderer_nested_structure() {
        let mut tree = String::new();
        tree.push_str(&TreeRenderer::branch("parent1"));
        tree.push('\n');
        tree.push_str(&TreeRenderer::continuation(&TreeRenderer::branch("child1")));
        tree.push('\n');
        tree.push_str(&TreeRenderer::continuation(&TreeRenderer::last_branch(
            "child2",
        )));
        tree.push('\n');
        tree.push_str(&TreeRenderer::last_branch("parent2"));

        assert!(tree.contains("├──"));
        assert!(tree.contains("└──"));
        assert!(tree.contains("│"));
    }

    #[test]
    fn test_tree_renderer_empty_text() {
        assert_eq!(TreeRenderer::branch(""), "├── ");
        assert_eq!(TreeRenderer::last_branch(""), "└── ");
        assert_eq!(TreeRenderer::continuation(""), "│   ");
        assert_eq!(TreeRenderer::empty_continuation(""), "    ");
    }

    // ==================== StatusIndicator Tests ====================

    #[test]
    fn test_status_indicator_pending() {
        assert_eq!(StatusIndicator::pending(), "◷");
    }

    #[test]
    fn test_status_indicator_info() {
        assert_eq!(StatusIndicator::info(), "ℹ");
    }

    #[test]
    fn test_all_status_indicators() {
        let indicators = [
            StatusIndicator::pass(),
            StatusIndicator::fail(),
            StatusIndicator::warning(),
            StatusIndicator::pending(),
            StatusIndicator::info(),
        ];
        // All should be non-empty and unique
        for indicator in &indicators {
            assert!(!indicator.is_empty());
        }
        // Check uniqueness
        let mut unique: Vec<&str> = indicators.to_vec();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), indicators.len());
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_progress_bar_width_zero() {
        let bar = ProgressBar::new(0);
        let result = bar.render(0.5);
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_progress_bar_render_nan() {
        let bar = ProgressBar::new(10);
        // NaN gets clamped via clamp()
        let result = bar.render(f64::NAN);
        // NaN.clamp() returns the lower bound (0.0)
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
    }

    #[test]
    fn test_sparkline_render_all_zeros() {
        let spark = Sparkline::default();
        let result = spark.render(&[0, 0, 0, 0]);
        assert_eq!(result.chars().count(), 4);
        assert!(result.chars().all(|c| c == '▁'));
    }

    #[test]
    fn test_sparkline_render_all_sevens() {
        let spark = Sparkline::default();
        let result = spark.render(&[7, 7, 7, 7]);
        assert_eq!(result.chars().count(), 4);
        assert!(result.chars().all(|c| c == '█'));
    }

    #[test]
    fn test_box_drawer_all_junctions() {
        let drawer = BoxDrawer::default();
        // Verify all junction characters exist
        assert_ne!(drawer.t_left, '\0');
        assert_ne!(drawer.t_right, '\0');
        assert_ne!(drawer.t_top, '\0');
        assert_ne!(drawer.t_bottom, '\0');
        assert_ne!(drawer.cross, '\0');
    }
}
