#!/usr/bin/env bash
#
# Changelog format gate (STD-0005) — Keep a Changelog 2.0.0.
#
#   ./check-changelog.sh [path]     default: CHANGELOG.md
#
# Checks that the file EXISTS and is WELL FORMED. It deliberately does not check
# that the current change added an entry:
#
#   "Do not make a changelog edit a required check on every change. That teaches
#    people to add a line to pass the check, which fills the changelog with noise."
#      — Keep a Changelog 2.0.0
#
# Whether a change is notable is the author's judgement, prompted by the PR
# template. Format is mechanical, so format is what is enforced.

set -uo pipefail

FILE="${1:-CHANGELOG.md}"

if [[ -t 1 ]]; then RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; RST=$'\033[0m'
else RED=""; GRN=""; YLW=""; RST=""; fi
ERRORS=0; WARNS=0
err() { printf '%s✘%s %s\n' "$RED" "$RST" "$1"; ERRORS=$((ERRORS+1)); }
wrn() { printf '%s!%s %s\n' "$YLW" "$RST" "$1"; WARNS=$((WARNS+1)); }

if [[ ! -f "$FILE" ]]; then
  err "$FILE is missing (STD-0005: every repository keeps a changelog)"
  exit 1
fi

# The six categories exist on purpose. A repository that invents "Performance" or
# "Dependencies" starts a taxonomy that diverges across every other repository.
CANONICAL='Added|Changed|Deprecated|Removed|Fixed|Security'

head -1 "$FILE" | grep -qE '^# Changelog' \
  || err "first line must be '# Changelog'"

grep -qiE 'keepachangelog\.com' "$FILE" \
  || wrn "no link to the Keep a Changelog spec in the preamble"

grep -qE '^## \[Unreleased\]' "$FILE" \
  || err "no '## [Unreleased]' section — releases move entries out of it (STD-0005)"

# ── version headings ─────────────────────────────────────────────────────────
versions=0
while IFS= read -r line; do
  [[ "$line" =~ ^##\ \[Unreleased\] ]] && continue
  versions=$((versions+1))
  # ## [1.2.0] - 2026-08-06   (optionally trailed by [YANKED])
  if [[ ! "$line" =~ ^##\ \[[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?\]\ -\ [0-9]{4}-[0-9]{2}-[0-9]{2}(\ \[YANKED\])?[[:space:]]*$ ]]; then
    err "malformed version heading: '$line'"
    printf '      expected: ## [1.2.0] - YYYY-MM-DD\n'
    continue
  fi
  # Reject impossible dates that still match the shape.
  d="$(sed -E 's/.*- ([0-9]{4}-[0-9]{2}-[0-9]{2}).*/\1/' <<<"$line")"
  date -j -f %Y-%m-%d "$d" +%s >/dev/null 2>&1 || date -d "$d" +%s >/dev/null 2>&1 \
    || err "not a real date: $d"
done < <(grep -E '^## ' "$FILE")

# ── category headings ────────────────────────────────────────────────────────
while IFS= read -r cat; do
  # Not ${cat#### } — a run of '#' immediately after the parameter name is parsed
  # ambiguously by bash and silently fails to strip, making every category invalid.
  name="$(sed -E 's/^###[[:space:]]*//' <<<"$cat")"
  [[ "$name" =~ ^($CANONICAL)$ ]] \
    || err "'### $name' is not one of the six canonical categories (STD-0005): Added, Changed, Deprecated, Removed, Fixed, Security"
done < <(grep -E '^### ' "$FILE")

# A canonical name used TWICE under one version is still wrong, and checking names
# alone missed it — a duplicated `### Added` was committed to this very file and the
# gate stayed green. The reader cannot know which half to scan, which is the whole
# failure a changelog format check exists to prevent.
awk '
  /^## /  { ver = $0; delete seen; next }
  /^### / { if (seen[$0]++) printf "%s|%s\n", ver, $0 }
' "$FILE" | while IFS='|' read -r ver cat; do
  err "duplicate '$cat' under '$ver' — merge them; a category may appear once per version (STD-0005)"
done

# ── entries must be curated, not dumped ──────────────────────────────────────
# A commit hash or a bare Conventional Commit prefix in an entry means a generated
# line survived to main without anyone rewriting it for the reader.
while IFS= read -r entry; do
  if [[ "$entry" =~ \([0-9a-f]{7,40}\)[[:space:]]*$ ]]; then
    wrn "entry ends with a commit hash — generated, not curated: ${entry:0:64}"
  fi
  if [[ "$entry" =~ ^-\ (feat|fix|chore|docs|refactor|test|ci|build|perf|style)(\([a-z0-9_/-]+\))?!?: ]]; then
    wrn "entry is a raw commit subject — rewrite for the reader: ${entry:0:64}"
  fi
done < <(grep -E '^- ' "$FILE")

# ── deprecation before removal ───────────────────────────────────────────────
# A Removed entry with no Deprecated section anywhere earlier means upgraders meet
# the breakage without having met the warning.
if grep -qE '^### Removed' "$FILE" && ! grep -qE '^### Deprecated' "$FILE"; then
  wrn "'Removed' appears with no 'Deprecated' in any release — deprecate in one release, remove in a later one (STD-0005)"
fi

# ── comparison links ─────────────────────────────────────────────────────────
if [[ $versions -gt 0 ]] && ! grep -qE '^\[Unreleased\]:' "$FILE"; then
  wrn "no [Unreleased] comparison link at the bottom of the file"
fi

printf '\n'
if [[ $ERRORS -gt 0 ]]; then
  printf '%s%d error(s)%s, %d warning(s) in %s\n' "$RED" "$ERRORS" "$RST" "$WARNS" "$FILE"
  exit 1
fi
printf '%s✔%s %s valid — %d released version(s), %d warning(s)\n' "$GRN" "$RST" "$FILE" "$versions" "$WARNS"
