//! The tamper-evident record of every document Paridyum has issued.
//!
//! Issuance only. Whether a document came BACK executed is a separate fact,
//! learned later, and lives in `domain::signature` -- recording it here would
//! mean mutating a sealed entry and invalidating every digest after it.

use crate::domain::chain::{self, Chained};
use serde::{Deserialize, Serialize};

pub use crate::domain::chain::{GENESIS, hex, sha256};

/// One issued document.
///
/// `template_sha256` ties the entry to the WORDING that was issued, not to
/// whatever the template says today. Without it, editing a template would
/// silently rewrite the history of what people signed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// `prev_sha256` / `entry_sha256` name the chain links and are the ON-DISK
// field names in issued.jsonl. Renaming them to satisfy the lint would
// change the register format, which existing entries are sealed against.
#[allow(clippy::struct_field_names)]
pub struct Entry {
    /// Serial as printed on the document.
    pub serial: String,
    /// Document code, e.g. `NDA`.
    pub code: String,
    /// Sequence within this code and year.
    pub seq: u32,
    /// Year of issue.
    pub year: i32,
    /// Permanent, opaque identifier of the person it was issued to.
    ///
    /// The authoritative link (ADR-0051). `slug` is kept alongside for human
    /// convenience and is never keyed off: it is a filename, not an identity.
    ///
    /// `None` only for a document issued to an organisation (RFC-0136), which names
    /// `counterparty_id` instead. Every entry written before that existed holds one, and
    /// serialises exactly as it always did, so its sealed digest is unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub person_id: Option<crate::domain::identity::PersonId>,
    /// The engagement it was issued under. `None` exactly when `person_id` is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engagement_id: Option<crate::domain::identity::EngagementId>,
    /// The organisation it was issued to, where it was issued to one (RFC-0136).
    ///
    /// Absent from every person's entry, so their bytes and digests are unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counterparty_id: Option<crate::domain::counterparty::CounterpartyId>,
    /// Slug at the time of issue, for human reference only.
    pub slug: String,
    /// Document id from the lifecycle.
    pub document: String,
    /// Title as printed.
    pub title: String,
    /// How the document is handled.
    pub kind: String,
    /// ISO date of issue.
    pub issued_on: String,
    /// Retention class governing it.
    pub retention: String,
    /// Hash of the template AS ISSUED, not as it stands today.
    pub template_sha256: String,
    /// Hash of the issued PDF. What `verify` compares against.
    pub pdf_sha256: String,
    /// Location within the store.
    pub pdf_path: String,
    /// Recorded so a later reproduction attempt knows which toolchain to pin.
    pub texlive: String,
    /// Which typesetter produced it (ADR-0054 point 3).
    ///
    /// `#[serde(default)]` so entries written before two engines existed still
    /// deserialise, and `skip_serializing_if` so their bytes -- and therefore
    /// their sealed digests -- are unchanged. An entry with no engine recorded
    /// was produced by pdflatex, which is what `engine_or_default` returns.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// The engine's reported version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_version: Option<String>,
    /// Signatory ids who signed for Paridyum, as resolved at issue.
    ///
    /// RECORDED BECAUSE IT CAN BE SUBSTITUTED. An officer's documents route to the
    /// Director, and where the Director is the SUBJECT the routing falls through to
    /// another signatory so that nobody signs their own document. Who actually signed
    /// is therefore not derivable from `lifecycle.yaml` alone, and a register that
    /// cannot answer "who signed this" for a document it issued is not doing its job.
    ///
    /// `skip_serializing_if` so entries sealed before this field existed keep their
    /// exact bytes, and therefore their digests.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signed_by: Vec<String>,
    /// Digest of the entry before this one.
    pub prev_sha256: String,
    /// Digest of this entry, sealing it onto the chain.
    pub entry_sha256: String,
}

impl Entry {
    /// Whether this entry was issued to `person`. Never true of an organisation's entry.
    #[must_use]
    pub fn is_for(&self, person: &crate::domain::identity::PersonId) -> bool {
        self.person_id.as_ref() == Some(person)
    }

    /// Refuse an entry that does not name exactly one recipient.
    ///
    /// A person's entry names a person AND the engagement it was issued under; an organisation's
    /// names its `counterparty_id`; never both, never neither (RFC-0136). Enforced where entries
    /// are appended and where they are read, by both stores, so the shape is a rule rather than
    /// the convention `render_and_register` happens to follow -- an entry naming nobody would
    /// otherwise store its PDF at `issued//…`.
    ///
    /// # Errors
    /// Fails, naming the serial, when the entry names nobody, both, or half a person.
    pub fn validate(&self) -> anyhow::Result<()> {
        let person = (self.person_id.is_some(), self.engagement_id.is_some());
        match (person, self.counterparty_id.is_some()) {
            ((true, true), false) | ((false, false), true) => Ok(()),
            ((false, false), false) => anyhow::bail!(
                "{}: names no recipient -- neither a person nor an organisation",
                self.serial
            ),
            (_, true) => anyhow::bail!(
                "{}: names both a person and an organisation; an entry has one recipient",
                self.serial
            ),
            _ => anyhow::bail!(
                "{}: a person's entry needs both person_id and engagement_id",
                self.serial
            ),
        }
    }

    /// The identifier of whoever it was issued to: the person's, or the organisation's.
    ///
    /// For naming a blob folder or a report column, never for matching -- `is_for` matches.
    #[must_use]
    pub fn recipient(&self) -> &str {
        match (&self.person_id, &self.counterparty_id) {
            (Some(p), _) => p.as_str(),
            (None, Some(c)) => c.as_str(),
            (None, None) => "",
        }
    }

    /// The engine that produced this document.
    ///
    /// Entries written before engine detection existed carry none; they were
    /// all produced by pdflatex, so that is what they report rather than
    /// "unknown". Guessing here is safe precisely because the alternative did
    /// not exist when they were written.
    #[must_use]
    pub fn engine_or_default(&self) -> &str {
        self.engine.as_deref().unwrap_or("pdflatex")
    }
}

impl Chained for Entry {
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
        self.serial.clone()
    }
}

impl Entry {
    /// The digest of everything in this entry except the digest itself.
    ///
    /// # Errors
    /// Fails if the entry cannot be serialised.
    pub fn digest(&self) -> anyhow::Result<String> {
        chain::digest_of(self, "entry_sha256")
    }

    /// Seal this entry onto the chain, computing both hashes.
    ///
    /// # Errors
    /// Fails if the entry cannot be serialised for hashing.
    pub fn seal(self, previous: Option<&Self>) -> anyhow::Result<Self> {
        chain::seal(self, previous)
    }
}

/// A break in the register, reported in the register's own vocabulary.
pub type ChainFault = chain::Fault;

/// Walk the register and report every fault.
///
/// # Errors
/// Fails if an entry cannot be re-serialised for hashing.
pub fn verify_chain(entries: &[Entry]) -> anyhow::Result<Vec<ChainFault>> {
    chain::verify(entries)
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{ChainFault, Entry, GENESIS, sha256, verify_chain};

    fn entry(serial: &str, seq: u32) -> Entry {
        Entry {
            serial: serial.to_owned(),
            code: "NDA".to_owned(),
            seq,
            year: 2026,
            person_id: Some(crate::domain::identity::PersonId::new(1).expect("valid")),
            engagement_id: Some(
                crate::domain::identity::EngagementId::new("PAR", "INT", 2026, 1).expect("valid"),
            ),
            counterparty_id: None,
            slug: "asha-verma".to_owned(),
            document: "nda".to_owned(),
            title: "Non-Disclosure Agreement".to_owned(),
            kind: "signed".to_owned(),
            issued_on: "2026-08-19".to_owned(),
            retention: "contract".to_owned(),
            template_sha256: "a".repeat(64),
            pdf_sha256: "b".repeat(64),
            pdf_path: "issued/asha-verma/x.pdf".to_owned(),
            texlive: "2026".to_owned(),
            engine: None,
            engine_version: None,
            signed_by: vec!["cto".to_owned()],
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        }
    }

    fn chain_of(n: u32) -> Vec<Entry> {
        let mut out: Vec<Entry> = Vec::new();
        for i in 1..=n {
            let sealed = entry(&format!("PAR/NDA/2026/{i:04}"), i)
                .seal(out.last())
                .expect("sealing a well-formed entry succeeds");
            out.push(sealed);
        }
        out
    }

    #[test]
    fn a_clean_chain_has_no_faults() {
        assert!(verify_chain(&chain_of(3)).expect("verifiable").is_empty());
    }

    #[test]
    fn the_seal_survives_a_json_round_trip() {
        // ADR-0093 / migration 0006 store each sealed entry as JSONB, and RFC-0039 D1 is
        // unconditional: the chain moves with its guarantees or the move does not happen. A JSONB
        // column IS a record -> JSON -> record path, so proving that path preserves both the link
        // and the digest proves the store move is safe. `verify` re-serialises the struct, so what
        // must hold is that the struct survives the round trip byte-for-byte.
        let sealed = chain_of(3);
        let restored: Vec<Entry> = sealed
            .iter()
            .map(|e| {
                let json = serde_json::to_string(e).expect("serialise as it would enter JSONB");
                serde_json::from_str::<Entry>(&json).expect("deserialise as it would leave JSONB")
            })
            .collect();
        for (before, after) in sealed.iter().zip(&restored) {
            assert_eq!(
                serde_json::to_value(before).expect("value"),
                serde_json::to_value(after).expect("value"),
                "the round trip changed the record, which would change its digest"
            );
        }
        assert!(
            verify_chain(&restored).expect("verifiable").is_empty(),
            "the seal did not survive the JSON round trip"
        );
    }

    #[test]
    fn a_persons_entry_serialises_exactly_as_before_organisations_existed() {
        // Every entry already sealed was written with these keys in this order. If a person's entry
        // gained or reordered one, its digest would change and every chain on disk would break.
        let json = serde_json::to_value(entry("PAR/NDA/2026/0001", 1)).expect("value");
        let keys: Vec<&str> = json
            .as_object()
            .expect("object")
            .keys()
            .map(String::as_str)
            .collect();
        assert!(keys.contains(&"person_id") && keys.contains(&"engagement_id"));
        assert!(!keys.contains(&"counterparty_id"), "{keys:?}");
    }

    #[test]
    fn an_organisations_entry_names_no_person_and_seals_onto_the_same_chain() {
        // RFC-0136: one register for both kinds of recipient.
        let mut chain = chain_of(2);
        let org = crate::domain::counterparty::CounterpartyId::new(1).expect("id");
        let mut e = entry("PAR/ICL/2026/0001", 1);
        e.person_id = None;
        e.engagement_id = None;
        e.counterparty_id = Some(org.clone());
        let sealed = e.seal(chain.last()).expect("seals");
        assert_eq!(sealed.recipient(), org.as_str());
        let person = crate::domain::identity::PersonId::new(1).expect("id");
        assert!(!sealed.is_for(&person) && chain[0].is_for(&person));
        let json = serde_json::to_string(&sealed).expect("json");
        assert!(
            !json.contains("person_id") && !json.contains("engagement_id"),
            "{json}"
        );
        chain.push(serde_json::from_str(&json).expect("reads back"));
        assert!(verify_chain(&chain).expect("verifiable").is_empty());
    }

    /// Three sealed entries in the register's on-disk format as it stood before RFC-0136: one
    /// legacy (no engine, no `signed_by`), one current, one with non-ASCII text.
    const GOLDEN: &str = include_str!("../../tests/fixtures/register-v1.jsonl");

    #[test]
    fn entries_already_on_disk_still_verify_and_reserialise_byte_for_byte() {
        // Pins the FORMAT, not just three key names: renaming or re-keying any field on write
        // changes these bytes and these digests, and every register on disk would then fail to
        // verify. A presence check on the recipient keys did not catch that; this does.
        let entries: Vec<Entry> = GOLDEN
            .lines()
            .map(|l| serde_json::from_str(l).expect("parses"))
            .collect();
        assert!(verify_chain(&entries).expect("verifiable").is_empty());
        for (line, e) in GOLDEN.lines().zip(&entries) {
            assert_eq!(serde_json::to_string(e).expect("json"), line);
            e.validate().expect("a person's entry is valid");
        }
        assert_eq!(
            entries.last().map(|e| e.entry_sha256.as_str()),
            Some("668dcbea89ce59c3c108181409d109b592f0eae916fc63db963590db8431d8c7")
        );
    }

    #[test]
    fn an_entry_names_exactly_one_recipient() {
        let person = entry("PAR/NDA/2026/0001", 1);
        person.validate().expect("a person's entry");
        let org = crate::domain::counterparty::CounterpartyId::new(1).expect("id");
        let mut both = person.clone();
        both.counterparty_id = Some(org.clone());
        let mut neither = person.clone();
        neither.person_id = None;
        neither.engagement_id = None;
        let mut half = person.clone();
        half.engagement_id = None;
        let mut organisation = neither.clone();
        organisation.counterparty_id = Some(org);
        organisation.validate().expect("an organisation's entry");
        for (bad, why) in [
            (both, "both"),
            (neither, "no recipient"),
            (half, "both person_id"),
        ] {
            let err = bad.validate().expect_err(why);
            assert!(err.to_string().contains(why), "{err}");
        }
    }

    #[test]
    fn the_first_entry_links_to_genesis() {
        let chain = chain_of(1);
        assert_eq!(chain.first().map(|e| e.prev_sha256.as_str()), Some(GENESIS));
    }

    #[test]
    fn backdating_an_entry_is_detected() {
        // The attack this defends: quietly changing an issue date so a
        // document appears to predate something.
        let mut chain = chain_of(3);
        if let Some(e) = chain.get_mut(1) {
            e.issued_on = "2020-01-01".to_owned();
        }
        let faults = verify_chain(&chain).expect("verifiable");
        assert!(
            faults
                .iter()
                .any(|f| matches!(f, ChainFault::Altered { index: 2, .. }))
        );
    }

    #[test]
    fn removing_an_entry_is_detected() {
        let mut chain = chain_of(3);
        chain.remove(1);
        let faults = verify_chain(&chain).expect("verifiable");
        assert!(
            faults
                .iter()
                .any(|f| matches!(f, ChainFault::BrokenLink { index: 2, .. }))
        );
    }

    #[test]
    fn digest_ignores_only_its_own_field() {
        let mut e = chain_of(1).remove(0);
        let before = e.digest().expect("digest");
        e.entry_sha256 = "different".to_owned();
        assert_eq!(before, e.digest().expect("digest"), "own field excluded");
        e.pdf_sha256 = "c".repeat(64);
        assert_ne!(before, e.digest().expect("digest"), "other fields included");
    }

    #[test]
    fn sha256_matches_the_known_empty_digest() {
        assert_eq!(
            sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
