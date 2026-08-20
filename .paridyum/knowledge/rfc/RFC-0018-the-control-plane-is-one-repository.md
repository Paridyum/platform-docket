---
id: RFC-0018
title: The control plane is one repository, with a closed list and a split trigger
type: rfc
status: proposed
created: 2026-08-11
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform]
applies_to: ["*"]
tags: [decomposition, control-plane, anti-0006, adr-0042, repository, naming]
breaking: true
enforced_by: []
---

# RFC-0018 — The control plane is one repository

## Status

**PROPOSED** as of 2026-08-11. Feedback deadline **2026-09-10**.

Revises **ADR-0042**, which is `accepted` rather than `locked`, so an RFC is the right instrument.
Argues against **ANTI-0006**, which is `accepted` — binding on judgement, not on CI — and this RFC
is that judgement being exercised in the open rather than quietly.

## The question

ADR-0042 splits the first wave into five contexts, one repository each (ADR-0001). Four of them are
control-plane: `platform-tenancy`, `platform-identity`, `platform-entitlement`, `platform-routing`.
RFC-0014 then established that all four share one database in one region.

So the question is whether four repositories that share a plane, a database, a region and a failure
domain should be four deployables — asked now, while none of them exists and the cost of answering
either way is a directory.

ADR-0042 already names the cost of its own answer as accepted cost 1: *"Five repositories before one
product ships. Four of them are platform plumbing a customer never sees, and each carries the full
weight of STD-0002's required files, CI, and a manifest."*

## Decision 1 — one repository, `platform-control`

**The four control-plane contexts become one repository and one deployable.** The regional data
plane is untouched, and so is `platform-identity`'s regional half — Zitadel is a vendor deployment
per region (ADR-0047), not code in this repository.

**It passes ANTI-0006's own test**, which is the reason this is a judgement call rather than a
violation. That test is: *"A service exists to own one business capability. If you cannot name it in
one sentence without 'and', it is more than one service."*

> `platform-control` owns every fact that is true globally rather than within one region.

No "and". Contrast the shape ANTI-0006 actually describes — *users, auth, audit log, invoices,
payments, reporting, notifications, file uploads, the cron jobs* — which has no such sentence, and
that absence is the anti-pattern. **The boundary here is the plane**, which RFC-0014 defines
independently of this decision and which a table already has to declare.

**The name is not `core`.** ANTI-0006 opens with *"`svc-core`. Or `svc-platform`, `svc-api`,
`svc-backend`, `svc-main`"* — those names describe where things go when they belong nowhere.
`control` names a plane that exists in the architecture whether or not this repository does. C-0007
requires `<class-prefix><name>` against `class-registry.yaml`, so `platform-control` it is, with
crate `paridyum-control` under C-0007's derivation rule.

## Decision 2 — the closed list is what keeps it honest

ANTI-0006's mechanism is accretion: *"a feature arrives that does not obviously belong anywhere, and
this is where things go that do not obviously belong anywhere. Repeat for eighteen months."*

**So the contents are a closed list, and adding to it takes an RFC.**

| Owns | Never owns |
| --- | --- |
| Operator identifiers, and which tenants each owns | Operator personal data — that is regional (RFC-0017) |
| Subscriptions and entitlements | Any tenant's business data, ever |
| The region registry | Anything a tenant's request path reads on every call |
| Domain → region bindings | Zitadel — a vendor deployment, not our code |
| Staff identity, in the control region | Billing, which is deferred (ADR-0042 decision 3) |

The right column is the load-bearing half. A closed list with a written procedure for extending it
is the difference between a bounded context and a drawer.

**The one-sentence test is the admission rule.** If a proposed addition cannot be described as a
fact that is true globally rather than within one region, it does not go here — and per ANTI-0006,
that difficulty *"is information — the most useful signal in the design."*

## Decision 3 — the trigger to split, written now

Deferring a split without naming its trigger is how a temporary decision becomes permanent. Same
shape as ADR-0024 decision 8's deferred isolation tiers.

**Split when any one of these fires:**

1. **Two contexts inside it need different availability targets.** ADR-0042's own reason for keeping
   identity separate: *"their availability profiles differ by an order of magnitude: login must be
   near-perfect, provisioning may be down for an hour."* That argument is dormant only because
   Zitadel serves login from the regional plane today. If any control-plane path becomes
   latency-critical, it wants its own deployable.
2. **A second engineer owns one of the contexts.** ANTI-0006's ownership argument — *"nine
   responsibilities means the real owners are nine teams, so review has no expert"* — is void at one
   engineer and true at three.
3. **Release cadence diverges.** One context needing a deploy that another must not receive is the
   coupling made visible.
4. **The closed list is extended twice.** Not once — twice. One addition is a boundary being refined;
   two is a drawer forming.

Any of these fires, the split is back to ADR-0042's decomposition, which is why this RFC revises
that ADR rather than replacing it. **The contexts remain real as modules** (STD-0004), so the split
is a repository operation and not a redesign.

## Consequences

**Accepted costs**

1. **The blast radius is the whole control plane.** A bad deploy takes operators, entitlements,
   regions and routing together. RFC-0015 decision 1 already makes the control plane one failure
   domain by design, so this changes the probability of an outage rather than its scope — but it
   does raise that probability, and saying otherwise would be false.

2. **One resource profile for four workloads.** ANTI-0006's scaling argument. Weak here because all
   four are low-volume metadata operations off the request path, and it stops being weak the moment
   any of them isn't.

3. **The boundary now lives in review rather than in a repository wall.** Four repositories enforce
   their boundaries by making a violation impossible to write. One repository enforces them with
   STD-0004 module discipline and a closed list, both of which a determined afternoon can erode.

4. **ADR-0042 needs revising, not just extending.** Its decision 1 table and its accepted cost 1
   both change. `platform-tenancy` being built first (decision 2) survives as *the first module*.

5. **Reversing costs more than never doing it.** Splitting four modules out of one repository later
   means four sets of CI, manifests and charts created under time pressure, rather than four created
   calmly now.

**Gained**

- One repository, one CI pipeline, one chart, one manifest, one deploy — before a product exists.
  ADR-0042's accepted cost 1 goes from four repositories of plumbing to one.
- The plane boundary, which RFC-0014 makes a checkable property of every table, becomes the same
  boundary as the deployable. Fewer boundaries that must agree.
- The split trigger is written while the decision is fresh, which is the only time it gets written
  honestly.

## Alternatives considered

- **Keep ADR-0042's four repositories.** The status quo, and correct at the point four people are
  working. Rejected now on ADR-0042's own accepted cost 1, and reversible on decision 3's triggers.

- **Two repositories — tenancy plus identity, and everything else.** Splits on ADR-0042's
  availability argument specifically. Rejected because the argument is dormant: Zitadel serves login
  from the regional plane (ADR-0047 decision 4), so no control-plane path is latency-critical today.
  Trigger 1 is exactly this alternative, held until it is true.

- **Name it `platform-core`.** Rejected on ANTI-0006's opening line. The name matters more than it
  looks: `core` invites the accretion decision 2 exists to prevent, and a name that describes no
  boundary is how the boundary stops being defended.

## Open questions

1. **Does `platform-identity` split across the repository boundary?** RFC-0017 puts operator personal
   data in the operator's region while staff stay in the control region, so identity now has a
   control-plane half and a regional half. Whether the regional half is a module here or belongs to
   the data plane is unresolved, and RFC-0012's successor owns it.

2. **What is the module boundary inside the repository?** STD-0004 organises by responsibility rather
   than size, and the four contexts are the obvious modules — but ADR-0042's decomposition was made
   for repositories, and a boundary that was right for a repository is not automatically right for a
   module.

3. **Does the closed list belong in `manifest.yaml`?** A list enforced by review is weaker than a
   list enforced by a check, and ADR-0022's pattern is to move exactly this kind of list into
   registry data. Worth doing if the list is ever extended once.

## Reversal conditions

- **Any of decision 3's four triggers.** The split returns to ADR-0042's decomposition, and the
  modules are already shaped for it.
- **Reverses immediately if the closed list cannot be held.** If the first three features to arrive
  all need an RFC to place, the boundary was wrong and this RFC was the mistake.

## Related

- ADR-0042 — the five-context decomposition this revises, and its accepted cost 1
- ANTI-0006 — the anti-pattern this argues against, on its own one-sentence test
- ADR-0001 — one context per repository, which is what makes this a decision rather than a detail
- ADR-0019 — decomposition precedes implementation, and requires a human between the two
- RFC-0014 — the plane boundary this makes the deployable boundary
- RFC-0015 — the control plane as one failure domain, which is why cost 1 is about probability
- RFC-0017 — the operator split that creates open question 1
- C-0007 — `<class-prefix><name>`, and the crate-name derivation
- STD-0004 — responsibility never size, which is what keeps the modules real
- ADR-0024 — decision 8's deferred-behind-a-trigger shape, reused by decision 3

