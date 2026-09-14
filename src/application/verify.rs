//! Checking that a PDF is the document a serial was issued for.

use crate::adapters::blobs::Blobs;
use crate::domain::register::{Entry, sha256};
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

/// The outcome of a verification.
#[derive(Debug)]
pub struct Outcome {
    /// The register entry the serial resolves to.
    pub serial: String,
    /// Document title, for the report.
    pub title: String,
    /// Date it was issued.
    pub issued_on: String,
    /// Hash recorded at issuance.
    pub expected: String,
    /// Hash of the file presented.
    pub actual: String,
    /// The file that was checked.
    pub path: PathBuf,
    /// The code printed on the document, recomputed from the register.
    pub code: String,
}

impl Outcome {
    /// Whether the presented file is the issued document.
    #[must_use]
    pub fn matches(&self) -> bool {
        self.expected == self.actual
    }
}

/// Verify a presented file, or the stored one when none is given.
///
/// This compares against the hash RECORDED AT ISSUANCE and deliberately does
/// not re-render. A future TeX Live positions glyphs differently and would
/// change every re-rendered hash, so re-rendering would report forgery on
/// every genuine document the day the toolchain moved.
///
/// # Errors
/// Fails if the serial is not registered or the file cannot be read.
pub fn verify(
    blobs: &dyn Blobs,
    register: &[Entry],
    serial: &str,
    file: Option<&Path>,
) -> Result<Outcome> {
    let Some(entry) = register.iter().find(|e| e.serial == serial) else {
        bail!("{serial} is not in the register");
    };
    // A file presented by the person is read from disk; otherwise the stored copy is read back
    // through the blob port (design §11), so `verify` works against the served backend's Garage
    // store the same as against local files. The reported `path` is the file, or the blob key.
    let (bytes, path) = if let Some(f) = file {
        if !f.exists() {
            bail!("no such file: {}", f.display());
        }
        let bytes = fs::read(f).with_context(|| format!("reading {}", f.display()))?;
        (bytes, f.to_path_buf())
    } else {
        if !blobs.exists(&entry.pdf_path)? {
            bail!("no stored copy of {serial} in the blob store");
        }
        let bytes = blobs
            .get(&entry.pdf_path)
            .with_context(|| format!("reading the stored copy of {serial}"))?;
        (bytes, PathBuf::from(&entry.pdf_path))
    };
    Ok(Outcome {
        code: crate::domain::serial::verification_code(
            &entry.serial,
            &entry.slug,
            &entry.issued_on,
            &entry.template_sha256,
        ),
        serial: entry.serial.clone(),
        title: entry.title.clone(),
        issued_on: entry.issued_on.clone(),
        expected: entry.pdf_sha256.clone(),
        actual: sha256(&bytes),
        path,
    })
}
