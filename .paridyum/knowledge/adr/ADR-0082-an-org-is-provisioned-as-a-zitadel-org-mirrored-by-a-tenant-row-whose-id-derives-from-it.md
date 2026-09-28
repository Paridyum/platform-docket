---
id: ADR-0082
title: An org is provisioned as a Zitadel org mirrored by a tenant row, whose id derives from the org
type: adr
status: accepted
created: 2026-09-12
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, service, infra]
applies_to: ["*"]
product: [tenancy]
tags: [tenancy, identity, zitadel, provisioning, org, users, uuid, adr-0038, adr-0047, adr-0049, adr-0078, c-0008, req-0022]
breaking: false
enforced_by: []
---

# ADR-0082 — An org is provisioned as a Zitadel org mirrored by a tenant row, whose id derives from the org

> **Superseded in part, 2026-09-28 — ADR-0113.** In this ADR, **"org" means the tenant's Zitadel organization** —
> the vendor object — and not an Org (GLO-0012). In code it is `zitadel_org` / `zitadel_org_id`
> (ADR-0113 D2). The derivation rule stands. **The example ids below** (Paridyum's tenant and its
> Zitadel organization) **belong to the estate before its 2026-09-28 rebuild from empty** (ADR-0118 D3);
> the rebuilt tenant has new ones.
>
> The tool is now `provision-tenant.py`, run as `make tenant-plan` / `make tenant-provision`; `provision-org.py` and the `org-*` targets were renamed on 2026-09-28 (ADR-0113 D4) and no longer exist.

## Status

**Accepted**, 2026-09-12. This is what makes tenancy *usable* rather than merely modelled, and its
first consumer is Paridyum, which is the first tenant (C-0008, ADR-0049) — not a hypothetical one.
Not `locked`: the provisioning path is an operator-run tool, not yet a check; it graduates when a
check asserts every Zitadel org the registry declares has a matching `tenants` row and vice versa.

## Context

Everything downstream of a login assumes a tenant, and until now nothing created one. Three facts
collided the moment we tried to actually *be* the first tenant:

1. **The tenant id in the database is a `uuid`.** `tenants.id` is `uuid`, the accessors return
   `uuid`, `current_tenant()` casts the session setting `::uuid`, and every tenanted table's
   `tenant_id` is a `uuid` foreign key (ADR-0024, migration 0001).

2. **The tenant id in the token is a Zitadel snowflake.** ADR-0047 makes a tenant a Zitadel
   organization and ADR-0038 sources `paridyum.tenant_id` from the `urn:zitadel:iam:user:resourceowner:id`
   claim — which is the org id, a numeric snowflake like `389623583590057140`, **not a uuid**.
   `'389623583590057140'::uuid` throws. `lib-auth-typescript` even asserts the value is a uuid and would reject
   the raw claim. This never fired only because the tables are empty behind the ADR-0071 gate; the
   first real login-and-write would hit it.

3. **The `tenants` table is empty.** `principals.tenant_id` references `tenants(id)`, so
   `resolve_account` — the find-or-create every first write runs — would fail a foreign key the
   instant a real user acted, because no row represents their org.

ADR-0038 anticipated (1)↔(2) and called the fix "a lookup" on the hot path, and noted a custom
claim as the alternative. A lookup has a cost this estate cannot pay yet: it puts a **database read
on the login path**, and the database is gated behind ADR-0071 — login must work before the store
holds real data, so login must not require the store. A custom claim needs a Zitadel Action per org.
Both are heavier than the situation needs.

## Decision

1. **A tenant's id is derived from its Zitadel org id, deterministically:**
   `tenant_id = uuidv5(namespace, zitadel_org_id)` with the fixed namespace
   `caed8397-80a3-4860-8247-df68b7de8fd3`. The derivation is pure — the app computes it in-process at
   login from the `resourceowner` claim, with **no database read and no dependency** (a ~15-line
   `uuidv5` over `node:crypto`), and provisioning computes the identical value (Python's `uuid.uuid5`)
   when it seeds the row. Same namespace, same org id, same uuid, both sides, forever. Paridyum's is
   `68445ee9-175a-58a9-834e-a2f0c9c9b124` (from org `389623583590057140`).

2. **`tenants` carries the org id as a column, and its `id` is derived rather than minted.**
   `zitadel_org_id text NOT NULL UNIQUE` records the natural key; `tenants.id` no longer defaults to
   `uuidv7()` because it is not a minted surrogate — it is a function of the org id. This is a
   **documented opt-out of ADR-0078**: 0078's rule is for keys the schema mints, and a tenant's
   identity is not ours to mint — it is the Zitadel org's, and deriving it is what lets a login
   resolve the tenant without asking the database. Every other surrogate (principals, profiles,
   applications, and a minted `account_id`) is still `uuidv7()`.

3. **Provisioning an org is one operator-run, registry-declared, idempotent act.** It (a) ensures the
   Zitadel organization exists (ADR-0047, one org per tenant), (b) seeds the `tenants` row with the
   derived id and the org id, (c) creates the human users the registry declares, and (d) grants their
   roles. Declared in a registry and reconciled, the way OIDC clients already are (ADR-0070) — a
   tenant nobody can diff or count is a tenant nobody governs.

4. **Users are Zitadel users; the database does not become a second identity store.** ANTI-0001. The
   `principals` table maps a verified subject to an account and nothing more; it is not a user
   directory. A human's password is never set by the provisioning tool — the tool creates the user
   and Zitadel's init flow lets the human set their own password into their password manager
   (RUN-0004, STD-0017). A machine identity gets a key handled the same way a secret is.

5. **A role is a grant in a project (ADR-0070), and the session reads it.** An operator is a user
   holding the `operator` role in `paridyum-operations`; the token carries it and the session sets
   `operator` from it. Granting the role is part of provisioning, declared in the registry.

6. **Paridyum uses this exact path, first and unprivileged (ADR-0049 D2).** The Paridyum org is
   provisioned by the same tool, its `tenants` row derived by the same rule, its operator user
   created the same way. There is no god-tenant and no bootstrap shortcut that a customer tenant
   could not also be given.

7. **An unprovisioned org fails closed at login.** If a verified token's org has no `tenants` row,
   the user's tenant is not provisioned; login denies with a clear reason, never a uuid-cast error and
   never a silent empty context. Because the id derivation needs no lookup, this check is the only
   place login touches tenancy state, and it can be deferred to the first write (where the FK already
   enforces it) while the store is gated.

## Consequences

- **Migration 0005** adds `tenants.zitadel_org_id text NOT NULL UNIQUE` and drops the `uuidv7()`
  default on `tenants.id`. Reversible (ADR-0081); the tables are empty so there is no backfill.
- **A provisioning tool and an org registry** live beside the Zitadel app provisioner. `make
  org-provision` reconciles the org, the `tenants` row, the users and the grants; a plan mode writes
  nothing.
- **The web callback derives `tenant_id = uuidv5(ns, orgIdClaim)`** and stores that uuid in the
  session, replacing the raw snowflake. `lib-auth-typescript`'s `tenantId` is the raw org id and is renamed to
  `orgId` when the library is next versioned; until then the app derives from `principal.tenantId`
  and the mismatch is contained to one line with a comment.
- **Paridyum is seeded**: its `tenants` row and its first operator user. Once seeded, a Paridyum user
  logs in, resolves to tenant `68445ee9-…`, and `resolve_account` succeeds — the FK that would have
  failed now has its row.
- The residual `token-inspector` machine user in the Paridyum org (left from token debugging) is
  removed as part of getting the org to a declared state (ANTI-0010: nothing undeclared survives).

## Related

- **ADR-0038** — the session-context contract; this settles its "iss → tenant_id" question as a
  derivation rather than a lookup, for the reason 0038 could not: login must not read the store.
- **ADR-0047** — one Zitadel org per tenant; the org this mirrors.
- **ADR-0078** — the uuidv7 surrogate rule this documents an opt-out of, for `tenants.id` only.
- **ADR-0049 / C-0008** — Paridyum is the first tenant and uses this path unprivileged.
- **ADR-0070** — projects as role vocabularies; how the operator role is granted.
- **REQ-0022** — tenancy as a capability; provisioning is how a tenant comes to exist to use it.
