//! Accessibility Utilities - WCAG 2.1 AA Compliance
//!
//! Provides utilities for ensuring accessibility compliance:
//! - Color contrast ratio calculation (4.5:1 minimum)
//! - Focus indicator validation
//! - Screen reader support

use crate::state::Color;

/// Calculate contrast ratio between two colors (WCAG 2.1)
///
/// Returns a value between 1 and 21, where:
/// - 4.5:1 is minimum for normal text (AA)
/// - 3:1 is minimum for large text (AA)
/// - 7:1 is minimum for normal text (AAA)
pub fn contrast_ratio(fg: Color, bg: Color) -> f64 {
    let l1 = fg.luminance();
    let l2 = bg.luminance();

    let lighter = l1.max(l2);
    let darker = l1.min(l2);

    (lighter + 0.05) / (darker + 0.05)
}

/// Check if contrast meets WCAG 2.1 AA for normal text
pub fn meets_aa_normal_text(fg: Color, bg: Color) -> bool {
    contrast_ratio(fg, bg) >= 4.5
}

/// Check if contrast meets WCAG 2.1 AA for large text
pub fn meets_aa_large_text(fg: Color, bg: Color) -> bool {
    contrast_ratio(fg, bg) >= 3.0
}

/// Check if contrast meets WCAG 2.1 AAA for normal text
pub fn meets_aaa_normal_text(fg: Color, bg: Color) -> bool {
    contrast_ratio(fg, bg) >= 7.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrast_white_on_black() {
        let white = Color::from_hex("#ffffff").unwrap();
        let black = Color::from_hex("#000000").unwrap();
        let ratio = contrast_ratio(white, black);
        assert!(
            ratio > 20.0,
            "White on black should have ~21:1 ratio, got {}",
            ratio
        );
    }

    #[test]
    fn test_contrast_dashboard_theme() {
        // Dashboard dark theme colors
        let fg = Color::from_hex("#ffffff").unwrap();
        let bg = Color::from_hex("#1a1a2e").unwrap();
        let ratio = contrast_ratio(fg, bg);
        assert!(
            ratio >= 4.5,
            "Dashboard theme contrast {} below 4.5:1",
            ratio
        );
    }

    #[test]
    fn test_meets_aa_functions() {
        let white = Color::from_hex("#ffffff").unwrap();
        let dark_bg = Color::from_hex("#1a1a2e").unwrap();

        assert!(meets_aa_normal_text(white, dark_bg));
        assert!(meets_aa_large_text(white, dark_bg));
    }

    #[test]
    fn test_poor_contrast_fails() {
        let light_gray = Color::from_hex("#cccccc").unwrap();
        let white = Color::from_hex("#ffffff").unwrap();

        assert!(!meets_aa_normal_text(light_gray, white));
    }
}
