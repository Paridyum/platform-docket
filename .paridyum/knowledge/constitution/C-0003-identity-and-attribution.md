---
id: C-0003
title: Every commit carries a real, signed, attributable identity
type: constitution
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [identity, git, signing, ai-attribution, provenance]
breaking: true
enforced_by:
  - check: git.author_email_allowlisted
    severity: error
    rationale: An unattributable commit cannot be audited, and a wrong email silently detaches work from its author on GitHub.
  - check: git.commits_are_signed
    severity: warn
    rationale: Warn not error while GitHub Free cannot require signatures on private repos (ADR-0004); promote on upgrade.
  - check: git.no_ai_authored_as_human
    severity: error
    rationale: An AI listed as Author misrepresents provenance; Co-Authored-By states it accurately.
  - check: git.conventional_commit_subject
    severity: error
---

# Constitution C-0003 — Identity and attribution

Provenance is the one property that cannot be reconstructed later. A commit whose
author is wrong is wrong permanently, and rewriting history to fix it violates the
rule that makes history worth trusting.

## Human identity

Exactly one of these, on every commit:

```
Rohan Batra <116573125+rohanbatrain@users.noreply.github.com>
Rohan Batra <rohan@rohanbatra.in>
```

Role addresses on the controlled domain are permitted where the work is genuinely
performed in that capacity:

```
platform@rohanbatra.in    security@rohanbatra.in    infra@rohanbatra.in
ai@rohanbatra.in          release@rohanbatra.in
```

Forbidden, and checked:

- Personal webmail (`gmail.com`, `outlook.com`, …) — not controlled, not attributable
  to the organisation.
- Machine-generated defaults (`rohan@Rohans-MacBook-Pro.local`) — these silently
  detach the commit from the GitHub account, so it shows as an unlinked author forever.
- Any identity belonging to another person. Never commit as someone else.

## AI attribution

An AI must never appear as **Author**. It appears as a trailer:

```
Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>
```

The human who initiated and accepted the work is the author, and remains accountable
for it. This is not a formality: the author field is who to ask when the change breaks
something in a year, and an AI cannot answer.

Equally, the trailer must not be omitted when an AI did substantial work. Both
directions are misattribution. Given that most Paridyum code will be agent-written
(ADR-0007), an honest record of which commits were agent-produced is the only way to
later ask "did the agent-heavy areas fail more often?" — a question worth being able
to answer.

Never acceptable as an identity: `AI`, `Assistant`, `Developer`, `bot`, or a
human-looking name that is not a real person.

## Signing

Every commit is signed, using SSH signing with the key already used for push
authentication:

```sh
git config --global commit.gpgsign true
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
```

The public key must be registered on GitHub **twice** — once as an authentication key,
once as a signing key. Same key, two entries. Commits show "Unverified" until the
second registration exists, which is the single most common cause of a correctly
signed commit appearing unsigned.

**Current limitation.** GitHub Free cannot *require* signatures on private
repositories (ADR-0004), so this is enforced locally and in CI, not at the merge
boundary. The check is `warn` rather than `error` for that reason, and is promoted on
plan upgrade.

## Commit subjects

Conventional Commits, because the release tooling parses them:

```
<type>(<optional scope>): <subject>

feat(auth): add refresh-token rotation
fix(billing): correct proration on mid-cycle downgrade
chore(deps): bump axum to 0.8
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`,
`chore`, `revert`. Subject in the imperative, no trailing period, under 72 characters.

`feat` and `fix` drive the minor and patch version. A breaking change is `!` after the
type plus a `BREAKING CHANGE:` footer, and drives the major version.

## Why this is constitution rather than a standard

Naming and formatting can be corrected by a follow-up commit. Identity cannot — the
only fix is a history rewrite, which is forbidden by C-0004. A rule whose violation is
permanent belongs at the highest precedence.
