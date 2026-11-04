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

## Local clones — `forge audit`

`forge audit` (ADR-0004, RFC-0060 D1) reports, for every clone in the RUN-0001 workspace, what it
holds that its remote does not: a branch other than `main`, `main` behind or ahead of
`origin/main`, uncommitted changes, stashes, live and dead worktrees, local branches already held
on `origin` and ones held nowhere else, snapshot drift from `knowledge`, and a name with no
registered prefix. It changes nothing.

```sh
cd ~/Developer/paridyum
forge audit            # as of each clone's last fetch
forge audit --fetch    # fetch first
```

Installed by `scripts/install-tools.sh forge` (RUN-0001). Its SOP in `tool-forge` says what each
finding asks of you.

**Clearing what it finds is a separate act, and only on request.** `forge tidy` prints a plan
and removes only what is provably held elsewhere — on `origin/main`, in its own origin branch,
patch-equivalent to `origin/main`, or inside a pull request's head — and only under `--apply`.
`forge knowledge sync --all` refreshes stale snapshots, each on its own branch. Neither is run as
part of an audit: the rule below — *do not fix drift silently* — still holds.

## Repositories not cloned here

`forge audit` sees the clones on this machine. A repository in the organisation that nobody has
cloned is invisible to it, so sweep the organisation's names directly.

**Derive the prefix list from `class-registry.yaml`; never restate it here.** The list is
registry data (C-0007: *the grammar is constitutional, the list is data*). This skill carried a
hardcoded seven-prefix pattern that had gone stale against the classes the registry declares —
it omitted `web-`, `console-`, `mobile-` and `sdk-`, so it reported every compliant `web-`
repository as a C-0002 violation. That is the
defect `class-registry.yaml` exists to kill, inside the tool that audits for it. Adding a class
must not require editing this file.

```sh
# Run from a repository root; resolves the registry from either the knowledge
# repository itself or a vendored snapshot.
python3 - <<'PY'
import json, re, subprocess, glob, yaml, sys
path = next((p for p in (
    'class-registry.yaml',
    '.paridyum/knowledge/class-registry.yaml',
) if glob.glob(p)), None)
if not path:
    sys.exit('class-registry.yaml not found — run from a repository root')
prefixes = [c['prefix'] for c in yaml.safe_load(open(path))['classes']]
pat = re.compile('^(' + '|'.join(re.escape(p) for p in prefixes) + ')')
repos = json.loads(subprocess.check_output(
    ['gh', 'repo', 'list', 'Paridyum', '--limit', '200',
     '--json', 'name,visibility,isArchived']))
# TWO EXCLUSIONS, both deliberate.
#
# `knowledge` is class-less by design (ADR-0015 point 6), not a violation.
#
# An ARCHIVED repository is a predecessor, not a live one. `pd-coming-soon` — the standalone
# landing superseded by web-paridyum, archived and untouched since 2026-05-01 — was the only
# thing this sweep flagged once the prefix list came from the registry, and it would have been
# flagged on every run forever. C-0002 governs what the estate NAMES; an archive is what it
# used to name, and cannot be renamed without unarchiving it to do so. Reporting it every time
# is the false positive that gets a check switched off, which this skill's own §"a prefix
# mismatch is not the same defect" already argues about.
for r in sorted(repos, key=lambda r: r['name']):
    if r['isArchived'] or r['name'] == 'knowledge':
        continue
    if not pat.match(r['name']):
        print(f"NAMING: {r['name']} ({r['visibility']}) does not match C-0002")
PY
```

**A prefix mismatch is not the same defect as a class mismatch**, and the sweep only sees the
first. A repository may carry a legitimate prefix and declare a different `class` in its
`manifest.yaml` — `tool-docket` declares `platform` and `console-docket` declares `web`, both by
ADR-0093 §3. Read each manifest's `class` against its prefix before reporting; the sweep cannot
tell you, and a rename that only fixes the directory leaves the other half behind (STD-0022).

Then per repository, `scripts/check-repo.sh --root <clone>` checks what `forge audit` does not —
`CODEOWNERS`, `.github/workflows/` and the knowledge check present, the plugin reachable — and
`forge audit` covers the snapshot: present, and identical to `knowledge` `main` file for file. The
default branch is `main` on every repository (`gh repo list Paridyum --json defaultBranchRef`).

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
