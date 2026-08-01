#![cfg_attr(coverage_nightly, coverage(off))]
//! Shared box-drawing helpers for the TDG table renderers.
//!
//! The renderers used to hand-pad every row with a literal run of spaces, so
//! the right-hand border moved with the content: on one frame
//! `│  Overall Score: 99.5/100 (A+)                  │` and
//! `│  Total Files: 1593                               │` were emitted inside
//! the same fixed-width box. These helpers compute the padding from the row's
//! visible width instead.

/// Number of columns between the two vertical borders.
pub(crate) const INNER_WIDTH: usize = 49;

/// Terminal columns occupied by one char: 0 for a variation selector, 2 for
/// emoji / misc-symbol code points (which is why the emoji rows were padded by
/// hand), 1 otherwise. Box-drawing and block characters stay at 1.
pub(crate) fn char_width(ch: char) -> usize {
    match ch as u32 {
        0xFE00..=0xFE0F | 0x200D => 0,
        0x2600..=0x27BF | 0x2B00..=0x2BFF | 0x1F300..=0x1FAFF => 2,
        _ => 1,
    }
}

/// Visible width of `text`, ignoring ANSI SGR escape sequences (colour codes
/// are zero-width on screen but count as chars in `str::len`).
pub(crate) fn visible_width(text: &str) -> usize {
    let mut width = 0usize;
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            // Skip up to and including the terminating byte of the escape.
            for esc in chars.by_ref() {
                if esc.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        width += char_width(ch);
    }
    width
}

/// Top border of the frame.
pub(crate) fn box_top() -> String {
    format!("╭{}╮", "─".repeat(INNER_WIDTH))
}

/// Separator between header and body.
pub(crate) fn box_separator() -> String {
    format!("├{}┤", "─".repeat(INNER_WIDTH))
}

/// Bottom border of the frame.
pub(crate) fn box_bottom() -> String {
    format!("╰{}╯", "─".repeat(INNER_WIDTH))
}

/// Number of columns of body text a row can hold (frame minus the indent).
const LEAD: usize = 2;
pub(crate) const BODY_WIDTH: usize = INNER_WIDTH - LEAD;

/// Copy `content` until `budget` visible columns are used; ANSI escapes are
/// copied verbatim and cost nothing. Returns the text and the columns used.
///
/// If the text is cut short, a colour reset is appended — clipping away the
/// closing escape would leave the rest of the terminal painted.
fn clip_to_width(content: &str, budget: usize) -> (String, usize) {
    let mut kept = String::new();
    let mut used = 0usize;
    let mut saw_escape = false;
    let mut clipped = false;
    let mut chars = content.chars();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            saw_escape = true;
            kept.push(ch);
            for esc in chars.by_ref() {
                kept.push(esc);
                if esc.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        let w = char_width(ch);
        if used + w > budget {
            clipped = true;
            break;
        }
        kept.push(ch);
        used += w;
    }
    if clipped && saw_escape {
        kept.push_str("\u{1b}[0m");
    }
    (kept, used)
}

/// A body row: two leading spaces, `content`, then padding so the closing
/// border always lands in the same column. Over-long content is clipped
/// (never allowed to push the border out of line).
pub(crate) fn box_row(content: &str) -> String {
    let (text, used) = clip_to_width(content, BODY_WIDTH);
    format!(
        "│{}{}{}│",
        " ".repeat(LEAD),
        text,
        " ".repeat(BODY_WIDTH - used)
    )
}

/// Clip `content` to `budget` columns, marking the cut with `...` so a
/// shortened line is never mistaken for the whole text.
pub(crate) fn ellipsize(content: &str, budget: usize) -> String {
    if visible_width(content) <= budget {
        return content.to_string();
    }
    let (kept, _) = clip_to_width(content, budget.saturating_sub(3));
    format!("{kept}...")
}

/// An empty body row.
pub(crate) fn box_blank() -> String {
    box_row("")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every emitted line must be exactly as wide as the frame, whatever the
    /// content. Regression for the misaligned right-hand border, which moved
    /// with content width: `│  Overall Score: 99.5/100 (A+)                  │`
    /// and `│  Total Files: 1593                               │` came out of
    /// the same fixed-width frame.
    #[test]
    fn test_all_rows_share_one_width() {
        let lines = vec![
            box_top(),
            box_row("Overall Score: 99.5/100 (A+)"),
            box_separator(),
            box_row("Overall Score: 0.0/100 (F)"),
            box_row("Total Files: 1593"),
            box_row("📊 Breakdown:"),
            box_row("⚠️  Code needs improvement in several areas."),
            box_row("├─ Structural:     25.0/25  ██████████"),
            box_blank(),
            box_bottom(),
        ];
        let expected = INNER_WIDTH + 2;
        for line in &lines {
            assert_eq!(
                visible_width(line),
                expected,
                "misaligned row: {line:?} ({} cols)",
                visible_width(line)
            );
        }
    }

    #[test]
    fn test_ansi_colour_codes_do_not_shift_the_border() {
        let plain = box_row("Overall Score: 100.0/100 (A-)");
        let coloured =
            box_row("Overall Score: \u{1b}[1;37m100.0\u{1b}[0m/100 (\u{1b}[32mA-\u{1b}[0m)");
        assert_eq!(visible_width(&plain), visible_width(&coloured));
        assert!(plain.ends_with(" │"));
        assert!(coloured.ends_with(" │"));
    }

    #[test]
    fn test_overlong_content_is_truncated_not_overflowed() {
        let row = box_row(&"x".repeat(200));
        assert_eq!(visible_width(&row), INNER_WIDTH + 2);
    }

    #[test]
    fn test_overlong_wide_content_is_truncated_not_overflowed() {
        let row = box_row(&"📊".repeat(200));
        assert_eq!(visible_width(&row), INNER_WIDTH + 2);
    }

    /// Clipping must not swallow the closing colour reset, or every later line
    /// of the terminal stays painted.
    #[test]
    fn test_clipped_colour_is_reset_before_the_border() {
        let row = box_row(&format!("path: \u{1b}[36m{}\u{1b}[0m", "x".repeat(200)));
        assert_eq!(visible_width(&row), INNER_WIDTH + 2);
        assert!(
            row.contains("\u{1b}[0m"),
            "clipped row must still reset colour: {row:?}"
        );
    }
}
