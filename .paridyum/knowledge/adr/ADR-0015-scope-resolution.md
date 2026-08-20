---
id: ADR-0015
title: Org scope binds every repository; a classless repo is governed, not exempt
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-07
supersedes: null
superseded_by: null
scope: [org, platform, service, library, tool, infra, spec]
applies_to: ["*"]
tags: [governance, scope, forge, artifact-model]
breaking: false
enforced_by:
  - check: knowledge.scope_values_cover_repo_classes
    severity: error
    rationale: Two C-0002 prefixes had no scope value and the gap was invisible for the whole life of the model. It has now occurred twice; a check is cheaper than finding it a third time.
  - check: knowledge.supersession_is_symmetric
    severity: error
    rationale: Nothing verified that both halves of a supersession agreed. Skip the second and the old and new artifacts both read as in force, on the one question where they disagree, with the gate green.
---

# ADR-0015 — Org scope binds every repository; a classless repo is governed, not exempt

## Status

**LOCKED** as of 2026-08-07.

Held at `proposed` for a day first, which earned its keep: an audit found that binding
the `org` set to `knowledge` made two checks unsatisfiable at the source, and that defect
was fixed before locking rather than after. The third gap named in the original status
note — the absent supersession-symmetry check — is now implemented and cited above. Two
defects in locked artifacts remain uncorrectable and are tracked in REQ-0002.

## Context

`knowledge.schema.json` documents `scope` as *"which repo classes this binds"*, and
`forge context <repo>` is specified to *"select artifacts by matching repo class against
scope"*. The enum has seven values: `org`, `platform`, `service`, `library`, `tool`,
`infra`, `spec`.

C-0002 defines **eight** repository prefixes. Six map cleanly — `platform-`, `svc-`,
`lib-`, `tool-`, `infra-`, `spec-`. Two map to nothing:

| C-0002 prefix | Example | Scope value |
| --- | --- | --- |
| *(none)* — "the knowledge base, sui generis" | `knowledge` | **none exists** |
| `template-` | `template-service` | **none exists** |

Read literally, `forge context knowledge` matches no scope and returns nothing. **The
repository that defines every rule is governed by none of them.** `template-*` repos are
equally unreachable. Neither is a hypothetical: `knowledge` exists today and
`templates/rust-service/` is destined to become `template-service`.

This surfaced while auditing which of the 72 unimplemented checks genuinely require
Rust, not from reading the schema. Nothing in the model reports it, which is the deeper
problem: a scope value that matches no repo, and a repo class that matches no scope,
are both silent.

### The answer is deducible, not a matter of taste

Four artifacts are scoped `[org]` and nothing else, and every one of them carries
checks that are unambiguously per-repository:

| Artifact | Per-repo checks it carries |
| --- | --- |
| C-0003 | `git.author_email_allowlisted`, `git.commits_are_signed`, `git.no_ai_authored_as_human`, `git.conventional_commit_subject` |
| C-0004 | `repo.secret_scan_clean`, `repo.no_credential_files_committed` |
| ADR-0004 | `repo.owner_is_paridyum_org`, `repo.has_codeowners`, `repo.default_branch_is_main` |
| ADR-0006 | `repo.scripts_are_lf_only`, `repo.gitattributes_normalizes_eol` |

If `org` did not mean "every repository", then `forge context svc-billing` would omit
C-0003 and C-0004 — and commit identity, secret scanning and credential-file checks
would bind nothing, in any repository, ever. That is obviously not the intent. `org`
already means "every repository"; it had simply never been written down.

The eight artifacts scoped `[org, platform, service, library, tool, infra, spec]` are
evidence in the same direction, from the other side: their authors enumerated every
class *in addition to* `org` because the semantics were undefined and belt-and-braces
was cheaper than being wrong.

## Decision

**`scope` is a union of selectors, and `org` is the selector that matches every
repository regardless of class.**

1. An artifact whose `scope` contains `org` binds **every repository in the
   organisation**, whatever its class, including a repository with no recognised class.
2. An artifact whose `scope` contains a class value binds repositories of that class.
3. `forge context <repo>` returns the union: every artifact matching `org`, plus every
   artifact matching the repo's class, narrowed by `applies_to`.
4. **A repository with no recognised class is governed by the `org`-scoped set, not
   exempt.** `knowledge` and `template-*` are legitimately classless. This is a normal
   state, not an error.
5. `template` is added to the `scope` enum, so a future template-specific standard has
   something to target. Nothing uses it yet.
6. `knowledge` is **not** given a class. Nothing needs to target it specifically, and
   inventing a class to satisfy a model is how models acquire vestigial concepts.

### `[org, …every class]` is exactly equivalent to `[org]`

Under point 1 the eight artifacts that enumerate both are stating the same thing twice.
They are **left alone**. Simplifying them would mean editing `scope:` on eight locked
artifacts — eight supersessions — for precisely zero behavioural change. A reader
encountering the redundant form should read it as `[org]` and not look for a
distinction.

### A check whose subject is the vendoring of knowledge is vacuous at the source

Making the `org` set bind `knowledge` also binds two checks it can never satisfy:

- `repo.has_vendored_knowledge` (ADR-0001, `error`) — asks whether the repository
  contains a `.paridyum/knowledge/` snapshot. `knowledge` is what gets vendored.
- `repo.no_vendored_harness` (ADR-0007, `error`) — asks whether hook and skill files have
  been copied in rather than loaded from the plugin. `knowledge` is the plugin's home,
  at `plugins/paridyum-platform/`.

Left unaddressed this ADR would create two permanently-failing `error` checks with no
legal remedy — the exact condition it rejects *fail-safe-to-everything* for, three
paragraphs below. So it must be addressed here rather than discovered later.

**These checks are not exempted at the source; they are vacuous there.** The distinction
matters. An exemption is a policy carve-out that someone could argue about extending. A
vacuous question has no answer to carve out: "does the upstream contain a vendored copy of
itself" is malformed, not permitted-to-fail.

Concretely: a check whose subject is the *distribution* of knowledge or the harness does
not apply to the repository that is their source. `forge knowledge check` skips such a
check when the repository under test is the upstream, and reports it as `n/a` rather than
as a pass, so the output never claims a guarantee it did not evaluate.

This is deliberately defined **here**, in the artifact about resolution, rather than as
`args: { except: [knowledge] }` on the two checks. ADR-0001 and ADR-0007 are both locked
and neither carries `args`, so the mechanical route would mean superseding two locked
ADRs to add an exception clause — expensive, and it would scatter one rule across three
artifacts. Resolution rules belong with the resolution rule.

### An unrecognised prefix is still caught

Point 4 is permissive by design, which raises the obvious worry: does a typo'd repo
name like `srv-billing` get silently under-governed, receiving only the `org` set?

No, and the reason is that the guarantee lives elsewhere. `repo.name_matches_scheme` is
an `error` check cited by ADR-0001, ADR-0009 and C-0002 — the most-cited id in the
registry. A repository whose prefix is not one of C-0002's fails that check outright.
Scope resolution does not need to double as a spell-checker, and making it one would
mean the two mechanisms could disagree.

## Consequences

**Accepted costs**

- **Two checks become vacuous at the source, and "vacuous" is a third outcome the gate
  did not previously have.** `forge knowledge check` must now distinguish pass, fail and
  not-applicable, and report the third honestly rather than folding it into pass. That is
  more reporting surface, and a category an implementer can get wrong in the direction of
  false confidence.
- **`repo.has_codeowners` now unambiguously binds this repository, which violates it.**
  There is no CODEOWNERS file anywhere in `knowledge`. Making the resolution rule
  explicit converts a hidden ambiguity into a visible failure. That is the point, but it
  is a real new red mark and it should be fixed rather than exempted.
- **The backlog grows by one** — `knowledge.scope_values_cover_repo_classes`, 103 → 104.
  Stated plainly, as ADR-0014 requires of itself.
- **The redundant eight stay redundant.** Anyone reading `scope: [org, platform, …]`
  will reasonably wonder whether the enumeration means something. It does not, and only
  this ADR says so. The alternative was eight supersessions.
- **`template` is a scope value nothing uses.** An enum entry with no referent is exactly
  the kind of thing `knowledge.orphan_check_declarations` exists to catch for check ids,
  and there is no equivalent for scope values. Added deliberately anyway, because the
  cost of the gap has now been demonstrated twice.

**Gained**

- `forge context` becomes implementable. It was not, before: no rule said what a
  classless repository resolves to, and the honest reading returned the empty set.
- The knowledge base is governed by its own constitution — identity, protected
  operations, secret scanning, line endings, changelog format — which it was not.
- `template-*` repositories become reachable by class-specific artifacts.
- The resolution rule is written down once rather than re-derived per implementation,
  which is the whole argument of ADR-0002 applied to the model itself.

**Rejected alternatives**

- *Give `knowledge` its own class value.* Does not answer the question — a rule for what
  `org` means is still required, so this is an addition rather than an alternative. Worse,
  to make any existing standard bind `knowledge` you must edit its `scope:`, and every
  standard is locked; STD-0002 would be first and it *should not* bind `knowledge`, since
  `manifest.yaml` and `ARCHITECTURE.md` are meaningless for a repository that is not a
  deployable unit. Rejected as unnecessary work that buys a capability nothing wants.
- *An unrecognised class receives every artifact — fail safe.* Superficially the cautious
  option and in fact the worst. `knowledge` would be bound by ADR-0003 (must ship a Helm
  chart), ADR-0005 (must pin a Rust toolchain), and STD-0002 (must carry a `manifest.yaml`
  declaring a class, which is circular). The gate would go permanently red with no legal
  fix — the exact condition ADR-0014 was written to escape. Fail-safe only fails safe when
  the safe direction is meaningful; here it fails into absurdity.
- *Keep `scope` as classes only and add a separate `binds_all: true` flag.* Cleaner in
  isolation: `org` stops being a class-shaped value that is not a class. But it is a
  schema change requiring a new field on nineteen artifacts, all locked — nineteen
  supersessions to express what `org` already expresses. The conceptual tidiness is real
  and it is not worth that.
- *Leave it undefined and decide inside `tool-forge`.* This is what would have happened
  by default, and it is the failure ADR-0013 exists to prevent: a load-bearing semantic
  decision made implicitly by whoever writes the resolution function, discovered later by
  reading code rather than a decision record.

## Known limitation

Nothing checks that a scope *value* has any referent. `knowledge.scope_values_cover_repo_classes`
catches the direction that bit us — a C-0002 prefix with no scope value — but not the
reverse, a scope value no repository can ever match. `template` is deliberately in that
state right now. The reverse check is not added because with one legitimate exception it
would fire immediately, and a check whose first act is to flag an intentional decision
teaches people to disable checks.
