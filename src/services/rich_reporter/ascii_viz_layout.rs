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
        debug_assert!(width > 0, "width must be positive");
        self.h.to_string().repeat(width)
    }

    /// Draw a box around content
    pub fn draw_box(&self, content: &[&str], width: usize) -> String {
        debug_assert!(width > 0, "width must be positive");
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
        debug_assert!(width > 0, "width must be positive");
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
        debug_assert!(!text.is_empty(), "text must not be empty");
        format!("├── {}", text)
    }

    /// Render a tree item (last in group)
    pub fn last_branch(text: &str) -> String {
        debug_assert!(!text.is_empty(), "text must not be empty");
        format!("└── {}", text)
    }

    /// Render a continuation line
    pub fn continuation(text: &str) -> String {
        debug_assert!(!text.is_empty(), "text must not be empty");
        format!("│   {}", text)
    }

    /// Render empty continuation
    pub fn empty_continuation(text: &str) -> String {
        debug_assert!(!text.is_empty(), "text must not be empty");
        format!("    {}", text)
    }
}
