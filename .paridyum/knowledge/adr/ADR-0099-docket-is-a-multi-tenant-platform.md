---
id: ADR-0099
title: docket is a multi-tenant platform; the tenant comes from the token and configuration is per-tenant data
type: adr
status: accepted
created: 2026-09-15
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [multi-tenancy, docket, auth, rls, config, adr-0024, adr-0038, adr-0082, adr-0092, adr-0093, adr-0098, adr-0070]
breaking: true
enforced_by: []
---

# ADR-0099 — docket is a multi-tenant platform

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0114.** Where this ADR extends the `provision-org` path,
> read `provision-tenant.py` / `make tenant-provision`: the tool was renamed for what it provisions (ADR-0113 D4),
> and every tenant it provisions now has an owning org (ADR-0114 D4). The decision itself stands.

## Status

**ACCEPTED** as of 2026-09-15. Extends ADR-0093 (docket served, DB-backed,
multi-user) from multi-*user* to multi-*tenant*, and reframes ADR-0092's operator
identity as tenant-scoped. Not locked; the config-plane migration lands in stages
and each stage is checkable as it arrives.

## Context

ADR-0093 made docket a served, database-backed, multi-*user* staff platform, and
the schema it produced (migrations 0006/0007) is already tenant-scoped: every
row carries `tenant_id`, every table `FORCE`s RLS, and `pg.rs` sets
`paridyum.tenant_id` per transaction. The data plane is multi-tenant already.

Two things are not. The **served backend** reads one fixed `DOCKET_TENANT_ID` at
startup and stamps it on every request, so one deployment serves exactly one
tenant. And the **configuration** docket runs on — the companies it issues
documents for, the signatories who countersign, the operators who act, the field
sets, the templates and the document lifecycle — lives in git under
`tool-docket/registry/`, declared once, for Paridyum. A second tenant has no
company, no signatory, no template of its own; it would issue Paridyum's.

The estate already knows how to be multi-tenant, and docket should not invent a
second way. ADR-0024 is the RLS posture, ADR-0038 the tenant/account accessors,
ADR-0082 derives a tenant's uuid from its Zitadel organization id, and web-console
demonstrates the pattern end to end: the token carries the `resourceowner` (org)
claim, the app derives the tenant, and RLS does the isolation. docket adopts it.

The operator model needs the same move. ADR-0092 binds a control-plane subject
to a docket operator through a **global** `operators.yaml` — correct for one
tenant, wrong for many, because operator identity (who may act, and whose name
goes on a document) is a property of a tenant, not of the estate.

## Decision

**1. The tenant is derived from the operator token, per request — never from a
process-wide env.** `auth.rs`'s verified operator carries the `resourceowner`
(Zitadel org id) alongside the subject; the backend derives the tenant uuid from
it exactly as ADR-0082/`deriveTenantId` does, and that tenant is the RLS context
for every store call in that request. `DOCKET_TENANT_ID` is removed from the
served backend (it remains only for the break-glass CLI, which names its one
tenant explicitly). One deployment serves every tenant; RLS keeps them apart, as
it already does for `applications`, `profiles` and the docket chains.

**2. Configuration is per-tenant data, not git.** docket's registry —
`companies`/`org`, `signatories`, `operators`, `fields`, `person-classes`, the
document `lifecycle`, and the `templates` — moves into tenant-scoped tables in
`platform-tenancy` (RLS forced, keyed by `tenant_id`), the same shape the data
plane already uses. What is Paridyum's today becomes **the first tenant's rows**
(ADR: Paridyum is the first tenant), seeded from the current git registry, not a
special case in code. A tenant with no configuration is onboarded, not broken:
onboarding seeds a default config it then edits.

**3. Operator identity and document attribution are tenant-scoped.** An operator
is a subject that (a) carries the `operator` role in a tenant's org token and (b)
has an operator row in *that tenant's* config, which supplies the name, title and
signatory role a document is attributed to. ADR-0092's subject binding stands;
its registry becomes a per-tenant table. Cross-tenant action is impossible by
construction: the token's org IS the tenant, and RLS admits no other.

**4. A tenant configures itself through the served surface.** web-docket
(ADR-0098) gains the admin area for a tenant's own companies, signatories,
operators, templates and field sets — operator-only, tenant-scoped, behind the
same BFF. The git registry stops being the source of truth once a tenant's rows
exist; it survives only as the seed for onboarding and as Paridyum's own
historical record until its rows are migrated.

**5. It ships in stages, and Paridyum keeps working throughout.** The order is:
(i) schema for the per-tenant config tables; (ii) backend reads tenant-from-token
and loads config from the DB, falling back to the git registry only while a
tenant's rows are absent; (iii) seed Paridyum's rows from its git registry and
cut over; (iv) the onboarding flow for a new tenant; (v) the web-docket admin
surface. Each stage is a reviewed change; none flips Paridyum's live deployment
until its rows are seeded and verified.

## Consequences

- **`DOCKET_TENANT_ID` leaves the served backend.** The chart drops it; a token
  with no derivable tenant is refused, not defaulted — the failure is loud.
- **A large schema addition** in platform-tenancy: the config tables, each
  tenant-scoped and FORCE-RLS, validated the way every migration here is. The
  document lifecycle and templates are the hardest, because a template is
  reviewed content and a tenant editing its own templates is a content-review
  surface, not a form.
- **The break-glass CLI stays single-tenant-explicit.** It names its tenant
  (`DOCKET_TENANT_ID`) and reaches the DB directly; nothing about it changes,
  because a break-glass path that inferred the tenant would be less legible, not
  more.
- **Onboarding becomes a first-class operation.** Creating a tenant is: a Zitadel
  org (ADR-0045), its derived tenant row (ADR-0082), the `operator` role granted
  to its staff, and a seeded default config. This is the platform-tenancy
  `provision-org` path extended to docket's config.
- **The registry in git is demoted from truth to seed.** RFC-0041's "declared in
  git" reasoning already gave way for postings (ADR-0095); this completes the
  move for the rest of docket's configuration, for the same reason: what a tenant
  owns and edits cannot live in Paridyum's repository.
- **Security posture is unchanged in kind, wider in blast radius.** The isolation
  is RLS, already forced and already tested cross-tenant (platform-tenancy
  logbook). The new risk is a backend bug that derives the wrong tenant from a
  token; it is mitigated by deriving deterministically from a signed claim
  (ADR-0082, no lookup) and by RLS refusing any row outside the derived tenant.
