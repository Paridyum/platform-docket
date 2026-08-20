---
id: ADR-0012
title: Contributors get read access and contribute from forks; nobody but an owner has write
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org, platform, service, library, tool, infra, spec]
applies_to: ["*"]
tags: [access-control, least-privilege, github, onboarding, interns, ci, security]
breaking: false
enforced_by:
  - check: org.base_permission_is_none
    severity: error
    rationale: The org default is `read on every repository`. Left at the default, adding a person to the org grants them the whole estate and no team configuration can take it back.
  - check: org.contributors_are_read_only
    severity: error
    args: { exempt_roles: ["admin"] }
    rationale: This ADR's guarantee is that a contributor cannot push. A single promotion to write deletes that guarantee silently — there is no red build, no diff, nothing else that would notice.
  - check: org.private_forking_enabled
    severity: error
    rationale: With forking disabled a read-only contributor has no path to contribute at all, and the pressure to "just give them write for now" becomes irresistible. The permissive setting is what makes the restrictive one survivable.
  - check: org.two_factor_required
    severity: error
    rationale: Read access to the whole architecture is granted on the strength of one password otherwise. This is the cheapest control in the ADR and the only one that protects against a compromised contributor account rather than a careless one.
  - check: ci.no_pull_request_target
    severity: error
    rationale: pull_request_target runs the base repo's workflow with its secrets and a write-scoped token. Combined with a fork PR checkout it is arbitrary code execution as the org.
  - check: ci.secret_jobs_guarded_on_fork
    severity: warn
    rationale: A job needing a secret does not fail informatively on a fork PR — it fails confusingly, and the reflex fix is to reach for pull_request_target.
---

# ADR-0012 — Contributors get read access and contribute from forks

## Status

**LOCKED** as of 2026-08-06.

## Context

Paridyum is about to have contributors who are not the owner — interns first, and on the
same mechanism, any future hire or external collaborator.

Three already-locked decisions collide at that moment.

**ADR-0002** says a decision is not locked until something executable rejects its
violation. **ADR-0004** records that the strongest form of that gate — a required status
check on a protected branch — is unavailable for private repositories on the Free plan,
and accepts the gap because the only person who can merge is the owner. That acceptance
is load-bearing and it expires the instant a second person can push:

> **ADR-0004, upgrade trigger:** move to GitHub Team when *a second person gains write
> access to any repository*.

The obvious reading is that onboarding an intern forces a plan upgrade. That reading is
correct but incomplete, because it treats write access as the only way to contribute.
It is not, and the alternative is not a workaround — it is a stronger guarantee than the
one the paid plan would buy.

There is also a second-order problem specific to a GitHub organisation, and it is the
one most likely to be got wrong silently: **the org default base permission is `read`
across every repository.** Adding a person to the org therefore grants them the entire
estate before any team is configured. No team assignment can subtract from a base
permission — teams only add. A carefully scoped team on top of a `read` base is
decoration.

## Decision

### 1. Base permission is `None`

Organisation → Settings → Member privileges → **Base permissions: None**.

Org membership grants access to nothing. Every grant is explicit, per repository, via a
team. This is the setting that makes least privilege real rather than aspirational, and
it is the first check in this ADR because everything else is void without it.

### 2. Two-factor authentication is required for the organisation

Organisation → Settings → Authentication security → **Require two-factor
authentication**.

Enabled *before* the first contributor is invited, because turning it on later removes
non-compliant members from the organisation — a disruptive way to discover the setting
exists. Every contributor is being handed read access to the architecture, the roadmap
and the knowledge base; that should not rest on one password.

### 3. Contributors are organisation members with **read**, never write

A contributor is an org member, added to a team that has **read** on exactly the
repositories they need — and always including `knowledge`, because the agent harness
installs from it as a plugin marketplace (ADR-0007) and the vendored constitution is
useless to someone who cannot install the guard that enforces it.

They are org members rather than outside collaborators deliberately: forking of private
repositories is an organisation-member privilege, and routing contributors through
membership avoids depending on the narrower and less stable question of what an outside
collaborator may fork. That choice has a cost, recorded under Rejected alternatives.

### 4. Contribution happens from a fork

Organisation → Settings → Member privileges → **Allow forking of private repositories:
enabled**.

```
contributor forks svc-x  →  pushes to their fork  →  opens PR to Paridyum/svc-x
                                                      ↓
                                          owner reviews and merges
```

The contributor cannot push to the base repository, cannot merge, and cannot move
`main` — not because a rule forbids it, but because they hold no permission that would
allow it. **The platform enforces this, so it is the one control in Paridyum that does
not depend on `tool-forge` existing.** Every other lock in this repository is currently
honoured by compliance (see `CLAUDE.md`, "Honest current state"); this one is real today.

The checks in `enforced_by` therefore have a different job than usual. They do not create
the guarantee — they detect *configuration drift away from it*: a base permission
silently reset, a contributor promoted to write "temporarily", forking switched off.

### 5. Fork PRs constrain CI

This follows from the fork decision and is not optional, because getting it wrong
converts the security win into a remote code execution path.

- Workflows trigger on **`pull_request`**. A fork PR run gets **no secrets** and a
  read-only token. That is the correct and intended behaviour.
- **`pull_request_target` is denied outright** in any repository under this
  organisation. It runs the base repo's workflow with full secrets and a write token; if
  it also checks out the PR head, an attacker's first PR executes arbitrary code as the
  org. There is no use case here worth that.
- Jobs requiring a secret — GHCR push, coverage upload, deploy — are guarded:

  ```yaml
  if: github.event.pull_request.head.repo.fork == false
  ```

  Guarded, not deleted: they must still run on the owner's branches. `infra-ci`'s
  reusable workflow (ADR-0010) owns this logic once, so no repository re-derives it.
- **No workflow-approval gate protects a contributor's first PR.** Every option under
  "Fork pull request workflows" excludes organisation members — the strictest,
  *"Require approval for all external contributors"*, covers only users *"not a member or
  owner of the repository and not part of the organization"*. A contributor's fork PR
  therefore runs workflows immediately, on push, with no human in between. Fork PRs still
  receive no secrets and a read-only token, so this is an **Actions-minutes exposure**
  (2,000/month on Free, ADR-0004), **not** a code-execution one. Treat minutes as a
  monitored resource rather than assuming a gate exists; the lever if it is ever abused
  is outside-collaborator status, which does get the gate — see Rejected alternatives.

### 6. Relationship to ADR-0004's upgrade trigger

**This ADR does not supersede or amend ADR-0004.** ADR-0004's first trigger fires when a
second person *gains write access*. Under this decision that never happens, so the
trigger is satisfied rather than avoided — the condition it protects against does not
arise.

ADR-0004's other two triggers stand untouched and are now the operative ones:

- an agent bypasses the gate and merges a violation that reaches `main`
- the first service handles real user data

Both are about the **owner's** merges, which is the correct residual risk. This ADR closes
the enforcement gap for contributor-authored code; it does nothing about the owner
merging a red build, and it should not be read as having done so.

## Consequences

**Accepted costs**

- **Review is a hard bottleneck by construction.** Every contributor change requires the
  owner. This is the point — it is the review gate the Free plan cannot otherwise
  provide — but it means contributor throughput is capped by owner attention, and an
  onboarding plan that ignores that will produce a queue of stale PRs.
- **Actions minutes are exposed to contributors with no gate.** Per §5, a member's fork
  PR runs CI on every push and Free allows 2,000 minutes per month for private
  repositories. A handful of contributors iterating noisily can exhaust that, and the
  first symptom is unrelated pipelines failing to start. Mitigated only by ADR-0010's
  decision to keep mutation testing and audits weekly rather than per-PR.
- **Fork divergence.** A contributor's fork drifts from `main`, and the vendored
  knowledge snapshot inside it ages against the 30-day freshness window
  (`repo.has_vendored_knowledge`, ADR-0001). Rebasing before opening a PR becomes part of
  the contribution procedure rather than a nicety.
- **`forge audit` gains a surface.** It must now enumerate org settings, team grants and
  per-repo collaborators through the GitHub API, not just repository contents. That is a
  new class of check — org-level rather than file-level — and it is what the first four
  entries in `enforced_by` require.
- **CI complexity.** Two execution contexts (trusted branch, untrusted fork) instead of
  one. Every secret-consuming job needs the fork guard, and a missing guard shows up as a
  confusing failure on a newcomer's first PR — the worst possible audience for it.
- **A private fork is a copy of private source on a personal account.** Bounded by
  granting read only where needed, but it is a real widening of where the code lives, and
  offboarding must include confirming the fork is deleted.

**Gained**

- Contributor-authored code cannot reach `main` without owner review, **enforced by the
  platform**, on the Free plan, today. This is the ADR-0002 guarantee actually holding
  for the majority of incoming change.
- The ADR-0004 plan upgrade is decoupled from headcount. It happens when real user data
  or a demonstrated bypass demands it, not on the day a second person appears.
- Least privilege is the default rather than a policy: a new member starts with access to
  nothing, and every grant is a visible, auditable, per-repository act.
- Onboarding scales without a permissions conversation each time. The mechanism is
  identical for an intern, a hire, and an open-source contributor.

**Rejected alternatives**

- **Outside collaborators instead of organisation members.** Outside collaborators *do*
  fall under the workflow-approval gate, which would close the Actions-minutes exposure
  in §5 — their first PR would wait for a human. Rejected because it reintroduces exactly
  the question membership was chosen to avoid: whether, and under what policy, an outside
  collaborator may fork a private repository. That trades a bounded, observable cost
  (minutes) for an unbounded one (no path to contribute at all if the answer turns out to
  be no). Revisit if minute exhaustion becomes a real incident rather than a hypothetical.
- **Upgrade to GitHub Team ($4/user/month) and grant write with protected branches.**
  Cheap, and it closes the owner-side gap too, which this ADR does not. Rejected as the
  *first* move because it is strictly weaker on the contributor axis: write plus branch
  protection is a rule that can be reconfigured by an admin, while read plus fork is an
  absence of capability. Adopt it when ADR-0004's remaining triggers fire — and note that
  it composes with this decision rather than replacing it.
- **Make repositories public to get free branch protection.** Already rejected in
  ADR-0004 for disclosure reasons; adding contributors does not change that calculus.
- **Write access scoped to one repository per contributor.** The intuitive move, and what
  the polyrepo topology invites. Rejected: on Free, write on any private repo means the
  ability to push to `main` and merge one's own PR, because nothing can require a review.
  Least privilege at repository granularity is not least privilege at *operation*
  granularity, and the operation that matters is merge.
- **A shared `staging` branch contributors may push to, with the owner merging to
  `main`.** Approximates the fork model without forks, but Free cannot protect `main`
  from someone with write on the repo — the branch discipline is honour-based, which is
  precisely what ADR-0002 rejects.
- **Path-level permissions inside a monorepo** (`alice → /frontend`). Considered and
  discarded as impossible, not merely unavailable: GitHub does not offer directory-scoped
  repository permissions on **any** plan, including Enterprise. This is worth recording
  explicitly because it is a common misconception, and because it is *not* among the
  reasons ADR-0001 chose polyrepo — that decision rests on agent context locality and
  independent lifecycles, and must not be retroactively justified by this.

## Platform facts, verified 2026-08-06

Recorded with their evidence because the whole decision rests on them and every one is
platform behaviour that has changed before. Re-verify before relying on any of it a year
from now.

| Claim | Verdict | Evidence |
| ----- | ------- | -------- |
| Private-repo forking is available on Free | **yes** | `members_can_fork_private_repositories` is present and settable on the `Paridyum` Free org via `GET /orgs/Paridyum`. GitHub's forking-policy documentation states no plan restriction. |
| Branch protection / required reviews are unavailable on Free for private repos | **yes — ADR-0004 stands** | Rulesets apply "for customers on **GitHub Team and GitHub Enterprise plans**". Branch restrictions work "in public repositories owned by a GitHub Free organization and in all repositories owned by an organization using GitHub Team or GitHub Enterprise Cloud". The plans page lists Required pull request reviewers, Protected branches and Code owners as GitHub Team features in private repositories. |
| CODEOWNERS review enforcement is unavailable on Free | **yes** | "Code owners" is listed as a GitHub Team feature. Confirms the `templates/rust-service/.github/CODEOWNERS` comment. |
| Fork PRs get no secrets and a read-only token | **yes** | "Allows users to run workflows from fork pull requests, using a `GITHUB_TOKEN` with read-only permission, and with no access to secrets." |
| The workflow-approval gate covers org members | **no** | "Require approval for all external contributors — All users not a member or owner of the repository and not part of the organization need approval." See §5. |

Two consequences of the second row: ADR-0004 requires no supersession, and if a future
check finds rulesets have become available on Free for private repositories, **both**
ADR-0004 and this ADR should be revisited by supersession rather than by editing.

The org's live configuration is recorded and reconciled in **RUN-0002**, not here — an
ADR states what must be true, and a runbook is where what *is* true gets checked.
