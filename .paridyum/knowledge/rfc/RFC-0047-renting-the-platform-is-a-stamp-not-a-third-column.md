---
id: RFC-0047
title: Renting the platform is a stamp, not a third column
type: rfc
status: proposed
created: 2026-09-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra, web, console]
applies_to: ["*"]
product: [tenancy]
tags: [white-label, reseller, partner, franchise, nested-tenancy, third-level, stamp, control-plane, royalty, adr-0024, adr-0039, adr-0049, adr-0100, adr-0103, rfc-0045, bet-0004, bet-0005, bet-0007, c-0006, glo-0004]
breaking: false
sources:
  - url: https://learn.microsoft.com/en-us/azure/architecture/guide/multitenant/approaches/governance-compliance
    title: "Architectural Approaches for Governance and Compliance in Multitenant Solutions — Azure Architecture Center"
    retrieved: 2026-09-15
  - url: https://learn.microsoft.com/en-us/azure/architecture/guide/multitenant/approaches/overview
    title: "Architectural Approaches for Multitenancy — Deployment Stamps pattern — Azure Architecture Center"
    retrieved: 2026-09-15
  - url: https://www.flosum.com/blog/salesforce-hyperforce-data-residency-compliance
    title: "What Is Salesforce Hyperforce? Architecture, Migration & Data Residency — Flosum"
    retrieved: 2026-09-24
  - url: https://support.auth0.com/center/s/article/Tenant-Creation-in-a-Specific-Sub-region
    title: "Tenant Creation in a Specific Sub-Region — Auth0 Support Center"
    retrieved: 2026-09-24
  - url: https://oneuptime.com/blog/post/2026-09-18-choose-regional-saas-storage-tenancy-model/view
    title: "Data Residency Storage: Shared, Schema-per-Tenant, or Database-per-Region — OneUptime"
    retrieved: 2026-09-24
  - url: https://www.cockroachlabs.com/blog/multi-region-database-architecture-sql-placement-locality/
    title: "Multi-Region Database Architecture: SQL, Placement & Query Optimization — Cockroach Labs"
    retrieved: 2026-09-24
  - url: https://www.clickittech.com/software-development/multi-tenant-architecture/
    title: "Designing Multi-tenant SaaS Architecture on AWS: The Complete Guide for 2026 — ClickIT"
    retrieved: 2026-09-24
---

# RFC-0047 — Renting the platform is a stamp, not a third column

> ## ⚠ TRIGGER NOT FIRED — DESIGN ONLY, NOT BUILDABLE
>
> **ADR-0024 decision 9 is locked and reads:**
>
> > A third level is deferred behind a named trigger. […] **Trigger: a real third level with a
> > paying customer behind it** — *a reseller or white-label partner above the operator*, or
> > sub-accounts within an account. **Not "we can imagine one."**
>
> This RFC describes precisely the party D9 names. **No paying customer exists.** The trigger has
> **not** fired, and this document is therefore a design held against the day it does — it must not
> be implemented, sliced, or cited as a commitment. Writing it is legal; building from it is not.
>
> Two further gates stand in front of it, both independent of D9:
> **C-0006 anti-goal 1** — *we will not franchise a vertical we have not operated* — and
> **BET-0005 trigger clause 1** — Paridyum's own tenant reaching one hundred paying end customers
> on its first catalogue product within nine months of that product going live. Neither is met.
>
> **Difficulty: the hardest thing in the estate's design record.** Not because any single piece is
> novel, but because it recurses the platform through itself and every recursion multiplies an
> existing unsolved problem. §Why this is hard is not throat-clearing; it is the substance.

## Problem

A tenant has asked — in the abstract, as an intention rather than a contract — to **rent the whole
business**: take Paridyum's platform, brand it as their own, and use it to sign and serve tenants
of their own. Their customers would never learn Paridyum exists. In the operator's own words, a
tenant would *"deploy its mother."*

Today this is impossible, and the impossibility is structural rather than incidental:

1. **There is exactly one control plane and it is not a product.** ADR-0100 decision 4 is explicit:
   *"the provisioner is not provisioned by a tenant."* `/dashboard` is deployed by the estate's normal
   pipeline as tenant zero's control plane, and is the one thing in the estate the ADR-0039 saga
   does not produce. A second control plane has no mechanism to come into existence.

2. **The tenancy tree has no room for the party.** GLO-0004's tree is
   `Paridyum → operator → tenant → account`, and ADR-0024 implements the bottom two as columns. A
   white-label partner sits **above** the tenant and **below** Paridyum — a position that exists in
   the glossary's prose and in no schema.

3. **Nothing in the revenue model addresses the level.** BET-0007's table covers a franchised
   tenant, an account inside Paridyum's own tenant, and Paridyum's own tenant. A partner earning
   from *its* tenants, on whose GMV we would take a share, is a fourth row nobody has written.

4. **The information asymmetry RFC-0022 exists to bound acquires a second layer.** RFC-0022's
   entire defence is that nothing tenant-derived informs where Paridyum competes. A partner is
   simultaneously our customer, our competitor's supplier, and the party whose tenants' data we
   would now be one hop further from — and RFC-0022 was written assuming exactly three levels.

**Who experiences it.** Nobody, today. That is the honest answer and it is why this document is
gated rather than scheduled.

**What happens if nothing is done.** The opportunity is refused when it arrives, or — far worse —
accepted and improvised, which is how a platform acquires a second control plane nobody designed
and a migration path that only one person understands.

**How we will know it is fixed.** A partner can run the platform under their own brand, sign their
own tenants, and be entirely unable to reach Paridyum's data or any other partner's — while
Paridyum can still bill them, upgrade them, and prove to each of their tenants that the isolation
they were promised is real.

## Constraints

- **ADR-0024 (locked) decision 9** — the third level is deferred behind a named trigger which this
  party *is*, and which has not fired. Also: *"Two explicit columns can be migrated to a path; a
  path cannot easily be migrated back."* This is the sharpest technical constraint in the document.
- **ADR-0024 (locked) decisions 1–7** — unchanged and binding at every level of any design here.
- **ADR-0100 (accepted) decision 4** — the bootstrap asymmetry: the provisioner is not provisioned
  by a tenant. §Proposal breaks this deliberately and says so.
- **ADR-0103 (proposed)** — containment as an entitlement attribute; C3 (dedicated stamp) is the
  rung this RFC turns out to need, and decision 3's *a named rung is an operable rung* governs when
  any of this may be spoken about publicly.
- **RFC-0045 (proposed)** — the ladder, the placement resolver, the attestation. This RFC is
  substantially an application of it, which is the main reason it is tractable at all.
- **ADR-0049 (locked)** — nothing runs outside a tenant, and Paridyum holds no scope a customer
  could not hold. A partner's control plane is bound by this exactly as ours is.
- **ADR-0039 (accepted)** — the provisioning saga, which would have to become recursive.
- **ADR-0041 (accepted)** — decision 3's *"the Zitadel masterkey is immutable once initialised, per
  region — a compliance artifact"* becomes a per-partner compliance artifact.
- **ADR-0047** — identity is Zitadel with one organization per tenant. A partner needs an
  *instance*, not an organization, which is a different object with different operations.
- **BET-0004** — the catalogue is the product. §Proposal's central move depends on this.
- **BET-0005** — Paridyum operates what it franchises; *"tenants run under their own brand"* is
  already the model, and *"the closer analogy is a private-label manufacturer that also sells its
  own label."* White-label is not new here; white-labelling **the platform** is.
- **BET-0007** — price is a floor plus a royalty, and the royalty follows the level in the tree.
- **RFC-0022** — the operator competes with its tenants; the informational conflict and the bound
  accepted against it.
- **C-0006** — anti-goal 1 (*we will not franchise a vertical we have not operated*), anti-goal 5
  (no enterprise sales before self-serve works), non-commodity item 2 (*provable isolation at two
  levels*, extended 2026-08-15 to isolation **from the operator**).
- **REQ-0008** — migrations deploy without downtime. Returns here as the hardest constraint, for
  the third time in the tenancy record.
- **GLO-0001, GLO-0002, GLO-0004** — the vocabulary, which §Findings shows is already inconsistent
  in a way this party makes acute.

## Finding: the glossary is already inconsistent about "operator", and this party exposes it

Recorded before the proposal because it changes how the proposal must be written.

- **GLO-0001** — *"An operator is the party that holds a Paridyum subscription and runs one or more
  tenants. **The operator is our customer.** Nobody else in the hierarchy is."* Column: `operator_id`.
- **GLO-0004** — the tree is `Paridyum → operator → tenant → account`, with Paridyum **above** the
  operator.
- **BET-0005** — its three-levels table puts **Paridyum itself** in the Operator row, and the
  customer in the Tenant row.

GLO-0001 and BET-0005 place different parties at the same name. Today this is harmless because
Paridyum is the only party that provisions anything, so both readings collapse onto one entity. **A
white-label partner is precisely the party that separates them**, and any design written on top of
the ambiguity inherits it.

There is also a column, `operator_id`, declared in GLO-0001 and present in no migration.

**This must be resolved before any implementation, and probably before this RFC is accepted.** It is
a glossary correction, not a tenancy change — but it is the kind that silently mis-scopes a table.

## Finding: a partner is a tenant that deployed the `app` product — corrected 2026-09-15

Operator direction, and it collapses most of this document:

> *Other users use the app that was deployed by Paridyum to create their tenants just like Paridyum,
> and can deploy `app` and other stuff too if they want to. Tenant zero just seeds the data — that is
> what they do differently.*

Under RFC-0049, `app` is a **product** and a deployed product is a **Release**. So:

> **A white-label partner is not a new kind of party. It is a tenant with an `app` Release.**

Everything follows from that, and three things this RFC proposed stop being needed:

| Earlier draft proposed | Under the corrected model |
| --- | --- |
| P2 — a `partners` registry table | **Unnecessary.** "Partners" is a query: tenants holding an `app` Release |
| P1 — the control plane becomes a product | **Already true.** `app` is one product in the catalogue, deployed like `careers` or `landing` |
| A new structural position between Paridyum and tenant | **None.** A tenant is a tenant; some run more products than others |

This is ADR-0049 decision 2 carried to its conclusion. Tenant zero is not a privileged kind of
tenant — it is a tenant that happens to hold every Release and to have **seeded the data**. Any
tenant may hold the same set.

**What this does NOT collapse.** The genuinely hard items in §Why this is hard survive unchanged,
because they are consequences of *someone else running a platform*, not of how that party is
modelled: metering and royalty across a boundary designed to prevent observation (item 4), migration
fan-out and version skew (2 and 3), the controller/processor chain (5), support recursion (8), a
Zitadel **instance** per platform-running tenant (9), and exit and escrow (7). A simpler model does
not make a recursive business simpler to operate.

### The crux this raises, and it is the whole question

When tenant **B** — holding an `app` Release — creates tenant **C**, *where does C live?*

| Reading | Consequence |
| --- | --- |
| **Flat** — C is a row in the same `tenants` table, a sibling of B | **ADR-0024 decision 9 never fires.** There is no third level; B simply has a console that creates tenants. The hard part becomes *authorisation* — what lets B create tenants, and what stops B reading C's data — which is an entitlement and operator-scope question, not a tenancy-depth one |
| **Nested** — C lives inside B's own placement, in B's database | B is a stamp; C's rows never touch ours. Still no third *column*, but Paridyum now cannot see C at all — which is what makes metering (item 4) and the compliance chain (item 5) hard |

**These are very different systems and the choice is not mine to make.** The flat reading is far
cheaper and is probably what "other users use the app Paridyum deployed to create their tenants"
literally describes — everyone shares one `tenants` table and one control plane instance. The nested
reading is what "deploy `app` **themselves**" implies, since a second `app` instance needs its own
store to create tenants into.

They may also both be true at different times: **flat first** (B gets a console over the shared
platform), **nested later** (B gets its own stamp when it needs its own brand, region or data
boundary). That is a migration between the two, and it is the same export/restore machinery RFC-0045
§Proposal F builds.

Recorded as open question 9 rather than answered here.

## Finding: open question 9 is answered — nested, and a stamp chooses its own region — 2026-09-24

The operator raised a third case the flat/nested table above does not distinguish: **B (a partner
tenant) is domiciled in one region, but B's own customer C wants — for latency or local law — to be
served from a different region than B.** Flat cannot do this at all: C is a sibling row of every
other tenant in one shared database with one region column (T-G.3), so C is served from wherever B
is. The question was whether this forces a *third* model — region pinned per account row inside a
shared database, independent of both B's and Paridyum's placement.

**It does not, and building it would not survive contact with how the rest of the industry treats
this exact problem.**

### What was checked before answering

Three questions, checked rather than assumed, because RFC-0020 requires external facts dated and
cited before they inform a decision here:

**1. Does anyone offer per-sub-tenant region pinning inside one shared multi-tenant database, at
production scale?** No mature platform found does. Salesforce Hyperforce — the most developed
data-residency product examined, spanning 18 countries — provides residency **at the organisation
level only**; it *"does not natively support country-specific data isolation within a single
org."* Auth0's standard multi-tenant product does not support choosing a tenant's sub-region at
all; that control exists only in their **Private Cloud** tier — a dedicated deployment per
customer, which is architecturally a stamp, not a shared-database row attribute.
[Flosum — Salesforce Hyperforce data residency](https://www.flosum.com/blog/salesforce-hyperforce-data-residency-compliance),
[Auth0 Support — tenant creation in a specific sub-region](https://support.auth0.com/center/s/article/Tenant-Creation-in-a-Specific-Sub-region)

**2. Does true row-level region placement exist anywhere as a real pattern?** Yes — but only inside
distributed SQL databases purpose-built for it. CockroachDB's table locality lets an operator pin
individual *rows* to a region as a first-class primitive.
[Cockroach Labs — multi-region database architecture: SQL, placement & locality](https://www.cockroachlabs.com/blog/multi-region-database-architecture-sql-placement-locality/)
**This is not what ADR-0023 chose.** Paridyum runs one Postgres instance under CloudNativePG, on
one hypervisor (ADR-0064), with isolation carried by RLS over two plain columns (ADR-0024) — a
fundamentally different substrate from a geo-distributed SQL engine. Row-level region placement is
a property of the *database engine*, not a column anyone can bolt onto Postgres, and adopting it
would mean replacing ADR-0023's and ADR-0064's engine choice — a supersession this finding does not
propose and the operator has not asked for.

**3. What does the general architecture literature recommend when residency and latency compete?**
The consistent recommendation is to fix the regional footprint as **separate deployments per
region** first, and choose the tenancy model *inside* each one second — with cross-border
replication and failover treated as a compliance risk to be designed out, not an optimisation to
reach for: *"even partial or sharded datasets remain jurisdiction-bound,"* and disaster recovery
*"must remain fully domestic."*
[OneUptime — data residency storage: shared, schema-per-tenant, or database-per-region](https://oneuptime.com/blog/post/2026-09-18-choose-regional-saas-storage-tenancy-model/view)
This is the same posture ADR-0041 decision 3 already takes for Paridyum's own tenants (region
contractual and immutable) and the one ADR-0105 just took for the estate's own failure domain
(decline speculative infrastructure ahead of a funded need) — row-level residency inside one shared
database would be the one place in the whole design that quietly took the opposite posture.

### The answer

**Open question 9 is nested, not flat — and nesting is extended one level: a stamp chooses its own
region at its own provisioning, independent of its parent's.**

A partner's customer who needs African latency or African data law is not a row with a `region`
tag inside a shared Asian database. They are served by **a stamp placed in Africa** — which may be
the partner's own stamp if the partner provisions regionally, or, if a partner's stamp itself needs
to place *its* tenants across regions, a **second tier of stamps**, each a real regional deployment
with its own placement, reachable through the same recursive provisioning saga P3 already
describes.

| | Flat (rejected for this case) | Row-level region-per-account (rejected) | **Nested, region chosen per stamp (adopted)** |
| --- | --- | --- | --- |
| What C's data physically is | A row in the same database as every other tenant | A row tagged with a region, physically anywhere | A tenant inside a real deployment placed in C's region |
| Engine required | Current Postgres/RLS, unchanged | A geo-distributed SQL engine (e.g. CockroachDB) — supersedes ADR-0023 | Current Postgres/RLS, unchanged, once per stamp |
| Failure domain if the wrong region goes down | Shared with every other tenant, everywhere | Unclear — depends on replication topology | Exactly the stamp in that region, and no other |
| Matches an existing production pattern | N/A (doesn't solve the ask) | No mature platform found doing this in a shared DB | Yes — deployment-per-region is the industry default |
| Gated on | Nothing extra | A database-engine change nobody has proposed | T-G.0b's second-failure-domain trigger (ADR-0105) and D9's stamp gate, both already named |

**This does not add a new gate to this RFC.** It sharpens the existing one: the resolution to open
question 9 is **nested**, and "a stamp's region is chosen at its own provisioning, not inherited"
is a one-line addition to §Proposal P3 (the saga that provisions a stamp already asks for a region
the same way ADR-0039's saga asks a tenant for one — nothing here invents a second mechanism). The
crux table above is answered: nested is the reading, and it is the one the direction's "deploy `app`
**themselves**" phrasing already implied.

**What stays correctly blocked.** This finding does not move RFC-0047's own trigger, ADR-0024 D9,
or ADR-0105's decline of a second failure domain. A partner's stamp needing its own region is
*more* infrastructure, not less — so this finding makes the eventual build larger, not sooner. It
answers **where the design points**, not **when it is buildable**.

## Finding: there are two independent gates, and they are not the same gate

An earlier draft treated this as one blocked thing. It is two, and they are unrelated:

| Gate | Governs | Fires on | Who controls it |
| --- | --- | --- | --- |
| **`paridyum` → `franchised`** | whether the platform may be *sold* | the dossier above, and C-0006 anti-goal 6's renewed customer | Paridyum, starting now |
| **ADR-0024 D9's third level** | whether a party may exist *above a tenant in the tenancy tree* | *"a real third level with a paying customer behind it"* | a contract |

**And the second gate may not be engaged by this design at all.** D9 defers a third *discriminator*
— a path or a third column in the shared database. §Proposal refuses exactly that: a partner is a
C3 stamp whose `tenant_id`/`account_id` space starts empty and never joins ours. Nothing is added
to the tenancy tree; a second tree is stood up beside the first.

Whether "a partner in a registry, isolated by deployment" is D9's third level is a **reading
question**, and it is the same shape as ADR-0103 §Status: an agent must not decide for itself that a
lock is narrower than it reads. It is recorded here for a human and is open question 8.

If the reading holds, this RFC is gated by the product dossier and by ADR-0064's hardware question,
and **not** by D9 — a materially shorter path than a third column would ever have been.

## Why this is hard

The honest enumeration. Each item is an existing unsolved problem multiplied, not a new one.

1. **The control plane must become a product, which inverts ADR-0100 decision 4.** Today `/dashboard` is
   the one thing the saga does not produce. Making it producible means the provisioning system must
   be able to provision provisioning systems — and the bootstrap question ("what provisions the
   first one?") acquires a real answer rather than an asymmetry.

2. **Migration fan-out returns, at the worst possible granularity.** RFC-0006 killed
   schema-per-tenant on REQ-0008. RFC-0045 §Proposal B re-admitted a *bounded* fan-out at C1 on the
   argument that a contract-bounded N is not an O(tenants) N. A partner stamp is a **whole estate**
   to migrate, not a database — schema, control plane, identity instance, edge configuration. The
   bounded-N argument has to survive being applied to a much larger unit.

3. **Version skew becomes a product decision.** Partners will not all upgrade at once. Either we
   force lockstep (and own their downtime windows), or we support N versions (and every artifact in
   the estate acquires a compatibility matrix). There is no third option and both are expensive.

4. **Royalty requires metering across a boundary designed to prevent observation.** BET-0007 takes a
   share of a franchisee's GMV. A partner's tenants' GMV lives inside a stamp we deliberately cannot
   read. Billing therefore needs a channel that reports **aggregates without content**, and that
   channel is a data-exfiltration path by construction — the one thing the isolation guarantee
   exists to prevent, built on purpose, with our own incentive pointing at it.

5. **The controller/processor chain becomes three deep, and is jurisdiction-dependent.** A partner's
   tenant's end customer's personal data: the tenant is likely controller, the partner a processor,
   we a sub-processor — and every partner must be able to name us in *their* DPA while their tenants
   name them in theirs. Sub-processor notification obligations chain upward. RFC-0045's attestation
   must become transitive: a tenant of a partner must be able to verify isolation properties of an
   infrastructure operator they have no contract with.

6. **RFC-0022's conflict recurses, and the new layer is worse.** RFC-0022 bounds Paridyum competing
   with its tenants. A partner competes with *its* tenants too, and RFC-0022's defence — a published
   data-use commitment plus the technical guarantee — is ours to make, not theirs. We would be
   supplying the machinery for a conflict we have no standing to bound, to a party whose commitments
   we cannot enforce, in a business that carries our software.

7. **Exit has no answer.** A partner with live tenants who stops paying, or fails, leaves running
   businesses with real customers stranded. Source escrow, data escrow, right-to-operate,
   transfer-of-tenants — all unaddressed, all standard in franchise and reseller agreements, none of
   which the estate has ever needed.

8. **Support becomes two hops, and the middle hop can be asleep.** A partner's tenant's incident
   reaches us through the partner. Response times compound; accountability diffuses; and the party
   with the contractual relationship to the affected business is not the party who can fix it.

9. **Identity: a partner needs a Zitadel *instance*, not an organization.** ADR-0047 gives one org
   per tenant. An instance is a different object with different lifecycle and, per ADR-0041 D3, an
   **immutable masterkey that is a compliance artifact**. Every partner adds one permanently.

10. **Trust is now transitive and we are invisible at the bottom.** A partner's tenant's customer
    has a relationship with a business that has a relationship with a business that runs on us. Our
    reputation is exposed at a depth where our brand is contractually absent — which is BET-0005's
    *"their reputation, not ours"* inverted: the failure is ours and the name on the door is not.

## Proposal

### The central move: a partner is not a third column. A partner is a stamp.

The instinct — and the thing ADR-0024 decision 9 was written to make expensive — is to add a third
discriminator: `partner_id`, `tenant_id`, `account_id`, three RLS predicates, one shared database.
**Reject it.** Three reasons, in order of strength:

1. **It buys nothing the ladder does not already buy, and costs the thing D9 protects.** The entire
   value proposition of renting the platform is *their brand, their control plane, their identity,
   their domain, their data*. That is not a row-filtering problem. Every property the partner is
   buying is a property a shared database cannot provide regardless of how many columns it has.

2. **A partner's tenants need their own accounts, so a third column is really a third silo with all
   the failure modes of the second, at a level where the blast radius is a whole business's
   customer base.** ADR-0024 cost 2 already names the inner boundary as *"not our reputation to
   spend"*; a third makes it someone else's reputation twice removed.

3. **D9's own warning applies literally.** *"Two explicit columns can be migrated to a path; a path
   cannot easily be migrated back."* Adding the third column is the one-way door. Not adding it
   leaves it available.

**Instead: a partner is a C3 dedicated stamp (RFC-0045 §Proposal B), plus a control-plane
deployment.** Their world is a separate cluster, database, Zitadel instance, edge and brand, with
its own `tenant_id`/`account_id` space that starts empty. Inside it, ADR-0024 is unchanged and
unaware that a partner exists — two columns, two predicates, forced RLS, exactly as today.

**This is the buyer C3 never had.** RFC-0045 listed C3 and noted (open question 6) that nothing
could currently deliver it. Here is the party for whom a stamp is not an expensive isolation
upgrade but *the actual product*.

```
Paridyum control plane  (paridyum.com/dashboard)            ← the root, provisions stamps
    │
    ├── Paridyum's own stamp ─── tenant zero ─── tenants ─── accounts
    │
    └── Partner stamp ──── partner's control plane (partner.com/dashboard)
                                   │
                                   └── partner's tenants ─── accounts
```

Paridyum's relationship to a partner is **operational and contractual**, not row-level. There is no
query anywhere that joins across a partner boundary, because there is no shared table to join.

### P1 — This is a product-stage advance, not a new product

**Corrected 2026-09-15.** An earlier draft of this section proposed adding the control plane to the
catalogue as a new product. That was wrong, and the registry already says so.

`product-registry.yaml` carries **`paridyum`, `kind: platform`, `stage: building`**, and the stage
enum is `proposed → building → operated → franchised → retired`. **Renting the platform is
`paridyum` reaching `franchised`.** No new product is created, and the `kind` is `platform`, not
`catalogue` — which matters, because the two kinds owe different dossiers.

The registry encoded BET-0005's *"operating a vertical precedes franchising it"* as **stage
ordering**, so the wager is a gate rather than a slogan: `building → franchised` is not reachable
without passing through `operated`.

| Stage | Owes (per the `platform` dossier) |
| --- | --- |
| `building` (today) | `REQ` |
| `operated` | `REQ`, `RUN`, slot `metrics-framework` |
| `franchised` | + `BET`, and slots `product-contract`, `go-to-market`, `sales-playbook`, `support-sla`, `renewal-evidence` |

`renewal-evidence` is the load-bearing slot, and the registry says why in its own note: it is
*"not a document about selling but the fact that makes selling permissible — C-0006 anti-goal 6
counts renewed customers, not signed ones."*

ADR-0100 decision 4's bootstrap asymmetry is still **broken deliberately and narrowly**: the root
control plane remains un-provisioned (deployed by the estate's pipeline, as today); every *other*
control plane is produced by it. The asymmetry becomes a property of the root rather than of control
planes in general, which is the smallest possible change to D4's claim.

### P2 — The partner registry is the only shared table

One platform-owned table in Paridyum's control plane: `partners`, carrying the contract, the stamp
reference, the entitled catalogue subset, the royalty terms, the version channel, and the lifecycle
state. It holds **no tenant of the partner and no data from inside the stamp**, and nothing in the
estate joins it to anything inside one.

### P3 — Provisioning recurses, with the same saga

ADR-0039's saga gains a step that provisions a stamp, and the stamp's own control plane runs the
*same* saga for the partner's tenants. One implementation, two depths. A partner provisioning a
tenant is bit-for-bit the operation Paridyum performs; that identity is what keeps the code paths
from forking, and ADR-0049's rule — no scope a customer could not hold — is what keeps a partner's
control plane from becoming privileged.

**A stamp's region is an input to its own provisioning, never inherited from its parent.** Per
§Finding "open question 9 is answered," a partner's stamp asks the same region question ADR-0039's
saga already asks a tenant — refused rather than defaulted, per ADR-0103 decision 4 — so a stamp
placed to serve customers in a different region than its parent tenant is the ordinary case the
saga already handles, applied one level up, not a new mechanism.

### P4 — Metering reports counts, never content

The royalty channel (§Why this is hard, item 4) is the design's sharpest edge and gets the
strictest rule: **a stamp emits signed aggregate counters and nothing else** — tenant count, active
accounts, GMV totals in minor units and currency (ADR-0033). No identifiers, no names, no rows, no
free text. The schema is declared, closed, and validated at the receiving end; anything not in it is
rejected rather than stored.

This is the one deliberate channel out of a stamp, so it must be the narrowest thing in the estate
and it must be **auditable by the partner** — they can see exactly what their stamp reports,
because a billing channel a customer cannot inspect is indistinguishable from telemetry they did
not consent to.

### P5 — Version channels, explicitly

Each partner is pinned to a channel with a stated support window and a stated upgrade obligation.
Skew is bounded by contract rather than by hope. Migration fan-out is then O(partners) with
partners bounded by contract — RFC-0045's bounded-N argument, applied at a larger unit, and it needs
the ceiling from RFC-0045 open question 1 stated *before* the first partner rather than after.

### P6 — The attestation becomes transitive

RFC-0045 §Proposal E's attestation gains a `platform_operator` field and a chain: a partner's tenant
receives an attestation naming the partner, which references the partner's own attestation from
Paridyum. A business four levels down can follow the chain to the infrastructure operator and check
each link. This is the mechanism that makes C-0006 non-commodity item 2 — *provable isolation*,
extended to isolation **from the operator** — survive an extra level, and without it the promise
silently weakens with depth.

### P7 — Exit terms are designed with the first contract, never after

Data escrow, tenant-transfer rights, a right to operate through a wind-down, and a defined path by
which a partner's tenants become Paridyum tenants if the partner fails. Standard in reseller
agreements and absent from the estate. Item 7 of §Why this is hard, given an owner.

## Alternatives considered

1. **Third discriminator column (`partner_id`) in the shared database.** Rejected in §Proposal:
   buys nothing the partner is actually paying for, adds a third silo with the second's failure
   modes at greater blast radius, and walks through ADR-0024 D9's one-way door.

2. **Hierarchical path (`ltree` or materialised path) replacing the two columns.** RFC-0007's option
   B. Generalises to arbitrary depth, which is genuinely attractive here. Rejected on D9's own
   recorded grounds — it indexes worse on the hot path and cannot easily be migrated back — and
   because arbitrary depth is a solution to a problem nobody has: nothing suggests partners of
   partners.

3. **Partner's tenants are `accounts` inside the partner's tenant** (reuse the existing two columns
   with no schema change). Superficially elegant and briefly tempting. Rejected as incoherent: a
   partner's tenant is a business that needs *its own customers*, and accounts have no children.
   The model runs out of levels exactly one step in.

4. **Refuse the business entirely; stay a single-operator platform.** Genuinely defensible, and the
   status quo. It forgoes what may be the highest-leverage version of BET-0005's franchise wager —
   franchising the *platform* rather than a vertical — and costs nothing to defer, since the trigger
   gating this RFC is unfired anyway. **This remains the correct answer until a paying customer
   appears**, which is precisely what D9's trigger says.

5. **Licence the software for self-hosting instead of operating stamps.** The partner runs it on
   their own infrastructure. Removes operational burden, metering reach and most compliance chain
   problems in one move — and removes BET-0007's royalty enforceability along with them, contradicts
   BET-0009's hosting posture, and makes support and version skew worse rather than better. Worth
   re-examining if §Why this is hard item 4 proves unsolvable, because it is the alternative that
   dissolves item 4 completely.

## Consequences

**What this makes worse.**

1. **Two things called a control plane, forever.** Every runbook, diagram and incident procedure
   acquires a "which one" qualifier. ADR-0100's clean statement that there is *one* provisioning
   surface stops being true.
2. **A deliberate channel out of an isolation boundary, with our revenue on the other end.** P4 is
   as narrow as it can be made and it is still the thing the whole estate exists to prevent, built
   on purpose. Our incentive to widen it is permanent and structural.
3. **The estate's support and compliance surface doubles at one contract.** Sub-processor chains,
   two-hop incidents, a per-partner immutable masterkey, and a version matrix — all arriving with
   the first partner, none amortised.
4. **C3 stops being theoretical, which surfaces ADR-0064.** A stamp needs a failure domain, and
   `sitar` is currently the only one. This RFC is undeliverable on one hypervisor, and says so
   rather than discovering it during a sale.
5. **RFC-0022's defence is extended to a party we cannot bind.** We supply the machinery for a
   partner-versus-its-tenants conflict and have no standing to commit on the partner's behalf.
6. **The glossary ambiguity in §Findings must be fixed first**, and fixing a glossary entry that
   names a column present in no migration may surface more than expected.

**What it does not change.** ADR-0024's mechanism, at any depth. Inside every stamp — ours, a
partner's — there are two columns, two predicates, forced RLS, session context from the verified
token, and a runtime role that cannot bypass. The recursion happens at the deployment layer and
stops cleanly at the database, which is the single property that makes the design safe enough to
write down.

## Rollout

**None. The trigger has not fired.** Recording a shape only, so that the first contract does not
start from nothing:

Ordered against the **`platform` dossier ladder**, because that is the ladder the registry already
enforces — not against phases invented here.

| Phase | What it would be | Gate |
|---|---|---|
| 0 | Resolve the GLO-0001/GLO-0004/BET-0005 ambiguity in §Findings | none — **do this now regardless**, it is a correction, not a feature |
| 1 | **`paridyum` → `operated`**: Paridyum runs a real business on its own runtime; `RUN` written; `metrics-framework` slot filled | ADR-0064 D3's backup gate, which currently blocks the first real applicant row — so the *backup rehearsal* is what gates the product stage |
| 2 | RFC-0045 slices 1–7; C1 proven with Paridyum as the tenant | RFC-0045 |
| 3 | C2 and C3 proven | **ADR-0064 hardware answer** |
| 4 | Root asymmetry narrowed; partner registry; metering channel; transitive attestation | phase 3 |
| 5 | The `franchised` slots: `BET`, `product-contract`, `go-to-market`, `sales-playbook`, `support-sla` | legal and commercial, not engineering |
| 6 | **`renewal-evidence`** — a renewed customer, not a signed one | C-0006 anti-goal 6 |
| 7 | First partner | open question 8's reading; C-0006 anti-goal 1; BET-0005 trigger clause 1 |

**Phase 0 is the only thing here that should happen now**, and it is not a feature.

**Phase 1 is the chain worth noticing.** The off-host backup and rehearsed restore that gate the
first real applicant row are the same thing that gates `building → operated`, which gates
everything below it. One unglamorous piece of infrastructure work sits under the entire franchise
path.

**Graduation.** If ever built, this becomes an ADR superseding ADR-0024 D9 and amending ADR-0100 D4,
and it would be among the most consequential in the record. It stays an RFC — and probably a
`deferred` one — until then.

## Open questions

1. **Who is `operator` — us or our customer?** §Findings. Blocking, and cheap now.
2. **Can the royalty be measured without a channel out of the stamp?** Partner-declared with audit
   rights, payment-rail-derived, or something else. If not, alternative 5 (licensing) deserves
   another look, because it is the only option that removes the channel entirely.
3. **Is C3 deliverable at all?** Inherited from RFC-0045 open question 6 and RFC-0046 open question
   3. Blocking for phase 2, and the same hardware answer resolves all three.
4. **One version or many?** §Why this is hard item 3. A product decision with a permanent
   engineering consequence, and the wrong default is very expensive.
5. **What is Paridyum's liability for a partner's tenant's data?** Legal, jurisdiction-dependent,
   and it determines whether the business is insurable.
6. **Does a partner get the catalogue, or a subset?** BET-0004 and C-0006 anti-goal 1 interact here:
   we may not franchise a vertical we have not operated, and a partner reselling a product we have
   not run would breach that through an intermediary.
7. **What stops a partner becoming a competitor with our own software?** BET-0005's adverse-selection
   cost, one level up and considerably sharper.

8. **Does a stamp-isolated partner engage ADR-0024 decision 9 at all?** §Finding argues it may not,
   since no third discriminator is added to the shared database. A human must settle the reading;
   until then this RFC assumes the stricter answer (that it does) and stays gated.

9. ~~**Flat or nested — where does a partner's tenant live?**~~ — **answered 2026-09-24: nested**,
   per §Finding "open question 9 is answered." Flat cannot serve a partner's customer from a
   different region than the partner without a row-level-residency database engine nobody has
   proposed (checked against Salesforce Hyperforce, Auth0, and CockroachDB — see that finding's
   sources); nested, with a stamp choosing its own region at its own provisioning, matches how the
   industry solves this and requires no new mechanism beyond what P3 already recurses. D9 is
   engaged exactly as §Finding "two independent gates" already described — open question 8's
   reading still governs whether that matters, and is unaffected by this answer.

## Related

- **ADR-0024 decision 9** — the locked, unfired trigger this RFC describes, and the one-way-door
  warning that decides §Proposal
- **ADR-0100 decision 4** — the bootstrap asymmetry P1 narrows
- **ADR-0103, RFC-0045** — the containment ladder; C3 is what a partner actually is
- **RFC-0046** — BYOK, which a partner will ask for on day one
- **BET-0004, BET-0005, BET-0007** — the catalogue, the franchise wager, and the royalty with no row
  for this level
- **RFC-0022** — the competition conflict that recurses
- **C-0006** — anti-goal 1 and non-commodity item 2, both load-bearing
- **GLO-0001, GLO-0002, GLO-0004** — the vocabulary §Findings shows must be repaired first
- **REQ-0008** — no-downtime migrations, back for a third time
- **ADR-0064** — the one hypervisor that makes every stamp in this document currently undeliverable
- **ADR-0105** — the second-failure-domain decline; open question 9's nested-with-region-per-stamp
  answer is a design held against that trigger, same as the rest of this RFC
- **T-G.0c** — the Genesis task that raised the region-per-account case answered here
