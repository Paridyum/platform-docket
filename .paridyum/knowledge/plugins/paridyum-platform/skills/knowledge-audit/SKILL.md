---
name: knowledge-audit
description: Audit Paridyum knowledge integrity and repository compliance. Use when checking for stale vendored snapshots, unenforced locked decisions, dangling references, or repos drifting from the naming scheme.
---

# Audit Paridyum knowledge

Reports drift. Run before locking a new decision, after syncing snapshots, and when
a repository has been untouched for a while.

## Knowledge base integrity

```sh
scripts/check-knowledge.sh
```

Reports, in order of severity:

1. **Invalid frontmatter** — an artifact the gate cannot parse is one it cannot
   enforce. Hard failure.
2. **Dangling references** — an `ADR-00NN` cited in prose with no such artifact.
   Usually a decision that was discussed but never written down.
3. **Unresolvable check ids** — an `enforced_by.check` not in `tool-forge`'s
   registry. Hard failure by design: a typo'd check id would silently enforce
   nothing, which is worse than no rule at all.
4. **Locked but unenforced** — `status: locked` with empty `enforced_by`. This is
   the enforcement backlog. Expect it to be non-empty; the point is that it is
   *visible* rather than assumed. Do not "fix" these by downgrading them to
   `accepted` unless the rule genuinely cannot be checked.

## Repository compliance

Until `forge audit` ships, sweep the org directly:

```sh
gh repo list Paridyum --limit 200 --json name,visibility,defaultBranchRef \
  --jq '.[] | select(.name | test("^(platform|svc|lib|tool|infra|spec|template)-") | not)
        | "NAMING: \(.name) does not match C-0002"'
```

Then per repository, check that:

- `.paridyum/knowledge/` exists and is newer than 30 days (ADR-0001)
- the vendored snapshot is unmodified relative to `Paridyum/knowledge`
- `CODEOWNERS`, `.github/workflows/`, and the knowledge check are present
- the default branch is `main`

## Why this matters more than usual here

The organisation is on GitHub Free, where private repositories cannot require a
status check before merge (ADR-0004). CI reports red but the merge proceeds. That
makes this audit the only layer that catches a violation which reached `main` —
it is a backstop, not a nicety.

The agent-side hooks (ADR-0007) block violations before they are written, which
covers work done through Claude Code. This audit covers everything else.

## Reporting

State findings plainly, most severe first, with the artifact id or repository name
and the rule violated. Do not fix drift silently — a repository that has drifted
may have drifted for a reason worth knowing.
