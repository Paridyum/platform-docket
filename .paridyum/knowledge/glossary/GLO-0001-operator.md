---
id: GLO-0001
title: Operator — a principal with breadth within one tenant
type: glossary
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [tenancy, identity, vocabulary, authorization]
breaking: false
enforced_by:
  - check: db.operator_scope_is_an_explicit_grant
    severity: error
    rationale: >
      The corrected definition, stated as a check. Operator scope is what this term now names, and
      a policy that widens access with a NULL, a wildcard or a coalesce on the account
      discriminator is the inferred breadth the definition forbids (ADR-0109 D1, GLO-0005).
---

# GLO-0001 — Operator

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0114.** The party this entry once defined — the one that
> holds a Paridyum subscription and owns several tenants — **now has a term: org** (GLO-0012). This
> entry's corrected meaning is unchanged: an operator is a principal with breadth within **one**
> tenant.

## Correction — 2026-09-26, by ADR-0109

**This entry used to define the party that pays Paridyum, and that was the wrong word for it.**
The correction is recorded here rather than by superseding the entry (ADR-0109 decision 2), because
what was wrong was the *word*, not the structural claim: a paying party may indeed hold several
tenants, and that remains true.

Two readings of `operator` were in circulation. **Reading A** — the commercial party, spanning
*tenants* — lived in this entry, in GLO-0004's tree and in BET-0005's table. **Reading B** — a
runtime grant, spanning *accounts within one tenant* — lives in GLO-0005, in twelve **applied and
therefore immutable** migrations, in `@paridyum/auth`'s published API and in `tool-docket`.

Reading B wins, and the reason is not preference. Renaming it would mean editing applied migrations,
which REQ-0008 forbids and `migrate.py` refuses, plus a breaking change to a published library API.
Reading A cost nothing to correct because it was a declaration and not an implementation. **When two
readings disagree, the one already written into something immutable wins and the one that is only
claimed gets corrected** — the estate's rule for schemas, applied to vocabulary.

`operator_id` is removed below. It was declared here and exists in **no migration and no code**,
which is the same defect class as an `enforced_by` check id that resolves to nothing (ADR-0014): it
reads as a fact and is a plan. It returns when a migration introduces it.

**The paying party above a tenant now has no single word, deliberately** — see *What it is not*.

## Definition

**An operator is a principal that acts with breadth across a whole tenant, rather than within one
account inside it.**

Operator scope is an **explicit positive grant** on a verified token, never an absence and never an
inference (GLO-0005). It is the claim that distinguishes a tenant's own administrator — who
legitimately reads every account its business holds — from an end user, who may read one.

In the database it is `paridyum.operator_scope`, one of the session values set per transaction
(ADR-0038), read by the second half of every inner policy predicate:
`account_id = paridyum.current_account() OR paridyum.has_operator_scope()`.

## What it is not

- **Not the party that pays.** A paying party may hold several tenants — BET-0002 prices per running
  business, plural — and that party is **not modelled and is deliberately unnamed**. It is ADR-0024
  decision 9's deferred third level, and D9's rule governs: *"a real third level with a paying
  customer behind it… **Not 'we can imagine one.'**"* Naming a rung nothing implements is a
  commitment that something exists to be named. It gets a term when it is funded, from whoever funds
  it. Until then, documents that need the concept write the phrase out — a small tax that keeps the
  imagined level visible as imagined.
- **Not a tenant.** A tenant (GLO-0002) is the isolated business. Operator scope is something a
  principal *holds over* one.
- **Not a user, and not a role name in itself.** In Zitadel it arrives as a project role grant
  (`operator`) on a member of the tenant's organization; the word here names the *scope that grant
  confers*, not the string.
- **Not an account.** An account (GLO-0003) is one product instance and its record set, a level
  below. Operator scope is what reads across all of them.
- **Not Paridyum-the-platform.** GLO-0004's tree has Paridyum above every tenant as the platform
  that runs them; that position is not operator scope, and tenant zero holds no grant a customer
  could not hold (ADR-0049 D2).

## Why the word

The alternative was to rename the runtime grant and keep this entry's original subject. That is the
option ADR-0109 rejected, and the cost is worth stating plainly: **this entry's title no longer
matches the concept it was created for, and the original subject has no term at all.** That is the
honest state and it is uncomfortable on purpose — an empty slot invites someone to fill it when
ADR-0024 D9's trigger fires, where a comfortable half-definition would not.

"Operator" reads correctly for what it now means: the person operating the business inside the
runtime. C-0006's posture is unchanged — the operator runs the business, Paridyum runs the runtime.

## Related

- GLO-0005 operator scope — the grant this term names, and why an absence is never one
- GLO-0002 tenant — what operator scope spans
- GLO-0003 account — the level operator scope reads *across*
- GLO-0004 nested tenancy — the model these terms describe
- ADR-0109 — the decision that corrected this entry
- ADR-0024 — `locked`; decision 9 defers the commercial level above a tenant
- ADR-0038 — the session values, of which operator scope is one
- BET-0002 — prices per running business; that party is the unnamed one above a tenant
