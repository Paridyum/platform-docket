//! Drawing a full-screen view on a plain terminal.

use anyhow::Result;
use std::io::Write;

/// Clear the screen and put the cursor at the top left.
///
/// ANSI escapes and nothing else. A terminal UI library would mean fourteen
/// transitive crates for a read-only view, and reading a single keypress --
/// the one thing that genuinely needs a library -- requires raw mode, which
/// requires `termios`, which requires `unsafe`. `unsafe_code` is forbidden
/// crate-wide, so navigation is line-based and the dependency is unnecessary
/// rather than merely avoided.
///
/// # Errors
/// Fails if the terminal cannot be written.
pub fn clear(out: &mut impl Write) -> Result<()> {
    // 2J clears the screen, H homes the cursor.
    write!(out, "\x1b[2J\x1b[H")?;
    Ok(())
}

/// A heading with a rule under it, sized to the content.
///
/// # Errors
/// Fails if the terminal cannot be written.
pub fn heading(out: &mut impl Write, text: &str) -> Result<()> {
    writeln!(out, "{text}")?;
    writeln!(out, "{}", "-".repeat(text.chars().count()))?;
    Ok(())
}

/// Truncate to `width` display columns, with an ellipsis when it does not fit.
///
/// Counts characters rather than bytes, so a name outside ASCII does not get
/// cut mid-codepoint and does not throw the column alignment out.
#[must_use]
pub fn fit(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_owned();
    }
    let keep = width.saturating_sub(1);
    let mut out: String = text.chars().take(keep).collect();
    out.push('…');
    out
}

/// Pad to `width` display columns.
#[must_use]
pub fn pad(text: &str, width: usize) -> String {
    let shown = fit(text, width);
    let used = shown.chars().count();
    format!("{shown}{}", " ".repeat(width.saturating_sub(used)))
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{clear, fit, heading, pad};

    #[test]
    fn clearing_emits_the_two_escapes_and_nothing_else() {
        let mut out = Vec::new();
        clear(&mut out).expect("writes");
        assert_eq!(String::from_utf8_lossy(&out), "\x1b[2J\x1b[H");
    }

    #[test]
    fn a_heading_is_ruled_to_its_own_width() {
        let mut out = Vec::new();
        heading(&mut out, "Roster").expect("writes");
        assert_eq!(String::from_utf8_lossy(&out), "Roster\n------\n");
    }

    #[test]
    fn fitting_leaves_short_text_alone() {
        assert_eq!(fit("Asha", 10), "Asha");
        assert_eq!(fit("Asha", 4), "Asha");
    }

    #[test]
    fn fitting_marks_what_it_cut() {
        assert_eq!(fit("Asha Verma", 6), "Asha …");
    }

    #[test]
    fn fitting_counts_characters_not_bytes() {
        // A multi-byte name must not be cut mid-codepoint, and must not throw
        // the column alignment out by counting bytes.
        let name = "Ruhaní Khuraná";
        assert_eq!(fit(name, 8).chars().count(), 8);
        assert_eq!(pad(name, 20).chars().count(), 20);
        assert_eq!(pad("Asha", 8), "Asha    ");
    }

    #[test]
    fn padding_never_shrinks_below_the_requested_width() {
        assert_eq!(pad("", 3), "   ");
        assert_eq!(pad("toolong", 3).chars().count(), 3);
    }
}
