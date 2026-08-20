#!/usr/bin/env bash
#
# Change-lifecycle gate (ADR-0013).
#
#   ./check-significance.sh <base-ref> [pr-body-file]
#
# Computes whether a diff is SIGNIFICANT from the diff itself, never from a
# self-declaration. "Is this significant?" asked of the person who wants to start
# coding is not a neutral question, so it is not asked.
#
# A significant change must reference an RFC or ADR in the PR body. Everything else
# proceeds straight to implementation — most work is not significant, and treating
# it as though it were is how a process gets abandoned.

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


BASE="${1:-origin/main}"
BODY_FILE="${2:-}"

if [[ -t 1 ]]; then RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; DIM=$'\033[2m'; RST=$'\033[0m'
else RED=""; GRN=""; YLW=""; DIM=""; RST=""; fi

git rev-parse --verify -q "$BASE" >/dev/null 2>&1 || { echo "unknown base ref: $BASE" >&2; exit 2; }

mapfile -t changed < <(git diff --name-only "$BASE"...HEAD 2>/dev/null)
[[ ${#changed[@]} -eq 0 ]] && { echo "No changes against $BASE."; exit 0; }

reasons=()

for f in "${changed[@]}"; do
  case "$f" in
    specs/*openapi*|specs/*.proto|**/openapi.yaml)
      reasons+=("interface contract changed: $f") ;;
    migrations/*)
      reasons+=("database migration added or altered: $f") ;;
    helm/*|terraform/*)
      reasons+=("deployment configuration changed: $f") ;;
    .github/workflows/*)
      reasons+=("CI/CD pipeline changed: $f") ;;
    manifest.yaml)
      # Only a dependency change is significant; lifecycle/owner edits are not.
      if git diff "$BASE"...HEAD -- "$f" | grep -qE '^\+.*(services:|infra:|events:)'; then
        reasons+=("service dependency added in manifest.yaml")
      fi ;;
  esac
done

# A new crate: a Cargo.toml appearing where there was none.
while IFS= read -r f; do
  reasons+=("new crate introduced: $f")
done < <(git diff --name-only --diff-filter=A "$BASE"...HEAD -- '**/Cargo.toml' 'Cargo.toml' 2>/dev/null)

# A new top-level module under src/ — src/foo.rs or src/foo/mod.rs, not src/foo/bar.rs.
while IFS= read -r f; do
  case "$f" in
    src/*/*/*) continue ;;
    src/*/mod.rs|src/*.rs)
      [[ "$f" == "src/lib.rs" || "$f" == "src/main.rs" ]] && continue
      reasons+=("new top-level module: $f") ;;
  esac
done < <(git diff --name-only --diff-filter=A "$BASE"...HEAD -- 'src/*' 2>/dev/null)

# Dependency added outside the ADR-0005 locked stack.
LOCKED_STACK='tokio|axum|tower|serde|serde_json|sqlx|tracing|tracing-subscriber|opentelemetry|thiserror|anyhow|uuid|chrono'
while IFS= read -r line; do
  dep="$(sed -E 's/^\+[[:space:]]*([A-Za-z0-9_-]+).*/\1/' <<<"$line")"
  [[ -z "$dep" || "$dep" =~ ^(version|features|workspace|default-features|path|git|tag|optional)$ ]] && continue
  [[ "$dep" =~ ^($LOCKED_STACK)$ ]] && continue
  reasons+=("dependency added outside the locked stack: $dep")
done < <(git diff "$BASE"...HEAD -- '**/Cargo.toml' 'Cargo.toml' 2>/dev/null \
         | grep -E '^\+[a-zA-Z0-9_-]+[[:space:]]*=' || true)

# Public API surface of a library changed.
if git diff --name-only "$BASE"...HEAD | grep -q '^src/'; then
  added_pub="$(git diff "$BASE"...HEAD -- 'src/**/*.rs' 'src/*.rs' 2>/dev/null \
               | grep -cE '^[+-][[:space:]]*pub (fn|struct|enum|trait|type|const|mod) ' || true)"
  if [[ "${added_pub:-0}" -gt 0 ]] && [[ -f Cargo.toml ]] && grep -qE '^\[lib\]|^name[[:space:]]*=' Cargo.toml; then
    case "$(basename "$PWD")" in
      lib-*) reasons+=("public API changed (${added_pub} pub item lines)") ;;
    esac
  fi
fi

# ── report ───────────────────────────────────────────────────────────────────
if [[ ${#reasons[@]} -eq 0 ]]; then
  printf '%s✔%s Not a significant change — no design artifact required (ADR-0013).\n' "$GRN" "$RST"
  exit 0
fi

printf '%sSignificant change%s (ADR-0013). Triggered by:\n\n' "$YLW" "$RST"
printf '  - %s\n' "${reasons[@]}" | sort -u
printf '\n'

# Look for an RFC/ADR reference in the PR body, else in the commit messages.
haystack=""
[[ -n "$BODY_FILE" && -f "$BODY_FILE" ]] && haystack="$(cat "$BODY_FILE")"
haystack+=$'\n'"$(git log --format=%B "$BASE"..HEAD 2>/dev/null)"

if grep -qoE '\b(RFC|ADR)-[0-9]{4}\b' <<<"$haystack"; then
  refs="$(grep -oE '\b(RFC|ADR)-[0-9]{4}\b' <<<"$haystack" | sort -u | tr '\n' ' ')"
  printf '%s✔%s Design referenced: %s\n' "$GRN" "$RST" "$refs"

  # Ordering check (warn): the design should predate the implementation. An RFC
  # committed alongside the code it justifies is a rationalisation.
  for ref in $(grep -oE '\b(RFC|ADR)-[0-9]{4}\b' <<<"$haystack" | sort -u); do
    file="$(git ls-files "**/${ref}-*.md" 2>/dev/null | head -1)"
    [[ -z "$file" ]] && continue
    added_in="$(git log --diff-filter=A --format=%H -1 -- "$file" 2>/dev/null)"
    [[ -z "$added_in" ]] && continue
    if git merge-base --is-ancestor "$added_in" "$BASE" 2>/dev/null; then
      printf '  %s%s predates this branch%s\n' "$DIM" "$ref" "$RST"
    else
      printf '  %s!%s %s was added on this branch — design and implementation in one PR.\n' "$YLW" "$RST" "$ref"
      printf '    %sIf the design was genuinely done first, land it separately next time.%s\n' "$DIM" "$RST"
    fi
  done
  exit 0
fi

printf '%s✘%s No RFC or ADR referenced in the PR body or commit messages.\n\n' "$RED" "$RST"
cat <<'EOF'
A significant change is designed and reviewed before it is implemented (ADR-0013):

  1. Write an RFC in rfc/ — problem, design, and rejected alternatives.
  2. Get it accepted (status: accepted).
  3. Reference it here: "Implements RFC-00NN".

If this is genuinely not significant, the trigger list above is wrong. Amend
ADR-0013's criteria in a separate PR rather than working around this check.
EOF
exit 1
