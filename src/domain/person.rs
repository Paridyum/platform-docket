//! The record of one person, and what must be true before documents issue.

use crate::domain::lifecycle::Condition;
use serde_yaml::Value;

/// A person record, held loosely because templates address fields the
/// domain has no opinion about.
///
/// Only the fields that carry a RULE are typed. Everything else flows
/// through to the templates unmodelled, so adding a field to a person record
/// does not require a code change.
#[derive(Debug, Clone)]
pub struct Person {
    /// Slug the record is filed under.
    pub slug: String,
    /// The parsed YAML, as given.
    pub value: Value,
}

impl Person {
    /// Wrap a parsed record.
    #[must_use]
    pub fn new(slug: String, value: Value) -> Self {
        Self { slug, value }
    }

    /// Follow a dotted path, e.g. `internship.end_date`.
    #[must_use]
    pub fn get(&self, path: &str) -> Option<&Value> {
        path.split('.')
            .try_fold(&self.value, |node, key| node.get(key))
    }

    /// A string field, absent if unset or not a string.
    #[must_use]
    pub fn string(&self, path: &str) -> Option<&str> {
        self.get(path).and_then(Value::as_str)
    }

    /// A boolean field, defaulting to false when unset.
    #[must_use]
    pub fn flag(&self, path: &str) -> bool {
        self.get(path).and_then(Value::as_bool).unwrap_or(false)
    }

    /// The stage this person is at. Defaults to the first stage.
    #[must_use]
    pub fn stage(&self) -> &str {
        self.string("engagement.stage").unwrap_or("offered")
    }

    /// Whether this person is a child under the DPDP Act on a given date.
    ///
    /// The Act defines a child as anyone who has not completed eighteen
    /// years (s2(f)), and s9 requires verifiable guardian consent before
    /// processing their data. Measured at the engagement start date rather
    /// than today, because that is when processing begins.
    ///
    /// `None` when no date of birth is recorded: not knowing is a distinct
    /// state from being an adult, and must not be silently treated as one.
    #[must_use]
    pub fn is_child_on(&self, on: time::Date) -> Option<bool> {
        let dob = crate::adapters::clock::parse_iso(self.string("identity.date_of_birth")?).ok()?;
        let eighteenth = dob.replace_year(dob.year().checked_add(18)?).ok()?;
        Some(on < eighteenth)
    }

    /// Whether a guardian block is recorded.
    #[must_use]
    pub fn has_guardian(&self) -> bool {
        self.string("guardian.name")
            .is_some_and(|n| !n.trim().is_empty())
    }

    /// Whether a conditional document applies to this person.
    #[must_use]
    pub fn satisfies(&self, condition: Condition) -> bool {
        match condition {
            Condition::EquipmentIssued => self.flag("equipment.issued"),
            Condition::ReferenceRequested => self.flag("reference.requested"),
            Condition::InstitutionRequiresNoc => self.flag("education.institution_requires_noc"),
            Condition::IsChild => self
                .string("engagement.start_date")
                .and_then(|d| crate::adapters::clock::parse_iso(d).ok())
                .and_then(|d| self.is_child_on(d))
                .unwrap_or(false),
        }
    }
}

/// A reason issuance must not proceed.
///
/// These are not warnings. Each one names a condition under which the
/// resulting document would be defective, so the default is to refuse.
#[derive(Debug, PartialEq, Eq)]
pub struct Blocker(pub String);

impl std::fmt::Display for Blocker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The value the template ships with, wherever a person's own details go.
///
/// It exists because a *plausible* placeholder cannot be told apart from real
/// data by anything except a person noticing, and that is exactly what failed:
/// `docket new` copied a template carrying a realistic name, two freshly enrolled
/// interns both displayed as the same person, and `docket check` reported nothing
/// wrong because every required field was present. The template satisfied the
/// schema.
///
/// Chosen to be impossible to mistake for a name, a date or an address, and to
/// survive a copy-paste into a form. Anything containing it is unfilled.
pub const UNFILLED: &str = "FILL-ME";

/// Paths still holding the template's placeholder, deepest first.
///
/// Scans values rather than a list of paths, so a field added to
/// `registry/fields.yaml` and the template is covered without editing this
/// function — a hardcoded list is a list that goes stale silently.
#[must_use]
pub fn unfilled(person: &Person) -> Vec<String> {
    let mut out = Vec::new();
    collect_unfilled(&person.value, String::new(), &mut out);
    out.sort();
    out
}

fn collect_unfilled(value: &Value, path: String, out: &mut Vec<String>) {
    match value {
        Value::String(text) if text.contains(UNFILLED) => out.push(path),
        Value::Mapping(map) => {
            for (key, child) in map {
                let Some(name) = key.as_str() else { continue };
                let next = if path.is_empty() {
                    name.to_owned()
                } else {
                    format!("{path}.{name}")
                };
                collect_unfilled(child, next, out);
            }
        }
        Value::Sequence(items) => {
            for (i, child) in items.iter().enumerate() {
                collect_unfilled(child, format!("{path}[{i}]"), out);
            }
        }
        _ => {}
    }
}

/// Everything that must hold before this person's documents may be issued.
///
/// `forum` is threaded in from `org.yaml` rather than read here, because the
/// domain does not do I/O.
#[must_use]
pub fn blockers(person: &Person, forum: Option<&str>) -> Vec<Blocker> {
    let mut out = Vec::new();

    // FIRST, and on its own, because everything below it is a question about a
    // record someone has actually filled in. An unedited template answers every
    // one of those questions plausibly and wrongly, and issuing from it produces
    // real documents, with real serials, naming somebody else -- which cannot be
    // corrected by editing, only by re-issuing under new serials.
    let unfilled = unfilled(person);
    if !unfilled.is_empty() {
        out.push(Blocker(format!(
            "{} field(s) still hold the template's {UNFILLED} placeholder and have not been \
             filled in: {}. Issuing now would produce documents that name nobody.",
            unfilled.len(),
            unfilled.join(", ")
        )));
        // Returning early is deliberate. The blockers below would each report a
        // missing or unparsable value that is really the same one fact, and a
        // list of eleven consequences buries the one instruction that matters.
        return out;
    }

    if forum.is_none_or(str::is_empty) {
        out.push(Blocker(
            "org.jurisdiction.forum is unset: the governing-law clause would name no court"
                .to_owned(),
        ));
    }
    if person
        .string("identity.legal_name")
        .is_none_or(str::is_empty)
    {
        out.push(Blocker("identity.legal_name is unset".to_owned()));
    }
    if !person.flag("education.currently_enrolled") {
        out.push(Blocker(
            "education.currently_enrolled is false: the unpaid basis rests on it".to_owned(),
        ));
    }
    // DPDP s2(f) defines a child as anyone under eighteen, and s9 requires
    // verifiable guardian consent before processing their data at all.
    // Measured at the start date, because that is when processing begins.
    match person
        .string("engagement.start_date")
        .and_then(|d| crate::adapters::clock::parse_iso(d).ok())
        .and_then(|d| person.is_child_on(d))
    {
        None => out.push(Blocker(
            "identity.date_of_birth is unset: without it Paridyum cannot tell whether \
             this person is a child under DPDP s2(f), and so cannot tell whether s9 \
             guardian consent is required"
                .to_owned(),
        )),
        Some(true) if !person.has_guardian() => out.push(Blocker(
            "this person is under eighteen on their start date: DPDP s9 requires \
             verifiable guardian consent, and no guardian is recorded"
                .to_owned(),
        )),
        Some(_) => {}
    }
    if person.string("engagement.end_date").is_none() {
        out.push(Blocker(
            "engagement.end_date is unset: an unpaid internship must be time-bound".to_owned(),
        ));
    }
    out
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{Person, blockers, unfilled};
    use crate::domain::lifecycle::Condition;

    fn person(yaml: &str) -> Person {
        Person::new("t".to_owned(), serde_yaml::from_str(yaml).expect("parses"))
    }

    fn complete() -> Person {
        person(
            "identity: {legal_name: Asha Verma, date_of_birth: 2005-04-12}\n\
             education: {currently_enrolled: true}\n\
             engagement: {stage: preboarding, start_date: 2026-09-01, end_date: 2026-11-30}\n",
        )
    }

    #[test]
    fn a_complete_record_has_no_blockers() {
        assert!(blockers(&complete(), Some("Dehradun")).is_empty());
    }

    #[test]
    fn an_unset_forum_blocks_issuance() {
        let b = blockers(&complete(), None);
        assert!(b.iter().any(|x| x.0.contains("forum")));
        assert!(
            blockers(&complete(), Some(""))
                .iter()
                .any(|x| x.0.contains("forum"))
        );
    }

    #[test]
    fn an_unenrolled_person_blocks_the_unpaid_basis() {
        let p = person(
            "identity: {legal_name: X}\n\
             education: {currently_enrolled: false}\n\
             engagement: {end_date: 2026-11-30}\n",
        );
        assert!(
            blockers(&p, Some("Dehradun"))
                .iter()
                .any(|x| x.0.contains("unpaid"))
        );
    }

    #[test]
    fn a_missing_end_date_blocks_because_unpaid_must_be_time_bound() {
        let p = person("name: {legal: X}\neducation: {currently_enrolled: true}\ninternship: {}\n");
        assert!(
            blockers(&p, Some("Dehradun"))
                .iter()
                .any(|x| x.0.contains("end_date"))
        );
    }

    #[test]
    fn a_seventeen_year_old_is_a_child_on_their_start_date() {
        // DPDP s2(f): a child is anyone who has not COMPLETED eighteen years.
        let p = person("identity: {date_of_birth: 2009-04-12}\n");
        let start = crate::adapters::clock::parse_iso("2026-09-01").expect("date");
        assert_eq!(p.is_child_on(start), Some(true));
    }

    #[test]
    fn the_boundary_is_the_eighteenth_birthday_itself() {
        let p = person("identity: {date_of_birth: 2008-09-01}\n");
        let day_before = crate::adapters::clock::parse_iso("2026-08-31").expect("date");
        let birthday = crate::adapters::clock::parse_iso("2026-09-01").expect("date");
        assert_eq!(p.is_child_on(day_before), Some(true));
        assert_eq!(
            p.is_child_on(birthday),
            Some(false),
            "eighteen on the day itself"
        );
    }

    #[test]
    fn an_unrecorded_date_of_birth_is_unknown_not_adult() {
        // Not knowing must never be silently treated as "no obligation".
        let p = person("identity: {legal_name: X}\n");
        let start = crate::adapters::clock::parse_iso("2026-09-01").expect("date");
        assert_eq!(p.is_child_on(start), None);
    }

    #[test]
    fn a_missing_date_of_birth_blocks_issuance() {
        let p = person(
            "identity: {legal_name: X}\n\
             education: {currently_enrolled: true}\n\
             engagement: {start_date: 2026-09-01, end_date: 2026-11-30}\n",
        );
        assert!(
            blockers(&p, Some("Dehradun"))
                .iter()
                .any(|b| b.0.contains("date_of_birth"))
        );
    }

    #[test]
    fn a_child_without_a_recorded_guardian_blocks_issuance() {
        let p = person(
            "identity: {legal_name: X, date_of_birth: 2009-04-12}\n\
             education: {currently_enrolled: true}\n\
             engagement: {start_date: 2026-09-01, end_date: 2026-11-30}\n",
        );
        assert!(
            blockers(&p, Some("Dehradun"))
                .iter()
                .any(|b| b.0.contains("s9"))
        );
    }

    #[test]
    fn a_child_with_a_guardian_does_not_block() {
        let p = person(
            "identity: {legal_name: X, date_of_birth: 2009-04-12}\n\
             guardian: {name: A Parent}\n\
             education: {currently_enrolled: true}\n\
             engagement: {start_date: 2026-09-01, end_date: 2026-11-30}\n",
        );
        assert!(blockers(&p, Some("Dehradun")).is_empty());
    }

    #[test]
    fn stage_defaults_to_the_first_one() {
        assert_eq!(person("identity: {legal_name: X}\n").stage(), "offered");
        assert_eq!(complete().stage(), "preboarding");
    }

    #[test]
    fn conditions_read_the_record() {
        let p = person("equipment: {issued: true}\n");
        assert!(p.satisfies(Condition::EquipmentIssued));
        assert!(!p.satisfies(Condition::ReferenceRequested));
    }

    #[test]
    fn a_missing_path_is_absent_rather_than_a_panic() {
        let p = person("identity: {legal_name: X}\n");
        assert!(p.get("does.not.exist").is_none());
        assert!(!p.flag("also.missing"));
    }

    #[test]
    fn an_unedited_template_value_is_found_at_any_depth() {
        let p = person(
            "identity: {legal_name: FILL-ME, preferred_name: Asha}\ncontact: {email: FILL-ME}\n",
        );
        assert_eq!(unfilled(&p), vec!["contact.email", "identity.legal_name"]);
    }

    #[test]
    fn a_placeholder_inside_a_list_is_found_with_its_index() {
        // certificate.areas and reference.contributions are sequences, and a
        // scan that only walked mappings would report a filled-in record.
        let p = person("certificate: {areas: [Rust, FILL-ME]}\n");
        assert_eq!(unfilled(&p), vec!["certificate.areas[1]"]);
    }

    #[test]
    fn a_filled_record_reports_nothing() {
        let p = person("identity: {legal_name: Vikram Rao}\ncontact: {email: v@example.com}\n");
        assert!(unfilled(&p).is_empty());
    }

    #[test]
    fn a_placeholder_embedded_in_a_longer_value_still_counts() {
        // Someone half-edits a line. It looks filled in, and is not.
        let p = person("engagement: {role: Intern, project_description: FILL-ME later}\n");
        assert_eq!(unfilled(&p), vec!["engagement.project_description"]);
    }

    #[test]
    fn a_non_string_value_is_never_a_placeholder() {
        let p = person("engagement: {hours_per_week: 20}\neducation: {currently_enrolled: true}\n");
        assert!(unfilled(&p).is_empty());
    }

    #[test]
    fn an_unfilled_record_blocks_issuance_before_anything_else_is_reported() {
        let p = person("identity: {legal_name: FILL-ME}\n");
        let blocking = blockers(&p, Some("Dehradun"));
        // Exactly one, and it is the placeholder. The record is also missing a
        // date of birth, an end date and enrolment, and reporting those too would
        // bury the single instruction that matters.
        assert_eq!(blocking.len(), 1, "{blocking:?}");
        assert!(blocking[0].0.contains("FILL-ME"), "{:?}", blocking[0]);
    }

    #[test]
    fn a_record_with_no_placeholders_reports_its_real_blockers() {
        // Proves the early return above is not swallowing everything.
        let p = person("identity: {legal_name: Vikram Rao}\n");
        let blocking = blockers(&p, Some("Dehradun"));
        assert!(blocking.len() > 1, "{blocking:?}");
        assert!(!blocking.iter().any(|b| b.0.contains("FILL-ME")));
    }
}
