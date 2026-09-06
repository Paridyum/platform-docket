//! Rendering the messages that carry documents to the people who need them.

use crate::adapters::{registry::Registry, store};
use crate::config::Paths;
use crate::domain::person::Person;
use anyhow::{Result, bail};
use serde::Deserialize;
use std::collections::BTreeMap;

/// Who a message goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Recipient {
    /// The person themselves.
    Intern,
    /// Their parent or lawful guardian.
    Guardian,
    /// Their academic institution.
    Institution,
}

impl Recipient {
    /// The name used in the registry and the report.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Intern => "intern",
            Self::Guardian => "guardian",
            Self::Institution => "institution",
        }
    }
}

/// One message in the sequence.
#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    /// Stable identifier.
    pub id: String,
    /// The lifecycle stage at which it is sent.
    pub stage: String,
    /// When to send it, in prose, for the person doing the sending.
    pub when: String,
    /// Who it goes to.
    pub to: Recipient,
    /// Subject line, with placeholders.
    pub subject: String,
    /// Document codes carried with it.
    pub attach: Vec<String>,
    /// Body, with placeholders.
    pub body: String,
}

/// Wrapper matching the top level of `communications.yaml`.
#[derive(Debug, Deserialize)]
pub struct MessageFile {
    /// The declared messages.
    pub messages: Vec<Message>,
}

/// A message with its placeholders filled and its attachments resolved.
#[derive(Debug)]
pub struct Rendered {
    /// Which message this is.
    pub id: String,
    /// Stage it belongs to.
    pub stage: String,
    /// Who it goes to.
    pub to: Recipient,
    /// The address, where the record holds one.
    pub address: Option<String>,
    /// When to send it.
    pub when: String,
    /// Filled subject.
    pub subject: String,
    /// Filled body.
    pub body: String,
    /// Attachments: (code, title, serial or a note that it is not issued).
    pub attachments: Vec<(String, String, String)>,
    /// Attachments the message names that have not been issued.
    pub missing: Vec<String>,
}

/// Build the substitution table for one person.
///
/// Values come from the record and the registry, never from a template, so a
/// message cannot say something the documents do not.
fn placeholders(registry: &Registry, person: &Person) -> BTreeMap<&'static str, String> {
    let get = |path: &str| person.string(path).unwrap_or_default().to_owned();
    let org = |path: &str| -> String {
        registry
            .org_node()
            .and_then(|o| path.split('.').try_fold(o, |n, k| n.get(k)))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned()
    };
    let sig = |id: &str, field: &str| -> String {
        registry
            .signatories
            .iter()
            .find(|s| s.id == id)
            .map(|s| match field {
                "name" => s.legal_name.clone(),
                _ => s.designation.clone(),
            })
            .unwrap_or_default()
    };

    let mut m = BTreeMap::new();
    m.insert("legal_name", get("identity.legal_name"));
    m.insert("first_name", {
        let preferred = get("identity.preferred_name");
        if preferred.is_empty() {
            get("identity.legal_name")
        } else {
            preferred
        }
    });
    m.insert("role", get("engagement.role"));
    m.insert("supervisor", get("engagement.supervisor"));
    m.insert("institution", get("education.institution"));
    m.insert("degree", get("education.degree"));
    m.insert("guardian_name", get("guardian.name"));
    m.insert("org_email", org("email"));
    m.insert("security_email", org("security.email"));
    m.insert("sender_name", sig("cto", "name"));
    m.insert("sender_designation", sig("cto", "designation"));

    // Dates print in the long form the documents use, not as ISO.
    for (path, key) in [
        ("engagement.start_date", "start_date"),
        ("engagement.end_date", "end_date"),
    ] {
        let raw = get(path);
        let long = crate::adapters::clock::parse_iso(&raw)
            .map(crate::adapters::clock::to_long)
            .unwrap_or(raw);
        m.insert(key, long);
    }
    m
}

/// Replace every `{placeholder}`, and report any that had no value.
fn fill(text: &str, values: &BTreeMap<&'static str, String>) -> (String, Vec<String>) {
    let mut out = text.to_owned();
    let mut unresolved = Vec::new();
    for (k, v) in values {
        if v.is_empty() {
            continue;
        }
        out = out.replace(&format!("{{{k}}}"), v);
    }
    // Anything still in braces had no value. Left visible rather than blanked:
    // a message with a hole in it must not look finished.
    let mut rest = out.as_str();
    while let Some(open) = rest.find('{') {
        let Some(close) = rest[open..].find('}') else {
            break;
        };
        let name = &rest[open + 1..open + close];
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
            unresolved.push(name.to_owned());
        }
        rest = &rest[open + close + 1..];
    }
    unresolved.sort_unstable();
    unresolved.dedup();
    (out, unresolved)
}

/// Render every message due at or before a person's current stage.
///
/// # Errors
/// Fails if the registry, record or register cannot be read, or a message
/// names a stage the lifecycle does not declare.
pub fn due(paths: &Paths, registry: &Registry, slug: &str) -> Result<Vec<Rendered>> {
    let who = crate::application::personnel::identify(paths, slug)?;
    let person = store::person(paths, slug)?;
    let values = placeholders(registry, &person);
    let by_code = registry.lifecycle.by_code()?;
    let register = store::register(paths)?;

    // Cumulative, like the documents: reaching `active` does not mean the
    // preboarding messages were sent.
    let reached = registry
        .lifecycle
        .owed_upto_for(person.stage(), Some(person.class()))?;
    let _ = reached;
    let stages: Vec<&str> = registry
        .lifecycle
        .stages
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    let Some(at) = stages.iter().position(|s| *s == person.stage()) else {
        bail!("unknown stage {:?}", person.stage());
    };
    let reachable: Vec<&str> = stages.into_iter().take(at + 1).collect();

    let mut out = Vec::new();
    for m in &registry.messages {
        if !reachable.contains(&m.stage.as_str()) {
            continue;
        }
        // A message to a guardian is only due where there is a guardian.
        if m.to == Recipient::Guardian && !person.has_guardian() {
            continue;
        }
        if m.to == Recipient::Institution && !person.flag("education.institution_requires_noc") {
            continue;
        }

        let (subject, mut holes) = fill(&m.subject, &values);
        let (body, more) = fill(&m.body, &values);
        holes.extend(more);
        holes.sort_unstable();
        holes.dedup();

        let mut attachments = Vec::new();
        let mut missing = Vec::new();
        for code in &m.attach {
            let title = by_code
                .get(code.as_str())
                .map_or_else(|| code.clone(), |d| d.title.clone());
            // An attachment that has not been issued is LISTED, not hidden:
            // the operator is about to attach it to a real email.
            let issued = register
                .iter()
                .find(|e| e.person_id == who.person_id && &e.code == code);
            let serial = if let Some(e) = issued {
                e.serial.clone()
            } else {
                missing.push(code.clone());
                "NOT ISSUED".to_owned()
            };
            attachments.push((code.clone(), title, serial));
        }

        let address = match m.to {
            Recipient::Intern => person.string("contact.email").map(ToOwned::to_owned),
            Recipient::Guardian => person.string("guardian.email").map(ToOwned::to_owned),
            Recipient::Institution => person.string("education.email").map(ToOwned::to_owned),
        };

        out.push(Rendered {
            id: m.id.clone(),
            stage: m.stage.clone(),
            to: m.to,
            address,
            when: m.when.trim().to_owned(),
            subject,
            body: if holes.is_empty() {
                body
            } else {
                format!("{body}\n[unresolved placeholders: {}]", holes.join(" "))
            },
            attachments,
            missing,
        });
    }
    Ok(out)
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::fill;
    use std::collections::BTreeMap;

    fn values() -> BTreeMap<&'static str, String> {
        let mut m = BTreeMap::new();
        m.insert("first_name", "Asha".to_owned());
        m.insert("role", "Software Engineering Intern".to_owned());
        m.insert("end_date", String::new());
        m
    }

    #[test]
    fn placeholders_are_filled() {
        let (text, holes) = fill("Dear {first_name}, as a {role}.", &values());
        assert_eq!(text, "Dear Asha, as a Software Engineering Intern.");
        assert!(holes.is_empty());
    }

    #[test]
    fn an_empty_value_is_reported_rather_than_silently_blanked() {
        // A message with a hole must not look finished. Sending "your
        // internship ends on ." is worse than not sending it.
        let (text, holes) = fill("Ends on {end_date}.", &values());
        assert_eq!(text, "Ends on {end_date}.");
        assert_eq!(holes, ["end_date"]);
    }

    #[test]
    fn an_unknown_placeholder_is_reported() {
        let (_, holes) = fill("Hello {nobody_defined_this}.", &values());
        assert_eq!(holes, ["nobody_defined_this"]);
    }

    #[test]
    fn prose_braces_are_not_mistaken_for_placeholders() {
        let (text, holes) = fill("Use {SOME_ENV} or {a b}.", &values());
        assert_eq!(text, "Use {SOME_ENV} or {a b}.");
        assert!(holes.is_empty(), "{holes:?}");
    }
}
