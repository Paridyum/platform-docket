---
id: ADR-0016
title: Commit identity is gated where it is still fixable, and reported where it is not
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [ci, identity, enforcement, c-0003, gate-design]
breaking: false
enforced_by:
  - check: git.identity_range_non_empty
    severity: error
    rationale: The identity gate ran green on every push for the life of the workflow while examining zero commits. A gate that examines nothing must fail, not pass.
---

# ADR-0016 — Commit identity is gated where it is still fixable, and reported where it is not

## Status

**LOCKED** as of 2026-08-06.

## Context

C-0003 declares four `git.*` checks at `error` severity: author email on an allowlist,
Conventional Commit subject, no AI listed as Author, and signed commits (`warn`).
`knowledge-check.yml` ran them via:

```
./scripts/check-commits.sh "origin/${{ github.base_ref || 'main' }}..HEAD"
```

On a `push` event `github.base_ref` is empty, so the range became `origin/main..HEAD`.
**After a push, `origin/main` already contains the pushed commit, so that range is always
empty.** `check-commits.sh` treated an empty range as a clean pass and exited 0. The CI
log is unambiguous:

```
Run ./scripts/check-commits.sh "origin/main..HEAD"
No commits in range origin/main..HEAD
```

The step showed a green tick having examined **zero commits**. C-0003's four checks had
never run in CI, on any commit, since the workflow was written. They ran only in the
pre-commit hook, which `--no-verify` skips.

Two distinct defects hid inside one green tick: a caller computing an impossible range,
and a callee treating "nothing to check" as "everything is fine." Either alone would have
been caught by the other. Together they produced exactly the failure ADR-0002 names — a
rule that silently enforces nothing, manufacturing false confidence.

### Why `continue-on-error` was not the bug

The step also carried `continue-on-error: ${{ github.event_name == 'push' }}`, which made
it non-fatal on pushes. That looked like the defect and is not: with an always-empty range
the step could not fail regardless. More importantly, the leniency is **correct**, for a
reason nobody had written down.

An identity defect on a commit that is already on `main` cannot be corrected. Author
email, authorship and commit subject are fixed at commit time; changing them requires
rewriting history, which C-0004 designates a protected operation. So a hard failure on
push would leave `main` permanently red **with no legal remedy** — the precise trap
ADR-0014 exists to escape, and the reason ADR-0015 rejected fail-safe-to-everything.

C-0003's own script header already says this: *"Identity is the one property that cannot
be corrected later without a history rewrite, which C-0004 forbids. So this runs before
the commit exists."* The design was right. It was undocumented, and the range bug meant it
never mattered.

## Decision

**Identity is enforced where a violation can still be fixed, and reported where it
cannot.**

| Layer | Behaviour | Why |
| --- | --- | --- |
| pre-commit hook | **error** — blocks the commit | the commit does not exist yet; the defect is free to fix |
| pull request | **error** — fails the check | the branch can still be amended or rebased before merge |
| push to `main` | **reported, non-fatal** | the commit exists and history rewriting is forbidden; failing hard yields permanent red with no legal fix |
| `forge audit` | reports historical violations | an inventory, not a gate |

### The range must be the commits that were actually pushed

On `push`, the correct range is `github.event.before..github.sha`. Two cases fall back to
the tip commit alone, because `before` is not usable in either:

- a branch's first push, where `before` is all zeros
- a force-push, where `before` is not an ancestor of `github.sha`

On `pull_request`, `origin/${{ github.base_ref }}..HEAD` is correct and unchanged.

A full clone is required for the range to resolve — `fetch-depth: 0`, which this workflow
already needs for the secret scan.

### An explicitly requested empty range is an error

`check-commits.sh` now fails when a range is passed on the command line and resolves to
zero commits. The only reason to pass a range is that you believe it contains commits; if
it does not, the caller is wrong. An empty range remains legitimate when no range was
requested at all, which is the local default and the `--staged` path.

This is the half of the fix that generalises. Correcting the workflow's range fixes today's
bug; making the callee refuse to pass on nothing prevents the next caller from
reintroducing it.

## Consequences

**Accepted costs**

- **The backlog grows by one** — `git.identity_range_non_empty`, 104 → 105.
- **Identity violations reaching `main` are reported, not blocked, and that is a real
  hole.** Someone using `git commit --no-verify` and pushing directly to `main` gets a
  yellow annotation, not a red build. On GitHub Free there is no branch protection to force
  the PR path (ADR-0004), so direct push is available to anyone with write access — which
  under ADR-0012 is admins only. The mitigation is social and structural, not mechanical,
  and this ADR does not pretend otherwise.
- **The range logic is inline shell in a workflow**, which is harder to test than a script
  and duplicated the day a second workflow needs it. Accepted for now; it belongs in
  `check-commits.sh` once a second caller exists, and moving it then is cheap.
- **`git.commits_are_signed` stays `warn`** and is now visibly warning on every commit in
  this repository, because `commit.gpgsign` is unset. That noise is correct — signatures
  cannot be added retroactively either.

**Gained**

- C-0003's four checks actually execute in CI, on the commits that were pushed. They did
  not before.
- A gate that examines nothing now fails instead of passing, which closes a whole class of
  bug rather than one instance.
- The leniency on push is a recorded decision with a reason, rather than an incidental
  `continue-on-error` that reads as carelessness.

**Rejected alternatives**

- *Fail hard on push.* Rejected: unfixable without a forbidden history rewrite, so `main`
  goes permanently red. The gate would then be disabled or bypassed, which is strictly
  worse than an honest warning.
- *Drop the CI identity step and rely on the pre-commit hook.* Rejected: the hook is
  bypassable with `--no-verify`, and the CI report is the only signal that someone did so.
  A warning nobody is forced to act on still tells you it happened.
- *Compare against `github.event.commits` from the webhook payload instead of a git range.*
  Rejected: the payload is capped at 20 commits per push and omits the rest silently,
  which is the same silent-truncation failure this ADR is about.
- *Allow the empty range to pass but emit a warning.* Rejected: a warning in a step that
  exits 0 is indistinguishable from success in the check summary, which is how this went
  unnoticed for the life of the workflow.

## Known limitation

This ADR fixes one caller. `templates/infra-ci/rust-ci.yml` has **no identity step at
all**, so every future Rust repository will run without C-0003 enforcement in CI until one
is added there. That is tracked as a gap against `git.*` in `check-registry.yaml` rather
than fixed here, because `Paridyum/infra-ci` does not yet exist as a repository and the
template cannot be exercised until it does.
