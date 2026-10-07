#!/usr/bin/env bash
#
# Decides whether a pull request's changes can reach the check suite (scripts/test-checks.sh).
# ADR-0124, 2026-09-29 addendum.
#
#   git diff --name-only BASE HEAD | scripts/check-suite-scope.sh
#
# Prints exactly one line on stdout, `run=true` or `run=false`, in the form $GITHUB_OUTPUT takes.
# Explains the answer on stderr. ANY doubt is `run=true`: an empty or unreadable input, a missing
# suite, an internal error. Skipping is the only outcome that has to be earned.
#
# WHAT CAN REACH THE SUITE, found by reading it rather than assumed:
#   * the checkers and their inputs — scripts/, templates/, plugins/, tests/, .github/;
#   * the registries and schemas at the root (*.yaml, *.json), which the checkers read as rules;
#   * TESTING.md, which the suite plants into and which records its own tier;
#   * ANY FILE THE SUITE NAMES under a directory. The knowledge cases copy the real knowledge base and plant each
#     defect into a real artifact on real anchor text (`kplant … bets/BET-0005-… "status: accepted"`).
#     Editing that artifact's anchor breaks the case, and the edit looks like documentation. So a
#     changed path that appears anywhere in test-checks.sh reaches the suite.
#
# WHAT DOES NOT: every other artifact. The suite's remaining dependence on content is that the real
# knowledge base checks clean before a defect is planted, and the "Knowledge integrity" step proves
# exactly that on every pull request. Residual risk is caught on every push to main and nightly.
set -uo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
SUITE="$ROOT/scripts/test-checks.sh"

answer() { printf 'run=%s\n' "$1"; exit 0; }
trap 'printf "check-suite-scope: internal error, so the suite runs\n" >&2; answer true' ERR

[[ -f "$SUITE" ]] || { echo "check-suite-scope: $SUITE is missing, so the suite runs (and will fail)" >&2; answer true; }

changed=()
while IFS= read -r line || [[ -n "$line" ]]; do
  [[ -n "${line//[[:space:]]/}" ]] && changed+=("$line")
done
if (( ${#changed[@]} == 0 )); then
  echo "check-suite-scope: no changed paths were given, so the suite runs" >&2
  answer true
fi

reasons=()
for f in "${changed[@]}"; do
  case "$f" in
    scripts/*|templates/*|plugins/*|tests/*|.github/*) reasons+=("$f — checker code or its fixtures") ;;
    TESTING.md)                                         reasons+=("$f — the suite plants into it") ;;
    */*) ;;
    *.yaml|*.yml|*.json)                                reasons+=("$f — a registry or schema the checkers read") ;;
  esac
  # Only a path with a directory counts as "named". The root files the suite mentions —
  # CHANGELOG.md, DECISION-DEBT.md, README.md, CLAUDE.md — are SYNTHETIC files in its temporary
  # scaffolds (`: > "$d/CHANGELOG.md"`), not the real ones; matching them would run the suite on
  # nearly every pull request. The root files it does plant into are the registries, the schemas
  # and TESTING.md, which the rules above already catch.
  if [[ "$f" == */* ]] && grep -qF -- "$f" "$SUITE"; then
    reasons+=("$f — named in test-checks.sh, which plants a defect into it")
  fi
done

if (( ${#reasons[@]} > 0 )); then
  echo "check-suite-scope: the suite runs, because this change reaches it:" >&2
  printf '  · %s\n' "${reasons[@]}" | sort -u >&2
  answer true
fi

echo "check-suite-scope: ${#changed[@]} changed path(s), none a checker, registry, schema or file the suite plants into." >&2
echo "  The suite is skipped on this pull request; it runs on the push to main and nightly (ADR-0124)." >&2
answer false
