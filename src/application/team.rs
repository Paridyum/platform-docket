//! What Paridyum publishes about its officers, joined from the registries that own it.

use crate::adapters::registry::Registry;
use crate::config::Paths;
use crate::domain::chain::sha256;
use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::fs;

/// One published officer, as the public surface consumes it.
///
/// A JOIN, NOT A FILE. `name` and `designation` come from `signatories.yaml` and the
/// summary from `team.yaml`, because the office someone holds must not be able to differ
/// between the document they sign and the page that names them — the same rule
/// `org.yaml` states for the office address, for the same reason.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Member {
    /// Signatory id. Stable, and the join key.
    pub id: String,
    /// Legal name, exactly as it is printed on a document.
    pub name: String,
    /// Full designation, e.g. `Chief Executive Officer (CEO)`.
    pub designation: String,
    /// Short form, where a layout has no room for the full one.
    pub short: String,
    /// One paragraph per locale.
    pub summary: std::collections::BTreeMap<String, String>,
    /// Who signs THIS officer's own documents, as short designations.
    ///
    /// A RECORDED FACT, NOT A PLEASANTRY, and the reason the public page can say something
    /// no personnel list can. Officer documents route to the Director; the Director's own
    /// therefore route to himself, and `docket` substitutes another signatory so that
    /// nobody signs their own document. Which one is not derivable from `lifecycle.yaml`.
    ///
    /// Computed with the SAME function issuance uses, so the page cannot disagree with the
    /// paper. Empty where it cannot be determined without a person record.
    pub documents_signed_by: Vec<String>,
}

/// Everyone Paridyum publishes, in declared order.
///
/// # Errors
/// Fails if a `team.yaml` entry names a signatory that does not exist, or names one who
/// is not active. Both are the same defect from a reader's point of view — a page
/// claiming an officer the company does not have — and it is exactly the kind of thing
/// that survives a review because the name looks right.
pub fn published(registry: &Registry) -> Result<Vec<Member>> {
    let mut entries: Vec<_> = registry.team.iter().filter(|m| m.public).collect();
    entries.sort_by_key(|m| (m.order, m.id.clone()));

    let mut out = Vec::with_capacity(entries.len());
    for entry in entries {
        let Some(sig) = registry.signatories.iter().find(|s| s.id == entry.id) else {
            bail!(
                "team.yaml names {:?}, which signatories.yaml does not declare; \
                 a page must not claim an officer the company does not have",
                entry.id
            );
        };
        if !sig.active {
            bail!(
                "team.yaml publishes {:?}, who is no longer an active signatory; \
                 set `public: false` rather than leaving them on the site",
                entry.id
            );
        }
        out.push(Member {
            id: sig.id.clone(),
            name: sig.legal_name.clone(),
            designation: sig.designation.clone(),
            short: sig.short.clone().unwrap_or_else(|| sig.id.clone()),
            // TRIMMED. `team.yaml` uses folded scalars so the paragraphs stay readable
            // in the registry, and a folded scalar carries a trailing newline that
            // renders as stray whitespace inside a page element.
            documents_signed_by: countersignatories(registry, sig),
            summary: entry
                .summary
                .iter()
                .map(|(k, v)| (k.clone(), v.trim().to_owned()))
                .collect(),
        });
    }
    Ok(out)
}

/// Who signs an officer's own documents, as short designations.
///
/// Runs `issue::resolve_signatories` against a stand-in record carrying only this
/// officer's legal name and class, which is all the substitution rule reads. Using the
/// real function rather than restating its logic is the whole point: a second
/// implementation would drift, and it would drift on a page making a claim about who
/// signed a legal document.
fn countersignatories(
    registry: &Registry,
    sig: &crate::adapters::registry::Signatory,
) -> Vec<String> {
    let Some(doc) = registry
        .lifecycle
        .documents
        .iter()
        .find(|d| d.code == "NDO")
    else {
        return Vec::new();
    };
    let stand_in = crate::domain::person::Person::new(
        sig.id.clone(),
        serde_yaml::from_str(&format!(
            "engagement: {{class: officer}}\nidentity: {{legal_name: {}}}\n",
            sig.legal_name
        ))
        .unwrap_or_default(),
    );
    crate::application::issue::resolve_signatories(
        registry,
        &stand_in,
        doc,
        // The signatory's own company: a countersignature is resolved among the
        // officers of the company whose office this person holds.
        sig.company.as_deref().unwrap_or("PAR"),
    )
    .iter()
    .filter_map(|id| {
        registry
            .signatories
            .iter()
            .find(|s| &s.id == id)
            .map(|s| s.short.clone().unwrap_or_else(|| s.id.clone()))
    })
    .collect()
}

/// The published team as JSON, for the web surface to commit.
///
/// EXPORTED RATHER THAN READ ACROSS REPOSITORIES. `web-paridyum` and `tool-docket` have
/// no shared build (ADR-0001), and a public page that reads a gitignored directory at
/// build time would be a page whose content depends on which machine built it. So this
/// emits a file, the file is reviewed in a diff like every other public claim, and
/// docket stays the source of truth for who holds which office.
///
/// # Errors
/// As `published`, or if the value cannot be serialised.
pub fn export_json(registry: &Registry) -> Result<String> {
    let members = published(registry)?;
    let mut json = serde_json::to_string_pretty(&members)?;
    json.push('\n');
    Ok(json)
}

/// What the last export was made from.
///
/// D-48. The export is a file committed in another repository, so nothing over there can tell
/// that `signatories.yaml` moved and nobody re-ran `docket team`. Nothing HERE could tell
/// either, until this — the digest of the two files the export is derived from, written when
/// the export runs and compared on every `docket check`.
///
/// It cannot prove the other repository actually received it. It proves the narrower thing that
/// was actually going wrong: an export that no longer matches the registries it came from.
#[derive(Debug, Serialize, serde::Deserialize)]
pub struct Export {
    /// Digest of the sources at the moment `docket team` last ran.
    pub source_digest: String,
    /// ISO date of that run.
    pub exported_on: String,
    /// How many officers it carried, so a silent truncation is visible.
    pub members: usize,
}

/// Digest of every file the team export is derived from.
///
/// Both, because the export is a join: `signatories.yaml` supplies name and designation and
/// `team.yaml` supplies the summary and the order. A change to either changes the output, so a
/// digest of one alone would miss half the ways it can go stale.
///
/// # Errors
/// Fails if either registry file cannot be read.
pub fn source_digest(paths: &Paths) -> Result<String> {
    let mut buf = Vec::new();
    for name in ["signatories", "team"] {
        let path = paths.registry(name);
        buf.extend_from_slice(
            &fs::read(&path).with_context(|| format!("reading {}", path.display()))?,
        );
    }
    Ok(sha256(&buf))
}

/// Where the export receipt lives. In the store, not the registry: it records what happened
/// rather than what is intended, which is the same reason the register is not a registry.
fn receipt_path(paths: &Paths) -> std::path::PathBuf {
    paths.store.join("exports").join("team.json")
}

/// Record that an export was taken, and from what.
///
/// # Errors
/// Fails if the receipt cannot be written.
pub fn record_export(paths: &Paths, registry: &Registry, on: &str) -> Result<()> {
    let receipt = Export {
        source_digest: source_digest(paths)?,
        exported_on: on.to_owned(),
        members: published(registry)?.len(),
    };
    let path = receipt_path(paths);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(
        &path,
        format!("{}\n", serde_json::to_string_pretty(&receipt)?),
    )
    .with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

/// Whether the last export still matches the registries, and why not where it does not.
///
/// `Ok(None)` means current. `Ok(Some(reason))` is a warning for `docket check` to print — not
/// an error, because a stale export is a thing to go and fix rather than a reason to refuse
/// every other operation.
///
/// **NO RECEIPT IS SILENT, AND THAT IS DELIBERATE.** The receipt lives in `store/`, which is
/// gitignored, so a fresh checkout and every CI run would otherwise warn forever — and
/// `lifecycle.yaml` already records what that costs: "a warning that is always present is one
/// an operator learns to scroll past, which is how the next genuine orphan hides."
///
/// The failure D-48 describes is narrower than "has anyone ever exported": it is *the
/// registries were edited and nobody re-exported*, which by definition needs a prior export to
/// detect. Scoping to that keeps the warning rare enough to mean something.
///
/// # Errors
/// Fails only if the registries cannot be read.
pub fn export_staleness(paths: &Paths) -> Result<Option<String>> {
    let current = source_digest(paths)?;
    let path = receipt_path(paths);
    let Ok(body) = fs::read_to_string(&path) else {
        return Ok(None);
    };
    let Ok(receipt) = serde_json::from_str::<Export>(&body) else {
        return Ok(Some(format!(
            "{} is unreadable; re-run `docket team`",
            path.display()
        )));
    };
    if receipt.source_digest == current {
        return Ok(None);
    }
    Ok(Some(format!(
        "signatories.yaml or team.yaml has changed since the officers page was exported on {}; \
         the public site still names the old set. Re-run `docket team > \
         ../../web/web-paridyum/team.json` and commit it there.",
        receipt.exported_on
    )))
}
