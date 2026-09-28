# Agent entrypoint — read this before doing anything

You are working inside Paridyum. This repository is the **source of truth for every rule that
binds you**. It is not documentation to skim; it is the specification you are implementing
against.

**This file is a router, not a manual.** It holds only the rules you need in order to decide
what to read next, plus the ones whose violation is destructive. Everything else is one hop away
in the table below. Where a rule appears here *and* in a standard, the standard is the
authority — this file is deliberately not a second copy of anything.

## The one rule that generates the others

> **Never infer architecture. Retrieve it.**

If you are about to decide how something should be named, structured, deployed, or exposed — the
decision has probably already been made and written down. Find it. If it genuinely has not been
made, say so and propose an RFC. Do not quietly invent an answer and encode it in code; that is
how drift starts.

## Precedence

When sources conflict, higher wins. This ordering is not negotiable.

1. `constitution/` — foundational, org-wide. Changing one is a deliberate act.
2. `adr/` with `status: locked` — decided and closed.
3. `standards/` — how to write things.
4. `patterns/` — preferred implementations. `anti-patterns/` — forbidden ones.
5. `adr/` with `status: accepted` or `proposed` — direction, not yet binding.
6. `bets/` — the current commercial wager. Binding as direction, expected to change.
7. Existing code in the repository you are editing.

Within `constitution/`, C-0001's own rule settles conflicts: **the lower-numbered principle
wins.** So C-0001 outranks C-0006, and C-0006 specifies it rather than amending it.

**Existing code is the lowest authority.** Code that contradicts a locked artifact is a bug in
the code, not an amendment to the rule. Do not pattern-match off a neighbouring file and assume
it is correct — check the rule.

`glossary/` is absent from this ordering because it defines terms rather than rules. Read it
anyway. Four of its entries are about tenancy, and tenancy here is **nested** (GLO-0004) — the
single easiest thing to get structurally wrong. One is about locale (GLO-0006), which is four
independent fields rather than a setting called "language" — the second easiest.

## What `status: locked` means

A locked artifact **must not be contradicted**. Not by you, not by a human, not "temporarily",
not "just for this service".

There are **two** legal changes, and only two. Supersession: write a new artifact that supersedes
it, get it accepted, set `superseded_by` on the old one. And an **erratum** (ADR-0031): a dated
correction appended under a trailing `## Errata` heading, for a factual error rather than a wrong
decision. Everything above the heading must stay byte-identical.

Anything else — editing a locked file in place — is a CI failure
(`knowledge.locked_artifacts_immutable`) because it destroys the record of what was true when
the code was written.

This file said "the only legal way" until 2026-08-08, while the errata path was implemented in
*both* enforcement layers. An agent obeying this router would have refused a legal correction.

**If a task requires violating a locked artifact: stop and say so.** Name the artifact id. Do not
proceed on the assumption that the rule is stale or that the user meant to grant an exception.
Ask.

You do not need to memorise which artifacts are locked. The `SessionStart` hook lists the
constitution and every locked ADR whose `scope` matches this repository's class at the top of
each session — **filtered by class**, which the hand-maintained table this file used to carry
was not.

## Paths you must never write

Three, each denied by a `PreToolUse` hook rather than left to your memory (ADR-0007):

- **`.paridyum/knowledge/`** — a read-only vendored snapshot, so every rule is readable offline
  from one checkout (ADR-0001). Edits belong in `Paridyum/knowledge` and travel via
  `forge knowledge sync`. A change here is destroyed on the next sync.
- **A locked artifact's content** — see above. The one permitted edit is recording supersession:
  `status`, `superseded_by`, `locked_at`, and nothing else in the same edit.
- **Generated output** — regenerated, never hand-edited (C-0004).

This is not advice you are expected to remember — **when the plugin is installed, the tool call
fails.** ADR-0007 scopes layer 1 to *"sessions with the plugin enabled"*, and it is declared in
`.claude/settings.json` rather than installed by it, so the guard can be absent while the config
looks complete. `scripts/check-repo.sh` warns when it is. If that warning is present, none of the
three is enforced and all three are on you.

## Your work is committed automatically

Per ADR-0025 a `Stop` hook runs every gate, then commits and pushes everything. Work here was
lost from local disk once; this is the response.

- **A failing gate does not stop the commit.** Results go into the commit message instead — a red
  commit that exists beats a green working tree that does not. Find them with
  `git log --grep 'gates: .*FAIL'`.
- **A secret does stop it**, and it is the only thing that does. A credential in history cannot be
  removed by a later commit (C-0004).
- **`git add -A` means everything.** If a file must never be committed, add it to `.gitignore`
  rather than leaving it untracked and hoping.
- Need it off for a rebase or a spike? `.paridyum-no-checkpoint`, or `PARIDYUM_NO_CHECKPOINT=1`.
  It already skips during rebase, merge, cherry-pick and bisect.

## Where to go next

**One hop. Every target is complete on its own — none of them is another router.** Read the row
that matches what you are about to do, before you do it.

| If you are about to… | Read | Which decides |
| --- | --- | --- |
| **build any runtime capability at all** | [C-0008](constitution/C-0008-tenant-zero.md) · [ADR-0116](adr/ADR-0116-nothing-in-the-runtime-resolves-without-an-org-or-a-tenant-and-no-capability-ships-without-a-buyer.md) | nothing resolves without an org or a tenant · staff read only the personless skeleton · Paridyum is tenant zero, never privileged · no capability ships without a buyer |
| **write "org", "tenant", "account", "operator" or "business" — in code, a UI or prose** | [ADR-0113](adr/ADR-0113-org-tenant-and-account-are-three-words-for-three-levels-and-the-vendors-organization-is-named-as-the-vendors.md) · [GLO-0012](glossary/GLO-0012-org.md) | **org** = the customer of paridyum.in, which **owns** tenants · **tenant** = a business, the outer silo · **account** = a silo inside a tenant · *operator* = breadth within one tenant · the vendor object is `zitadel_org`, never a bare `org` · frontmatter `scope: [org]` means *binds everywhere* |
| write any non-trivial code | `design-change` skill | ADR-0013 — significance is computed from the diff, never self-declared |
| turn a large spec into contexts | `decompose-spec` skill | ADR-0019 — decomposition precedes implementation |
| plan the PRs for a requirement | `slice-requirement` skill · [STD-0007](standards/STD-0007-change-slices-and-pr-size.md) | slices are vertical, and one sentence with no "and" |
| open a pull request | [STD-0007](standards/STD-0007-change-slices-and-pr-size.md) | ≤250 fine · 251–400 warn · **>400 fails**, on the branch diff |
| write or supersede a knowledge artifact | `new-decision` skill · [knowledge.schema.json](knowledge.schema.json) | ADR-0002 — `locked` requires a named check |
| **finish anything you had to work out** — a corrected path, a non-obvious cause, a chosen threshold | [STD-0011](standards/STD-0011-capturing-what-recurs.md) | six triggers, where each kind goes, and what needs no capture |
| add a new artifact *type*, or a new test type | [artifact-registry.yaml](artifact-registry.yaml) · [test-registry.yaml](test-registry.yaml) | ADR-0022, ADR-0029 — both lists are registry data, not a hardcoded set |
| write a migration | [STD-0023](standards/STD-0023-migration-conventions-a-table-declares-its-scope-its-plane-and-its-data-class.md) · [ADR-0115](adr/ADR-0115-data-is-scoped-to-the-platform-an-org-a-tenant-or-an-account-in-each-planes-database-and-operator-scope-is-a-grant.md) · [GLO-0005](glossary/GLO-0005-operator-scope.md) | four scopes — `platform`, `org`, `tenant`, `account` — and a plane · business data never names its org · no person in the control plane · operator scope is a **grant, never an absence** |
| name an account, or create a login for a service | [STD-0017](standards/STD-0017-accounts-and-their-credentials.md) · [RUN-0004](runbooks/RUN-0004-handling-a-secret.md) | role never person; four reserved names; the vault folder **is** the register |
| name a repo, crate, table, event, env var, image or label | [C-0007](constitution/C-0007-naming.md) · [class-registry.yaml](class-registry.yaml) | naming, and which repository classes exist |
| draw or generate an architecture view | [ADR-0040](adr/ADR-0040-the-system-view-is-generated-from-manifests.md) · [STD-0002](standards/STD-0002-documentation.md) | the model is `manifest.yaml`; cross-repo views are **generated**, never hand-authored |
| organise modules, or split a file | [STD-0004](standards/STD-0004-code-organization.md) | responsibility never size; no `utils`, `common`, `helpers` |
| write tests | [STD-0001](standards/STD-0001-testing.md) · [STD-0008](standards/STD-0008-test-tiers-and-gates.md) | how a test is written · which tier runs it, what blocks, what it leaves behind |
| test anything that crosses a tenant or account boundary | [STD-0009](standards/STD-0009-tenant-isolation-testing.md) | two boundaries, every access path, and the **denial** is the assertion |
| add a dependency, lint or toolchain pin | [STD-0012](standards/STD-0012-rust-lints-and-editor.md) · [ADR-0005](adr/ADR-0005-rust-as-primary-language.md) | the locked stack |
| document a repository | [STD-0002](standards/STD-0002-documentation.md) | required files per class, and `manifest.yaml` |
| draw a service boundary | [anti-patterns/](anti-patterns/) | six forbidden shapes, three of them about exactly this |
| make a pricing, market or ICP call | [C-0006](constitution/C-0006-market-posture.md) · [bets/](bets/) | six anti-goals — and **check whether a bet's `trigger` has fired** |
| **write anything that binds one product rather than all of them** | [ADR-0050](adr/ADR-0050-a-product-is-a-governance-axis.md) · [product-registry.yaml](product-registry.yaml) | `product:` is a **fourth axis**, never a `scope` value · absent means every product · Paridyum is registered as product zero |
| record market or customer evidence — a segment, a competitor, a TAM, an interview | [ADR-0050](adr/ADR-0050-a-product-is-a-governance-axis.md) · `markets/` | an `MKT-` **must** cite `sources`; a wager is a `BET-` instead |
| propose a new vertical, or move a product's stage | [C-0006](constitution/C-0006-market-posture.md) · [product-registry.yaml](product-registry.yaml) | anti-goal 6 — **no second vertical before the first has a renewed customer**, and the registry counts |
| touch health, payment-card or regulated data | [ANTI-0007](anti-patterns/ANTI-0007-regulated-data-before-controls.md) | the one anti-pattern that can end the company in a single incident |
| find out what is actually enforced | `knowledge-audit` skill · `scripts/check-knowledge.sh` | ADR-0014 — declared versus implemented |
| answer a compliance or security question | [check-registry.yaml](check-registry.yaml) `frameworks:` · [ADR-0028](adr/ADR-0028-controls-map-outward-to-frameworks.md) | controls map **outward**; never write a rule because a framework named it, and never compute a coverage percentage |

Skills are invoked by name and the harness loads them (ADR-0007). Everything else is a path in
this repository, or in the vendored snapshot at `.paridyum/knowledge/`.

**Check a bet's `trigger` before following it.** C-0006 states what a fired trigger means.

## Writing changelog entries

Inline because STD-0005 requires it *here specifically*: Keep a Changelog 2.0.0 recommends the
brief be reproduced in `CLAUDE.md`, and STD-0005 adopts that. The standard still governs.

> Summarise notable, **user-facing** changes. Do not paste a git log. Sort each change into one of
> the six types — Added, Changed, Deprecated, Removed, Fixed, Security. Explain the reason in the
> text, not just the mechanism. Mark breaking changes. If a change is not notable to a reader,
> leave it out.

There is no per-PR changelog gate, deliberately. STD-0005 explains why, and what counts as
notable.

## Honest current state

`tool-forge` exists and ships `forge context`, `forge knowledge sync` and `forge check`. It implements **5 of the
76 checks that named it as owner** — an existing repository is not an enforcing one, and the registry counts
implementations, not repositories. Roughly half the checks declared in `check-registry.yaml`
are implemented in the bash gates and the rest still await it — `scripts/check-knowledge.sh` prints the
split on every run, so ask the repository rather than trusting a figure written here.

For everything not yet implemented, **the locks are enforced by your compliance rather than by
machinery.** Behave as though the gate were already running, and treat a green check as unproven
until you have watched it fail.
