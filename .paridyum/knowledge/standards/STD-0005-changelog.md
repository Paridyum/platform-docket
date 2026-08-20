---
id: STD-0005
title: Every repository keeps a curated changelog in Keep a Changelog 2.0.0 format
type: standard
status: locked
created: 2026-08-06
locked_at: 2026-08-06
supersedes: null
superseded_by: null
scope: [org, platform, service, library, tool, infra, spec]
applies_to: ["*"]
tags: [changelog, releases, documentation, keepachangelog]
breaking: true
enforced_by:
  - check: changelog.file_present
    severity: error
    args: { path: "CHANGELOG.md" }
  - check: changelog.format_valid
    severity: error
    args: { spec: "keepachangelog-2.0.0" }
  - check: changelog.only_canonical_categories
    severity: error
    args: { allowed: ["Added", "Changed", "Deprecated", "Removed", "Fixed", "Security"] }
  - check: changelog.dates_are_iso
    severity: error
  - check: changelog.unreleased_section_present
    severity: error
  - check: changelog.no_raw_commit_dump
    severity: warn
    rationale: Entries that are verbatim commit subjects with hashes indicate a generated file nobody curated.
  - check: changelog.removed_was_deprecated_first
    severity: warn
    rationale: A Removed entry with no prior Deprecated release breaks upgraders without warning.
---

# STD-0005 — Changelog

Every repository carries `CHANGELOG.md`, following
[Keep a Changelog 2.0.0](https://keepachangelog.com/en/2.0.0/) — the June 2026
revision, the first since 1.0.0 in 2017.

## The format

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- Refresh-token rotation on the `/v1/sessions` endpoint.

## [1.2.0] - 2026-08-06

### Changed
- **Breaking:** `settle()` now requires a `PaymentIntent` rather than an amount.

### Fixed
- Proration was computed against the wrong period on mid-cycle downgrade.

### Security
- CVE-2026-1234: bumped `tokio` to 1.42.1, closing a broadcast-channel leak.

[Unreleased]: https://github.com/Paridyum/svc-billing/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/Paridyum/svc-billing/compare/v1.1.0...v1.2.0
```

**Six categories, and only six**: `Added`, `Changed`, `Deprecated`, `Removed`,
`Fixed`, `Security`. The spec is explicit that there are six on purpose. Dependency
bumps and performance work are `Changed` or `Fixed` — not new headings. A repository
that invents `Performance` or `Dependencies` starts a taxonomy that diverges across
forty repositories.

Dates are `YYYY-MM-DD`. Security entries with a CVE lead with the identifier so
readers and scanners can match the advisory. Breaking changes carry a `**Breaking:**`
prefix. Comparison links go at the bottom.

## The rule that is easy to get wrong

**A changelog edit is not a required check on every pull request.**

This is counter-intuitive and it is the spec's own guidance:

> Do not make a changelog edit a required check on every change. That teaches people
> to add a line to pass the check, which fills the changelog with noise.

The instinct is to gate every PR on a changelog entry. It backfires: contributors add
a line to make CI green, and within months the file is a list of entries written to
satisfy a robot. That is strictly worse than a shorter, curated file, because a
changelog that records everything is as useless as one that records nothing — the
reader can no longer find what mattered.

**What is enforced instead:**

| Enforced | Not enforced |
| -------- | ------------ |
| The file exists | That this PR edited it |
| The format is valid | That every change appears |
| Only the six categories | Entry wording |
| Dates are ISO | Entry count |
| An `Unreleased` section exists | |

The judgement — *is this notable to a reader?* — stays with the author, prompted by
the PR template checklist rather than blocked by CI.

## Notable means notable

A changelog records what a **reader** needs to know, not what happened in the
repository. It is partly an act of restraint.

**Include:** new capability; behaviour that changed; anything breaking; deprecations;
security fixes; bugs a user could have hit; config, schema, or API changes.

**Exclude:** internal refactors with no observable effect; test changes; formatting;
CI edits; dependency bumps with no user-visible consequence.

One entry frequently spans several commits, and many commits deserve no entry. That
asymmetry is why a generated changelog is raw material rather than output.

## Automation, and its boundary

> Machines can draft, but humans curate.

`release-plz` and `git-cliff` (ADR-0010) draft from Conventional Commits and open a
**Release PR**. That PR is where curation happens — the draft is reviewed and rewritten
for the reader before it becomes permanent. Automation handles the mechanical part:
moving `Unreleased` into a dated version at release, updating comparison links,
validating format.

A generated entry that survives to `main` unedited is usually a commit subject in
disguise. The `changelog.no_raw_commit_dump` check flags entries carrying commit
hashes or bare Conventional Commit prefixes.

## Brief for agents

Keep a Changelog 2.0.0 explicitly recommends documenting the brief for coding agents
in `CLAUDE.md`. Paridyum's, reproduced there:

> Summarise notable, user-facing changes. Do not paste a git log. Sort each change
> into one of the six types. Explain the reason in the text, not just the mechanism.
> Mark breaking changes. If a change is not notable to a reader, leave it out.

An LLM can draft a fluent changelog from a diff in seconds, which makes it newly easy
to publish one nobody has read. The draft is a starting point.

## Deprecation before removal

Mark a feature `Deprecated` in one release and `Removed` only in a later one, so
anyone upgrading meets the warning before the breakage. A `Removed` entry with no
prior `Deprecated` release is flagged — it is the difference between an upgrade that
warns and one that fails.

## Monorepo note

Not currently applicable — Paridyum is polyrepo (ADR-0001), so each repository keeps
its own changelog, which is the spec's guidance for unrelated projects. Should a
workspace ever hold several published crates, each keeps its own **and** the
repository keeps a central one; readers should not have to assemble a release from a
dozen component files.
