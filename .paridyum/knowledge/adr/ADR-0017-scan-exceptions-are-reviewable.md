---
id: ADR-0017
title: A secret-scan exception is a tracked fingerprint, never an inline comment
type: adr
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [security, secrets, c-0004, enforcement, escape-hatch]
breaking: false
enforced_by:
  - check: repo.no_inline_scan_suppressions
    severity: error
    rationale: The flag makes the comment ineffective; this makes it visible. A stale suppression left in the tree becomes a live bypass again the day someone drops the flag.
---

# ADR-0017 — A secret-scan exception is a tracked fingerprint, never an inline comment

## Status

**LOCKED** as of 2026-08-06.

## Context

C-0004 declares `repo.secret_scan_clean` at `error` severity, and it is the substitute
for GitHub's own secret scanning, unavailable on Free private repositories (ADR-0004).
Getting it to work took four attempts — an uncompilable regex, a capture group that made
gitleaks report the URI scheme as the secret, a licensed action wrapper, and a missing
job in `rust-ci.yml`.

It was then bypassable with a comment. Appending `# gitleaks:allow` to a line containing
a real password makes the scan exit 0. Verified directly: the same planted
`postgres://admin:…@host` passes with the comment and fails without it, in every layer —
pre-commit hook, `knowledge-check.yml`, and the reusable workflow.

gitleaks ships `--ignore-gitleaks-allow` to disable the behaviour. It was used nowhere.

### Why this matters more here than elsewhere

The premise of this knowledge base is that agents drift, and that a rule is only real if
something rejects its violation. An agent that hits a red secret scan and needs to
proceed has, in `# gitleaks:allow`, a fix that is one token long, looks idiomatic,
requires no permission, and leaves no artefact a reviewer would notice. It is the single
cheapest way to defeat a control in the repository.

Nothing about this is specific to gitleaks. **Infisical's scanner honours
`infisical-scan:ignore` for the same purpose**, so migrating engines would reproduce the
hatch under a new name — and a comment already in the tree would survive the migration.
This is a property of the category.

### Exceptions are still necessary

Removing the hatch outright is wrong. Real false positives exist: an example connection
string in a test fixture, a public key that pattern-matches a private one, a
high-entropy string that is genuinely not a secret. A control with no exception path
gets disabled wholesale the first time it is wrong, which is worse than a control with a
reviewable one.

So the question is not whether to allow exceptions. It is **where an exception lives**.

## Decision

**Inline suppression comments are disabled. An exception is a fingerprint entry in
`.gitleaksignore`, at the repository root, tracked in git.**

1. Every invocation of the scan passes `--ignore-gitleaks-allow`. All three layers call
   one script (`templates/infra-ci/scripts/secret-scan.sh`), so this is one change, not
   three.
2. `.gitleaksignore` is the sole legitimate exception path. An entry is a fingerprint —
   `<file>:<rule-id>:<line>` — naming one finding in one place.
3. Every entry carries a comment above it stating why the finding is not a real secret.
   An unexplained entry is indistinguishable from a smuggled one, and reviewers wave
   those through.
4. `repo.no_inline_scan_suppressions` fails the build when `gitleaks:allow` or
   `infisical-scan:ignore` appears in a tracked file.

### Why a fingerprint file beats a comment

Three properties a comment does not have:

- **It appears in a diff.** Adding one is a visible act with an author and a timestamp.
  A comment appended mid-file inside an unrelated change is not.
- **It is pinned to a line.** The exception lapses when the file shifts, rather than
  silently continuing to permit whatever moves into its place.
- **It is countable.** `wc -l .gitleaksignore` is an honest measure of how much has been
  waived. Suppressions scattered across a tree are not measurable, and what is not
  measurable does not get paid down.

This is ADR-0002's argument applied to exceptions rather than rules: the value is not
that violation is impossible, it is that it is *visible*.

### Point 4 is not redundant with point 1

The flag makes the comment ineffective. The check makes it absent. Both are needed,
because a stale `# gitleaks:allow` left in the tree reads as protection someone relied
on, and becomes a live bypass the moment anyone drops the flag while tidying a command
line. The flag defends today; the check defends the next edit to the script.

## Consequences

**Accepted costs**

- **The backlog grows by one** — `repo.no_inline_scan_suppressions`, 105 → 106.
- **Adding an exception is now more work**, deliberately: run the scan, copy the
  fingerprint, write a justification. That friction is the mechanism, not a side effect.
  It is also the point at which someone notices they are waiving a real secret.
- **Fingerprints are brittle by design.** They contain a line number, so ordinary edits
  invalidate them and a previously-suppressed finding returns. That is correct — the
  suppression was for one finding at one place, and the place moved — but it will
  occasionally look like the check "broke" when a file was merely reformatted.
- **The check must exclude the files that define the rule**, and the list is longer than
  it first appears: `secret-scan.sh`, `.gitleaksignore`, `check-registry.yaml` (whose
  `spec` for this very check must quote both keywords) and every `*.md` including this
  ADR. Scanning them makes the check self-inflicting — permanently red and unfixable, the
  same trap `check-knowledge.sh` documents at length for `DECISION-DEBT.md`. The first
  implementation here omitted `check-registry.yaml` and went red on itself immediately,
  which is a fair warning about how easy this is to get wrong. The exclusion list is
  load-bearing and must not be "cleaned up"; anything added to it needs the same
  justification — that the file's purpose requires naming what it forbids.

**Gained**

- The cheapest bypass in the repository is closed, and closed in one place rather than
  three.
- Exceptions become an auditable, countable list instead of an unknown number of
  comments scattered through the tree.
- The rule survives an engine migration: the check names Infisical's keyword too, so a
  comment cannot ride along into a new scanner.

**Rejected alternatives**

- *Migrate to Infisical's scanner.* Considered directly, since Infisical was on the
  table for secret management. It does not fix this: `infisical-scan:ignore` is the same
  hatch, and `.infisicalignore` is the same fingerprint file. Migration would add a
  dependency and a second config while leaving the defect exactly where it was.
- *Remove the escape hatch entirely, no exception path.* Rejected: false positives are
  real, and a control that cannot be excepted gets disabled wholesale the first time it
  is wrong. That trades a narrow, visible hole for a total one.
- *Allow inline comments but require a justification in the same comment.* Rejected:
  unenforceable in any useful way — any string satisfies it — and it keeps the two
  properties that make comments bad, invisibility in review and uncountability.
- *Keep inline comments and audit them periodically with `forge audit`.* Rejected: a
  control whose enforcement is a scheduled human review is an honour system. C-0004 is
  the wrong place for one.
- *Set `--ignore-gitleaks-allow` only in CI, leaving the hatch for local runs.*
  Rejected, and this was the tempting one. It would let a developer move fast locally,
  but it recreates the local/CI divergence that took four attempts to eliminate from
  this exact check, and it means the pre-commit hook passes on something CI will reject —
  the worst possible place to disagree.
