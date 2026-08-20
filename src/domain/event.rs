//! The history of what happened to a person's engagement.

use crate::domain::chain::Chained;
use crate::domain::identity::PersonId;
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

/// Something that happened, worth being able to prove later.
///
/// A person record holds the CURRENT state. That is not enough to answer
/// "when was their repository access removed?", which is exactly what the
/// offboarding checklist asserts and what an audit would ask for. The record
/// is the position; this is the history that produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Record created and an identifier allocated.
    Created,
    /// Lifecycle stage moved.
    StageChanged,
    /// Role, supervisor, hours or arrangement changed.
    EngagementChanged,
    /// A system or account was granted.
    AccessGranted,
    /// A system or account was removed.
    AccessRevoked,
    /// Equipment handed over.
    EquipmentIssued,
    /// Equipment returned.
    EquipmentReturned,
    /// A document was issued and registered.
    DocumentIssued,
    /// A document was signed by all parties.
    DocumentSigned,
    /// The engagement ended.
    Separated,
    /// Personal data erased on request; the identifier survives.
    Erased,
}

impl Kind {
    /// The `snake_case` name used in the log.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::StageChanged => "stage_changed",
            Self::EngagementChanged => "engagement_changed",
            Self::AccessGranted => "access_granted",
            Self::AccessRevoked => "access_revoked",
            Self::EquipmentIssued => "equipment_issued",
            Self::EquipmentReturned => "equipment_returned",
            Self::DocumentIssued => "document_issued",
            Self::DocumentSigned => "document_signed",
            Self::Separated => "separated",
            Self::Erased => "erased",
        }
    }

    /// Parse a kind from its written name.
    ///
    /// # Errors
    /// Fails on an unknown name, listing the ones that exist.
    pub fn parse(name: &str) -> Result<Self> {
        const ALL: [Kind; 11] = [
            Kind::Created,
            Kind::StageChanged,
            Kind::EngagementChanged,
            Kind::AccessGranted,
            Kind::AccessRevoked,
            Kind::EquipmentIssued,
            Kind::EquipmentReturned,
            Kind::DocumentIssued,
            Kind::DocumentSigned,
            Kind::Separated,
            Kind::Erased,
        ];
        ALL.into_iter().find(|k| k.as_str() == name).map_or_else(
            || {
                let names: Vec<&str> = ALL.iter().map(|k| k.as_str()).collect();
                bail!("unknown event {name:?}; known: {}", names.join(" "))
            },
            Ok,
        )
    }

    /// Whether this event carries no personal data and survives erasure.
    ///
    /// An erased person keeps a skeletal history: that an engagement
    /// existed, and when it ended -- because the register still refers to
    /// their identifier. The detail, which may name systems or equipment,
    /// does not survive.
    #[must_use]
    pub fn survives_erasure(self) -> bool {
        matches!(self, Self::Created | Self::Separated | Self::Erased)
    }
}

/// One entry in the personnel event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Who it happened to.
    pub person_id: PersonId,
    /// ISO date it happened.
    pub at: String,
    /// What happened.
    pub kind: Kind,
    /// Free text: which system, which document, which piece of equipment.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// Who recorded it.
    pub recorded_by: String,
    /// Digest of the record before this one.
    pub prev_sha256: String,
    /// Digest of this record.
    pub entry_sha256: String,
}

impl Chained for Event {
    fn prev(&self) -> &str {
        &self.prev_sha256
    }
    fn set_prev(&mut self, value: String) {
        self.prev_sha256 = value;
    }
    fn digest_field(&self) -> &str {
        &self.entry_sha256
    }
    fn set_digest_field(&mut self, value: String) {
        self.entry_sha256 = value;
    }
    fn digest_key() -> &'static str {
        "entry_sha256"
    }
    fn identity(&self) -> String {
        format!("{} {}", self.person_id, self.kind.as_str())
    }
}

/// Events for one person, oldest first.
#[must_use]
pub fn for_person<'a>(log: &'a [Event], person_id: &PersonId) -> Vec<&'a Event> {
    log.iter().filter(|e| &e.person_id == person_id).collect()
}

/// Systems granted and not since revoked.
///
/// Derived from the log rather than stored, so it cannot disagree with the
/// history. This is what makes the offboarding checklist's access section
/// checkable instead of merely assertable.
#[must_use]
pub fn open_access(log: &[Event], person_id: &PersonId) -> Vec<String> {
    let mut open: Vec<String> = Vec::new();
    for e in for_person(log, person_id) {
        let Some(detail) = e.detail.as_deref() else {
            continue;
        };
        match e.kind {
            Kind::AccessGranted => {
                if !open.iter().any(|o| o == detail) {
                    open.push(detail.to_owned());
                }
            }
            Kind::AccessRevoked => open.retain(|o| o != detail),
            _ => {}
        }
    }
    open
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{Event, Kind, for_person, open_access};
    use crate::domain::chain::{seal, verify};
    use crate::domain::identity::PersonId;

    fn pid(seq: u32) -> PersonId {
        PersonId::new(seq).expect("valid")
    }

    fn event(seq: u32, kind: Kind, detail: Option<&str>) -> Event {
        Event {
            person_id: pid(seq),
            at: "2026-08-19".to_owned(),
            kind,
            detail: detail.map(ToOwned::to_owned),
            recorded_by: "cto".to_owned(),
            prev_sha256: String::new(),
            entry_sha256: String::new(),
        }
    }

    fn chain(events: Vec<Event>) -> Vec<Event> {
        let mut out: Vec<Event> = Vec::new();
        for e in events {
            out.push(seal(e, out.last()).expect("seals"));
        }
        out
    }

    #[test]
    fn access_still_open_is_derived_from_the_log() {
        let log = chain(vec![
            event(1, Kind::AccessGranted, Some("github")),
            event(1, Kind::AccessGranted, Some("vpn")),
            event(1, Kind::AccessRevoked, Some("github")),
        ]);
        assert_eq!(open_access(&log, &pid(1)), ["vpn"]);
    }

    #[test]
    fn granting_the_same_system_twice_does_not_duplicate_it() {
        let log = chain(vec![
            event(1, Kind::AccessGranted, Some("github")),
            event(1, Kind::AccessGranted, Some("github")),
        ]);
        assert_eq!(open_access(&log, &pid(1)), ["github"]);
    }

    #[test]
    fn one_persons_access_does_not_leak_into_anothers() {
        let log = chain(vec![
            event(1, Kind::AccessGranted, Some("github")),
            event(2, Kind::AccessGranted, Some("vpn")),
        ]);
        assert_eq!(open_access(&log, &pid(1)), ["github"]);
        assert_eq!(open_access(&log, &pid(2)), ["vpn"]);
        assert_eq!(for_person(&log, &pid(1)).len(), 1);
    }

    #[test]
    fn the_log_seals_into_a_chain() {
        let log = chain(vec![
            event(1, Kind::Created, None),
            event(1, Kind::Separated, None),
        ]);
        assert!(verify(&log).expect("verifiable").is_empty());
    }

    #[test]
    fn an_unknown_event_names_the_ones_that_exist() {
        let err = Kind::parse("exploded")
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();
        assert!(err.contains("access_granted"), "{err}");
    }

    #[test]
    fn every_kind_round_trips_through_its_name() {
        for name in [
            "created",
            "stage_changed",
            "engagement_changed",
            "access_granted",
            "access_revoked",
            "equipment_issued",
            "equipment_returned",
            "document_issued",
            "document_signed",
            "separated",
            "erased",
        ] {
            assert_eq!(Kind::parse(name).expect("known").as_str(), name);
        }
    }

    #[test]
    fn only_skeletal_events_survive_erasure() {
        assert!(Kind::Created.survives_erasure());
        assert!(Kind::Separated.survives_erasure());
        assert!(
            !Kind::AccessGranted.survives_erasure(),
            "detail may name systems"
        );
        assert!(!Kind::EquipmentIssued.survives_erasure());
    }
}
