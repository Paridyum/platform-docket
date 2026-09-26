---
id: ADR-0103
title: Containment is an entitlement attribute, and a rung may be named publicly but never self-served
type: adr
status: accepted
created: 2026-09-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra, web, console]
applies_to: ["*"]
product: [tenancy]
tags: [tenancy, isolation, containment, entitlement, plans, residency, region, byok, adr-0024, adr-0039, adr-0041, adr-0049, rfc-0045, c-0004, c-0006]
breaking: false
enforced_by:
  - check: domain.containment_is_granted_never_inferred
    severity: error
    rationale: >
      A containment class derived from a plan name, a price, or the absence of a flag is escalated
      or downgraded by every rename and every pricing experiment. Placement must be a row someone
      wrote, not a string someone parsed.
  - check: domain.no_self_serve_path_to_a_dedicated_rung
    severity: error
    rationale: >
      ADR-0024 decision 8 gates escalation on a signed contract. A signup flow that provisions a
      dedicated database has moved that gate into a form, where nobody reviews it and nothing
      records who approved it.
  - check: domain.a_named_rung_is_an_operable_rung
    severity: error
    rationale: >
      Naming a rung publicly is a commitment to operate it. A rung named before it is built and
      rehearsed is a promise the first buyer discovers is false, at the worst moment.
  - check: db.region_is_chosen_at_provisioning_and_immutable
    severity: error
    rationale: >
      ADR-0041 decision 3 makes region contractual. A default region silently places a tenant in a
      jurisdiction nobody chose, and an UPDATE moves data across a legal boundary with no migration.
---

# ADR-0103 — Containment is an entitlement attribute, and a rung may be named publicly but never self-served

## Status

**ACCEPTED** as of 2026-09-24 by the operator. Answers **T-G.0e**: the §Relationship-to-ADR-0024
reading is confirmed — naming a rung publicly, with no price and no self-serve path, is not
"publishing a tier" under D8. It cannot reach `locked` until the four declared checks are
implemented (ADR-0002), which puts them in ADR-0014's `declared` state meanwhile — the same state
ADR-0024's own checks sat in when it was locked before any migration existed.

Originally proposed 2026-09-15, awaiting exactly this review.

## Relationship to ADR-0024 decision 8, stated first because it is the reason this ADR exists

ADR-0024 is **locked** and decision 8 reads:

> **Escalation is deferred behind a funded trigger, not published as a tier.** Dedicated database
> and virtual cluster remain the path (RFC-0006 options C and D); neither is built or offered until
> a signed contract pays for it.

**This ADR does not supersede it and must not be read as contradicting it.** The distinction it
draws, and the one on which its legitimacy rests:

- D8 gates **provisioning** on a signed contract. This ADR keeps that gate exactly where it is, and
  adds a C-0004 human approval on top of it (decision 2). Nothing here provisions a dedicated rung
  without a contract.
- D8 forbids **publishing a tier**. What D8 does not decide — because the question had not been
  asked — is whether the *existence and name* of the escalation path may be stated publicly at all.
  This ADR decides that it may, **with no price and no self-serve path**, which is the narrowest
  possible reading consistent with "neither is built or offered until a signed contract pays for
  it": a page that says *talk to us* offers a conversation, not a resource.

**If a reviewer judges that naming a rung publicly is itself "publishing a tier", then this ADR
contradicts a locked artifact and is void until ADR-0024 is superseded under C-0004.** That
judgement belongs to a human and is recorded here rather than assumed away, because the whole
mechanism CLAUDE.md describes depends on an agent not deciding for itself that a lock is narrower
than it reads. The human direction on 2026-09-15 chose "named, no price, contact us" over both a
self-serve tier and silence; this ADR is the conservative encoding of that direction.

## Context

RFC-0045 proposed a containment ladder (C0 shared → C1 dedicated database → C2 dedicated cluster →
C3 dedicated stamp) and left two questions open: whether the ladder is ever published, and how it
relates to commercial plans. Both were put to a human.

The direction given was that **the platform will have plans, that plans are the boundary which
justifies dedicated isolation not existing everywhere, and that the plan names are not yet
decided.** That last part is the load-bearing constraint on this ADR. An artifact that encodes
plan names is invalidated by the first pricing change, and pricing changes more often than tenancy
does — writing "the business plan gets a dedicated database" would make the tenancy record a
downstream dependency of a marketing decision, which inverts the precedence CLAUDE.md sets.

The second direction given was that **region is chosen by every tenant regardless of plan**, per
RFC-0015 decision 5's position that an operator chooses their own region.

The third was that **BYOK is in scope and designed now, built later on an explicit signal.**

## Decision

**1. Containment is an attribute of an entitlement, never a property of a plan name.**

The containment vocabulary (`c0`, `c1`, `c2`, `c3`, per RFC-0045 §Proposal B) is **stable platform
data**. A plan is a commercial object that **grants** a containment class, the way it grants a seat
count or a quota ceiling. Plans may be renamed, repriced, merged or retired without any tenancy
artifact changing.

The prohibition that makes this real: **no code may derive a containment class from a plan name, a
price, a product tier string, or the absence of a flag.** Placement is read from the `placements`
row and from nowhere else. `domain.containment_is_granted_never_inferred`.

This is GLO-0005's rule — *a grant, never an absence* — applied one layer up from the session. The
failure it prevents is specific and has happened to other platforms: a plan is renamed, a string
comparison silently stops matching, and a tenant is placed a rung lower than it pays for, or a rung
higher than it should reach.

**2. A rung's existence and name may be published. A self-serve path to a rung may not exist.**

Concretely permitted: a pricing page, docs, or a capability matrix that names a plan, states that it
includes a dedicated database, and routes to a conversation.

Concretely forbidden: a signup form, a self-service upgrade button, an API call authenticated only
by a tenant's own token, or any other path by which a tenant reaches C1 or above without (a) a
signed contract and (b) a recorded human approval. `domain.no_self_serve_path_to_a_dedicated_rung`.

**No price is shown against a rung above C0.** A published price is an offer; ADR-0024 decision 8's
"neither is built or offered" is what keeps the price off the page.

**3. A named rung is an operable rung.**

Nothing is named publicly before it is built, provisioned end to end at least once, and rehearsed —
including its restore path and its downgrade path. ADR-0024 decision 8's underlying argument was
that *publishing an isolation tier is a commitment to operate it*; that argument survives this ADR
intact and decision 3 is where it is honoured. `domain.a_named_rung_is_an_operable_rung`.

The practical consequence is an ordering, not a prohibition: RFC-0045 slice 9 (a C1 reference
implementation exercised with Paridyum as the tenant, per ADR-0049 D2) **precedes** any public page
that names a dedicated-database plan. We are our own first buyer of every rung we name.

**4. Region is orthogonal to containment and to plan.**

Every tenant chooses a region at provisioning, at every containment class, on every plan. There is
no plan at which region is assigned by us and no plan at which it is withheld. RFC-0015 decision 5.

Region is **refused, never defaulted** — a provisioning request naming no region is rejected, and
`suggest_for` in `region-registry.yaml` orders what an operator is shown and never picks. It is
**immutable after placement**, per ADR-0041 decision 3; a move is a migration and a contract
amendment, enforced by a trigger that refuses `UPDATE`.
`db.region_is_chosen_at_provisioning_and_immutable`.

The cost this accepts is named in §Consequences: every region must be able to host every tenant at
every class, which raises the fixed cost of opening one.

**5. Escalation and downgrade are both migrations, and both are recorded.**

Up: export, provision, restore, re-place, re-attest. Down: the same in reverse. Neither is a
placement `UPDATE`, neither is silent, and both produce a new attestation (RFC-0045 §Proposal E)
because the tenant's isolation statement has materially changed and the old one is now false.

Downgrade is supported rather than blocked. A tenant returning to a shared failure domain is a
thing the attestation will state plainly, which is the control that makes it safe to allow.

**6. Ownership is split and stays split.**

`platform-tenancy` owns placement — the `placements` row, the region, the containment class.
Entitlement owns the grant that authorises a class. Neither reads the other's tables (ANTI-0003);
the saga carries the authorised class into placement at provisioning time, once, as data.
ANTI-0001: one fact, one owner.

**7. BYOK is named at C2 and is not available.**

Per the human direction, customer-managed keys are in scope and are designed before they are built.
RFC-0046 is the design. Until it is built and rehearsed under decision 3, **no public artifact,
capability matrix, README or sales page may list BYOK as available** — ADR-0041 decision 4 forbids
claiming a capability before it is real, and this is the capability most likely to be claimed early
because it is the one enterprises ask for by name.

The permitted statement is that it is planned, with no date.

## Consequences

**Accepted costs**

1. **Publishing a rung name creates an obligation we did not previously have.** Decision 3 means the
   C1 reference implementation, its restore rehearsal and its downgrade path all become blocking
   work for a *marketing* page. That is the correct dependency direction and it will feel backwards
   the first time a page is ready and the rung is not.

2. **Region at every class raises the cost of a region.** Decision 4 means a new region must host
   free-of-charge tenants alongside contracted ones, so its fixed cost cannot be amortised against
   the contracts that motivated opening it. The alternative — region choice as a paid capability —
   was rejected as making residency a commercial lever, which is exactly what ADR-0041 decision 3
   means by calling it contractual rather than an optimisation.

3. **Two classes of tenant, forever, with every check and migration applying to both.** Inherited
   from RFC-0045 §Consequences 1 and restated here because decision 1 makes it permanent rather than
   provisional. The C1 ceiling question (RFC-0045 open question 1) remains unanswered and is now
   more urgent, not less.

4. **A contract-gated rung is slow, and the slowness is visible to the buyer.** Decision 2 means a
   tenant who wants a dedicated database waits for a conversation, a contract and a human. Some will
   go elsewhere. C-0006 anti-goal 5's position is that this is the right customer to lose for now.

5. **This ADR reads as a lawyer's distinction, and that is a real risk.** Decisions 2 and 3 rest on
   a narrow reading of a locked artifact's words. A future reader may reasonably find the reading
   too clever. The remedy is written into §Status rather than left to be discovered: if the reading
   does not hold, the ADR is void and ADR-0024 must be superseded properly.

**What this does not change**

Nothing in ADR-0024's mechanism. Two columns, RLS enabled and forced at **every** containment class,
operator scope as a positive grant, both identifiers from the verified session, a runtime role that
cannot bypass. A dedicated database is defence *added*; it never substitutes for a policy, and a C1
tenant's tables carry `tenant_id` and force RLS exactly as a C0 tenant's do. Nothing in ADR-0038's
session contract changes at any rung.

## Alternatives considered

1. **Encode the plan names directly** (`free` / `solo` / `business` → `c0` / `c0` / `c1`). Rejected:
   plans are undecided, and an artifact naming them would be invalidated by the first pricing
   decision and would make tenancy downstream of marketing.

2. **Supersede ADR-0024 wholesale and publish self-serve tiers.** Rejected for now as a larger
   change than the direction requires: a wholesale supersession must restate decisions 1–7 and 9
   verbatim, and a self-serve path to a dedicated database needs automated provisioning that does
   not exist. Remains the correct move *if* self-serve escalation is ever wanted — §Status names
   the trigger.

3. **Publish nothing; quote everything privately.** The strictest reading of D8, and what RFC-0045
   originally proposed. Rejected by the human direction on 2026-09-15: a prospective tenant cannot
   ask for something whose existence is unstated, and "contact us" is how every platform of this
   shape names a contract-gated capability without offering it.

4. **Make region a paid capability.** Rejected — see consequence 2.

## Related

- **ADR-0024** — the mechanism, and decision 8, which this ADR operates inside rather than replaces
- **ADR-0038** — the session contract, unchanged at every rung
- **ADR-0039** — the saga that carries the authorised class and the chosen region into placement
- **ADR-0041** — decision 3 (region contractual and immutable) and decision 4 (no capability claimed
  before it is real), both load-bearing here
- **ADR-0049** — Paridyum is tenant zero and is the first buyer of every rung, per decision 3
- **RFC-0045** — the ladder, the placement resolver, and the attestation this ADR presumes
- **RFC-0046** — the BYOK design decision 7 defers to
- **RFC-0015** — an operator chooses their own region, which decision 4 generalises to every plan
- **C-0004** — the human approval on every escalation
- **C-0006** — anti-goal 5, the reason a contract-gated rung is acceptable and a self-serve one is not
- **GLO-0005** — a grant, never an absence, applied above the session in decision 1
- **ANTI-0001, ANTI-0003** — the ownership split in decision 6
