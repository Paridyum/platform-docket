//! Helpers shared by the integration suites.

// `unreachable_pub` fires because each integration binary includes this module
// privately, so its `pub` items are pub-in-private there. They are genuinely
// public to the suites that use them, and there is no narrower visibility that
// works across six separate test binaries.
#![allow(dead_code, unreachable_pub, clippy::expect_used)]

use std::path::Path;

/// Copy the real `_TEMPLATE.yaml` into a sandbox, with every `FILL-ME` replaced.
///
/// The production template ships placeholders that `check` reports and `issue`
/// refuses, because a realistic placeholder cannot be told apart from real data
/// (D-42). Tests that need an issuable record therefore have to fill it in, the
/// same as an operator does.
///
/// Doing it to the TEMPLATE rather than to each record means every `docket new`
/// in a sandbox produces a complete record, which is what the suites assumed
/// before the placeholders existed.
///
/// The values are deliberately obvious test data — `Test Person`, `example.com` —
/// so that a fixture leaking into a real store or a rendered PDF is recognisable
/// on sight rather than plausible.
pub fn filled_template(from_root: &Path, into: &Path) {
    let source = from_root.join("people").join("_TEMPLATE.yaml");
    let body = std::fs::read_to_string(&source).expect("template readable");

    let mut out = String::with_capacity(body.len());
    for line in body.lines() {
        out.push_str(&fill_line(line));
        out.push('\n');
    }

    assert!(
        !out.lines()
            .any(|l| !l.trim_start().starts_with('#') && l.contains("FILL-ME")),
        "the fixture still holds a placeholder; a new FILL-ME field needs a value in \
         tests/common/mod.rs. Leaving it would make every suite that issues fail with \
         a refusal that looks like a product bug."
    );
    std::fs::write(into, out).expect("fixture written");
}

/// One line, with its placeholder replaced by something valid for that field.
///
/// Keyed on the field name because the values are not interchangeable: a date
/// must parse, `graduation_year` must be a number, and `identity.date_of_birth`
/// decides whether DPDP s9 guardian consent applies — a fixture born in 2015
/// would make every issuance test fail for the right reason at the wrong time.
fn fill_line(line: &str) -> String {
    if !line.contains("FILL-ME") || line.trim_start().starts_with('#') {
        return line.to_owned();
    }
    let key = line.split(':').next().unwrap_or("").trim();
    let value = match key {
        "role" => "Software Engineering Intern",
        "start_date" => "2026-09-01",
        "end_date" => "2026-11-30",
        "project_description" => "the internal document generation system",
        "legal_name" => "Test Person",
        "preferred_name" => "Test",
        // An adult on the start date above, so guardian consent is not in play.
        // A test that wants the child path sets this itself.
        "date_of_birth" => "2005-04-12",
        "phone" => "\"+91 90000 00000\"",
        "address" => "12 Example Road, Dehradun, Uttarakhand 248001",
        "institution" => "Example University",
        "degree" => "B.Tech Computer Science",
        "specialization" => "Software Engineering",
        "graduation_year" => "2027",
        "linkedin" => "linkedin.com/in/example",
        "github" => "github.com/example",
        "skills" => "Rust, TypeScript, PostgreSQL",
        "qualities" => "technical ability and clear communication",
        "specific_strength" => "ship a maintainable system",
        // `email` appears under both contact and education. Either address is
        // valid for a fixture, so it does not need disambiguating by position.
        "email" => "test.person@example.com",
        // A sequence item: `    - FILL-ME`.
        "- FILL-ME" | "-" => return line.replace("FILL-ME", "Rust service development"),
        _ => return line.replace("FILL-ME", "example"),
    };
    line.replacen("FILL-ME", value, 1)
}
