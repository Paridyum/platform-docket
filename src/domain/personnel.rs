//! What may be recorded about a person, and what must be.

use crate::domain::person::Person;
use anyhow::{Result, bail};
use serde::Deserialize;
use std::collections::BTreeMap;

/// One person class, e.g. an unpaid intern.
#[derive(Debug, Clone, Deserialize)]
// `class` is the key in person-classes.yaml. Renaming the field to satisfy
// the lint would change the registry format that the file already uses.
#[allow(clippy::struct_field_names)]
pub struct Class {
    /// Class name, matching `engagement.class` in a record.
    pub class: String,
    /// Identifier prefix people of this class are issued.
    pub prefix: String,
    /// What the class is, and why its permissions are what they are.
    pub description: String,
    /// Field groups that may be collected.
    pub permits: Vec<String>,
    /// Dotted field paths that must be present.
    pub requires: Vec<String>,
    /// Groups or paths that must NOT be present.
    pub forbids: Vec<String>,
    /// Whether an engagement of this class is unpaid.
    #[serde(default)]
    pub unpaid: bool,
}

/// One field, and the reason it exists.
#[derive(Debug, Clone, Deserialize)]
pub struct Field {
    /// Dotted path in a person record.
    pub path: String,
    /// Group this field belongs to, matching a class's `permits`.
    pub group: String,
    /// Which store holds it.
    pub store: String,
    /// Why it is collected. A field without one should not be collected.
    pub purpose: String,
    /// Whether it may be absent.
    #[serde(default)]
    pub optional: bool,
}

/// The class and field registries together.
#[derive(Debug)]
pub struct Catalogue {
    /// Declared classes.
    pub classes: Vec<Class>,
    /// Declared fields.
    pub fields: Vec<Field>,
}

/// Wrapper matching the top level of `person-classes.yaml`.
#[derive(Debug, Deserialize)]
pub struct ClassFile {
    /// The declared classes.
    pub classes: Vec<Class>,
}

/// Wrapper matching the top level of `fields.yaml`.
#[derive(Debug, Deserialize)]
pub struct FieldFile {
    /// The declared fields.
    pub fields: Vec<Field>,
}

impl Catalogue {
    /// Look a class up by name.
    #[must_use]
    pub fn class(&self, name: &str) -> Option<&Class> {
        self.classes.iter().find(|c| c.class == name)
    }

    /// Index fields by dotted path, rejecting duplicates.
    ///
    /// # Errors
    /// Fails if a field path is declared twice, which would make its purpose
    /// and its store ambiguous.
    pub fn fields_by_path(&self) -> Result<BTreeMap<&str, &Field>> {
        let mut map = BTreeMap::new();
        for f in &self.fields {
            if map.insert(f.path.as_str(), f).is_some() {
                bail!("field {:?} is declared twice in fields.yaml", f.path);
            }
        }
        Ok(map)
    }

    /// Every group named by any field.
    #[must_use]
    pub fn groups(&self) -> Vec<&str> {
        let mut g: Vec<&str> = self.fields.iter().map(|f| f.group.as_str()).collect();
        g.sort_unstable();
        g.dedup();
        g
    }
}

/// A way in which a record does not match what its class declares.
#[derive(Debug, PartialEq, Eq)]
pub enum Violation {
    /// A field the class requires is absent.
    Missing {
        /// The dotted path that must be present.
        path: String,
    },
    /// A field still holds the template's placeholder.
    ///
    /// Separate from `Missing` because the two need different instructions and
    /// have different causes. `Missing` means nobody supplied a value; this means
    /// the template's value was left in place, which looks supplied and is not.
    Unfilled {
        /// The dotted path still holding `person::UNFILLED`.
        path: String,
    },
    /// A field is present that the class forbids.
    ///
    /// This is the important one. Collecting a PAN for an unpaid intern is
    /// not untidy, it is processing personal data for which no purpose
    /// exists -- the thing the DPDP Act's purpose limitation forbids.
    Forbidden {
        /// The dotted path that must not be present.
        path: String,
        /// Why this class must not hold it.
        reason: String,
    },
    /// A field is present that belongs in a different store.
    ///
    /// `person-classes.yaml` declares four stores and the reason for each.
    /// Emergency contact details are the case that matters: they are data about
    /// a THIRD PARTY who has no relationship with Paridyum, they are the
    /// shortest-retained class in the system, and a personnel record readable
    /// by a supervisor is not where they belong. The split was declared and
    /// nothing enforced it, so the file could hold them and pass.
    WrongStore {
        /// The dotted path found in the record.
        path: String,
        /// The store it is declared to live in.
        declared: String,
    },
    /// A field is present that no class permits, because nothing declares it.
    Undeclared {
        /// The dotted path found in the record.
        path: String,
    },
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing { path } => write!(f, "{path} is required and is not set"),
            Self::Unfilled { path } => write!(
                f,
                "{path} still holds the template's {} placeholder -- it looks filled in and is not",
                crate::domain::person::UNFILLED
            ),
            Self::Forbidden { path, reason } => write!(f, "{path} must not be collected: {reason}"),
            Self::WrongStore { path, declared } => write!(
                f,
                "{path} is declared to live in the {declared:?} store, not in the \
                 person record; move it there"
            ),
            Self::Undeclared { path } => write!(
                f,
                "{path} is not declared in fields.yaml; \
                 add it with a stated purpose or remove it"
            ),
        }
    }
}

/// One thing to ask when opening a record interactively.
#[derive(Debug, Clone)]
pub struct Question {
    /// Dotted path the answer is written to.
    pub path: String,
    /// Field group, used to order and head the questions.
    pub group: String,
    /// The declared purpose, shown as help. If it cannot be written down the
    /// field should not exist, so it is always worth showing.
    pub purpose: String,
    /// Whether the class requires an answer.
    pub required: bool,
}

/// What to ask when opening a record for this class.
///
/// **Derived from the catalogue, never listed.** The prompts therefore cannot
/// drift from the schema: a field added to `fields.yaml` is asked for, a field
/// a class forbids is never asked, and the help text is the stated purpose.
///
/// Only fields declared to live in the `personnel` store are asked. The person
/// record IS that store; emergency contact details belong in another one and
/// asking for them here would put them in the wrong file.
#[must_use]
pub fn questions(catalogue: &Catalogue, class: &Class) -> Vec<Question> {
    let mut out: Vec<Question> = catalogue
        .fields
        .iter()
        .filter(|f| f.store == "personnel")
        .filter(|f| class.permits.contains(&f.group))
        .filter(|f| !class.forbids.contains(&f.group) && !class.forbids.contains(&f.path))
        // `engagement.class` is decided by the command, not by an answer.
        .filter(|f| f.path != "engagement.class")
        .map(|f| Question {
            path: f.path.clone(),
            group: f.group.clone(),
            // FIRST SENTENCE ONLY. Several purposes in the catalogue run to a
            // paragraph because they carry the reasoning for the field; that
            // belongs in the file, not in front of someone answering a prompt.
            purpose: first_sentence(f.purpose.trim()),
            required: class.requires.contains(&f.path),
        })
        .collect();
    // Required first; then in the order a person would answer, not
    // alphabetically. Who they are, how to reach them, what the engagement is,
    // then everything else.
    out.sort_by(|a, b| {
        b.required
            .cmp(&a.required)
            .then_with(|| group_rank(&a.group).cmp(&group_rank(&b.group)))
            .then_with(|| field_rank(&a.path).cmp(&field_rank(&b.path)))
            .then_with(|| a.path.cmp(&b.path))
    });
    out
}

/// The order a person would answer these in.
fn group_rank(group: &str) -> usize {
    const ORDER: [&str; 6] = [
        "identity",
        "contact",
        "engagement",
        "education",
        "professional",
        "device",
    ];
    ORDER
        .iter()
        .position(|g| *g == group)
        .unwrap_or(ORDER.len())
}

/// A few fields that should lead their group regardless of alphabet.
///
/// Asking someone's date of birth before their name is the kind of small
/// wrongness that makes a tool feel like a form.
fn field_rank(path: &str) -> usize {
    const FIRST: [&str; 4] = [
        "identity.legal_name",
        "contact.email",
        "engagement.role",
        "education.institution",
    ];
    FIRST.iter().position(|p| *p == path).unwrap_or(FIRST.len())
}

/// The first sentence of a purpose, for use as prompt help.
fn first_sentence(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    match flat.find(". ") {
        Some(at) => flat.get(..=at).unwrap_or(&flat).trim().to_owned(),
        None => flat,
    }
}

/// Flatten a record into the dotted paths that actually carry a value.
///
/// A `null` counts as absent: a key present with no value has not been
/// collected, and treating it as collected would report a violation for a
/// field the template file merely mentions.
#[must_use]
pub fn present_paths(person: &Person) -> Vec<String> {
    let mut out = Vec::new();
    walk(&person.value, "", &mut out);
    out.sort();
    out
}

fn walk(value: &serde_yaml::Value, prefix: &str, out: &mut Vec<String>) {
    let Some(map) = value.as_mapping() else {
        return;
    };
    for (key, val) in map {
        let Some(key) = key.as_str() else { continue };
        let path = if prefix.is_empty() {
            key.to_owned()
        } else {
            format!("{prefix}.{key}")
        };
        match val {
            serde_yaml::Value::Null => {}
            serde_yaml::Value::Mapping(_) => {
                // A leaf group like `pronouns` is recorded as the group
                // itself, since fields.yaml declares it at that level.
                out.push(path.clone());
                walk(val, &path, out);
            }
            _ => out.push(path),
        }
    }
}

/// The nearest declared ancestor of a dotted path, if any.
fn ancestor_of<'a>(by_path: &BTreeMap<&'a str, &'a Field>, path: &str) -> Option<&'a Field> {
    let mut cursor = path;
    while let Some(cut) = cursor.rfind('.') {
        cursor = cursor.get(..cut)?;
        if let Some(f) = by_path.get(cursor) {
            return Some(f);
        }
    }
    None
}

/// Check a record against what its class declares.
///
/// `version` and `engagement.class` are structural rather than personal and
/// are not subject to the field catalogue.
///
/// # Errors
/// Fails if the field catalogue itself is malformed.
pub fn validate(catalogue: &Catalogue, class: &Class, person: &Person) -> Result<Vec<Violation>> {
    let by_path = catalogue.fields_by_path()?;
    let present = present_paths(person);
    let mut out = Vec::new();

    // Before anything else. A record that has not been filled in satisfies every
    // structural rule below it, which is the whole reason this exists.
    for path in crate::domain::person::unfilled(person) {
        out.push(Violation::Unfilled { path });
    }

    for required in &class.requires {
        if !present.iter().any(|p| p == required) {
            out.push(Violation::Missing {
                path: required.clone(),
            });
        }
    }

    for path in &present {
        if path == "version" || path == "engagement" || path == "engagement.class" {
            continue;
        }
        let Some(field) = by_path.get(path.as_str()) else {
            // A container that only groups declared leaves is fine.
            if by_path.keys().any(|k| k.starts_with(&format!("{path}."))) {
                continue;
            }
            // So is a sub-key of a field declared at a higher level:
            // `identity.pronouns` is one field, not three.
            if ancestor_of(&by_path, path).is_some() {
                continue;
            }
            out.push(Violation::Undeclared { path: path.clone() });
            continue;
        };
        // FORBIDDEN IS CHECKED FIRST, and the order matters. For an intern's
        // PAN both are true -- the class forbids it AND it belongs in another
        // store -- and "no statute requires this for an unpaid intern" is the
        // reason worth telling someone. "Wrong file" would send them to move it
        // rather than to delete it.
        if class.forbids.iter().any(|f| f == path || f == &field.group) {
            out.push(Violation::Forbidden {
                path: path.clone(),
                reason: format!(
                    "class {:?} forbids the {:?} group -- {}",
                    class.class, field.group, field.purpose
                ),
            });
        } else if !class.permits.contains(&field.group) {
            out.push(Violation::Forbidden {
                path: path.clone(),
                reason: format!(
                    "class {:?} does not permit the {:?} group",
                    class.class, field.group
                ),
            });
        } else if field.store != "personnel" {
            // Permitted for this class, but the person record IS the personnel
            // store, so a field declared to live elsewhere is in the wrong file
            // however legitimate the data is.
            out.push(Violation::WrongStore {
                path: path.clone(),
                declared: field.store.clone(),
            });
        }
    }

    Ok(out)
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{Catalogue, Violation, present_paths, questions, validate};
    use crate::domain::person::Person;

    fn catalogue() -> Catalogue {
        let classes: super::ClassFile = serde_yaml::from_str(
            r"
classes:
  - class: intern
    prefix: INT
    description: d
    permits: [identity, education]
    requires: [identity.legal_name, education.currently_enrolled]
    forbids: [statutory]
    unpaid: true
  - class: employee
    prefix: EMP
    description: d
    permits: [identity, education, statutory]
    requires: [identity.legal_name]
    forbids: []
",
        )
        .expect("classes parse");
        let fields: super::FieldFile = serde_yaml::from_str(
            r"
fields:
  - {path: identity.legal_name, group: identity, store: personnel, purpose: names the party}
  - {path: education.currently_enrolled, group: education, store: personnel, purpose: unpaid basis}
  - {path: statutory.pan, group: statutory, store: statutory, purpose: TDS}
",
        )
        .expect("fields parse");
        Catalogue {
            classes: classes.classes,
            fields: fields.fields,
        }
    }

    fn person(yaml: &str) -> Person {
        Person::new("t".to_owned(), serde_yaml::from_str(yaml).expect("parses"))
    }

    #[test]
    fn a_complete_intern_record_is_valid() {
        let c = catalogue();
        let class = c.class("intern").expect("declared");
        let p = person("identity: {legal_name: Asha}\neducation: {currently_enrolled: true}\n");
        assert!(validate(&c, class, &p).expect("validates").is_empty());
    }

    #[test]
    fn collecting_a_pan_for_an_unpaid_intern_is_a_violation() {
        // The finding this whole mechanism exists for: an unpaid student
        // intern is not an employee, no statute requires their PAN, and
        // collecting it anyway is processing without a purpose.
        let c = catalogue();
        let class = c.class("intern").expect("declared");
        let p = person(
            "identity: {legal_name: Asha}\n\
             education: {currently_enrolled: true}\n\
             statutory: {pan: ABCDE1234F}\n",
        );
        let v = validate(&c, class, &p).expect("validates");
        assert!(
            v.iter()
                .any(|x| matches!(x, Violation::Forbidden { path, .. } if path == "statutory.pan")),
            "expected statutory.pan to be forbidden, got {v:?}"
        );
    }

    #[test]
    fn an_employee_may_hold_a_pan_but_not_in_the_person_record() {
        // The class permits it -- unlike an intern's -- and it still belongs in
        // the `statutory` store rather than in people/<slug>.yaml. Two separate
        // rules, and this is the case that distinguishes them.
        let c = catalogue();
        let class = c.class("employee").expect("declared");
        let p = person("identity: {legal_name: A}\nstatutory: {pan: ABCDE1234F}\n");
        let v = validate(&c, class, &p).expect("validates");
        assert!(
            v.iter().all(|x| !matches!(x, Violation::Forbidden { .. })),
            "the class permits it: {v:?}"
        );
        assert!(
            v.iter()
                .any(|x| matches!(x, Violation::WrongStore { declared, .. }
                if declared == "statutory")),
            "but not in this file: {v:?}"
        );
    }

    #[test]
    fn a_missing_required_field_is_reported() {
        let c = catalogue();
        let class = c.class("intern").expect("declared");
        let p = person("identity: {legal_name: Asha}\n");
        let v = validate(&c, class, &p).expect("validates");
        assert!(v.iter().any(|x| matches!(x, Violation::Missing { path }
            if path == "education.currently_enrolled")));
    }

    #[test]
    fn an_undeclared_field_is_reported_rather_than_silently_stored() {
        // Otherwise a field creeps in with no stated purpose, which is how
        // a record quietly grows past what anyone agreed to.
        let c = catalogue();
        let class = c.class("intern").expect("declared");
        let p = person(
            "identity: {legal_name: A}\n\
             education: {currently_enrolled: true}\n\
             identity_extra: {shoe_size: 9}\n",
        );
        let v = validate(&c, class, &p).expect("validates");
        assert!(
            v.iter().any(|x| matches!(x, Violation::Undeclared { .. })),
            "{v:?}"
        );
    }

    #[test]
    fn prompt_help_is_the_first_sentence_only() {
        // Several purposes run to a paragraph because they carry the reasoning
        // for the field. That belongs in the record, not in a prompt.
        assert_eq!(
            super::first_sentence("Names the party. And a long second sentence."),
            "Names the party."
        );
        assert_eq!(
            super::first_sentence("No full stop here"),
            "No full stop here"
        );
        assert_eq!(
            super::first_sentence("Folded\n  over lines."),
            "Folded over lines."
        );
    }

    #[test]
    fn questions_are_derived_from_the_catalogue_not_listed() {
        let classes: super::ClassFile = serde_yaml::from_str(
            "classes:\n  - {class: c, prefix: CCC, description: d,\n     permits: [identity, emergency_contact], requires: [identity.legal_name],\n     forbids: []}\n",
        )
        .expect("classes parse");
        let fields: super::FieldFile = serde_yaml::from_str(
            "fields:\n\
             \x20 - {path: identity.legal_name, group: identity, store: personnel, purpose: names the party}\n\
             \x20 - {path: identity.nickname, group: identity, store: personnel, purpose: how they are addressed}\n\
             \x20 - {path: emergency_contact.name, group: emergency_contact, store: emergency, purpose: who to call}\n",
        )
        .expect("fields parse");
        let c = Catalogue {
            classes: classes.classes,
            fields: fields.fields,
        };
        let class = c.class("c").expect("declared");
        let qs = questions(&c, class);

        let paths: Vec<&str> = qs.iter().map(|q| q.path.as_str()).collect();
        assert_eq!(
            paths,
            ["identity.legal_name", "identity.nickname"],
            "required first, and nothing from another store"
        );
        assert!(qs.first().is_some_and(|q| q.required));
        assert!(
            qs.first()
                .is_some_and(|q| q.purpose.contains("names the party"))
        );
    }

    #[test]
    fn a_field_belonging_to_another_store_is_reported() {
        let classes: super::ClassFile = serde_yaml::from_str(
            "classes:\n  - {class: c, prefix: CCC, description: d,\n     permits: [contact], requires: [], forbids: []}\n",
        )
        .expect("classes parse");
        let fields: super::FieldFile = serde_yaml::from_str(
            "fields:\n  - {path: contact.kin, group: contact, store: emergency,\n     purpose: third-party contact details}\n",
        )
        .expect("fields parse");
        let c = Catalogue {
            classes: classes.classes,
            fields: fields.fields,
        };
        let class = c.class("c").expect("declared");
        let p = person("contact: {kin: Someone}\n");
        let v = validate(&c, class, &p).expect("validates");
        assert!(
            v.iter()
                .any(|x| matches!(x, Violation::WrongStore { declared, .. }
                if declared == "emergency")),
            "{v:?}"
        );
    }

    #[test]
    fn a_null_value_counts_as_absent() {
        let p = person("statutory: {pan: null}\n");
        let paths = present_paths(&p);
        assert!(!paths.contains(&"statutory.pan".to_owned()), "{paths:?}");
    }

    #[test]
    fn a_sub_key_of_a_declared_field_is_covered_by_it() {
        // `identity.pronouns` is one declared field with three sub-keys; it
        // must not be reported as three undeclared ones.
        let classes: super::ClassFile = serde_yaml::from_str(
            "classes:\n  - {class: c, prefix: CCC, description: d, permits: [identity],\n     requires: [], forbids: []}\n",
        ).expect("classes parse");
        let fields: super::FieldFile = serde_yaml::from_str(
            "fields:\n  - {path: identity.pronouns, group: identity, store: personnel, purpose: correct reference}\n",
        ).expect("fields parse");
        let c = Catalogue {
            classes: classes.classes,
            fields: fields.fields,
        };
        let class = c.class("c").expect("declared");
        let p = person("identity: {pronouns: {subject: they, object: them}}\n");
        assert!(validate(&c, class, &p).expect("validates").is_empty());
    }

    #[test]
    fn nested_paths_are_flattened_with_dots() {
        let p = person("identity: {legal_name: A}\n");
        assert!(present_paths(&p).contains(&"identity.legal_name".to_owned()));
    }
}
