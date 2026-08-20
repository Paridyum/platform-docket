---
id: ADR-0001
title: Polyrepo topology with knowledge vendored into every repository
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org, platform, service, library, tool, infra, spec]
applies_to: ["*"]
tags: [topology, git, github, knowledge-distribution]
breaking: true
enforced_by:
  - check: repo.name_matches_scheme
    severity: error
    rationale: A repo whose name does not declare its class cannot be routed by forge.
  - check: repo.has_vendored_knowledge
    severity: error
    args: { path: ".paridyum/knowledge", max_age_days: 30 }
    rationale: An agent working in a repo must be able to read the constitution from disk without network access.
  - check: repo.knowledge_snapshot_not_modified
    severity: error
    rationale: Vendored knowledge is read-only. Edits belong upstream in Paridyum/knowledge.
---

# ADR-0001 — Polyrepo topology with knowledge vendored into every repository

## Status

**LOCKED** as of 2026-08-06.

## Context

Paridyum needs a repository topology before any code is written, because the choice
determines how `tool-forge` works, how shared crates are versioned, and — most
importantly — how an AI agent working inside one repository discovers the rules that
bind it.

Two properties were weighed:

1. **Agent context locality.** Agents work inside a single repository checkout, often
   many in parallel, often without network access to sibling repositories. Any rule an
   agent must obey has to be readable from the checkout it already has.
2. **Independent service lifecycles.** Services should be deployable, versionable and
   ownable without coordinating a monorepo release.

A monorepo gives property 1 for free and makes property 2 harder. A polyrepo gives
property 2 for free and breaks property 1 — an agent in `svc-billing` cannot see the
constitution living in a different repository.

## Decision

**Polyrepo.** Every service, library, tool, platform component, spec bundle and
infrastructure module is its own GitHub repository under the `Paridyum` organisation.

**Knowledge is vendored, and redundancy is accepted as the cost of context locality.**
Every repository carries a read-only snapshot of the knowledge base at
`.paridyum/knowledge/`, written by `forge` at scaffold time and refreshed by
`forge knowledge sync`. The canonical source is the `Paridyum/knowledge` repository;
the vendored copy is a build artifact that happens to be committed.

Vendoring is deliberately **not** a git submodule. Submodules require a second fetch,
are frequently left uninitialised, and would produce exactly the failure this ADR
exists to prevent: an agent finding an empty directory where the constitution should be.

### Repository classes

| Prefix      | Contains                                  | Example              |
| ----------- | ----------------------------------------- | -------------------- |
| *(none)*    | The knowledge base itself — sui generis    | `knowledge`          |
| `platform-` | Platform services every product depends on | `platform-gateway`   |
| `svc-`      | Business capability services               | `svc-billing`        |
| `lib-`      | Shared Rust crates, no binaries            | `lib-core`           |
| `tool-`     | Developer-facing executables               | `tool-forge`         |
| `infra-`    | Deployment and cluster configuration       | `infra-kubernetes`   |
| `spec-`     | Interface contracts (OpenAPI, protobuf)    | `spec-openapi`       |
| `template-` | Scaffolding sources consumed by forge      | `template-service`   |

The knowledge base is the one repository with **no prefix**. The absence of a prefix is
itself the signal: it is not a member of any class, it is the thing that defines the
classes.

## Consequences

**Accepted costs**

- The same constitution text exists in N repositories. Staleness is now possible, so
  `repo.has_vendored_knowledge` enforces a freshness window (30 days) and
  `forge audit` sweeps the org for drift.
- Shared crates are consumed by git tag, not path. A breaking change to `lib-core`
  requires a coordinated bump across dependents; `forge graph` exists to find them.
- N CI configurations. Mitigated by generating all of them from `template-*` and
  reconciling with `forge update`.

**Gained**

- An agent cloning any single repository has the complete set of rules that bind it,
  offline, with zero setup.
- Services version and deploy independently.
- Repository name alone determines ownership, CI shape, and applicable knowledge scope.

**Rejected alternatives**

- *Monorepo now, split later* — best for solo velocity, but defers the agent-context
  problem rather than solving it, and the split is only "mechanical" until services
  have divergent histories.
- *Git submodule for knowledge* — correct in theory; uninitialised submodules in
  practice.
- *Fetch knowledge from network at agent start* — reintroduces a network dependency on
  the one artifact that must never be unavailable.
