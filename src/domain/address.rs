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
    /// `firstname.p000147c7.paridyum@gmail.com` — where mail lands.
    pub mailbox: String,
}

/// Derive both addresses from a legal name and a person identifier.
///
/// ONE SHAPE, AND IT CANNOT COLLIDE. Every address is `firstname.<person id>`, derived rather
/// than allocated. The identifier is unique across Paridyum forever and is never reissued
/// (ADR-0051), so uniqueness and never-reuse are inherited rather than enforced.
///
/// **This is what removes the address register.** A `firstname.lastname` scheme has to know
/// every address ever issued before it can hand out the next one — including to people who
/// have left, and to people whose records were erased, because reuse delivers one person's
/// password resets to another. Deriving from an identifier that already guarantees both
/// properties means there is nothing to look up and nothing to keep.
///
/// It also makes an address self-validating: a mistyped `firstname.lastname` goes nowhere
/// silently, while a mistyped identifier fails the MOD 11-2 check character ADR-0051 attached
/// to it — the error is detectable without asking anybody.
///
/// The cost, and it is real: this is harder to read aloud than a surname. The check character
/// is why that is survivable rather than free.
#[must_use]
pub fn derive(legal_name: &str, person_id: &PersonId) -> Addresses {
    let given = name_parts(legal_name)
        .first()
        .cloned()
        .unwrap_or_else(|| "person".to_owned());
    let code = squash(person_id);

    Addresses {
        published: format!("{given}.{code}@{DOMAIN}"),
        mailbox: format!("{given}.{code}.paridyum@{MAILBOX_DOMAIN}"),
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

/// `PAR-P-000147-7` as `p000147c7`, for a namespace that allows no hyphens.
///
/// The `PAR-` prefix is dropped because the account name already ends in `paridyum`, and the
/// check character is joined with `c` so the identifier can be read back out unambiguously —
/// `p0001477` would not say where the sequence stops and the check begins.
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
    fn both_addresses_share_one_derivation() {
        let a = derive("ROHAN BATRA", &id(147));
        assert_eq!(a.published, "rohan.p000147c7@paridyum.com");
        assert_eq!(a.mailbox, "rohan.p000147c7.paridyum@gmail.com");
    }

    #[test]
    fn two_people_with_the_same_full_name_cannot_collide() {
        // THE WHOLE POINT. There is no cascade, no suffix and no register to consult, because
        // the identifier already guarantees uniqueness and refuses reissue (ADR-0051).
        let first = derive("ROHAN BATRA", &id(1));
        let second = derive("ROHAN BATRA", &id(2));
        assert_ne!(first.published, second.published);
        assert_ne!(first.mailbox, second.mailbox);
    }

    #[test]
    fn the_surname_is_not_used_at_all() {
        // Not shortened, not initialised -- absent. Two people sharing a first and last name
        // is the case a surname cannot disambiguate anyway.
        let a = derive("ADITYA KUMAR SHRIVASTAVA", &id(3));
        assert!(!a.published.contains("shrivastava"));
        assert!(!a.published.contains("kumar"));
        assert!(
            a.published.starts_with("aditya.p000003c"),
            "{}",
            a.published
        );
    }

    #[test]
    fn a_single_name_needs_no_special_case() {
        // Common enough in India that it is not an edge case, and it costs nothing here
        // because only the given name was ever used.
        let a = derive("RAVI", &id(4));
        assert!(a.published.starts_with("ravi.p000004c"), "{}", a.published);
    }

    #[test]
    fn punctuation_and_case_are_dropped_rather_than_transliterated() {
        let a = derive("D'SOUZA  Maria-Luisa", &id(6));
        assert!(
            a.published.starts_with("dsouza.p000006c"),
            "{}",
            a.published
        );
    }

    #[test]
    fn a_name_with_no_ascii_yields_a_visibly_wrong_local_part() {
        // Not an address that looks plausible and is wrong. An operator seeing `person.` knows
        // to intervene; an invented romanisation would be permanent, because ADR-0067 forbids
        // renaming an address once issued.
        let a = derive("\u{440}\u{43e}\u{43c}\u{430}\u{43d}", &id(7));
        assert!(
            a.published.starts_with("person.p000007c"),
            "{}",
            a.published
        );
    }

    #[test]
    fn the_identifier_carries_its_check_character_readably() {
        // `p0001477` cannot be read back: the check character and the sequence run together.
        // ADR-0051 added that character because this string gets typed and read aloud -- which
        // is now true of every address, not only a folder name.
        let Addresses { published, .. } = derive("ASHA VERMA", &id(147));
        let code = published
            .split('@')
            .next()
            .and_then(|l| l.split('.').nth(1));
        let code = code.expect("has a code segment");
        assert!(code.starts_with("p000147c"), "{code}");
        assert_eq!(code.len(), "p000147c".len() + 1, "one check character");
    }
}
