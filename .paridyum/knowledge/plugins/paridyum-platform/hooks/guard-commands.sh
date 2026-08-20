#!/usr/bin/env bash
#
# PreToolUse guard for Bash — dangerous and irreversible operations (C-0004).
#
# Decision model, and the reason this hook is mostly "ask" rather than "deny":
#
#   deny  — never legitimate. History rewrites on a shared branch.
#   ask   — legitimate sometimes, but the blast radius warrants a human. Most
#           entries here. An agent cannot self-approve; the user sees the exact
#           command before it runs.
#
# A guard that denies everything dangerous gets disabled within a week, and a
# disabled guard protects nothing. Escalation preserves the check while keeping
# the work possible.
#
# Exits 0 on internal error: a crashed guard must not wedge the session.

set -uo pipefail

decide() {
  jq -n --arg d "$1" --arg r "$2" '{
    hookSpecificOutput: {
      hookEventName: "PreToolUse",
      permissionDecision: $d,
      permissionDecisionReason: $r
    }
  }'
  exit 0
}
deny() { decide deny "$1"; }
ask()  { decide ask  "$1"; }

command -v jq >/dev/null 2>&1 || {
  echo "paridyum guard: jq missing; dangerous-command enforcement is OFF" >&2; exit 0; }

payload="$(cat)"
jq -e . >/dev/null 2>&1 <<<"$payload" || {
  echo "paridyum guard: unparseable hook input; command enforcement SKIPPED" >&2; exit 0; }

[[ "$(jq -r '.tool_name // empty' <<<"$payload")" == "Bash" ]] || exit 0
cmd="$(jq -r '.tool_input.command // empty' <<<"$payload")"
[[ -z "$cmd" ]] && exit 0

# Strip quotes so `git push "--force"` cannot slip past a literal match.
norm="$(tr -d "\"'" <<<"$cmd" | tr -s '[:space:]' ' ')"

# --- never legitimate -------------------------------------------------------
if grep -qE '(^|[;&|] *)git +push +([^;&|]* )?(--force|-f)( |$)' <<<"$norm" \
   && grep -qE '(origin|upstream)? *(main|master)( |$)' <<<"$norm"; then
  deny "LAW: history is sacred. Force-pushing to main rewrites published history and breaks every existing clone. If a commit must be undone, revert it — the revert is itself part of the record."
fi

for pat in 'git +filter-branch' 'git +filter-repo' 'bfg' 'git +reflog +expire.*--expire=now'; do
  grep -qE "$pat" <<<"$norm" && \
    deny "LAW: history is sacred. History-rewriting tools are not permitted on shared branches. If secrets were committed, rotate them — rewriting history does not un-publish what was already fetched."
done

# --- irreversible, escalate -------------------------------------------------
case "$norm" in
  *"git push --force"*|*"git push -f"*)
    ask "Force-push rewrites published history on the target branch. Confirm the branch is unshared." ;;
esac

grep -qE '(^|[;&|] *)git +reset +--hard' <<<"$norm" && \
  ask "git reset --hard discards uncommitted work irrecoverably. Confirm nothing unstaged matters."

# rm -rf: only escalate when the target is absolute, home-anchored, a glob, or
# the filesystem root. `rm -rf ./target` and `rm -rf node_modules` are routine
# and escalating them would train the habit of approving without reading.
if grep -qE '\brm\b[^;&|]*-[a-zA-Z]*[rR][a-zA-Z]*f|\brm\b[^;&|]*-[a-zA-Z]*f[a-zA-Z]*[rR]' <<<"$norm"; then
  # Match on how the ARGUMENT STARTS, not on any character within it. Testing for
  # "/" anywhere flags `rm -rf ./target`, and escalating routine build cleanup is
  # how approving without reading becomes a habit.
  args="${norm#*rm }"
  if grep -qE '(^|[[:space:]])(/|~|\$HOME|\$\{HOME\})' <<<"$args" \
     || grep -qE '(^|[[:space:]])[^[:space:]]*\*' <<<"$args"; then
    ask "Recursive delete targeting an absolute path, home directory, or glob. Verify the target before this runs — there is no undo."
  fi
fi

# --- infrastructure ---------------------------------------------------------
grep -qE '\bterraform +(apply|destroy)' <<<"$norm" && \
  ask "terraform apply/destroy mutates real infrastructure (ADR-0003). Review the plan output first; destroy is unrecoverable."

grep -qE '\bkubectl +delete' <<<"$norm" && \
  ask "kubectl delete removes live cluster resources. Confirm the context and namespace — a wrong --context deletes from the wrong environment."

grep -qE '\bkubectl +.*(--context|config +use-context) +[^ ]*prod' <<<"$norm" && \
  ask "This targets a production context. Confirm this is intended."

grep -qE '\b(sqlx +migrate +(run|revert)|alembic +(upgrade|downgrade)|diesel +migration +(run|revert))' <<<"$norm" && \
  ask "Database migration. Confirm the target database and that a rollback path exists."

grep -qE '\bhelm +(upgrade|uninstall|rollback)' <<<"$norm" && \
  ask "Helm release mutation against a live cluster. Confirm the context and release."

# --- credentials ------------------------------------------------------------
grep -qE '\bgh +secret +(set|delete)|\bkubectl +.*secret|\bopenssl +.*genrsa|\bssh-keygen\b' <<<"$norm" && \
  ask "Credential or secret operation. Confirm the target and that rotation is coordinated."

# --- repository governance (C-0004) ----------------------------------------
grep -qE '\bgh +(repo +delete|api +.*-X *DELETE)' <<<"$norm" && \
  ask "Destructive GitHub API call. Repository and org-level deletions are not reversible."

grep -qE '\bgh +api +.*(branch_protection|rulesets|/orgs/Paridyum/)' <<<"$norm" && \
  ask "This modifies organisation or branch-protection settings — the governance layer itself (ADR-0004). Changes here weaken every other control."

exit 0
