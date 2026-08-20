//! What a template needs from a record before it can render.

use std::collections::BTreeSet;

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

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::required_commands;

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
