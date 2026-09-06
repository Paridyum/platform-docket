//! Which documents a person owes at each stage of an internship.

use anyhow::{Result, bail};
use serde::Deserialize;
use std::collections::BTreeMap;

/// How a document is handled, which decides whether it gets a serial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Paridyum produces and signs it. Serialised, registered, verifiable.
    Issued,
    /// Filled in and returned by the person. Held, not issued.
    Collected,
    /// Both parties sign. Serialised, registered, countersigned.
    Signed,
}

impl Kind {
    /// The lowercase name used in the register.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Issued => "issued",
            Self::Collected => "collected",
            Self::Signed => "signed",
        }
    }
}

/// A condition that decides whether a document applies to a given person.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    /// Only when Paridyum actually issued equipment.
    EquipmentIssued,
    /// Only when a reference has been asked for.
    ReferenceRequested,
    /// Only when the person is under eighteen on their start date.
    IsChild,
    /// Only when the academic institution asks for a confirmation letter.
    InstitutionRequiresNoc,
}

/// One document the system knows how to produce.
#[derive(Debug, Clone, Deserialize)]
pub struct Document {
    /// Short code used in serials, e.g. `NDA`.
    pub code: String,
    /// Stable identifier.
    pub id: String,
    /// Title as printed on the document.
    pub title: String,
    /// Path to the template, from the root.
    pub template: String,
    /// Person classes this document applies to.
    ///
    /// WRITTEN OUT EXPLICITLY IN THE REGISTRY, never left to a default. Empty means
    /// every class, which is the backwards-compatible reading and NOT the one the
    /// registry relies on: an officer is not offered a position, has no guardian and
    /// has no institution to confirm, and several templates say "Intern" in their body
    /// text. A document owed to the wrong class produces a document that is false.
    #[serde(default)]
    pub classes: Vec<String>,
    /// How the document is handled.
    pub kind: Kind,
    /// Retention class name.
    pub retention: String,
    #[serde(default)]
    /// Signatory ids who sign for Paridyum.
    pub signed_by: Vec<String>,
    /// Signatories for a particular person class, where they differ.
    ///
    /// RFC-0042 decision 4: an officer's documents are executed by the Director, not by
    /// the CEO/CTO pair, so that no document is signed by its own subject. Keyed by
    /// class because who signs depends on who the document is ABOUT, which `signed_by`
    /// alone cannot express.
    #[serde(default)]
    pub signed_by_class: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    /// Who countersigns, where anyone does.
    pub countersigned_by: Option<String>,
    #[serde(default)]
    /// Condition under which this document applies at all.
    pub required_if: Option<Condition>,
    /// Issued when someone asks for it, rather than owed by a lifecycle stage.
    ///
    /// Three documents are like this and each for a different reason: a pack
    /// cover sheet describes a pack rather than being part of one, a business
    /// letter is not about an internship at all, and publicity consent must not
    /// be demanded — a consent a stage REQUIRES is not freely given.
    ///
    /// Declared rather than left to `check` to warn about forever. The warning
    /// "declared but owed by no stage" is right for a document nobody wired up and
    /// wrong for these, and a warning that is always present is one an operator
    /// learns to scroll past — which is how the next genuine orphan hides among
    /// the three known ones.
    #[serde(default)]
    pub on_request: bool,
}

/// One stage of the internship, and what it adds to the obligations.
#[derive(Debug, Clone, Deserialize)]
pub struct Stage {
    /// Stage name. `stage:` is the key in lifecycle.yaml, so the field is
    /// renamed for Rust and the wire format is preserved by serde.
    #[serde(rename = "stage")]
    pub name: String,
    /// Document codes that must be recorded as EXECUTED before a person may
    /// leave this stage. Issuance is not enough.
    #[serde(default)]
    pub requires_signed: Vec<String>,
    /// What this stage is.
    pub description: String,
    /// Document codes this stage adds to the obligations.
    pub owes: Vec<String>,
}

/// The lifecycle as declared in `registry/lifecycle.yaml`.
#[derive(Debug, Clone, Deserialize)]
pub struct Lifecycle {
    /// Every document the system can produce.
    pub documents: Vec<Document>,
    /// The ordered stages.
    pub stages: Vec<Stage>,
}

impl Document {
    /// Whether this document is owed to someone of `class` at all.
    ///
    /// An empty `classes` means every class. That is the permissive reading, chosen so
    /// that a document added without the field keeps working rather than silently
    /// disappearing from everyone's obligations — the failure that is hardest to see.
    #[must_use]
    pub fn applies_to(&self, class: &str) -> bool {
        self.classes.is_empty() || self.classes.iter().any(|c| c == class)
    }

    /// Who signs this document for Paridyum, given who it is about.
    ///
    /// Falls back to `signed_by` where the class has no override, which is every
    /// document for an intern.
    #[must_use]
    pub fn signatories_for(&self, class: &str) -> &[String] {
        self.signed_by_class
            .get(class)
            .map_or(self.signed_by.as_slice(), Vec::as_slice)
    }

    /// Whether this document is owed a signature back.
    ///
    /// True for anything both parties sign, anything collected from the
    /// person, and anything with a `countersigned_by`. That last clause
    /// matters: the Internship Offer is `kind: issued` because Paridyum
    /// produces it, yet it carries an acceptance block the person signs, so
    /// it IS owed back. Keying only on `kind` marked it complete the moment
    /// it was posted.
    ///
    /// A certificate and a reference letter are issued BY Paridyum with no
    /// countersignature, so nobody owes them back.
    #[must_use]
    pub fn needs_countersignature(&self) -> bool {
        matches!(self.kind, Kind::Signed | Kind::Collected) || self.countersigned_by.is_some()
    }
}

impl Lifecycle {
    /// Index documents by code, rejecting duplicates.
    ///
    /// A duplicate code would make serial allocation ambiguous and let one
    /// document silently shadow another.
    ///
    /// # Errors
    /// Fails if a document code is declared twice.
    pub fn by_code(&self) -> Result<BTreeMap<&str, &Document>> {
        let mut map = BTreeMap::new();
        for d in &self.documents {
            if map.insert(d.code.as_str(), d).is_some() {
                bail!(
                    "document code {:?} is declared twice in lifecycle.yaml",
                    d.code
                );
            }
        }
        Ok(map)
    }

    /// The stage after `stage`, or `None` at the end of the lifecycle.
    ///
    /// # Errors
    /// Fails if `stage` is not declared.
    pub fn next_after(&self, stage: &str) -> Result<Option<&Stage>> {
        let Some(at) = self.stages.iter().position(|s| s.name == stage) else {
            let known: Vec<&str> = self.stages.iter().map(|s| s.name.as_str()).collect();
            bail!("unknown stage {stage:?}; registry declares {known:?}");
        };
        Ok(self.stages.get(at + 1))
    }

    /// The stage by name.
    #[must_use]
    pub fn stage(&self, name: &str) -> Option<&Stage> {
        self.stages.iter().find(|s| s.name == name)
    }

    /// Every document owed at `stage` and at every stage before it.
    ///
    /// Obligations are cumulative: reaching `active` does not discharge what
    /// `preboarding` owed.
    ///
    /// # Errors
    /// Fails if `stage` is not declared in the registry.
    pub fn owed_upto(&self, stage: &str) -> Result<Vec<String>> {
        self.owed_upto_for(stage, None)
    }

    /// Every document owed at `stage` and before, narrowed to a person class.
    ///
    /// `None` means every class, which is what callers that have no person want — the
    /// pack manifest and `check`'s "declared but owed by no stage" sweep both reason
    /// about the registry rather than about somebody.
    ///
    /// # Errors
    /// Fails if `stage` is not declared in the registry.
    pub fn owed_upto_for(&self, stage: &str, class: Option<&str>) -> Result<Vec<String>> {
        let Some(position) = self.stages.iter().position(|s| s.name == stage) else {
            let known: Vec<&str> = self.stages.iter().map(|s| s.name.as_str()).collect();
            bail!("unknown stage {stage:?}; registry declares {known:?}");
        };
        let mut owed = Vec::new();
        for s in self.stages.iter().take(position + 1) {
            for code in &s.owes {
                if !self.applies(code, class) {
                    continue;
                }
                if !owed.contains(code) {
                    owed.push(code.clone());
                }
            }
        }
        Ok(owed)
    }

    /// Whether `code` applies to `class`, treating an unknown code as applying.
    ///
    /// An undeclared code is `check`'s problem to report, not this function's to hide:
    /// dropping it here would turn a registry error into a document quietly never owed.
    fn applies(&self, code: &str, class: Option<&str>) -> bool {
        let Some(class) = class else { return true };
        self.documents
            .iter()
            .find(|d| d.code == code)
            .is_none_or(|d| d.applies_to(class))
    }
}

#[cfg(test)]
// A test that cannot assert an expected value is not a test. The lint stays
// denied for the crate; only assertions inside test modules are exempt.
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::{Kind, Lifecycle};

    fn fixture() -> Lifecycle {
        serde_yaml::from_str(
            r"
documents:
  - {code: OFR, id: offer, title: Offer, template: t/o.tex, kind: issued, retention: contract}
  - {code: NDA, id: nda, title: NDA, template: t/n.tex, kind: signed, retention: contract}
  - {code: EQP, id: eqp, title: Equipment, template: t/e.tex, kind: collected,
     retention: personnel, required_if: equipment_issued}
stages:
  - {stage: offered, description: d, owes: [OFR]}
  - {stage: preboarding, description: d, owes: [NDA]}
  - {stage: active, description: d, owes: [EQP]}
",
        )
        .expect("fixture parses")
    }

    #[test]
    fn obligations_accumulate_across_stages() {
        let l = fixture();
        assert_eq!(l.owed_upto("offered").expect("known"), ["OFR"]);
        assert_eq!(l.owed_upto("preboarding").expect("known"), ["OFR", "NDA"]);
        assert_eq!(l.owed_upto("active").expect("known"), ["OFR", "NDA", "EQP"]);
    }

    #[test]
    fn an_unknown_stage_names_the_ones_that_exist() {
        let err = l_err(&fixture(), "onboarding");
        assert!(
            err.contains("offered"),
            "error should list known stages: {err}"
        );
    }

    fn l_err(l: &Lifecycle, stage: &str) -> String {
        l.owed_upto(stage)
            .err()
            .map_or_else(String::new, |e| e.to_string())
    }

    #[test]
    fn duplicate_document_codes_are_rejected() {
        let l: Lifecycle = serde_yaml::from_str(
            r"
documents:
  - {code: NDA, id: a, title: A, template: a.tex, kind: signed, retention: contract}
  - {code: NDA, id: b, title: B, template: b.tex, kind: signed, retention: contract}
stages: []
",
        )
        .expect("parses");
        assert!(
            l.by_code().is_err(),
            "a duplicate code must not be indexable"
        );
    }

    #[test]
    fn kinds_round_trip_to_the_register_spelling() {
        assert_eq!(Kind::Issued.as_str(), "issued");
        assert_eq!(Kind::Collected.as_str(), "collected");
        assert_eq!(Kind::Signed.as_str(), "signed");
    }
}
