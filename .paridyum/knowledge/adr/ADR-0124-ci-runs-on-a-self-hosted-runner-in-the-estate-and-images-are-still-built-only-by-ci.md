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
sources:
  - url: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows
    title: "GitHub Docs — Events that trigger workflows (pull_request GITHUB_SHA; schedule runs on the default branch)"
    retrieved: 2026-09-29
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
- ADR-0056 (locked 2026-10-04) — machines are declared in OpenTofu, **and what is true inside them is
  Ansible's.** This line cited only the first half until 2026-10-04; see the addendum of that date.
- RFC-0037 — addresses come from the registry.
- T-G.21 — watch the meter.
- infra-substrate `roles/ci_runner` and its `SOP.md` ("Applying the CI host") — the runner's install
  and registration. (infra-terraform's `SOP.md` and `ci-runner/install.sh` until 2026-10-04.)

## Addendum — 2026-09-29: the knowledge check suite runs when a change can reach it

**Accepted** 2026-09-29. The operator directed "research and go ahead".

**What forced it.** On this runner, `scripts/test-checks.sh` takes 40 to 90 minutes. On 2026-09-28 one
knowledge run took 85 minutes. Its 49 knowledge cases each copy the knowledge base and run
`check-knowledge.sh`, which takes 73 s here against 16 s on a workstation. The VM lost about 14% of
its CPU to steal from the cluster, and each process start is expensive there. Every knowledge pull
request and every push held one of the runner's two slots for that long, and 33 runs queued behind
them. Decision 2's accepted cost, *"jobs queue instead of running in parallel"*, turned out to be
most of the day.

**What the research found.** A rule of "checker code only" would be unsafe. The knowledge cases plant
each defect into a **real** artifact on **real** anchor text: `bets/BET-0005-…`, `adr/ADR-0024-…`,
`glossary/GLO-0011-…`, `TESTING.md`, the registries. Editing one of those looks like documentation,
yet breaks a case. The suite's other dependence on content is that the real knowledge base checks
clean, and the "Knowledge integrity" step proves that on every pull request anyway.

**Decision.**

1. **On a pull request, the suite runs when the change can reach it.** `scripts/check-suite-scope.sh`
   decides. It runs the suite for any of:
   - checker code or fixtures (`scripts/`, `templates/`, `plugins/`, `tests/`, `.github/`);
   - a root registry or schema (`*.yaml`, `*.json`);
   - `TESTING.md`;
   - any path under a directory that `test-checks.sh` names.

   The root `.md` files the suite mentions are synthetic scaffold files, so they do not count.
   Any doubt runs it: no input, a failed diff, or an internal error.
2. **It always runs on a push to main, and nightly** on main's latest commit, at 03:00 IST.
3. **The diff is exact.** On a pull request `GITHUB_SHA` is the merge commit (GitHub: *"the last merge
   commit of the pull request merge branch"*), so `HEAD^1..HEAD` is what the pull request changes
   against its base.
4. **The nightly run skips the identity and regression steps.** They gate a change as it arrives,
   and a scheduled run brings none.
5. **The scope rule is itself in the suite:** six cases, and the ones that matter are those that must
   *run* — for example, a planted artifact that looks like documentation.

**Measured.** Replayed over main's last 30 commits, the rule skips the suite on 17 and runs it on 13,
including every commit that touched a checker, a registry or a planted artifact.

**Accepted cost.** A documentation-only pull request that breaks a case in some way the rule does not
foresee is caught on the push to main, or overnight, rather than on the pull request. The repository
commits straight to main on the Free plan (ADR-0004), so "caught on main" is the same moment most
changes were caught before.

**Rejected.**

- **A `paths:` filter on the trigger.** It would skip every step, the integrity and secret checks
  included.
- **A third-party changed-files action.** A new supply-chain dependency (ADR-0010) for one `git diff`.
- **Parallelising the suite's cases.** Worth doing, and it does not conflict with this. It needs a
  knowledge-base copy per worker, and edits the estate's most safety-critical script. It is left to
  its own change.

## Addendum — 2026-10-04: the runner's inside is Ansible's (ADR-0056's other half)

This decision cited ADR-0056 as *"machines are declared in OpenTofu"* and pointed the runner's install
at infra-terraform's SOP. That is half of ADR-0056, which also says what is true on a machine is
Ansible's and rejects scripts for it by name. Read that way, the citation looked compliant to anyone
checking it, and `ci-runner/install.sh` — a script piped over SSH from the OpenTofu repository —
configured the runner for five days and passed every gate (`docs/2026-10-03-why-adr-0056-was-not-
followed.md`, cause 2).

Corrected: `ci-runner-01` **exists** by infra-terraform's OpenTofu, and **is configured** by
infra-substrate's `roles/ci_runner` (ADR-0131 item 4), which the second runner host (ADR-0131) shares.
The scripts are retired. Decisions D3 to D6 are unchanged; the egress firewall of D4 gained an IPv6 half
under ADR-0132. ADR-0056 is now locked and `repo.hosts_are_configured_only_by_ansible` rejects the
script shape in any repository.


## Addendum — 2026-10-04: the runner's disk returns the space it frees

Every guest on the Proxmox thin pool `pve/data` was declared with no `discard`, which the provider
reads as `ignore`: a block a guest frees is never returned to the pool, so each thin volume's
allocation only ratchets up. Measured on 2026-10-04: `ci-runner-01`'s 48 GB volume was **99.55%**
allocated while its filesystem had 12 GB free; it alone held 48 of the pool's ~95 GB used, and the
pool stood at **83.7%** of 112.9 GB, with 248 GB promised to volumes and LVM's thin-pool autoextend
off (`thin_pool_autoextend_threshold=100`). A full thin pool fails writes for every guest on it — the
staging control planes, workers and mail gateway, not only CI. The runner's churn (Docker layers,
cargo targets) is the workload most likely to get it there.

Decided: `ci-runner-01`'s disk is declared `discard = "on"` in infra-terraform. The guest already
mounts `/` with `discard` and runs `fstrim.timer`, so freed space now leaves the volume. The disk is
**not** grown: growing it to 64 GB would put the pool's worst case near 98%. If the runner is short of
disk again, the order is: discard (this), extend `pve/data` into the volume group's free 16 GB (an
irreversible storage change that needs a human under C-0004, configured by Ansible under ADR-0056),
and only then raise `runner_disk_gb`.

Open, not decided here: the staging nodes and the mail gateway carry the same `discard=ignore`.
Changing it restarts cluster nodes, so it is a separate change against infra-terraform.
