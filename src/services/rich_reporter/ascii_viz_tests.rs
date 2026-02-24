#[cfg_attr(coverage_nightly, coverage(off))]
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

#[cfg_attr(coverage_nightly, coverage(off))]
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
}
