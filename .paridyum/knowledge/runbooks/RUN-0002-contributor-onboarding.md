---
id: RUN-0002
title: Onboarding a contributor — access, first pull request, and offboarding
type: runbook
status: accepted
created: 2026-08-06
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [onboarding, offboarding, access-control, interns, github, teams]
breaking: false
enforced_by:
  - check: org.no_members_without_team
    severity: warn
    rationale: A member in no team holds no grants, which means onboarding stalled halfway or offboarding did. Both are invisible without this.
---

# RUN-0002 — Onboarding a contributor

The procedure for giving someone their first Paridyum access and their first merged pull
request, and for removing both cleanly. Implements **ADR-0012** (contributors get read
access and contribute from forks); read that first if you want the reasoning, because
several steps here look unnecessarily restrictive without it.

Applies identically to an intern, a hire, and an external collaborator. There is one
mechanism.

## The one-line summary

> They get **read** on the repositories they need, they **fork**, they **open a pull
> request**. They never get write, and this does not change as they become more senior.

## Before the first contributor: verify the org

Two organisation settings carry ADR-0012. Both are org-level API state, **not files** —
no pre-commit hook or PreToolUse guard can protect them (C-0004 covers governance
*files*), so they are verified here by hand and by `forge audit` thereafter.

```sh
gh api /orgs/Paridyum --jq '{
  base:    .default_repository_permission,
  forking: .members_can_fork_private_repositories,
  twofa:   .two_factor_requirement_enabled
}'
```

Required output:

```json
{ "base": "none", "forking": true, "twofa": true }
```

If it is not, fix it. These are governance-altering operations, so an agent must escalate
rather than run them (C-0004 §3), and they need a scope the default `gh` login does not
carry — the same `admin:org` gap RUN-0001 documents:

```sh
gh auth refresh -h github.com -s admin:org
gh api -X PATCH /orgs/Paridyum -F default_repository_permission=none
gh api -X PATCH /orgs/Paridyum -F members_can_fork_private_repositories=true
gh api -X PATCH /orgs/Paridyum -F two_factor_requirement_enabled=true
```

**Enable 2FA before the first invitation, not after.** Turning it on later *removes*
members who are not compliant, which is a disruptive way to find out the setting exists.

### Org configuration as found — 2026-08-06

Recorded so a later reader can tell what this runbook actually changed.

```
plan                                    free
filled seats                            1
private repos / public repos            0 / 1
default_repository_permission           read     ← ADR-0012 requires none
members_can_fork_private_repositories   false    ← ADR-0012 requires true
two_factor_requirement_enabled          false    ← ADR-0012 requires true
gh token scopes                         gist, read:org, repo  (no admin:org)
```

All three settings were non-compliant on the day ADR-0012 was written, and the token could
not change them. Nothing was granted to anyone at that point, so the exposure was
theoretical — but it is exactly the drift `org.base_permission_is_none` and
`org.two_factor_required` exist to catch, and the first check run after `forge audit`
ships should report it until it is fixed.

**`base: none` matters more than it looks.** The GitHub default is `read` on *every*
repository in the org. Teams only ever **add** permission — no team configuration can
subtract from a base permission. On the default, a carefully scoped team is decoration
and the new member already has the whole estate.

**`forking: true` looks like the opposite of least privilege and is what makes it
possible.** Without it a read-only member has no way to contribute at all, and the
pressure to grant write becomes irresistible within a week.

## Teams

Membership grants nothing; teams grant read. One team per area of work, `<area>-team`,
matching the `@Paridyum/platform-team` handle the generated `CODEOWNERS` already
references.

```sh
gh api -X POST /orgs/Paridyum/teams -f name=platform-team -f privacy=closed
gh api -X PUT  /orgs/Paridyum/teams/platform-team/repos/Paridyum/svc-example \
  -f permission=pull        # pull = read. Never push.
```

`pull` is the only permission this runbook ever grants. If you find yourself typing
`push`, stop — that is the decision ADR-0012 made, and changing it needs a superseding
ADR, not a convenient exception.

## Onboarding, step by step

### 1. Grant read — the minimum set

Every contributor needs `knowledge`, without exception. Not as a courtesy: the agent
harness installs from it as a plugin marketplace (ADR-0007), and a vendored constitution
is worthless to someone who cannot install the guard that enforces it.

| Contributor works on | Read access to |
| -------------------- | -------------- |
| *anyone*             | `knowledge` |
| a service            | `svc-<name>`, `spec-openapi`, plus `lib-*` it depends on |
| a library            | `lib-<name>`, and the repos consuming it (so they can see what they break) |
| a tool               | `tool-<name>` |
| infrastructure       | `infra-*` — **sandbox and `dev` only**, never a production credential |
| documentation        | `knowledge` alone |

Nothing else. Not "read on everything, it's all internal anyway" — that is the base
permission default this runbook exists to undo.

### 2. Invite

```sh
gh api -X POST /orgs/Paridyum/invitations \
  -f email=<their-email> \
  -f role=direct_member \
  -F 'team_ids[]=<team-id>'
```

`direct_member`, never `admin`. Invite into the team in the same call so the window
where they are a member of no team stays empty — that state is what
`org.no_members_without_team` warns about.

### 3. Workstation

Hand them **RUN-0001**. It is written for a blank machine and needs no adaptation, with
two differences for a read-only contributor:

- The `gh auth refresh -s workflow,admin:org` step is **owner-only**. `admin:org` is
  organisation administration; a contributor needs neither scope. Default `repo` is
  enough to clone, fork, and push to their own fork.
- `forge` subcommands that mutate the org (`forge org reconcile`, repository scaffolding)
  will fail for them on permissions. Expected, not a misconfiguration.

Then the harness, which is the same for everyone (ADR-0007):

```
/plugin marketplace add Paridyum/knowledge
/plugin install paridyum-platform@paridyum
```

### 4. Read before writing

Not ceremony — `CLAUDE.md` states the rule that generates every other one
(*never infer architecture, retrieve it*), and a contributor who has not read the
precedence order will pattern-match off neighbouring code, which is explicitly the
lowest authority in this organisation.

Minimum before the first pull request: `constitution/` in full (it is short and all of
it binds), `STD-0002` (documentation), the standard for their language, and the locked
ADRs whose `scope` includes their repo class.

## The contribution loop

```
issue  →  fork  →  branch  →  commit  →  rebase  →  PR from fork  →  review  →  owner merges
```

```sh
gh repo fork Paridyum/svc-example --clone \
  --fork-name svc-example  # lands in ~/Developer/paridyum/services/ per RUN-0001
cd svc-example
git remote add upstream git@github.com:Paridyum/svc-example.git

git switch -c fix/short-description
# ... work, commit as Conventional Commits (C-0003), signed ...

git fetch upstream && git rebase upstream/main    # do this before every PR
git push origin fix/short-description
gh pr create --repo Paridyum/svc-example
```

**Rebase before every PR, not just the first.** A fork drifts, and the vendored
`.paridyum/knowledge/` snapshot inside it ages against the 30-day freshness window that
`repo.has_vendored_knowledge` enforces (ADR-0001). A stale fork fails CI for a reason
that has nothing to do with the change, which is a miserable first experience.

### What the contributor sees in CI, and why it is not broken

| Observation | Cause | Correct response |
| ----------- | ----- | ---------------- |
| Workflows run on their very first PR, with no approval step | Every "require approval" option excludes **organisation members** — the gate only covers outside collaborators (ADR-0012 §5) | Nothing to configure; there is no setting for this. Watch Actions minutes instead — see below |
| Coverage upload / GHCR push jobs skipped | Fork PRs get **no secrets** and a read-only token, by design (ADR-0012 §5) | Nothing. The guard `if: github.event.pull_request.head.repo.fork == false` is correct |
| Merge button greyed out for them | They hold `pull`. They cannot merge anything | Nothing. This is the entire guarantee |

**Actions minutes are the resource to watch**, precisely because no approval gate stands
in front of a member's fork PR. Free allows 2,000 minutes per month for private
repositories (ADR-0004), every push to an open PR spends some, and the first symptom of
exhaustion is *unrelated* pipelines silently failing to start:

```sh
gh api /orgs/Paridyum/settings/billing/actions \
  --jq '{used: .total_minutes_used, included: .included_minutes}'
```

If it becomes a real problem the lever is outside-collaborator status, which *does* get
the approval gate — recorded as a rejected alternative in ADR-0012, with the reason it
lost and the condition that would change the answer.

The failure mode to watch for is an agent or contributor "fixing" a skipped job by
switching the trigger to `pull_request_target`. That hands the base repo's secrets and a
write-scoped token to fork code. It is denied by `ci.no_pull_request_target` and it is
the single most dangerous change anyone can make to this setup.

## The ladder

Task complexity grows. **Permissions do not.**

| Stage | Work | What actually changes |
| ----- | ---- | --------------------- |
| 0 | Read the constitution and the ADRs for their area | nothing |
| 1 | Documentation correction — a real one they hit themselves | nothing |
| 2 | A scoped bug fix with a test that fails before it | nothing |
| 3 | An internal tool or a `tool-*` subcommand | read on one more repo |
| 4 | A feature behind an existing interface | read on `spec-openapi` |
| 5 | A feature that changes an interface — RFC first | may author an RFC in `knowledge` (still by PR) |
| 6 | Ownership of a component | a `CODEOWNERS` entry, and their team named as `owner` in `manifest.yaml` |

Stage 6 is the one worth being explicit about, because "ownership" normally implies write
access and here it does not. Ownership means **being the required reviewer**, recorded in
`CODEOWNERS`. That entry is inert on the Free plan (ADR-0004) and becomes enforcement the
day the plan is upgraded — which is why the generated `CODEOWNERS` must already be
correct rather than written later.

`manifest.yaml` names a **team**, never a person (STD-0002). A manifest naming an
individual is wrong the day they change role, and interns change role by definition.

## Review

The owner is the only merger, so review is the throughput bottleneck by construction —
that is the review gate the Free plan cannot otherwise provide (ADR-0012). Two
consequences to actually act on:

- **Queue depth is the metric that matters.** Stale PRs are the predictable failure of
  this model. If review latency exceeds a couple of days, the answer is fewer concurrent
  contributors or the ADR-0004 plan upgrade — never a write-access exception.
- **Say which rule, with its id.** "Rename this to `/v1/invoices`, C-0002 requires plural
  nouns" teaches the retrieval habit. "Rename this" teaches dependence on the reviewer.

## Offboarding

Do all four. The first is the one that gets forgotten, and it is the one that matters.

1. **Confirm the fork is deleted.** A fork of a private repository is a copy of private
   source on a personal account. Removing org membership revokes their access to the
   *parent*; ask them to delete the fork and verify:

   ```sh
   gh api /repos/<their-login>/svc-example --silent   # want: 404
   ```

2. **Remove org membership** — this revokes every team grant at once:

   ```sh
   gh api -X DELETE /orgs/Paridyum/memberships/<their-login>
   ```

   A destructive `gh api -X DELETE` is escalated under C-0004; a human runs it.

3. **Reassign `CODEOWNERS` and `manifest.yaml` entries** if they reached stage 6. An
   owner entry pointing at a departed team member routes reviews into a void.

4. **Rotate any shared credential they could observe.** Under ADR-0012 a contributor
   never held a production secret, so this is usually empty — record that it was checked
   rather than skipping the step. Per C-0004: a credential is burned the moment it is
   exposed, and rewriting history is not remediation.

## Known gaps

Stated rather than glossed, per the house rule.

- **Org settings are unguarded between audits.** `base_permission` and the forking policy
  are API state, not files, so no hook protects them. An admin can reset them silently and
  only `forge audit` — which does not exist yet — would notice. Until it ships, the
  verification command at the top of this runbook is the whole control, run by hand.
- **The owner's own merges are unprotected.** ADR-0012 closes the gap for contributor
  code; it does nothing about the owner merging a red build on Free. Unchanged from
  ADR-0004, and one of that ADR's remaining upgrade triggers.
- **The platform facts were verified on 2026-08-06** and are tabulated with their evidence
  in ADR-0012, "Platform facts, verified". Private-repo forking is available on Free;
  branch protection and CODEOWNERS enforcement are not, so ADR-0004 stands unchanged.
  Re-verify before relying on any of it a year from now — if rulesets have by then become
  available on Free for private repositories, ADR-0004 and ADR-0012 both need revisiting
  by supersession rather than editing.
