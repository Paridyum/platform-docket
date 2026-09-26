---
id: ADR-0073
title: A candidate profile is a minimal classified subset with a retention clock and a delete button
type: adr
status: accepted
created: 2026-09-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, web]
applies_to: ["*"]
product: [tenancy]
tags: [profile, candidate, personal-data, dpdp, req-0003, adr-0068, adr-0063, adr-0038, adr-0071, tool-docket, minimisation, retention, deletion]
breaking: false
enforced_by: []
---

# ADR-0073 — A candidate profile is a minimal, classified subset with a retention clock

## Status

**Accepted**, 2026-09-07. Not `locked`: the rules here are mostly checkable — a column with no
`paridyum:class`, a `pii` field with no retention entry — and `check-migrations.sh` already
asserts the classification half. It graduates when a check asserts the retention and deletion
halves against the live schema.

## Context

The flow is decided: a person lands, signs up, builds a profile, and applies (ADR-0068). Sign-in
is live. The profile is the missing middle, and it is the first place this estate stores personal
data a real person typed about themselves — so REQ-0003, deferred since 2026-08-07, comes fully
due here rather than in the abstract.

Three questions have no answer yet, and each is a decision rather than a lookup.

**What fields.** `tool-docket`'s `fields.yaml` defines a person's full record — legal name, date
of birth, contact, education, professional, and the statutory identifiers (PAN, Aadhaar, UAN) an
enrolled intern needs. A candidate is not enrolled. Collecting a PAN to *apply* is the
over-collection `person-classes.yaml` already forbids for an intern, one stage earlier.

**What classification.** REQ-0003 item 1 requires every column carry `public`, `internal`,
`confidential` or `pii`, in the migration that creates it.

**How long, and how it leaves.** ADR-0063 explicitly deferred candidate retention to "the field
registry Phase B needs". This is Phase B. An unsuccessful application kept forever is a standing
liability; deleted the moment it is rejected is useless for a re-application a month later.

## Decision

### 1. The field set is the smallest that lets an operator evaluate a candidate, and no more

Reusing `fields.yaml`'s definitions (ADR-0068 §3 — one definition for both stores), the candidate
profile is exactly:

| Field | Class | Required | Why it is needed to *apply* |
| --- | --- | --- | --- |
| `legal_name` | pii | yes | The name on any offer that follows. |
| `preferred_name` | pii | no | How the candidate is addressed. |
| `pronouns` | internal | no | Correct reference; never inferred (defaults they/them). |
| `date_of_birth` | pii | yes | **The DPDP s9 gate.** Whether the applicant is a child decides whether their data may be processed at all without guardian consent. Collected to *see* the obligation, per `fields.yaml`'s own correction. |
| `phone` | pii | no | A second contact channel. |
| `institution` | confidential | yes | An internship is tied to enrolment; this is the evaluation. |
| `currently_enrolled` | internal | yes | Unpaid interns must be enrolled (ADR/`person-classes`). |
| `degree`, `specialization`, `graduation_year` | internal | no | Context for the evaluation. |
| `linkedin`, `github` | internal | no | Portfolio the candidate chose to share. |
| `skills` | internal | no | What they say they can do. |

**Email is not a profile field.** Zitadel already holds the verified login email (ADR-0038: the
account and its email are the identity's, not ours). Re-collecting it would be the second writer
ANTI-0001 forbids. Contact is the login email plus the optional phone above.

**No statutory identifiers, no address, no compensation.** A candidate is pre-offer. Those fields
belong to `tool-docket` at onboarding (the `offered` seam, ADR-0068 §3), and collecting them here
is over-collection by the same rule that governs an intern's record.

### 2. Every column is classified in the migration, and `pii` is the tripwire

REQ-0003 item 1, mechanically: each column carries a `-- paridyum:class` directive, and
`check-migrations.sh`'s `data.regulated_class_requires_control_evidence` already reads it. A
column that stores a person and carries no class is the defect the check exists to catch.

### 3. A profile has a retention clock, and the clock is a stated maximum, not a hope

**An unsuccessful profile is retained twelve months from its last update, then deleted.** Long
enough that a candidate re-applying within an academic year is not re-typing everything; short
enough that a rejected application is not a liability held for years. A profile that becomes an
offer graduates into `tool-docket` and its retention (`retention.yaml`) takes over — this clock
governs only the pre-offer life.

**The clock is not a control until something runs it.** REQ-0003 item 4 is explicit: "a retention
policy nobody has executed is a claim". So the deletion job is owed alongside the schema, and
until it exists and has run once, the retention here is a documented intention, not a discharged
obligation. This is stated so it is not mistaken for done.

### 4. Deletion on request is a button the candidate has, and it reaches the backups

REQ-0003 item 5. A candidate can delete their own profile. The row is deleted, and — per
ADR-0071 D5 — the person is excluded from the next dump while older dumps age out under the
retention schedule, which is the stated maximum on how long an erasure takes to become total. A
delete that leaves the row in every backup forever is not a delete.

### 5. The profile is account-scoped, and the write is gated exactly like `apply`

The `profiles` table lives beside `applications`: account-scoped, RLS enabled and forced, tenant
and account from `resolve_account` (ADR-0038). Every write is behind `PARIDYUM_BACKUPS_CONFIRMED`
(ADR-0071 D3) — a profile is personal data, so it waits for the off-host restore rehearsal the
same as an application does. Building the schema and the pages is not gated; storing a real
person is.

### 6. Sign-up is Zitadel's, in the hiring org, and it is the only way in

A candidate registers through Zitadel's own registration on the `paridyum-hiring` organization —
Paridyum never runs its own password form (ADR-0047 D4: Paridyum does not mint or validate). The
profile is reachable only to a verified session; there is no anonymous profile.

## Consequences

**Accepted costs**

- **REQ-0003 is now live and only partly dischargeable.** Classification (item 1) and deletion-
  on-request (item 5) ship with the schema and the endpoint. Retention execution (item 4), the
  logging discipline (item 3), and the AI/ML-use answer (item 9) are owed and not yet built; each
  is named here rather than silently skipped.
- **A twelve-month retention with no runner is a claim until the job exists.** The honest state is
  that the clock is documented and unenforced, and a real profile should not be stored until the
  runner has run once — the same shape as ADR-0071's "a backup never restored is a belief".
- **`date_of_birth` on a candidate will read as over-collection to anyone who has not read the
  DPDP s9 argument.** It is the opposite: it is collected to *know whether* s9's guardian-consent
  obligation applies, and not collecting it would remove the ability to see the obligation, not
  the obligation. `fields.yaml` made and corrected this exact mistake once already.

**Gained**

- The portal has a defined, minimal, classified data model instead of a form that grows by
  accretion — which is how a privacy liability is built without anyone deciding to.
- One field definition serves both the candidate profile and the eventual docket record, so the
  offer seam (ADR-0068 §3) is a copy of known fields, not a translation.
- REQ-0003 stops being deferred and starts being tracked item by item, with the undischarged
  items visible.

**Rejected alternatives**

- **Store the full `fields.yaml` record from the start.** Simplest to map to docket later, and it
  collects a child's Aadhaar to apply for an internship. Rejected by minimisation (REQ-0003 item
  2) and by `person-classes.yaml`'s existing prohibition, one stage earlier.
- **Re-collect email into the profile.** Convenient for a single query. Rejected: it is Zitadel's
  fact, and a second writer drifts (ANTI-0001).
- **No retention clock, decide later.** What ADR-0063 did, correctly, when there was no store.
  There is a store now, so "later" has arrived, and leaving it unset is choosing indefinite
  retention by omission.
- **Collect date of birth only for successful candidates.** Defers the s9 question past the point
  where data is already being processed, which is exactly when the answer is needed.
