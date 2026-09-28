#!/usr/bin/env bash
#
# Repository hygiene gate — the `repo.*` and `docs.*` checks that need no Rust toolchain,
# no network and no repository of a class that does not yet exist.
#
# WHY THIS IS NOT tool-forge. Every check here was declared with `owner: tool-forge`, which
# put it behind a binary nobody has written. But none of them need Rust: they test for the
# presence and shape of files in a checkout. Leaving them declared meant 79 checks waiting on
# one dependency when roughly a dozen were waiting on nothing at all. ADR-0002's whole
# argument is that a lock is an executable check; a check that could run today and does not is
# the same gap in a more comfortable form.
#
# The checks that genuinely DO need tool-forge — cargo, clippy, rustfmt, helm, and the
# per-crate module rules — need a Rust repository to run against, and none exists. Those stay
# declared, honestly, rather than being implemented against nothing and reported as passing.
#
#   ./check-repo.sh              check this repository
#   ./check-repo.sh --root DIR   check DIR instead (used by the fixture suite)
#   ./check-repo.sh --expect-fail  exit 0 only if at least one error was found

set -uo pipefail

SELF_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
R="$SELF_ROOT"
EXPECT_FAIL=false
ROOT_GIVEN=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --root) R="$(cd "$2" && pwd)"; ROOT_GIVEN=true; shift 2 ;;
    --expect-fail) EXPECT_FAIL=true; shift ;;
    *) shift ;;
  esac
done

if [[ -t 1 ]]; then
  RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; RST=$'\033[0m'
else
  RED=""; GRN=""; YLW=""; RST=""
fi

VENDORED_SELF="$SELF_ROOT"

# ── A VENDORED COPY MUST NOT INSPECT ITSELF ───────────────────────────────────────────────────
#
# `R` defaults to this script's own parent. That is right when the script is run from the
# `knowledge` repository, and WRONG in every other repository — where the script lives at
# `.paridyum/knowledge/scripts/`, a read-only vendored snapshot (ADR-0001). Run without `--root`
# there, it inspects the snapshot instead of the repository and reports a cheerful `n/a`.
#
# ⛔ THAT COST FOUR REPOSITORIES IN TWO DAYS. `platform-tenancy` was carrying 11 real errors,
# `lib-auth-typescript` 8, `infra-kubernetes` 3 — every one of them behind a run that said it had
# passed. `n/a` is indistinguishable from a pass to everybody who has ever read one, which is the
# defect ADR-0014's audit found four separate times and which this script's own header names.
#
# So this refuses rather than reporting. The message names the command that works, because the
# alternative is a reader who now knows something is wrong and not what to type.
if [[ "$VENDORED_SELF" == */.paridyum/knowledge && "$ROOT_GIVEN" != true ]]; then
  printf '%s✘%s refusing to inspect a vendored snapshot of itself.\n' "${RED:-}" "${RST:-}" >&2
  printf '\n' >&2
  printf '  This script is at %s, which is\n' "$VENDORED_SELF/scripts" >&2
  printf '  a read-only copy of the knowledge repository (ADR-0001) — not the repository you meant.\n' >&2
  printf '\n' >&2
  printf '  Without --root it would inspect that copy and report `n/a` for almost everything, which\n' >&2
  printf '  reads exactly like a pass. Four repositories in this estate carried real errors behind\n' >&2
  printf '  such a run. Pass the repository instead:\n' >&2
  printf '\n' >&2
  printf '      %s --root "$PWD"\n' "$0" >&2
  printf '\n' >&2
  exit 2
fi

ERRORS=0; WARNS=0; NA=0
err() { printf '%s✘%s %s\n' "$RED" "$RST" "$1"; ERRORS=$((ERRORS+1)); }
wrn() { printf '%s!%s %s\n'  "$YLW" "$RST" "$1"; WARNS=$((WARNS+1)); }
na()  { printf '%sn/a%s %s\n' "$YLW" "$RST" "$1"; NA=$((NA+1)); }

# The class of the repository under inspection decides which checks apply. A repository with
# no manifest.yaml is class-less, which for `knowledge` is correct and deliberate (ADR-0015
# point 6) rather than a missing file.
CLASS=""
[[ -f "$R/manifest.yaml" ]] && CLASS="$(awk -F': *' '/^class:/{sub(/ *#.*/,"",$2);print $2;exit}' "$R/manifest.yaml")"
IS_KNOWLEDGE=false
[[ -f "$R/knowledge.schema.json" && -d "$R/constitution" ]] && IS_KNOWLEDGE=true

printf 'Checking repository hygiene in %s (class: %s)\n\n' "${R##*/}" "${CLASS:-<class-less>}"

# --- repo.claude_md_under_line_budget ---------------------------------------
# ADR-0007. CLAUDE.md is read in full at the start of every agent session, so its length is a
# tax on every task. The budget is the point at which people stop reading it.
#
# Lowered from 250 to 150 when CLAUDE.md became a router rather than a manual. A budget at twice
# the actual size constrains nothing, and the old 250 was sized for a file that restated six
# standards inline. The router holds only what is needed to choose a route, plus the rules whose
# violation is destructive; if it grows past 150 again, something has been inlined that belongs
# behind a link.
MAX_CLAUDE_LINES=200   # ADR-0036 §4: from ADR-0007's args, the only threshold ever declared
if [[ -f "$R/CLAUDE.md" ]]; then
  n="$(wc -l < "$R/CLAUDE.md" | tr -d ' ')"
  if [[ "$n" -gt "$MAX_CLAUDE_LINES" ]]; then
    err "CLAUDE.md is $n lines, over the $MAX_CLAUDE_LINES-line budget — it is read in full every session, and past this length it stops being read at all"
  fi
else
  err "CLAUDE.md is absent — agents have no entrypoint (ADR-0007)"
fi

# --- repo.claude_md_links_resolve -------------------------------------------
# CLAUDE.md is a router (one hop to a complete target). A router with a dangling edge is worse
# than no router: an agent follows the link, finds nothing, and falls back to inferring — which
# is the single failure "never infer, retrieve" exists to prevent. So every markdown link target
# and every named skill must exist.
#
# Only relative links are checked. An http(s) link is somebody else's uptime.
if [[ -f "$R/CLAUDE.md" ]]; then
  missing=0
  while IFS= read -r target; do
    [[ -n "$target" ]] || continue
    case "$target" in http://*|https://*|"#"*|mailto:*) continue ;; esac
    t="${target%%#*}"                       # strip any anchor
    [[ -n "$t" ]] || continue
    if [[ ! -e "$R/$t" ]]; then
      err "CLAUDE.md links to '$t', which does not exist — a router with a dangling edge sends an agent to infer, which is the one thing the entrypoint forbids"
      missing=$((missing+1))
    fi
  done < <(grep -oE '\]\([^)]+\)' "$R/CLAUDE.md" | sed -E 's/^\]\(//; s/\)$//')

  # Skills are referenced by name, not path, because the harness resolves them (ADR-0007).
  # Checked separately, and only where the plugin is present in this repository.
  if [[ -d "$R/plugins/paridyum-platform/skills" ]]; then
    while IFS= read -r sk; do
      [[ -n "$sk" ]] || continue
      [[ -f "$R/plugins/paridyum-platform/skills/$sk/SKILL.md" ]] \
        || { err "CLAUDE.md names the '$sk' skill, which has no SKILL.md — the row points nowhere"; missing=$((missing+1)); }
    done < <(grep -oE '`[a-z][a-z-]+` skill' "$R/CLAUDE.md" | sed -E 's/^`//; s/` skill$//' | sort -u)
  fi
  [[ $missing -eq 0 ]] || err "CLAUDE.md has $missing unresolvable route(s)"
fi

# --- repo.entrypoint_does_not_restate_standards -----------------------------
# The guard the other two were missing. links_resolve catches a dead route and
# under_line_budget catches growth, but a router decays by somebody inlining a SUMMARY of a
# standard — which fits inside the budget and leaves every link resolving. That is the failure
# ADR-0026 exists to prevent and nothing detected it.
#
# Found two real instances in the router on its first run, in a file written that same hour to
# be a router. Rules are hard to not restate.
# Resolved against SELF_ROOT, not $R: the helper ships with the GATE, not with the repository
# under inspection. Resolving it against $R meant every synthetic fixture lacked it and the
# whole block skipped silently — the planted defect went uncaught and the suite said so.
# check-knowledge.sh resolves validate-schema.py the same way, from its own root.
DUP="$SELF_ROOT/scripts/lib/check-duplication.py"
if [[ -f "$R/CLAUDE.md" && -f "$DUP" ]]; then
  # In a consuming repository the standards live in the vendored snapshot (ADR-0001).
  dirs=()
  for d in standards constitution .paridyum/knowledge/standards .paridyum/knowledge/constitution; do
    [[ -d "$R/$d" ]] && dirs+=("$R/$d")
  done
  if [[ ${#dirs[@]} -gt 0 ]]; then
    dup_out="$(python3 "$DUP" "$R/CLAUDE.md" "${dirs[@]}" 2>&1)"
    case $? in
      0) ;;
      1) while IFS= read -r line; do [[ -n "$line" ]] && err "${line#✘ }"; done < <(grep '^✘' <<<"$dup_out") ;;
      *) wrn "duplication check unavailable (${dup_out%%$'\n'*}) — the entrypoint is unchecked for restated standards" ;;
    esac
  else
    na "repo.entrypoint_does_not_restate_standards — no standards/ or vendored snapshot to compare against"
  fi
fi

# --- repo.decision_debt_ledger_present --------------------------------------
[[ -f "$R/DECISION-DEBT.md" ]] \
  || err "DECISION-DEBT.md is absent — open loops are captured and resurfaced through it (ADR-0011)"

# --- repo.declares_paridyum_marketplace -------------------------------------
# --- repo.enables_paridyum_platform_plugin ----------------------------------
# Two separate checks deliberately. Declaring the marketplace without enabling the plugin
# installs nothing, so the hooks are available and unused — the ADR-0007 layer-1 enforcement
# silently absent while the configuration looks complete.
S="$R/.claude/settings.json"
# PARSED AS JSON, NOT GREPPED FOR SUBSTRINGS, and the difference is the whole check.
#
# This validates the file that declares ADR-0007's layer-1 PreToolUse guard — the thing that
# refuses writes to a locked artifact and to the vendored snapshot. Deciding whether that guard
# is configured by looking for the text `"paridyum"` anywhere in the file is a parser
# differential: grep and the tool reading this file disagree about what it says.
#
# The ways it disagreed, none of them exotic:
#   - the string appearing in an unrelated key, a path, or a permissions rule
#   - `"enabledPlugins": {"paridyum-platform@paridyum": false}` — present, and disabled
#   - the plugin named in a comment or in a block the reader ignores
#
# The last one is the dangerous shape, because it reports enforcement that is switched off.
# A check for a security control must ask the question the consumer asks.
if [[ -f "$S" ]]; then
  PLUGIN_STATE="$(python3 - "$S" <<'PYEOF' 2>/dev/null || echo unreadable
import json, sys
try:
    d = json.load(open(sys.argv[1]))
except Exception:
    print("unreadable"); raise SystemExit
market = "paridyum" in (d.get("extraKnownMarketplaces") or {}) or "paridyum" in (d.get("marketplaces") or {})
plugins = d.get("enabledPlugins")
if not market:
    print("no-marketplace")
elif not isinstance(plugins, dict):
    print("no-plugins-block")
elif plugins.get("paridyum-platform@paridyum") is True:
    print("enabled")
elif "paridyum-platform@paridyum" in plugins:
    print("disabled")
else:
    print("not-enabled")
PYEOF
)"
  case "$PLUGIN_STATE" in
    enabled)         : ;;
    unreadable)      err ".claude/settings.json is not valid JSON — nothing can read it, including the tool that installs ADR-0007's guard" ;;
    no-marketplace)  err ".claude/settings.json does not declare the Paridyum plugin marketplace (ADR-0007)" ;;
    no-plugins-block) err ".claude/settings.json has no enabledPlugins block — the marketplace is declared and nothing is installed (ADR-0007)" ;;
    disabled)        err ".claude/settings.json enables paridyum-platform explicitly as FALSE — ADR-0007's layer 1 is switched off while the plugin is named, which is the configuration that looks most complete and enforces least" ;;
    *)               err ".claude/settings.json declares the marketplace but does not enable paridyum-platform — the hooks are available and unused, so ADR-0007's layer 1 is off while the config looks complete" ;;
  esac
else
  err ".claude/settings.json is absent — the plugin marketplace is not declared (ADR-0007)"
fi

# --- repo.paridyum_plugin_is_installed --------------------------------------
# DECLARED IS NOT INSTALLED, and conflating the two hid a total enforcement failure. An audit on
# 2026-08-07 found `.claude/settings.json` declaring and enabling paridyum-platform while
# ~/.claude/plugins/known_marketplaces.json had never heard of the marketplace. Consequence:
# ADR-0007 layer 1 had never fired — no PreToolUse denial, no SessionStart injection — and the
# ADR-0025 Stop hook had never run automatically. Every checkpoint commit that day was invoked by
# hand. The repository looked compliant because repo.enables_paridyum_platform_plugin reads the
# config file, and a config file is a statement of intent.
#
# WARN, not error, and n/a off a workstation. This is machine state, not a repository property: CI
# will never have the plugin installed and an error would fail every build for a condition CI
# cannot satisfy. A warning is the strongest honest severity here — which is itself the argument
# for ADR-0004's plan upgrade, since GitHub-side enforcement would not depend on local install.
if [[ -d "$HOME/.claude" ]]; then
  KM="$HOME/.claude/plugins/known_marketplaces.json"
  if [[ ! -f "$KM" ]] || ! grep -qi 'paridyum' "$KM" 2>/dev/null; then
    wrn "the paridyum marketplace is NOT installed on this machine, so ADR-0007 layer 1 is OFF:
      no PreToolUse denial, no SessionStart context, and the ADR-0025 checkpoint hook never fires
      automatically. .claude/settings.json declaring it is intent, not installation. Fix with:
        /plugin marketplace add Paridyum/knowledge
        /plugin install paridyum-platform@paridyum"
  fi
else
  na "repo.paridyum_plugin_is_installed — no ~/.claude, so this is not a developer workstation"
fi

# --- repo.hooks_are_reachable -----------------------------------------------
# ADR-0027. The question that matters is not "is the plugin declared" — that check existed, passed
# throughout, and proved nothing — but "will a hook actually load by SOME path". Either the
# marketplace is installed, or settings.json registers the scripts directly with resolvable
# executable paths.
#
# `error`, unlike the plugin-specific warning above, because this IS a repository property: a repo
# that contains the hooks can always register them, and one where neither channel exists is one
# where ADR-0007 layer 1 and ADR-0025 durability are both structurally off.
#
# The python lives in scripts/lib/ rather than inline: a heredoc inside an `if` condition is a
# parse error, which this script has now hit twice.
HOOKS_VIA=""
KM2="$HOME/.claude/plugins/known_marketplaces.json"
[[ -f "$KM2" ]] && grep -qi 'paridyum' "$KM2" 2>/dev/null && HOOKS_VIA="plugin"
HREG="$SELF_ROOT/scripts/lib/check-hook-registration.py"
if [[ -f "$S" && -f "$HREG" ]]; then
  python3 "$HREG" "$S" "$R" 2>/dev/null
  case $? in
    0) HOOKS_VIA="${HOOKS_VIA:+$HOOKS_VIA and }settings.json" ;;
    2) err ".claude/settings.json registers a hook whose path is missing or not executable — configured, silent, and indistinguishable from working (ADR-0027)" ;;
  esac
fi
if [[ -n "$HOOKS_VIA" ]]; then
  printf '    hooks reachable via: %s\n' "$HOOKS_VIA"
elif [[ ! -d "$HOME/.claude" ]]; then
  # n/a off a workstation, for the reason the plugin check above already applies — and this one
  # was written without it, which turned main permanently red the day it landed.
  #
  # The severity reasoning in ADR-0027 holds for a workstation and only there. Of the two
  # channels, one is pure machine state ($HOME/.claude/plugins/known_marketplaces.json) and the
  # other is satisfiable anywhere; on a CI runner the first cannot exist, so unless settings.json
  # registers hooks directly the check is UNSATISFIABLE. Failing hard on a condition the
  # environment cannot meet leaves main red with no legal remedy — the trap ADR-0014 exists to
  # escape, and the same one knowledge-check.yml's identity step already documents.
  #
  # NOT a severity demotion: on a workstation this is still `error`. The distinction is
  # applicability, not strength, which is why knowledge.no_check_id_demotion stays quiet.
  # Note what is deliberately NOT skipped — the exit-code-2 branch above still errors
  # everywhere, because "settings.json names a hook that is missing or not executable" is a
  # repository property that CI can and must judge.
  na "repo.hooks_are_reachable — no ~/.claude, so neither hook channel is evaluable here"
else
  err "no hooks will load: the paridyum marketplace is not installed AND .claude/settings.json registers no PreToolUse/Stop hooks with resolvable executable paths. ADR-0007 layer 1 is off, and the ADR-0025 checkpoint will never fire — so nothing preserves session work and nothing denies an edit to a locked artifact (ADR-0027)"
fi

# --- repo.plugin_snapshot_matches_head --------------------------------------
# ADR-0027 finding 3. installed_plugins.json pins a gitCommitSha, so a committed hook fix is not
# active until `/plugin update` — correct in git, stale on disk, and invisible from either side.
# That is the ADR-0022 failure (configured, silent, indistinguishable from working) arriving through
# the distribution layer instead of the config file.
#
# `warn`: a snapshot legitimately lags a working tree between commits, and the pin can also name a
# commit this clone has never fetched.
IP="$HOME/.claude/plugins/installed_plugins.json"
if [[ -f "$IP" ]] && git -C "$R" rev-parse --git-dir >/dev/null 2>&1; then
  psha="$(python3 -c "
import json,sys
try:
    d=json.load(open('$IP'))['plugins']['paridyum-platform@paridyum'][0]
    print(d.get('gitCommitSha',''))
except Exception:
    pass" 2>/dev/null)"
  if [[ -n "$psha" ]]; then
    if ! git -C "$R" cat-file -e "$psha" 2>/dev/null; then
      wrn "the installed plugin pins commit ${psha:0:12}, which is not in this clone — cannot tell whether the running hooks match the repository (ADR-0027)"
    else
      # SUBTREE, not commit distance. The first version compared the pinned sha to HEAD and warned
      # on any gap, which fires on every commit that does not touch the harness — and a check that
      # fires constantly gets disabled (ADR-0008). Caught on its first real gap: the pin was one
      # commit behind while `plugins/paridyum-platform/` was byte-identical, so the running hooks
      # were provably the repository's and the warning was noise.
      #
      # What matters is whether the INSTALLED CODE differs, which is a diff over the plugin path.
      if ! git -C "$R" diff --quiet "$psha"..HEAD -- plugins/paridyum-platform/ 2>/dev/null; then
        nf="$(git -C "$R" diff --name-only "$psha"..HEAD -- plugins/paridyum-platform/ 2>/dev/null | wc -l | tr -d ' ')"
        wrn "the installed plugin is pinned at ${psha:0:12} and $nf harness file(s) have changed since — those fixes are NOT running until '/plugin update paridyum-platform@paridyum' (ADR-0027)"
      fi
    fi
  fi
fi

# --- repo.gate_warnings_are_surfaced ----------------------------------------
# ADR-0059. EVERY GATE IN THIS REPOSITORY EXITS 0 ON A WARNING, deliberately: roughly a dozen
# checks are `warn` on arguments each artifact makes carefully -- CI cannot satisfy machine state,
# a fresh clone legitimately lacks a git hook, most fetches are dead ends. A caller that keeps only
# the exit code therefore keeps only the errors, and every one of those arguments silently becomes
# "do not run this check".
#
# Not hypothetical, and the cost was measured. `run_gate` in checkpoint-session.sh ran each gate as
# `"$@" >/dev/null 2>&1`, and the generated pre-commit hook ran check-repo.sh the same way.
# repo.plugin_snapshot_matches_head reported the installed plugin as twelve days stale, by name and
# with the exact remedy, into both closed pipes -- so RFC-0027 research capture had never run once
# while every checkpoint commit recorded `repo: pass`.
#
# `error`, unlike the machine-state warnings above, and the difference is the point: this reads a
# TRACKED FILE. CI can run it and whoever it fails can fix it.
#
# It reads the SOURCES of the hooks -- plugins/ and scripts/install-hooks.sh -- rather than
# .git/hooks, which is untracked and legitimately absent in a fresh clone. Checking the generator
# is what makes this a repository property instead of a second machine-state warning.
gw_srcs=()
[[ -d "$R/plugins" ]] && while IFS= read -r f; do gw_srcs+=("$f"); done   < <(find "$R/plugins" -name '*.sh' -type f 2>/dev/null)
[[ -f "$R/scripts/install-hooks.sh" ]] && gw_srcs+=("$R/scripts/install-hooks.sh")
if [[ ${#gw_srcs[@]} -gt 0 ]]; then
  for f in "${gw_srcs[@]}"; do
    # A gate invocation whose output goes to /dev/null. Matches `check-foo.sh ... >/dev/null`
    # on one line, which is how every occurrence has actually been written.
    bad="$(grep -nE 'check-[a-z-]+\.sh"?[^|&;]*>[[:space:]]*/dev/null' "$f" 2>/dev/null            | grep -v '^[[:space:]]*#' | head -3 || true)"
    if [[ -n "$bad" ]]; then
      while IFS= read -r hit; do
        [[ -n "$hit" ]] && err "${f#"$R"/}:${hit%%:*} invokes a gate with its output discarded — every gate exits 0 on a warning, so this keeps the errors and throws away the warnings (ADR-0059)"
      done <<<"$bad"
    fi
  done
else
  na "repo.gate_warnings_are_surfaced — no plugins/ and no install-hooks.sh in this repository"
fi

# --- repo.gitattributes_normalizes_eol --------------------------------------
# ADR-0006. A CRLF shell script fails inside WSL and in containers with "bad interpreter",
# an error that never mentions line endings.
GA="$R/.gitattributes"
if [[ -f "$GA" ]]; then
  grep -qE '^\*[[:space:]]+text=auto[[:space:]]+eol=lf' "$GA" \
    || err ".gitattributes does not normalise line endings to LF for all files (ADR-0006)"
  grep -qE '^\*\.sh[[:space:]]+text[[:space:]]+eol=lf' "$GA" \
    || err ".gitattributes does not pin shell scripts to LF (ADR-0006)"
else
  err ".gitattributes is absent — line endings are unpinned (ADR-0006)"
fi

# --- repo.supply_chain_posture_is_declared ----------------------------------
# RFC-0003, REQ-0014. A TypeScript repository's supply-chain posture lives in
# pnpm-workspace.yaml, and it fails in two directions that both look like nothing.
#
# ABSENT: pnpm has no posture and says so once, quietly, at install time. Every setting
# RFC-0003 decided — the quarantine window, blockExoticSubdeps, trustPolicy — is simply not
# applied. This is what happened scaffolding web-dashboard: the file was never copied from
# the sibling it was cloned from, because "scaffold a web repo" means hand-picking files and
# hoping, and RFC-0028's template-web does not exist.
#
# PRESENT BUT A STUB: worse, and the reason this check reads content rather than existence.
# pnpm REWRITES allowBuilds on an install that meets an unapproved build script, replacing
# the mapping with `pkg: set this to true or false` and discarding the comments around it.
# The file then exists, looks deliberate, and enforces nothing. web-admin's own copy records
# that this already happened once and the placeholder was committed.
#
# The placeholder is a string where a boolean belongs, so it is mechanically detectable —
# which is the whole reason this is a check and not a note in a runbook.
if [[ -f "$R/package.json" ]]; then
  WS="$R/pnpm-workspace.yaml"
  if [[ ! -f "$WS" ]]; then
    err "pnpm-workspace.yaml is absent — RFC-0003's supply-chain posture is not applied (RFC-0003, REQ-0014)"
  else
    grep -qE '^minimumReleaseAge:' "$WS" \
      || err "pnpm-workspace.yaml declares no minimumReleaseAge — the quarantine window is a pnpm default, and a defence that depends on a default disappears on an older pnpm (RFC-0003)"
    grep -qE '^trustPolicy:' "$WS" \
      || err "pnpm-workspace.yaml declares no trustPolicy — not a pnpm 11 default, so omitting it means not having it (RFC-0003)"
    grep -qE '^blockExoticSubdeps:' "$WS" \
      || err "pnpm-workspace.yaml declares no blockExoticSubdeps (RFC-0003)"
    grep -qE '^allowBuilds:' "$WS" \
      || err "pnpm-workspace.yaml declares no allowBuilds — empty is the correct starting state, absent is not a decision (RFC-0003)"
    # The rewritten placeholder. A string where a boolean belongs.
    if grep -qE '^[[:space:]]+[^:#]+:[[:space:]]*set this to true or false' "$WS"; then
      grep -nE '^[[:space:]]+[^:#]+:[[:space:]]*set this to true or false' "$WS" | while IFS= read -r line; do
        err "pnpm-workspace.yaml carries pnpm's rewritten allowBuilds placeholder, which enforces nothing: ${line} — replace it with an explicit true or false (RFC-0003)"
      done
    fi
    # A version-pinned first-party exemption is pnpm's rewrite, not the estate's posture:
    # it expires on the next release and sends every @paridyum package into the quarantine.
    if grep -qE "^[[:space:]]+- *'?@paridyum/[a-z-]+@[0-9]" "$WS"; then
      err "pnpm-workspace.yaml exempts a first-party package BY VERSION — the posture exempts '@paridyum/*' as a class, and a pinned exemption expires on the next release (RFC-0003)"
    fi
  fi
fi

# --- repo.has_codeowners ----------------------------------------------------
if [[ ! -f "$R/CODEOWNERS" && ! -f "$R/.github/CODEOWNERS" && ! -f "$R/docs/CODEOWNERS" ]]; then
  err "no CODEOWNERS in root, .github/ or docs/ — nobody is named as reviewer (ADR-0012)"
fi

# --- repo.no_credential_files_committed -------------------------------------
# Tracked files only. An untracked .env is a developer's business; a tracked one is everyone's.
# Complements repo.secret_scan_clean, which reads contents — this reads names, and the two
# fail on different mistakes.
if git -C "$R" rev-parse --git-dir >/dev/null 2>&1; then
  bad="$(git -C "$R" ls-files | grep -EI '(^|/)(\.env(\..*)?|id_rsa|id_ed25519|.*\.pem|.*\.p12|kubeconfig|.*service-account.*\.json)$' || true)"
  if [[ -n "$bad" ]]; then
    while IFS= read -r f; do
      [[ -n "$f" ]] && err "credential-shaped file is tracked: $f — rotate it, then remove it from history (REQ-0001)"
    done <<<"$bad"
  fi
else
  na "repo.no_credential_files_committed needs a git checkout"
fi

# --- repo.no_vendored_harness -----------------------------------------------
# ADR-0007: the harness ships as a plugin and its content stays vendored. A hook script copied
# into a repository is a fork that stops receiving fixes — including the guard-locked-decisions
# hole ADR-0022 found, which would live on in every copy.
if [[ "$IS_KNOWLEDGE" == false ]]; then
  copied="$(find "$R" -path '*/hooks/*.sh' -not -path '*/.git/*' -not -path '*/plugins/*' 2>/dev/null | head -3)"
  [[ -z "$copied" ]] || err "hook scripts are copied into this repository rather than loaded from the versioned plugin: $(tr '\n' ' ' <<<"$copied") (ADR-0007)"
else
  na "repo.no_vendored_harness — this repository is the harness source"
fi

# --- repo.precommit_hook_installed ------------------------------------------
# Layer 2 of ADR-0007's ladder. Reported as a warning, not an error: the hook lives in
# .git/hooks, which is not tracked, so a fresh clone legitimately lacks it until
# install-hooks.sh runs. An error here would fail CI on every clean checkout.
H="$R/.git/hooks/pre-commit"
if [[ -f "$H" ]]; then
  grep -q 'check-knowledge' "$H" \
    || err "the pre-commit hook exists but does not invoke the knowledge gate (ADR-0007 layer 2)"
else
  wrn "no pre-commit hook installed — run scripts/install-hooks.sh. Layer 2 of ADR-0007's ladder is off in this checkout"
fi

# --- repo.ci_runs_knowledge_check -------------------------------------------
W="$R/.github/workflows"
if [[ -d "$W" ]]; then
  # A repository satisfies this EITHER by running the gate itself, OR by calling the shared
  # reusable workflow that runs it. The second case was unrepresentable until 2026-08-09: a
  # thin caller under locked ADR-0021 contains `uses: Paridyum/infra-ci/...` and never the
  # string `check-knowledge`, so this check failed every repository that followed the locked
  # decision correctly. Found by building tool-forge — the first repository to have a caller.
  #
  # Twinned with the H3 defect in infra-ci's own governance job, which did not run
  # check-knowledge.sh at all. Both halves had to be true before either could be verified,
  # which is why neither surfaced while no repository called the workflow.
  if grep -rqE 'uses:[[:space:]]*Paridyum/infra-ci/' "$W" 2>/dev/null; then
    :   # delegated to the shared workflow, which runs the gate on both triggers
  elif ! grep -rlq 'check-knowledge' "$W" 2>/dev/null; then
    err "no CI workflow invokes the knowledge gate, directly or via the shared workflow (ADR-0010, ADR-0021)"
  else
    # ANY workflow running the gate on both triggers satisfies this — not the first one `grep`
    # happens to return. `head -1` made the verdict depend on filesystem ordering, and it broke
    # on the first repository that had two: `infra-ci` holds a `workflow_call` REUSABLE workflow,
    # which by definition has no triggers of its own because the caller supplies them, plus its
    # own self-CI which has both. Picking the reusable one reported a correct repository as
    # broken. Found by building infra-ci, not by reading (2026-08-09).
    _wf_ok=""; _wf_seen=""
    while IFS= read -r wf; do
      [[ -n "$wf" ]] || continue
      _wf_seen="${_wf_seen}${_wf_seen:+, }${wf##*/}"
      grep -q 'pull_request' "$wf" || continue
      grep -qE '^[[:space:]]*push[[:space:]]*:' "$wf" || continue
      _wf_ok="$wf"; break
    done < <(grep -rl 'check-knowledge' "$W" 2>/dev/null | sort)
    [[ -n "$_wf_ok" ]] || err "no workflow runs the knowledge gate on both pull_request and push — checked: ${_wf_seen:-none}. A reusable (workflow_call) workflow cannot satisfy this on its own; the repository needs one with its own triggers (ADR-0021)"
  fi
else
  err ".github/workflows/ is absent — nothing runs in CI (ADR-0010)"
fi

# --- repo.session_checkpoint_hook_registered --------------------------------
# ADR-0025. An automation that silently stops being registered protects nothing, and its
# absence looks exactly like its success: no commits either way, right up until the disk is
# gone. So the registration is checked rather than assumed.
#
# Order is verified too, not just presence. capture-open-loops.sh rewrites DECISION-DEBT.md,
# so a checkpoint running before it leaves the regenerated ledger uncommitted until the next
# session — the exact gap ADR-0025 closes, reintroduced one file at a time.
HJ="$R/plugins/paridyum-platform/hooks/hooks.json"
CPH="$R/plugins/paridyum-platform/hooks/checkpoint-session.sh"
if [[ -f "$HJ" ]]; then
  python3 -c 'import json,sys;d=json.load(open(sys.argv[1]));stop=[h["command"] for g in d.get("hooks",{}).get("Stop",[]) for h in g.get("hooks",[])];n=[c.rsplit("/",1)[-1] for c in stop];sys.exit(1) if "checkpoint-session.sh" not in n else None;sys.exit(2) if "capture-open-loops.sh" in n and n.index("checkpoint-session.sh") < n.index("capture-open-loops.sh") else sys.exit(0)' "$HJ" 2>/dev/null
  case $? in
    0) ;;
    1) err "hooks.json does not register checkpoint-session.sh as a Stop hook — session work is not being committed or pushed automatically, and nothing else will notice (ADR-0025)" ;;
    2) err "checkpoint-session.sh runs BEFORE capture-open-loops.sh — the regenerated DECISION-DEBT.md would stay uncommitted until the next session (ADR-0025)" ;;
    *) err "hooks.json could not be parsed while checking the checkpoint hook (ADR-0025)" ;;
  esac
  [[ -x "$CPH" ]] || err "checkpoint-session.sh is registered but not executable — the hook is configured and does nothing (ADR-0025)"
else
  na "repo.session_checkpoint_hook_registered — no plugin hooks.json in this repository"
fi

# --- repo.git_remote_configured ---------------------------------------------
# ADR-0025 decision 4. A commit on one disk is still on one disk, and one disk is what failed.
# DECISION-DEBT.md reported this state for a day, and that is why ADR-0025 exists.
if git -C "$R" rev-parse --git-dir >/dev/null 2>&1; then
  if ! git -C "$R" remote 2>/dev/null | grep -q .; then
    err "no git remote is configured — every checkpoint commit still exists on exactly one disk, which is the failure ADR-0025 was written for"
  fi
else
  na "repo.git_remote_configured needs a git checkout"
fi

# --- docs.required_files_present --------------------------------------------
# Per STD-0002, keyed on class. A class-less repository is governed by the org-scoped set and
# has no manifest.yaml by design (ADR-0015 point 6), so requiring one here would be requiring
# `knowledge` to become something it deliberately is not.
for f in README.md CHANGELOG.md; do
  [[ -f "$R/$f" ]] || err "$f is absent (STD-0002)"
done
if [[ -n "$CLASS" ]]; then
  [[ -f "$R/ARCHITECTURE.md" ]] || err "ARCHITECTURE.md is absent (STD-0002)"
  case "$CLASS" in
    service|platform)
      for f in API.md RUNBOOK.md; do
        [[ -f "$R/$f" ]] || err "$f is absent, and class '$CLASS' requires it (STD-0002)"
      done ;;
  esac

  # --- docs.sop_present -----------------------------------------------------
  # STD-0013. A separate id rather than a second `args` set on docs.required_files_present,
  # because knowledge.checks_have_consistent_parameters rejects one id cited two ways and
  # STD-0002 already set the precedent with docs.service_files_present.
  [[ -f "$R/SOP.md" ]] \
    || err "SOP.md is absent — how this is operated exists only in the transcript of the session that built it (STD-0013)"
else
  na "docs.required_files_present (ARCHITECTURE.md, manifest.yaml) — this repository is class-less by design (ADR-0015)"
  na "docs.service_files_present (API.md, RUNBOOK.md) — no svc-* or platform-* repository exists"
  na "docs.sop_present — class-less by design (ADR-0015 point 6); operating procedure lives in runbooks/"
fi

# --- docs.diagram_source_is_present / docs.diagram_ir_is_valid ---------------
# --- docs.diagram_is_self_contained ------------------------------------------
# STD-0020 §3: an IR beside every render, and both committed. The three failures are distinct
# and only one of them is visible to a reader.
#
#   NO SOURCE      the HTML opens perfectly and cannot be changed by anyone. C-0004 makes
#                  generated output regenerable; a render with no IR is neither generated nor
#                  authored, and the defect surfaces the day someone needs to fix a label.
#   INVALID IR     renders nothing, and the failure lands on whoever regenerates next rather
#                  than on whoever introduced it.
#   EXTERNAL FETCH the vendor's template links JetBrains Mono from fonts.googleapis.com. The
#                  diagram works offline, so this is not a rendering failure -- it is a
#                  disclosure: opening a committed governance diagram tells a third party that
#                  someone read it. scripts/render-diagram.sh strips it; a bare `archify
#                  deliver` does not, which is exactly the mistake this catches.
#
# THE INVERSE IS NOT CHECKED. An IR with no render is legal: it is authored source in progress,
# and STD-0020 §6 requires both in the same COMMIT, not both at every moment.
#
# VALIDATION SKIPS WHEN ARCHIFY IS ABSENT, AND SAYS SO. A hard failure would make every
# contributor install a Node tool at a pinned sha to commit an unrelated change, which is a gate
# people bypass -- ADR-0002's argument, applied to the cost of running rather than to the cost
# of writing. The skip is printed, never silent. The other two need no tool and always run.
DIAGDIR="$R/docs/diagrams"
if [[ ! -d "$DIAGDIR" ]]; then
  na "docs.diagram_* — no docs/diagrams/ in this repository"
else
  ARCHIFY=""
  if [[ -x "$R/scripts/tool-path.sh" ]]; then
    ARCHIFY="$("$R/scripts/tool-path.sh" archify 2>/dev/null || true)"
  fi
  [[ -n "$ARCHIFY" && -x "$ARCHIFY" ]] || ARCHIFY=""
  [[ -n "$ARCHIFY" ]] || wrn "docs.diagram_ir_is_valid — archify is not installed at its pinned ref, so committed IR was NOT validated. Run scripts/install-tools.sh archify"

  shopt -s nullglob
  for html in "$DIAGDIR"/*.html; do
    base="${html%.html}"
    [[ -f "$base.json" ]] \
      || err "${html#"$R"/} has no IR beside it — a render nobody can regenerate is not generated output (C-0004, STD-0020 §3)"
    if grep -qE '(src|href)="https?://' "$html"; then
      err "${html#"$R"/} reaches an external origin when opened — render through scripts/render-diagram.sh, not a bare \`archify deliver\` (ADR-0065 decision 7)"
    fi
  done

  for ir in "$DIAGDIR"/*.json; do
    stem="${ir%.json}"
    kind="${stem##*.}"
    case "$kind" in
      workflow|sequence|dataflow|lifecycle) ;;
      architecture)
        err "${ir#"$R"/} is a hand-authored architecture IR — the model is manifest.yaml and every cross-repository view is generated from it (ADR-0040 decision 2, ADR-0065 decision 2)"
        continue ;;
      *)
        err "${ir#"$R"/} does not name its kind — the filename must end .<kind>.json so a reader knows which command regenerates it (STD-0020 §3)"
        continue ;;
    esac
    [[ -n "$ARCHIFY" ]] || continue
    if ! out="$("$ARCHIFY" validate "$kind" "$ir" 2>&1)"; then
      err "${ir#"$R"/} does not validate as $kind: $(printf '%s' "$out" | head -2 | tr '\n' ' ')"
      continue
    fi
    # docs.diagram_is_current. Delegated to render-diagram.sh --check so the font strip has one
    # home; a second copy of it here would drift, and the day it drifted every diagram would
    # report stale for a reason nobody could find. Only runs when the renderer script exists in
    # the repository under test, which is what makes this safe against a repository that carries
    # diagrams without the script.
    if [[ -x "$R/scripts/render-diagram.sh" ]]; then
      if ! cur="$("$R/scripts/render-diagram.sh" --check "$ir" 2>&1)"; then
        err "${ir#"$R"/}: the committed render does not match its IR — re-run scripts/render-diagram.sh on it (C-0004, STD-0020 §6)"
      fi
    fi
  done
  shopt -u nullglob
fi

# --- test.* — STD-0008, the four that need no Rust ---------------------------
# Declared `owner: tool-forge` when STD-0008 was written and moved here the same week, which
# is the third time this repository has caught itself doing that. The header above states the
# rule: a check that reads files in a checkout does not need a binary nobody has written, and
# leaving it declared is ADR-0002's gap in a more comfortable form.
#
# WHAT STAYS DECLARED, and why, so this list is not read as the whole of STD-0008:
# test.no_io_in_unit_tier, test.mutation_score_meets_threshold, test.tenant_isolation_*,
# test.authorization_matrix_*, test.operator_scope_denial_is_asserted and
# test.migration_is_applied_and_reverted_in_ci all need a test suite to RUN, not a file to
# read. They cannot be honestly implemented against a repository that has no tests.

HAS_TESTS=false
[[ -d "$R/tests" || -d "$R/src" || -n "$(find "$R" -maxdepth 2 -name 'test-*.sh' -not -path '*/.git/*' 2>/dev/null)" ]] && HAS_TESTS=true

# --- test.testing_md_declares_untested_surface -------------------------------
# STD-0001 asked for the untested surface to be stated "in the repository's TESTING.md" and
# nothing required that file to exist, so the requirement was satisfiable by omission — the
# same shape as docs.required_files_present, and the reason the fix is a required file rather
# than a prose reminder. The four sections are what make it a decision rather than a placeholder.
TESTING_MD="$R/TESTING.md"
if [[ "$HAS_TESTS" != true ]]; then
  na "test.testing_md_declares_untested_surface — no tests in this repository to describe"
elif [[ ! -f "$TESTING_MD" ]]; then
  err "TESTING.md is absent, and this repository has tests (STD-0008)"
else
  missing=""
  for s in Tiers "Not tested" Quarantine Evidence; do
    grep -qiE "^#{1,3} +${s}\b" "$TESTING_MD" || missing="$missing '$s'"
  done
  [[ -z "$missing" ]] || err "TESTING.md is missing required section(s):$missing (STD-0008)"
fi

# --- test.tiers_are_declared_in_ci -------------------------------------------
# Keeps the pipeline and test-registry.yaml one vocabulary rather than two. Implemented against
# the TIERS SECTION of TESTING.md rather than by parsing CI YAML, and the difference is worth
# stating: parsing a workflow means guessing which step is which test type, which is exactly the
# kind of inference that produces a check verifying less than its name implies. STD-0008 already
# requires the mapping to be declared; this checks the declaration resolves.
TREG="$R/test-registry.yaml"
[[ -f "$TREG" ]] || TREG="$R/.paridyum/knowledge/test-registry.yaml"
if [[ ! -f "$TESTING_MD" || ! -f "$TREG" ]]; then
  na "test.tiers_are_declared_in_ci — needs TESTING.md and test-registry.yaml (vendored or local)"
else
  known_ids="$(grep -oE '^  - test_type: TST-[0-9]{4}' "$TREG" | awk '{print $NF}' | sort -u)"
  known_tiers="$(grep -oE '^  - tier: [a-z-]+' "$TREG" | awk '{print $NF}' | sort -u)"
  # `\b` is a GNU grep extension and is NOT in POSIX ERE, so awk matches it as a literal or
  # as backspace depending on the implementation — either way the section never matched and
  # three planted defects walked straight through. Anchored on the heading's end instead.
  tiers_block="$(awk '/^#{1,3}[[:space:]]+[Tt]iers[[:space:]]*$/{f=1;next} f && /^#{1,3}[[:space:]]/{exit} f' "$TESTING_MD")"
  unknown=""
  while IFS= read -r id; do
    [[ -n "$id" ]] || continue
    grep -qx "$id" <<<"$known_ids" || unknown+="$id "
  done < <(grep -oE 'TST-[0-9]{4}' <<<"$tiers_block" | sort -u)
  [[ -z "${unknown// /}" ]] \
    || err "TESTING.md's Tiers section names test type(s) absent from test-registry.yaml: ${unknown% } (ADR-0029)"
  # TIER NAMES ARE DELIBERATELY NOT VALIDATED HERE, and the first version of this check did
  # try. It read every backticked lowercase word in the section as a tier, which made `push`
  # — a GitHub event named in this repository's own TESTING.md — a tier that does not exist,
  # and four unrelated fixture cases failed with it. Prose is not a declaration: a `TST-` id is
  # a distinctive token that can only be one thing, and a backticked word is anything at all.
  # Validating it would need a table format STD-0008 never required, so the check verifies the
  # half it can and `partial: true` says so.
fi

# --- test.flaky_tests_are_quarantined_with_an_owner --------------------------
# The second clause of the spec is the one that matters. An entry with full paperwork that is
# still running in a blocking tier has documented the flake rather than removed it, and the
# "just rerun CI" habit it teaches costs more than the failures the suite would have caught.
# Five columns, because each answers a question that decides whether the entry can ever close:
# who, since when, how often, tracked where, and by when. With the test name that is SIX cells,
# and the threshold was `-ge 5` — so an entry missing exactly one column was accepted. The
# column it was missing in practice is the expiry, which is the one that matters most: FLAKY-003
# says an entry expires and "silence is not renewal", and an entry with no expiry is the
# permanent quarantine that rule exists to prevent. The registry spec names the expiry
# explicitly, so this was the implementation under-enforcing its own declaration (ADR-0014).
if [[ -f "$TESTING_MD" ]] && grep -qiE '^#{1,3} +Quarantine\b' "$TESTING_MD"; then
  qblock="$(awk '/^#{1,3}[[:space:]]+[Qq]uarantine[[:space:]]*$/{f=1;next} f && /^#{1,3}[[:space:]]/{exit} f' "$TESTING_MD")"
  while IFS= read -r row; do
    # Table rows only; skip the header, the separator, and the empty-state placeholder row.
    [[ "$row" == \|* ]] || continue
    grep -qiE '^\| *(Test|-+|—) *\|' <<<"$row" && continue
    # Emptiness is decided in bash, not by `tr`. `tr -d '|—- '` read the `-` between the
    # em-dash and the space as a RANGE: GNU tr rejects it ("range-endpoints ... in reverse
    # collating sequence order"), exits 1 and prints nothing, so `-z` was true for EVERY row
    # and each one was skipped as blank. BSD tr accepts the same set, which is why this passed
    # on macOS and enforced nothing in CI for as long as it existed. A `-` in a tr set is only
    # safe last, and an external tool that fails must never be indistinguishable from a clean
    # result. Bash substitution has no locale, no implementation and no exit status to swallow.
    stripped="${row//[|[:space:]-]/}"; stripped="${stripped//—/}"
    if [[ -z "$stripped" ]]; then
      # ABSENT IS NOT PASS. `| — | | | | | |` is the documented empty state and is legitimate.
      # A row still holding letters or digits is NOT blank, and reading it as blank is exactly
      # how the defect above stayed invisible: the filter is wrong, not the table. Say so
      # rather than skipping, because a skipped row is a silent pass.
      [[ "$row" =~ [A-Za-z0-9] ]] \
        && err "TESTING.md quarantine row was read as empty but contains content — the row filter in this check is broken, not the table (report it; do not edit the table to silence it): ${row:0:60}"
      continue
    fi
    cells="$(awk -F'|' '{for(i=2;i<NF;i++){gsub(/^ +| +$/,"",$i); printf "%s\n",$i}}' <<<"$row")"
    filled="$(grep -cvE '^$' <<<"$cells" || true)"
    [[ "$filled" -ge 6 ]] \
      || err "TESTING.md quarantine entry is incomplete — needs test, owner, first seen, failure rate, tracking reference and expiry (STD-0008 FLAKY-002, FLAKY-003): ${row:0:60}"
  done <<<"$qblock"
fi

# --- test.default_profile_does_not_retry -------------------------------------
# STD-0001 sets retries = 0 in the default profile and a bounded backoff in CI, and explains
# the split: CI retries absorb infrastructure noise, not flakiness. The default profile is the
# one a person runs, and a retry there removes the only cheap opportunity to see a flake.
NEXTEST="$R/.config/nextest.toml"
if [[ ! -f "$NEXTEST" ]]; then
  na "test.default_profile_does_not_retry — no .config/nextest.toml in this repository"
else
  dflt="$(awk '/^\[profile\.default\]/{f=1;next} /^\[/{f=0} f' "$NEXTEST")"
  retries="$(grep -oE '^[[:space:]]*retries[[:space:]]*=[[:space:]]*[0-9]+' <<<"$dflt" | grep -oE '[0-9]+$' || true)"
  [[ -z "$retries" || "$retries" -eq 0 ]] \
    || err ".config/nextest.toml sets retries = $retries in [profile.default] — a retry there hides the flake at the only moment it is cheap to see (STD-0001, STD-0008)"
fi

printf '\n'
if [[ "$EXPECT_FAIL" == true ]]; then
  [[ $ERRORS -gt 0 ]] && { printf '%s✔%s expected failure produced %d error(s)\n' "$GRN" "$RST" "$ERRORS"; exit 0; }
  printf '%s✘%s expected at least one error and got none — the planted defect was NOT caught\n' "$RED" "$RST"; exit 1
fi
if [[ $ERRORS -gt 0 ]]; then
  printf '%s%d error(s)%s, %d warning(s), %d n/a\n' "$RED" "$ERRORS" "$RST" "$WARNS" "$NA"; exit 1
fi
printf '%sRepository check passed%s (%d warning(s), %d n/a)\n' "$GRN" "$RST" "$WARNS" "$NA"
