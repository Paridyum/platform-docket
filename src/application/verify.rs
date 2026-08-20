//! Checking that a PDF is the document a serial was issued for.

use crate::config::Paths;
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
    paths: &Paths,
    register: &[Entry],
    serial: &str,
    file: Option<&Path>,
) -> Result<Outcome> {
    let Some(entry) = register.iter().find(|e| e.serial == serial) else {
        bail!("{serial} is not in the register");
    };
    let path = file.map_or_else(|| paths.store.join(&entry.pdf_path), Path::to_path_buf);
    if !path.exists() {
        bail!("no such file: {}", path.display());
    }
    let bytes = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
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
