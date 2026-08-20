---
id: RFC-0009
title: Decomposing the business runtime into bounded contexts
type: rfc
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
resolved_by: ADR-0042
scope: [org, platform, service, spec, web]
applies_to: ["*"]
tags: [decomposition, bounded-contexts, runtime, architecture, ownership]
breaking: true
enforced_by: []
---

# RFC-0009 — Decomposing the business runtime into bounded contexts

## Status

**PROPOSED** as of 2026-08-07. Stage 2 of ADR-0013, produced by the `decompose-spec` procedure
ADR-0019 decision 4 mandates.

**Nothing here is built, and one deliberate deviation from the procedure:** step 7 says write
one `REQ-` per context and *then* the RFC. That order is inverted here, because the procedure's
own opening paragraph says the failure it exists to prevent is *"a plausible implementation of
the wrong decomposition, produced faster than anyone can review it, after which the code has
momentum."* Eleven `REQ-` files are artifacts with momentum. The contexts are the decision; the
`REQ-` set follows acceptance.

## 0. What the knowledge base already decides

Most of the substance is settled and must not be rebuilt.

| Already decided | Where |
| --- | --- |
| Two nested isolation silos, `tenant_id` + `account_id`, RLS forced | ADR-0024 |
| The vocabulary: operator, tenant, account, nested tenancy, operator scope | GLO-0001 … GLO-0005 |
| What is ours to build versus commodity | C-0006 |
| The application layer is generated against a contract | BET-0003 |
| Quota enforcement is a correctness feature, not a billing one | BET-0002 cost 4 |
| Export exists at both levels | REQ-0013 |
| One bounded context is one repository | ADR-0001 |
| Six forbidden shapes | ANTI-0001 … ANTI-0006 |
| Migration declaration syntax | STD-0006 |

C-0006's not-commodity list is effectively the requirement source, and this RFC is largely the
act of turning its four bullet points into boundaries.

## 1. Atomic requirements

Technology-neutral, one sentence each, split at every "and". Numbered for citation.

**Identity and access**
1. A person authenticates and receives a session naming the scope they hold.
2. A person acting for an operator can never obtain a session scoped to another operator.
3. A person acting for an account can never obtain operator scope (GLO-0005).
4. Operator scope is a positive claim in the session, never the absence of a claim.

**Tenancy**
5. An operator owns one or more tenants.
6. A tenant contains zero or more accounts.
7. A tenant is reachable at a hostname its operator chooses.
8. A tenant can be suspended without being deleted.

**Entitlement and consumption**
9. What a tenant is permitted to do is derived from its operator's plan.
10. Consumption is recorded per tenant and per account.
11. Consumption beyond a permitted limit is refused, not billed.
12. An operator can see consumption before it becomes a bill.

**The application layer**
13. A business is described by a machine-readable specification.
14. A specification produces a deployable artifact without a human writing code.
15. A specification is versioned, and a previous version can be restored.
16. An artifact runs and serves traffic for exactly one tenant.
17. Generated output is regenerated, never hand-edited (C-0004).

**Record and exit**
18. Every state change attributable to a principal is recorded immutably.
19. An operator can export everything a tenant owns.
20. An account can export everything it owns, and nothing another account owns.

**Not requirements, deliberately.** Nothing here says how money reaches a tenant from its own
accounts. That is the tenant's business, built in its generated application on commodity
payment processing (C-0006), and treating it as a platform requirement would be the first step
toward becoming a payment processor.

## 2. Entities, and the two names that must be split

Every noun with identity and a lifecycle. The procedure's step-2 question — *are two of these
actually the same thing?* — does most of the work here, and it finds two collisions.

| Entity | Created by | Changed by | Notes |
| --- | --- | --- | --- |
| Operator | signup | itself | GLO-0001. Pays us. |
| Tenant | an operator | its operator | GLO-0002. Outer silo. |
| Account | a tenant | its tenant | GLO-0003. Inner silo. |
| **OperatorMember** | an operator | its operator | a person in Paridyum's console |
| **AccountMember** | an account | its account | a person in a *tenant's* application |
| PlatformPlan | Paridyum | Paridyum | what an operator buys **from us** |
| Subscription | an operator | billing | an operator's active PlatformPlan |
| Entitlement | derived | entitlement | what a tenant may do |
| UsageRecord | the runtime | append-only | high volume |
| QuotaCounter | derived | metering | current consumption against a limit |
| AppSpec | an operator | its operator | BET-0003's contract |
| Artifact | generation | immutable | output of one AppSpec version |
| Release | deployment | deployment | an Artifact serving a Tenant |
| Domain | an operator | routing | hostname → Release |
| AuditEvent | everything | immutable | never updated, never deleted |
| ExportJob | an operator or an account | itself | REQ-0013, two readings |

**Collision 1 — `member`/`user`.** An OperatorMember and an AccountMember are **not one entity
with two roles.** They are two populations in two trust domains: one is our customer's staff,
the other is our customer's customer's staff, and requirement 3 says a bug must never let the
second become the first. Collapsing them into a `users` table with a `type` column is how the
most dangerous privilege escalation in this system gets built, and it is exactly the
`User`/`Customer`/`Employee`/`Vendor` mistake the procedure warns about. **They share no
credential store and no token audience.**

**Collision 2 — `plans` and `invoices`.** ADR-0024's own example has `plans` as *tenant-scoped*
— a tenant offering plans to its accounts. But a PlatformPlan is what an operator buys from
Paridyum, and that is platform-owned. Two different entities sharing a name, which step 4 says
must be split rather than reconciled. Same for `invoices`: ours to an operator, versus a
tenant's to its accounts. **Renamed here: `PlatformPlan` versus a tenant's own `plans`, which
is generated application data and not a platform entity at all.**

## 3. Candidate bounded contexts

Eleven. Each responsibility sentence contains no "and" — where a draft needed one, it became
two contexts, which is how `deployment` and `routing` separated.

| Context | Responsibility (one sentence, no "and") | Owns |
| --- | --- | --- |
| `platform-identity` | Proxies and provisions the identity engine on a tenant's behalf. | Zitadel instances, organizations, projects, grants, capability policy |
| `platform-tenancy` | Maintains the operator → tenant → account hierarchy. | Operator, Tenant, Account |
| `platform-entitlement` | Decides what a tenant is permitted to do. | PlatformPlan, Subscription, Entitlement |
| `platform-metering` | Records consumption against a permitted limit. | UsageRecord, QuotaCounter |
| `platform-billing` | Collects money from an operator. | Invoice, PaymentMethod |
| `platform-spec` | Stores and versions a business specification. | AppSpec, SpecVersion |
| `platform-generation` | Produces a deployable artifact from a specification. | GenerationRun, Artifact |
| `platform-deployment` | Runs an artifact for one tenant. | Release |
| `platform-routing` | Resolves a hostname to a release. | Domain, Certificate, RouteBinding |
| `platform-audit` | Records what happened, immutably. | AuditEvent |
| `platform-export` | Produces a complete copy of what a principal owns. | ExportJob |

**Eleven contexts is eleven repositories under ADR-0001, and that is not a starting point for a
company with no code.** The procedure says to treat this step as deciding how many repositories
exist, so it is treated as such: see §7, which sequences them rather than merging them.

## 4. Ownership matrix

Exactly one owner per entity. Two owners is ANTI-0001 and is the one boundary mistake that is
both mechanically detectable and permanent once shipped.

| Entity | Owner | Everyone else |
| --- | --- | --- |
| Operator, Tenant, Account | `platform-tenancy` | reads via API; reacts to events |
| OperatorMember, AccountMember, session | `platform-identity` | validates a session, never reads the table |
| PlatformPlan, Subscription, Entitlement | `platform-entitlement` | asks "may this tenant do X" |
| UsageRecord, QuotaCounter | `platform-metering` | publishes consumption; reads its own counters |
| Invoice, PaymentMethod | `platform-billing` | consumes Subscription events |
| AppSpec, SpecVersion | `platform-spec` | generation consumes a published version |
| Artifact | `platform-generation` | deployment consumes it |
| Release | `platform-deployment` | routing binds to it |
| Domain, Certificate | `platform-routing` | tenancy publishes intent, never writes |
| AuditEvent | `platform-audit` | everyone publishes, nobody updates |
| ExportJob | `platform-export` | reads every context through its public contract |

**Three near-conflicts, resolved rather than left implicit:**

- **Quota.** Metering owns the *counter*; entitlement owns the *limit*. Different entities,
  different write rates, and merging them would put a high-volume append path inside a
  low-volume policy service.
- **Domain.** Tenancy knows a tenant *wants* a hostname; routing owns the hostname. Tenancy
  publishes intent and never writes routing's tables (ANTI-0003).
- **Export.** `platform-export` reads across every context, which looks like ANTI-0003. It is
  not, provided it reads through public contracts. If it ever reads another context's tables
  directly it becomes the god service ANTI-0006 describes, and the export requirement is exactly
  the pressure that would cause it.

## 5. Events, not calls

The procedure's test: *does the caller need an answer to finish its own job?* Most do not.

```
tenancy    ──TenantCreated──▶  routing (provisions), entitlement (initialises), audit
spec       ──SpecPublished──▶  generation
generation ──ArtifactBuilt──▶  deployment
deployment ──ReleaseLive────▶  routing (binds), audit
metering   ──QuotaExceeded──▶  entitlement (suspends), audit
billing    ──PaymentFailed──▶  entitlement (degrades), audit
```

**Synchronous, because an answer is genuinely needed:** session validation on a request path
(identity), and the entitlement check on a request path — the latter cached, because a policy
lookup per request is a synchronous dependency on the hot path and BET-0002's flat pricing
makes latency our cost rather than the customer's.

## 6. The graph against the ANTI- set

- **ANTI-0002, no synchronous cycles.** Identity is called and calls nothing. Tenancy publishes
  and is read. The only synchronous edges are inbound to identity and entitlement. No cycle.
- **ANTI-0001, no shared writes.** §4 is the matrix, written rather than implicit.
- **ANTI-0003, no foreign table reads.** The pressure point is `platform-export`; flagged above.
- **ANTI-0005, compensating paths.** Creating a tenant spans tenancy, routing and deployment. A
  half-created tenant — hierarchy row present, hostname unprovisioned — needs a **named**
  compensating path, and this decomposition does not yet have one. **Open question 3.**
- **ANTI-0006, no "and" in a responsibility.** Enforced during §3; it is what split routing out
  of deployment.
- **ANTI-0004, no domain rule in a shared library.** The real tension in this decomposition, and
  it deserves its own answer.

### ANTI-0004 and the tenancy primitive

Every context must scope its queries per ADR-0024. Putting that in a shared library looks like
ANTI-0004's forbidden shape, and ANTI-0004's own test — *if the business changed its mind, would
this file need editing?* — appears to say yes.

**The resolution is that two different things were being conflated.**

- **The mechanism** — connect as the non-owning role, set the session tenant and account
  variables, refuse to build a query without them. This has no domain opinion. It is ANTI-0004's
  own list of legitimate shared code: *"a tracing subscriber setup, an error type, a retry
  wrapper."* It belongs in `lib-tenancy`.
- **The policy** — which tables are platform, tenant or account scoped, and what operator scope
  means for each. This is a domain decision, it lives in each context's own migrations, and
  STD-0006 plus `check-migrations.sh` enforce it there.

A shared library that *enforces the mechanism is present* while owning none of the policy is the
shape that satisfies both ADR-0024 and ANTI-0004. If `lib-tenancy` ever grows a table name, it
has crossed the line.

## 7. Sequencing, because eleven repositories is not a starting point

The decomposition names eleven contexts. It does not follow that eleven exist now — a context
not yet built needs no repository, and `status: deferred` with a trigger is what ADR-0018
provides for exactly this.

**First, for one running business (requirements 1–8, 13–17):**
`platform-identity`, `platform-tenancy`, `platform-spec`, `platform-generation`,
`platform-deployment`, `platform-routing`.

**Deferred, with triggers:**

| Context | Trigger |
| --- | --- |
| `platform-entitlement` | the first plan distinction that must be enforced rather than described |
| `platform-metering` | the first resource whose overuse costs us money |
| `platform-billing` | the first operator who must be charged |
| `platform-audit` | the first state change a customer could dispute — starts as a table, becomes a context when a second writer appears |
| `platform-export` | REQ-0013's trigger: the first tenant-authored row |

Six repositories is still a lot for a company with no code, and that is the honest cost of a
correct decomposition rather than a reason to merge two of them. **Merging contexts to reduce
repository count is how ANTI-0006 gets built**, and it is the one mistake here that is
effectively permanent.

## 8. Alternatives considered

**One `platform-core` service, split later.** Fastest to a running product and the strongest
temptation. Rejected on ANTI-0006 and on C-0001 principle 1 — but the honest reason it is
rejected is narrower: the boundary that would be hardest to extract later is
identity-versus-tenancy, and that is precisely the boundary requirement 3 makes a security
property. A god service containing both is one bug away from an account member holding operator
scope.

**Merge identity into tenancy.** They are adjacent and both concern who exists. Rejected because
identity owns two populations that must never mix, while tenancy owns the hierarchy those
populations act *within*; one context owning both would make the trust-domain separation a
convention inside a service rather than a boundary between two.

**Split identity into two contexts, one per population.** Genuinely defensible and the strongest
alternative — it would make the trust boundary a network boundary. Rejected because the
mechanism is identical, so two contexts means the same domain rule in two places, and version
skew between them would mean *two answers to who may hold operator scope*. That is ANTI-0004's
"version skew becomes behaviour skew" arriving on the most dangerous possible question.
**Recorded as open question 1, because it is close and the cost of being wrong is asymmetric.**

**Generation inside deployment.** They are sequential, so one context looks natural. Rejected on
the "and" test: *produces an artifact* and *runs an artifact* are different responsibilities with
different failure modes and wildly different resource profiles.

**A context per generated application.** Rejected: an application is data, not a bounded
context. This is the reading of BET-0003 that would recreate the template catalogue as a
repository sprawl.

## Revision — 2026-08-08

**Q1 is answered by RFC-0012**, and answering it changed the premise rather than picking one of
the two options offered. Identity is not infrastructure this platform *has*; it is a product it
*sells* — headless, per-tenant branded, built on self-hosted Zitadel. The §3 responsibility
sentence for `platform-identity` above has been corrected accordingly: it does not issue
sessions, because Zitadel does.

The original question — one identity context or two — is resolved as **one**, but for a
different reason than §8 gave. The two populations are separated by *Zitadel instance* rather
than by service boundary, which yields distinct issuers and distinct user stores without
operating two identity services.

**Q3 is answered by ADR-0039** — an asynchronous compensating saga owned by `platform-tenancy`,
which satisfies the ANTI-0005 gap §6 records as unsatisfied.

Q2, Q4 and Q5 remain open. Q4 in particular is narrowed rather than closed: an AccountMember
authenticates against their tenant's Zitadel **organization**, so the seam is between a product
and a shared per-region issuer.

### Revision 2 — 2026-08-08 — `platform-spec` and `platform-generation` re-justified

**BET-0003 was withdrawn (BET-0004), and it was the entire justification for these two contexts.**
Under a generated-application-as-product wager they were the product. Under a catalogue they are
not. Both survive, both change role, and both change class — and one leaves the first wave.

**`platform-spec` → a `spec-` repository.** Its subject was *"stores and versions a business
specification"* — a customer's description of their business. That artifact no longer exists,
because customers pick products rather than describe them.

What does exist, and now has no home: **the contract a catalogue product declares itself in.**
RFC-0012 requires every product to declare its Zitadel Project, its role vocabulary, its audience
and its tenancy scope, and ADR-0039's saga reads that declaration to grant correctly. Today that
is prose. It should be a versioned, machine-readable artifact — which is exactly what
`class: spec` is for: *"interface contracts… no executable source."* So the context survives as
**product contracts rather than business specifications**, and moves from `platform-` to `spec-`.

**`platform-generation` → a `tool-` repository, and it leaves the first wave.** BET-0004 claims
*"product number ten costs far less than product number one."* That claim needs a mechanism, and
generation is the only one proposed. What changes is who it serves: it was the customer-facing
product; it is now **the factory Paridyum builds its own catalogue with** — build-time, internal,
never in a request path. `class: tool` is `deployable: false`, so ADR-0020 stops binding it and
the Kubernetes obligations disappear.

**And it is no longer first-wave.** Under BET-0003 nothing shipped without it. Under BET-0004
products one and two are hand-built; a factory earns its place around product four or five, once
the repetition is visible and its shape is known. Building it first would mean designing a
generator against a catalogue that does not exist — the same inference-before-evidence failure
ADR-0019 exists to prevent.

**Deferred, with a trigger:** *the third catalogue product is scoped, or two shipped products are
found to share a structure that was copied rather than generated.*

**Net effect on the first wave: six contexts become five.** `platform-identity`,
`platform-tenancy`, `platform-entitlement`, `platform-routing` and a first product.

## Open questions

1. **One identity context or two?** §8 recommends one, with two populations sharing no
   credential store and no token audience. The alternative makes the trust boundary a network
   boundary at the cost of duplicating the rule. Asymmetric downside; worth a second opinion.
2. **Does `platform-audit` start as a context or a table?** §7 says table. A table that every
   context writes to is ANTI-0001 the moment there are two writers, so the trigger has to fire
   early — possibly immediately, which would make the answer "context".
3. **What is the compensating path for a half-created tenant?** ANTI-0005 requires it named and
   this RFC does not name it. It must be settled before `platform-tenancy` is built.
4. **Where does an AccountMember's authentication live in the generated application?** Identity
   owns the entity, but the login *page* is part of the tenant's app and therefore generated. The
   seam between a platform-owned identity and a generated interface to it is unspecified.
5. **Is `lib-tenancy` one library or one per stack?** Rust services need it; a generated Next.js
   application needs the same guarantee in TypeScript. `class-registry.yaml` permits `library` on
   both stacks, so this is allowed — but two implementations of one mechanism is the skew risk
   from §8 in a different place.

## Report

```
Found        20 requirements, 16 entities, 2 name collisions requiring a split
Contexts     11 candidates, one repository each under ADR-0001
             6 for a first running business, 5 deferred with triggers
Already      ADR-0024 tenancy · GLO-0001..5 vocabulary · C-0006 scope · BET-0003 generation
             STD-0006 migrations · ANTI-0001..6 boundaries — none rebuilt here
Conflicts    ANTI-0005 unsatisfied: no compensating path for a half-created tenant (Q3)
             ANTI-0003 pressure at platform-export; ANTI-0004 resolved via mechanism/policy split
Unknowns     5 open questions, of which Q1 and Q3 block the first repository
```

**Stopping here.** The decomposition is the decision and ADR-0013 requires a human between it
and any code. Q1 and Q3 must be answered before `platform-tenancy` or `platform-identity` is
created — and creating either escalates to a human anyway, per ADR-0019 decision 5.

## Related

- ADR-0019 — decomposition precedes implementation; the procedure this followed
- ADR-0024, GLO-0001…GLO-0005 — the tenancy model and vocabulary this decomposes around
- C-0006 — the not-commodity list that is effectively this RFC's requirement source
- BET-0003 — the generated application layer §3's spec and generation contexts serve
- ANTI-0001 … ANTI-0006 — checked in §6
- STD-0006 — the migration conventions each context's own migrations must follow
- ADR-0001 — one context, one repository, which is what makes §7 a real cost
- ADR-0018 — deferred with a trigger, which is how §7 defers five contexts
