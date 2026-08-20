//! Sealing records into a tamper-evident chain.

use anyhow::Result;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// The digest a chain starts from, before any record exists.
pub const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// A record that carries its position in a chain.
///
/// Three chains use this: issued documents, person identities, and personnel
/// events. They are separate files because they answer separate questions,
/// but the integrity argument is identical and is written once here.
pub trait Chained: Serialize {
    /// Digest of the record before this one.
    fn prev(&self) -> &str;
    /// Set the previous digest, when sealing.
    fn set_prev(&mut self, value: String);
    /// This record's own digest.
    fn digest_field(&self) -> &str;
    /// Set this record's digest, when sealing.
    fn set_digest_field(&mut self, value: String);
    /// The serialised field name holding this record's digest, excluded
    /// from its own computation.
    fn digest_key() -> &'static str;
    /// Identifier used when reporting a fault.
    fn identity(&self) -> String;
}

/// Hash a record, excluding the field that holds the hash.
///
/// Serialised through `serde_json::Value`, whose map is a `BTreeMap`, so key
/// order is deterministic without depending on struct field order. A digest
/// that changed when a field was reordered would invalidate every historical
/// record on a refactor.
///
/// # Errors
/// Fails if the record cannot be serialised.
pub fn digest_of<T: Serialize>(record: &T, exclude: &str) -> Result<String> {
    let mut value = serde_json::to_value(record)?;
    if let Some(map) = value.as_object_mut() {
        map.remove(exclude);
    }
    Ok(hex(&Sha256::digest(
        serde_json::to_string(&value)?.as_bytes(),
    )))
}

/// Seal a record onto the end of a chain.
///
/// # Errors
/// Fails if the record cannot be serialised for hashing.
pub fn seal<T: Chained>(mut record: T, previous: Option<&T>) -> Result<T> {
    record.set_prev(previous.map_or_else(|| GENESIS.to_owned(), |p| p.digest_field().to_owned()));
    record.set_digest_field(String::new());
    let digest = digest_of(&record, T::digest_key())?;
    record.set_digest_field(digest);
    Ok(record)
}

/// A break in a chain, or a record whose contents no longer match its hash.
#[derive(Debug, PartialEq, Eq)]
pub enum Fault {
    /// The record does not follow the one before it: something was removed.
    BrokenLink {
        /// One-based position in the chain.
        index: usize,
        /// Identifier of the record that does not link.
        identity: String,
    },
    /// The record's contents were changed after it was sealed.
    Altered {
        /// One-based position in the chain.
        index: usize,
        /// Identifier of the altered record.
        identity: String,
    },
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BrokenLink { index, identity } => {
                write!(
                    f,
                    "entry {index} ({identity}): does not link to the entry before it"
                )
            }
            Self::Altered { index, identity } => {
                write!(
                    f,
                    "entry {index} ({identity}): contents were altered after sealing"
                )
            }
        }
    }
}

/// Walk a chain and report every fault, rather than stopping at the first.
///
/// Reporting all of them matters: one altered record breaks the link of the
/// record after it too, and an operator seeing a single error would not know
/// whether one record or the whole tail was tampered with.
///
/// # Errors
/// Fails if a record cannot be re-serialised for hashing.
pub fn verify<T: Chained>(records: &[T]) -> Result<Vec<Fault>> {
    let mut faults = Vec::new();
    let mut expected = GENESIS.to_owned();
    for (i, record) in records.iter().enumerate() {
        let index = i + 1;
        if record.prev() != expected {
            faults.push(Fault::BrokenLink {
                index,
                identity: record.identity(),
            });
        }
        if digest_of(record, T::digest_key())? != record.digest_field() {
            faults.push(Fault::Altered {
                index,
                identity: record.identity(),
            });
        }
        expected.clear();
        expected.push_str(record.digest_field());
    }
    Ok(faults)
}

/// Lowercase hex, the form every hash in the system takes.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            // Writing to a String cannot fail; the Result is discarded rather
            // than unwrapped because `unwrap_used` is denied crate-wide.
            let _ = write!(s, "{b:02x}");
            s
        })
}

/// SHA-256 of a byte slice, as lowercase hex.
#[must_use]
pub fn sha256(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{Chained, Fault, GENESIS, seal, sha256, verify};
    use serde::Serialize;

    #[derive(Debug, Serialize, Clone)]
    struct Rec {
        id: String,
        payload: String,
        prev_sha256: String,
        entry_sha256: String,
    }

    impl Chained for Rec {
        fn prev(&self) -> &str {
            &self.prev_sha256
        }
        fn set_prev(&mut self, v: String) {
            self.prev_sha256 = v;
        }
        fn digest_field(&self) -> &str {
            &self.entry_sha256
        }
        fn set_digest_field(&mut self, v: String) {
            self.entry_sha256 = v;
        }
        fn digest_key() -> &'static str {
            "entry_sha256"
        }
        fn identity(&self) -> String {
            self.id.clone()
        }
    }

    fn chain(n: usize) -> Vec<Rec> {
        let mut out: Vec<Rec> = Vec::new();
        for i in 1..=n {
            let r = Rec {
                id: format!("R{i}"),
                payload: format!("p{i}"),
                prev_sha256: String::new(),
                entry_sha256: String::new(),
            };
            out.push(seal(r, out.last()).expect("seals"));
        }
        out
    }

    #[test]
    fn a_clean_chain_has_no_faults() {
        assert!(verify(&chain(3)).expect("verifiable").is_empty());
    }

    #[test]
    fn the_first_record_links_to_genesis() {
        assert_eq!(
            chain(1).first().map(|r| r.prev_sha256.as_str()),
            Some(GENESIS)
        );
    }

    #[test]
    fn altering_a_record_is_detected() {
        let mut c = chain(3);
        if let Some(r) = c.get_mut(1) {
            r.payload = "tampered".to_owned();
        }
        assert!(
            verify(&c)
                .expect("verifiable")
                .iter()
                .any(|f| matches!(f, Fault::Altered { index: 2, .. }))
        );
    }

    #[test]
    fn removing_a_record_is_detected() {
        let mut c = chain(3);
        c.remove(1);
        assert!(
            verify(&c)
                .expect("verifiable")
                .iter()
                .any(|f| matches!(f, Fault::BrokenLink { index: 2, .. }))
        );
    }

    #[test]
    fn sha256_matches_the_known_empty_digest() {
        assert_eq!(
            sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
