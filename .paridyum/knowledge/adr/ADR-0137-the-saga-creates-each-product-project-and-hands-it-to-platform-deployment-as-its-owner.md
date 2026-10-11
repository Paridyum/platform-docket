---
id: ADR-0137
title: The saga creates each product's project and hands it to platform-deployment as that project's owner
type: adr
status: accepted
created: 2026-10-09
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform]
applies_to: ["*"]
product: [tenancy, identity]
tags: [saga, zitadel, least-privilege, service-accounts, projects, adr-0039, adr-0126, adr-0070, x2]
breaking: false
enforced_by: []
---

# ADR-0137 — The saga creates each product's project and hands it to platform-deployment as that project's owner

## Status

Accepted, 2026-10-09, by the operator, who chose this over admitting platform-deployment as each tenant
organization's owner. It records a privilege that was decided only in code until now.

## Context

The saga (platform-tenancy, ADR-0039 D1) creates a tenant's Zitadel organization. platform-deployment
then creates a Release per product, including that product's project and sign-in client in the
tenant's organization (ADR-0126). To let deployment do that, the saga admitted its service account,
`deployment-provisioner`, to each tenant's organization as `ORG_PROJECT_CREATOR`. That rested on an
assumption, noted on 2026-09-30 to verify on the first run: that Zitadel makes a project's creator its
owner.

**It does not, in v4.15.3.** `cmd/defaults.yaml` gives `ORG_PROJECT_CREATOR` the permission
`project.create`, and no `project.read`, `project.write`, `project.role.write` or `project.app.write`. X2
(2026-10-09) measured the consequence:
- deployment created `paridyum-paridyum` in tenant-a2's organization, with no members;
- adding its roles failed;
- every retry's search could not see the project, and creating it again answered 409.

The Release step loops forever.

The org role that would let deployment work alone is `ORG_OWNER`: 55 permissions, including managing every
user of that tenant. The project-scoped `PROJECT_OWNER` holds what deployment needs (project, roles,
apps) inside one project. The saga's account, `tenancy-provisioner` (`IAM_ORG_MANAGER`), can create
projects and add their members.

## Decision

1. **The saga creates each enabled product's project** in the tenant's organization, as
   `tenancy-provisioner`, in a step `projects` between `identity_provider` and `releases`.
   - Named `paridyum-<product>`, as platform-deployment's catalogue and the `owner_access` step already
     name it.
   - With role assertion on.
   - It is found by name before it is created, so a retry finds it (ADR-0039 D4).
2. **The saga makes `deployment-provisioner` the `PROJECT_OWNER` of each of those projects, and of nothing
   else.** Its organization-level admission (`ORG_PROJECT_CREATOR`) is withdrawn.
3. **platform-deployment never creates a project.** It finds the project it owns by name, then ensures its
   roles and its sign-in client inside it, as before. A project it cannot find is a saga error (the
   `projects` step did not run), not something deployment repairs by creating one.
4. **Compensation is unchanged.** Deactivating the tenant's organization deactivates its projects, and
   releases fail as before. No project is deleted (ADR-0039 D3).

## Consequences

- **deployment's blast radius is one tenant's product projects**, and within them its sign-in clients
  and grants. If its key leaks, no tenant's users and no organization settings are exposed.
- **The saga gains a step, and tenancy a write to the tenant instance.** It already holds
  `IAM_ORG_MANAGER` there, so no privilege is added.
- **An orphaned project from before this decision is adopted, not replaced.** The step finds it by name
  and admits deployment as its owner, which is how tenant-a2 (X2) recovers.
- **Verified, not assumed:** the next X2 run is the proof. The assumption this replaces went unverified
  for ten days.

## Alternatives considered

1. **`ORG_OWNER` for `deployment-provisioner` in each tenant's organization.** Rejected by the operator: it
   would let deployment manage every user of every tenant.
2. **deployment adds itself as the project's owner after creating it.** Impossible: a creator without
   membership holds no `project.member.write`.
3. **`ORG_PROJECT_CREATOR` plus `ORG_PROJECT_PERMISSION_EDITOR`.** Rejected: that grants read, not write,
   so roles and clients still could not be created.

## Related

ADR-0039 D1, D3, D4 (the saga, compensation, derived ids); ADR-0126 (per-tenant clients); ADR-0070 D2 (a
project is a role vocabulary); ADR-0130 D5 (`owner_access` names projects the same way); PLAN-TENANT-HOSTS
§6 (X2).
