---
id: ADR-0124
title: CI runs on a self-hosted runner in the estate, and images are still built only by CI
type: adr
status: accepted
created: 2026-09-28
locked_at: null
supersedes: null
superseded_by: null
scope: [platform, infra, tool, service, library, web, spec]
applies_to: ["*"]
product: [paridyum]
tags: [ci, github-actions, self-hosted-runner, supply-chain, cost, proxmox, adr-0004, adr-0021, adr-0012, adr-0056]
breaking: false
enforced_by: []
---

# ADR-0124 — CI runs on a self-hosted runner in the estate, and images are still built only by CI

## Status

**Accepted** 2026-09-28. The operator directed "go for 2 or 4" of four options:

1. repair billing;
2. a self-hosted runner;
3. make repositories public;
4. build images on a workstation.

This records the choice of **2**, and why 4 lost.

It is not `locked`: "every workflow names the estate's runner labels" is checkable, but no check
asserts it yet.

## Context

**Every CI job in the estate has stopped starting.** GitHub's annotation on the failed jobs, read on
2026-09-28, says: *"The job was not started because recent account payments have failed or your
spending limit needs to be increased."* The organization is on the Free plan (ADR-0004) with private
repositories, whose included Actions minutes are capped — ADR-0004 named that cap as a consequence.
The effect is total:

- no gate runs on any pull request;
- no image is published, so nothing merged since 2026-09-15 can be released (`image-publish` is
  `needs: ci`, ADR-0021);
- the estate rebuilt from empty that day runs images older than its own configuration.

T-G.21 ("somebody watches the meter") existed to see this coming, and had not been finished.

**Two constraints bound the answer.**

- **The gate may not be bypassed.** An image is built only from a commit whose checks passed.
  `image-publish` is `needs: ci` because *"an image built from a commit whose tests failed is an
  image somebody eventually deploys and the registry keeps it forever."* Option 4 builds images
  outside CI, so it would need that rule suspended.
- **The operator's workstation is short of disk.** It had 251 MiB free that day. The substrate host
  is not short: 16 cores, and Proxmox had 11 GiB of memory and 70 GiB of storage free.

## Decision

1. **CI runs on a self-hosted GitHub Actions runner, a Proxmox guest in the estate.**
   - It is declared in infra-terraform's OpenTofu like every other machine (ADR-0056).
   - Hostname `ci-runner-01`, address `10.1.1.41`, vmid `1041` (address-registry.yaml).
   - Debian 13 from the image already on the Proxmox store, with Docker (buildx) for image builds.
   - It is registered to the organization with the labels `self-hosted`, `linux`, `x64`,
     `paridyum`.

2. **Every workflow runs on those labels.**
   - The reusable workflows in infra-ci name them, and moving `v1` moves every caller (ADR-0021).
   - A repository whose workflow names `runs-on` directly is edited to match.
   - GitHub-hosted runners stay possible for a job that needs one, by naming it; nothing depends on
     that.

3. **The rule that images come only from CI is unchanged.** The same workflows, the same `needs: ci`,
   the same `GITHUB_TOKEN`-scoped push to ghcr and the same provenance run on a different machine.

4. **The runner holds no estate credential.**
   - It has no kubeconfig, no SOPS key and no cloud credential. It deploys nothing; releases stay the
     operator's step.
   - Its only credential is its registration, which lets it run jobs and nothing else, plus whatever
     token a workflow hands a job.

5. **It is persistent, not ephemeral, and it cleans up after every job.**
   - An ephemeral runner needs a new registration token per job, and minting one takes an
     organization-admin credential. Keeping that on the runner would be a standing admin
     credential on a machine that executes pull-request code.
   - So the runner registers once. A job-completed hook removes the workspace and prunes Docker to a
     disk ceiling.

6. **Pull requests from forks never run on it.** Contributors work from forks (ADR-0012). The
   organization setting that runs fork pull-request workflows on private repositories stays off, and
   the runner group admits this organization's repositories only.

## Consequences

**Accepted costs.**

- **CI shares a hypervisor with the cluster.** A heavy build competes with the staging nodes for
  CPU; its memory is dedicated (6 GiB), and so is its disk.
- **It is one runner.** Jobs queue instead of running in parallel, and a runner that is down stops
  CI as completely as the billing did — but visibly, and fixable by us.
- **The runner runs pull-request code on the estate network.** Decision 4 keeps credentials off it;
  its network reach is not yet narrowed. A firewall that lets it reach GitHub, the package
  registries and the gateway, but not the cluster's addresses, is owed.
- **Persistent means state can leak between jobs.** Decision 5's cleanup is the mitigation, not a
  guarantee. Ephemeral runners through an in-cluster controller are the stronger design, once the
  cluster has room to build on.
- **Arch is amd64 only**, which is what the estate runs.

**Gained.**

- CI and releases work again, with no bill, and the supply-chain rule is intact.
- The minutes cap stops being an architectural constraint. ADR-0004's "knowledge checks must stay
  fast" remains good practice, no longer a budget.

**Rejected alternatives.**

- **(4) Build images on the workstation.** It suspends ADR-0021's rule that an image comes only
  from a green commit, and it puts the build on the machine with the least disk. It also makes an
  image's provenance "someone's laptop".
- **(1) Repair billing.** Still worth doing — hosted runners are a fallback — but it leaves the
  estate's delivery on a meter nothing watches (T-G.21). The operator chose 2 or 4.
- **(3) Make repositories public.** ADR-0004 rejected it for the knowledge base and the product
  repositories, and it would not unblock them.
- **A runner on the substrate host itself.** infra-substrate ends at the host and provisions nothing
  on it by shell (its ARCHITECTURE.md); the hypervisor is not a build machine.
- **Runners in the cluster** (actions-runner-controller). The two workers have 8 GiB each and already
  carry Postgres, Zitadel and the applications; builds would starve them. Revisit when the cluster
  grows.

## Related

- ADR-0004 — the Free plan, and the minutes cap it accepted.
- ADR-0021 — reusable workflows; `image-publish` is `needs: ci`.
- ADR-0012 — contributors fork.
- ADR-0056 — machines are declared in OpenTofu.
- RFC-0037 — addresses come from the registry.
- T-G.21 — watch the meter.
- infra-terraform `SOP.md` — the runner's install and registration.
