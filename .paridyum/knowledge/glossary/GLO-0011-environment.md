---
id: GLO-0011
title: Environment — a deploy target, never the approval gate in front of it
type: glossary
status: accepted
created: 2026-08-19
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform, service]
applies_to: ["*"]
tags: [environment, deployment, ci, cd, cluster, vocabulary, adr-0021, rfc-0023]
breaking: false
enforced_by: []
---

# GLO-0011 — Environment

## Definition

**An environment is a deploy target: one Kubernetes cluster and the namespace a workload lands in.
It is named, it is enumerable, and today there is exactly one — `staging`.**

An environment is not the mechanism that decides *whether* a deploy proceeds. That mechanism is a
**deployment gate** — on GitHub Free, a GitHub Environment with a protection rule (ADR-0021). The
two are different axes that happen to share a vocabulary, and the shared vocabulary is what this
entry exists to break.

## Why this entry exists

Two artifacts used the word for the two different axes, and because no entry defined it, neither
noticed the other.

**ADR-0021 (`locked`)** describes gates: *"CI builds and pushes an image to GHCR on merge to `main`,
then deploys to `dev` automatically. `staging` and `prod` require manual approval through a GitHub
Environment."*

**RFC-0023 decision 3** describes a target: *"One cluster, and it is called `staging`. Not `dev`"* —
because naming it `dev` means acquiring metal produces `dev` and `prod` and nothing staging, so
something gets renamed, and a rename is a migration.

Read as one axis these contradict, and the contradiction would have surfaced at the first deploy
with a cluster already named. Read as two, they do not: ADR-0021 states what happens *when* a deploy
targets each name and never requires that a `dev` exist.

## The rule that follows

> **One environment named `staging`, gated on a human, satisfies ADR-0021's intent.**

The automatic path in ADR-0021 has no target, so it does not fire. This is ADR-0021
**under-satisfied, not contradicted** — which is the distinction that keeps the substrate legal
without superseding a locked, org-scoped artifact.

⚠️ **Corrected 2026-08-20. This entry originally read "gated manually" and named ADR-0021's GitHub
Environment approval as the mechanism. That mechanism is not available here**, and the claim was
inherited from ADR-0021 rather than checked. Verified against the API: an environment can be created
on a private repository in this organisation and it carries no protection rules, because required
reviewers on private repositories are a paid-plan feature and ADR-0004 puts the organisation on Free.
The refusal is recorded in ADR-0021's 2026-08-20 erratum.

**So the gate is a `workflow_dispatch` deploy that a person triggers, not an approval they grant.**
The distinction is not cosmetic and belongs in this entry rather than in the runbook that implements
it: a trigger records who *ran* a deploy, an approval records who *authorised* it, and only the
second is a second pair of eyes. Under ADR-0044 the audit trail must answer who caused a change;
"the person who clicked run" is the same person as the author, and that is the weaker answer.
Restoring the stronger one is a billing change rather than an engineering one.

⚠️ **The other accepted cost, which cuts against Phase 2's own thesis.** The harness must fail
before it is trusted, and ADR-0021's automatic post-merge deploy path is the one part of the
pipeline this arrangement never exercises. It stays unexercised until a second environment exists,
and it will first run against whatever that second environment is.

## What it is not

**It is not a deployment gate.** A gate answers *may this proceed*; an environment answers *where
does it land*. One environment can sit behind several gates, and a gate protects an environment
without being one.

**It is not a cluster name, quite.** RFC-0023 names one Talos cluster `staging` and one environment
`staging`, so today they coincide. They are still separate: RFC-0019 decision 3 forbids the platform
sharing a cluster with anything else, which means a second cluster on the same hypervisor is a
second environment, while a second *namespace* on the same cluster may or may not be — that is a
decision each case makes, not something the word settles.

**It is not `dev`.** No environment is called `dev` and none should be. RFC-0023 decision 3's
argument is the whole reason: the name has to survive metal arriving, and `dev` does not.

**It is not a region.** GLO-0008 and GLO-0007 own that axis. A region is a residency boundary and a
legal act; an environment is a deploy target and an operational one. Two environments in one region
is ordinary; one environment spanning two regions contradicts RFC-0015 decision 3.

## The list is data, and that fix is owed

Three environment names sit hardcoded inside a locked artifact, which is the defect class this
repository has already fixed three times — C-0002's repository classes moved to
`class-registry.yaml`, ADR-0003's prefix globs became ADR-0020's `deployable: true`, and RFC-0021's
family list became `device-registry.yaml`. C-0007 states the rule: **the grammar is constitutional,
the list is data.**

The same fix here is an `environment-registry.yaml` under ADR-0022, each entry declaring its cluster
and whether deploys to it are automatic — and it would require superseding ADR-0021, which ADR-0020
hoped would be the last supersession of its kind.

**Not done now, and deliberately.** With one environment the registry would hold one row and the
defect costs nothing. **Trigger: the day a second environment exists** — which is the day metal
arrives and `prod` is created. Doing it then is cheap; doing it after three services have hardcoded
a name is not.

This is ADR-0048's test applied honestly, and it returns the opposite answer to the toolchain case:
an environment list does not change on a cadence Paridyum does not control, so it is ADR-0022
registry data — a list that gets extended — rather than ADR-0048 registry data.

## Related

- ADR-0021 — the gates, and the `dev` this entry explains the absence of
- RFC-0023 — decision 3, the one cluster and the argument for its name
- RFC-0019 — decision 3, why a second cluster rather than a second namespace is the usual answer
- GLO-0008 · GLO-0007 — the region axis this one is repeatedly confused with
- GLO-0004 — the precedent: a word meaning two things is worth an entry
- ADR-0022 — registries carry lists that get extended, which is what the owed fix is
- ADR-0048 — decision 2's test, which this term fails and the toolchain pin passes
- C-0007 — the grammar is constitutional, the list is data
- ADR-0020 — the deployable classes that land in an environment
