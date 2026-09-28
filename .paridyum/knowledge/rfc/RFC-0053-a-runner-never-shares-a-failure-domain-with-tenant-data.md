---
id: RFC-0053
title: CI executes on our own substrate, and a runner never shares a failure domain with tenant data
type: rfc
status: proposed
created: 2026-09-25
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [ci, github-actions, self-hosted, runner, containment, isolation, supply-chain, adr-0021, adr-0024, adr-0103, glo-0004, rfc-0014, rfc-0019, rfc-0045, req-0015]
breaking: false
---

# RFC-0053 — CI executes on our own substrate, and a runner never shares a failure domain with tenant data

## Problem

**On 2026-09-24 at 12:34 UTC every GitHub Actions run in this organisation began failing at
startup.** Four seconds, zero steps, no logs — the signature of a run the service refuses to begin
rather than a test that fails. It hit every repository, including two that had been green an hour
earlier, and it hit the `publish` workflows that merges to `main` trigger. `knowledge` had been
failing the same way since 2026-09-23 22:05 and nobody noticed, because nobody was reading a gate
that had already stopped running.

**The cause is still unconfirmed.** The org billing endpoint returns `410 This endpoint has been
moved`, and the user-scoped one needs a scope this account's token does not carry. A spending limit
is the most likely explanation for a simultaneous org-wide startup failure, and it is a hypothesis,
not a finding. **Slice 1 of this RFC is to stop guessing.**

> ### ✔ CONFIRMED 2026-09-26 — the included Actions allowance is exhausted
>
> **Added as an erratum (ADR-0031), not by editing the paragraph above:** the sentence "the cause is
> still unconfirmed" was true when written, and the record of what was known at the time is part of
> the argument. It is now confirmed, and the hypothesis was right.
>
> The `410` endpoint has a successor. `GET /orgs/Paridyum/settings/billing/usage` answers, and for
> **September 2026 it reports `Actions Linux` at exactly `2000.0` minutes** with
> `grossAmount == discountAmount` and `netAmount: 0.0` — the whole included allowance consumed and
> nothing billed beyond it. GitHub Free for an organisation includes 2,000 Actions minutes a month;
> `2000.0` to one decimal place is a ceiling, not a coincidence.
>
> ```sh
> gh api "/orgs/Paridyum/settings/billing/usage?year=2026&month=9"
> ```
>
> **The last day with any recorded Actions usage is 2026-09-20**, which burned 641 minutes. So the
> allowance was exhausted four days before anyone noticed the symptom, and the "12:34 UTC on
> 2026-09-24" in the heading above is when a person read a run page — not when CI stopped. Nothing
> was watching the meter, which is problem (2) of the three, demonstrated rather than argued.
>
> **~~The storage allowance went the same way.~~ Withdrawn 2026-09-26 — see the correction below.**
> This read: *"September's `Actions storage` rows are the first in this organisation's history with a
> non-zero `netAmount` (`tool-docket`, 14.14 GiB-hours), so the included storage is also spent."*
>
> **That is not supportable, and the error is instructive.** It came from calling the endpoint with
> **no** `?year=&month=`, which returns month rollup rows — one of which showed `Actions storage` for
> `tool-docket` with `discountAmount: 0` and a non-zero `netAmount`. Called **with** the filter, the
> daily rows for the same month sum to `discount == gross` **exactly**, for every SKU including
> storage. Two views of one endpoint disagree about one month.
>
> The filtered view is the one to trust: a per-day series that sums to a stated total is auditable,
> and a rollup that contradicts it is not. `infra-ci/scripts/check-included-minutes.sh` uses the
> filtered view and carries this discrepancy in its header, so the next reader meets it rather than
> rediscovering it.
>
> **The minutes finding is unaffected** — `2000.0` of 2,000 to one decimal place, independently
> corroborated by the symptom — and the diagnosis does not depend on storage. Written as a withdrawal
> rather than a deletion because an erratum that can quietly erase itself is not a record (ADR-0031),
> and because the *shape* of the mistake is worth keeping: an aggregate read once, believed, and
> published inside an hour.
>
> **What this changes, and what it does not.** Problem (1) — CI has exactly one path — is unchanged
> and is what this RFC answers. Problem (2) is now specific: the meter is 2,000 minutes a month, it
> resets on the 1st, and nothing reads it. **A check is owed for that and it is cheaper than a
> runner:** `gh api …/billing/usage` is one call, and a warning at 80% of the allowance would have
> turned a four-day silent outage into a Monday morning note. That does not replace self-hosting —
> an allowance that resets monthly is still a single path with a cliff in it — but it is the part
> that could ship today.
>
> **Three ways out, in ascending cost.** (a) Wait for 2026-10-01, when the allowance resets — free,
> and it leaves the estate exactly as fragile. (b) Attach a payment method and set a spending limit,
> which is money and a decision only the operator makes. (c) This RFC. Self-hosted runners consume
> **no** included minutes, which is the whole reason the operator raised "unlimited minutes" in the
> first place.

**What it cost, stated plainly because it is the argument.** Fourteen pull requests were merged on
2026-09-24 with no gate having run against any of them — including a tenancy migration correction
that exists precisely because an earlier change went in unreviewed. The estate's answer to "is this
safe" became "revert is cheap", which is true and is not the same thing.

Three separate problems are visible in that paragraph, and only one of them is about money:

1. **CI has exactly one path.** When it stops, there is no second one. Nothing in the estate degrades
   to a slower or narrower gate; it degrades to no gate.
2. **The meter is someone else's and nobody watches it.** ADR-0002 calls the CI gate the lock.
   A lock that opens when an invoice lapses is a lock with an undocumented key.
3. **A gate that stops running looks exactly like a gate that passes,** if what you read is a
   document rather than a run. That is the failure `TASKS.md` T-G.4 already recorded once.

Self-hosted runners answer (1) and (2). They do not answer (3), and this RFC should not pretend
otherwise.

## Constraints

**ADR-0021 is `locked` and silent on where jobs run.** It fixes CI as a reusable workflow per stack,
called by every class declared `ci`. It says nothing about `runs-on`. So this is a gap rather than
an amendment, and nothing here contradicts a locked artifact. That is worth stating up front because
it is the first thing a reader will check.

**ADR-0024 is `locked` and GLO-0004 prices the failure.** Crossing the tenant boundary is
"catastrophic, reportable"; crossing the account boundary is "catastrophic for that business, and it
is their reputation, not ours". The cluster that would most conveniently host a runner is the one
holding `docket_persons`, the four sealed chains, Zitadel, Garage and CNPG.

**ADR-0103 is `accepted` and supplies both the vocabulary and an obligation.** Containment is a
placement attribute; a named rung is an operable rung; and decision 3 states the rule this RFC leans
on — *"We are our own first buyer of every rung we name."*

**RFC-0019 fixes the substrate.** One Proxmox cluster, Kubernetes on virtual machines above it. The
capacity question is therefore "which VMs", not "what hardware".

**Region `in` is the only `status: live` region and is `control: true`.** Everything provisioned
today lands in one jurisdiction, which makes RFC-0014's plane question (§Open questions 2) real
rather than academic.

**The operator population is one person deep.** Every consequence below is priced against that, and
it is the strongest argument for the smallest thing that works.

## Proposal

### A. A runner is a machine that executes other people's code, and the estate should say so

Not "a build server". A runner is a process that fetches a revision it did not write, runs whatever
that revision says to run, and holds credentials while doing it. Every decision below follows from
taking that sentence literally.

The reason this framing is load-bearing rather than rhetorical: the convenient design — a long-lived
runner in the existing cluster, with a cached workspace and a token that can reach the registry —
is *exactly* the shape that turns one bad revision into estate-wide compromise. Convenience and the
threat model point in opposite directions here, and naming the machine correctly is what keeps the
argument from drifting.

### B. Runners run in a failure domain that holds no tenant data

The rungs, deliberately named to echo ADR-0103's ladder without claiming to be it:

| rung | shape | shares with tenant data |
| --- | --- | --- |
| **R0** | runners in the existing cluster, default namespace | kernel, API server, network, node |
| **R1** | dedicated namespace, NetworkPolicy, non-privileged | kernel, API server, node |
| **R2** | dedicated Kubernetes on dedicated Proxmox VMs | nothing but the Proxmox host |
| **R3** | one ephemeral VM per job | nothing but the Proxmox host |

**This RFC proposes R2.**

R0 and R1 are rejected on the same ground: a namespace is not a security boundary against a job that
can run arbitrary code, and both leave a compromised build one `kubectl` — or one kernel bug — from
the sealed chains. R3 is stronger and is not proposed *yet*: per-job VMs cost minutes of start-up
latency and a provisioning path that does not exist, and no contract requires it today. §Open
questions 3 records what would change that.

R2's residual risk is the Proxmox host itself, and it is named here rather than left implied.

### C. Every runner is ephemeral — one job, then destroyed

A persistent runner is a laundering machine: job A leaves a workspace, a cache and possibly a
credential that job B inherits, and the two jobs may come from different revisions with different
authors. Actions Runner Controller's runner scale sets do one-job-per-pod natively, so this costs
configuration rather than engineering.

**This is the single non-negotiable.** Every other control below reduces blast radius; this one
prevents the compromise from persisting at all.

### D. No privileged containers, and images build rootless

`publish.yml` in several repositories builds and pushes container images on a push to `main`. The
default way to do that inside Kubernetes is Docker-in-Docker with `privileged: true`, which is
equivalent to root on the node.

Images build with **rootless BuildKit or Kaniko**. If a stack cannot build rootless, that stack's
publish job stays on GitHub-hosted runners rather than earning a privileged pod.

### E. Egress is deny-by-default, and the policy is the artifact

A runner must not resolve or reach the tenant data plane — Postgres, Garage, Zitadel. Expressed as a
`NetworkPolicy` with a default-deny egress and an explicit allowlist (GitHub, the package
registries, the internal cache). **Written as policy, never as convention**, because a convention is
not something `verify-isolation.py` can read.

### F. No self-hosted runner on a repository that accepts contributions from outside the org

Every repository is private today, so this rule costs nothing to adopt and is worthless to adopt
later: the day the first repository opens is the day a stranger's pull request would otherwise run
code on our substrate. GitHub's own guidance is unusually blunt on this, and the rule belongs in the
estate's own words before it is load-bearing.

### G. CI keeps two paths, and the repositories that manage the runners stay on the other one

The trap this avoids is circular: if `infra-kubernetes`'s CI runs on a cluster `infra-kubernetes`
deploys, then a bad change removes the ability to ship its own fix. 2026-09-24 is the general form
of that failure already — one path, and when it stopped there was no gate at all.

So: **`infra-*` repositories remain GitHub-hosted**, and GitHub-hosted stays configured everywhere
as break-glass. The point of self-hosting is to stop being *metered*, not to stop being *able*.

### H. The cache is a mounted volume, not a long-lived runner

The real speed win is a warm Cargo cache, and the tempting way to get it is a runner that survives
between jobs — which is precisely what §C forbids. Instead the cache is an external volume, mounted
read-mostly into ephemeral runners, with writes going to a separate layer that a job cannot use to
reach the next one.

This keeps `coverage` and `test` fast without reintroducing the thing §C exists to prevent.

## Alternatives considered

- **Raise the spending limit and change nothing.** Necessary immediately and insufficient as an
  answer: it restores the gate without giving the estate a second path, and leaves a meter nobody
  monitors. This RFC's slice 2 does it anyway — it is the unblock, not the decision.
- **Runners inside the existing cluster (R0/R1).** Cheapest by a wide margin, and rejected in §B.
  Worth recording that it would work fine right up until it did not.
- **One ephemeral VM per job (R3).** Strongest isolation. Deferred, not rejected — §Open questions 3.
- **A different hosted CI provider.** Trades one metered third party for another, reopens a `locked`
  ADR-0021, and does nothing about having a single path.
- **Self-host everything, retire GitHub-hosted.** Rejected by §G: it is the circular dependency with
  extra steps, and it would have made 2026-09-24 unrecoverable rather than merely expensive.

## Consequences

**Accepted costs**

1. **A second Kubernetes cluster to patch, monitor and restore**, owned by a team of one. This is the
   real price and it is not small.
2. **A security boundary that becomes ours.** Today a runner compromise is GitHub's blast radius.
   After this it is our blast radius, bounded by controls we wrote and must keep honest.
3. **Runner images become a maintained artifact** — another thing that drifts, another thing to pin.
4. **The estate takes on this work while it cannot currently validate anything.** Adding
   infrastructure during an outage is a recognised way to compound one, which is why slices 1 and 2
   precede every other slice.

**Gained**

- **CI stops being metered**, and the failure mode of 2026-09-24 stops being possible for the same
  reason.
- **A second path**, so the gate degrades rather than disappears.
- **A rehearsal of operating an isolated failure domain, with Paridyum as the tenant** — which is
  what ADR-0103 D3 requires *before* a dedicated rung may be named publicly. This is the cheapest
  possible first buyer: if we cannot run an isolated cluster for our own builds, we have learned
  something important before a customer pays to learn it for us.

## Rollout

1. ~~**Establish why CI stopped, and record it.**~~ **Done 2026-09-26** — the included 2,000-minute
   Actions allowance is exhausted; see the erratum under Problem for the evidence and the exact API
   call. The `workflow_dispatch` test described here was never needed: the billing endpoint's
   successor answers directly.
   **1b (new, and it ships before any runner).** A check that reads the allowance and warns before it
   is spent. `ci.included_minutes_are_watched` — one `gh api` call, and the reason it is slice 1b
   rather than a footnote is that the outage lasted four days because nothing read the meter.
2. **Restore CI, then re-validate the fourteen pull requests merged unvalidated on 2026-09-24.**
   Reverting is available and cheap; knowing whether it is *needed* is not, until the gate runs.
3. **Provision the runner VMs and cluster, inert.** Nothing points at it yet.
4. **One repository end to end**, chosen for low blast radius — `tool-forge` or `console-testing`.
5. **Prove the controls, not the happy path**: a job that tries to reach Postgres fails; a job that
   requests a privileged container is refused; a second job cannot read the first one's workspace.
6. **Migrate by stack**, `infra-*` excepted per §G.
7. **Write the checks**, because ADR-0030 requires a binding rule to name its verification:
   `ci.runner_is_ephemeral`, `ci.runner_has_no_tenant_plane_egress`,
   `ci.no_self_hosted_runner_on_an_open_repository`, `ci.images_build_unprivileged`.

## Open questions

1. ~~**What actually caused the 2026-09-24 outage?**~~ **Closed 2026-09-26: the included 2,000-minute
   Actions allowance was exhausted on 2026-09-20.** It was a spending ceiling, so the premise does
   not narrow — but it sharpens: the ceiling *resets monthly*, so CI returns on 2026-10-01 without
   anyone doing anything, and returns to being a single path with a cliff in it. See the erratum
   under Problem.
2. **Which plane does the runner cluster live in?** RFC-0014 separates *whose* a row is from *where*
   it lives. A runner holds no tenant rows, which suggests control-plane; but it builds artifacts
   that will run in a region, and region `in` is currently both. Unanswered here deliberately.
3. **What would trigger R3?** A candidate: the first tenant contract that names a containment rung
   above C0 and whose auditor asks where its artifacts were built. Worth deciding before it is asked.
4. **Do D-69 and D-21 change shape or merely move?** A self-hosted runner still needs a
   `contents:read` token for the cross-repository migration checkout, and still authenticates to
   Packages. Self-hosting must not be allowed to *hide* that debt by making it locally convenient.
5. **How does a runner receive a registry credential without becoming a standing credential store?**
   STD-0017 and RUN-0004 govern; the ephemerality rule in §C makes this easier and does not answer it.
6. **Who operates this at 03:00?** The honest answer today is "the same one person", and that is an
   argument for R2 over R3 rather than an argument against the RFC.

## Related

- **ADR-0021** — `locked`; CI is a reusable workflow per stack, and is silent on `runs-on`
- **ADR-0002** — the CI gate is the lock; this RFC is about the lock's power supply
- **ADR-0024** — `locked`; nested tenancy, and the boundary a runner must never share
- **ADR-0103** — containment as placement; D3's "our own first buyer" is §Consequences' argument
- **GLO-0004**, **GLO-0005** — what a crossing costs, and why a grant is never an absence
- **RFC-0019** — one Proxmox cluster, Kubernetes on VMs above it
- **RFC-0014** — plane versus scope, which §Open questions 2 defers to
- **RFC-0045** — placement, and the containment ladder's origin
- **REQ-0015** — threat modelling; a runner is a new entry in it
- **tool-docket D-69**, **console-docket D-21** — the CI credential debt this must not hide
