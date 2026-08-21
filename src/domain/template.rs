//! What a template needs from a record before it can render.

use crate::domain::escape;
use crate::domain::personnel::Field;
use anyhow::{Result, bail};
use std::collections::{BTreeMap, BTreeSet};

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
