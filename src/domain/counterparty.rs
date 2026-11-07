//! An organisation a document is issued to, and the three facts docket may hold about it.
//!
//! RFC-0136 open question 3, resolved 2026-11-07 ("extend docket"): a university, incubator or
//! company can be the party a document is issued **to**. Docket holds its legal name, its address
//! and the office that receives the document — and nothing else. Not a contact list, not a person
//! record for anyone who works there: a register needs to say who a serial was issued to, and
//! those three facts say it.
//!
//! **"Only these three" is enforced by the parser, not by a check that runs later.** The record
//! is `deny_unknown_fields`, so a contact name, an email or a phone number added by hand is a
//! record that does not load. A field with no stated purpose must not exist (`fields.yaml`), and
//! the purposes are written below, on the fields.

use crate::domain::identity::check_character;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// Width of the sequence inside a counterparty identifier.
const DIGITS: usize = 6;

/// An organisation counterparty's permanent identifier, e.g. `PAR-O-000001-7`.
///
/// The SHAPE of ADR-0051's person identifier with `O` where it has `P`, so the register's two
/// kinds of recipient cannot be confused and both survive being typed and read aloud — the check
/// character is the same ISO 7064 MOD 11-2. Allocated once and never reissued.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CounterpartyId(String);

impl CounterpartyId {
    /// Allocate from a sequence, appending a check character.
    ///
    /// # Errors
    /// Fails if `seq` will not fit in six digits.
    pub fn new(seq: u32) -> Result<Self> {
        let body = format!("{seq:0DIGITS$}");
        if body.len() > DIGITS {
            bail!("counterparty sequence {seq} exceeds {DIGITS} digits");
        }
        let check = check_character(&body);
        Ok(Self(format!("PAR-O-{body}-{check}")))
    }

    /// Parse and validate an identifier, check character included.
    ///
    /// # Errors
    /// Fails on a malformed identifier or a failed check character.
    pub fn parse(text: &str) -> Result<Self> {
        let mut parts = text.split('-');
        let (Some("PAR"), Some("O"), Some(body), Some(check), None) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) else {
            bail!("{text:?} is not a counterparty identifier (expected PAR-O-NNNNNN-C)");
        };
        if body.len() != DIGITS || !body.chars().all(|c| c.is_ascii_digit()) {
            bail!("{text:?}: the sequence must be exactly {DIGITS} digits");
        }
        if check != check_character(body).to_string() {
            bail!("{text:?}: check character does not match -- the identifier was mistyped");
        }
        Ok(Self(text.to_owned()))
    }

    /// The identifier as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The sequence, for allocating the next one.
    #[must_use]
    pub fn seq(&self) -> Option<u32> {
        self.0.split('-').nth(2)?.parse().ok()
    }
}

impl TryFrom<String> for CounterpartyId {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl From<CounterpartyId> for String {
    fn from(id: CounterpartyId) -> Self {
        id.0
    }
}

impl std::fmt::Display for CounterpartyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One organisation counterparty, as recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counterparty {
    /// Filename the record is kept under. Not part of the record, and never an identity.
    #[serde(skip)]
    pub slug: String,
    /// Record format version.
    #[serde(default = "one")]
    pub version: u32,
    /// The identifier the register keys on.
    pub id: CounterpartyId,
    /// Names the party on the document, exactly as the organisation names itself.
    pub legal_name: String,
    /// Where the document is addressed, and where a formal notice to the organisation would go.
    pub address: String,
    /// The office or role that receives the document, e.g. "The President, Institution's
    /// Innovation Council". An office, never a person: whoever holds it receives it.
    pub office: String,
}

const fn one() -> u32 {
    1
}

impl Counterparty {
    /// Read a record from its YAML body.
    ///
    /// # Errors
    /// Fails if the body does not parse, holds a field other than the three permitted, or carries
    /// an invalid identifier.
    pub fn parse(slug: &str, body: &str) -> Result<Self> {
        let mut record: Self = serde_yaml::from_str(body).with_context(|| {
            format!(
                "counterparties/{slug}.yaml: a counterparty record holds only id, legal_name, \
                 address and office (RFC-0136)"
            )
        })?;
        slug.clone_into(&mut record.slug);
        Ok(record)
    }

    /// What is wrong with this record, if anything. Empty when it may be issued against.
    #[must_use]
    pub fn problems(&self) -> Vec<String> {
        [
            ("legal_name", &self.legal_name),
            ("address", &self.address),
            ("office", &self.office),
        ]
        .into_iter()
        .filter(|(_, v)| v.trim().is_empty())
        .map(|(f, _)| format!("{}: {f} is empty", self.slug))
        .collect()
    }

    /// The record as written to disk, under a header stating what each field is for.
    ///
    /// **Serialised by `serde_yaml`, never by hand.** Rust's `{:?}` escaping is not YAML's: it
    /// writes a Devanagari vowel sign or a decomposed accent as `\u{…}`, which YAML cannot read,
    /// and one such record made every later `counterparty` command and `docket check` fail.
    /// The bytes are parsed back before they are returned, so a record that would not load is
    /// refused here rather than written.
    ///
    /// # Errors
    /// Fails if the record cannot be serialised, or would not read back unchanged.
    pub fn to_yaml(&self) -> Result<String> {
        let body = serde_yaml::to_string(self).context("serialising a counterparty record")?;
        let out = format!(
            "# An organisation counterparty (RFC-0136). Written by `docket counterparty new`.\n\
             # It holds three facts and nothing else; a field added here is a record that no\n\
             # longer loads. Never add a person's name, email or phone.\n\
             #   legal_name  names the party on the document\n\
             #   address     where the document is addressed\n\
             #   office      the office that receives it -- an office, never a person\n\
             {body}"
        );
        if Self::parse(&self.slug, &out)? != *self {
            bail!("{}: the record would not read back unchanged", self.slug);
        }
        Ok(out)
    }
}

/// The next identifier after every one already used.
///
/// # Errors
/// Fails if the sequence is exhausted.
pub fn next_id<'a>(used: impl IntoIterator<Item = &'a CounterpartyId>) -> Result<CounterpartyId> {
    let top = used
        .into_iter()
        .filter_map(CounterpartyId::seq)
        .max()
        .unwrap_or(0);
    CounterpartyId::new(top.saturating_add(1))
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample() -> Counterparty {
        Counterparty {
            slug: "example-university".to_owned(),
            version: 1,
            id: CounterpartyId::new(1).expect("id"),
            legal_name: "Example University".to_owned(),
            address: "1 College Road, Dehradun -- 248001".to_owned(),
            office: "The President, Institution's Innovation Council".to_owned(),
        }
    }

    #[test]
    fn an_identifier_round_trips_and_a_mistyped_one_is_refused() {
        let id = CounterpartyId::new(147).expect("id");
        assert!(id.as_str().starts_with("PAR-O-000147-"));
        assert_eq!(CounterpartyId::parse(id.as_str()).expect("parses"), id);
        let mistyped = id.as_str().replace("000147", "000174");
        assert!(CounterpartyId::parse(&mistyped).is_err());
        // A person identifier is never read as an organisation's.
        let person = crate::domain::identity::PersonId::new(1).expect("person id");
        assert!(CounterpartyId::parse(person.as_str()).is_err());
    }

    #[test]
    fn a_written_record_reads_back_unchanged() {
        let record = sample();
        let back = Counterparty::parse("example-university", &record.to_yaml().expect("yaml"))
            .expect("parses");
        assert_eq!(back, record);
    }

    #[test]
    fn any_text_an_institution_writes_its_name_in_reads_back_unchanged() {
        // Each of these broke the hand-rolled writer: Devanagari vowel signs and viramas, an
        // accent pasted decomposed (NFD) from a PDF, a zero-width space, and YAML's own syntax.
        for name in [
            "भारतीय प्रौद्योगिकी संस्थान रुड़की",
            "Universite\u{301} de Montre\u{301}al",
            "Zero\u{200b}Width",
            "Name: with \"quotes\", 'apostrophes' and # hashes",
            "yes",
            "null",
            "- leading dash",
            "Two\nlines",
        ] {
            let mut record = sample();
            record.legal_name = name.to_owned();
            record.address = format!("{name}, Dehradun");
            record.office = format!("Registrar: {name}");
            let yaml = record.to_yaml().expect("serialises");
            let back = Counterparty::parse("example-university", &yaml).expect(name);
            assert_eq!(back, record, "{name:?}");
        }
    }

    #[test]
    fn a_contact_person_added_by_hand_is_refused_rather_than_kept() {
        // The whole point of the decision: no person's data rides along with an organisation.
        let body = format!(
            "{}contact_email: someone@example.edu\n",
            sample().to_yaml().expect("yaml")
        );
        assert!(Counterparty::parse("example-university", &body).is_err());
    }

    #[test]
    fn an_empty_fact_is_a_problem() {
        let mut record = sample();
        record.office = "  ".to_owned();
        assert_eq!(record.problems(), ["example-university: office is empty"]);
        assert!(sample().problems().is_empty());
    }

    #[test]
    fn allocation_continues_after_the_highest_identifier_used() {
        let used = [
            CounterpartyId::new(3).expect("id"),
            CounterpartyId::new(1).expect("id"),
        ];
        assert_eq!(next_id(&used).expect("next").seq(), Some(4));
        assert_eq!(next_id(&[]).expect("first").seq(), Some(1));
    }
}
