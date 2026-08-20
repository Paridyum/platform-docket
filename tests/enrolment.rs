//! Opening a record by answering questions rather than editing YAML.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use tool_docket::adapters::registry::Registry;
use tool_docket::application::enrol;
use tool_docket::config::Paths;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn registry() -> Registry {
    Registry::load(&Paths::resolve(&root())).expect("registry loads")
}

/// Answers in the order the interview asks for them, then `n` to skip optional.
const REQUIRED_ANSWERS: &str = "Asha Verma\n2005-04-12\nasha@example.com\n\
     Software Engineering Intern\n2026-11-30\n2026-09-01\nGraphic Era University\n\nn\n";

fn interview(script: &str) -> (BTreeMap<String, String>, String) {
    let reg = registry();
    let mut out = Vec::new();
    let mut input = script.as_bytes();
    let answers = enrol::interview(&reg, "intern", &mut out, &mut input).expect("interviews");
    (answers, String::from_utf8_lossy(&out).into_owned())
}

#[test]
fn the_interview_asks_for_everything_the_class_requires() {
    let reg = registry();
    let class = reg.catalogue.class("intern").expect("declared");
    let (answers, _) = interview(REQUIRED_ANSWERS);
    for path in &class.requires {
        assert!(
            answers.contains_key(path),
            "{path} was never asked: {answers:?}"
        );
    }
}

#[test]
fn the_interview_never_asks_for_a_field_the_class_forbids() {
    // An unpaid intern's PAN. Asking for it at all is the failure — a refusal
    // afterwards is too late, because the operator has already typed it.
    let (answers, shown) = interview(REQUIRED_ANSWERS);
    assert!(!shown.to_lowercase().contains("pan"), "{shown}");
    assert!(
        answers.keys().all(|k| !k.starts_with("statutory.")),
        "{answers:?}"
    );
}

#[test]
fn the_interview_never_asks_for_a_field_from_another_store() {
    // Emergency contact details live in their own store. Asking here would put
    // third-party data in the personnel record.
    let (answers, shown) = interview(REQUIRED_ANSWERS);
    assert!(
        answers.keys().all(|k| !k.starts_with("emergency_contact.")),
        "{answers:?}"
    );
    assert!(!shown.contains("emergency contact"), "{shown}");
}

#[test]
fn the_prompt_shows_the_fields_stated_purpose() {
    let (_, shown) = interview(REQUIRED_ANSWERS);
    assert!(
        shown.contains("Names the party on every agreement"),
        "the declared purpose is the help text: {shown}"
    );
}

#[test]
fn a_required_answer_cannot_be_skipped() {
    // Blank, blank, then the real answer.
    let (answers, shown) = interview(&format!("\n\n{REQUIRED_ANSWERS}"));
    assert!(shown.contains("this one is required"), "{shown}");
    assert_eq!(
        answers.get("identity.legal_name").map(String::as_str),
        Some("Asha Verma")
    );
}

#[test]
fn the_operator_can_stop_after_the_required_questions() {
    // Fifty prompts is an interrogation, not a wizard.
    let reg = registry();
    let class = reg.catalogue.class("intern").expect("declared");
    let optional = tool_docket::domain::personnel::questions(&reg.catalogue, class)
        .into_iter()
        .filter(|q| !q.required)
        .count();
    assert!(
        optional > 20,
        "this test is about there being many optional fields"
    );

    let (answers, shown) = interview(REQUIRED_ANSWERS);
    assert!(shown.contains("Everything required is answered"), "{shown}");
    assert_eq!(
        answers.len(),
        class.requires.len(),
        "only the required ones: {answers:?}"
    );
}

#[test]
fn the_generated_record_is_valid_yaml_and_passes_validation() {
    let reg = registry();
    let (answers, _) = interview(REQUIRED_ANSWERS);
    let body = enrol::to_yaml(&reg, "intern", &answers);

    let parsed: serde_yaml::Value =
        serde_yaml::from_str(&body).expect("the generated record must parse");
    let person = tool_docket::domain::person::Person::new("t".to_owned(), parsed);
    let class = reg.catalogue.class("intern").expect("declared");
    let violations = tool_docket::domain::personnel::validate(&reg.catalogue, class, &person)
        .expect("validates");
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn the_generated_record_carries_each_fields_purpose_as_a_comment() {
    // The file is the thing someone edits later, so the reasoning belongs in it
    // even though the prompt only showed the first sentence.
    let reg = registry();
    let (answers, _) = interview(REQUIRED_ANSWERS);
    let body = enrol::to_yaml(&reg, "intern", &answers);
    assert!(
        body.contains("# Names the party on every agreement"),
        "{body}"
    );
    assert!(body.contains("legal_name: \"Asha Verma\""), "{body}");
}

#[test]
fn booleans_and_pronouns_are_written_as_the_templates_expect() {
    let reg = registry();
    let mut answers = BTreeMap::new();
    answers.insert("identity.legal_name".to_owned(), "A".to_owned());
    answers.insert("education.currently_enrolled".to_owned(), "yes".to_owned());
    answers.insert("identity.pronouns".to_owned(), "she/her/her".to_owned());
    let body = enrol::to_yaml(&reg, "intern", &answers);

    assert!(
        body.contains("currently_enrolled: true"),
        "a flag, not a string: {body}"
    );
    let parsed: serde_yaml::Value = serde_yaml::from_str(&body).expect("parses");
    let person = tool_docket::domain::person::Person::new("t".to_owned(), parsed);
    assert_eq!(person.string("identity.pronouns.subject"), Some("she"));
    assert_eq!(person.string("identity.pronouns.object"), Some("her"));
    assert_eq!(person.string("identity.pronouns.possessive"), Some("her"));
}

#[test]
fn pronouns_default_to_they_them_when_only_partly_given() {
    let reg = registry();
    let mut answers = BTreeMap::new();
    answers.insert("identity.pronouns".to_owned(), "she".to_owned());
    let body = enrol::to_yaml(&reg, "intern", &answers);
    let parsed: serde_yaml::Value = serde_yaml::from_str(&body).expect("parses");
    let person = tool_docket::domain::person::Person::new("t".to_owned(), parsed);
    assert_eq!(person.string("identity.pronouns.subject"), Some("she"));
    assert_eq!(
        person.string("identity.pronouns.object"),
        Some("them"),
        "never guessed from the subject"
    );
}

#[test]
fn an_unknown_class_is_refused_before_any_question_is_asked() {
    let reg = registry();
    let mut out = Vec::new();
    let mut input = "".as_bytes();
    let err = enrol::interview(&reg, "freelancer", &mut out, &mut input)
        .err()
        .map(|e| e.to_string())
        .unwrap_or_default();
    assert!(
        err.contains("intern"),
        "it names the declared classes: {err}"
    );
    assert!(
        out.is_empty(),
        "nothing should be printed before the class is known"
    );
}

#[test]
fn writing_a_record_refuses_to_overwrite_one() {
    let dir = tempfile::tempdir().expect("temp dir");
    let paths = Paths {
        root: root(),
        people: dir.path().join("people"),
        store: dir.path().join("store"),
    };
    enrol::write_record(&paths, "t", "version: 1\n").expect("writes");
    assert!(enrol::write_record(&paths, "t", "version: 1\n").is_err());
}

#[test]
fn a_minimal_record_is_told_which_fields_a_document_still_needs() {
    // The class's `requires` says what it needs to be LAWFUL, not what every
    // template happens to print. Answering only the required questions leaves
    // documents that cannot render, and the operator should hear that as field
    // paths rather than as `Undefined control sequence` at TeX time.
    let dir = tempfile::tempdir().expect("temp dir");
    let people = dir.path().join("people");
    std::fs::create_dir_all(&people).expect("people dir");
    let paths = Paths {
        root: root(),
        people,
        store: dir.path().join("store"),
    };
    let reg = registry();

    let (answers, _) = interview(REQUIRED_ANSWERS);
    enrol::write_record(&paths, "t", &enrol::to_yaml(&reg, "intern", &answers)).expect("writes");
    tool_docket::application::personnel::create(
        &paths,
        &reg,
        "t",
        "intern",
        tool_docket::adapters::clock::parse_iso("2026-08-20").expect("date"),
        "rohan",
    )
    .expect("creates");

    let person = tool_docket::adapters::store::person(&paths, "t").expect("record");
    let who = tool_docket::application::personnel::identify(&paths, "t").expect("identified");
    let blocked = tool_docket::application::issue::unrenderable(&paths, &reg, &person, &who, &[])
        .expect("checks");

    assert!(
        !blocked.is_empty(),
        "a minimal record cannot render everything"
    );
    let (code, missing) = blocked.first().expect("at least one");
    assert_eq!(code, "OFR", "the offer is owed first: {blocked:?}");
    assert!(
        missing.iter().any(|m| m.contains("Supervisor")),
        "it names what is missing: {missing:?}"
    );
}

#[test]
fn a_complete_record_has_nothing_unrenderable() {
    // The checked-in template is complete, so it must pass this cleanly —
    // otherwise the default path is broken and nobody would know.
    let dir = tempfile::tempdir().expect("temp dir");
    let people = dir.path().join("people");
    std::fs::create_dir_all(&people).expect("people dir");
    std::fs::copy(
        root().join("people").join("_TEMPLATE.yaml"),
        people.join("_TEMPLATE.yaml"),
    )
    .expect("template");
    let paths = Paths {
        root: root(),
        people,
        store: dir.path().join("store"),
    };
    let reg = registry();

    tool_docket::application::personnel::create(
        &paths,
        &reg,
        "t",
        "intern",
        tool_docket::adapters::clock::parse_iso("2026-08-20").expect("date"),
        "rohan",
    )
    .expect("creates");

    let person = tool_docket::adapters::store::person(&paths, "t").expect("record");
    let who = tool_docket::application::personnel::identify(&paths, "t").expect("identified");
    let blocked = tool_docket::application::issue::unrenderable(&paths, &reg, &person, &who, &[])
        .expect("checks");
    assert!(
        blocked.is_empty(),
        "the shipped template must render everything: {blocked:?}"
    );
}
