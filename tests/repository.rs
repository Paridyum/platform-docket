//! Facts about the repository itself that a fresh clone depends on.
//!
//! These do not test behaviour. They test that the files behaviour needs are actually going to
//! arrive on somebody else's machine — a class of defect that every other suite passes straight
//! through, because a test running here has the working tree in front of it.

#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Whether git can answer questions about this checkout.
///
/// Skips rather than fails where it cannot — a source tarball or a vendored copy is a
/// legitimate place to run the suite, and neither has an index to consult.
fn git_available() -> bool {
    Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(root())
        .output()
        .is_ok_and(|o| o.status.success())
}

/// Whether git tracks this path, or would once committed.
///
/// `ls-files --cached --others --exclude-standard` is deliberate: `--cached` catches a file
/// already committed, `--others --exclude-standard` catches one that is new and NOT ignored.
/// Together they answer the question that matters — "will a fresh clone have this?" — for a
/// file that has only just been written, which is exactly when the mistake gets made.
fn will_reach_a_clone(relative: &Path) -> bool {
    Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
        ])
        .arg(relative)
        .current_dir(root())
        .output()
        .is_ok_and(|o| !o.stdout.is_empty())
}

#[test]
fn every_class_template_reaches_a_fresh_clone() {
    // D-49. `people/*` is gitignored so personal data never enters history, with one negation
    // per template. `_TEMPLATE-officer.yaml` was written and silently ignored: `docket new
    // <slug> officer` then falls back to the intern template on a fresh checkout, producing a
    // record carrying `education` — which the officer class FORBIDS — so `docket check` rejects
    // it the instant it is written. That reads as a product defect rather than a missing file.
    //
    // Checked per class rather than per file, so adding a class with a template and forgetting
    // the negation fails here instead of on somebody else's machine.
    if !git_available() {
        eprintln!("SKIP every_class_template_reaches_a_fresh_clone: git cannot read this tree");
        return;
    }

    let classes: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(root().join("registry/person-classes.yaml"))
            .expect("person-classes.yaml is readable"),
    )
    .expect("person-classes.yaml parses");
    let declared = classes
        .get("classes")
        .and_then(|c| c.as_sequence())
        .expect("classes is a sequence");
    assert!(!declared.is_empty(), "at least one class is declared");

    for class in declared {
        let name = class
            .get("class")
            .and_then(serde_yaml::Value::as_str)
            .expect("every class has a name");
        let relative = PathBuf::from(format!("people/_TEMPLATE-{name}.yaml"));
        if !root().join(&relative).exists() {
            // No class-specific template is legitimate: `docket new` falls back to
            // `_TEMPLATE.yaml`, which is right for a class whose record shape matches it.
            continue;
        }
        assert!(
            will_reach_a_clone(&relative),
            "{} exists but is gitignored, so a fresh clone will not have it and `docket new \
             <slug> {name}` will silently fall back to the intern template. Add a negation to \
             .gitignore beside the existing ones — one line per template, never a glob, because \
             this directory is the one place where a false negative is personal data in git \
             history.",
            relative.display()
        );
    }
}

#[test]
fn the_base_template_reaches_a_fresh_clone() {
    // The one every class falls back to. Its negation has been there since the directory was
    // first ignored; this asserts nobody removes it while tidying.
    if !git_available() {
        eprintln!("SKIP the_base_template_reaches_a_fresh_clone: git cannot read this tree");
        return;
    }
    let relative = PathBuf::from("people/_TEMPLATE.yaml");
    assert!(
        will_reach_a_clone(&relative),
        "people/_TEMPLATE.yaml is gitignored; a checkout could not open a record at all"
    );
}

#[test]
fn no_real_person_record_is_tracked() {
    // The other direction, and the more expensive one to get wrong. The negations above are
    // narrow on purpose: a broad `!people/_TEMPLATE*.yaml` would also admit a real record
    // somebody named that way, and git history cannot be erased — which is precisely what
    // would make the DPDP erasure right unhonourable.
    if !git_available() {
        eprintln!("SKIP no_real_person_record_is_tracked: git cannot read this tree");
        return;
    }
    let out = Command::new("git")
        .args([
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            "people/",
        ])
        .current_dir(root())
        .output()
        .expect("git ls-files runs");
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let name = Path::new(line)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(line);
        assert!(
            name.starts_with('_'),
            "people/{name} would enter git history. Only templates belong there — a person \
             record in git cannot be erased, and an unerasable record makes the erasure right \
             unhonourable."
        );
    }
}

#[test]
fn a_template_hash_covers_the_clauses_it_includes() {
    // The extraction of `clauses/governing-law.tex` is only safe because of this. Before the
    // closure, `template_sha256` was a hash of the top-level file — so amending a shared clause
    // would have left every document recording the hash it had before, and the verification
    // code printed on them would have been a claim that the text was unchanged.
    use tool_docket::domain::template::{closure, hashable};

    let nda = root().join("templates/agreement/nda.tex");
    let files = closure(&root(), &nda).expect("closure resolves");

    let names: Vec<String> = files
        .iter()
        .map(|(p, _)| p.to_string_lossy().into())
        .collect();
    assert!(
        names.iter().any(|n| n.contains("clauses/governing-law")),
        "the shared clause is part of the NDA's closure: {names:?}"
    );

    // And the bytes actually reach the hash, rather than the path merely being listed.
    let hashed = String::from_utf8_lossy(&hashable(&files)).into_owned();
    assert!(
        hashed.contains("dkOrgJurisdictionForum"),
        "the clause body is inside the hashed bytes"
    );
}

#[test]
fn every_declared_template_resolves_its_inputs() {
    // A missing `\input` target renders as a LaTeX error at issuance — the worst moment to find
    // out, because an operator is mid-onboarding. Checked across every template the registry
    // declares rather than the five that happen to have one today.
    use tool_docket::domain::template::closure;

    let lifecycle: serde_yaml::Value = serde_yaml::from_str(
        &std::fs::read_to_string(root().join("registry/lifecycle.yaml")).expect("readable"),
    )
    .expect("lifecycle parses");
    let docs = lifecycle
        .get("documents")
        .and_then(|d| d.as_sequence())
        .expect("documents is a sequence");

    for doc in docs {
        let template = doc
            .get("template")
            .and_then(serde_yaml::Value::as_str)
            .expect("every document declares a template");
        let path = root().join(template);
        if !path.exists() {
            continue; // `check` reports a missing template; not this test's job to duplicate it.
        }
        let resolved = closure(&root(), &path);
        assert!(
            resolved.is_ok(),
            "{template}: an included file cannot be resolved: {:?}",
            resolved.err()
        );
    }
}

#[test]
fn an_adapted_template_is_reviewed_when_its_source_changes() {
    // D-46. Five officer documents were adapted from five intern ones. More than half their
    // sections differ in substance and two are inverted, so there is no shared clause to
    // extract — the measurement is in the logbook. What remains is drift: the intern document
    // is amended, the officer one is not, and nothing notices until somebody signs the stale
    // half.
    //
    // This does NOT compare content. The two are supposed to differ. It makes it impossible to
    // change one without being told to review the other.
    use std::collections::BTreeMap;

    let mut pinned = BTreeMap::new();
    for entry in walk_templates() {
        let body = std::fs::read_to_string(&entry).expect("template is readable");
        let Some(line) = body.lines().find(|l| l.starts_with("% adapted-from:")) else {
            continue;
        };
        let spec = line.trim_start_matches("% adapted-from:").trim();
        let parsed = spec.rsplit_once('@');
        assert!(
            parsed.is_some(),
            "{}: malformed pin {spec:?}",
            entry.display()
        );
        let (source, expected) = parsed.expect("checked on the line above");
        pinned.insert(entry, (source.to_owned(), expected.to_owned()));
    }

    assert!(!pinned.is_empty(), "the officer templates carry pins");

    for (template, (source, expected)) in pinned {
        let source_path = root().join(&source);
        let read = std::fs::read(&source_path);
        assert!(
            read.is_ok(),
            "{}: pinned source {source} is unreadable: {:?}",
            template.display(),
            read.as_ref().err()
        );
        let bytes = read.expect("checked on the line above");
        let actual = tool_docket::domain::chain::sha256(&bytes);
        assert_eq!(
            actual,
            expected,
            "\n{} was adapted from {source}, and {source} has changed since.\n\
             Review the adapted copy, make the matching change or decide it does not apply, \
             then update its `% adapted-from:` digest to {actual} in the same commit.\n",
            template.display()
        );
    }
}

/// Every `.tex` under `templates/`.
fn walk_templates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root().join("templates")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "tex") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}
