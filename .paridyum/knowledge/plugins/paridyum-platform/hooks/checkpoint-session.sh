#!/usr/bin/env bash
#
# Stop hook — durability checkpoint (ADR-0025).
#
# Runs the full gate pipeline, then commits and pushes everything. The reason is not tidiness:
# work in this repository has been lost from local disk once, and DECISION-DEBT.md has been
# reporting "governance edits that exist on one disk only" ever since. A knowledge base whose
# whole purpose is that rules survive people is worthless if it does not survive a disk.
#
# THE ORDERING IS THE DESIGN, and it is the opposite of a normal pre-commit gate:
#
#   secret scan   -> BLOCKS the commit. The only blocker.
#   every other   -> RECORDED in the commit message, never blocking.
#
# A gate that refuses to commit failing work protects the history and loses the work, which is
# precisely backwards when the failure mode being defended against is data loss. A red commit
# that exists beats a green working tree that does not. Secrets are the single exception,
# because committing a credential is the one outcome that a later commit cannot undo — it is in
# the history, and history is what an attacker clones (C-0004, repo.secret_scan_clean).
#
# Exits 0 unconditionally, like capture-open-loops.sh. A Stop hook that fails must never
# disrupt the session — and one that blocks the session to protect a branch would have
# inverted its own purpose.

set -uo pipefail

secret_scan_absent=0
payload="$(cat 2>/dev/null || true)"
command -v jq >/dev/null 2>&1 || exit 0
jq -e . >/dev/null 2>&1 <<<"$payload" || exit 0

cwd="$(jq -r '.cwd // empty' <<<"$payload")"; [[ -z "$cwd" ]] && cwd="$PWD"
session="$(jq -r '.session_id // "unknown"' <<<"$payload")"

R="$(git -C "$cwd" rev-parse --show-toplevel 2>/dev/null)" || exit 0
[[ -n "$R" ]] || exit 0
cd "$R" || exit 0

# Opt-out, for the case this hook cannot anticipate: a rebase, a bisect, a deliberate
# experiment nobody wants in the history. An automation with no escape hatch gets disabled
# wholesale, and then it protects nothing.
[[ -f "$R/.paridyum-no-checkpoint" ]] && exit 0
[[ -n "${PARIDYUM_NO_CHECKPOINT:-}" ]] && exit 0

# Mid-operation states. Committing during a rebase or merge rewrites somebody's intent.
for f in rebase-merge rebase-apply MERGE_HEAD CHERRY_PICK_HEAD BISECT_LOG; do
  [[ -e "$R/.git/$f" ]] && exit 0
done

# Nothing to preserve, nothing to do. This is what keeps a Stop hook that fires on every
# response from producing a commit on every response.
[[ -n "$(git status --porcelain 2>/dev/null)" ]] || exit 0

say() { printf 'paridyum checkpoint: %s\n' "$1" >&2; }

# Two sessions in one repository both fire this hook. Tested: one wins, the other hits git's own
# index.lock and no-ops, nothing is lost (the loser's work is picked up by the next checkpoint,
# which fires on every response) and no lock is left behind. The only defect was noise — the loser
# leaked git's raw `fatal: Unable to create index.lock` before its own message.
#
# This pre-flight makes that outcome explicit rather than accidental. It is deliberately NOT a
# mutex: check-then-act is not atomic, so git's own locking remains the real guard. macOS ships no
# `flock`, and a portable mkdir-based lock would add a stale-lock failure mode worse than the noise
# it removes.
# --- 1. the blocking gate: secrets ------------------------------------------
SCAN="$R/templates/infra-ci/scripts/secret-scan.sh"
if [[ -x "$SCAN" ]]; then
  if ! scan_out="$("$SCAN" --dir "$R" 2>&1)"; then
    say "SECRET SCAN FAILED — nothing was committed, and this is the one gate that blocks."
    say "A credential in history cannot be removed by a later commit (C-0004)."
    printf '%s\n' "$scan_out" | tail -15 >&2
    say "Rotate the credential, remove it from the tree, then re-run. Your work is still on disk."
    exit 0
  fi
else
  say "secret-scan.sh not found or not executable — committing WITHOUT a secret scan."
  # Recorded in the commit body, not only on stderr. C-0004 makes a credential in history
  # unrecoverable, so a commit made without the one blocking gate must be distinguishable
  # from one that passed it -- forever, and by someone reading the log years later.
  secret_scan_absent=1
fi

# --- 2. the recorded gates: everything else ---------------------------------
# Results go into the commit message rather than into a decision about whether to commit.
# `git log --grep 'gates: .*FAIL'` is then the list of checkpoints that need attention, which
# is a better artifact than a working tree that was never saved.
gate_lines=""; failed=0; absent=0
run_gate() {
  local name="$1"; shift
  # ABSENT IS NOT PASS. check-migrations.sh states this principle for its own empty-input case
  # -- "a gate that prints 'passed' over an empty input set is the defect class the ADR-0014
  # audit found four separate times" -- and this hook violated it: a missing gate script
  # incremented nothing, so a checkpoint that ran zero gates still recorded `green`.
  if [[ ! -x "$1" ]]; then gate_lines+="  ${name}: ABSENT (not run)"$'\n'; absent=$((absent+1)); return; fi
  # OUTPUT IS CAPTURED, NOT DISCARDED (ADR-0059). This was `>/dev/null 2>&1` with only the exit
  # code kept, and every gate here exits 0 on a warning by design -- so roughly a dozen `warn`
  # checks were reachable from nowhere. repo.plugin_snapshot_matches_head reported the installed
  # plugin as twelve days stale, by name and with the remedy, into this redirect: RFC-0027
  # research capture had never run once while every checkpoint recorded `repo: pass`.
  local out rc
  out="$("$@" 2>&1)"; rc=$?
  if [[ $rc -eq 0 ]]; then
    # The gate's own `!` prefix is the warning marker; counting it here avoids each gate needing
    # to agree on a machine-readable summary line it does not currently emit.
    # Output is captured, so it is not a tty and the gates emit no colour: a warning line is
    # literally `! text` and an error line `\u2718 text`. Matching the plain form rather than an
    # escape sequence is deliberate -- the colour branch is unreachable here by construction.
    local w
    w="$(grep -c '^! ' <<<"$out" 2>/dev/null || true)"; w="${w:-0}"
    if [[ "$w" -gt 0 ]]; then
      gate_lines+="  ${name}: pass (${w} warning(s))"$'\n'
      # Into the commit message, where `git log --grep` can find it later, AND onto stderr, where
      # the operator is looking now. A channel that only reaches the log is half a channel.
      while IFS= read -r line; do
        [[ -n "$line" ]] && gate_lines+="    ! ${line}"$'\n'
      # FIRST LINE ONLY. Some `wrn` calls span several lines (the plugin-not-installed one
      # carries its two-command remedy on continuation lines), and those do not carry the `!`
      # prefix so they are dropped here. The check id and the condition survive, which is what
      # makes the warning findable; the full text is one `scripts/check-repo.sh` away. Recorded
      # rather than solved with a fragile continuation parser.
      done < <(grep '^! ' <<<"$out" | sed 's/^! *//')
      say "${name} gate passed with ${w} warning(s) -- see the checkpoint commit message"
    else
      gate_lines+="  ${name}: pass"$'\n'
    fi
  else
    gate_lines+="  ${name}: FAIL"$'\n'; failed=$((failed+1))
    # A failing gate's output was discarded too, which made `git log --grep 'FAIL'` a list of
    # checkpoints with no way to tell what failed without re-running the gate at that commit.
    while IFS= read -r line; do
      [[ -n "$line" ]] && gate_lines+="    x ${line}"$'\n'
    done < <(grep '^✘ ' <<<"$out" | sed 's/^. *//' | head -20)
  fi
}

# test-checks.sh is DELIBERATELY NOT RUN HERE, and it was until an audit measured it: 90 seconds,
# because it invokes check-knowledge.sh once per planted defect. This hook fires on every response,
# so that made every interaction cost ~96s against a 120s timeout — one slow addition away from the
# hook being killed mid-commit, which is the one failure mode ADR-0025 cannot tolerate.
#
# The split is not just about speed. This hook exists for DURABILITY: get the work off the disk.
# Proving the gates still reject what they claim is VERIFICATION, it belongs where latency is
# invisible, and it runs in CI on every push and pull request. A durability mechanism that makes
# every interaction three times slower gets switched off, and then it preserves nothing.

# --- 3. stage, then gate -----------------------------------------------------
# ORDER CHANGED, and the order is the point. Gates used to run BEFORE `git add -A`, so a
# concurrent session writing to the tree during the ~6s gate pipeline had its file staged and
# committed with a `green` verdict that provably predated it. Staging first freezes the content
# the verdict is about. The residual is smaller and points the safe way: a peer writing AFTER
# the snapshot changes what the gates read, but cannot change what was committed.
#
# The lock pre-flight lives here, immediately before the first index write, rather than 6
# seconds earlier where it was checking a condition that had ample time to change. It is still
# deliberately NOT a mutex -- git's own locking remains the real guard -- but the window it
# narrows is now milliseconds instead of the whole gate pipeline.
if [[ -f "$R/.git/index.lock" ]]; then
  say "another git process holds the index — skipping this checkpoint. Nothing is lost: the next"
  say "response fires this hook again, and your work is still on disk."
  # The loser is the ONLY component that positively detects a concurrent session, and it used to
  # discard that knowledge. Leave it for the next checkpoint to record.
  date -u +%Y-%m-%dT%H:%M:%SZ > "$(cd "$(git rev-parse --git-common-dir)" && pwd)/paridyum-concurrent" 2>/dev/null || true
  exit 0
fi

# ADR-0043 narrows the entanglement to a shared INDEX rather than a shared branch, and this is
# where the residual lives. A linked worktree has its own index, so a session working in one
# cannot stage a peer's files no matter what `git add -A` does; the main worktree shares an index
# with every other session in it.
#
# `--git-dir` and `--git-common-dir` are equal in the main worktree and differ in a linked one,
# which makes the distinction free to detect. Recorded rather than enforced: refusing to run
# outside a worktree would disable durability for the ordinary single-session case, which is the
# failure ADR-0025 exists to prevent.
GIT_DIR_P="$(git rev-parse --absolute-git-dir 2>/dev/null)"
GIT_COMMON="$(cd "$(git rev-parse --git-common-dir 2>/dev/null)" 2>/dev/null && pwd)"
if [[ -z "$GIT_COMMON" ]]; then GIT_COMMON="$GIT_DIR_P"; fi
if [[ "$GIT_DIR_P" == "$GIT_COMMON" ]]; then
  SHARED_INDEX=true
else
  SHARED_INDEX=false
fi

# The breadcrumb lives in the COMMON dir, not in "$R/.git". In a linked worktree `.git` is a
# FILE pointing at the common directory, so the original path silently resolved to nothing and
# a peer observed from a worktree was never recorded. Found by driving both cases rather than by
# reading — the main-worktree path worked, which is exactly why the other one went unnoticed.
# It also belongs there on the merits: a concurrent session is a property of the repository, not
# of one working copy.
CONCURRENT_FLAG="$GIT_COMMON/paridyum-concurrent"

git add -A || { say "git add failed; nothing committed"; exit 0; }

# Belt and braces: --dir scans the working tree, --staged scans what is actually going in.
# They differ when a file is ignored by one path and not the other.
if [[ -x "$SCAN" ]] && ! "$SCAN" --staged >/dev/null 2>&1; then
  say "staged secret scan failed after the tree scan passed — unstaging, nothing committed"
  git reset >/dev/null 2>&1
  exit 0
fi

# --- bulk guard -------------------------------------------------------------
# `git add -A` stages everything, and a 400MB scratch file is permanent once pushed — git keeps
# every object forever, and C-0004 denies the history rewrite that would remove it. The secret
# scan reads content and cannot see size.
#
# A large file is UNSTAGED rather than blocking the commit: the actual work still gets preserved,
# which is the whole point of ADR-0025, while the bulk is left on disk for a human to decide
# about. Refusing the entire commit would trade a bloated repository for a lost session.
#
# WHAT IS MEASURED IS THE STAGED BLOB, NOT THE FILE ON DISK (ADR-0133 decision 9). They differ
# exactly when a clean filter ran: a Git LFS asset is full-size on disk and a small pointer in the
# index, so measuring the disk left out the very files the message below recommends tracking with
# LFS. A deletion has no staged blob and is skipped. Paths are read NUL-separated because
# `--name-only` quotes a non-ASCII path, and a quoted path matched no file and slipped past.
MAX_FILE_BYTES=$((5 * 1024 * 1024))
skipped=""
while IFS= read -r -d '' f; do
  sz="$(git cat-file -s ":$f" 2>/dev/null)" || continue
  [[ -n "$sz" && "$sz" -gt "$MAX_FILE_BYTES" ]] || continue
  git reset -q -- "$f" 2>/dev/null && skipped+="  ${f} ($((sz / 1024 / 1024))MB)"$'\n'
done < <(git diff --cached --name-only -z)
if [[ -n "$skipped" ]]; then
  say "left OUT of this commit — over $((MAX_FILE_BYTES / 1024 / 1024))MB each, and permanent once pushed:"
  printf '%s' "$skipped" >&2
  say "still on disk. Add to .gitignore, or use git-lfs if it genuinely belongs in the repository."
fi

n="$(git diff --cached --name-only | wc -l | tr -d ' ')"
[[ "$n" -gt 0 ]] || exit 0

run_gate knowledge  "$R/scripts/check-knowledge.sh"
run_gate repo       "$R/scripts/check-repo.sh"
run_gate migrations "$R/scripts/check-migrations.sh"
run_gate changelog  "$R/templates/infra-ci/scripts/check-changelog.sh"
# C-0003 was enforced NOWHERE for checkpoint commits: --no-verify below skips the pre-commit
# hook that runs check-commits.sh --staged, and CI marks it continue-on-error on push. Recorded
# rather than blocking, like every other gate here.
run_gate identity   "$R/scripts/check-commits.sh" --staged

if [[ "$secret_scan_absent" -eq 1 ]]; then
  gate_lines+="  secret-scan: ABSENT — THIS COMMIT WAS NOT SCANNED"$'\n'
  absent=$((absent+1))
else
  gate_lines+="  secret-scan: pass (blocking)"$'\n'
fi

concurrent=""
if [[ -f "$CONCURRENT_FLAG" ]]; then
  concurrent="A concurrent session was observed since the last checkpoint (at $(cat "$CONCURRENT_FLAG" 2>/dev/null))."$'\n'
  if [[ "$SHARED_INDEX" == true ]]; then
    concurrent+="This is the MAIN worktree, so the index is shared and this commit may contain that
session's work. A linked worktree (git worktree add) has its own index and cannot.
See ADR-0043 for the decision and RFC-0011 for the evidence."$'\n'
  else
    concurrent+="This is a linked worktree with its own index, so the peer's files could not have
been staged here. Recorded for the record, not as a warning. See ADR-0043."$'\n'
  fi
  rm -f "$CONCURRENT_FLAG"
fi

# `green` asserts the gates passed on this content. It must not survive a gate that never ran,
# nor a tree two sessions wrote to.
verdict="green"
[[ -n "$concurrent" && "$SHARED_INDEX" == true ]] && verdict="unverified (shared tree)"
[[ $absent -gt 0 ]] && verdict="unverified (${absent} gate(s) absent)"
[[ $failed -gt 0 ]] && verdict="$failed gate(s) failing"

# C-0003: the human who initiated and accepted the work is the Author — the hook does not
# override author, so it inherits the configured identity, which is correct rather than
# convenient. The AI appears only as a trailer. Signing is inherited too: if commit.gpgsign
# is unset this commit is unsigned, exactly like every commit before it, and
# git.commits_are_signed will keep reporting that gap rather than having it hidden here.
subject="chore(checkpoint): preserve session work — ${verdict}"
[[ ${#subject} -gt 72 ]] && subject="chore(checkpoint): preserve session work"

if ! git commit --no-verify -q -m "$subject" -m "Automated durability checkpoint (ADR-0025). ${n} path(s).

Gate results at checkpoint time:
${gate_lines}
A failing gate does not block this commit. The work is preserved and the failure is
recorded, because the failure mode this hook exists to prevent is losing the work.
Find these with: git log --grep 'gates: .*FAIL'

checkpointed-by: ${session}
${concurrent}

Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>" 2>/dev/null
then
  say "commit failed; work remains staged on disk"
  exit 0
fi

# --no-verify is deliberate and is not a bypass of the gates: this hook has already run all
# of them, above, and recorded the results. Letting the pre-commit hook run them again would
# re-run the same checks and, worse, would let a failing gate abort the commit — reinstating
# exactly the refuse-to-save behaviour this design rejects.

# --- 4. push, because a commit on one disk is still on one disk --------------
branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null)"
if ! git remote | grep -q .; then
  say "committed locally, but NO REMOTE is configured — this is still one disk (ADR-0025)"
  exit 0
fi
if git push --quiet 2>/dev/null; then
  say "checkpointed and pushed (${n} path(s), ${verdict})"
else
  say "committed locally but PUSH FAILED on '${branch}' — the work is not off this machine yet."
  say "Most likely the remote has moved. Resolve by hand: git pull --rebase && git push."
  say "Never force-push to resolve this; C-0004 denies it."
fi
exit 0
