---
id: ADR-0036
title: A check's parameters come from `args`, and a multi-citer conflict is resolved in the registry
type: adr
status: accepted
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [checks, registries, enforcement, precedence, drift]
breaking: false
enforced_by:
  - check: knowledge.no_unresolved_check_parameter_conflict
    severity: error
    rationale: >
      The defect is silent by construction: two locked artifacts each look correct in
      isolation, the implementation quietly picks a third value, and nothing compares the
      three. It stayed hidden until an audit read both artifacts and the script in one sitting.
      `error` rather than `warn` because unlike a stale count, a conflict here means the gate
      is enforcing something no artifact asked for.
---

# ADR-0036 — Check parameters come from `args`

## Status

**ACCEPTED** as of 2026-08-08.

## Context

`repo.claude_md_under_line_budget` is cited by two **locked** artifacts, and they disagree:

| Source | Severity | Threshold |
| --- | --- | --- |
| ADR-0007 `enforced_by` | `warn` | `args: { max_lines: 200 }` |
| ADR-0026 `enforced_by` | `error` | *no args* |
| ADR-0026 prose, line 83 | — | "**150-line budget**, down from 250" |
| `scripts/check-repo.sh:63` | `err` | `MAX_CLAUDE_LINES=150` |

Four sources, three thresholds, two severities. `check-registry.yaml`'s header states the design
that produced it — *"Severity is deliberately NOT stored here. It lives in each citing artifact's
`enforced_by`"* — a model that is correct and assumes exactly one citer. Nothing ever checked
that assumption, and nothing compares citers to each other or to the implementation.

CLAUDE.md's precedence order does not help. It ranks *tiers* — constitution over locked ADR over
standard — and says nothing about two artifacts at the same tier. C-0001 solved this inside the
constitution with "the lower-numbered principle wins" and the rule was never generalised.

The sharp detail, and the reason this resolves without a supersession: **ADR-0026 declares no
`args`.** Its 150 exists only in prose. So the implementation took a parameter from a sentence,
and `max_lines: 200` is the only threshold any artifact has ever made machine-readable.

ADR-0026 anticipated this, at line 116: *"150 lines is arbitrary and will eventually be wrong.
It is one line to change here, which is the point; the failure to avoid is exceptions
accumulating against a number nobody revisits."* Revision is expected. Exceptions are the thing
it guards against, and none is being added.

## Decision

**1. A check's parameters come from `args` in a citing artifact's `enforced_by`. Never from
prose.** A number in a paragraph is rationale — it explains why the parameter was chosen and does
not set it. An implementation that reads a threshold out of prose is inventing a rule nobody
declared, which is what happened here.

**2. Where more than one artifact cites the same check, `check-registry.yaml` records the
resolved severity and args**, in a `resolution:` block naming the citers and the reasoning. The
resolution binds; the individual citations become the inputs to it.

**3. Absent a resolution, conflicting citations are a hard failure.** Not a silent pick.
`knowledge.no_unresolved_check_parameter_conflict` compares every citation of every check.

**4. This check resolves to `severity: error`, `max_lines: 200`.** Severity from ADR-0026, which
is stricter and is the artifact specifically about CLAUDE.md's shape. Threshold from ADR-0007,
which is the only `args` ever declared. The implementation moves from 150 to 200.

**5. Nothing is superseded.** ADR-0007's `args` and ADR-0026's severity are both adopted as
written. ADR-0026's prose remains the rationale for a smaller budget and stops being read as the
parameter, which is what decision 1 says it never was.

## Consequences

**Accepted costs**

1. **The budget loosens from 150 to 200, and CLAUDE.md is at 148.** It gains 52 lines of room it
   did not have this morning, and ADR-0026's argument against a slack budget — *"a budget at twice
   the actual size constrains nothing"* — applies at 200 more than it did at 150. This is the real
   cost of the ruling and it is not mitigated, only recorded.

2. **A third place to look.** A reader now checks the artifact, and then the registry when a check
   has more than one citer. Better than three places that disagree, worse than one.

3. **`resolution:` is a new schema field** and a new way for the registry to be wrong.

4. **The rule is asymmetric and deliberately so.** Severity resolved to the stricter, threshold to
   the only declared value. Those are different principles, and a future conflict may not have
   such a clean answer — decision 3 makes that stop the build rather than resolve itself.

**Gained**

- The implementation stops being a third opinion. Whatever the gate enforces is now something an
  artifact actually declared.
- The class is detected rather than audited: a future double-citation with different parameters
  fails at the gate instead of surviving until somebody reads both files at once.
- No supersession, no erratum, and two locked artifacts keep every word they say.

**Rejected alternatives**

- **Move severity and args into `check-registry.yaml` for every check.** The structurally correct
  answer and the one matching five prior registry migrations. Rejected because `severity` sits in
  the frontmatter of dozens of artifacts, most of them locked and therefore uneditable — the
  migration cannot be completed, and a half-migrated parameter is worse than an unmigrated one.

- **Strictest citation always wins.** Simple, and it gives 150 here, contradicting the ruling and
  ADR-0007's only declared value. It also has no answer when the strict/permissive axis is
  undefined, as with two different string arguments.

- **Most recent citation wins.** Would give ADR-0026 both fields and make the parameter depend on
  artifact numbering, which is an ordering nobody chose for this purpose.

- **Supersede ADR-0026.** Correct if 150 were a decision. It is prose explaining a parameter the
  artifact never declared, and rewriting a whole ADR to change a number it did not set would be
  ceremony standing in for a fix.

## Related

- ADR-0014 — the check registry; this adds the reconciliation the one-citer model assumed away
- ADR-0035 — the sibling failure: a number in prose that nothing keeps true
- ADR-0007, ADR-0026 — the two citers, both intact
- C-0001 — "the lower-numbered principle wins", the same problem solved inside the constitution
- CLAUDE.md — the precedence order, which ranks tiers and not peers
