//! Allocation and formatting of document serial numbers.

use anyhow::{Context, Result, bail};

/// A serial such as `PAR/NDA/2026/0001`.
///
/// Serials are per document code and per year, so the NDA and the IP
/// assignment issued to the same person on the same day do not share a
/// number, and the sequence restarts cleanly each January.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Serial(String);

impl Serial {
    /// Render a serial from the format string in `org.yaml`.
    ///
    /// The format is data rather than code because the shape of a reference
    /// number is a business decision. Only the three documented placeholders
    /// are substituted; anything else is a configuration error and is
    /// reported rather than silently left in the serial.
    ///
    /// # Errors
    /// Fails if the format contains an unknown placeholder.
    pub fn render(format: &str, code: &str, year: i32, seq: u32) -> Result<Self> {
        let rendered = format
            .replace("{kind}", code)
            .replace("{year}", &year.to_string())
            .replace("{seq:04d}", &format!("{seq:04}"));
        if rendered.contains('{') {
            bail!(
                "serial format {format:?} contains an unknown placeholder; \
                 only {{kind}}, {{year}} and {{seq:04d}} are substituted"
            );
        }
        Ok(Self(rendered))
    }

    /// The serial as it appears on the document.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// A filesystem-safe form, since a serial contains `/`.
    #[must_use]
    pub fn file_stem(&self) -> String {
        self.0.replace('/', "-")
    }
}

impl std::fmt::Display for Serial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The next sequence number for a code and year, given those already used.
///
/// Takes the maximum rather than the count: if an entry is ever removed, the
/// next issuance must still not reuse a number that has been on a document.
#[must_use]
pub fn next_seq(used: &[u32]) -> u32 {
    used.iter()
        .copied()
        .max()
        .map_or(1, |m| m.saturating_add(1))
}

/// A short code a third party can check a document against.
///
/// Printed ON the document, so it cannot be a hash of the finished PDF --
/// that would be circular. It is derived from the facts the register holds
/// about the issuance instead: serial, recipient, date, and the hash of the
/// template as issued. Anyone holding the paper can quote the serial and
/// this code; Paridyum recomputes it from the register and they agree.
///
/// Twelve hex characters. It is a lookup check, not a signature: it proves
/// the document matches a register entry, not that the holder is the person
/// named. Forgery resistance comes from the register being private.
#[must_use]
pub fn verification_code(
    serial: &str,
    slug: &str,
    issued_on: &str,
    template_sha256: &str,
) -> String {
    let material = format!("{serial}|{slug}|{issued_on}|{template_sha256}");
    crate::domain::chain::sha256(material.as_bytes())
        .get(..12)
        .unwrap_or_default()
        .to_uppercase()
}

/// Parse a year from an ISO date, for register grouping.
///
/// # Errors
/// Fails if the date does not begin with a four-digit year.
pub fn year_of(iso_date: &str) -> Result<i32> {
    iso_date
        .split('-')
        .next()
        .context("empty date")?
        .parse::<i32>()
        .with_context(|| format!("date {iso_date:?} does not start with a year"))
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{Serial, next_seq, year_of};

    #[test]
    fn renders_the_documented_format() {
        let s = Serial::render("PAR/{kind}/{year}/{seq:04d}", "NDA", 2026, 7)
            .expect("documented format renders");
        assert_eq!(s.as_str(), "PAR/NDA/2026/0007");
    }

    #[test]
    fn unknown_placeholder_is_an_error_not_a_serial() {
        // A serial reading "PAR/NDA/{month}/0001" on a signed certificate is
        // worse than a failed build.
        assert!(Serial::render("PAR/{kind}/{month}", "NDA", 2026, 1).is_err());
    }

    #[test]
    fn file_stem_has_no_path_separators() {
        let s = Serial::render("PAR/{kind}/{year}/{seq:04d}", "IPA", 2026, 1).expect("renders");
        assert_eq!(s.file_stem(), "PAR-IPA-2026-0001");
    }

    #[test]
    fn first_sequence_is_one() {
        assert_eq!(next_seq(&[]), 1);
    }

    #[test]
    fn sequence_never_reuses_a_removed_number() {
        assert_eq!(next_seq(&[1, 2, 3]), 4);
        assert_eq!(next_seq(&[1, 3]), 4, "must not fill the gap left by 2");
    }

    #[test]
    fn a_verification_code_is_stable_for_the_same_issuance() {
        let a = super::verification_code("PAR/CRT/2026/0001", "asha", "2026-08-19", "abc");
        let b = super::verification_code("PAR/CRT/2026/0001", "asha", "2026-08-19", "abc");
        assert_eq!(a, b);
        assert_eq!(a.len(), 12);
        assert!(
            a.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_lowercase())
        );
    }

    #[test]
    fn a_verification_code_changes_with_every_input() {
        let base = super::verification_code("PAR/CRT/2026/0001", "asha", "2026-08-19", "abc");
        for other in [
            super::verification_code("PAR/CRT/2026/0002", "asha", "2026-08-19", "abc"),
            super::verification_code("PAR/CRT/2026/0001", "priya", "2026-08-19", "abc"),
            super::verification_code("PAR/CRT/2026/0001", "asha", "2026-08-20", "abc"),
            super::verification_code("PAR/CRT/2026/0001", "asha", "2026-08-19", "def"),
        ] {
            assert_ne!(base, other, "every input must change the code");
        }
    }

    #[test]
    fn year_is_read_from_an_iso_date() {
        assert_eq!(year_of("2026-08-19").expect("valid"), 2026);
        assert!(year_of("not-a-date").is_err());
    }
}
