---
id: RFC-0014
title: A region is a residency boundary, and every table declares which plane it lives in
type: rfc
status: proposed
created: 2026-08-11
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [region, residency, datacentre, control-plane, data-plane, tenancy, adr-0024, adr-0041]
breaking: true
enforced_by:
  - check: db.tables_declare_their_plane
    severity: error
    rationale: >
      A table's plane is not inferable from its schema, and a pii column landing in the
      control plane instead of a region is a cross-border transfer every other check
      reports as green.
  - check: db.tenant_scoped_tables_are_regional
    severity: error
    rationale: >
      A tenant's rows live in that tenant's region and nowhere else, so a tenant-scoped
      table declared control-plane is one of two declarations being wrong.
sources:
  - url: https://eur-lex.europa.eu/eli/reg/2016/679/oj/eng
    title: Regulation (EU) 2016/679 — GDPR
    retrieved: 2026-08-12
    supports: >
      Art. 1(3) — free movement of personal data within the Union may be neither restricted
      nor prohibited. Why the four EEA countries are ONE jurisdiction, and why a region code
      cannot simply be a country code.
  - url: https://tietosuoja.fi/en/transfers-of-personal-data-out-of-the-eea
    title: Transfers of personal data out of the EEA — Finnish Data Protection Ombudsman
    retrieved: 2026-08-12
    supports: A transfer to another EEA state is treated as one staying in Finland.
  - url: https://docs.cloud.google.com/model-armor/data-residency
    title: Google Cloud — data residency and endpoints
    retrieved: 2026-08-12
    supports: >
      Residency published at jurisdiction level (a country, or a multi-region such as `eu`)
      separately from the infrastructure identifier — precedent for the two-layer split.
  - url: https://www.recordinglaw.com/world-laws/world-data-privacy-laws/data-localization-laws-by-country/
    title: Data localization laws by country, 2026
    retrieved: 2026-08-12
    supports: The per-jurisdiction `localization` values in region-registry.yaml.
  - url: https://www.pertamapartners.com/insights/cross-border-data-transfers-asia
    title: Cross-border data transfers in Asia — 2026 guide
    retrieved: 2026-08-12
    supports: >
      India's DPDP Rules (November 2025) use a negative-list transfer model with the list
      unpublished; Japan and South Korea hold EU adequacy.
  - url: https://practiceguides.chambers.com/practice-guides/data-protection-privacy-2026/indonesia/trends-and-developments
    title: Data Protection & Privacy 2026 — Indonesia (Chambers)
    retrieved: 2026-08-12
    supports: >
      No general localization; banking requires primary and DR centres in-country, healthcare
      requires domestic storage. The sectoral trigger.
  - url: https://www.forcepoint.com/blog/insights/tracking-global-data-protection-laws-2026
    title: Tracking global data protection laws, 2026
    retrieved: 2026-08-12
    supports: >
      Regime names for the remaining jurisdictions — LGPD, POPIA, PIPEDA, Privacy Act 1988,
      Law 21.719, and the UAE's federal PDPL alongside DIFC and ADGM.
---

# RFC-0014 — A region is a residency boundary

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0115, ADR-0117.** Decisions 3 and 4 are resolved by
> **ADR-0115** (four scopes — `platform`, `org`, `tenant`, `account` — one schema per plane's
> database; ADR-0024 superseded) and **STD-0023** (the plane directive ratified). Where this RFC
> says an **operator** owns tenants, read **org** (GLO-0012); its people are org-scoped and regional
> (ADR-0117). The retired name for the control region is quoted below because this RFC retired it.
> Everything else is unchanged and still `proposed`.

## Status

**PROPOSED** as of 2026-08-11. Feedback deadline **2026-09-10**, set inside ADR-0011's 30-day
staleness warning rather than beyond it.

**First of four.** This defines the structure — what a region is, what the planes are, how a table
declares where it lives. RFC-0015 decides how the planes behave, RFC-0016 the edge, RFC-0017 where
an operator's own data lives.

If accepted this resolves into a new ADR recording the region model, and supersedes both `locked`
artifacts it contradicts: **ADR-0024**, whose title carries the sentence at issue — *"in one
database"* — and **STD-0006**, the standard that ratifies the migration directive set, to which
decision 3 adds a second directive.

Neither may be edited in place. C-0004's decision table puts *"overwriting locked artifacts"* in
**deny — never legitimate**, and `knowledge.locked_artifacts_immutable` enforces it.

## The question

`region` is load-bearing across the decisions already taken, and defined in none of them.

ADR-0041 makes `tenant.region` contractual and immutable. ADR-0047 puts one Zitadel deployment,
one virtual instance and one Postgres per region, and a `paridyum` instance in what it calls the
home region — renamed **control region** below, for reasons decision 1 gives.
ADR-0042 gives `platform-routing` the job of resolving *"a request to a tenant and a region"*.
ADR-0039's saga writes `region` at step 1 and claims the same machinery serves region migration.
ADR-0045 makes the Zitadel masterkey immutable per region and a region's loss unrecoverable.

Not one of them says what a region **is**, how many exist, where the boundary between global and
regional falls, or what happens to ADR-0024's *"one Postgres database, one schema"* when a tenant's
data may not leave Frankfurt.

CLAUDE.md states the consequence of leaving that unwritten: *"The absence of a written answer is not
neutral here; it is an instruction to guess."* The archive is the evidence. REGION-MODEL.md §7 records
what guessing produced: eight regions declared in four or more places with nothing validating
agreement, a schema shape branching on `cfg.is_dev` so dev and prod differed structurally, two
competing region-disable authorities nothing reconciled, and all eight seeded pointing at one
cluster-local endpoint. Modelled, never deployed.

## Decision 1 — a region is a residency boundary, and nothing else

**A region is the set of infrastructure within which one tenant's data is permitted to exist.**
It is a legal boundary. It is not a latency zone, not a capacity pool, not a failure domain, and
**not a building.**

**A region contains one or more datacentres.** A datacentre is a failure domain; a region is a
residency boundary. They are different questions and conflating them is what makes residency look
like it costs availability, which it does not.

The rule that separates them:

> **Adding a datacentre is an operational act. Adding a region is a legal one.**

A datacentre is invisible to a tenant, absent from every contract, and may be added, drained or
retired without anyone being told. A region is `tenant.region` — contractual, immutable, and
changeable only by migration and contract amendment (ADR-0041). Nothing tenant-facing may ever
name a datacentre, because doing so would make an operational fact contractual by accident.

**How many datacentres a region has is a property of that region, recorded in the registry, not a
platform-wide policy.** Some jurisdictions offer several; some will offer one. RFC-0015 decision 3
depends on this number, and a design that assumes every region has two would be wrong in exactly
the jurisdictions residency matters most.

**Naming warning, and one rename this RFC already makes.** A datacentre must not be called a *zone*
or a *site*: both words are already in use here — `zone` for time zones (GLO-0006, REQ-0009, the
`timestamptz` checks) and DNS zones, `site` for a location in the source.

**`home region` is retired, and replaced by two terms**, because it had already acquired the second
meaning GLO-0004 exists to prevent. ADR-0047 uses it for *where Paridyum's own staff instance lives*;
read plainly it means *the region a customer belongs to*. Both readings are natural, which is what
makes the word unusable:

| Term | Means | How many |
| --- | --- | --- |
| **control region** | where the control plane and the `paridyum` staff instance live | exactly one |
| a tenant's **region** | where that tenant's data lives | one per tenant |

ADR-0047 decision 1's *"a `paridyum` instance in the home region"* means **control region** in this
vocabulary. Both terms now carry glossary entries — GLO-0007 and GLO-0008 — written before either
reached a registry column, which is the order GLO-0004's cost argues for.

**A region code is its jurisdiction code, and never carries a number.**

| | Form | Example |
| --- | --- | --- |
| Region | lowercase ISO 3166-1 alpha-2, or a supranational code where one governs | `in`, `eu`, `sg` |
| Datacentre | `<region>-<IATA city>-<ordinal>` | `in-mum-1`, `in-hyd-1` |

> **If you are reaching for a number on a region, you want a datacentre.**

Because two locations in one jurisdiction are, by this decision's own definition, two datacentres —
so a number can only ever distinguish buildings. The archive is the counter-example and the reason
this is stated: it used AWS-shaped codes, and in AWS `ap-south-1` is Mumbai while `ap-south-2` is
Hyderabad, which AWS calls **two regions**. Both are India. Adopting the shape imports a meaning that
contradicts this model, and it did mislead once. The registry schema now rejects `ap-south-1` as a
region code outright. This two-layer split — a jurisdiction for compliance, an identifier for
infrastructure — is also how Google Cloud publishes its own residency commitments.

### The markets currently contemplated

Seventeen countries, but **not seventeen jurisdictions** — the EEA is one, so Finland, Sweden,
Poland and Spain share a residency boundary rather than having four.

**The list itself is `region-registry.yaml`'s `jurisdictions:` block, and is deliberately not
repeated here.** It carries each jurisdiction's code, the countries it covers, its governing regime
and — the field that actually drives planning — whether that regime forces localization. A table in
prose beside a registry is the hand-maintained duplicate ADR-0035 exists to stop.

Two things about it are worth stating in the argument rather than the data. **It is a namespace, not
a build list**: most of these regimes permit cross-border transfer with safeguards, so market
ambition does not imply a region per entry, and RFC-0015 decision 2 still runs one on a
signed-contract trigger. And **the entries that will actually force a region are sectoral** —
Indonesian banking and healthcare, Indian payment data — which fire on the first customer in that
sector rather than that country, and which a country-driven region plan would miss entirely.

**IP address allocation is a different axis.** The archive's IPAM catalogue allocates address space
per *country*; a region is per *jurisdiction*, and a country routinely has the first without the
second.

Three consequences follow directly and are stated so they are not rediscovered:

1. **A tenant lives in exactly one region, permanently.** ADR-0041 already says this; the design
   consequence is that there is **no cross-region consistency problem for tenant data at all.** No
   replication, no conflict resolution, no distributed consensus, no global transaction. Regions
   are shared-nothing with respect to tenant data. This is the single largest simplification
   available and every decision below is built to preserve it.
2. **Regions are not tiers.** No region is sold as faster, cheaper or better, and none is a paid
   upgrade — offering one as a performance or pricing option would make it a product feature and
   invite exactly the isolation-tier conversation C-0006 anti-goal 5 defers. This is a rule about
   what is *sold*, not about what is *considered*: proximity to a tenant's users is a legitimate
   input when choosing where to put them (RFC-0015 decision 5), because being near your users is not a tier,
   it is the ordinary case.
3. **Adding a region is never a scaling decision.** Capacity is answered by adding datacentres
   inside a region, which needs nobody's permission. ADR-0041 already accepts
   the cost: *"Immutable regions constrain operations… Capacity planning becomes per-region and
   permanent."*

**The region registry is generated from one manifest.** Region codes, geography, endpoints and
status derive from a single declared source, in the shape ADR-0040 already established for the
system view and ADR-0022 for artifact types. The archive's first and worst problem was eight
regions declared in four places with nothing checking they agreed; a generated registry is the
only form that cannot drift.

**A region has exactly one status authority.** The archive carried two — a `DISABLE_REGION` env
CSV and a `regions.status` column — with nothing reconciling them, so a region could be `active`
in the database and disabled in configuration simultaneously. The registry is the authority; every
other consumer reads through it and none carries an independent denylist.

## Decision 2 — three planes

| Plane | Where | Holds | Written by |
| --- | --- | --- | --- |
| **Edge** | Global, rented | Nothing durable | Nobody |
| **Control plane** | Control region, one | The global skeleton — which operator owns which tenant, in which region, on which plan — plus the region registry, domain bindings, and staff identity. **No operator personal data** (RFC-0017). | Itself, solely |
| **Data plane** | Each region, N | Tenants, accounts, all business data, regional Zitadel, regional storage | Itself, solely |

The edge is stateless. The control plane is the only writer of global facts. Each data plane is
the only writer of its own tenants' data. **No plane writes into another.** ANTI-0005 is satisfied
by construction rather than by discipline.

## Decision 3 — every table declares its plane

This is the hazard this RFC creates, and the one thing that must not be deferred.

ADR-0024 declares three table categories and argues that the category must be **declared** because
*"the two cases are genuinely indistinguishable from the schema alone: `plans` is platform-owned and
`invoices` is tenant-scoped, and nothing about either name says which."*

Multi-plane adds a fourth question no column answers: **does this table live in the control plane or
in every region?** `check-migrations.sh` parses `CREATE TABLE` statements statically with no live
connection, so it will pass a table placed in the wrong plane without complaint. A `pii` column
landing in the control plane instead of a region is a residency breach that every check reports as
green — the same silent-failure shape ADR-0023 cost 1 describes for a missing `tenant_id`.

**Every `CREATE TABLE` declares `control-plane` or `regional`, by exactly the mechanism ADR-0024
already uses for its three categories** — a line comment immediately above the `CREATE TABLE`.

**And that mechanism belongs to a locked standard, which this RFC therefore also has to supersede.**
STD-0006 is `locked` and it is the artifact that *fixes* the directive syntax — its own frontmatter
says *"the directive this standard fixes is the input to that check. Without a ratified directive…"*.
A second directive is not a free extension of a convention; it is a change to the standard that
ratifies conventions. This was missed when the check below was first written, and is recorded here
rather than left for the gate to find:

```sql
-- paridyum:scope  platform | tenant | account     -- whose the rows are   (ADR-0024)
-- paridyum:plane  control  | regional             -- which database      (this RFC)
```

Two check ids, declared in `check-registry.yaml` and **implemented in `check-migrations.sh` on the
day this RFC was written** rather than left to the ADR:

| Id | Spec | State |
| --- | --- | --- |
| `db.tables_declare_their_plane` | Fails when a `CREATE TABLE` declares neither `control` nor `regional`. | implemented |
| `db.tenant_scoped_tables_are_regional` | Fails when a table declared tenant-scoped or account-scoped is also declared `control`. | implemented |

Implementing before ratification is the cheaper order: the gate parses migrations statically, none
exists yet, and a rule arriving with its enforcement never spends time in the state CLAUDE.md calls
*"enforced by your compliance rather than by machinery."*

**Both were verified by planting the defect they should catch**, per CLAUDE.md's *"treat a green
check as unproven until you have watched it fail"*. Fixtures live beside the existing ones, and
each fails for exactly one reason.

The second keys on the **declared category**, not on the presence of a `tenant_id` column. A
platform-owned table may legitimately reference a tenant id as a key without being tenant-scoped —
the domain-binding table is exactly that, and ADR-0024 already treats `regions` this way.

Both are decidable by reading a migration, which is why they are errors rather than review notes.

## Decision 4 — ADR-0024 is superseded, narrowly

ADR-0024 is `locked` and its **title** reads *"Nested tenancy is two columns in one database."*
The real topology is one control-plane database plus one database per region. That is a contradiction
of a locked artifact, not a clarification of it, and CLAUDE.md permits exactly two responses:
supersession, or an erratum. ADR-0031 scopes errata to *"a factual error rather than a wrong
decision"*, which this is not. **Supersession.**

The supersession is narrow in substance and total in form. Everything ADR-0024 decided survives
unchanged — three table categories, RLS `ENABLE` and `FORCE`, operator scope as a positive grant,
both identifiers from the verified session only, the runtime role unable to bypass RLS, no
per-tenant infrastructure, and both deferred triggers. What changes is the **scope** of the phrase
"one database": it describes the relationship between tenants and a database, not a global
inventory. Tenants share a database; regions do not.

**The mechanical cost is near zero.** Of the five `db.*` checks ADR-0024 declares, four are owned by
`check-migrations.sh`, which parses migration files statically and holds no database connection;
the fifth, `db.no_tenant_id_from_client_input`, is owned by `tool-forge` and is not built. **None of
the five counts databases**, which is the property that matters — the count was inspected in
`check-registry.yaml` rather than inferred from the owner. The successor ADR inherits every
`enforced_by` entry unchanged and adds the two ids from decision 3.


## Consequences

**Accepted costs**

1. **A fourth thing every migration author must get right**, on top of ADR-0024's three categories.
   The declaration is cheap to write and the check is decidable, but it is one more place a
   repository can be wrong, and being wrong is silent until a residency question is asked.

2. **The plane checks run, and they still cannot see the case that matters most.** Both catch a
   *wrong or missing declaration*. Neither can catch a table declared platform-owned and
   control-plane that actually holds one tenant's data, because no check reads the meaning of a
   row. That is the same gap ADR-0024 accepts for its own three categories, inherited rather than
   introduced — but it is the gap through which a residency breach would actually arrive.

3. **Superseding a locked ADR costs a full restatement.** REQ-0002 makes supersession whole-artifact:
   the successor must restate everything ADR-0024 said, not only the sentence that changes. That is
   deliberate friction and it is being paid for one word.

**Gained**

- The region concept has a definition, one authority, and one generated form — closing the archive's
  first and worst problem before it can recur.
- The plane hazard is visible and checkable before the first migration exists, which is the only
  moment it is cheap. ADR-0024 itself warned that the first migration settles this and *"none of it
  is a later refactor."*
- Region and datacentre stop being the same word, which is what made availability look like it cost
  residency.

## Alternatives considered

- **One global database with region as a column.** Simplest, and it preserves ADR-0024 verbatim.
  Rejected because it is not a residency architecture at all: rows for an EU-resident tenant sitting
  in a cluster in another jurisdiction is the exact thing ADR-0041 made contractual.

- **Declare the plane per repository in `manifest.yaml`** rather than per table. Simpler to check.
  Rejected because `platform-tenancy` owns the operator → tenant → account hierarchy (ADR-0042) and
  operators are control-plane while tenants and accounts are regional — the context straddles both
  planes. A repository-level declaration would force splitting it and reopening ADR-0042's
  five-context decomposition for a reason that has nothing to do with bounded contexts.

- **A sixth context owning operators and subscriptions**, so no context straddles a plane. Cleaner
  conceptually and rejected on cost: ADR-0042 already calls five repositories before one product
  ships *"platform plumbing a customer never sees."* Revisit if the straddle causes a real defect.

- **An erratum on ADR-0024 rather than a supersession.** Cheaper, and wrong: ADR-0031 scopes errata
  to *"a factual error rather than a wrong decision"*, and the scope of "one database" is a decision.

## Open questions

1. **Where does the region registry live?** This RFC says it is generated from one manifest and does
   not say which repository holds it. `knowledge/` matches the pattern of the five existing
   registries and the argument that a region binds code, infrastructure and agents alike. An `infra-`
   repository matches ADR-0037's *"infrastructure repositories define the runtime"*. The registries
   in `knowledge/` today all describe the governance system rather than the running product, which
   is the strongest argument against putting it there.

2. **Is `platform-routing` a service at all?** ADR-0042 already called it *"the least justified of
   the five."* If the edge is rented (RFC-0016) and the routing table is generated configuration,
   the context is a build step rather than a runtime — which would make it `tool-` rather than
   `platform-`, and drop ADR-0020's Kubernetes obligations entirely. This RFC does not settle it and
   should not; ADR-0042 owns that boundary.

## Reversal conditions

- **Decision 3 reverses** if the plane turns out to be derivable from the schema after all — which
  would require a naming rule strong enough that `plans`, `invoices` and `regions` are separable
  without a declaration, and ADR-0024 already argues no such rule exists.
- **Decision 4 does not reverse.** A supersession is one-way; ADR-0024 would have to be superseded
  again.

## Related

- ADR-0024 — the locked artifact decision 4 proposes to supersede, and the source of decision 3's argument
- ADR-0041 — `tenant.region` as contractual and immutable; the decision this RFC builds the machinery for
- ADR-0047 — one instance per region, and the control region this RFC's control plane occupies
- ADR-0042 — `platform-routing` owns region resolution; also the context that straddles two planes
- ADR-0045 — the masterkey that is immutable per region, and the practice of naming accepted risks
- ADR-0040, ADR-0022 — the generated-from-a-registry shape decision 1 follows
- ADR-0037 — infrastructure repositories define the runtime rather than run in it
- REQ-0002 — why a supersession must restate the whole artifact
- REQ-0003 — item 8 owns residency, which is what decision 3's check defends
- STD-0006 — `locked`; the standard that ratifies migration directives, which decision 3 requires superseding
- GLO-0007, GLO-0008 — control region and datacentre, the vocabulary decision 1 settles
- GLO-0004 — the precedent: one word, two isolation boundaries, and what it cost
- RFC-0015, RFC-0016 — the other two parts of this decomposition
- `archived/paridyum-docs/REGION-MODEL.md` — what the previous estate built, and its own §7 decision list
