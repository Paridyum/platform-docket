//! Recording that an issued document came back executed.

use crate::adapters::store::Store;
use crate::adapters::{clock, registry::Registry};
use crate::config::Paths;
use crate::domain::chain::sha256;
use crate::domain::event::Kind;
use crate::domain::register::Entry;
use crate::domain::signature::{Method, Record, is_signed};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

/// What was recorded, for the report.
#[derive(Debug)]
pub struct Signed {
    /// The signature record as sealed.
    pub record: Record,
    /// Person the document was issued to.
    pub person_id: String,
    /// Document title.
    pub title: String,
}

/// Record a document as executed.
///
/// The scan, where one is given, is copied into the store and hashed. Holding
/// only a path would leave the evidence outside the thing that claims it
/// exists; holding only a hash would leave nothing to look at.
///
/// # Errors
/// Fails if the serial is unknown, already signed, the dates are malformed,
/// the signing date precedes issuance, a wet-ink signature has no scan, or
/// the ledger cannot be written.
// One more than clippy's cap now that the record store is a parameter (design §4): the extra
// argument is the `Store` port, and the rest are the independent facts of one signature — serial,
// method, two dates, an optional scan, the recorder. Bundling them into a struct would only move the
// same fields behind a name and read no more clearly.
#[allow(clippy::too_many_arguments)]
pub fn record(
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    serial: &str,
    method: Method,
    signed_on: time::Date,
    received_on: time::Date,
    scan: Option<&Path>,
    recorded_by: &str,
) -> Result<Signed> {
    let register = store.register()?;
    let Some(entry) = register.iter().find(|e| e.serial == serial) else {
        bail!("{serial} is not in the register; nothing was issued under it");
    };
    let signatures = store.signatures()?;
    if is_signed(&signatures, serial) {
        bail!(
            "{serial} is already recorded as executed. A signature is recorded once; \
             correcting one means recording why, not overwriting it"
        );
    }

    // A document cannot be signed before it existed.
    let issued = clock::parse_iso(&entry.issued_on)?;
    if signed_on < issued {
        bail!(
            "{serial} was issued on {} but the signing date given is {}; \
             a document cannot be signed before it exists",
            entry.issued_on,
            clock::to_iso(signed_on)?
        );
    }
    if received_on < signed_on {
        bail!("the received date precedes the signing date");
    }

    // Wet ink leaves no trail of its own, so the scan IS the evidence.
    if method.needs_scan() && scan.is_none() {
        bail!(
            "method {} leaves no evidence of its own: pass --scan <file> with the \
             returned copy, or record a different method",
            method.as_str()
        );
    }

    let (scan_sha256, scan_path) = match scan {
        Some(src) => {
            if !src.exists() {
                bail!("no such scan: {}", src.display());
            }
            let bytes = fs::read(src).with_context(|| format!("reading {}", src.display()))?;
            let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("pdf");
            let dir = paths.scans_dir(entry.person_id.as_str());
            fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
            let dest = dir.join(format!("{}-signed.{ext}", serial.replace('/', "-")));
            fs::copy(src, &dest).with_context(|| format!("copying to {}", dest.display()))?;
            let rel = dest
                .strip_prefix(&paths.store)
                .unwrap_or(&dest)
                .to_string_lossy()
                .into_owned();
            (Some(sha256(&bytes)), Some(rel))
        }
        None => (None, None),
    };

    let sealed = store.append_signature(Record {
        serial: serial.to_owned(),
        signed_on: clock::to_iso(signed_on)?,
        method,
        scan_sha256,
        scan_path,
        received_on: clock::to_iso(received_on)?,
        recorded_by: recorded_by.to_owned(),
        prev_sha256: String::new(),
        entry_sha256: String::new(),
    })?;

    // The event log carries the history; the ledger carries the evidence.
    crate::application::personnel::record_event(
        store,
        &entry.person_id,
        Kind::DocumentSigned,
        Some(serial),
        recorded_by,
        received_on,
    )?;

    let _ = registry;
    Ok(Signed {
        record: sealed,
        person_id: entry.person_id.to_string(),
        title: entry.title.clone(),
    })
}

/// Documents issued to a person that are owed a signature and have none.
///
/// Only `signed` and `collected` documents are owed one: a certificate is
/// issued BY Paridyum and nobody signs it back.
///
/// # Errors
/// Fails if the registry or store cannot be read.
pub fn outstanding(
    store: &dyn Store,
    registry: &Registry,
    person_id: &crate::domain::identity::PersonId,
) -> Result<Vec<Entry>> {
    let by_code = registry.lifecycle.by_code()?;
    let signatures = store.signatures()?;
    Ok(store
        .register()?
        .into_iter()
        .filter(|e| &e.person_id == person_id)
        .filter(|e| {
            by_code
                .get(e.code.as_str())
                .is_some_and(|d| d.needs_countersignature())
        })
        .filter(|e| !is_signed(&signatures, &e.serial))
        .collect())
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use crate::domain::signature::Method;

    #[test]
    fn wet_ink_is_the_only_method_whose_evidence_is_the_scan() {
        assert!(Method::WetInk.needs_scan());
        assert!(!Method::AadhaarEsign.needs_scan());
    }
}
