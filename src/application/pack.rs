//! Assembling a person's documents into a folder they can be handed.

use crate::adapters::registry::Registry;
use crate::adapters::store::Store;
use crate::config::Paths;
use crate::domain::escape;
use crate::domain::lifecycle::{Document, Kind};
use crate::domain::register::Entry;
use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

/// What the recipient has to DO with a document.
///
/// Grouping by action rather than by document type, because "which of these
/// must I sign and send back" is the only question someone opening the folder
/// actually has. Derived from the registry, never listed by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Action {
    /// Both parties sign; a signed copy comes back.
    SignAndReturn,
    /// Filled in by the person and returned.
    CompleteAndReturn,
    /// Theirs to keep. Nothing comes back.
    ForRecords,
    /// Offered, never required. Declining changes nothing.
    Optional,
}

impl Action {
    /// The folder this action's documents go into.
    #[must_use]
    pub fn folder(self) -> &'static str {
        match self {
            Self::SignAndReturn => "01_Sign_and_return",
            Self::CompleteAndReturn => "02_Complete_and_return",
            Self::ForRecords => "03_For_your_records",
            Self::Optional => "04_Optional",
        }
    }

    /// The heading used on the cover sheet.
    #[must_use]
    pub fn heading(self) -> &'static str {
        match self {
            Self::SignAndReturn => "Sign and return",
            Self::CompleteAndReturn => "Complete and return",
            Self::ForRecords => "For your records",
            Self::Optional => "Optional",
        }
    }
}

/// Which action a document implies.
///
/// `owed` is the set of codes some stage requires. A document no stage owes is
/// optional by construction: that is how the publicity consent stays a genuine
/// choice rather than a box on a checklist.
#[must_use]
pub fn action_for(doc: &Document, owed: &BTreeSet<&str>) -> Action {
    if !owed.contains(doc.code.as_str()) {
        return Action::Optional;
    }
    if doc.kind == Kind::Signed || doc.countersigned_by.is_some() {
        return Action::SignAndReturn;
    }
    if doc.kind == Kind::Collected {
        return Action::CompleteAndReturn;
    }
    Action::ForRecords
}

/// A file placed in the pack.
#[derive(Debug)]
pub struct Placed {
    /// Path relative to the pack root.
    pub relative: String,
    /// Serial of the document.
    pub serial: String,
    /// SHA-256 of the copied file.
    pub sha256: String,
}

/// What was assembled.
#[derive(Debug)]
pub struct Pack {
    /// The pack root.
    pub root: PathBuf,
    /// Files placed, in folder order.
    pub files: Vec<Placed>,
    /// The cover sheet's serial.
    pub cover_serial: String,
}

/// A filesystem-safe name for a document title.
fn file_name(index: usize, title: &str) -> String {
    let cleaned: String = title
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    let squeezed = cleaned
        .split('_')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    format!("{index:02}_{squeezed}.pdf")
}

/// Group a person's issued documents by what must be done with them.
///
/// # Errors
/// Fails if the registry or register cannot be read.
pub fn grouped(
    store: &dyn Store,
    registry: &Registry,
    slug: &str,
    company: Option<&str>,
) -> Result<Vec<(Action, Vec<Entry>)>> {
    let who = crate::application::personnel::identify_at(store, slug, company)?;
    let by_code = registry.lifecycle.by_code()?;
    let owed: BTreeSet<&str> = registry
        .lifecycle
        .stages
        .iter()
        .flat_map(|s| s.owes.iter().map(String::as_str))
        .collect();

    let mut buckets: Vec<(Action, Vec<Entry>)> = [
        Action::SignAndReturn,
        Action::CompleteAndReturn,
        Action::ForRecords,
        Action::Optional,
    ]
    .into_iter()
    .map(|a| (a, Vec::new()))
    .collect();

    for e in store.register()? {
        if e.person_id != who.person_id {
            continue;
        }
        // The cover sheet describes the pack; it is not an item in it.
        if e.code == "PCK" {
            continue;
        }
        let Some(doc) = by_code.get(e.code.as_str()) else {
            continue;
        };
        let action = action_for(doc, &owed);
        if let Some(slot) = buckets.iter_mut().find(|(a, _)| *a == action) {
            slot.1.push(e);
        }
    }
    Ok(buckets)
}

/// Build the pack folder for one person.
///
/// Copies rather than moves: the store stays authoritative, and the pack is a
/// distribution copy that can be rebuilt or deleted without touching the
/// record.
///
/// # Errors
/// Fails if the person has no issued documents, or the pack cannot be written.
pub fn assemble(
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    company: Option<&str>,
    out: &Path,
    date: time::Date,
    by: &str,
) -> Result<Pack> {
    let who = crate::application::personnel::identify_at(store, slug, company)?;
    let buckets = grouped(store, registry, slug, company)?;
    if buckets.iter().all(|(_, v)| v.is_empty()) {
        bail!("{slug} has no issued documents; run `docket onboard {slug}` first");
    }

    let root = out.join(who.person_id.as_str());
    if root.exists() {
        fs::remove_dir_all(&root).with_context(|| format!("clearing {}", root.display()))?;
    }
    fs::create_dir_all(&root).with_context(|| format!("creating {}", root.display()))?;

    // The cover sheet needs the contents, so it is issued after grouping and
    // before the files are placed.
    let cover = issue_cover(store, paths, registry, slug, &buckets, date, by)?;

    let mut files = Vec::new();
    for (action, entries) in &buckets {
        if entries.is_empty() {
            continue;
        }
        let dir = root.join(action.folder());
        fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
        for (i, e) in entries.iter().enumerate() {
            let src = paths.store.join(&e.pdf_path);
            if !src.exists() {
                bail!("{}: issued PDF is missing from the store", e.serial);
            }
            let name = file_name(i + 1, &e.title);
            let dest = dir.join(&name);
            fs::copy(&src, &dest).with_context(|| format!("copying to {}", dest.display()))?;
            let bytes = fs::read(&dest).with_context(|| format!("reading {}", dest.display()))?;
            files.push(Placed {
                relative: format!("{}/{name}", action.folder()),
                serial: e.serial.clone(),
                sha256: crate::domain::chain::sha256(&bytes),
            });
        }
    }

    // Cover sheet at the top, named so it sorts first in any file browser.
    let cover_src = paths.store.join(&cover.pdf_path);
    let cover_dest = root.join("00_Read_Me_First.pdf");
    fs::copy(&cover_src, &cover_dest)
        .with_context(|| format!("copying to {}", cover_dest.display()))?;
    let cover_bytes = fs::read(&cover_dest).context("reading the cover sheet")?;
    files.insert(
        0,
        Placed {
            relative: "00_Read_Me_First.pdf".to_owned(),
            serial: cover.serial.clone(),
            sha256: crate::domain::chain::sha256(&cover_bytes),
        },
    );

    // A manifest, so the pack verifies on its own with `shasum -c`.
    let mut manifest = String::new();
    for f in &files {
        let _ = writeln!(manifest, "{}  {}", f.sha256, f.relative);
    }
    fs::write(root.join("MANIFEST-sha256.txt"), manifest).context("writing the pack manifest")?;

    Ok(Pack {
        root,
        files,
        cover_serial: cover.serial,
    })
}

/// Issue the cover sheet, with the pack contents rendered into it.
fn issue_cover(
    store: &dyn Store,
    paths: &Paths,
    registry: &Registry,
    slug: &str,
    buckets: &[(Action, Vec<Entry>)],
    date: time::Date,
    by: &str,
) -> Result<Entry> {
    let by_code = registry.lifecycle.by_code()?;
    let Some(doc) = by_code.get("PCK") else {
        bail!("PCK (the pack cover sheet) is not declared in lifecycle.yaml");
    };
    let person = store.person(slug)?;

    let mut extra = crate::domain::definitions::Definitions::new();
    for (action, entries) in buckets {
        let mut body = String::new();
        for e in entries {
            let _ = write!(
                body,
                "\\item {} \\hfill \\texttt{{\\footnotesize {}}}",
                escape::tex(&e.title),
                escape::tex(&e.serial)
            );
        }
        if body.is_empty() {
            body.push_str("\\item \\textit{nothing in this group}");
        }
        let name = match action {
            Action::SignAndReturn => "dkPackSignItems",
            Action::CompleteAndReturn => "dkPackCompleteItems",
            Action::ForRecords => "dkPackRecordsItems",
            Action::Optional => "dkPackOptionalItems",
        };
        extra.set_raw(name, body);
    }

    crate::application::issue::one_with(
        store,
        paths,
        registry,
        &person,
        doc,
        date,
        Some(&extra),
        by,
    )
}

/// Where a pack is written by default.
#[must_use]
pub fn default_out(paths: &Paths) -> PathBuf {
    paths.store.join("packs")
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{Action, file_name};

    #[test]
    fn folders_sort_in_the_order_a_reader_needs_them() {
        let mut names = [
            Action::Optional.folder(),
            Action::ForRecords.folder(),
            Action::SignAndReturn.folder(),
            Action::CompleteAndReturn.folder(),
        ];
        names.sort_unstable();
        assert_eq!(
            names,
            [
                "01_Sign_and_return",
                "02_Complete_and_return",
                "03_For_your_records",
                "04_Optional"
            ],
            "the numeric prefix is what makes a file browser show them in order"
        );
    }

    #[test]
    fn file_names_are_safe_and_ordered() {
        assert_eq!(
            file_name(1, "Information Security & Acceptable Use Policy"),
            "01_Information_Security_Acceptable_Use_Policy.pdf"
        );
        assert_eq!(
            file_name(12, "Parent / Guardian Consent"),
            "12_Parent_Guardian_Consent.pdf"
        );
    }

    #[test]
    fn a_title_of_only_punctuation_still_produces_a_name() {
        assert_eq!(file_name(3, "///"), "03_.pdf");
    }
}
