//! Answering what a person still owes.

use crate::adapters::registry::Registry;
use crate::domain::person::Person;
use crate::domain::register::Entry;
use anyhow::Result;

/// One line of a status report.
#[derive(Debug, PartialEq, Eq)]
pub struct Line {
    /// Document code.
    pub code: String,
    /// Document title.
    pub title: String,
    /// The state this document is in for this person.
    pub state: State,
}

/// Where a document stands.
#[derive(Debug, PartialEq, Eq)]
pub enum State {
    /// Issued and recorded as executed.
    Signed(String),
    /// Issued, awaiting the executed copy back.
    Issued(String),
    /// Owed and not issued.
    Outstanding,
    /// Owed by the stage, but not applicable to this person.
    NotApplicable,
}

/// A person's obligations at their current stage.
#[derive(Debug)]
pub struct Report {
    /// Name as recorded, falling back to the slug.
    pub name: String,
    /// Slug the record is filed under.
    pub slug: String,
    /// The stage the record declares.
    pub stage: String,
    /// One line per document owed.
    pub lines: Vec<Line>,
}

impl Report {
    /// How many owed documents are issued.
    #[must_use]
    pub fn issued(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| matches!(l.state, State::Issued(_) | State::Signed(_)))
            .count()
    }

    /// How many are recorded as executed.
    ///
    /// A document that went out and has not come back is not finished, which
    /// is the distinction `issued()` alone cannot make.
    #[must_use]
    pub fn signed(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| matches!(l.state, State::Signed(_)))
            .count()
    }

    /// Codes issued but not yet returned executed.
    #[must_use]
    pub fn awaiting_signature(&self) -> Vec<&str> {
        self.lines
            .iter()
            .filter(|l| matches!(l.state, State::Issued(_)))
            .map(|l| l.code.as_str())
            .collect()
    }

    /// How many are owed at all, excluding the not-applicable ones.
    #[must_use]
    pub fn owed(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| !matches!(l.state, State::NotApplicable))
            .count()
    }

    /// Codes still outstanding.
    #[must_use]
    pub fn outstanding(&self) -> Vec<&str> {
        self.lines
            .iter()
            .filter(|l| l.state == State::Outstanding)
            .map(|l| l.code.as_str())
            .collect()
    }
}

/// Build the report.
///
/// # Errors
/// Fails if the lifecycle declares a duplicate code or unknown stage.
pub fn report(
    registry: &Registry,
    person: &Person,
    who: &crate::domain::identity::Record,
    register: &[Entry],
    signatures: &[crate::domain::signature::Record],
) -> Result<Report> {
    let by_code = registry.lifecycle.by_code()?;
    let mut lines = Vec::new();

    for code in registry.lifecycle.owed_upto(person.stage())? {
        let Some(doc) = by_code.get(code.as_str()) else {
            continue;
        };
        let state = if doc.required_if.is_some_and(|c| !person.satisfies(c)) {
            State::NotApplicable
        } else if let Some(e) = register
            .iter()
            .find(|e| e.person_id == who.person_id && e.code == code)
        {
            // A document that is owed a signature and has none is not done.
            if !doc.needs_countersignature()
                || crate::domain::signature::is_signed(signatures, &e.serial)
            {
                State::Signed(e.serial.clone())
            } else {
                State::Issued(e.serial.clone())
            }
        } else {
            State::Outstanding
        };
        lines.push(Line {
            code,
            title: doc.title.clone(),
            state,
        });
    }

    Ok(Report {
        name: person
            .string("name.legal")
            .unwrap_or(&person.slug)
            .to_owned(),
        slug: person.slug.clone(),
        stage: person.stage().to_owned(),
        lines,
    })
}
