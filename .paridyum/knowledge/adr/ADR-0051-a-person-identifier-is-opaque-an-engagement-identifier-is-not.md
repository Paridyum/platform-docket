---
id: ADR-0051
title: A person identifier is opaque and permanent; an engagement identifier carries class and period
type: adr
status: locked
created: 2026-08-19
locked_at: 2026-08-19
supersedes: null
superseded_by: null
scope: [org, tool]
applies_to: ["*"]
tags: [identifiers, records, archival, dpdp, tool-docket, adr-0040, rfc-0015]
breaking: true
enforced_by:
  - check: docket.person_identifier_is_opaque
    severity: error
    rationale: >
      An identifier carrying a mutable fact stops being true, and a folder
      named by it then lies about its contents. Enforced by construction:
      there is no code path that builds a person identifier from anything but
      a sequence number.
  - check: docket.engagement_identifier_carries_class_and_year
    severity: error
    rationale: >
      The other half of the decision. Dropping class from the engagement
      identifier as well would lose the fact entirely, and class is exactly
      what is immutable at this level.
  - check: docket.identifiers_are_never_reused
    severity: error
    rationale: >
      Reuse is the one failure that silently reattaches one person's documents
      to another. It is the reason sequence allocation takes a maximum rather
      than a count.
---

# ADR-0051 — A person identifier is opaque and permanent; an engagement identifier carries class and period

## Status

**LOCKED** as of 2026-08-19. Not locked: the rule is mechanically checkable and is
checked, but only by `docket check` inside `tool-docket`. Promoting it to `locked`
requires check ids in `check-registry.yaml`, which is a separate registry edit and a
separate commit (§5b of the `new-decision` skill, and the ordering ADR-0029's errata
exist because of).

## Context

`tool-docket` issues Paridyum's internship documents and keeps the register that says who
signed what. It allocated one identifier per person, of the form `PAR-INT-2026-0001`:
organisation, **class**, **year**, sequence.

Two forces met.

**The archive wants one identifier per human, forever.** A certificate issued in 2026 must
still name the same person in 2036. Personal documents are about to be linked to a remote
cloud folder, and that folder's name is the most permanent string in the system — it is
copied into links, quoted in mail, and read aloud.

**The identifier as designed encodes two facts that change.** `INT` is a class and `2026`
is a year. The predictable case is the one that breaks it: an intern is hired. Either the
folder stays labelled `INT` forever and lies, or the person gets a second identifier and
the archive splits in two. A returning intern doing a second summer has the same problem.

Persistent-identifier practice is explicit about this and has been for two decades:
identifiers meant to persist should be **opaque**, because non-opaque identifiers "do not
age or travel well" — roles, titles and dates are exactly the mutable attributes that
defeat persistence. The ARK scheme makes its authority numbers opaque for this reason.

A third force pointed the same way. A folder named `asha-verma/` in a shared drive
discloses who Paridyum engages to anyone who can see the *listing*, without opening
anything. Under the DPDP Act that listing is personal data. An opaque identifier is not.

Against all that: two identifiers are harder to explain than one, and an opaque string
tells a human nothing. That cost is real and is accepted below.

## Decision

**Two identifiers, with a line between them that is not arbitrary.**

**A person identifier is opaque and permanent.**

    PAR-P-000147-3
    │   │ │      └── ISO 7064 MOD 11-2 check character (0-9 or X)
    │   │ └───────── opaque sequence, zero-padded to six digits
    │   └─────────── P for person
    └─────────────── organisation

It encodes **no** fact about the person. Not their class, not their year, not their name.
It is allocated once, never reissued — not after separation, not after erasure — and it is
the **only** string used to name a folder, a directory, or anything else outside this
system.

The check character exists because this string gets typed and read aloud. MOD 11-2 catches
every single-digit error and every transposition of adjacent digits, which is the error
class a hand-copied folder name actually produces.

**An engagement identifier carries class and period, and that is correct.**

    PAR-INT-2026-0001

The distinction is the whole decision: **class is mutable for a person and immutable for an
engagement.** A person may be an intern and later an employee. An engagement that was an
internship was an internship permanently. Encoding class in the engagement identifier
therefore encodes a fact that cannot go stale, which is the test the person identifier
fails.

**Documents reference the engagement. Folders and archives are named by the person.** One
person may hold many engagements; the archive stays whole across all of them.

**A slug is a filename, never an identity.** It exists so a human can find a record on
disk. Nothing keys off it — not the register, not a folder, not a document. It is reserved
forever regardless, because reusing one would attach a previous person's documents to a
new one.

## Consequences

**Accepted costs.**

- **Two identifiers to hold in your head** where there was one, and the opaque one tells a
  reader nothing. Preservation practice's own answer applies: opaque identifiers are
  workable *because* metadata sits beside them. The generated index (ADR-0040's pattern —
  generated, never hand-authored) is that metadata, and it is now load-bearing rather than
  a convenience.
- **A migration.** Existing identifiers of the form `PAR-INT-2026-0001` become engagement
  identifiers and every person needs a new person identifier. `breaking: true` is not
  decoration. It is cheap only because it is being done before any folder exists, which is
  the entire reason for deciding now rather than later.
- **A check character makes the identifier uglier** and adds an arithmetic step to
  validation. Accepted for the transcription safety.
- **Nothing enforces this outside `tool-docket` yet.** See Status.

**Gained.**

- An intern who becomes an employee keeps one folder and one archive.
- A folder listing no longer discloses who Paridyum engages.
- No identifier in the system encodes a fact that can become false.
- Transposing two digits of a folder name is detected rather than silently opening the
  wrong person's records.

**Rejected alternatives.**

- **One opaque identifier, class as data** (`PAR-P-000147` only). Simpler, and it fixes the
  staleness. Rejected because nothing then distinguishes "documents from her 2026
  internship" from "her 2027 employment" without reading every document's dates — and the
  retention class of an internship record differs from that of an employment record, so the
  distinction has to be addressable.
- **Keep one identifier and issue a second one on class change**, linked by a `previous_id`
  field. Rejected: it makes the archive a chain to be walked, and every consumer must know
  to walk it. One of them will not.
- **Encode class but not year** (`PAR-INT-000147`). Rejected for the same reason as the
  original: class is the fact that changes, so it is the worse of the two to encode.
- **A UUID.** Maximally opaque and needs no allocator. Rejected because it is unusable as
  something a person types, reads aloud, or recognises in a folder listing, and because a
  sequence plus a check character gives the same persistence for a string a human can
  handle.
- **Name folders by slug.** Rejected: it discloses identity in a listing, and a slug is a
  filename that may be corrected, which is precisely the mutability being designed out.
