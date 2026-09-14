//! The register of everyone Paridyum has engaged.

use crate::adapters::registry::Registry;
use crate::adapters::store::Store;
use crate::domain::identity::Standing;
use anyhow::Result;

/// One line of the roster.
#[derive(Debug)]
pub struct Line {
    /// Permanent, opaque person identifier. Names the archive folder.
    pub person_id: String,
    /// Identifier of the current engagement.
    pub engagement_id: String,
    /// Record slug, or `(erased)` once the record is gone.
    pub slug: String,
    /// Person class.
    pub class: String,
    /// Current standing.
    pub standing: Standing,
    /// Name, where the record still exists.
    pub name: Option<String>,
    /// Role, where the record still exists.
    pub role: Option<String>,
    /// Engagement start, where recorded.
    pub start_date: Option<String>,
    /// Engagement end, where recorded.
    pub end_date: Option<String>,
    /// Documents issued to this person.
    pub documents: usize,
}

/// Everyone, with the current standing of each.
///
/// Built from the LEDGER rather than from the files on disk, so a person
/// whose record has been erased still appears with their identifier. A
/// roster that lost people when their data was erased would not be a
/// register of who was engaged.
///
/// # Errors
/// Fails if the ledger, register or a person record cannot be read.
pub fn build(store: &dyn Store, _registry: &Registry) -> Result<Vec<Line>> {
    let ledger = store.ledger()?;
    let register = store.register()?;

    let latest = crate::domain::identity::current(&ledger);

    let mut out = Vec::new();
    for r in latest {
        let person = if r.standing == Standing::Erased {
            None
        } else {
            store.person(&r.slug).ok()
        };
        out.push(Line {
            person_id: r.person_id.to_string(),
            engagement_id: r.engagement_id.to_string(),
            slug: if r.standing == Standing::Erased {
                "(erased)".to_owned()
            } else {
                r.slug.clone()
            },
            class: r.class.clone(),
            standing: r.standing,
            name: person
                .as_ref()
                .and_then(|p| p.string("identity.legal_name").map(ToOwned::to_owned)),
            role: person
                .as_ref()
                .and_then(|p| p.string("engagement.role").map(ToOwned::to_owned)),
            start_date: person
                .as_ref()
                .and_then(|p| p.string("engagement.start_date").map(ToOwned::to_owned)),
            end_date: person
                .as_ref()
                .and_then(|p| p.string("engagement.end_date").map(ToOwned::to_owned)),
            documents: register
                .iter()
                .filter(|e| e.person_id == r.person_id)
                .count(),
        });
    }
    Ok(out)
}

/// Counts by standing, for the summary line.
#[must_use]
pub fn tally(lines: &[Line]) -> (usize, usize, usize) {
    let count = |s: Standing| lines.iter().filter(|l| l.standing == s).count();
    (
        count(Standing::Active),
        count(Standing::Separated),
        count(Standing::Erased),
    )
}
