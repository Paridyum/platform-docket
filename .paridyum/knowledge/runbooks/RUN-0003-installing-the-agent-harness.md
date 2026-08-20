---
id: RUN-0003
title: Installing and verifying the Paridyum agent harness
type: runbook
status: accepted
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [plugin, hooks, agent-harness, setup, verification]
breaking: false
enforced_by: []
---

# RUN-0003 — Installing and verifying the Paridyum agent harness

Layer 1 of the enforcement ladder (ADR-0007) is a Claude Code plugin. Until it is
installed, the guards in this repository are **files on disk that nothing runs**.

This lives in `runbooks/` rather than a `docs/` directory because that is where the
README already says operational procedures go, and a second documentation tree would
split the answer to "where do I look" in two.

`enforced_by` is empty because a runbook describes a procedure and cannot reject its own
violation (ADR-0002). The outcome *is* checked, by `repo.declares_paridyum_marketplace`
and `repo.enables_paridyum_platform_plugin`, both declared by ADR-0007.

## What you are turning on

| Event | Matches | Does |
| --- | --- | --- |
| `PreToolUse` | `Write` `Edit` `NotebookEdit` | **denies** edits to locked or superseded artifacts and to vendored knowledge; **asks** on governance files and on creating a new service |
| `PreToolUse` | `Bash` | **asks** before irreversible commands (C-0004) |
| `SessionStart` | every session | injects the binding rules and carried-over open loops |
| `Stop` | every session end | records unresolved decisions into `DECISION-DEBT.md` (ADR-0011) |

Plus four skills: `decompose-spec`, `design-change`, `new-decision`, `knowledge-audit`.

The `PreToolUse` guards are the only layer that stops a violation **before it is written**.
Everything else — pre-commit, CI, `forge audit` — reacts after the fact.

## Step 1 — install `jq` first, and do not skip this

**This is the step that matters.** Every guard begins with:

```sh
command -v jq >/dev/null 2>&1 || {
  echo "paridyum guard: jq not installed; locked-decision enforcement is OFF" >&2
  exit 0
}
```

Without `jq` the hooks **exit 0 and permit everything**. That is deliberate — a guard that
crashes must not wedge your session — but the consequence is that a missing dependency
turns enforcement off while the plugin still reports as installed. The warning goes to
stderr, where it is easy to miss.

```sh
command -v jq || brew install jq        # macOS
command -v jq || sudo apt install -y jq # Debian/Ubuntu
```

`scripts/bootstrap.sh` installs it as part of workstation provisioning (RUN-0001). If you
ran that, you already have it — confirm anyway.

## Step 2 — add the marketplace and install

```
/plugin marketplace add Paridyum/knowledge
/plugin install paridyum-platform@paridyum
```

`Paridyum/knowledge` is **private** (ADR-0012), so this needs working git access to it.
If the first command fails to find the repository, check `git ls-remote
git@github.com:Paridyum/knowledge.git` succeeds before assuming the plugin is at fault.

Working from a local clone instead — useful when developing the harness itself:

```
/plugin marketplace add /Users/you/Developer/paridyum/knowledge
/plugin install paridyum-platform@paridyum
```

Both resolve the same `marketplace.json` at the repository root.

## Step 3 — verify each hook actually fires

Installed is not the same as working, and this session found four separate controls that
reported success while enforcing nothing. Check rather than assume.

**Locked artifacts are denied.** Ask an agent to edit `adr/ADR-0013-change-lifecycle.md`.
Expect a refusal citing ADR-0002. If the edit goes through, the guard is not running.

**Governance files ask.** Ask it to edit `.github/workflows/knowledge-check.yml`. Expect a
prompt, not silence and not a refusal.

**Creating a service asks.** Ask it to create `svc-example/manifest.yaml`. Expect a prompt
naming RFC-0002 and asking which bounded context it is.

**Ordinary work is untouched.** Ask it to edit `README.md`. Expect nothing — no prompt.
A guard that fires on everything gets disabled within a week (ADR-0008), so this check
matters as much as the others.

You can drive the guards directly, without an agent:

```sh
cd ~/Developer/paridyum/knowledge
printf '{"tool_name":"Edit","tool_input":{"file_path":"%s/adr/ADR-0013-change-lifecycle.md"},"cwd":"%s"}' "$PWD" "$PWD" \
  | bash plugins/paridyum-platform/hooks/guard-locked-decisions.sh
```

Expect JSON containing `"permissionDecision": "deny"`. **Empty output means the guard
permitted it** — check `jq` first.

## Step 4 — install the git hooks too

The plugin covers agent sessions. It does not cover an editor, a script, or a colleague.

```sh
scripts/install-hooks.sh
```

That installs a pre-commit hook running identity, knowledge integrity, changelog format
and the secret scan. It is bypassable with `--no-verify`, which is why CI exists as a
third layer.

## When it blocks you

**It denied an edit to a locked artifact.** Working as designed. Write a new artifact that
supersedes it — `/new-decision` walks the two-step. Never edit in place; that destroys the
record of what was true when the existing code was written.

**It asked about a governance file.** Read the diff and confirm it does not weaken
enforcement. This prompt exists because a change here can silently disable every other
control.

**It asked about creating a service.** Answer the question it poses: which bounded context
is this, what does it exclusively own, and which accepted design decided that. If there
isn't one, run `/decompose-spec`. A spike is legitimate — say yes and carry on.

**It fires constantly on ordinary work.** That is a bug, not discipline, and it should be
reported rather than worked around. C-0004 states the reason: escalating routine actions
trains the habit of approving without reading, which is how a real one gets waved through.

## Known gaps

- **Nothing verifies the plugin is installed on your machine.**
  `repo.declares_paridyum_marketplace` and `repo.enables_paridyum_platform_plugin` check
  that a *repository declares* it, not that a *developer installed* it. The gap is the same
  shape as `repo.precommit_hook_installed`, which also cannot be checked from CI.
- **A missing `jq` degrades to permissive silently.** Step 1 exists because of this, and
  a check for it would be worth having.
- **The guards are not covered by automated tests.** They are verified by driving them with
  synthetic JSON, as in step 3, which is a manual procedure rather than a suite.

## Related

- ADR-0007 — the enforcement ladder and why the harness ships as a plugin
- ADR-0002 / ADR-0014 — why locked artifacts are immutable
- C-0004 — protected operations, which the Bash guard enforces
- RFC-0002 — the service-creation guard
- RUN-0001 — workstation provisioning, which installs `jq`
