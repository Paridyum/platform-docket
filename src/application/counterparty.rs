//! Enrolling and listing the organisations documents are issued to (RFC-0136).
//!
//! FILE STORE ONLY, for now. The records live under `store/counterparties/`; the Postgres store has
//! no table for them until a platform-tenancy migration adds one, so the CLI refuses these
//! commands there rather than writing to a disk the served backend never reads (DECISION-DEBT).

use crate::config::Paths;
use crate::domain::counterparty::{Counterparty, CounterpartyId, next_id};
use anyhow::{Context, Result, bail};
use std::fs;

/// The retention class in `registry/retention.yaml` that governs a counterparty record.
///
/// Named here so `check` can refuse a registry that does not declare it (REQ-0003: every record
/// is retained on a stated schedule, and a schedule nobody wrote down is not one).
pub const RETENTION_CLASS: &str = "counterparty";

/// The three facts an operator supplies.
#[derive(Debug, Clone, Copy)]
pub struct Facts<'a> {
    /// The organisation's legal name.
    pub legal_name: &'a str,
    /// Its address.
    pub address: &'a str,
    /// The office that receives documents.
    pub office: &'a str,
}

/// Record a new organisation counterparty and allocate its identifier.
///
/// # Errors
/// Fails if the slug is malformed or taken, a fact is empty, or the record cannot be written.
pub fn enrol(paths: &Paths, slug: &str, facts: Facts<'_>) -> Result<Counterparty> {
    if slug.is_empty()
        || !slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        bail!("slug {slug:?} must be lowercase letters, digits and hyphens");
    }
    let path = paths.counterparty(slug);
    if path.exists() {
        bail!("a counterparty is already recorded at {}", path.display());
    }
    let existing = list(paths)?;
    let used: Vec<&CounterpartyId> = existing.iter().map(|c| &c.id).collect();
    let record = Counterparty {
        slug: slug.to_owned(),
        version: 1,
        id: next_id(used)?,
        legal_name: facts.legal_name.trim().to_owned(),
        address: facts.address.trim().to_owned(),
        office: facts.office.trim().to_owned(),
    };
    if let Some(problem) = record.problems().into_iter().next() {
        bail!("{problem}; nothing was recorded");
    }
    fs::create_dir_all(paths.counterparties())
        .with_context(|| format!("creating {}", paths.counterparties().display()))?;
    fs::write(&path, record.to_yaml()).with_context(|| format!("writing {}", path.display()))?;
    Ok(record)
}

/// Read one counterparty's record.
///
/// # Errors
/// Fails if there is no record by that slug, or it does not parse.
pub fn read(paths: &Paths, slug: &str) -> Result<Counterparty> {
    let path = paths.counterparty(slug);
    if !path.exists() {
        bail!("no counterparty {slug:?}; record one with `docket counterparty new {slug}`");
    }
    let body = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Counterparty::parse(slug, &body)
}

/// Every recorded counterparty, by slug.
///
/// # Errors
/// Fails if the directory cannot be listed or any record does not parse. One unreadable record
/// fails the whole list on purpose: allocation reads this, and skipping a record it cannot read
/// could hand its identifier out a second time.
pub fn list(paths: &Paths) -> Result<Vec<Counterparty>> {
    let dir = paths.counterparties();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut slugs = Vec::new();
    for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "yaml")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
        {
            slugs.push(stem.to_owned());
        }
    }
    slugs.sort();
    slugs.iter().map(|s| read(paths, s)).collect()
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sandbox() -> (tempfile::TempDir, Paths) {
        let dir = tempfile::tempdir().expect("temp dir");
        let paths = Paths {
            root: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
            people: dir.path().join("people"),
            store: dir.path().join("store"),
        };
        (dir, paths)
    }

    const FACTS: Facts<'static> = Facts {
        legal_name: "Example University",
        address: "1 College Road, Dehradun",
        office: "The Registrar",
    };

    #[test]
    fn enrolling_allocates_identifiers_in_sequence_and_lists_them() {
        let (_dir, paths) = sandbox();
        let a = enrol(&paths, "example-university", FACTS).expect("first");
        let b = enrol(&paths, "example-incubator", FACTS).expect("second");
        assert_eq!(a.id.seq(), Some(1));
        assert_eq!(b.id.seq(), Some(2));
        let all = list(&paths).expect("list");
        assert_eq!(all.len(), 2);
        assert_eq!(read(&paths, "example-university").expect("read"), a);
    }

    #[test]
    fn a_taken_slug_or_an_empty_fact_records_nothing() {
        let (_dir, paths) = sandbox();
        enrol(&paths, "example-university", FACTS).expect("first");
        assert!(enrol(&paths, "example-university", FACTS).is_err());
        let blank = Facts {
            office: " ",
            ..FACTS
        };
        assert!(enrol(&paths, "another", blank).is_err());
        assert!(enrol(&paths, "Bad Slug", FACTS).is_err());
        assert_eq!(list(&paths).expect("list").len(), 1);
    }
}
