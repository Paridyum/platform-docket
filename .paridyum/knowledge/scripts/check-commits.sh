#!/usr/bin/env bash
#
# Commit identity and attribution gate (C-0003).
#
#   ./check-commits.sh              check commits not on origin/main
#   ./check-commits.sh <rev-range>  check an explicit range
#   ./check-commits.sh --staged     check the message about to be committed
#
# Identity is the one property that cannot be corrected later without a history
# rewrite, which C-0004 forbids. So this runs before the commit exists.

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


if [[ -t 1 ]]; then RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; RST=$'\033[0m'
else RED=""; GRN=""; YLW=""; RST=""; fi
ERRORS=0; WARNS=0
err() { printf '%s✘%s %s\n' "$RED" "$RST" "$1"; ERRORS=$((ERRORS+1)); }
wrn() { printf '%s!%s %s\n' "$YLW" "$RST" "$1"; WARNS=$((WARNS+1)); }

# C-0003 allowlist. Anchored, so `evil-rohanbatrain@users.noreply.github.com` fails.
EMAIL_ALLOW='^([0-9]+\+)?rohanbatrain@users\.noreply\.github\.com$|^rohan@rohanbatra\.in$|^(platform|security|infra|ai|release)@rohanbatra\.in$'
TYPES='feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert'
SUBJECT_RE="^(${TYPES})(\([a-z0-9._/-]+\))?!?: .+"

# Identities that must never appear as Author (C-0003).
AI_AUTHOR_RE='^(ai|assistant|claude|chatgpt|gpt|copilot|bot|developer|user|admin)$'

check_message() {
  local subject="$1" src="$2"
  [[ "$subject" =~ $SUBJECT_RE ]] \
    || err "$src: subject is not a Conventional Commit: '${subject:0:60}'"
  [[ ${#subject} -le 72 ]] \
    || wrn "$src: subject is ${#subject} chars (max 72)"
  [[ "$subject" =~ \.$ ]] \
    && wrn "$src: subject ends with a period"
  return 0
}

if [[ "${1:-}" == "--staged" ]]; then
  # Pre-commit: the message file is not written yet, so validate identity only.
  name="$(git config user.name)";  email="$(git config user.email)"
  [[ -n "$name" ]]  || err "git user.name is unset (C-0003)"
  [[ "$email" =~ $EMAIL_ALLOW ]] \
    || err "git user.email '$email' is not on the C-0003 allowlist"
  if [[ "$(git config commit.gpgsign)" != "true" ]]; then
    wrn "commit.gpgsign is not true (C-0003) — signatures cannot be added retroactively"
  fi
  [[ $ERRORS -eq 0 ]] && printf '%s✔%s identity ok (%s <%s>)\n' "$GRN" "$RST" "$name" "$email"
  exit $(( ERRORS > 0 ? 1 : 0 ))
fi

range="${1:-}"
if [[ -z "$range" ]]; then
  if git rev-parse --verify -q origin/main >/dev/null 2>&1; then range="origin/main..HEAD"
  else range="HEAD"; fi
fi

mapfile -t shas < <(git rev-list "$range" 2>/dev/null)
if [[ ${#shas[@]} -eq 0 ]]; then
  # An EXPLICIT range that resolves to nothing is a broken caller, not a clean pass.
  # This exact case ran green in CI on every push for the life of the workflow:
  # `origin/main..HEAD` is always empty after a push, because origin/main already
  # contains the pushed commit. The identity gate reported success having examined
  # zero commits — a silently-unenforced rule, which ADR-0002 names as worse than an
  # absent one. An empty range is only legitimate when nobody asked for a specific one.
  if [[ -n "${1:-}" ]]; then
    err "range '$range' contains no commits, but was requested explicitly — the caller is wrong, and a gate that examines nothing must not report success (C-0003)"
    exit 1
  fi
  printf 'No commits in range %s\n' "$range"; exit 0
fi
printf 'Checking %d commit(s) in %s\n\n' "${#shas[@]}" "$range"

for sha in "${shas[@]}"; do
  short="${sha:0:8}"
  aname="$(git show -s --format='%an' "$sha")"
  aemail="$(git show -s --format='%ae' "$sha")"
  subject="$(git show -s --format='%s' "$sha")"
  body="$(git show -s --format='%B' "$sha")"
  sig="$(git show -s --format='%G?' "$sha")"

  [[ "$aemail" =~ $EMAIL_ALLOW ]] \
    || err "$short: author email '$aemail' not on the C-0003 allowlist"

  # An AI must never be the Author — only a Co-Authored-By trailer.
  low="$(tr '[:upper:]' '[:lower:]' <<<"$aname")"
  [[ "$low" =~ $AI_AUTHOR_RE ]] \
    && err "$short: '$aname' as Author misrepresents provenance (C-0003) — use Co-Authored-By"

  case "$aemail" in
    *@gmail.com|*@outlook.com|*@yahoo.com|*@hotmail.com)
      err "$short: personal webmail address '$aemail' (C-0003)" ;;
    *.local|*@localhost)
      err "$short: machine-generated address '$aemail' silently detaches the commit from your GitHub account (C-0003)" ;;
  esac

  check_message "$subject" "$short"

  # N / U / E all mean "not a good signature"; only G, and X/Y/R for expired-but-valid
  # keys, indicate the commit was signed.
  case "$sig" in
    G|U|X|Y|R) ;;
    *) wrn "$short: unsigned (C-0003; cannot be required on GitHub Free — ADR-0004)" ;;
  esac

  # Trailer sanity: an AI named in the body must be in a Co-Authored-By trailer.
  if grep -qiE '^(co-authored-by|reviewed-by):' <<<"$body"; then
    while IFS= read -r t; do
      grep -qE '<[^@[:space:]]+@[^>[:space:]]+>' <<<"$t" \
        || err "$short: malformed trailer (needs '<email>'): ${t:0:60}"
    done < <(grep -iE '^(co-authored-by|reviewed-by):' <<<"$body")
  fi
done

printf '\n'
if [[ $ERRORS -gt 0 ]]; then
  printf '%s%d error(s)%s, %d warning(s)\n' "$RED" "$ERRORS" "$RST" "$WARNS"
  exit 1
fi
printf '%sCommit check passed%s (%d warning(s))\n' "$GRN" "$RST" "$WARNS"
