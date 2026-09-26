---
id: ADR-0101
title: The first wave is six bounded contexts, and deployment and entitlement are authorised
type: adr
status: accepted
created: 2026-09-20
locked_at: null
supersedes: [ADR-0042]
superseded_by: null
scope: [org, platform, service, spec, web]
applies_to: ["*"]
tags: [decomposition, bounded-contexts, deployment, entitlement, release, tenancy, quota, adr-0019, adr-0039, adr-0042, adr-0049, adr-0100, rfc-0009, rfc-0049, bet-0007, anti-0001, anti-0003, anti-0006]
breaking: true
enforced_by:
  - check: domain.entity_has_single_owner
    severity: error
    rationale: >
      Carried forward from ADR-0042 unchanged, and it now has a sixth owner to resolve against.
      `Release` was homeless for the whole period ADR-0042 was in force — the ⚠️ in its own
      consequence list — and an entity with no owner is owned by whoever touched it last, which
      is the state this ADR ends.
  - check: domain.no_service_dependency_cycles
    severity: error
    rationale: >
      Carried forward from ADR-0042 unchanged. The sixth context adds two edges — deployment
      reads tenancy for placement and entitlement for what a tenant may hold — and a wave that
      grows by a context is exactly when a cycle is introduced without anyone choosing to.
---

# ADR-0101 — The first wave is six bounded contexts, and deployment and entitlement are authorised

## Status

**ACCEPTED** as of 2026-09-20 by the operator. **Supersedes ADR-0042**, which was `accepted` and
not `locked`, so supersession was the ordinary path rather than the constrained one.

**`supersedes:` was deliberately `null` while this was `proposed`, and the reason is worth keeping
once.** Declaring it then failed the gate — `ADR-0101 supersedes ADR-0042, but ADR-0042 has
superseded_by 'null'` — because supersession is enforced as an atomic pair. Satisfying that pair
before acceptance would have marked an `accepted` decomposition retired in favour of a `proposed`
one, leaving the estate with **no accepted decomposition at all** for however long review took.
CLAUDE.md states the order that avoids it: *"write a new artifact that supersedes it, **get it
accepted**, set `superseded_by` on the old one."* Every one of the nine existing supersessions in
this repository is `accepted` or `locked`; none was ever declared from `proposed`.

**Acceptance is what closed the pair**, in one edit: `supersedes: [ADR-0042]` here,
`superseded_by: ADR-0101` and `status: superseded` there. ADR-0042 was in force until that moment.

**The operator authorised both contexts on 2026-09-20.** ADR-0019 decision 5 and ADR-0042 decision
7 each require a human before a service exists, and that requirement is met — the authorisation is
recorded in `PLAN-TENANT-FLOW.md` §1 as decisions D1 and D2, taken in session.

**That authorisation and this acceptance were two separate acts, deliberately.** Answering
*"authorise the sixth context"* is not the same as adopting the text that follows, and while this
sat `proposed` an agent moving it would have been deciding by assumption that the two are
equivalent. The operator did both, in that order.

## Context

ADR-0042 fixed the first wave at five contexts on 2026-08-08 and **did not mention
`platform-deployment` at all**, while narrowing `platform-routing` from RFC-0009's *"hostname to
release"* to *"a request to a tenant and a region."* The consequence was recorded in its own
consequence list and in the ROADMAP as an unresolved ⚠️: **`Release`, `Certificate` and the act of
running an artifact were left with no owner.**

That gap has now been load-bearing for six weeks, and the estate demonstrates the cost. Nine Helm
releases have been serving for eighteen days and **nothing in the estate records that they
exist** — not as rows, not as a read model, not anywhere a tenant or an operator could read them.
`PLAN-THREE-SURFACES.md` F0b found four separate records disagreeing with the cluster, and the
reason none of them could be authoritative is that no context owns the fact.

Two things now force the question rather than merely inviting it:

**RFC-0049 re-admits the context and gives it an entity.** A tenant is a row *plus what it runs*,
and `/dashboard` is a view over that set. Without `Release` there is nothing for the tenant console
to list, so the self-serve tenant flow has no middle.

**Self-serve signup needs a quota before it needs a price.** The operator's decision D5 opens
account creation to anyone; D6 caps tenants at three per operator. `platform-entitlement` is
already one of ADR-0042's five and has never been built, because its trigger — *the first plan
distinction that must be enforced rather than described* — had not fired. It fires now, and
BET-0007 is explicit that this is not a billing feature: *"quota enforcement is a **correctness**
feature, not a billing one, and it must exist before the price does."*

## Decision

### 1. The first wave is six contexts, one repository each

| Context | Responsibility — one sentence, no "and" | Owns |
| --- | --- | --- |
| `platform-tenancy` | Maintains the operator → tenant → account hierarchy. | Operator, Tenant, Account |
| `platform-identity` | Proxies and provisions the identity engine on a tenant's behalf. | Zitadel organizations, projects, grants, capability policy |
| `platform-entitlement` | Decides what a tenant is permitted to do. | PlatformPlan, Subscription, Entitlement |
| `platform-routing` | Resolves a request to a tenant and a region. | Route, DomainBinding |
| **`platform-deployment`** | **Runs a product for one tenant.** | **Release** |
| *first catalogue product* | The first vertical application sold. | its own entities |

**The sixth is RFC-0009's original, restored under its original one-liner.** It is not a new idea
admitted late; it is the one ADR-0042 dropped, and RFC-0049 §A is the design.

### 2. `platform-deployment` is authorised, and its boundary is stated by what it does *not* own

| Owns | Does not own |
| --- | --- |
| `Release` — a product instance for a tenant | `Tenant`, `Account` — `platform-tenancy` |
| the deploy / upgrade / rollback lifecycle | `Domain`, `Certificate` — `platform-routing` |
| version and channel per tenant | `Entitlement` — `platform-entitlement` |
| health and last-known state | the *placement* — `platform-tenancy`, via RFC-0045 |

**`Domain` and `Certificate` stay with `platform-routing`.** RFC-0009's matrix already assigns
them, and the ROADMAP's ⚠️ grouped them with `Release` only because all three were homeless — not
because they belong together. Routing binds a hostname to a Release; **deployment never writes a
routing table** (ANTI-0003).

**A Release is `(tenant, product, version, placement, state)`**, plus health and history.

### 3. `platform-entitlement` is authorised, and its first distinction is `max_tenants`

Its trigger has fired. The first enforcement is the one self-serve signup requires: **an operator's
plan carries `max_tenants`, default 3**, and a request to exceed it is a support conversation
rather than a form field.

**The refusal belongs to the saga, not to the surface.** `/dashboard` is a BFF that owns nothing
(ADR-0100 decision 1), and a limit enforced in a BFF is a limit the BFF can be talked out of —
ANTI-0003, and a surface enforcing a rule it does not own. `platform-tenancy`'s provisioning saga
asks entitlement and refuses; the surface renders the refusal.

### 4. `platform-billing` stays deferred, and the reason is restated because ADR-0042's has expired

ADR-0042 decision 3 deferred billing because *"BET-0004 makes per-product pricing tempting and
BET-0002 does not authorise it, so building billing before the pricing model is decided would
encode a guess."* **That reason no longer holds** — BET-0007 decided the shape, flat floor plus
royalty, which is what T-3.12 predicted would happen.

**The deferral survives on its original trigger rather than on its original reason:** *the first
operator who must be charged.* Nobody is charged. The trigger has not fired and the numbers do not
exist — T-0.7 owes both, with reasoning, and T-0.10 owes the payment path with dated citations and
is still ⛔.

**ADR-0042 decision 6 already settled the split this makes possible**, and it is retrieved here
rather than re-reasoned: *"`invoices` is two entities — Paridyum's invoice to an operator
(`platform-billing`) and a tenant's invoice to its accounts (a catalogue product). One word, two
owners, two lifecycles."* Paridyum→operator money lands on the control plane when its trigger
fires; tenant→its-own-customers money is a product feature of whatever that tenant sells, gated on
T-0.10 and bounded by ANTI-0007.

### 5. Carried forward from ADR-0042, unchanged and deliberately

These are not re-decided here. They are restated so that superseding ADR-0042 does not quietly
discard them, which is the failure supersession most often causes.

- **`platform-tenancy` is built first** (D2), because it owns ADR-0039's saga and every other
  context is a dependency of that saga rather than a peer of it.
- **`spec-product` replaces `platform-spec`** (D4) — product contracts as a `spec-` repository.
- **`tool-generate` replaces `platform-generation`, deferred** (D5) on its own trigger.
- **Both name collisions stay resolved by splitting** (D6) — `invoices`, and `member`/`user` as
  OperatorMember versus AccountMember.
- **Creating any of these still escalates to a human** (D7). This ADR passes that gate for two
  contexts by recording an authorisation that was given; it does not pre-authorise the rest.

### 6. `platform-deployment` declares rather than executes — the open question is narrowed, not answered

RFC-0049 open question 1 asks whether the context executes a deploy or declares intent that a
reconciler realises. **This ADR does not answer it**, because the answer decides the repository's
size and nothing yet forces the choice.

**What it does fix is the seam:** whichever way that resolves, `Release` is a row before it is an
action. T-G.8 seeds `Release` from the running cluster — a read model over deployments that already
exist — and that work is identical under both answers. Building it first is what makes the question
answerable against evidence instead of against preference.

## Consequences

**Accepted costs**

1. **Six repositories before one product ships**, up from five. ADR-0042 cost 1 already accepted
   the shape of this and the sixth is one more instance of it: STD-0002's required files, CI, a
   manifest. BET-0009 trigger clause 1 is where the aggregate is measured.

2. **ADR-0042's "Gained" claim is now false and is retired with it.** It read *"six contexts became
   five, and the one removed was removed by a commercial decision rather than by scope-cutting."*
   Five has become six again. The context returning is **not** the one removed — `platform-generation`
   stays deferred as `tool-generate` — so the commercial decision stands; what does not stand is the
   implication that five was a settled number.

3. **`platform-routing` is still the least justified**, carried forward from ADR-0042 cost 2, and
   the sixth context sharpens the question rather than answering it: routing owns `Domain` and
   `Certificate`, deployment owns `Release`, and RFC-0049 open question 6 asks whether that division
   survives contact. It is left open.

4. **Entitlement ships with no revenue signal**, which is ADR-0042 cost 4 arriving. It decides what
   a tenant may do with no knowledge of what they paid — and under decision 3 that is exactly
   right while the answer is the same for everyone, and exactly wrong the day it is not. The day it
   is not is the day billing's trigger fires.

5. **A quota is the only thing standing between a stranger and a provisioned tenant.** D5 opened
   signup; this ADR supplies the cap. Both `STD-0009` and `ADR-0049` are `locked` and **enforced by
   nothing that runs today** (T-2.3), so the isolation this depends on is currently guaranteed by
   compliance. That is a real cost and it is written here rather than in a risk register nobody
   reads.

**Gained**

- **`Release` has an owner**, which is the precondition for the estate recording what it runs. Nine
  releases have been serving for eighteen days with no record; T-G.8 becomes possible on the day
  this is accepted.
- **ADR-0042's ⚠️ is closed** rather than carried into a seventh week.
- **Entitlement's trigger is honoured rather than routed around.** The alternative on offer was a
  cap hardcoded in a BFF, which would have worked and would have put a platform rule in a surface.
- **The tenant flow has a middle.** A tenant console with nothing to list is a screen, not a
  product.

**Rejected alternatives**

- **`Release` lives in `platform-tenancy`.** No new repository, and tenancy already owns the saga
  that would create one. Rejected because tenancy would then own the hierarchy, the provisioning
  lifecycle *and* the deployment lifecycle — ANTI-0006's god context, reached by the most
  reasonable-sounding step available. RFC-0049 §Alternatives rejects it on the same grounds.

- **Defer `Release` and hardcode the tenant console's app list.** Fastest route to a demonstrable
  flow. Rejected because the list would be a lie for as long as it existed, and every task
  downstream of it is written against a picture of the estate that does not exist — which is the
  specific failure `PLAN-THREE-SURFACES.md` F0b documents at four separate records.

- **Authorise deployment, defer entitlement, cap tenants in the surface.** Tempting because it is
  one context instead of two. Rejected by decision 3's reasoning: the cap is the abuse control for
  a public signup route, and an abuse control in a BFF is enforced by whichever surface remembers
  to ask.

## Related

- **ADR-0042** — superseded by this; its five contexts, its ⚠️, and the five decisions carried forward
- **RFC-0009** — the decomposition that first named this context and its entity
- **RFC-0049** — a tenant is a set of Releases; §A is the design this authorises, and its open
  questions 1 and 6 stay open
- **ADR-0019** — decomposition precedes implementation, and D5 is the human gate this passes
- **ADR-0039** — the provisioning saga that asks entitlement and refuses
- **ADR-0100** — `/dashboard` owns nothing, which is why the refusal is the saga's
- **BET-0007** — quota is a correctness feature and precedes the price
- **ANTI-0001, ANTI-0003, ANTI-0006** — the three shapes the boundary in decision 2 defends against
- **T-G.0** — the Genesis task this closes; **T-3.8** — entitlement's trigger; **T-3.11** — the
  question of where `platform-deployment` went, answered here
- **PLAN-TENANT-FLOW.md** — decisions D1, D2 and D6, and the flow this serves
