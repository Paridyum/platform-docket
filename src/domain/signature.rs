//! The record that a document came back executed.

use crate::domain::chain::Chained;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// How a document was executed.
///
/// Recorded because it decides what evidence exists. A wet-ink signature is
/// only as good as the scan held against it; an Aadhaar eSign carries its own
/// cryptographic trail and does not need one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Method {
    /// Signed on paper and returned, usually as a scan.
    WetInk,
    /// Signed through Aadhaar eSign.
    AadhaarEsign,
    /// Signed through another electronic service.
    Electronic,
}

impl Method {
    /// The name used in the ledger.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WetInk => "wet-ink",
            Self::AadhaarEsign => "aadhaar-esign",
            Self::Electronic => "electronic",
        }
    }

    /// Parse a method from its written name.
    ///
    /// # Errors
    /// Fails on an unknown name, listing the ones that exist.
    pub fn parse(name: &str) -> Result<Self> {
        const ALL: [Method; 3] = [Method::WetInk, Method::AadhaarEsign, Method::Electronic];
        ALL.into_iter().find(|m| m.as_str() == name).map_or_else(
            || {
                let names: Vec<&str> = ALL.iter().map(|m| m.as_str()).collect();
                bail!(
                    "unknown signing method {name:?}; known: {}",
                    names.join(" ")
                )
            },
            Ok,
        )
    }

    /// Whether a scanned copy is the only evidence this method leaves behind.
    #[must_use]
    pub fn needs_scan(self) -> bool {
        matches!(self, Self::WetInk)
    }
}

/// One executed document.
///
/// A SEPARATE chain rather than a field on the register entry. Recording a
/// signature by mutating the issuance entry would invalidate its digest and
/// every digest after it -- the register is append-only precisely so that
/// what was issued cannot be rewritten once someone has relied on it. A
/// signature is a new fact learned later, so it is a new record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Serial of the document that was executed.
    pub serial: String,
    /// ISO date the last party signed.
    pub signed_on: String,
    /// How it was executed.
    pub method: Method,
    /// SHA-256 of the returned scan, where one is held.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_sha256: Option<String>,
    /// Where the scan is filed, relative to the store.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scan_path: Option<String>,
    /// ISO date Paridyum received it.
    pub received_on: String,
    /// Who recorded the receipt.
    pub recorded_by: String,
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
        self.serial.clone()
    }
}

/// Whether a serial has been recorded as executed.
#[must_use]
pub fn is_signed(signatures: &[Record], serial: &str) -> bool {
    signatures.iter().any(|s| s.serial == serial)
}

/// The signature for a serial, if one is recorded.
#[must_use]
pub fn for_serial<'a>(signatures: &'a [Record], serial: &str) -> Option<&'a Record> {
    signatures.iter().find(|s| s.serial == serial)
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{Method, Record, for_serial, is_signed};
    use crate::domain::chain::{seal, verify};

    fn record(serial: &str, method: Method) -> Record {
        Record {
            serial: serial.to_owned(),
            signed_on: "2026-09-01".to_owned(),
            method,
            scan_sha256: None,
            scan_path: None,
            received_on: "2026-09-02".to_owned(),
            recorded_by: "cto".to_owned(),
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        }
    }

    #[test]
    fn methods_round_trip_through_their_names() {
        for name in ["wet-ink", "aadhaar-esign", "electronic"] {
            assert_eq!(Method::parse(name).expect("known").as_str(), name);
        }
    }

    #[test]
    fn an_unknown_method_names_the_ones_that_exist() {
        let err = Method::parse("crayon")
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(err.contains("wet-ink"), "{err}");
    }

    #[test]
    fn only_wet_ink_depends_on_a_scan_for_its_evidence() {
        assert!(Method::WetInk.needs_scan());
        assert!(
            !Method::AadhaarEsign.needs_scan(),
            "eSign carries its own trail"
        );
        assert!(!Method::Electronic.needs_scan());
    }

    #[test]
    fn signatures_seal_into_their_own_chain() {
        let a = seal(record("PAR/NDA/2026/0001", Method::WetInk), None).expect("seals");
        let b = seal(record("PAR/IPA/2026/0001", Method::WetInk), Some(&a)).expect("seals");
        assert_eq!(b.prev_sha256, a.entry_sha256);
        assert!(verify(&[a, b]).expect("verifiable").is_empty());
    }

    #[test]
    fn the_seal_survives_a_json_round_trip() {
        // The fourth chain, held to the same bar as the other three: ADR-0093 / migration 0006 store
        // each sealed record as JSONB, and RFC-0039 D1 is unconditional — the chain moves with its
        // guarantees or the move does not happen. A JSONB column IS a record -> JSON -> record path,
        // so proving that path preserves both the link and the digest proves the store move is safe.
        // `Method` serialises kebab-case and `scan_*` are `skip_serializing_if`, so this also guards
        // that neither reshapes the bytes `verify` re-hashes.
        let a = seal(record("PAR/NDA/2026/0001", Method::WetInk), None).expect("seals");
        let b = seal(record("PAR/IPA/2026/0001", Method::AadhaarEsign), Some(&a)).expect("seals");
        let sealed = [a, b];
        let restored: Vec<Record> = sealed
            .iter()
            .map(|r| {
                let json = serde_json::to_string(r).expect("serialise as it would enter JSONB");
                serde_json::from_str::<Record>(&json).expect("deserialise as it would leave JSONB")
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
            verify(&restored).expect("verifiable").is_empty(),
            "the seal did not survive the JSON round trip"
        );
    }

    #[test]
    fn lookup_is_by_serial() {
        let chain = vec![record("PAR/NDA/2026/0001", Method::WetInk)];
        assert!(is_signed(&chain, "PAR/NDA/2026/0001"));
        assert!(!is_signed(&chain, "PAR/IPA/2026/0001"));
        assert!(for_serial(&chain, "PAR/NDA/2026/0001").is_some());
        assert!(for_serial(&chain, "nope").is_none());
    }
}
