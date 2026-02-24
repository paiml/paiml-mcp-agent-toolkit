#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod comprehensive_coverage_tests_part2 {
    use super::*;

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
