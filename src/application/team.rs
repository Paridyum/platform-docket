//! What Paridyum publishes about its officers, joined from the registries that own it.

use crate::adapters::registry::Registry;
use anyhow::{Result, bail};
use serde::Serialize;

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
    crate::application::issue::resolve_signatories(registry, &stand_in, doc)
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
