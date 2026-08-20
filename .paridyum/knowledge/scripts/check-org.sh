#!/usr/bin/env bash
#
# Organisation gate — the ADR-0012 contributor-access checks.
#
# ADR-0012 is `locked` and carries SIX checks, every one of them declared `owner: tool-forge` and
# therefore enforced by nobody. The audit on 2026-08-08 ranked it second of the ten unenforced
# locked artifacts, behind only STD-0009, and check-registry.yaml already said so about one of
# them: ci.no_pull_request_target is "the highest-severity thing in the ci namespace — a good
# candidate for an early declared → implemented move".
#
# None of the six needed Rust. Four are one `gh api` call against the organisation and two are a
# grep over workflow files. That is the same finding the check-registry header records about
# twenty other checks: "waiting on one dependency while roughly twenty were waiting on nothing."
#
# EVERY ONE OF THEM PASSED ON THE DAY THIS WAS WRITTEN, and that is the point rather than an
# anticlimax. The organisation's security posture was correct and unverified, which means nothing
# would have noticed it changing — a setting flipped in a web UI leaves no diff, no review and no
# red build.
#
#   ./check-org.sh                inspect the organisation named below
#   ./check-org.sh --org NAME     inspect a different organisation
#   ./check-org.sh --offline      skip the API checks, run the workflow scans only
#
# NETWORK AND CREDENTIALS ARE REQUIRED for the four org.* checks, and their absence is reported
# `n/a` rather than passed. A gate that prints success when it could not look is the defect class
# the ADR-0014 audit found four times, and check-migrations.sh states the rule this follows:
# "`n/a` is not a pass."

set -uo pipefail

# ADR-0006 names macOS a supported workstation and macOS ships bash 3.2, which has neither
# `mapfile` nor `declare -A`. This script uses both.
if [[ -z "${BASH_VERSINFO:-}" ]] || (( BASH_VERSINFO[0] < 4 )); then
  printf 'PARIDYUM GATE CANNOT RUN: needs bash 4+, found %s\n' "${BASH_VERSION:-unknown}" >&2
  printf '  macOS ships bash 3.2 (ADR-0006 supports macOS). Install a modern one:\n' >&2
  printf '    brew install bash   # then re-run, or invoke with: $(brew --prefix)/bin/bash %s\n' "$0" >&2
  printf '  Reported as a failure, not skipped: a gate that cannot run has not passed.\n' >&2
  exit 1
fi

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
ORG="Paridyum"
OFFLINE=false
ORG_JSON_FILE=""
WORKFLOW_ROOT=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --org)      ORG="$2"; shift 2 ;;
    --offline)  OFFLINE=true; shift ;;
    # --org-json and --workflow-root exist for the fixture suite, the same way check-migrations.sh
    # takes --root. Five of these checks were green on the day they were written, and a green
    # check is unproven until it has been watched failing; without a fixture path the only way to
    # test the failure branches would be to misconfigure the real organisation.
    --org-json) ORG_JSON_FILE="$2"; shift 2 ;;
    --workflow-root) WORKFLOW_ROOT="$2"; shift 2 ;;
    *) shift ;;
  esac
done
[[ -n "$WORKFLOW_ROOT" ]] || WORKFLOW_ROOT="$ROOT" 

if [[ -t 1 ]]; then
  RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; RST=$'\033[0m'
else
  RED=""; GRN=""; YLW=""; RST=""
fi
ERRORS=0; WARNS=0; NA=0
err() { printf '%s✘%s %s\n' "$RED" "$RST" "$1"; ERRORS=$((ERRORS+1)); }
wrn() { printf '%s!%s %s\n'  "$YLW" "$RST" "$1"; WARNS=$((WARNS+1)); }
ok()  { printf '%s✔%s %s\n' "$GRN" "$RST" "$1"; }
na()  { printf '%sn/a%s %s\n' "$YLW" "$RST" "$1"; NA=$((NA+1)); }

printf 'Checking organisation %s (ADR-0012)\n\n' "$ORG"

# --- the four org.* checks --------------------------------------------------
if [[ -n "$ORG_JSON_FILE" ]]; then
  ORGJSON="$(cat "$ORG_JSON_FILE")"
  FIXTURE=true
elif [[ "$OFFLINE" == true ]]; then
  na "org.* checks skipped (--offline) — four ADR-0012 controls not evaluated"
elif ! command -v gh >/dev/null 2>&1; then
  na "gh is not installed — four org.* controls could not be read. Not a pass."
elif ! gh auth status >/dev/null 2>&1; then
  na "gh is not authenticated — four org.* controls could not be read. Not a pass."
elif ! ORGJSON="$(gh api "orgs/$ORG" 2>/dev/null)"; then
  na "cannot read orgs/$ORG — no permission or no network. Four controls unevaluated, not passed."
fi
if [[ -n "${ORGJSON:-}" ]]; then
  FIXTURE="${FIXTURE:-false}"
  # `.x // "<absent>"` was the first version and it was WRONG in the one direction that mattered:
  # jq's `//` yields the alternative when the left side is null **or false**, so a literal
  # `false` — a disabled control, the exact thing this gate exists to catch — was reported as
  # `<absent>` and downgraded to n/a instead of failing. Found by the fixture suite, not by
  # reading. It is the same defect shape as everything else audited on 2026-08-08: a check that
  # returned a benign verdict while measuring the wrong property.
  field() { jq -r "if has(\"$1\") then (.$1 | tostring) else \"<absent>\" end" <<<"$ORGJSON"; }

  # --- org.two_factor_required ---------------------------------------------
  # The only control here that survives a compromised contributor account rather than a
  # careless one, which is why ADR-0012 calls it the cheapest control it has.
  v="$(field two_factor_requirement_enabled)"
  case "$v" in
    true)  ok "org.two_factor_required — 2FA is required" ;;
    false) err "org.two_factor_required: two-factor is NOT required. Read access to the entire architecture rests on one password (ADR-0012)" ;;
    *)     na "org.two_factor_required: the API did not return two_factor_requirement_enabled" ;;
  esac

  # --- org.base_permission_is_none -----------------------------------------
  # GitHub's default is `read on every repository`. ADR-0012: left at the default, adding a
  # person to the org grants them the whole estate and no team configuration takes it back.
  v="$(field default_repository_permission)"
  case "$v" in
    none) ok "org.base_permission_is_none — base permission is none" ;;
    "<absent>") na "org.base_permission_is_none: the API did not return default_repository_permission" ;;
    *)    err "org.base_permission_is_none: base permission is '$v', not 'none'. Every member holds it on every repository, present and future (ADR-0012)" ;;
  esac

  # --- org.private_forking_enabled -----------------------------------------
  # The permissive setting that makes the restrictive one survivable: with forking off, a
  # read-only contributor has no path to contribute and "just give them write" becomes
  # irresistible. ADR-0012's own words.
  v="$(field members_can_fork_private_repositories)"
  case "$v" in
    true)  ok "org.private_forking_enabled — private forking is permitted" ;;
    false) err "org.private_forking_enabled: private forking is disabled, so a read-only contributor has no way to contribute — which is the pressure that ends in a write grant (ADR-0012)" ;;
    *)     na "org.private_forking_enabled: the API did not return members_can_fork_private_repositories" ;;
  esac

  # --- org.contributors_are_read_only --------------------------------------
  # args: { exempt_roles: ["admin"] }. A single promotion to write deletes this ADR's guarantee
  # with no red build and no diff — so the check is the only thing that would ever notice.
  if [[ "$FIXTURE" == true ]]; then
    na "org.contributors_are_read_only: fixture mode, member list not read"
  elif MEMBERS="$(gh api "orgs/$ORG/members" --paginate --jq '.[].login' 2>/dev/null)" \
     && ADMINS="$(gh api "orgs/$ORG/members?role=admin" --paginate --jq '.[].login' 2>/dev/null)"; then
    n_members="$(grep -c . <<<"$MEMBERS" || true)"
    n_admins="$(grep -c . <<<"$ADMINS" || true)"
    n_other=$(( n_members - n_admins ))
    if [[ "$n_other" -eq 0 ]]; then
      # Vacuous. Reported as such rather than green: a check with no input has not been
      # satisfied, it has been unexercised (check-migrations.sh states the same rule).
      na "org.contributors_are_read_only: every member is an admin, so the rule has no subject yet. Vacuous, therefore not a pass."
    else
      bad=0
      while IFS= read -r m; do
        [[ -n "$m" ]] || continue
        grep -qx "$m" <<<"$ADMINS" && continue
        role="$(gh api "orgs/$ORG/memberships/$m" --jq '.role' 2>/dev/null || echo unknown)"
        [[ "$role" == "member" ]] || { err "org.contributors_are_read_only: '$m' holds org role '$role' (ADR-0012)"; bad=1; }
      done <<<"$MEMBERS"
      [[ "$bad" -eq 0 ]] && ok "org.contributors_are_read_only — $n_other non-admin member(s), none elevated"
    fi
  else
    na "org.contributors_are_read_only: member list unreadable"
  fi
fi

# --- ci.no_pull_request_target ----------------------------------------------
# ADR-0012: "pull_request_target runs the base repo's workflow with its secrets and a
# write-scoped token. Combined with a fork PR checkout it is arbitrary code execution as the
# org." Scans workflow files only — check-registry.yaml names the trigger in prose, and a gate
# that failed on its own description of the defect would be unusable.
# tests/fixtures/ is excluded on a default scan, and the exclusion is load-bearing rather than
# tidiness — check-migrations.sh records the identical bug: "the fixture suite contains
# DELIBERATELY BROKEN migrations, so a default scan from the repository root found them and
# failed — in CI and in the pre-commit hook, on every commit." This script reproduced that
# mistake on its first fixture run, in the same repository, against a comment describing it.
# `--workflow-root` deliberately does NOT apply the exclusion, because that is how the suite
# reaches the broken fixtures.
WF_PRUNE=()
[[ "$WORKFLOW_ROOT" == "$ROOT" ]] && WF_PRUNE=(-not -path '*/tests/fixtures/*')
mapfile -t WORKFLOWS < <(find "$WORKFLOW_ROOT" -path '*/.github/workflows/*' \( -name '*.yml' -o -name '*.yaml' \) -type f -not -path '*/.git/*' "${WF_PRUNE[@]}" 2>/dev/null | sort)
if [[ ${#WORKFLOWS[@]} -eq 0 ]]; then
  na "ci.no_pull_request_target: no workflow files found — the check has no input"
else
  hits=0
  for f in "${WORKFLOWS[@]}"; do
    grep -qE '^[[:space:]]*pull_request_target[[:space:]]*:' "$f" \
      && { err "ci.no_pull_request_target: ${f#"$WORKFLOW_ROOT"/} declares pull_request_target — arbitrary code execution as the org when combined with a fork checkout (ADR-0012)"; hits=1; }
  done
  [[ $hits -eq 0 ]] && ok "ci.no_pull_request_target — ${#WORKFLOWS[@]} workflow(s), none uses pull_request_target"
fi

# --- ci.secret_jobs_guarded_on_fork (warn) ----------------------------------
# `warn`, per ADR-0012, and the rationale is about ergonomics rather than breach: a job needing
# a secret "does not fail informatively on a fork PR — it fails confusingly, and the reflex fix
# is to reach for pull_request_target." So this check exists to stop the previous one being
# violated for a sympathetic reason.
#
# The guard is recognised loosely on purpose: `github.event.pull_request.head.repo.fork`,
# `head.repo.full_name == github.repository`, or an `if:` naming the event. A precise reading
# needs a YAML-aware job walker, which is tool-forge's, and a strict grep here would produce
# false warnings on workflows that are correct.
if [[ ${#WORKFLOWS[@]} -gt 0 ]]; then
  unguarded=0
  for f in "${WORKFLOWS[@]}"; do
    grep -q 'secrets\.' "$f" || continue
    grep -qE '^[[:space:]]*(on:|.*)pull_request[[:space:]]*:' "$f" || continue
    grep -qE 'head\.repo\.fork|head\.repo\.full_name|github\.event_name[[:space:]]*[!=]=' "$f" && continue
    wrn "ci.secret_jobs_guarded_on_fork: ${f#"$WORKFLOW_ROOT"/} uses secrets on a pull_request workflow with no visible fork guard — on a fork PR it fails opaquely, and the reflex fix is pull_request_target (ADR-0012)"
    unguarded=1
  done
  [[ $unguarded -eq 0 ]] && ok "ci.secret_jobs_guarded_on_fork — no unguarded secret usage on pull_request workflows"
fi

printf '\n'
if [[ $ERRORS -gt 0 ]]; then
  printf '%s%d error(s)%s, %d warning(s), %d n/a\n' "$RED" "$ERRORS" "$RST" "$WARNS" "$NA"; exit 1
fi
printf '%sOrganisation check passed%s (%d warning(s), %d n/a)\n' "$GRN" "$RST" "$WARNS" "$NA"
