---
id: ADR-0142
title: An org's region is chosen when the org is created, and its creator is admitted as a linked user
type: adr
status: accepted
created: 2026-10-10
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web, infra]
applies_to: ["*"]
product: [paridyum, tenancy, identity]
tags: [regions, residency, org, portal, signup, federation, linked-user, adr-0117, adr-0119, adr-0121, adr-0141, rfc-0015, rfc-0051]
breaking: false
enforced_by: []
---

# ADR-0142 — An org's region is chosen when the org is created, and its creator is admitted as a linked user

## Status

**ACCEPTED** 2026-10-10 by the operator. **It amends ADR-0117 D1**: an org's region is chosen
**when the org is created** — the personal org at signup, a team org at creation — and the creator
is admitted to that region as a **linked user** (ADR-0117 D7) when their credential lives elsewhere.
A global `paridyum.in` user may therefore own an org in another region, and **the credential never
moves** (ADR-0117 D6, ADR-0141 R1, R5).

It takes RFC-0015 and RFC-0051 as direction; both are `proposed`.

## Context

**ADR-0117 D1 says the region is "chosen explicitly at signup".** It was written when the only org
was the one a person got at signup. The portal has since made an org a first-class act: a person
signs in, and later creates a **team org** from a device whose credential already lives in a region
(ADR-0118, ADR-0113).

**"At signup" has no answer for a team org.** A team org is not created at a signup; it is created
from inside a signed-in session. Read literally, D1 leaves that org's region with no moment of
choice at all — and the nearest available default is the **creator's credential region**, which is
exactly the inference RFC-0015 D5 and ADR-0103 D4 refuse. *"Placement is never inferred from the
address a signup arrives from"* is the same rule one step further in: where the **creator's
credential** happens to live says nothing about where the org's people and data belong.

**paridyum.in is one global surface, so the creator and the org can be in different regions.**
A person whose credential lives in `in` can legitimately create an org whose people are in `eu`;
the org's data belongs in `eu`. ADR-0117 D2 already lets an org in `in` own a tenant in `eu`; this is
the same shape one level up, where the *org itself* is placed away from its creator.

**ADR-0117 D7 already has the mechanism.** A person admitted outside their credential's region is
recorded by the admitting side as a **linked user**, federated to their own region's portal instance
(ADR-0119 D4) — consented, minimal, refreshed at sign-in, removed with the last membership. Nothing
new has to be built to let a creator reach the org they made; the existing rule covers it.

## Decision

**1. An org's region is chosen when the org is created.**

- **The personal org:** at signup, unchanged.
- **A team org:** at creation, in the session that creates it.
- In both cases **explicitly**, from the regions that exist, on RFC-0051's terms: labelled by
  regime, ordered by `suggest_for` against the visitor's country, **nothing preselected**, never
  inferred, **refused rather than defaulted** (ADR-0103 D4).

**2. The creator is admitted to the org's region as a linked user.**

- When the creator's credential lives in another region, the org's region holds **its own record of
  the creator** — a linked user federated to the creator's portal instance (ADR-0117 D7, ADR-0119
  D4), plus the membership.
- Minimal, consented at creation, refreshed at each sign-in, removed with the last membership
  (ADR-0117 D7).

**3. The credential never moves.**

- The creator keeps their credential in the region they signed up in (ADR-0117 D6, ADR-0141 R1, R5).
- A global `paridyum.in` user may own an org in another region; ownership is a membership in the
  org's region, not a second credential.

**4. Immutability is unchanged.** An org's region remains immutable except by migration and contract
amendment (ADR-0117 D1, ADR-0041 D3). This decision changes *when* the region is chosen, not how
binding the choice is.

## Amendment to ADR-0117

This amends **ADR-0117 D1**. D1's *"chosen explicitly at signup"* is narrowed to *"chosen explicitly
when the org is created"* — the personal org at signup, a team org at creation. Everything else in D1
stands: the region is chosen from the regions that exist, on RFC-0051's terms, and is immutable
except by migration and contract amendment.

ADR-0117's body is **not rewritten**; it carries a dated note pointing here, which is how a
non-locked artifact is corrected (ADR-0113 D7).

## Consequences

**Accepted costs**

1. **An org created in a region other than its creator's costs a linked-user record and a federation
   link** — the cost ADR-0117 D7 already accepted, now on the ordinary path of a global user creating
   a team org.
2. **The creator's first visit to their new org's region is a cross-region sign-in**, with the extra
   step ADR-0117 cost 2 names.
3. **"At signup" becomes "at creation" in the vocabulary.** Prose and code that read the old phrasing
   for a team org must be re-read; the personal org is unaffected.

**Gained**

- **A global `paridyum.in` user can own an org in any region**, and the org's data stays in the
  region its people chose.
- **Every org has one nameable moment of choice** — creation — rather than a moment only the personal
  org has.
- **No inference from the creator's credential region**, which is the default that would otherwise
  arrive through this gap.

**Rejected alternatives**

- **Keep "at signup" literally.** It leaves a team org with no moment to choose, and invites the
  creator's credential region as a silent default.
- **Place a team org in the creator's credential region.** An inference ADR-0103 D4 and RFC-0015 D5
  refuse, and it puts an org's data where its people are not.
- **Move the creator's credential to the org's region.** Breaks ADR-0117 D6 and ADR-0141 R1: the
  credential never moves.
- **A global identity so the creator is "the same person" everywhere.** Rejected in ADR-0141.

## Related

- **ADR-0117** D1 (amended), D2, D6, D7 — the org region, tenants in any region, one credential in
  one region, linked-user federation
- **ADR-0141** — device-local routing, no global person-to-region index, R1/R5
- **ADR-0119** D4 — cross-plane access is federation, never a shared user store
- **RFC-0015** D5 — how a region is chosen; never inferred · **RFC-0051** — the jurisdiction control
- **ADR-0103** D4 — refused, never defaulted · **ADR-0041** D3 — region contractual and immutable
- **ADR-0113** D7 — a non-locked artifact is corrected by a dated note, not a rewrite
