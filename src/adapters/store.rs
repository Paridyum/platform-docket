//! The records store: person records, the register, and issued PDFs.

use crate::adapters::registry::read_yaml;
use crate::config::Paths;
use crate::domain::chain::{Chained, seal};
use crate::domain::event::Event;
use crate::domain::identity::Record as IdentityRecord;
use crate::domain::person::Person;
use crate::domain::register::Entry;
use anyhow::{Context, Result, bail};
use std::fs;
use std::io::Write as _;

/// Read one person record.
///
/// # Errors
/// Fails if the record is absent or does not parse.
pub fn person(paths: &Paths, slug: &str) -> Result<Person> {
    let path = paths.person(slug);
    if !path.exists() {
        bail!(
            "no person record: {}\n       create it from {}",
            path.display(),
            paths.people.join("_TEMPLATE.yaml").display()
        );
    }
    Ok(Person::new(slug.to_owned(), read_yaml(&path)?))
}

/// Read the whole register, in issue order.
///
/// A malformed line is an error rather than a skip. Silently ignoring one
/// would break the hash chain in a way that looks like tampering later.
///
/// # Errors
/// Fails if the register cannot be read, or if any line is
/// malformed -- skipping a bad line would break the chain in a way that
/// looks like tampering later.
pub fn register(paths: &Paths) -> Result<Vec<Entry>> {
    let path = paths.register();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let entry: Entry = serde_json::from_str(line)
            .with_context(|| format!("{}: line {} is not a valid entry", path.display(), i + 1))?;
        out.push(entry);
    }
    Ok(out)
}

/// Append one sealed entry.
///
/// Opened in append mode and never rewritten, so an existing record cannot
/// be replaced by a bug in this function.
///
/// # Errors
/// Fails if the register cannot be created or appended to.
pub fn append(paths: &Paths, entry: &Entry) -> Result<()> {
    let path = paths.register();
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let line = serde_json::to_string(entry).context("serialising a register entry")?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("opening {} for append", path.display()))?;
    writeln!(file, "{line}").with_context(|| format!("appending to {}", path.display()))?;
    Ok(())
}

/// Move a rendered PDF into the store, returning its path relative to it.
///
/// # Errors
/// Fails if the destination cannot be created or the copy fails.
pub fn keep_pdf(
    paths: &Paths,
    slug: &str,
    file_name: &str,
    source: &std::path::Path,
) -> Result<String> {
    let dir = paths.issued_dir(slug);
    fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let dest = dir.join(file_name);
    fs::copy(source, &dest)
        .with_context(|| format!("copying {} to {}", source.display(), dest.display()))?;
    let relative = dest
        .strip_prefix(&paths.store)
        .unwrap_or(&dest)
        .to_string_lossy()
        .into_owned();
    Ok(relative)
}

/// Read a hash-chained JSONL file, in order.
///
/// A malformed line is an error rather than a skip. Silently ignoring one
/// would break the chain in a way that looks like tampering later.
///
/// # Errors
/// Fails if the file cannot be read or any line does not parse.
pub fn read_chain<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<Vec<T>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(line).with_context(|| {
            format!("{}: line {} is not a valid record", path.display(), i + 1)
        })?);
    }
    Ok(out)
}

/// Append one record to a chained file, sealing it onto the end.
///
/// Opened in append mode and never rewritten, so an existing record cannot
/// be replaced by a bug in this function.
///
/// # Errors
/// Fails if the file cannot be created, sealed onto, or appended to.
pub fn append_chain<T>(path: &std::path::Path, record: T) -> Result<T>
where
    T: Chained + serde::de::DeserializeOwned + serde::Serialize,
{
    let existing: Vec<T> = read_chain(path)?;
    let sealed = seal(record, existing.last())?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let line = serde_json::to_string(&sealed).context("serialising a record")?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("opening {} for append", path.display()))?;
    writeln!(file, "{line}").with_context(|| format!("appending to {}", path.display()))?;
    Ok(sealed)
}

/// The identity ledger: which identifiers have been issued.
///
/// # Errors
/// Fails if the ledger cannot be read or a line does not parse.
pub fn ledger(paths: &Paths) -> Result<Vec<IdentityRecord>> {
    read_chain(&paths.ledger())
}

/// The signature ledger.
///
/// # Errors
/// Fails if the ledger cannot be read or a line does not parse.
pub fn signatures(paths: &Paths) -> Result<Vec<crate::domain::signature::Record>> {
    read_chain(&paths.signatures())
}

/// The personnel event log.
///
/// # Errors
/// Fails if the log cannot be read or a line does not parse.
pub fn events(paths: &Paths) -> Result<Vec<Event>> {
    read_chain(&paths.events())
}

/// Every person slug with a record on disk, sorted.
///
/// # Errors
/// Fails if the people directory cannot be listed.
pub fn slugs(paths: &Paths) -> Result<Vec<String>> {
    if !paths.people.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(&paths.people)
        .with_context(|| format!("reading {}", paths.people.display()))?
    {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "yaml")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && !stem.starts_with('_')
        {
            out.push(stem.to_owned());
        }
    }
    out.sort();
    Ok(out)
}
