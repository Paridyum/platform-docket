//! The negotiated terms of one issuance, checked against what its document kind declares.
//!
//! RFC-0136 D7, review M5. An agreement's term, notice period and schedules are agreed per
//! issuance, so they cannot live in the registry or the counterparty record. Rendered as blank
//! `\dkfill` lines they made the registered, hash-verified MOU a blank form, and `docket verify`
//! then proved nothing about what was agreed. They are now an input, validated strictly before a
//! serial exists, and read bare by the template, so the document cannot render without them and
//! their values are inside the PDF's hash.

use crate::domain::definitions::Definitions;
use crate::domain::escape::command_name;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::BTreeMap;

/// The prefix a term's command carries: `term_months` renders as `\dkTTermMonths`.
pub const PREFIX: &str = "dkT";

/// One term a document kind requires, and why it exists.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Term {
    /// The key in the terms file.
    pub key: String,
    /// What it is for. A term with no purpose must not exist, as for a field.
    pub purpose: String,
}

/// Read a terms file against the terms `declared`, as definitions a template reads.
///
/// STRICT: every declared key, each a non-empty scalar, and nothing else. An extra key is how a
/// contact person's name would arrive, so it is refused rather than ignored.
///
/// # Errors
/// Fails on unparseable YAML, an undeclared key, a missing or empty term, or a non-scalar value.
pub fn read(declared: &[Term], text: &str) -> Result<Definitions> {
    let given: BTreeMap<String, Value> =
        serde_yaml::from_str(text).context("the terms file is not a YAML mapping")?;
    let mut defs = Definitions::new();
    if let Some(extra) = given
        .keys()
        .find(|k| !declared.iter().any(|t| &t.key == *k))
    {
        bail!(
            "term {extra:?} is not declared for this document; it takes exactly: {}",
            keys(declared)
        );
    }
    for term in declared {
        let value = match given.get(&term.key) {
            Some(Value::String(s)) => s.trim().to_owned(),
            Some(Value::Number(n)) => n.to_string(),
            Some(_) => bail!("term {:?} must be text or a number", term.key),
            None => String::new(),
        };
        if value.is_empty() {
            bail!(
                "term {:?} is missing or empty ({})",
                term.key,
                term.purpose.trim()
            );
        }
        defs.set(format!("{PREFIX}{}", command_name(&term.key)), &value);
    }
    Ok(defs)
}

/// The declared keys, for a message.
#[must_use]
pub fn keys(declared: &[Term]) -> String {
    declared
        .iter()
        .map(|t| t.key.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    fn declared() -> Vec<Term> {
        ["term_months", "sessions"]
            .into_iter()
            .map(|k| Term {
                key: k.to_owned(),
                purpose: "test".to_owned(),
            })
            .collect()
    }

    #[test]
    fn every_declared_term_becomes_a_command() {
        let defs = read(&declared(), "term_months: 12\nsessions: Two talks\n").expect("valid");
        assert_eq!(defs.get("dkTTermMonths"), Some("12"));
        assert_eq!(defs.get("dkTSessions"), Some("Two talks"));
    }

    #[test]
    fn a_missing_an_empty_or_an_undeclared_term_is_refused() {
        for bad in [
            "term_months: 12\n",
            "term_months: 12\nsessions: '  '\n",
            "term_months: 12\nsessions: x\ncontact_name: A Person\n",
            "term_months: [1, 2]\nsessions: x\n",
        ] {
            assert!(read(&declared(), bad).is_err(), "{bad}");
        }
    }
}
