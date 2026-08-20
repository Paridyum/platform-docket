---
id: ADR-0039
title: Tenant provisioning is an asynchronous saga with per-step compensation and a distinct failure state
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, spec]
applies_to: ["platform-*", "svc-*", "spec-*"]
tags: [tenancy, saga, provisioning, compensation, idempotency, anti-0005, zitadel]
breaking: false
enforced_by:
  - check: domain.saga_steps_declare_compensation
    severity: error
    rationale: >
      ANTI-0005 requires a compensating path and does not require it to be written down, so
      "we'll compensate" has been satisfiable by intent. A step whose compensation is absent is
      discovered during the first partial failure, which is the worst possible moment to design
      one.
  - check: openapi.error_responses_declare_a_code
    severity: error
    rationale: >
      Provisioning fails in ways a caller must branch on — retryable, terminal, and
      residual-resources-may-exist are three different outcomes. Under ADR-0032 that branch is
      on a code, never on a message.
---

# ADR-0039 — Tenant provisioning is a compensating saga

## Status

**ACCEPTED** as of 2026-08-08. Resolves RFC-0009 Q3. Created `accepted` per `new-decision` §5b.

## Context

ANTI-0005 forbids a cross-service transaction and **requires** that a compensating path be
named. RFC-0009 §6 records that it does not name one — *"ANTI-0005 unsatisfied: no compensating
path for a half-created tenant"* — and §Report lists it as one of two questions blocking the
first repository.

Creating a tenant is irreducibly distributed. It touches `platform-tenancy` (the tenant row),
`platform-identity` (a Zitadel organization, its branding, its project grants and — for a
premium tenant — a custom domain), and eventually `platform-billing`. Two of those are external:
Zitadel's Management API, and DNS plus certificate issuance for a custom login domain. Neither participates in a transaction, and
certificate issuance can take minutes and fail for reasons no retry fixes.

The naive shape is a synchronous `POST /tenants` that calls three services and returns. It fails
the moment step three fails after steps one and two succeeded, and it fails worse when the
caller times out having no idea which steps completed.

## Decision

**1. `platform-tenancy` owns the saga.** The thing being created is a tenant; identity and
billing are its dependencies. Ownership follows the entity, per ANTI-0001.

**2. Tenant creation is asynchronous and the lifecycle is public.**

```
requested → provisioning → active
                 │
                 └──▶ compensating ──▶ failed
                              │
                              └──▶ compensation_failed ──▶ (reconciler) ──▶ failed
```

`POST /tenants` returns `202` with a tenant id. `GET /tenants/{id}` reports `status`. A caller
polls or subscribes; nothing blocks on certificate issuance.

**Status values are lowercase identifiers** — `compensation_failed`, not
`COMPENSATION_FAILED` — per ADR-0032 decision 1. `SCREAMING_SNAKE_CASE` is the error-code
vocabulary (STD-0010 §5), and a status is not an error code. The same word in the two contracts
takes two shapes.

**3. Every step declares a compensating operation, and compensation is not deletion.**

| Step | Forward | Compensating |
| --- | --- | --- |
| 1 | insert tenant row (with `region`) | mark `failed`; never hard delete |
| 2 | create Zitadel **organization** in the regional instance | delete organization — safe, it holds no user yet |
| 3 | apply branding and policy overrides | reset to instance defaults |
| 4 | grant entitled catalogue Projects to the org | revoke grants |
| 5 | *(premium only)* add custom domain to the instance, issue certificate | remove domain, revoke certificate |
| 6 | create billing customer | **deactivate, never delete** |
| 7 | mark `active` | mark `failed` |

Step 2 creates an **organization**, not a virtual instance — RFC-0012 moved to
organization-per-tenant on 2026-08-08, and step 5 is what a premium tenant buys: an additional
custom domain on the *same* regional instance, not an instance of its own.

Step 5 is the one that fails in ways retry does not fix. DNS propagation and ACME issuance are
outside our control and take minutes, which makes it both the reason this saga is asynchronous
and the likeliest source of `compensation_failed` in practice.

> **Compensate** means: return the resource to the safest valid state that semantically undoes
> the forward operation. For anything with financial, audit or legal weight, that is
> deactivation. `DELETE` is a legitimate compensation only where the resource provably carries
> no record anyone may later need.

**4. Every step is idempotent, keyed on a durable `provisioning_id`.** A crash between "the
remote call succeeded" and "we recorded that it succeeded" is the normal case, not the edge
case — it is what a timeout *is*. Without idempotency the retry creates a second organization and
the saga owns an orphan it does not know about.

**Verified 2026-08-08 — the mechanism is client-supplied identifiers, not an idempotency-key
header.** Zitadel's v2 `AddOrganization` accepts an optional `organizationId`, and admins may be
created with a supplied `user_id`. So the saga derives each remote id deterministically from the
`provisioning_id` — a UUIDv5 over it — and the first call creates while a retry returns
`AlreadyExists`, which the saga treats as success.

This is stronger than a key header in the case that matters most. An idempotency key must be
recorded *before* the call to be useful; a **derived** id can be recomputed from durable state
after a total crash, so a saga that lost its in-flight record can still discover whether the
resource exists. Zitadel's docs recommend omitting `organizationId` and letting the system
generate one — correct for interactive use, and exactly wrong for a saga, where the whole point
is that the id is knowable before the call succeeds.

**5. Saga state is durable and inspectable**, in `platform-tenancy`:

```
tenant_provisionings          -- paridyum:scope platform
  id, tenant_id, state, current_step, completed_steps[],
  failed_step, compensation_state, attempt_count,
  last_error_code, created_at, updated_at
```

`platform`-scoped deliberately: it is Paridyum's record of its own work, and it must be
writable and readable when the tenant row is `failed` or does not yet exist. A `tenant`-scoped
table here would require `paridyum.current_tenant()` to resolve for a tenant that is not yet
provisioned — a circular dependency on ADR-0038.

**6. `compensation_failed` is a distinct terminal state and it is operationally load-bearing.**

- `failed` — provisioning did not complete **and the system returned to a safe state.**
- `compensation_failed` — provisioning did not complete **and residual resources may still
  exist.**

The second is an alert. Most saga designs collapse them, and then a failed rollback is
indistinguishable from a clean one — which means orphaned Zitadel instances accumulate, each
holding a custom domain nobody released.

**7. A reconciler retries compensation with backoff, verifying remote state before acting.**
Convergence is eventual and must be, because the failure being recovered from is often the
remote system being unavailable. The reconciler reads Zitadel to determine what actually exists
rather than trusting `completed_steps`, which is what the crash may have corrupted.

**8. A tenant never becomes `active` through any path except step 6.** No partial success, no
"mostly provisioned", no inference from the presence of an instance.

## Consequences

**Accepted costs**

1. **The API is harder to consume.** Every client must handle an asynchronous lifecycle for what
   reads like a create. This is a real cost paid by every integrator forever, in exchange for
   not lying about an operation that genuinely takes minutes.

2. **A reconciler is a background process that must itself be operated** — monitored, alerted,
   and able to be run manually during an incident. It is new operational surface whose failure
   is silent by nature.

3. **Idempotency requires the remote to cooperate.** Zitadel's System API is the dependency here
   and its idempotency semantics are ours to verify, not assume. Where it does not cooperate,
   the saga must query-before-create, which is a race rather than a guarantee.

4. **`compensation_failed` will fire and will need a human.** Building the state is easy;
   staffing the response is the part that decays. A state nobody watches is `failed` with extra
   steps.

5. **Certificate issuance is a step we do not control the duration of.** ACME challenges depend
   on DNS propagation. The saga must tolerate a step that takes minutes and legitimately fails
   for reasons retry does not fix — which makes step 4 the most likely source of
   `compensation_failed` in practice.

**Gained**

- ANTI-0005 is satisfied by an artifact rather than by intention, which closes one of the two
  questions blocking `platform-tenancy`.
- A half-created tenant becomes a state with a name, a record and a recovery path, instead of an
  incident.
- The same machinery serves de-provisioning, region migration and tenant deletion — all of which
  are the same problem with a different step list.

**Rejected alternatives**

- **A distributed transaction / two-phase commit across services.** Forbidden by ANTI-0005, and
  unavailable anyway: Zitadel and an ACME issuer are not transaction participants.

- **Synchronous `POST /tenants` that calls three services.** Simplest to write and it makes the
  timeout the caller's problem while leaving the residue ours. It also cannot express step 4 at
  all.

- **Best-effort cleanup with no durable state.** "Try to undo it, log if it fails." This is what
  most systems do and it is why orphaned resources are a universal operational tax. The residue
  is invisible precisely when it matters.

- **One collapsed `failed` state.** Cheaper, and it discards the only signal that distinguishes
  a clean rollback from one that left a live Zitadel instance and a bound domain behind.

- **Orchestrate from an API gateway or a workflow engine outside the domain.** Puts the tenant
  lifecycle somewhere that does not own the tenant, which is ANTI-0001 with extra infrastructure.

## Related

- ANTI-0005 — the rule this satisfies; RFC-0009 §6 recorded it unsatisfied
- RFC-0009 — Q3, which this resolves
- ANTI-0001 — ownership follows the entity, which is why `platform-tenancy` orchestrates
- ADR-0032 — status values are lowercase identifiers; errors carry codes
- STD-0010 §5 — the error envelope callers branch on
- ADR-0038 — why the saga table is `platform`-scoped
- RFC-0012 — the Zitadel resources steps 2–4 create
- STD-0006 — the migration conventions the saga table follows
- REQ-0005 — events; a provisioning event stream is the natural subscriber path
