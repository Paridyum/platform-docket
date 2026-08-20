//! Allocating the identifiers that name a person and an engagement.

use crate::domain::chain::Chained;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// A person's permanent identifier, e.g. `PAR-P-000147-3`.
///
/// **Opaque by decision (ADR-0051).** It encodes no fact about the person —
/// not their class, not their year, not their name — because an identifier
/// that carries a mutable fact stops being true. The predictable case is an
/// intern who is hired: an identifier saying `INT` would either lie forever
/// or be replaced, splitting the archive.
///
/// This is the only string used to name a folder or a directory outside this
/// system. A folder named after a person discloses who Paridyum engages to
/// anyone who can read the listing; this does not.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PersonId(String);

/// Width of the opaque sequence inside a person identifier.
const PERSON_DIGITS: usize = 6;

impl PersonId {
    /// Allocate from an opaque sequence, appending a check character.
    ///
    /// # Errors
    /// Fails if `seq` will not fit in six digits. Running out of numbers must
    /// be an error, never a silently wider identifier that breaks every
    /// consumer expecting a fixed shape.
    pub fn new(seq: u32) -> Result<Self> {
        let body = format!("{seq:0PERSON_DIGITS$}");
        if body.len() > PERSON_DIGITS {
            bail!("person sequence {seq} exceeds {PERSON_DIGITS} digits");
        }
        let check = check_character(&body);
        Ok(Self(format!("PAR-P-{body}-{check}")))
    }

    /// Parse and validate an identifier, check character included.
    ///
    /// # Errors
    /// Fails on a malformed identifier or a failed check character. A failed
    /// check means the string was mistyped, and continuing would open the
    /// wrong person's records.
    pub fn parse(text: &str) -> Result<Self> {
        let parts: Vec<&str> = text.split('-').collect();
        let (Some(&"PAR"), Some(&"P"), Some(body), Some(check), 4) = (
            parts.first(),
            parts.get(1),
            parts.get(2),
            parts.get(3),
            parts.len(),
        ) else {
            bail!("{text:?} is not a person identifier (expected PAR-P-NNNNNN-C)");
        };
        if body.len() != PERSON_DIGITS || !body.chars().all(|c| c.is_ascii_digit()) {
            bail!("{text:?}: the sequence must be exactly {PERSON_DIGITS} digits");
        }
        let expected = check_character(body);
        if *check != expected.to_string() {
            bail!(
                "{text:?}: check character is {check:?} but should be {expected:?} \
                 -- the identifier was mistyped"
            );
        }
        Ok(Self(text.to_owned()))
    }

    /// The identifier as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The opaque sequence, for allocating the next one.
    #[must_use]
    pub fn seq(&self) -> Option<u32> {
        self.0.split('-').nth(2)?.parse().ok()
    }
}

impl std::fmt::Display for PersonId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// ISO 7064 MOD 11-2 check character over a digit string: `0`-`9` or `X`.
///
/// Chosen because this identifier is typed into folder names and read aloud.
/// MOD 11-2 detects every single-digit error and every transposition of
/// adjacent digits, which is exactly what hand-copying produces.
#[must_use]
pub fn check_character(digits: &str) -> char {
    let mut p: u32 = 0;
    for c in digits.chars() {
        let d = u32::from(c as u8).saturating_sub(u32::from(b'0'));
        p = ((p + d) * 2) % 11;
    }
    let check = (12 - p) % 11;
    if check == 10 {
        'X'
    } else {
        char::from(b'0' + u8::try_from(check).unwrap_or(0))
    }
}

/// One engagement's identifier, e.g. `PAR-INT-2026-0001`.
///
/// **Deliberately NOT opaque (ADR-0051).** Class is mutable for a person and
/// immutable for an engagement: a person may become an employee, but an
/// engagement that was an internship was an internship permanently. Encoding
/// class here encodes a fact that cannot go stale, which is the test the
/// person identifier fails.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EngagementId(String);

impl EngagementId {
    /// Build from a class prefix, year and sequence.
    ///
    /// # Errors
    /// Fails if the prefix is not two to four uppercase letters; anything
    /// else would make the identifier ambiguous to parse back.
    pub fn new(prefix: &str, year: i32, seq: u32) -> Result<Self> {
        if !(2..=4).contains(&prefix.len()) || !prefix.chars().all(|c| c.is_ascii_uppercase()) {
            bail!("class prefix {prefix:?} must be 2-4 uppercase letters");
        }
        Ok(Self(format!("PAR-{prefix}-{year}-{seq:04}")))
    }

    /// The identifier as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The class prefix this engagement was opened under.
    #[must_use]
    pub fn prefix(&self) -> Option<&str> {
        self.0.split('-').nth(1)
    }

    /// The year it was opened in.
    #[must_use]
    pub fn year(&self) -> Option<i32> {
        self.0.split('-').nth(2)?.parse().ok()
    }

    /// The sequence within class and year.
    #[must_use]
    pub fn seq(&self) -> Option<u32> {
        self.0.split('-').nth(3)?.parse().ok()
    }
}

impl std::fmt::Display for EngagementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Where a person stands with Paridyum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Standing {
    /// Currently engaged.
    Active,
    /// Engagement ended. Documents and obligations survive.
    Separated,
    /// Personal data erased on request. The identifiers remain as a
    /// tombstone so issued serials still resolve to something.
    Erased,
}

impl Standing {
    /// The lowercase name used in the ledger.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Separated => "separated",
            Self::Erased => "erased",
        }
    }
}

/// One row of the ledger: a person, and the engagement they are in.
///
/// Appended on every allocation and every standing change, so the last row
/// for a person identifier is their current state and the rows before it are
/// the history. Re-engagement appends a row with the SAME person identifier
/// and a new engagement identifier — which is how one archive spans an
/// internship and the employment that follows it.
///
/// Holds no personal data beyond the slug, so it survives an erasure that
/// removes the person's record entirely.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Permanent, opaque person identifier.
    pub person_id: PersonId,
    /// Identifier of the engagement this row concerns.
    pub engagement_id: EngagementId,
    /// Filename the record is kept under, while it exists. Never an identity.
    pub slug: String,
    /// Person class of this engagement.
    pub class: String,
    /// ISO date the person identifier was first allocated.
    pub created_on: String,
    /// Current standing.
    pub standing: Standing,
    /// ISO date standing last changed.
    pub standing_changed_on: String,
    /// Digest of the record before this one.
    pub prev_sha256: String,
    /// Digest of this record.
    pub entry_sha256: String,
}

impl Chained for Record {
    fn prev(&self) -> &str {
        &self.prev_sha256
    }
    fn set_prev(&mut self, value: String) {
        self.prev_sha256 = value;
    }
    fn digest_field(&self) -> &str {
        &self.entry_sha256
    }
    fn set_digest_field(&mut self, value: String) {
        self.entry_sha256 = value;
    }
    fn digest_key() -> &'static str {
        "entry_sha256"
    }
    fn identity(&self) -> String {
        self.person_id.to_string()
    }
}

/// The next opaque person sequence.
///
/// Takes the maximum, never the count, so an identifier is never reissued
/// after a row is removed — a certificate issued in 2026 must still name the
/// same person in 2036.
#[must_use]
pub fn next_person_seq(ledger: &[Record]) -> u32 {
    ledger
        .iter()
        .filter_map(|r| r.person_id.seq())
        .max()
        .map_or(1, |m| m.saturating_add(1))
}

/// The next engagement sequence for a class prefix and year.
#[must_use]
pub fn next_engagement_seq(ledger: &[Record], prefix: &str, year: i32) -> u32 {
    ledger
        .iter()
        .filter(|r| {
            r.engagement_id.year() == Some(year) && r.engagement_id.prefix() == Some(prefix)
        })
        .filter_map(|r| r.engagement_id.seq())
        .max()
        .map_or(1, |m| m.saturating_add(1))
}

/// Whether a slug has ever been allocated.
///
/// Reserved FOREVER, including after erasure. Reusing one would make a
/// previous person's documents appear to belong to a new one, which is the
/// worst failure available to a system whose job is saying who signed what.
#[must_use]
pub fn slug_taken(ledger: &[Record], slug: &str) -> bool {
    ledger.iter().any(|r| r.slug == slug)
}

/// The latest row for each person identifier, in identifier order.
#[must_use]
pub fn current(ledger: &[Record]) -> Vec<&Record> {
    let mut latest: Vec<&Record> = Vec::new();
    for r in ledger {
        if let Some(slot) = latest.iter_mut().find(|x| x.person_id == r.person_id) {
            *slot = r;
        } else {
            latest.push(r);
        }
    }
    latest.sort_by(|a, b| a.person_id.cmp(&b.person_id));
    latest
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{
        EngagementId, PersonId, Record, Standing, check_character, current, next_engagement_seq,
        next_person_seq, slug_taken,
    };
    use crate::domain::chain::{seal, verify};

    fn row(pseq: u32, eseq: u32, slug: &str, standing: Standing) -> Record {
        Record {
            person_id: PersonId::new(pseq).expect("valid"),
            engagement_id: EngagementId::new("INT", 2026, eseq).expect("valid"),
            slug: slug.to_owned(),
            class: "intern".to_owned(),
            created_on: "2026-08-19".to_owned(),
            standing,
            standing_changed_on: "2026-08-19".to_owned(),
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        }
    }

    #[test]
    fn a_person_identifier_carries_no_fact_about_the_person() {
        // ADR-0051: no class, no year, no name. Only an opaque sequence.
        let id = PersonId::new(147).expect("valid");
        assert_eq!(id.as_str(), "PAR-P-000147-7");
        assert!(!id.as_str().contains("INT"), "no class");
        assert!(!id.as_str().contains("2026"), "no year");
    }

    #[test]
    fn a_person_identifier_round_trips_through_parsing() {
        let id = PersonId::new(147).expect("valid");
        let back = PersonId::parse(id.as_str()).expect("parses");
        assert_eq!(id, back);
        assert_eq!(back.seq(), Some(147));
    }

    #[test]
    fn a_mistyped_person_identifier_is_rejected() {
        let id = PersonId::new(147).expect("valid");
        let mut wrong: Vec<char> = id.as_str().chars().collect();
        // Corrupt one digit of the sequence.
        wrong[8] = if wrong[8] == '9' { '8' } else { '9' };
        let text: String = wrong.into_iter().collect();
        assert!(
            PersonId::parse(&text).is_err(),
            "a single-digit error must not open the wrong person's records"
        );
    }

    #[test]
    fn transposing_adjacent_digits_is_detected() {
        // The error hand-copying a folder name actually produces, and the
        // reason MOD 11-2 was chosen over a simple modulus.
        let mut caught = 0;
        let mut total = 0;
        for seq in 1..400u32 {
            let id = PersonId::new(seq).expect("valid");
            let body: Vec<char> = id.as_str()[6..12].chars().collect();
            for i in 0..body.len() - 1 {
                if body[i] == body[i + 1] {
                    continue; // transposing equal digits is not an error
                }
                let mut t = body.clone();
                t.swap(i, i + 1);
                let swapped: String = t.into_iter().collect();
                total += 1;
                if check_character(&swapped) != check_character(&body.iter().collect::<String>()) {
                    caught += 1;
                }
            }
        }
        assert!(
            total > 100,
            "the sweep should cover real cases, got {total}"
        );
        assert_eq!(
            caught, total,
            "MOD 11-2 must catch every adjacent transposition"
        );
    }

    #[test]
    fn every_single_digit_error_is_detected() {
        for seq in [1u32, 42, 147, 999_999] {
            let id = PersonId::new(seq).expect("valid");
            let body: String = id.as_str()[6..12].to_owned();
            let expected = check_character(&body);
            for i in 0..body.len() {
                for d in b'0'..=b'9' {
                    let mut alt: Vec<char> = body.chars().collect();
                    if alt[i] == char::from(d) {
                        continue;
                    }
                    alt[i] = char::from(d);
                    let text: String = alt.into_iter().collect();
                    assert_ne!(check_character(&text), expected, "{body} vs {text}");
                }
            }
        }
    }

    #[test]
    fn an_engagement_identifier_does_carry_class_and_year() {
        // Correct, and the whole distinction: class is immutable for an
        // engagement even though it is mutable for a person.
        let e = EngagementId::new("INT", 2026, 7).expect("valid");
        assert_eq!(e.as_str(), "PAR-INT-2026-0007");
        assert_eq!(e.prefix(), Some("INT"));
        assert_eq!(e.year(), Some(2026));
        assert_eq!(e.seq(), Some(7));
    }

    #[test]
    fn a_malformed_class_prefix_is_rejected() {
        assert!(EngagementId::new("I", 2026, 1).is_err());
        assert!(EngagementId::new("INTERN", 2026, 1).is_err());
        assert!(EngagementId::new("IN7", 2026, 1).is_err());
        assert!(EngagementId::new("int", 2026, 1).is_err());
    }

    #[test]
    fn person_sequences_are_global_and_engagement_sequences_are_scoped() {
        let ledger = vec![
            row(1, 1, "a", Standing::Active),
            row(2, 2, "b", Standing::Active),
        ];
        assert_eq!(next_person_seq(&ledger), 3);
        assert_eq!(next_engagement_seq(&ledger, "INT", 2026), 3);
        assert_eq!(
            next_engagement_seq(&ledger, "EMP", 2026),
            1,
            "new class starts at one"
        );
        assert_eq!(
            next_engagement_seq(&ledger, "INT", 2027),
            1,
            "new year starts at one"
        );
    }

    #[test]
    fn an_identifier_is_never_reissued_after_erasure() {
        let ledger = vec![
            row(1, 1, "a", Standing::Active),
            row(1, 1, "a", Standing::Erased),
        ];
        assert_eq!(next_person_seq(&ledger), 2);
        assert!(slug_taken(&ledger, "a"), "the slug stays reserved too");
    }

    #[test]
    fn re_engagement_keeps_the_person_and_changes_the_engagement() {
        // The case the whole decision exists for: an intern who is hired.
        let mut ledger = vec![row(1, 1, "asha", Standing::Separated)];
        let mut second = row(1, 1, "asha", Standing::Active);
        second.engagement_id = EngagementId::new("EMP", 2027, 1).expect("valid");
        second.class = "employee".to_owned();
        ledger.push(second);

        let now = current(&ledger);
        assert_eq!(now.len(), 1, "one person, not two");
        let row = now.first().expect("one");
        assert_eq!(
            row.person_id,
            PersonId::new(1).expect("valid"),
            "same archive"
        );
        assert_eq!(row.engagement_id.prefix(), Some("EMP"));
        assert_eq!(row.standing, Standing::Active);
    }

    #[test]
    fn the_ledger_seals_into_a_chain() {
        let a = seal(row(1, 1, "a", Standing::Active), None).expect("seals");
        let b = seal(row(2, 2, "b", Standing::Active), Some(&a)).expect("seals");
        assert_eq!(b.prev_sha256, a.entry_sha256);
        assert!(verify(&[a, b]).expect("verifiable").is_empty());
    }

    #[test]
    fn standings_round_trip_to_their_ledger_spelling() {
        assert_eq!(Standing::Active.as_str(), "active");
        assert_eq!(Standing::Separated.as_str(), "separated");
        assert_eq!(Standing::Erased.as_str(), "erased");
    }
}
