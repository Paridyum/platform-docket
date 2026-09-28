---
id: ADR-0116
title: Nothing in the runtime resolves without an org or a tenant, and no capability ships without a buyer
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: [ADR-0049]
superseded_by: null
scope: [platform, service, library, spec]
applies_to: ["platform-*", "svc-*", "lib-*", "spec-*"]
product: [tenancy, paridyum]
tags: [tenancy, org, dogfooding, resale, self-preferencing, request-path, manifest, portal, staff, platform-context, adr-0038, adr-0049, adr-0114, adr-0115, c-0008, rfc-0017, rfc-0022]
breaking: true
enforced_by:
  - check: domain.no_tenantless_request_path
    severity: error
    rationale: >
      An operator bypass never looks like an exception at the time; it looks like the obvious way to
      get the first thing working. Carried from ADR-0049; its spec widens from "resolves a tenant" to
      "resolves exactly one of decision 1's contexts".
  - check: domain.platform_context_reads_only_platform_tables
    severity: error
    rationale: >
      Decision 1's third context is the one most likely to grow. It is legal only because it can read
      nothing but the personless skeleton; the first org- or tenant-scoped read added to it is the
      support console ADR-0049 D4 warned about, arriving under a new name.
  - check: domain.no_operator_only_capability
    severity: error
    rationale: Carried from ADR-0049 unchanged.
  - check: domain.operator_tenant_holds_no_extra_grant
    severity: error
    rationale: >
      GLO-0005 makes scope a grant; this is the one principal for whom nobody would think to check.
      Widened to the Paridyum org.
verified_by:
  - TST-0019
  - TST-0020
---

# ADR-0116 — Nothing in the runtime resolves without an org or a tenant

## Status

**ACCEPTED** 2026-09-28 by the operator. **Supersedes ADR-0049** (`locked`), restated in full. Only
decision 1 changes in substance; decisions 2 to 5 are ADR-0049's, with "org" beside "tenant" where the
org model requires it.

**Why D1 had to change.** ADR-0049 D1 required every request path to derive `tenant_id`. The portal at
paridyum.in (ADR-0118) acts for an **org**, which is not a tenant (ADR-0114). Satisfying the old D1
would have meant running the portal inside a tenant — calling the portal a business, which is the
mixing ADR-0113 exists to end.

⚠️ **As with ADR-0049, the named checks are declared and unbuilt.** Until they run, this is enforced
by your compliance. Behave as though they were live.

## Context

BET-0005 commits Paridyum to operating the catalogue it franchises, and declines every commercial
boundary — no vertical exclusivity, no territory, no exit. RFC-0022 D1 and D5 are the whole remaining
defence, and both rest on one assumption: **that Paridyum has no mechanism its tenants lack.**

That assumption is false by default. The cheapest way to build any first capability is to build the
half Paridyum needs and defer the tenant-facing half, and every such shortcut is invisible afterwards —
it does not look like a bypass, it looks like the code. C-0001 principle 6 names the shape, and an
operator bypass is the most attractive exception available.

**The org model adds two parties the old rule could not place:**

- **An org at the portal.** It is Paridyum's customer, and the portal's requests act for it — never
  for a tenant.
- **Paridyum's staff operating the platform.** Someone must run the saga's failures, suspend an abusive
  tenant, and answer "which region is this tenant in". ADR-0100 placed that inside tenant zero, which
  made Paridyum's own business the one tenant holding the provisioner's powers. ADR-0118 removes that,
  so the power needs a home that is **not** a back door into anyone's data.

## Decision

**1. Every runtime request resolves exactly one context from the verified session, and refuses when
it cannot.**

| Context | Who | Resolves | May read |
| --- | --- | --- | --- |
| **tenant** | a tenant's users, on the tenant's hosts and the APIs serving it | `tenant_id`, `account_id`, operator scope (ADR-0038) | that tenant's rows, under ADR-0115's predicates |
| **org** | an org's members, at the portal (paridyum.in) and the APIs serving it | `org_id` — the org the member chose, from their membership | that org's rows, and the registry rows of the tenants it owns |
| **platform** | Paridyum's staff, signed in at the staff instance (ADR-0119) | neither | **platform-scoped tables only** — the personless skeleton |

- **Derived from the verified session only**, never from a parameter (ADR-0115 D5).
- **No internal mode, no admin path, and no configuration disables the requirement.** A request that
  resolves no context is refused.
- **No request resolves two contexts.** A member acting inside one of their tenants holds a tenant
  context there, reached by a per-tenant grant (ADR-0120), not an org context that reaches in.
- **The platform context is safe by construction, not by policy.** It sets neither
  `paridyum.current_org()` nor `paridyum.current_tenant()`, and both raise when unset (ADR-0038), so
  every org-, tenant- and account-scoped predicate raises for it. The control-plane skeleton holds no
  person (ADR-0117 D3). Reaching into an org or a tenant from it is a grant the org or tenant sees and
  revokes (ADR-0120).

**2. Paridyum's own use is an org and a tenant.**

- **The Paridyum org is an org**, and **Paridyum's tenant** (tenant zero, C-0008) is a tenant it owns.
- Both are created through the paths any org and tenant use — ADR-0039's saga, or, until the portal
  runs it, the same provisioning tooling the saga wraps (ADR-0118 D3). Both are stored in the same
  silos and subject to the same entitlement and quota.
- **Neither holds a scope a customer could not also be granted.** GLO-0005 makes operator scope a
  grant rather than an absence; this names the principals for whom nobody would otherwise think to
  check. `domain.operator_tenant_holds_no_extra_grant`.
- **The platform context is not Paridyum's use.** Staff hold nothing inside the Paridyum org or
  Paridyum's tenant without the same grant any other org or tenant would issue.

**3. No deployable capability ships without a declared buyer.** A service's `manifest.yaml` names how
an org or a tenant buys what it does. A capability that cannot answer *which org or tenant would pay
for this* is not a platform capability — it is a product nobody has scoped or an internal tool
misfiled. `domain.no_operator_only_capability`.

**4. The boundary is the request path.** `tool-*` and `infra-*` are exempt: they run at build time or
in a pipeline, no customer traffic reaches them, and ADR-0042 classes such things `deployable: false`.
**If org or tenant data can reach it, it is in a request path and this ADR binds** — including
anything shaped like a support console or an internal dashboard, which is the case most likely to be
argued and most likely to be wrong. **A staff console is a platform-context surface**, and decision
1's last row is all it may read.

**5. A new runtime service names its buyer before it is created.** ADR-0019 requires decomposition
before implementation and escalates repository creation to a human; this adds one question to that
gate — *which org or tenant buys this, and through what interface?* An answer of "only us, for now"
is a decision to build a `tool-` instead.

## Consequences

**Accepted costs**

1. **The first capability is slower by the width of a tenant model**, as ADR-0049 said. Nothing can be
   demonstrated until tenancy, identity and entitlement exist.
2. **Paridyum inherits every limitation it sells**, and the pressure for an exception peaks during an
   incident affecting its own customers. Decision 2 is written to be checkable because that is the
   moment judgement is worst.
3. **Decision 3 will reject work that is genuinely useful.** Something will be needed internally, have
   no buyer, and not be build-time tooling. It becomes a product, is deferred, or is reclassified.
4. **A third context exists, and it will be pushed.** The platform context is the natural place to
   add "just one" read of a tenant's rows during an incident. The database refuses that read, and
   `domain.platform_context_reads_only_platform_tables` exists to refuse the code that asks.
5. **Staff see the skeleton.** Which orgs exist, which tenants each owns, their regions, plans and
   states. That is the landlord's ledger, and it is personless by ADR-0117 D3 — but it is not nothing,
   and RFC-0022's disclosure should say that Paridyum sees it.
6. **Four checks declared, none built.** Stated here rather than discovered in a gate run.

**Gained**

- **The portal acts for the party it serves.** No tenant pretends to be the portal, and no portal
  request can read a tenant's rows.
- **Staff power has a home that holds no customer data.** ADR-0100's arrangement — the provisioner
  inside Paridyum's own business — is gone, and so is the path from Paridyum's business into every
  other tenant's registry row.
- **Every context fails closed in the database**, not in a middleware ordering.

**Rejected alternatives**

- **The portal inside a tenant.** It satisfies ADR-0049 D1 as written, and it makes the portal a
  business and its customers that tenant's accounts. The operator rejected it for a first-class org.
- **Staff as members of the Paridyum org with a platform-wide grant.** It is decision 2's extra grant,
  taken by the one org most tempted to take it.
- **Staff operations only through `tool-*` pipelines.** Cleaner, and it leaves no way to answer an
  incident question without shell access to a database. A platform context that the database confines
  is narrower than a shell.
- **Keep it as a practice** (ADR-0049's own rejected alternative): the failure mode is silent and
  cumulative, and fails when BET-0005 needs it most.
- **Exempt the first product**: retrofitting is the rewrite, and product one sets the seams.

## Related

- **ADR-0049** — superseded; restated here · **C-0008** — the principle
- **ADR-0114** — the org · **ADR-0115** — the scopes and accessors · **ADR-0118** — the portal and
  genesis · **ADR-0119** — the staff instance · **ADR-0120** — grants into orgs and tenants
- **BET-0005** · **RFC-0022** D1, D5 · **C-0001** principles 1 and 6
- **ADR-0038** — identifiers from the session · **ADR-0039** — the saga · **ADR-0042** —
  `deployable: false` · **ADR-0019** — the design gate · **ADR-0014** — declared checks are honest
