//! The two addresses a person is issued, derived rather than typed.

use crate::domain::identity::PersonId;

/// The domain every published address sits on.
///
/// ROOT, NEVER A CLASS SUBDOMAIN (ADR-0067 decision 1). `intern.paridyum.com` was proposed
/// and rejected: class is mutable for a person, and ADR-0051 is locked against encoding a
/// mutable fact in an identifier meant to persist. The predictable case is the one that
/// breaks it — an intern is hired, and the address either lies or has to be renamed, which
/// decision 3 forbids.
pub const DOMAIN: &str = "paridyum.com";

/// Where the mailboxes actually live. ADR-0067 decision 5's accepted cost.
pub const MAILBOX_DOMAIN: &str = "gmail.com";

/// The two addresses a person holds.
///
/// SEPARATE BECAUSE THEY ARE DIFFERENT THINGS. `published` is the identity — what goes on a
/// document, on the officers page, and in a commit author line. `mailbox` is where mail
/// physically lands, and is not given out.
///
/// The split is what survives the consumer-account cost: Paridyum cannot suspend a Gmail
/// account it does not administer, but it can sever the alias alone, which ends the person's
/// ability to act as Paridyum whatever happens to the mailbox behind it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Addresses {
    /// `firstname.lastname@paridyum.com` — the published identity.
    pub published: String,
    /// `firstname.p000147c3.paridyum@gmail.com` — where mail lands.
    pub mailbox: String,
}

/// Derive both addresses from a legal name and a person identifier.
///
/// DERIVED RATHER THAN TYPED, so the two cannot disagree and neither is invented at the
/// keyboard. `suffix` disambiguates a collision — the SECOND Rohan Batra is `rohan.batra2`,
/// and the first is never touched (ADR-0067 decision 3): renaming an address already in use
/// strands every copy of it held by somebody else.
///
/// The mailbox carries the person identifier because it is already unique across Paridyum
/// forever and already refuses to be reissued (ADR-0051). Minting a second sequence beside it
/// would be a second thing that must never be reused, which is the failure that silently
/// reattaches one person's records to another.
#[must_use]
pub fn derive(legal_name: &str, person_id: &PersonId, suffix: Option<u32>) -> Addresses {
    let parts = name_parts(legal_name);
    let local = parts.join(".");
    let n = suffix.map_or_else(String::new, |n| n.to_string());
    let given = parts
        .first()
        .cloned()
        .unwrap_or_else(|| "person".to_owned());

    Addresses {
        published: format!("{local}{n}@{DOMAIN}"),
        mailbox: format!("{given}.{}.paridyum@{MAILBOX_DOMAIN}", squash(person_id)),
    }
}

/// A legal name reduced to address-safe segments.
///
/// Legal names in this estate are recorded as they are printed on a document, which means
/// upper case and, for many people, more than two words. Everything that is not an ASCII
/// letter or digit is dropped rather than transliterated: a guess at how a name should be
/// romanised is a guess embedded in a string that can never be corrected (decision 3).
///
/// A name that reduces to nothing yields no segments, and the caller gets `person` — which is
/// wrong, and visibly wrong, rather than an address that looks right and is not.
fn name_parts(legal_name: &str) -> Vec<String> {
    legal_name
        .split_whitespace()
        .map(|word| {
            word.chars()
                .filter(char::is_ascii_alphanumeric)
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|word| !word.is_empty())
        .collect()
}

/// `PAR-P-000147-3` as `p000147c3`, for a namespace that allows no hyphens.
///
/// The `PAR-` prefix is dropped because the account name already ends in `paridyum`, and the
/// check character is joined with `c` so the identifier can be read back out unambiguously —
/// `p0001473` would not say where the sequence stops and the check begins.
fn squash(person_id: &PersonId) -> String {
    let raw = person_id.as_str();
    let body = raw.strip_prefix("PAR-").unwrap_or(raw);
    match body.rsplit_once('-') {
        Some((head, check)) => format!("{}c{check}", head.replace('-', "")).to_lowercase(),
        None => body.replace('-', "").to_lowercase(),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::{Addresses, derive};
    use crate::domain::identity::PersonId;

    fn id(seq: u32) -> PersonId {
        PersonId::new(seq).expect("valid")
    }

    #[test]
    fn derives_both_addresses_from_a_two_part_name() {
        let a = derive("ROHAN BATRA", &id(147), None);
        assert_eq!(a.published, "rohan.batra@paridyum.com");
        assert!(
            a.mailbox.starts_with("rohan.p000147c") && a.mailbox.ends_with(".paridyum@gmail.com"),
            "{}",
            a.mailbox
        );
    }

    #[test]
    fn a_collision_suffixes_the_new_address_and_not_the_old() {
        // ADR-0067 decision 3. The second Rohan Batra gets the suffix; the first is never
        // touched, because every copy of an address in use is held by somebody else.
        let first = derive("ROHAN BATRA", &id(1), None);
        let second = derive("ROHAN BATRA", &id(2), Some(2));
        assert_eq!(first.published, "rohan.batra@paridyum.com");
        assert_eq!(second.published, "rohan.batra2@paridyum.com");
        assert_ne!(first.mailbox, second.mailbox, "mailboxes never collide");
    }

    #[test]
    fn a_three_part_name_keeps_every_part() {
        // Dropping a middle name would make two people with the same first and last name
        // collide when they need not, and the person's own name is not ours to shorten.
        let a = derive("ADITYA KUMAR SHRIVASTAVA", &id(3), None);
        assert_eq!(a.published, "aditya.kumar.shrivastava@paridyum.com");
    }

    #[test]
    fn punctuation_and_case_are_dropped_rather_than_transliterated() {
        let a = derive("D'SOUZA  Maria-Luisa", &id(4), None);
        assert_eq!(a.published, "dsouza.marialuisa@paridyum.com");
    }

    #[test]
    fn a_name_with_no_ascii_yields_a_visibly_wrong_local_part() {
        // Not an address that looks plausible and is wrong. An operator seeing `person@`
        // knows to intervene; an invented romanisation is permanent (decision 3).
        let a = derive("роман", &id(5), None);
        assert_eq!(a.published, "@paridyum.com");
        assert!(a.mailbox.starts_with("person."), "{}", a.mailbox);
    }

    #[test]
    fn the_mailbox_carries_the_check_character_readably() {
        // `p0001473` cannot be read back: the check character and the sequence run together.
        let Addresses { mailbox, .. } = derive("ASHA VERMA", &id(147), None);
        let code = mailbox.split('.').nth(1).expect("has a code segment");
        assert!(code.starts_with("p000147c"), "{code}");
        assert_eq!(code.len(), "p000147c".len() + 1, "one check character");
    }
}
