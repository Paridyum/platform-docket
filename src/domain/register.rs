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
    pub person_id: crate::domain::identity::PersonId,
    /// The engagement it was issued under.
    pub engagement_id: crate::domain::identity::EngagementId,
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
            person_id: crate::domain::identity::PersonId::new(1).expect("valid"),
            engagement_id: crate::domain::identity::EngagementId::new("INT", 2026, 1)
                .expect("valid"),
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
