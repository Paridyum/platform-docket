//! Supplying today's date, injectably.

use anyhow::{Context, Result};
use time::{Date, OffsetDateTime, format_description::FormatItem, macros::format_description};

const ISO: &[FormatItem<'_>] = format_description!("[year]-[month]-[day]");

/// A source of the current date.
///
/// A trait rather than a direct call because STD-0001 requires deterministic
/// tests, and `clippy.toml` forbids `SystemTime::now` for the same reason: a
/// document whose content depends on when the test ran cannot be asserted on.
pub trait Clock {
    /// Today, as the issuing date of a document.
    ///
    /// # Errors
    /// Fails if the platform clock cannot be read.
    fn today(&self) -> Result<Date>;
}

/// The real clock. Used by the binary and by nothing else.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    ///
    /// # Errors
    /// Fails if the platform clock cannot be read.
    fn today(&self) -> Result<Date> {
        // Falls back to UTC when the platform will not give a local offset,
        // which is sound here: the date is used for a serial and a printed
        // date, and refusing to issue over a timezone lookup would be worse.
        Ok(OffsetDateTime::now_local()
            .unwrap_or_else(|_| OffsetDateTime::now_utc())
            .date())
    }
}

/// A clock pinned to one date, for tests.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock(pub Date);

impl Clock for FixedClock {
    ///
    /// # Errors
    /// Fails if the platform clock cannot be read.
    fn today(&self) -> Result<Date> {
        Ok(self.0)
    }
}

/// Parse an ISO `YYYY-MM-DD` date.
///
/// # Errors
/// Fails when `text` is not `YYYY-MM-DD`.
pub fn parse_iso(text: &str) -> Result<Date> {
    Date::parse(text, &ISO).with_context(|| format!("{text:?} is not an ISO date (YYYY-MM-DD)"))
}

/// Format a date as ISO `YYYY-MM-DD`, the form the register stores.
///
/// # Errors
/// Fails if the date cannot be formatted.
pub fn to_iso(date: Date) -> Result<String> {
    date.format(&ISO).context("formatting a date as ISO")
}

/// Format a date as `19 August 2026`, the form the documents print.
#[must_use]
pub fn to_long(date: Date) -> String {
    format!("{} {} {}", date.day(), date.month(), date.year())
}

/// Seconds since the Unix epoch at midnight UTC on `date`.
///
/// Used for `SOURCE_DATE_EPOCH`, which is what makes a rendered PDF
/// byte-reproducible: without it pdfTeX stamps the wall clock into the file.
#[must_use]
pub fn epoch_seconds(date: Date) -> i64 {
    date.midnight().assume_utc().unix_timestamp()
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{Clock, FixedClock, epoch_seconds, parse_iso, to_iso, to_long};

    #[test]
    fn iso_round_trips() {
        let d = parse_iso("2026-08-19").expect("valid");
        assert_eq!(to_iso(d).expect("formats"), "2026-08-19");
    }

    #[test]
    fn long_form_matches_what_the_documents_print() {
        assert_eq!(
            to_long(parse_iso("2026-08-19").expect("valid")),
            "19 August 2026"
        );
        assert_eq!(
            to_long(parse_iso("2026-09-01").expect("valid")),
            "1 September 2026"
        );
    }

    #[test]
    fn a_malformed_date_is_rejected() {
        assert!(parse_iso("19-08-2026").is_err());
        assert!(parse_iso("not a date").is_err());
    }

    #[test]
    fn epoch_is_midnight_utc() {
        assert_eq!(epoch_seconds(parse_iso("1970-01-01").expect("valid")), 0);
        assert_eq!(
            epoch_seconds(parse_iso("2026-08-19").expect("valid")),
            1_787_097_600
        );
    }

    #[test]
    fn a_fixed_clock_does_not_move() {
        let c = FixedClock(parse_iso("2026-08-19").expect("valid"));
        assert_eq!(c.today().expect("fixed"), c.today().expect("fixed"));
    }
}
