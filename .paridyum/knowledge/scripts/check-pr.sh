#!/usr/bin/env bash
#
# Pull request gate — STD-0007's review budget.
#
# ADR-0013 stage 4 has required "small PRs, each independently reviewable" since it was
# locked, and nothing has ever measured it. An adjective in a locked artifact is a rule that
# cannot be violated because it cannot be evaluated — the same shape as a locked artifact with
# an empty enforced_by, which is exactly what ADR-0002 exists to prevent.
#
# Measured on the BRANCH DIFF against the merge base, never per commit: ADR-0025 commits
# automatically on every response, so commit count carries no information about review effort.
#
#   ./check-pr.sh                      compare HEAD against its merge base with the default branch
#   ./check-pr.sh --base REF           compare against REF instead
#   ./check-pr.sh --body FILE          read the PR body from FILE (for the exception marker)
#   ./check-pr.sh --root DIR           operate on DIR (used by the fixture suite)
#   ./check-pr.sh --expect-fail        exit 0 only if at least one error was found

set -uo pipefail

SELF_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
R="$SELF_ROOT"; BASE=""; BODY=""; EXPECT_FAIL=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --base) BASE="$2"; shift 2 ;;
    --body) BODY="$2"; shift 2 ;;
    --root) R="$(cd "$2" && pwd)"; shift 2 ;;
    --expect-fail) EXPECT_FAIL=true; shift ;;
    *) shift ;;
  esac
done
cd "$R" || exit 0

if [[ -t 1 ]]; then RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; RST=$'\033[0m'
else RED=""; GRN=""; YLW=""; RST=""; fi
ERRORS=0; WARNS=0
err() { printf '%s✘%s %s\n' "$RED" "$RST" "$1"; ERRORS=$((ERRORS+1)); }
wrn() { printf '%s!%s %s\n'  "$YLW" "$RST" "$1"; WARNS=$((WARNS+1)); }

WARN_AT=250
ERROR_AT=400

git rev-parse --git-dir >/dev/null 2>&1 || { printf 'n/a not a git checkout\n'; exit 0; }

# In CI the base is supplied. Locally, the merge base with the default branch is the honest
# comparison — HEAD~1 would measure the last commit, which under ADR-0025 is meaningless.
if [[ -z "$BASE" ]]; then
  for cand in "${GITHUB_BASE_REF:-}" origin/main main; do
    [[ -n "$cand" ]] && git rev-parse --verify --quiet "$cand" >/dev/null 2>&1 && { BASE="$cand"; break; }
  done
fi
[[ -n "$BASE" ]] || { printf 'n/a no base ref to compare against\n'; exit 0; }

MB="$(git merge-base HEAD "$BASE" 2>/dev/null)" || MB="$BASE"
if [[ "$(git rev-parse HEAD)" == "$(git rev-parse "$MB")" ]]; then
  printf '%sn/a%s HEAD is at the merge base with %s — no branch diff to review, so\n' "$YLW" "$RST" "$BASE"
  printf '    pr.diff_within_review_budget has no input. This is deliberately not a pass.\n'
  printf '    It is also the normal state of this repository, which commits straight to main.\n'
  exit 0
fi

# --- pr.diff_within_review_budget -------------------------------------------
# Excluded because a reviewer does not read them. NOT excluded: tests — excluding them would
# make "large feature, no tests" the cheapest way to pass, inverting STD-0001.
is_excluded() {
  case "$1" in
    # Cargo.lock, yarn.lock, uv.lock. NOT pnpm-lock.yaml or package-lock.json, which end in .yaml and
    # .json: this line once claimed to cover them and did not (2026-10-03), so they are named.
    *.lock) return 0 ;;
    pnpm-lock.yaml|*/pnpm-lock.yaml|package-lock.json|*/package-lock.json) return 0 ;;
    */.paridyum/knowledge/*|.paridyum/knowledge/*) return 0 ;;
    */tests/fixtures/*|tests/fixtures/*|*/testdata/*|*.golden|*.snap) return 0 ;;
    *.generated.*|*_generated.*|*/generated/*) return 0 ;;
    */target/*|*/node_modules/*|*/dist/*|*/.next/*) return 0 ;;
  esac
  return 1
}

counted=0; excluded=0; files=0
while IFS=$'\t' read -r add del path; do
  [[ -n "${path:-}" ]] || continue
  # A binary file reports "-" for both counts; it is unreadable rather than large.
  [[ "$add" == "-" || "$del" == "-" ]] && { excluded=$((excluded+1)); continue; }
  if is_excluded "$path"; then
    excluded=$((excluded+1)); continue
  fi
  files=$((files+1)); counted=$((counted+add+del))
done < <(git diff --numstat "$MB"...HEAD 2>/dev/null)

# A pure rename with no content change reports 0/0 and is already free. Detected as a
# convenience for the message rather than as a separate rule.
renames="$(git diff --diff-filter=R --name-only "$MB"...HEAD 2>/dev/null | wc -l | tr -d ' ')"

printf 'Branch diff vs %s: %d reviewable line(s) across %d file(s)' "$BASE" "$counted" "$files"
[[ "$excluded" -gt 0 ]] && printf ', %d file(s) excluded' "$excluded"
[[ "$renames" -gt 0 ]] && printf ', %d rename(s)' "$renames"
printf '\n\n'

# --- the exception, which must be visible to the reviewer, not in a config file ---
EXCEPTED=false; REASON=""
if [[ -n "$BODY" && -f "$BODY" ]]; then
  if grep -qiE '^Review-budget-exception:[[:space:]]*\S' "$BODY"; then
    EXCEPTED=true
    REASON="$(grep -iE '^Review-budget-exception:' "$BODY" | head -1 | sed 's/^[^:]*:[[:space:]]*//')"
  fi
fi

if [[ "$counted" -gt "$ERROR_AT" ]]; then
  if [[ "$EXCEPTED" == true ]]; then
    wrn "$counted reviewable lines exceeds the $ERROR_AT-line budget, permitted by an exception: \"$REASON\". A pattern of these is visible in the log, which is the point of recording it in the body (STD-0007)"
  else
    err "$counted reviewable lines exceeds the $ERROR_AT-line budget (STD-0007). ADR-0013 stage 4 requires each PR to be independently reviewable, and past this size review effectiveness collapses. Split it into slices citing REQ-NNNN §N, or add 'Review-budget-exception: <reason>' to the body — \"it was easier together\" is not a reason."
  fi
elif [[ "$counted" -gt "$WARN_AT" ]]; then
  wrn "$counted reviewable lines is over the $WARN_AT-line comfort threshold but under the $ERROR_AT-line limit — say in the PR body why it could not be split (STD-0007)"
fi

# --- pr.slice_cites_requirement ---------------------------------------------
# `warn`, because a slice legitimately may predate any requirement — a spike, a typo fix, or
# governance work like this repository's own. An error here would fire constantly on work that
# is fine, and a check that fires constantly gets disabled (ADR-0008).
if [[ -n "$BODY" && -f "$BODY" ]]; then
  grep -qE '\b(REQ|RFC|ADR)-[0-9]{4}\b' "$BODY" \
    || wrn "the PR body cites no REQ-, RFC- or ADR- id — the reason for this work exists only outside the repository (STD-0007, C-0001 principle 3)"
fi

printf '\n'
if [[ "$EXPECT_FAIL" == true ]]; then
  [[ $ERRORS -gt 0 ]] && { printf '%s✔%s expected failure produced %d error(s)\n' "$GRN" "$RST" "$ERRORS"; exit 0; }
  printf '%s✘%s expected at least one error and got none — the planted defect was NOT caught\n' "$RED" "$RST"; exit 1
fi
if [[ $ERRORS -gt 0 ]]; then
  printf '%s%d error(s)%s, %d warning(s)\n' "$RED" "$ERRORS" "$RST" "$WARNS"; exit 1
fi
printf '%sPull request check passed%s (%d warning(s))\n' "$GRN" "$RST" "$WARNS"
