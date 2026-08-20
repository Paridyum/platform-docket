#!/usr/bin/env bash
#
# A fix carries a regression test (STD-0008, test.bugfix_carries_a_regression_test).
#
# Destination: Paridyum/infra-ci/scripts/check-regression.sh
#
#   ./check-regression.sh                 commits not on origin/main
#   ./check-regression.sh <rev-range>     an explicit range
#   ./check-regression.sh --root <dir> …  run against another checkout (used by the suite)
#
# WHY THIS IS A GATE AND NOT A REVIEW COMMENT
#
# The only moment anybody knows how to reproduce a defect is the moment it is understood
# well enough to fix, and that knowledge decays within days. A regression suite built this
# way is the only part of a test suite where every entry is evidence that a specific failure
# was possible, rather than evidence that somebody imagined one. Asking for it in review
# means asking at the moment the author has already moved on.
#
# WHAT IT DELIBERATELY DOES NOT DO
#
# It cannot tell a real regression test from a touched test file, and it does not try. A
# check that claimed to would be verifying less than its name implies, which is this
# repository's most-repeated defect. It asserts the cheap, mechanical half — a `fix:` commit
# whose diff contains no test change — and STD-0008's prose owns the rest.
#
# THE EXCEPTION IS REAL AND NARROW. A rule with no legal exception is one people learn to
# phrase around: the next `fix:` becomes `chore:` and the gate has taught obfuscation. So a
# reproduction that genuinely cannot be automated says so, in the commit body, in a trailer
# that is greppable afterwards:
#
#   Regression-test-exception: reproduces only under a specific scheduler interleaving

set -uo pipefail

# ADR-0006 names macOS a supported workstation and macOS ships bash 3.2, which has neither
# `mapfile` nor `declare -A`. This script uses both. Without this guard it dies with
# `unbound variable` and no usable diagnosis -- which reads as a broken repository rather
# than a missing dependency, and is exactly the "green check that proves nothing" failure
# inverted: a red one that explains nothing. Found 2026-08-08 by running the gate under
# /bin/bash rather than the Homebrew bash that happened to be on PATH.
if [[ -z "${BASH_VERSINFO:-}" ]] || (( BASH_VERSINFO[0] < 4 )); then
  printf 'PARIDYUM GATE CANNOT RUN: needs bash 4+, found %s\n' "${BASH_VERSION:-unknown}" >&2
  printf '  macOS ships bash 3.2 (ADR-0006 supports macOS). Install a modern one:\n' >&2
  printf '    brew install bash   # then re-run, or invoke with: $(brew --prefix)/bin/bash %s\n' "$0" >&2
  printf '  Reported as a failure, not skipped: a gate that cannot run has not passed.\n' >&2
  exit 1
fi


ROOT="."
while [[ $# -gt 0 ]]; do
  case "$1" in
    --root) ROOT="$2"; shift 2 ;;
    *) break ;;
  esac
done
cd "$ROOT" || { printf 'cannot enter %s\n' "$ROOT" >&2; exit 2; }

if [[ -t 1 ]]; then RED=$'\033[31m'; GRN=$'\033[32m'; RST=$'\033[0m'
else RED=""; GRN=""; RST=""; fi
ERRORS=0
err() { printf '%s✘%s %s\n' "$RED" "$RST" "$1"; ERRORS=$((ERRORS+1)); }

# Conventional Commit `fix`, with an optional scope and an optional breaking marker.
FIX_RE='^fix(\([a-z0-9._/-]+\))?!?: '

# A path that is a test, across the stacks class-registry.yaml declares plus the two
# language-independent conventions. Deliberately generous: a false negative here lets a real
# fix through, and a false positive blocks a correct change — the second is much more
# expensive, because it is the one that gets the check switched off.
is_test_path() {
  case "$1" in
    tests/*|*/tests/*|test/*|*/test/*|__tests__/*|*/__tests__/*) return 0 ;;
    *_test.*|*.test.*|*_spec.*|*.spec.*|*_tests.rs) return 0 ;;
    # `test-checks.sh` and friends. A repository whose subject is shell gates keeps its
    # suite here rather than under tests/, and without this the rule would be unsatisfiable
    # in exactly the repository that wrote it.
    test-*.*|*/test-*.*) return 0 ;;
    benches/*|*/benches/*) return 0 ;;
    */testdata/*|testdata/*|*/fixtures/*|fixtures/*) return 0 ;;
  esac
  return 1
}

range="${1:-}"
if [[ -z "$range" ]]; then
  if git rev-parse --verify -q origin/main >/dev/null 2>&1; then range="origin/main..HEAD"
  else range="HEAD"; fi
fi

mapfile -t shas < <(git rev-list "$range" 2>/dev/null)
if [[ ${#shas[@]} -eq 0 ]]; then
  # Same rule check-commits.sh learned the hard way (ADR-0016): an EXPLICIT range resolving
  # to nothing is a broken caller, not a clean pass. That defect ran green in CI on every
  # push for the life of the workflow, examining zero commits and reporting success.
  if [[ -n "${1:-}" ]]; then
    err "range '$range' contains no commits, but was requested explicitly — a gate that examines nothing must not report success"
    exit 1
  fi
  printf 'No commits in range %s\n' "$range"; exit 0
fi

fixes=0
for sha in "${shas[@]}"; do
  subject="$(git show -s --format='%s' "$sha")"
  [[ "$subject" =~ $FIX_RE ]] || continue
  fixes=$((fixes+1))
  short="${sha:0:8}"

  if git show -s --format='%B' "$sha" | grep -qiE '^Regression-test-exception: .{8,}'; then
    printf '  %s exception: %s\n' "$short" \
      "$(git show -s --format='%B' "$sha" | grep -iE '^Regression-test-exception:' | head -1)"
    continue
  fi

  touched_test=false
  touched_src=false
  while IFS= read -r p; do
    [[ -n "$p" ]] || continue
    if is_test_path "$p"; then touched_test=true; else touched_src=true; fi
  done < <(git show --name-only --format='' "$sha")

  # A Rust unit test lives beside the code it tests, so the path heuristic cannot see it.
  # Look for a test attribute among the ADDED lines instead — `git show` prefixes those with
  # a single `+`, and the `+++` header is excluded by requiring a non-`+` next character.
  if [[ "$touched_test" == false ]] \
     && git show --format='' -U0 "$sha" | grep -qE '^\+[^+].*#\[(tokio::|async_std::|test_case\(|rstest|cfg\(test\)|test\])'; then
    touched_test=true
  fi

  if [[ "$touched_src" == true && "$touched_test" == false ]]; then
    err "$short: '${subject:0:60}' changes source and no test — a fix with no regression test is a defect that can recur silently (STD-0008). Add one, or record 'Regression-test-exception: <reason>' in the commit body."
  fi
done

printf '\n'
if [[ $fixes -eq 0 ]]; then
  printf 'No fix commits in %s\n' "$range"; exit 0
fi
if [[ $ERRORS -gt 0 ]]; then
  printf '%s%d of %d fix commit(s) carry no regression test%s\n' "$RED" "$ERRORS" "$fixes" "$RST"
  exit 1
fi
printf '%sAll %d fix commit(s) carry a regression test%s\n' "$GRN" "$fixes" "$RST"
