---
id: GLO-0005
title: Operator scope — the privileged read across every account of a tenant
type: glossary
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service]
applies_to: ["*"]
tags: [tenancy, isolation, security, authorisation, vocabulary]
breaking: false
verified_by:
  - TST-0018
enforced_by:
  - check: domain.containment_is_granted_never_inferred
    severity: error
    rationale: >
      This entry's whole content, stated as a check: operator scope is a GRANT, never an absence. A
      containment class derived from a plan name, a price, or the lack of a flag is exactly the
      inferred breadth this term exists to forbid.
---

# GLO-0005 — Operator scope

> **Superseded in part, 2026-09-28 — ADR-0113, ADR-0120.** Where this entry has an operator owning several
> tenants, read an **org member** holding a separate **per-tenant grant** in each (ADR-0120), never
> simultaneously in one session. The deferred support access is decided: it is the same grant, asked
> for and approved like anyone's (ADR-0120 D2, D9).

## Definition

**Operator scope is a session that may read every account within one tenant. It is a grant,
never an absence.**

It is what the operator's own console runs as: the person who built the product looking at all
their customers. It is legitimate, necessary, and the single most dangerous state in the
system.

## The rule

**Operator scope must be represented as an explicit positive claim in the session, verified per
request. It must never be represented by a missing, null, empty or wildcard `account_id`.**

The reason is the failure direction. If breadth of access is encoded as *the absence of a
restriction*, then every bug that loses a value — a dropped session field, an uninitialised
variable, a failed lookup that returns null, a middleware ordering mistake — **escalates
privilege silently and in the safest-looking way.** The query still runs. The rows still come
back. Row-level security is applied exactly as written, to the wrong scope. Nothing errors.

Encoded as a positive claim, the same bugs fail closed: no claim, no breadth, empty result set,
loud.

This is the same argument ADR-0023 made about `ENABLE` versus `FORCE` on row-level security,
and it has the same shape as the `superseded` immutability hole recorded in
`guard-locked-decisions.sh` — a guard that was narrower than the rule it enforced. A default
that grants is a guard that is narrower than it looks.

## What it is not

- **Not cross-tenant.** Operator scope is bounded by one tenant. An operator owning several
  tenants holds operator scope in each, separately, and never simultaneously in one session.
  There is no scope that spans tenants; support access is a different mechanism and a different
  decision (deferred, see ADR-0024).
- **Not an account role.** An administrator *of an account* administers that account. Operator
  scope sits above every account and belongs to the tenant's owner.
- **Not the database owner.** ADR-0024 requires the runtime role to be unable to bypass row-level
  security at all. Operator scope is expressed in the policy, not by escaping it.

## Related

- GLO-0004 nested tenancy — why this state has to exist
- GLO-0001 operator — who holds it
- GLO-0003 account — what it reads across
- ADR-0024 — the decision, and the check that operator scope is an explicit grant
