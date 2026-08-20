---
id: ADR-0008
title: Governance rules are adopted only where enforceable and currently applicable
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [governance, scope, enforcement, policy]
breaking: false
enforced_by:
  - check: knowledge.no_unenforced_locked_artifacts
    severity: warn
    rationale: The backlog must stay visible; this ADR exists to keep it from growing by aspiration.
  - check: knowledge.no_duplicate_id_namespaces
    severity: error
    rationale: A second id scheme (LAW-, POLICY-) would fragment cross-referencing.
---

# ADR-0008 — Governance rules are adopted only where enforceable and currently applicable

## Status

**LOCKED** as of 2026-08-06.

## Context

A comprehensive governance proposal was drafted covering identity, repository laws,
zero-trust git, AI restrictions, dangerous operations, hooks, repository health,
dependency governance, architecture governance, releases, documentation, security,
branch naming, merge strategy, CODEOWNERS, generated code, repository layout, ADRs,
RFCs, an internal CLI, and monorepo policy.

Much of it is correct and is now adopted. Some of it cannot be enforced on the current
GitHub plan, some contradicts decisions already locked, and some governs artifacts that
do not exist yet.

Adopting all of it would have produced a large body of rules that nothing checks —
precisely the failure ADR-0002 exists to prevent. A rule marked binding that nothing
enforces is worse than an absent rule, because it manufactures confidence.

## Decision

**A governance rule is adopted when it is enforceable now and governs something that
exists now.** Everything else is recorded as deferred, with the trigger that would
make it applicable, rather than written as if already binding.

### Adopted — implemented in this change

| Proposal | Where it landed |
| -------- | --------------- |
| Developer identity, allowed emails, signing | C-0003 + `check-commits.sh` |
| AI never authors as a human; `Co-Authored-By` | C-0003, checked |
| Conventional Commits | C-0003, checked |
| Dangerous operations require approval | C-0004 + `guard-commands.sh` (PreToolUse) |
| AI cannot silently alter CI, CODEOWNERS, licence, hooks | C-0004, escalates to the user |
| Generated code is regenerated, never edited | C-0004, denied outright |
| Credentials never committed | C-0004 + `.gitleaks.toml`, pre-commit and CI |
| History is sacred; no force-push to main, no filter-branch | C-0004, denied outright |

### Adopted with a deliberate change

**Most restrictions escalate rather than block.** The proposal framed AI restrictions
as prohibitions. Implemented, they are `ask`: the agent cannot self-approve, but a
human sees the exact command and may permit it. A guard that denies every dangerous
thing is disabled within a week, and a disabled guard protects nothing.

**Recursive deletes of relative build paths are not escalated.** `rm -rf ./target` is
routine. Escalating it trains the habit of approving without reading, which is how a
genuinely dangerous one gets waved through.

### Already covered — not duplicated

ADR numbering and immutability, supersession, the RFC→ADR lifecycle, the internal CLI,
and the "documentation explains, tooling enforces" principle were already established
in C-0001, C-0002, ADR-0002 and ADR-0007. The proposal's closing principle —
*every important rule should be enforced automatically; documentation explains the
rule, tooling enforces it* — is a restatement of ADR-0002 and is already the
foundation of this repository.

A `.governance/` directory of nineteen policy files was proposed. Rejected as a
duplicate: this repository **is** that directory, under a name chosen deliberately
(see README). Most of those nineteen files would be empty for a year, and empty
scaffolding is worse than none because it advertises coverage that does not exist.

`LAW-NNNN` was proposed as a second identifier scheme. Rejected: a second namespace
alongside `C-`/`ADR-`/`STD-` fragments cross-referencing for no gain. The proposed
laws are principles and already live in C-0001, or are enforceable rules and now live
in C-0004.

### Blocked by the plan — not adoptable today

Confirmed against the GitHub API on 2026-08-06: the `Paridyum` organisation is on the
**Free** plan, where private repositories have **no branch protection and no
rulesets**.

| Proposed | Status on GitHub Free (private) |
| -------- | ------------------------------- |
| Required reviews | Unavailable |
| Required status checks | Unavailable |
| Branch protection / no direct commits | Unavailable |
| Merge queue | Unavailable (Team+ regardless) |
| Required signed commits | Unavailable — enforced locally and in CI instead |
| Secret scanning push protection | Unavailable on private — `gitleaks` substitutes |
| CODEOWNERS *enforcement* | Unavailable — the file is still generated and kept correct |

These are not rejected. They are written and committed as inert configuration, and
become live on the plan upgrade whose trigger conditions ADR-0004 already records.
This is the single largest gap between the intended governance and the enforced
governance, and it costs a few dollars a month to close.

### Contradicts a locked decision — not adopted

**Reserved root directories (`apps/`, `libraries/`, `platform/`, …) and the
"one crate, one owner, no `utils` crate" monorepo policy** both describe a monorepo.
ADR-0001 locks polyrepo. The intent survives — one responsibility, one owner, one
README, one changelog, no `common`/`utils`/`misc` grab-bags — but it applies **per
repository**, and belongs in the service template rather than as a root layout rule.

**Dependency governance requiring purpose, alternatives, maintenance status, security
and licence review for every dependency** would, as written, gate `serde` and `tokio` —
which ADR-0005 already locks as the mandated stack. Adopted in narrowed form: the
locked stack is pre-approved; the review applies to dependencies **outside** it, and
is mechanised by `cargo-deny` for licence and advisory checks rather than by prose.

**`cargo test`, `cargo audit`, and integration tests in pre-commit** is rejected on
its own terms. A pre-commit hook that takes minutes trains `--no-verify` as a reflex,
which disables layer 2 of the enforcement ladder entirely (ADR-0007). Pre-commit runs
only fast checks — formatting, identity, secrets, knowledge integrity. Tests belong in
CI, where slowness costs nothing that matters.

### Deferred — governs what does not exist yet

Nightly repository health (dead code, orphan crates, circular dependencies, build
times), release governance, and per-service `API.md` / `OPERATIONS.md` requirements all
govern Rust code and services that have not been written. They are recorded here as
intended and become ADRs when there is something to check.

`README.md` + `ARCHITECTURE.md` + `OWNERS` in *every folder* is rejected at that
granularity — it produces stub files nobody reads. Per repository, enforced by the
template, is the right unit.

## Consequences

**Accepted costs**

- The governance actually in force is narrower than the proposal, and visibly so.
  Anyone reading the proposal and then the repository will find gaps; this ADR is the
  explanation.
- Several strong controls wait on a paid plan. Until then the org runs on
  agent-layer guards plus after-the-fact audit.
- Deferred items risk being forgotten. The trigger conditions above are the mitigation.

**Gained**

- Every locked rule in this repository is enforced by something executable, so `locked`
  keeps meaning what ADR-0002 says it means.
- The gap between intended and enforced governance is documented rather than assumed.
- Adding a rule still costs a check, which keeps the constitution to rules worth it.
