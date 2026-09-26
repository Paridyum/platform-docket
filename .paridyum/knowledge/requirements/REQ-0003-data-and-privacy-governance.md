---
id: REQ-0003
title: Personal data is classified, minimised, retained on a schedule and deletable
type: requirement
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [privacy, pii, retention, encryption, compliance, data-governance]
breaking: false
trigger: >
  The first service stores a field that identifies a natural person — name, email,
  phone, address, IP, device id, or any identifier that can be linked back to one.
review_date: 2027-02-01
verified_by:
  - TST-0041
enforced_by: []
---

# REQ-0003 — Personal data is classified, minimised, retained on a schedule and deletable

## Status

**DEFERRED** as of 2026-08-07. Nothing stores data. There are no services, no databases
and no users, so every rule here would today govern nothing — ADR-0008's test.

Deferred is not "unimportant". This is the domain with the shortest distance between a
missing control and a legal problem, and unlike most technical debt it **cannot be paid
down retroactively**: personal data written to logs a year ago is in a year of backups.
The trigger is deliberately the *first* personal field, not the first release.

## Status

**ACCEPTED**, 2026-09-11. The trigger — "the first service stores a field that identifies a
natural person" — fired when `platform-tenancy/migrations/0003_a_candidate_profile.sql` created
the `profiles` table with `legal_name`, `date_of_birth` and the rest. The SCHEMA exists; no real
person is stored yet (ADR-0071 D3's backup gate holds that), which is the window in which the
controls must be in place rather than retrofitted.

**Item by item (ADR-0073 fixed most of this for the candidate profile):**

1. **Classification** — done in 0003: every column carries a `-- <class>` comment. A check that
   asserts it against the live catalogue is owed (REQ-0022 criterion 4's cousin).
2. **Minimisation** — done: ADR-0073 D1 fixed the smallest field set; email is not re-collected
   (Zitadel's, ANTI-0001); no statutory ids pre-offer.
3. **Never in logs/traces** — **MET, 2026-09-12.** ADR-0080 decides it: a classified field has no
   exit into a log; logging is typed and takes primitives, never entities; an error logs a
   correlation id (ADR-0044), not the payload. Enforced today by `no-console` in the web app (zero
   console usage, kept that way), which is a proxy the ADR names honestly; a structured-logging
   check would lock it.
4. **Retention** — policy set (ADR-0073 D3: 12 months from `updated_at`), and the RUNNER now exists:
   `platform-tenancy/scripts/retention.py`, dry-run by default, executed live (0 rows past the
   window on empty tables). **What remains is scheduling** `--apply` on a cron — until it runs
   unattended on a schedule the control is present but not yet standing.
5. **Deletion on request** — done: `DELETE /api/profile` (REQ-0003 item 5), and ADR-0071 D5 reaches
   the backups.
6. **Non-production data** — held by the gate: no real data in staging until backups are rehearsed.
7. **Encryption** — at rest via the volume; backups are age-encrypted before leaving the host
   (ADR-0071 D1). Field-level encryption not used and not yet needed.
8. **Residency** — readable personal data stays in India; only ciphertext leaves (ADR-0071).
9. **AI/ML use** — **MET, 2026-09-12.** ADR-0080 D5/D6 answers it: candidate profile and application
   data is used to train, fine-tune, evaluate or profile **no** model, and there is no automated
   decision-making over it — datasets: none; by whom: no one. Introducing any such use supersedes
   that decision and passes ANTI-0007's gate first.

The remaining backlog this makes visible is now narrow: **schedule the retention runner** (item 4's
`--apply` on a cron). Logging discipline (item 3) and the AI/ML-use answer (item 9) are decided by
ADR-0080; the live-catalogue classification assertion (item 1's cousin) still awaits tool-forge.

## Requirement

**Every field holding personal data shall be classified at the point it is designed,
minimised to what the service actually needs, retained on a stated schedule, and
deletable on request — and personal data shall never reach logs, traces or error
reports.**

## Numbered items

1. **Classification.** Every column carries one of `public`, `internal`, `confidential`,
   `pii`. Classification happens in the migration that creates the column, not afterwards.
2. **Minimisation.** A field that is not needed to satisfy a stated requirement is not
   collected. This is the cheapest privacy control and the only one that is free.
3. **Never in logs, traces, analytics or error trackers.** The most common breach is not
   a stolen database; it is an email address in a log line replicated to three vendors.
   Structured logging must make it hard to pass a whole entity into a log.
4. **Retention.** Every `pii` field has a retention period and a deletion mechanism that
   has actually been run. A retention policy nobody has executed is a claim, not a control.
5. **Deletion on request.** GDPR/CCPA subject requests need deletion to be possible across
   every service that holds a copy — which is a decomposition question (ANTI-0003:
   projections mean copies) and must be answered when boundaries are drawn, not later.
6. **Non-production data.** No production personal data in dev or staging. Masked or
   synthetic only.
7. **Encryption.** At rest and in transit as a baseline; field-level for the most
   sensitive, with a stated key-management story.
8. **Residency.** Where personal data may physically live, and which cross-border
   transfers are permitted.
9. **AI/ML use.** Which datasets may be used for training or evaluation, and by whom.

## Why this is not startable today

Items 1, 4 and 5 need a schema. Item 3 needs a logging stack. Items 7–8 need infrastructure
and a jurisdiction. Writing them as binding rules now produces the failure ADR-0008 names:
*"empty scaffolding is worse than none because it advertises coverage that does not
exist."*

What **is** cheap now, and is already true: `owns.tables` in `manifest.yaml` is the natural
place for a per-table classification, and `db/grants.sql` is the natural place for
column-level access restriction. The hooks exist; the data does not.

## Related

- ANTI-0003 — projections mean copies, which is what makes deletion hard
- C-0004 — credentials are never committed; adjacent, not the same concern
- REQ-0001 — runtime secret delivery, a prerequisite for key management
- STD-0002 — `manifest.yaml`, where classification would live
- ADR-0004 — plan limitations that affect what controls are even available
