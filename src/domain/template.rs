//! What a template needs from a record before it can render.

use crate::domain::escape;
use crate::domain::personnel::Field;
use anyhow::{Context, Result, bail};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// The prefix a person record is flattened under by `Definitions::absorb`.
///
/// Named rather than written out at each site because `paths_by_command`
/// inverts a mapping that is only correct for the prefix the record was
/// actually absorbed with. If the two were separate literals they could drift,
/// and the symptom would be every field reported as unmapped.
pub const PERSON_PREFIX: &str = "dkP";

/// Commands a template dereferences directly, with no fallback.
///
/// `\dkfill{dkPFoo}` degrades to a ruled line when the value is absent, which
/// is deliberate for a form. A bare `\dkPFoo` does not: it fails the build with
/// "Undefined control sequence", which is the right behaviour for an agreement
/// and the wrong thing to discover at issuance.
///
/// Scanning finds the second kind so the tool can say what is missing before
/// the render is attempted.
#[must_use]
pub fn required_commands(body: &str) -> BTreeSet<String> {
    let mut direct = BTreeSet::new();
    let mut cursor = body;
    while let Some(at) = cursor.find("\\dkP") {
        let rest = cursor.get(at + 1..).unwrap_or_default();
        let name: String = rest
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect();
        if !name.is_empty() {
            direct.insert(name);
        }
        cursor = cursor.get(at + 4..).unwrap_or_default();
    }

    // Remove the ones reached through \dkfill, which have a fallback.
    let mut filled = BTreeSet::new();
    let mut cursor = body;
    while let Some(at) = cursor.find("\\dkfill") {
        let rest = cursor.get(at..).unwrap_or_default();
        if let Some(open) = rest.find('{')
            && let Some(close) = rest.find('}')
            && open < close
            && let Some(inner) = rest.get(open + 1..close)
        {
            filled.insert(inner.trim().to_owned());
        }
        cursor = cursor.get(at + 7..).unwrap_or_default();
    }

    // `...Items` and `...Count` come from list fields and are always defined
    // when the list exists; an absent list still yields a valid body.
    direct
        .into_iter()
        .filter(|c| !filled.contains(c))
        .filter(|c| !c.ends_with("Items") && !c.ends_with("Count"))
        .collect()
}

/// The field path behind each command a person record can define.
///
/// Built FORWARDS from the registry and then read backwards. Taking the macro
/// apart instead cannot work, and this was tried first: `escape::command_name`
/// drops every separator, so `dkPIdentityLegalName` is an equally faithful
/// rendering of `identity.legal_name`, of `identity_legal.name` and of
/// `identity.legal.name`, and nothing left in the name says which was meant.
/// The declared paths are the only record of where the dots were.
///
/// # Errors
/// Fails if two declared paths derive the same command. That is a registry
/// defect rather than a reporting problem -- `absorb` would define the command
/// twice and the later value would silently win on a signed document -- so it
/// is refused here rather than settled by picking one.
pub fn paths_by_command(fields: &[Field]) -> Result<BTreeMap<String, &str>> {
    let mut out = BTreeMap::new();
    for f in fields {
        let command = format!("{PERSON_PREFIX}{}", escape::command_name(&f.path));
        if let Some(first) = out.insert(command.clone(), f.path.as_str()) {
            bail!(
                "fields {first:?} and {:?} both derive \\{command}; \
                 one of them would silently overwrite the other",
                f.path
            );
        }
    }
    Ok(out)
}

/// Name one required command the way an operator can act on it.
///
/// A command with no declared field behind it is reported AS ITSELF and said to
/// be unmapped. The two alternatives are both worse: dropping it hides a real
/// blocker and leaves the build to fail at issuance, and inventing a path from
/// the name sends someone to edit a field that does not exist.
///
/// The known unmapped case is a sub-key of a field declared at a higher level.
/// `identity.pronouns` is one field, so `dkPIdentityPronounsSubject` has no
/// declaration of its own; the templates reach pronouns through the aliased
/// `dkPPronouns*`, which always carry a default, so it does not arise today.
#[must_use]
pub fn field_of(command: &str, paths: &BTreeMap<String, &str>) -> String {
    paths.get(command).map_or_else(
        || format!("\\{command} (unmapped: not declared in fields.yaml)"),
        |path| (*path).to_owned(),
    )
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod closure_tests {
    use super::{closure, hashable, inputs};
    use std::fs;

    fn tree(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("temp dir");
        for (name, body) in files {
            let path = dir.path().join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("parent");
            }
            fs::write(path, body).expect("write");
        }
        dir
    }

    #[test]
    fn a_template_with_no_inputs_is_just_itself() {
        let d = tree(&[("a.tex", "hello")]);
        let files = closure(d.path(), &d.path().join("a.tex")).expect("closure");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0.to_string_lossy(), "a.tex");
    }

    #[test]
    fn inputs_are_followed_depth_first_in_document_order() {
        // The order LaTeX reads them. Sorting would also be stable, but it would stop matching
        // the document, and the point of the hash is to be auditable against the .tex.
        let d = tree(&[
            ("a.tex", "\\input{c/one}\n\\input{c/two}\n"),
            ("c/one.tex", "\\input{c/deep}\n"),
            ("c/deep.tex", "deep"),
            ("c/two.tex", "two"),
        ]);
        let files = closure(d.path(), &d.path().join("a.tex")).expect("closure");
        let names: Vec<String> = files
            .iter()
            .map(|(p, _)| p.to_string_lossy().into())
            .collect();
        assert_eq!(names, ["a.tex", "c/one.tex", "c/deep.tex", "c/two.tex"]);
    }

    #[test]
    fn changing_a_shared_clause_changes_the_hash() {
        // THE WHOLE REASON THIS EXISTS. Before the closure, the hash was of the top-level file,
        // so a document could have its terms rewritten underneath it and record the same hash
        // -- and the verification code printed on it would say the text was unchanged.
        let before = tree(&[("a.tex", "\\input{c}\n"), ("c.tex", "original terms")]);
        let after = tree(&[("a.tex", "\\input{c}\n"), ("c.tex", "amended terms")]);
        let h = |d: &tempfile::TempDir| {
            hashable(&closure(d.path(), &d.path().join("a.tex")).expect("closure"))
        };
        assert_ne!(h(&before), h(&after));
    }

    #[test]
    fn moving_a_clause_changes_the_hash_even_when_the_text_does_not() {
        // A rename changes nothing about the rendered document and everything about which file
        // a reader opens to audit it. A hash that could not see it would let the register lose
        // track of where its own text lives.
        let here = tree(&[("a.tex", "\\input{c}\n"), ("c.tex", "terms")]);
        let there = tree(&[
            ("a.tex", "\\input{clauses/c}\n"),
            ("clauses/c.tex", "terms"),
        ]);
        let h = |d: &tempfile::TempDir, _n: &str| {
            hashable(&closure(d.path(), &d.path().join("a.tex")).expect("closure"))
        };
        assert_ne!(h(&here, "c"), h(&there, "clauses/c"));
    }

    #[test]
    fn a_cycle_is_refused_rather_than_followed() {
        // LaTeX would spin. So would this, without the guard.
        let d = tree(&[("a.tex", "\\input{b}"), ("b.tex", "\\input{a}")]);
        let err = closure(d.path(), &d.path().join("a.tex")).expect_err("cycle refused");
        assert!(err.to_string().contains("more than once"), "{err}");
    }

    #[test]
    fn a_commented_out_input_is_not_followed() {
        // It is not read by LaTeX, so hashing it would make the register sensitive to a change
        // that cannot reach a document.
        let d = tree(&[("a.tex", "% \\input{ghost}\ntext\n")]);
        let files = closure(d.path(), &d.path().join("a.tex")).expect("closure");
        assert_eq!(files.len(), 1, "only the top file");
    }

    #[test]
    fn an_input_after_real_content_on_the_same_line_is_still_found() {
        assert_eq!(
            inputs("text \\input{one} more \\input{two}"),
            ["one.tex", "two.tex"]
        );
    }

    #[test]
    fn an_explicit_tex_suffix_is_not_doubled() {
        assert_eq!(inputs("\\input{one.tex}"), ["one.tex"]);
    }

    #[test]
    fn a_path_cannot_be_confused_with_content() {
        // Separated by NUL, which cannot occur in a path, so no filename-plus-content can be
        // arranged to collide with a different one.
        let a = vec![("x/y".into(), b"z".to_vec())];
        let b = vec![("x".into(), b"/yz".to_vec())];
        assert_ne!(hashable(&a), hashable(&b));
    }
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{field_of, paths_by_command, required_commands};
    use crate::domain::personnel::{Field, FieldFile};

    fn fields(src: &str) -> Vec<Field> {
        let parsed: FieldFile = serde_yaml::from_str(src).expect("fields parse");
        parsed.fields
    }

    const CATALOGUE: &str = "fields:\n\
        \x20 - {path: identity.legal_name, group: identity, store: personnel, purpose: names the party}\n\
        \x20 - {path: pre_existing_ip, group: ip, store: personnel, purpose: carve-out}\n";

    #[test]
    fn a_nested_path_is_reported_with_its_dots() {
        // The whole point: `command_name` flattened the dot away, and only the
        // declared path can put it back.
        let f = fields(CATALOGUE);
        let by = paths_by_command(&f).expect("no collisions");
        assert_eq!(field_of("dkPIdentityLegalName", &by), "identity.legal_name");
    }

    #[test]
    fn a_top_level_path_is_reported_unchanged() {
        // D-44 was raised on this exact one: `IPA needs: dkPPreExistingIp`.
        let f = fields(CATALOGUE);
        let by = paths_by_command(&f).expect("no collisions");
        assert_eq!(field_of("dkPPreExistingIp", &by), "pre_existing_ip");
    }

    #[test]
    fn an_unmappable_command_is_named_and_said_to_be_unmapped() {
        // Never dropped and never guessed: the template really does need it,
        // and no field in the registry supplies it.
        let f = fields(CATALOGUE);
        let by = paths_by_command(&f).expect("no collisions");
        let said = field_of("dkPSomethingUndeclared", &by);
        assert!(said.contains("dkPSomethingUndeclared"), "{said}");
        assert!(said.contains("unmapped"), "{said}");
    }

    #[test]
    fn two_paths_deriving_one_command_are_refused_rather_than_resolved() {
        // Both would define \dkPIdentityLegalName and the second would win on
        // the rendered document. Choosing one here would hide that.
        let f = fields(
            "fields:\n\
             \x20 - {path: identity.legal_name, group: identity, store: personnel, purpose: a}\n\
             \x20 - {path: identity_legal.name, group: identity, store: personnel, purpose: b}\n",
        );
        let err = paths_by_command(&f).expect_err("a collision is a registry defect");
        assert!(err.to_string().contains("dkPIdentityLegalName"), "{err}");
    }

    #[test]
    fn the_prefix_used_to_invert_is_the_one_records_are_absorbed_under() {
        // If these ever diverge every field reports as unmapped, which reads
        // like a broken registry rather than a broken constant.
        let f = fields(CATALOGUE);
        let by = paths_by_command(&f).expect("no collisions");
        assert!(
            by.keys().all(|c| c.starts_with(super::PERSON_PREFIX)),
            "{by:?}"
        );
    }

    #[test]
    fn a_bare_command_is_required() {
        let found = required_commands(r"Name: \dkPIdentityLegalName.");
        assert!(found.contains("dkPIdentityLegalName"), "{found:?}");
    }

    #[test]
    fn a_command_reached_through_dkfill_is_not() {
        // \dkfill degrades to a ruled line, which is what a form wants.
        let found = required_commands(r"Name: \dkfill{dkPContactPhone}");
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn dkfill_with_a_width_is_still_optional() {
        let found = required_commands(r"\dkfill[6cm]{dkPContactPhone}");
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn a_command_used_both_ways_is_optional() {
        // If any use has a fallback the field can be absent; the bare use will
        // then render the same rule.
        let found = required_commands(r"\dkfill{dkPContactPhone} and \dkPContactPhone");
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn list_bodies_are_not_required_fields() {
        let found = required_commands(
            r"\begin{itemize}\dkPCertificateAreasItems\end{itemize} \dkPCertificateAreasCount",
        );
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn several_commands_on_one_line_are_all_found() {
        let found = required_commands(
            r"\dkPIdentityLegalName, \dkPEngagementRole, \dkPEngagementStartDate",
        );
        assert_eq!(found.len(), 3, "{found:?}");
    }

    #[test]
    fn a_command_followed_by_punctuation_stops_at_the_punctuation() {
        let found =
            required_commands(r"\dkPEngagementRole. \dkPContactEmail{} \dkPEngagementEndDate,");
        assert!(found.contains("dkPEngagementRole"), "{found:?}");
        assert!(found.contains("dkPContactEmail"), "{found:?}");
        assert!(found.contains("dkPEngagementEndDate"), "{found:?}");
    }

    #[test]
    fn org_and_signatory_commands_are_not_person_fields() {
        let found = required_commands(r"\dkOrgName \dkSigCeoName \dkDocRef");
        assert!(
            found.is_empty(),
            "only person fields are scanned: {found:?}"
        );
    }
}

/// Every file a template is actually made of, in the order LaTeX would read them.
///
/// **THE HASH HAS TO COVER THIS, NOT JUST THE TOP FILE.** `template_sha256` is what
/// `verification_code` is derived from and what gets printed on the document, so it is the
/// estate's claim about which text was issued. The moment a template gains an `\input`, a
/// hash of the top-level file alone stops being that claim: two documents whose shared clauses
/// differ would record the same hash, and the code printed on them would say they were the
/// same text when they were not.
///
/// **The path is hashed alongside the bytes.** Moving a clause to a different file changes
/// nothing about the rendered document, but it does change which file a reader has to open to
/// audit it — and a rename that leaves the hash untouched is a rename the register cannot see.
///
/// Order is the order LaTeX reads them: depth-first, in the order the inputs appear. Sorting
/// would be stable too, but it would stop matching the document, and the point of this is to
/// be auditable by someone reading the `.tex`.
///
/// # Errors
/// Fails if a file cannot be read, or if the inputs form a cycle — which LaTeX would spin on
/// forever, so refusing is strictly better than reproducing that.
pub fn closure(root: &Path, from: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    walk(root, from, &mut out, &mut seen)?;
    Ok(out)
}

fn walk(
    root: &Path,
    file: &Path,
    out: &mut Vec<(PathBuf, Vec<u8>)>,
    seen: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    let relative = file.strip_prefix(root).unwrap_or(file).to_path_buf();
    if !seen.insert(relative.clone()) {
        bail!(
            "{} is included more than once, or includes itself; LaTeX would not terminate",
            relative.display()
        );
    }
    let body = fs::read(file).with_context(|| format!("reading {}", file.display()))?;
    let text = String::from_utf8_lossy(&body).into_owned();
    out.push((relative, body));

    for name in inputs(&text) {
        // Resolved from the repository root, matching how the templates are written and how
        // the typesetter is invoked. A path relative to the including file would be more
        // conventional in LaTeX and would make a clause's meaning depend on where it is
        // included from, which is the opposite of what a shared clause is for.
        walk(root, &root.join(&name), out, seen)?;
    }
    Ok(())
}

/// `\input{...}` and `\include{...}` targets, in the order they appear.
///
/// A commented-out include is skipped: it is not read by LaTeX, so hashing it would make the
/// register sensitive to a change that cannot reach a document.
fn inputs(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in body.lines() {
        let live = line.split('%').next().unwrap_or("");
        let mut rest = live;
        while let Some(at) = rest.find("\\input{").or_else(|| rest.find("\\include{")) {
            let after = &rest[at..];
            let Some(open) = after.find('{') else { break };
            let Some(close) = after[open..].find('}') else {
                break;
            };
            let name = &after[open + 1..open + close];
            if !name.is_empty() {
                // LaTeX takes `\input{a/b}` and `\input{a/b.tex}` as the same file, so both
                // resolve to the same path here. Matched case-insensitively because the
                // extension is ours to normalise, not the author's to get exactly right.
                let name = if Path::new(name)
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("tex"))
                {
                    name.to_owned()
                } else {
                    format!("{name}.tex")
                };
                out.push(name);
            }
            rest = &after[open + close..];
        }
    }
    out
}

/// The bytes a template hash is taken over: every file, each preceded by its path.
///
/// The separator is a NUL, which cannot occur in a path, so no combination of filename and
/// content can be made to collide with a different combination.
#[must_use]
pub fn hashable(files: &[(PathBuf, Vec<u8>)]) -> Vec<u8> {
    let mut buf = Vec::new();
    for (path, bytes) in files {
        buf.extend_from_slice(path.to_string_lossy().as_bytes());
        buf.push(0);
        buf.extend_from_slice(bytes);
        buf.push(0);
    }
    buf
}
