#!/usr/bin/env bash
#
# Knowledge integrity gate. Runs in CI and pre-commit.
#
# This is the stand-in for `forge knowledge check` until tool-forge ships. When it
# does, this script is replaced by a call to the binary and deleted — it exists so
# the gate is real from day one rather than promised. See ADR-0002.
#
#   ./check-knowledge.sh          check everything
#   ./check-knowledge.sh --staged only files staged for commit

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


ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1

if [[ -t 1 ]]; then
  RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; RST=$'\033[0m'
else
  RED=""; GRN=""; YLW=""; RST=""
fi
ERRORS=0
WARNS=0
err()  { printf '%s✘%s %s\n' "$RED" "$RST" "$1"; ERRORS=$((ERRORS+1)); }
wrn()  { printf '%s!%s %s\n' "$YLW" "$RST" "$1"; WARNS=$((WARNS+1)); }

# Hoisted: the manifest checks in section 7 read the class registry, and defining it in
# section 8 left it unbound there. `set -u` turned that into a hard failure rather than a
# silent skip, which is the correct direction and is why it surfaced immediately.
CLASSREG="class-registry.yaml"
ARTREG="artifact-registry.yaml"
# Read once. A per-artifact `date` call is a fork per file, and the value must not drift
# mid-run — an artifact reviewed on either side of midnight would report inconsistently.
TODAY="$(date +%F)"
SCHEMA="knowledge.schema.json"

# --- 0. artifact-type registry (ADR-0022) -----------------------------------
# ART_DIRS, the id pattern, the cross-reference prefix set, the immutability directory
# filter and the per-type status vocabulary were five hardcoded lists. Adding a type meant
# editing all five and four more sites elsewhere, and missing any of the five failed
# SILENTLY: a directory absent from ART_DIRS is not scanned, so an entire artifact class
# goes unenforced while this script prints "Knowledge check passed". They are now derived
# from artifact-registry.yaml, and section 8b verifies the registry against the schema
# values it generates.
#
# Parsed with awk rather than a YAML library for the reason validate-schema.py exists: a
# pip dependency in a gate that runs on every commit is a gate people bypass the first time
# an index is down. `notes: >` bodies are indented six spaces and cannot collide with the
# four-space field matches below.
artreg_entries() {
  awk '
    function flush() {
      if (t != "")
        printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n", t,
          (pfx == "" ? "-" : pfx), (dir == "" ? "-" : dir),
          (lockable == "" ? "-" : lockable), (deferrable == "" ? "-" : deferrable),
          (rt == "" ? "-" : rt), (rs == "" ? "-" : rs), (imm == "" ? "-" : imm)
    }
    /^  - type: / { flush(); t = $3; pfx=""; dir=""; lockable=""; deferrable=""; rt=""; rs=""; imm=""; next }
    /^    prefix: /                { pfx = $2;        next }
    /^    directory: /             { dir = $2;        next }
    /^    lockable: /              { lockable = $2;   next }
    /^    deferrable: /            { deferrable = $2; next }
    /^    requires_trigger: /      { rt = $2;         next }
    /^    requires_sources: /      { rs = $2;         next }
    /^    immutable_when_locked: / { imm = $2;        next }
    END { flush() }
  ' "$1"
}

declare -A T_LOCKABLE=() T_DEFERRABLE=() T_TRIGGER=() T_SOURCES=() T_DIR=()
ART_DIRS=() IMMUTABLE_DIRS=() ID_PREFIXES=()

if [[ ! -f "$ARTREG" ]]; then
  # Fatal rather than a fallback to the old hardcoded lists. A gate that silently reverts
  # to a stale copy of the data it was supposed to read is the failure mode ADR-0014's
  # audit found four times; better to refuse to run than to run against a guess.
  printf '%s✘%s %s is missing — the artifact type list lives there (ADR-0022)\n' "$RED" "$RST" "$ARTREG"
  exit 1
fi

# Deduplicated as they are collected. Section 8b rejects a registry with two types sharing
# a `directory` or `prefix`, but section 1 runs first, and an undeduplicated ART_DIRS made
# `find` scan that directory twice — so every artifact in it was reported as a duplicate of
# ITSELF, with the same path printed on both sides. Planting the defect produced 27 errors
# of which 23 were that, burying the one that explained the cause. Deduplicating here
# changes nothing about what is rejected and everything about which message is read first.
declare -A _seen_dir=() _seen_pfx=()
while IFS=$'\t' read -r t pfx dir lockable deferrable rt rs imm; do
  [[ -n "$t" ]] || continue
  T_DIR[$t]="$dir"; T_LOCKABLE[$t]="$lockable"
  T_DEFERRABLE[$t]="$deferrable"; T_TRIGGER[$t]="$rt"; T_SOURCES[$t]="$rs"
  if [[ -z "${_seen_dir[$dir]:-}" ]]; then
    _seen_dir[$dir]=1
    ART_DIRS+=("$dir")
    [[ "$imm" == "true" ]] && IMMUTABLE_DIRS+=("$dir")
  fi
  if [[ -z "${_seen_pfx[$pfx]:-}" ]]; then
    _seen_pfx[$pfx]=1
    ID_PREFIXES+=("$pfx")
  fi
done < <(artreg_entries "$ARTREG")

if [[ ${#ID_PREFIXES[@]} -eq 0 ]]; then
  printf '%s✘%s %s declares no artifact types — nothing would be scanned (ADR-0022)\n' "$RED" "$RST" "$ARTREG"
  exit 1
fi

# The alternation used by the id validator and the cross-reference scanner. Longest-first
# so that alternation cannot short-circuit: with `C` before `CI`, an ERE would match the
# `C` and leave `I` unconsumed. No such pair exists today; sorting removes the class of bug
# rather than relying on it staying absent.
ID_ALT="$(printf '%s\n' "${ID_PREFIXES[@]}" | awk '{print length, $0}' | sort -rn -k1,1 -k2,2 | awk '{print $2}' | paste -sd'|' -)"
ID_RE="^($ID_ALT)-[0-9]{4}\$"
REF_RE="\\b($ID_ALT)-[0-9]{4}\\b"

mapfile -t FILES < <(
  for d in "${ART_DIRS[@]}"; do [[ -d "$d" ]] && find "$d" -name '*.md' -type f; done | sort
)


printf 'Checking %d knowledge artifacts\n\n' "${#FILES[@]}"

# Frontmatter reader, memoised. This forked an awk PER FIELD PER FILE — roughly 1100 forks
# across a run, which was most of the 4.4 seconds this gate cost. It is invoked by the ADR-0025
# Stop hook on every response and by the pre-commit hook on every commit, so its runtime is a tax
# on every interaction, and a slow gate is one people reach for --no-verify to skip.
#
# One awk pass per file now loads every top-level key. Nested keys (the `- check:` entries under
# enforced_by) are indented and so are correctly not captured, and a folded value like `trigger: >`
# yields `>` exactly as the per-field version did — callers only test it for emptiness.
declare -A FM_CACHE=()
fm_load() {
  local f="$1" k v
  while IFS=$'\t' read -r k v; do FM_CACHE["$f|$k"]="$v"; done < <(
    awk 'NR==1 && $0!="---"{exit} NR>1 && $0=="---"{exit}
         NR>1 && /^[a-z_]+:/ { k=$0; sub(/:.*/,"",k); v=$0; sub(/^[^:]*:[ \t]*/,"",v);
                               printf "%s\t%s\n", k, v }' "$f" 2>/dev/null
  )
  FM_CACHE["$f|__loaded"]=1
}
fm() {
  [[ -n "${FM_CACHE["$1|__loaded"]:-}" ]] || fm_load "$1"
  printf '%s' "${FM_CACHE["$1|$2"]:-}"
}

# Warmed HERE, not earlier. The first attempt put this loop above the function definitions: bash
# printed "fm_load: command not found" to stderr and carried on, because nothing here runs under
# `set -e`, and the `declare -A FM_CACHE=()` below it then reset the array. Two rounds of measuring
# showed no improvement for that reason, not because memoising was the wrong idea — the trace
# still showed 1693 fm_load awks after both attempts.
for _f in "${FILES[@]}"; do fm_load "$_f"; done

# THE CACHE MUST BE WARMED IN THE PARENT SHELL, and this line is the whole reason memoising
# worked. `fm` is almost always called as `$(fm "$f" id)` — a command substitution, which is a
# subshell — so a cache written lazily inside it is populated in the subshell and discarded when
# it exits. Every call was a miss, and the lazy version did strictly MORE work than the
# per-field awk it replaced, because it parsed all keys instead of one.
#
# Measured, not assumed: adding the memo alone moved the runtime from 4414ms to 4676ms, and the
# trace showed 1802 awk invocations still there. Warming it here runs one awk per file, in the
# parent, where the writes survive.

# --- 1. frontmatter validity -------------------------------------------------
# knowledge.frontmatter_valid: real JSON Schema validation, not field presence. This was
# an approximation for its whole life — it accepted frontmatter knowledge.schema.json
# rejects — and the registry counted it as implemented anyway. scripts/lib/validate-schema.py
# is a stdlib validator written for this; a pip dependency in a gate that runs on every
# commit is a gate people bypass the first time an index is down.
# Exit 2 means it could not validate (no PyYAML); that falls back to the field-presence
# checks below and SAYS SO, because a validator that cannot run must never look like one
# that passed.
if [[ -f scripts/lib/validate-schema.py && ${#FILES[@]} -gt 0 ]]; then
  vout="$(python3 scripts/lib/validate-schema.py frontmatter knowledge.schema.json "${FILES[@]}" 2>&1)"
  case $? in
    0) ;;
    1)
       while IFS= read -r line; do [[ -n "$line" ]] && err "${line#✘ }"; done \
         < <(grep '^✘' <<<"$vout") ;;
    *) wrn "frontmatter schema validation unavailable (${vout%%$'\n'*}) — falling back to field-presence checks only" ;;
  esac
fi

declare -A SEEN_ID
for f in "${FILES[@]}"; do
  head -1 "$f" | grep -q '^---$' || { err "$f: no frontmatter block"; continue; }
  for field in id title type status scope created; do
    [[ -n "$(fm "$f" "$field")" ]] || err "$f: missing required field '$field'"
  done
  id="$(fm "$f" id)"
  # Never use an unvalidated value as an associative-array subscript: an empty
  # subscript is a bash error that leaves the array unusable, which previously
  # cascaded into false-positive "dangling reference" reports for ids that exist.
  if [[ ! "$id" =~ $ID_RE ]]; then
    err "$f: id '${id:-<missing>}' does not match <PREFIX>-NNNN"
  elif [[ -n "${SEEN_ID[$id]:-}" ]]; then
    err "duplicate id $id: $f and ${SEEN_ID[$id]}"
  else
    SEEN_ID[$id]="$f"
  fi
  st="$(fm "$f" status)"
  ty="$(fm "$f" type)"
  # Same hazard as the id subscript above, one field over: an empty `type` used as an
  # associative-array subscript is a bash error that corrupts the array for every later
  # iteration. The missing field is already reported by the required-field loop, so this
  # only needs a subscript that is safe and absent from the registry.
  tykey="${ty:-__missing__}"
  if [[ -n "$ty" && -z "${T_DIR[$tykey]:-}" ]]; then
    err "$f: type '$ty' is not declared in $ARTREG (ADR-0022)"
  fi
  case "$st" in
    draft|proposed|accepted|superseded|deprecated) ;;
    locked)
      # ADR-0022: `lockable` is registry data. `locked` means must-not-be-contradicted
      # (CLAUDE.md), which an RFC filed under open questions cannot mean and which a bet
      # must never mean — a bet exists to be contradicted by evidence on a date already
      # written down. Defaulting to `true` keeps an undeclared type from silently gaining
      # a restriction it was never given.
      [[ "${T_LOCKABLE[$tykey]:-true}" == "true" ]] \
        || err "$f: type '$ty' may never be 'locked' — $ARTREG declares lockable: false, because locking a falsifiable artifact makes the word mean two incompatible things (ADR-0022)"
      ;;
    deferred)
      # ADR-0018: `deferred` is meaningless without the condition that would make it
      # current. A deferred requirement with no trigger is a TODO with better formatting —
      # nobody knows what would make it actionable, so it never gets revisited. That is the
      # whole failure REQ- exists to avoid, so it is an error rather than a warning.
      # The `type == requirement` comparison this replaced was correct and hardcoded; it is
      # now `deferrable` in the registry, which is what lets `bet` reuse the trigger
      # machinery without also inheriting `deferred`.
      [[ "${T_DEFERRABLE[$tykey]:-false}" == "true" ]] \
        || err "$f: status 'deferred' is only valid for a type declaring deferrable: true in $ARTREG (ADR-0018)"
      [[ -n "$(fm "$f" trigger)" ]] \
        || err "$f: status 'deferred' requires a 'trigger' — the condition that would make it current, not a date (ADR-0018)"
      ;;
    *) err "$f: invalid status '$st'" ;;
  esac
  # requires_trigger: always — ADR-0018's argument at its other setting. A bet with no
  # stated losing condition is an opinion with frontmatter, and a bet with no scheduled
  # verdict quietly becomes policy, so both fields are errors rather than warnings.
  if [[ "${T_TRIGGER[$tykey]:-never}" == "always" ]]; then
    [[ -n "$(fm "$f" trigger)" ]] \
      || err "$f: type '$ty' requires a 'trigger' — the evidence that would kill it, not a date (ADR-0022)"
    [[ -n "$(fm "$f" review_date)" ]] \
      || err "$f: type '$ty' requires a 'review_date' — a wager nobody scheduled a verdict for becomes policy by default (ADR-0022)"
  fi
  # knowledge.review_dates_not_overdue. Requiring the field and never reading it was the
  # defect: a bet could carry a scheduled verdict that passed months ago and the gate stayed
  # green, which is exactly the "quietly becomes policy" failure ADR-0022 claims to prevent.
  # A review_date nothing surfaces is the same artifact as no review_date.
  #
  # `warn`, not `err`, and time-dependent — the same bargain ADR-0011 struck for
  # knowledge.no_stale_proposed_artifacts: it can turn a previously green commit red with no
  # diff, so it must not block a commit. ISO dates compare correctly as strings.
  rd="$(fm "$f" review_date)"
  if [[ -n "$rd" && "$rd" != "null" && "$rd" < "$TODAY" ]]; then
    if [[ "${T_TRIGGER[$tykey]:-never}" == "always" ]]; then
      wrn "$(fm "$f" id) review_date $rd has passed — the verdict on this wager is overdue. Judge it against its trigger and either supersede it or move the date deliberately (ADR-0022)"
    else
      wrn "$(fm "$f" id) review_date $rd has passed — reconsider it even though its trigger has not fired, which is what the field is for (ADR-0018)"
    fi
  fi
done

# --- 1b. rfc.has_alternatives_section + knowledge.no_stale_proposed_artifacts
# Both were declared `owner: tool-forge` and neither needs Rust — they read frontmatter and
# headings, which this script already does. Twelve such checks were waiting on a binary while
# waiting on nothing at all.
#
# An RFC without rejected alternatives is a proposal wearing a comparison's clothes. C-0001
# principle 8 makes the same point about ADRs: the value of the record is the alternatives and
# the accepted costs, not the recommendation.
# The `[0-9]+\. ` prefix is not cosmetic tolerance: RFC-0009 numbers its sections because the
# decompose-spec procedure is numbered, and the check rejected it for having
# '## 8. Alternatives considered' rather than '## Alternatives considered'. The artifact was
# right and the check was too strict — found by writing an RFC that followed a locked
# procedure and watching the gate reject it.
ALT_HEADINGS='^#{2,3} ([0-9]+\. )?(Options|Alternatives|Options considered|Alternatives considered|Rejected alternatives|Alternatives rejected)'
MAX_PROPOSED_AGE_DAYS=30
for f in "${FILES[@]}"; do
  case "$(fm "$f" type)" in
    rfc)
      grep -qE "$ALT_HEADINGS" "$f" \
        || err "$(fm "$f" id): no alternatives section — an RFC recording only its recommendation is a proposal, not a comparison, and the rejected options are the part future readers need (ADR-0013)"
      ;;
  esac
  if [[ "$(fm "$f" status)" == "proposed" ]]; then
    created="$(fm "$f" created)"
    if [[ -n "$created" ]]; then
      # Portable day arithmetic without GNU date: compare as epoch via python only if the
      # artifact is plausibly old, so the common case forks nothing.
      if [[ "$created" < "$(python3 -c "
import datetime,sys
print((datetime.date.fromisoformat('$TODAY') - datetime.timedelta(days=$MAX_PROPOSED_AGE_DAYS)).isoformat())" 2>/dev/null || echo 0000-00-00)" ]]; then
        wrn "$(fm "$f" id) has been 'proposed' since $created, over $MAX_PROPOSED_AGE_DAYS days — an RFC nobody closed is a decision being made by default (ADR-0011)"
      fi
    fi
  fi
done

# --- 2. cross-reference resolution ------------------------------------------
# DECISION-DEBT.md is excluded, and the exclusion is load-bearing rather than tidiness.
# The ledger reports dangling ids by citing them, so scanning it makes this check
# self-inflicting: the moment the Stop hook records a dangling reference, the ledger
# itself becomes a citation of that id and the gate goes red permanently — unfixable,
# because the ledger is machine-generated and marked do-not-hand-edit. Excluding it
# means this check only ever fires on a reference a human actually wrote.
while IFS= read -r line; do
  ref="${line##*:}"; src="${line%%:*}"
  [[ -n "${SEEN_ID[$ref]:-}" ]] || err "dangling reference $ref in $src"
done < <(
  # ONE pass, not one per id. This was a full-repo grep for every unique id found — around
  # eighty of them — which cost 5.8 seconds and dominated the runtime of a gate the ADR-0025
  # Stop hook runs on every response. `-o` with filenames plus an awk group-by gives the same
  # id-to-files mapping from a single traversal.
  #
  # Every citing file, not `head -1`. Reporting only the first made a widely-cited typo look
  # like a single occurrence, which understates the work to fix it.
  # *.yaml included: the registries cite artifact ids too — check-registry.yaml's `blocks`,
  # framework-registry.yaml's `mapping_deferred_to` — and a typo there was invisible to
  # every check. `blocks` was covered by section 7's own resolution pass, but nothing else
  # in a registry was, so a new field naming an artifact got no scrutiny at all.
  grep -roE --include='*.md' --include='*.sh' --include='*.yaml' --exclude='DECISION-DEBT.md' \
    "$REF_RE" . 2>/dev/null \
  | sed 's|^\./||' \
  | awk -F: '{ if (!seen[$2 FS $1]++) { srcs[$2] = srcs[$2] (srcs[$2] ? " " : "") $1 } }
             END { for (id in srcs) printf "%s:%s\n", srcs[id], id }'
)

# --- 3. locked-artifact integrity -------------------------------------------
# A locked artifact must not be modified in place; only supersession is legal.
# Enforced live by the PreToolUse hook (ADR-0007); re-checked here for edits made
# outside Claude Code.
if git rev-parse --git-dir >/dev/null 2>&1 && git rev-parse HEAD >/dev/null 2>&1; then
  base="${1:-HEAD}"
  [[ "$base" == "--staged" ]] && base="HEAD"
  while IFS= read -r f; do
    [[ -f "$f" ]] || continue
    # Was a hardcoded alternation of six directories; now the set of types declaring
    # `immutable_when_locked: true` (ADR-0022). Built as a loop rather than a `case`
    # pattern from a variable: `|` separates case patterns at parse time, before parameter
    # expansion, so "$IMM" holding `a/*|b/*` matches a literal pipe and silently protects
    # nothing.
    protected=false
    for d in "${IMMUTABLE_DIRS[@]}"; do
      [[ "$f" == "$d"/* ]] && { protected=true; break; }
    done
    [[ "$protected" == true ]] || continue
    was_locked="$(git show "$base:$f" 2>/dev/null | awk 'NR==1 && $0!="---"{exit} NR>1 && $0=="---"{exit} /^status:/{print $2; exit}')"
    # `superseded` is included deliberately. Immutability used to lapse the moment an
    # artifact was superseded, which is exactly when it becomes purely historical and
    # most needs protecting: ADR-0002's central sentence was inverted in a test and
    # both this gate and the PreToolUse hook permitted it, green, no warning. A
    # superseded artifact should be MORE frozen than a locked one, not less — the
    # record of what was true when the existing code was written is the whole value of
    # the archive (ADR-0002, and the new-decision skill's "never delete the old file").
    case "$was_locked" in locked|superseded) ;; *) continue ;; esac
    changed="$(git diff "$base" -- "$f" | grep -cE '^[+-][^+-]' || true)"
    only_meta="$(git diff "$base" -- "$f" | grep -E '^[+-][^+-]' | grep -cvE '^[+-](status|superseded_by|locked_at):' || true)"
    if [[ "$changed" -gt 0 && "$only_meta" -gt 0 ]]; then
      # knowledge.errata_are_additive_only (ADR-0031). The second and larger exception to
      # immutability, and the only one whose boundary is a heading rather than a field name —
      # so it is checked three ways rather than described once:
      #
      #   1. everything above the first `## Errata` heading is byte-identical
      #   2. the committed errata are a strict PREFIX of the new ones (append-only)
      #   3. the section grew — an "erratum edit" that adds nothing is a body edit
      #
      # Condition 1 is what rejects the smuggling case, and it does so without a special rule:
      # a heading inserted mid-file truncates the body under the same split, so the two bodies
      # differ and the edit fails. Condition 2 makes existing errata as immutable as the text.
      #
      # `awk`, not `sed`: the split must be on the FIRST occurrence and must treat a heading
      # inside a fenced code block no differently, because a rule that reads the file's
      # structure is a rule an author can construct their way around.
      split_at_errata() { awk 'f{next} /^## Errata[[:space:]]*$/{f=1; next} {print}'; }
      errata_only()     { awk 'f{print} /^## Errata[[:space:]]*$/{f=1}'; }
      prev="$(git show "$base:$f" 2>/dev/null)"
      cur="$(cat "$f")"
      prev_body="$(split_at_errata <<<"$prev")"; cur_body="$(split_at_errata <<<"$cur")"
      prev_er="$(errata_only <<<"$prev")";       cur_er="$(errata_only <<<"$cur")"
      if [[ "$prev_body" == "$cur_body" ]] \
         && [[ "$cur_er" == "$prev_er"* ]] \
         && [[ ${#cur_er} -gt ${#prev_er} ]]; then
        printf '    %s: errata appended, body byte-identical (ADR-0031)\n' "$(fm "$f" id)"
      else
        err "$f is LOCKED but has content changes — append an erratum (ADR-0031) if the text is factually wrong, or supersede it if the decision has changed (ADR-0002)"
      fi
    fi
  done < <(git diff --name-only "$base" 2>/dev/null)
fi

# --- 3b. supersession symmetry (knowledge.supersession_is_symmetric) ---------
# Nothing verified that the two halves of a supersession agreed. It is correct today
# only because both halves were written by hand. If the second is ever skipped, the
# old and new artifacts both read as in force — on precisely the question where they
# disagree — and the gate stays green. ADR-0016's own acceptance had this shape.
for f in "${FILES[@]}"; do
  aid="$(fm "$f" id)"
  [[ -n "$aid" ]] || continue
  sups="$(fm "$f" supersedes | tr -d '[]," ')"
  if [[ -n "$sups" && "$sups" != "null" ]]; then
    for old in $(tr ',' ' ' <<<"$sups"); do
      of="${SEEN_ID[$old]:-}"
      if [[ -z "$of" ]]; then
        err "$aid supersedes $old, which does not exist"
      elif [[ "$(fm "$of" superseded_by)" != "$aid" ]]; then
        err "$aid supersedes $old, but $old has superseded_by '$(fm "$of" superseded_by)' — the second half of the supersession edit is missing (ADR-0015)"
      fi
    done
  fi
  sb="$(fm "$f" superseded_by)"
  if [[ -n "$sb" && "$sb" != "null" ]]; then
    nf="${SEEN_ID[$sb]:-}"
    if [[ -z "$nf" ]]; then
      err "$aid claims superseded_by $sb, which does not exist"
    elif ! grep -q "$aid" <<<"$(fm "$nf" supersedes)"; then
      err "$aid claims superseded_by $sb, but $sb does not list it in supersedes (ADR-0015)"
    fi
  fi
done

# --- 3c. stale resolutions (knowledge.resolved_by_names_live_decision) -------
# An RFC's `resolved_by` is a pointer to the decision that closed it. When that decision is
# superseded, the pointer still resolves — so the cross-reference check is silent — and a
# reader following it lands on a rule that was deliberately replaced. RFC-0006 acquired
# exactly this shape the moment ADR-0024 superseded ADR-0023.
#
# NARROWED, after a broader version was written and run. The first attempt flagged any live
# artifact citing a superseded one in prose, and it was wrong: citing ADR-0002 for its
# argument is normal and correct practice here — README.md does it, C-0001 does it, and
# ADR-0014 preserved ADR-0002's body precisely so it could be cited. Twenty-odd false
# positives on the dominant legitimate usage. `resolved_by` is frontmatter, structured, and
# means one specific thing, so it is decidable where prose is not.
#
# The fix is not to repoint `resolved_by` — RFC-0006 genuinely WAS resolved by ADR-0023, and
# rewriting that erases the record. It is to name the successor somewhere in the file, so a
# reader following the pointer learns the decision moved.
stale_res=0
for f in "${FILES[@]}"; do
  st="$(fm "$f" status)"
  case "$st" in superseded|deprecated) continue ;; esac
  rb="$(fm "$f" resolved_by)"
  [[ -n "$rb" && "$rb" != "null" ]] || continue
  rf="${SEEN_ID[$rb]:-}"
  [[ -n "$rf" ]] || continue
  [[ "$(fm "$rf" status)" == "superseded" ]] || continue
  succ="$(fm "$rf" superseded_by)"
  [[ -n "$succ" && "$succ" != "null" ]] || continue
  grep -q "\\b$succ\\b" "$f" && continue
  wrn "$(fm "$f" id): resolved_by names $rb, which is superseded by $succ, and $succ is not mentioned in the file — a reader following the pointer lands on a replaced decision (ADR-0022)"
  stale_res=$((stale_res+1))
done
[[ $stale_res -eq 0 ]] && printf '%s✔%s every resolved_by points at a live decision, or names its successor\n' "$GRN" "$RST"

# --- 4. enforcement backlog (informational, never fails the build) ----------
#
# THIS SECTION USED TO MEASURE THE WRONG PROPERTY, and reported the result as the right one.
# It tested `enforced_by: []` only, so it answered "was a check NAMED" and printed
# "every locked artifact names an enforcing check" — while ten locked artifacts, including
# STD-0009 (tenant isolation, the platform's largest failure class), had every one of their
# checks owned by `tool-forge`, which does not exist. A name is not enforcement.
#
# Three numbers now, not one, because that is the distinction the old output erased:
#   locked            — decisions that must not be contradicted
#   with a live check — at least one named check has an owner that exists today
#   unenforced        — named checks only, none of them running
#
# Still informational. Turning it red today would fail the gate on ten artifacts at once and
# teach people to ignore it, which is how a control becomes decoration.
printf '\n'

# Check id -> owner, read from check-registry.yaml. An `owner: tool-forge` check is DECLARED.
declare -A CHECK_OWNER=()
while IFS='|' read -r _cid _cown; do
  [[ -n "$_cid" ]] && CHECK_OWNER["$_cid"]="$_cown"
done < <(awk '
  /^  - check: / { if (id != "") print id "|" own; id = substr($0, index($0, ": ") + 2); own = ""; next }
  /^    owner: / { own = substr($0, index($0, ": ") + 2) }
  END            { if (id != "") print id "|" own }
' check-registry.yaml)

backlog=0; locked_total=0; locked_running=0
UNENFORCED=()
for f in "${FILES[@]}"; do
  [[ "$(fm "$f" status)" == "locked" ]] || continue
  locked_total=$((locked_total+1))
  _aid="$(fm "$f" id)"
  if grep -qE '^enforced_by:\s*\[\]\s*$' "$f"; then
    wrn "$_aid is locked but names no check — advisory only"
    backlog=$((backlog+1)); UNENFORCED+=("$_aid"); continue
  fi
  _running=0
  while IFS= read -r _cid; do
    [[ -n "$_cid" ]] || continue
    _own="${CHECK_OWNER[$_cid]:-}"
    [[ -n "$_own" && "$_own" != "tool-forge" ]] && _running=1
  done < <(awk '
    /^enforced_by:/   { f = 1; next }
    f && /^---$/      { exit }
    f && /^[a-z_]+:/  { f = 0 }
    f && /^  - check: / { print substr($0, index($0, ": ") + 2) }
  ' "$f")
  if [[ $_running -eq 1 ]]; then locked_running=$((locked_running+1)); else UNENFORCED+=("$_aid"); fi
done
[[ $backlog -eq 0 ]] && printf '%s✔%s every locked artifact names an enforcing check\n' "$GRN" "$RST"
printf '%d locked artifact(s) — %d with a live check, %d enforced by nothing that runs today\n' \
  "$locked_total" "$locked_running" "${#UNENFORCED[@]}"
if [[ ${#UNENFORCED[@]} -gt 0 ]]; then
  printf '    unenforced: %s\n' "${UNENFORCED[*]}"
  printf '    a name in enforced_by is a commitment, not a control (ADR-0014). This is the backlog.\n'
fi

# --- 4a. knowledge.no_unresolved_check_parameter_conflict (ADR-0036) --------
# `repo.claude_md_under_line_budget` was cited by two LOCKED artifacts with different severity
# and different args, and check-repo.sh enforced a third threshold taken from ADR-0026's prose.
# Four sources, three thresholds, two severities. Every file was correct on its own; nothing
# ever compared them, so it survived until an audit read all four in one sitting.
#
# check-registry.yaml's header states the model that allowed it -- "severity lives in each
# citing artifact's enforced_by" -- which is right, and silently assumes one citer.
CONFLICT=0
declare -A RESOLVED=()
while IFS= read -r _rid; do [[ -n "$_rid" ]] && RESOLVED["$_rid"]=1; done < <(awk '
  /^  - check: /    { id = substr($0, index($0, ": ") + 2) }
  /^    resolution:/ { if (id != "") print id }
' check-registry.yaml)

declare -A CITED=()
for f in "${FILES[@]}"; do
  # A superseded or deprecated artifact's enforced_by is a historical record, not a live
  # citation. Including them found ADR-0002 -- superseded by ADR-0014 -- "disagreeing" with its
  # own successor about a check the successor redefined, which is what supersession IS.
  case "$(fm "$f" status)" in superseded|deprecated) continue ;; esac
  aid="$(fm "$f" id)"
  while IFS='|' read -r _c _s _a; do
    [[ -n "$_c" ]] || continue
    CITED["$_c"]="${CITED[$_c]:-}${CITED[$_c]:+$'\n'}${_s}|${_a}|${aid}"
  done < <(awk '
    /^enforced_by:/     { f = 1; next }
    f && /^---$/        { exit }
    f && /^[a-z_]+:/    { f = 0 }
    f && /^  - check: / { if (id != "") print id "|" sev "|" args; id = substr($0, index($0, ": ") + 2); sev = ""; args = ""; next }
    f && /^    severity: / { sev  = substr($0, index($0, ": ") + 2) }
    f && /^    args: /     { args = substr($0, index($0, ": ") + 2) }
    END                 { if (id != "") print id "|" sev "|" args }
  ' "$f")
done

for _c in "${!CITED[@]}"; do
  # Severity is compared whenever declared. Args are compared only between citations that BOTH
  # declare them: an absent `args` is an unspecified parameter, not a competing one, so ADR-0026
  # naming no threshold does not conflict with ADR-0007 naming 200 -- it defers to it. What made
  # that pair a real conflict was the severity, and a number in ADR-0026's prose that no
  # `enforced_by` ever declared (ADR-0036 decision 1).
  _nsev="$(printf '%s\n' "${CITED[$_c]}" | cut -d'|' -f1 | grep . | sort -u | grep -c . || true)"
  _narg="$(printf '%s\n' "${CITED[$_c]}" | cut -d'|' -f2 | grep . | sort -u | grep -c . || true)"
  [[ "$_nsev" -gt 1 || "$_narg" -gt 1 ]] || continue
  [[ -n "${RESOLVED[$_c]:-}" ]] && continue
  _who="$(printf '%s\n' "${CITED[$_c]}" | awk -F'|' '{printf "%s(%s%s) ", $3, $1, ($2 == "" ? "" : ", " $2)}')"
  err "$_c is cited with conflicting severity or args and check-registry.yaml records no resolution: ${_who}— the implementation would otherwise pick a third value nobody declared (ADR-0036)"
  CONFLICT=$((CONFLICT+1))
done
[[ $CONFLICT -eq 0 ]] && printf '%s✔%s no check is cited with conflicting parameters and no resolution\n' "$GRN" "$RST"

# --- knowledge.sources_are_reviewed + knowledge.external_url_is_a_declared_source
# RFC-0020. ADR-0035 says a count in prose is a snapshot rather than a claim; an external fact
# is a snapshot too, and a worse one — a registry count can be re-derived on demand, while a
# regulatory position or a vendor's documented limit changes elsewhere, silently, with nothing
# here to notice. The `sources` block dates the observation; these two checks keep it honest.
#
# The completeness half is NOT here. `sources` entries require url, title and retrieved in
# knowledge.schema.json, so knowledge.frontmatter_valid already rejects an undated source and
# duplicating it would be a second place to keep in step.
# THE URL EXTRACTOR IS A FUNCTION BECAUSE IT HAS TWO CALLERS. This section asks "is this body
# url declared in frontmatter"; the RFC-0027 section below asks "is this FETCHED url declared in
# any frontmatter". Same question, same normalisation — a trailing full stop or bracket that one
# stripped and the other did not would make a source that is plainly cited report as missing.
# ADR-0022 is the standing argument against the second copy; this is the same defect class in a
# regex rather than a directory list.
urls_in() { grep -oE 'https?://[^ )>"'"'"']+' | sed 's/[.,;]$//' | sort -u; }

SRC_MAX_AGE_DAYS="${PARIDYUM_SOURCE_MAX_AGE_DAYS:-365}"
now_epoch="$(date -u +%s)"
src_stale=0 src_undeclared=0
for f in "${FILES[@]}"; do
  fm="$(awk 'NR==1 && $0!="---"{exit} NR>1 && $0=="---"{exit} {print}' "$f")"
  # every retrieval date in the frontmatter, aged against the budget
  while read -r d; do
    [[ -z "$d" ]] && continue
    if command -v gdate >/dev/null 2>&1; then de="$(gdate -u -d "$d" +%s 2>/dev/null || echo 0)"
    else de="$(date -u -j -f %Y-%m-%d "$d" +%s 2>/dev/null || echo 0)"; fi
    [[ "$de" == 0 ]] && continue
    age=$(( (now_epoch - de) / 86400 ))
    if (( age > SRC_MAX_AGE_DAYS )); then
      wrn "$f: a source was retrieved ${age}d ago (budget ${SRC_MAX_AGE_DAYS}d) — re-read it before relying on the decision it supports (RFC-0020)"
      src_stale=$((src_stale+1))
    fi
  done < <(printf '%s\n' "$fm" | sed -nE 's/^[[:space:]]*retrieved:[[:space:]]*([0-9]{4}-[0-9]{2}-[0-9]{2}).*/\1/p')

  # a bare URL in the BODY that the frontmatter does not declare. Code spans and fenced blocks
  # are excluded: RFC-0012 carries `https://${CUSTOM_DOMAIN}/scim/v2/{orgId}` as a URL TEMPLATE
  # in an example, which is not a citation and must not be reported as one.
  body="$(awk 'NR==1 && $0=="---"{s=1; next} s==1 && $0=="---"{s=2; next} s!=1{print}' "$f" \
        | sed -E '/^[[:space:]]*```/,/^[[:space:]]*```/d' | sed -E 's/`[^`]*`//g')"
  while read -r u; do
    [[ -z "$u" ]] && continue
    printf '%s' "$fm" | grep -qF "$u" && continue
    wrn "$f: body cites $u but frontmatter declares no such source — an external claim with no date is the thing RFC-0020 exists to prevent"
    src_undeclared=$((src_undeclared+1))
  done < <(printf '%s\n' "$body" | urls_in)
done

# --- RFC-0027: the research log, and what it reconciles ---------------------
# RFC-0020 made an external fact carry a url and a date. It did not — could not — notice a fact
# that was read and never written down at all, and it says so: its checks "catch the mechanical
# half only". log-web-sources.sh records every web fetch as it happens, capture-open-loops.sh
# appends a session record reconciling those fetches against the `sources` blocks the session
# produced, and these three checks read what both wrote.
#
# The log is DATA, not an artifact: no frontmatter, no id, no entry in artifact-registry.yaml.
# It is read by gates the same way check-registry.yaml is, and it must never appear in FILES.
RESEARCH_DIR="research"
if [[ -d "$RESEARCH_DIR" ]]; then
  RLOGS=(); while IFS= read -r _l; do RLOGS+=("$_l"); done < <(find "$RESEARCH_DIR" -name '*.jsonl' -type f 2>/dev/null | sort)
else
  RLOGS=()
fi

# THE GIT HALF RUNS FIRST AND UNCONDITIONALLY, because it is the only one whose subject can be
# made empty BY THE DEFECT IT DETECTS. An earlier version nested this inside the "logs exist"
# branch, so deleting the entire research/ directory removed the check along with the evidence
# and the gate reported success — history erased, green run. Found 2026-08-19 by deleting a
# leaked fixture and watching the gate not care. Removing a log is exactly the act this check
# exists to catch, and it cannot be conditional on the log surviving.
ro_bad=0
if git rev-parse --git-dir >/dev/null 2>&1; then
  # Only removals and rewrites. An added line is the ONLY legal edit, so `+` is ignored
  # entirely and `-` is the whole signal.
  while IFS= read -r _d; do
    [[ "$_d" =~ ^---  ]] && continue
    [[ "$_d" =~ ^-    ]] || continue
    err "$RESEARCH_DIR: a line was removed or rewritten — the log records what was READ, and a record a later run can edit is not evidence (knowledge.research_log_is_append_only, RFC-0027)"
    ro_bad=$((ro_bad+1))
    break
  done < <(git diff HEAD -- "$RESEARCH_DIR" 2>/dev/null)
fi

if [[ ${#RLOGS[@]} -eq 0 ]]; then
  # Armed, not passing. The same sentence knowledge.market_artifacts_cite_sources prints while
  # no MKT- exists, for the same reason: a check whose subject set is empty has proven nothing,
  # and saying "passed" here is the ADR-0014 defect class this repository keeps re-finding.
  [[ $ro_bad -eq 0 ]] && printf '%s✔%s no research log exists yet — the RFC-0027 checks are armed, not passing on evidence\n' "$GRN" "$RST"
else
  # --- knowledge.research_log_is_append_only --------------------------------
  # ERRORS, alone among the three. The other two report on what a human did or did not write
  # down, and RFC-0020 is right that failing a build for that gets suppressed. This reports on
  # whether the record itself is trustworthy — and a reconciliation check reading a log the same
  # run could have rewritten is enforcing nothing, which is the failure ADR-0014 exists to name.
  for _l in "${RLOGS[@]}"; do
    _n=0
    while IFS= read -r _line; do
      _n=$((_n+1))
      [[ -z "$_line" ]] && continue
      if ! jq -e 'type == "object"' >/dev/null 2>&1 <<<"$_line"; then
        err "$_l:$_n is not a JSON object — the research log is machine-read evidence, and one malformed line makes every reconciliation past it silently incomplete (RFC-0027)"
        ro_bad=$((ro_bad+1))
      fi
    done < "$_l"
  done
  [[ $ro_bad -eq 0 ]] && printf '%s✔%s %d research log(s) are append-only and parseable\n' "$GRN" "$RST" "${#RLOGS[@]}"

  # --- knowledge.fetched_sources_are_reconciled ------------------------------
  # Reads the `kind:session` records rather than recomputing the reconciliation. The Stop hook
  # knows which session it was and what that session touched; the gate runs later and does not.
  # Recomputing here would produce a second, worse answer — every fetch in the repository's
  # history compared against today's frontmatter, which reports a page cited in a superseded
  # artifact as uncited forever.
  #
  # WARNS. Most fetches are dead ends. A gate that fails a session for reading a page it decided
  # not to use teaches people to stop reading pages, and RFC-0020 makes this argument in full.
  rec_unc=0 rec_sess=0
  while IFS=$'\t' read -r _s _u; do
    [[ -z "$_u" ]] && continue
    wrn "research: session $_s read $_u and no artifact declares it in \`sources\` — an external fact entered this repository with no provenance (RFC-0020, RFC-0027)"
    rec_unc=$((rec_unc+1))
  done < <(jq -r 'select(.kind == "session") | . as $r | .uncited[]? | [$r.session, .] | @tsv' "${RLOGS[@]}" 2>/dev/null)
  rec_sess="$(jq -rs '[.[] | select(.kind == "session")] | length' "${RLOGS[@]}" 2>/dev/null || echo 0)"
  if [[ $rec_unc -eq 0 ]]; then
    printf '%s✔%s every page fetched across %s reconciled session(s) is cited in some `sources` block\n' "$GRN" "$RST" "${rec_sess:-0}"
  fi

  # --- knowledge.researched_artifacts_cite_sources ---------------------------
  # RFC-0020 open question 2, answerable at last. `when_researched` obliges an artifact to cite
  # sources only when the log shows the session that wrote it actually read something — the
  # middle setting between `always` (empty blocks on artifacts resting on nothing external) and
  # `never` (what let RFC-0014 and RFC-0019 assert a vendor limit with no provenance).
  #
  # WHICH TYPES SET IT IS REGISTRY DATA and today none do. Deliberate: RFC-0027 exists so that
  # question gets answered by counting a month of logs, not by guessing now.
  declare -A WR_SOURCES=()
  for _t in "${!T_SOURCES[@]}"; do
    [[ "${T_SOURCES[$_t]}" == "when_researched" ]] && WR_SOURCES[$_t]=1
  done
  if [[ ${#WR_SOURCES[@]} -eq 0 ]]; then
    printf '%s✔%s no type sets requires_sources: when_researched — armed, and the log is still counting which should (RFC-0027)\n' "$GRN" "$RST"
  else
    wr_bad=0
    while IFS= read -r _a; do
      [[ -n "$_a" && -f "$_a" ]] || continue
      _ty="$(fm "$_a" type)"
      [[ -n "$_ty" && -n "${WR_SOURCES[$_ty]:-}" ]] || continue
      awk 'NR==1 && $0!="---"{exit} NR>1 && $0=="---"{exit} {print}' "$_a" | grep -q '^sources:' && continue
      err "$_a: type '$_ty' sets requires_sources: when_researched and was written in a session that fetched a page, but declares no \`sources\` — the fact it rests on is in a log rather than in the artifact (RFC-0027)"
      wr_bad=$((wr_bad+1))
    done < <(jq -r 'select(.kind == "session" and .fetched > 0) | .artifacts[]?' "${RLOGS[@]}" 2>/dev/null | sort -u)
    [[ $wr_bad -eq 0 ]] && printf '%s✔%s every researched artifact of an obliged type cites its sources\n' "$GRN" "$RST"
  fi
fi

# --- knowledge.market_artifacts_cite_sources (ADR-0050) ---------------------
# The rule that makes MKT- an evidence type rather than a bet with a different prefix, and it
# lives here rather than in knowledge.schema.json because the schema cannot carry it:
# scripts/lib/validate-schema.py implements neither `if`/`then` nor `allOf` and reports unknown
# keywords as unenforced, so a conditional `required` there would advertise a constraint nothing
# applies. That is not hypothetical — artifact-registry.yaml's note on `market` records a
# sourceless MKT- being planted on 2026-08-17 against a schema everybody believed required one,
# and validating clean.
#
# WHICH TYPES ARE OBLIGED IS REGISTRY DATA, not the literal string `market` written here. The
# registry declares `requires_sources` beside `requires_trigger`, one axis at its own setting,
# and today exactly one type sets it to `always`. Hardcoding the type name would be the
# fifteen-sites problem ADR-0022 exists to end, and would make ADR-0050's own claim — that the
# grammar is constitutional and the list is data — false in the gate enforcing it.
#
# WHAT THIS DOES NOT COVER, twice over. It cannot tell whether a source is any good, or whether
# it is tenant-derived: C-0006 forbids aggregate operational data selecting where Paridyum
# competes, and no check reads a URL and knows whose data is behind it. C-0006 already concedes
# that commitment is "partly mechanical and partly not"; this is only the mechanical part.
# And the per-entry url/title/retrieved requirement OVERLAPS knowledge.frontmatter_valid, which
# rejects an incomplete `sources` entry on every type. It is repeated because the spec ADR-0050
# declared names it, and because the presence half — a market artifact with no `sources` key at
# all — is the half no schema here can express, so the two halves would otherwise be enforced by
# two mechanisms with only one of them named in the decision.
declare -A REQ_SOURCES=()
for _t in "${!T_SOURCES[@]}"; do
  [[ "${T_SOURCES[$_t]}" == "always" ]] && REQ_SOURCES[$_t]=1
done
if [[ ${#REQ_SOURCES[@]} -eq 0 ]]; then
  # A gate whose subject set is empty because the field it reads went missing reports success
  # and enforces nothing, which is the ADR-0014 defect class inside the check written to close
  # it. Say so rather than pass quietly.
  wrn "$ARTREG declares no type with requires_sources: always — knowledge.market_artifacts_cite_sources has no subject and enforces nothing (ADR-0050)"
else
  SOURCED_FILES=()
  for f in "${FILES[@]}"; do
    # Never an unvalidated value as an associative-array subscript — an empty one is a bash
    # error that leaves the array unusable, the same trap section 1 records twice.
    _ty="$(fm "$f" type)"
    [[ -n "$_ty" && -n "${REQ_SOURCES[$_ty]:-}" ]] && SOURCED_FILES+=("$f")
  done
  if [[ ${#SOURCED_FILES[@]} -eq 0 ]]; then
    # Vacuous, and it must stay implemented rather than deferred so it is armed the moment the
    # first MKT- appears — the same bargain section 7c makes while only one manifest exists.
    printf '%s✔%s no artifact of a type requiring sources exists yet — the check is armed, not passing on evidence\n' "$GRN" "$RST"
  elif ! command -v python3 >/dev/null; then
    wrn "python3 unavailable — ${#SOURCED_FILES[@]} artifact(s) obliged to cite sources are unverified"
  else
    mktout="$(python3 - "${SOURCED_FILES[@]}" <<'PY'
import re, sys

try:
    import yaml
except ImportError:
    sys.exit(2)

FM = re.compile(r"\A---\n(.*?)\n---\n", re.S)
checked = 0
bad = set()


def reject(aid, msg):
    bad.add(aid)
    print(f"ERR|{aid}: {msg}")


for path in sys.argv[1:]:
    with open(path, encoding="utf-8") as fh:
        m = FM.match(fh.read())
    if not m:
        continue          # section 1 owns a missing frontmatter block
    try:
        fm = yaml.safe_load(m.group(1)) or {}
    except yaml.YAMLError:
        continue          # section 1 owns malformed YAML; do not report it twice
    checked += 1
    aid = fm.get("id") or path
    ty = fm.get("type")
    srcs = fm.get("sources")
    if not srcs:
        print(f"ERR|{aid}: type '{ty}' carries no `sources` — a market finding with no cited "
              f"evidence is an opinion with frontmatter, and C-0006 rests on being able to see "
              f"that what selected a vertical was not tenant-derived (ADR-0050)")
        continue
    if not isinstance(srcs, list):
        print(f"ERR|{aid}: `sources` is not a list of entries")
        continue
    for i, s in enumerate(srcs):
        if not isinstance(s, dict):
            print(f"ERR|{aid}: sources[{i}] is not a mapping")
            continue
        missing = [k for k in ("url", "title", "retrieved") if not s.get(k)]
        if missing:
            print(f"ERR|{aid}: sources[{i}] lacks {', '.join(missing)} — an undated or "
                  f"unattributed source is a claim with no provenance, which is what "
                  f"`sources` exists to stop (RFC-0020, ADR-0050)")
print(f"OK|{checked} artifact(s) of a type requiring sources cite them completely")
PY
    )"
    case $? in
      0)
        while IFS='|' read -r kind msg; do
          [[ -n "$msg" ]] || continue
          case "$kind" in
            ERR) err "$msg" ;;
            OK)  printf '%s✔%s %s\n' "$GRN" "$RST" "$msg" ;;
          esac
        done <<<"$mktout"
        ;;
      2) wrn "PyYAML unavailable — ${#SOURCED_FILES[@]} artifact(s) obliged to cite sources are unverified" ;;
      *) err "knowledge.market_artifacts_cite_sources failed to run" ;;
    esac
  fi
fi

# --- 4b. knowledge.no_hardcoded_registry_counts (ADR-0035) ------------------
# Seven instances of a stale hand-maintained count were found by an audit and none by a gate.
# The eighth froze into ADR-0022 on the day ADR-0022 was locked — the artifact whose whole
# argument is that a list which must be extended should not be hardcoded.
#
# ADR-0035 does NOT ask for these to be corrected; the registry wins and the artifact is old,
# not wrong. This produces the REVIEW LIST that decision 6 needs: a count used as evidence has
# to be re-derived before the conclusion resting on it is relied upon.
#
# Aggregated deliberately. There are ~70 hits, and printing 70 warnings on every run is how a
# control becomes decoration — the exact failure ADR-0035's own rationale names. Count here,
# list behind --counts.
CNT_NUM='([0-9]+|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety|hundred)(-(one|two|three|four|five|six|seven|eight|nine))?'
CNT_NOUN='(checks|check ids|namespaces|artifacts|artifact types|test types|locked artifacts|repository classes|classes|axes|tiers)'
cnt_hits="$(grep -rEIn --include='*.md' --include='*.yaml' \
    --exclude-dir=.git --exclude-dir=tests \
    "[[:space:]]$CNT_NUM +$CNT_NOUN\b" . 2>/dev/null \
  | grep -vE '^\./(adr/ADR-0035|CHANGELOG\.md)' || true)"
# CHANGELOG.md is excluded and ADR-0035 §1 is the reason rather than convenience: a changelog
# entry is a dated record of what was true at release, which is precisely the "snapshot" the
# decision says is legitimate. It is also append-only and unbounded, so leaving it in would let
# it dominate the count within a few releases and turn the signal into noise. ADR-0035 itself is
# excluded because it quotes the stale numbers as its own evidence.
cnt_n="$(printf '%s' "$cnt_hits" | grep -c . || true)"
cnt_files="$(printf '%s' "$cnt_hits" | cut -d: -f1 | sort -u | grep -c . || true)"
if [[ "$cnt_n" -gt 0 ]]; then
  wrn "$cnt_n numeric claim(s) about registry-computed things, across $cnt_files file(s) — the registry wins (ADR-0035). Not defects; a review list for counts used as evidence. List them with --counts"
  if [[ "${1:-}" == "--counts" || "${PARIDYUM_SHOW_COUNTS:-}" == "1" ]]; then
    printf '%s\n' "$cnt_hits" | sed 's/^/    /'
  fi
fi

# --- knowledge.hardcoded_count_baseline_does_not_rise (ADR-0035) ------------
# The warning above ALWAYS fires, so it cannot see a regression. On 2026-08-10 two new counts
# were added to README.md and TESTING.md — the files ADR-0035's own context credits with having
# stopped stating them — and the number moved 48 -> 50 with nothing to notice. One of them then
# contradicted a sentence two lines below it.
#
# This judges arithmetic, not meaning. ADR-0035 rejected an error-severity check here because
# "six anti-patterns" is a count and "two isolation boundaries" is a structural fact and no
# regex separates them — that objection is about classifying a LINE, and still stands. Whether
# the total rose is a different question, and it is decidable.
#
# The baseline is read from a citing artifact's args (ADR-0036), never from prose here.
cnt_baseline="$(awk '
  /^enforced_by:/                                              { f = 1; next }
  f && /^[a-z_]+:/                                             { f = 0 }
  f && /- check: knowledge.hardcoded_count_baseline_does_not_rise/ { c = 1; next }
  c && /- check:/                                              { c = 0 }
  c && /baseline:/ {
    line = $0
    if (match(line, /baseline:[[:space:]]*[0-9]+/)) {
      v = substr(line, RSTART, RLENGTH); sub(/baseline:[[:space:]]*/, "", v); print v; exit
    }
  }
' adr/ADR-0035-*.md 2>/dev/null)"
if [[ -z "$cnt_baseline" ]]; then
  # ABSENT IS NOT PASS. No baseline means the parameter was not declared, and a check that
  # silently skips is indistinguishable from one that passed.
  err "knowledge.hardcoded_count_baseline_does_not_rise: no baseline declared in ADR-0035's enforced_by args (ADR-0036) — the check cannot run"
elif [[ "$cnt_n" -gt "$cnt_baseline" ]]; then
  err "hardcoded registry counts rose to $cnt_n, above the declared baseline of $cnt_baseline (ADR-0035 decision 5) — a NEW count was added. Cite the registry or the gate, or write the shape without its cardinality. List them with --counts"
else
  printf '%s✔%s hardcoded registry counts at %s, not above the baseline of %s\n' "$GRN" "$RST" "$cnt_n" "$cnt_baseline"
fi

# --- 5. declared-checks manifest (knowledge.declared_checks_manifest_valid) --
# check-registry.yaml is the id-space ADR-0014 requires.
#
# This comment used to say full schema validation "needs a real JSON Schema validator and
# lands with tool-forge". That stopped being true when scripts/lib/validate-schema.py was
# written for the artifact and class registries, and nobody noticed: check-registry.schema.json
# is 130 lines describing minLength, patterns and additionalProperties, and until 2026-08-08 it
# had NEVER been executed against the file it describes. class-registry.yaml has been validated
# against its schema this whole time, three sections further down, using the same helper.
#
# It is the ADR-0014 failure with the roles reversed — not a check declared and unimplemented,
# but a check implemented and never called — and it is worse, because the schema's existence
# was the reason nobody hand-wrote the constraints it contained.
#
# The bash checks below stay. They are not redundant: uniqueness, sortedness and the
# "Fails when " discipline are not expressible in JSON Schema, and they must survive a machine
# with no python3 or no PyYAML, where the schema pass degrades to a warning.
printf '\n'
MANIFEST="check-registry.yaml"
declare -A DECLARED=() DECL_BLOCKS=() DECL_OWNER=()

if [[ -f scripts/lib/validate-schema.py && -f check-registry.schema.json && -f "$MANIFEST" ]]; then
  mout="$(python3 scripts/lib/validate-schema.py document check-registry.schema.json "$MANIFEST" 2>&1)"
  case $? in
    0) ;;
    1) while IFS= read -r line; do [[ -n "$line" ]] && err "${line#✘ }"; done < <(grep '^✘' <<<"$mout") ;;
    *) wrn "$MANIFEST: schema validation unavailable — shape unverified, only the checks below ran" ;;
  esac
fi

manifest_entries() {
  awk '
    function flush() {
      if (id != "")
        printf "%s\t%s\t%s\t%s\t%s\n", id,
          (spec == "" ? "-" : spec), (owner == "" ? "-" : owner),
          (blocks == "" ? "-" : blocks), (declared == "" ? "-" : declared)
    }
    /^  - check: / { flush(); id = $3; spec = ""; owner = ""; blocks = ""; declared = ""; f = ""; next }
    /^    spec:/     { f = "spec"; next }
    /^    owner: /    { owner = substr($0, index($0, ": ") + 2);    f = ""; next }
    /^    blocks: /   { blocks = substr($0, index($0, ": ") + 2);   f = ""; next }
    /^    declared: / { declared = substr($0, index($0, ": ") + 2); f = ""; next }
    /^    [a-z_]+:/  { f = ""; next }
    f == "spec" && /^      / { if (spec == "") { s = $0; sub(/^[[:space:]]+/, "", s); spec = s } next }
    END { flush() }
  ' "$1"
}

# `- check:` also appears in skill templates and in prose, so ids are read only
# from inside an artifact's frontmatter block.
frontmatter_check_ids() {
  awk '
    NR == 1 && $0 != "---" { exit }
    NR == 1 { infm = 1; next }
    infm && $0 == "---" { exit }
    infm && /^[[:space:]]+- check:[[:space:]]/ { sub(/^[[:space:]]+- check:[[:space:]]*/, ""); print }
  ' "$1"
}

norm() { tr -d '[],' <<<"$1" | tr ' ' '\n' | grep -v '^$' | sort -u | tr '\n' ' '; }

if [[ ! -f "$MANIFEST" ]]; then
  err "$MANIFEST is missing — every enforced_by id resolves against it (ADR-0014)"
else
  while IFS=$'\t' read -r id spec owner blocks declared; do
    [[ -n "$id" ]] || continue
    if [[ ! "$id" =~ ^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*)+$ ]]; then
      err "$MANIFEST: check id '$id' does not match <namespace>.<name>"
      continue
    fi
    if [[ -n "${DECLARED[$id]:-}" ]]; then
      err "$MANIFEST: duplicate entry for '$id' — the second silently shadows the first"
      continue
    fi
    DECLARED[$id]=1
    [[ "$spec"     != "-" ]] || err "$MANIFEST: $id has no spec"
    [[ "$owner"    != "-" ]] || err "$MANIFEST: $id has no owner"
    [[ "$declared" != "-" ]] || err "$MANIFEST: $id has no declared date"
    [[ "$blocks"   != "-" ]] || err "$MANIFEST: $id names no blocked artifact"
    DECL_OWNER[$id]="$owner"
    DECL_BLOCKS[$id]="$(norm "$blocks")"
    # The manifest header claims "Sorted by id" and nothing verified it, so successive
    # insertions had quietly left five entries out of order. An unsorted registry is not
    # a correctness bug — it makes diffs noisy and near-duplicate ids easy to miss when
    # scanning — but a claim in a header that nothing checks is exactly the pattern this
    # repository keeps getting caught by.
    if [[ -n "${LAST_ID:-}" && "$id" < "$LAST_ID" ]]; then
      err "$MANIFEST: '$id' appears after '$LAST_ID' — entries must be sorted by id"
    fi
    LAST_ID="$id"
    if [[ "$spec" != "-" && "$spec" != "Fails when "* ]]; then
      err "$MANIFEST: $id spec must begin 'Fails when ' — a rule that cannot be stated as a rejection condition is a preference, not a check (ADR-0014)"
    fi
  done < <(manifest_entries "$MANIFEST")
fi

# --- 6. check id resolution (knowledge.check_ids_resolve) -------------------
# ADR-0014: an id in neither the manifest nor tool-forge's registry is a hard
# failure; an id in the manifest but unimplemented is `declared` and is reported,
# not fatal. The manifest is still the entire resolvable set — forge implements a
# subset of the ids declared here and invents none — which is precisely why an
# unresolved id here is a typo.
#
# `owner` decides implemented versus awaiting below: `forge` is the shipped binary
# that runs the check today, `tool-forge` is the repository that still owes it. The
# two read alike and mean opposite things, which is why the schema says so too.
declare -A CITED_BY=()
if [[ ${#DECLARED[@]} -gt 0 ]]; then
  unresolved=0
  for f in "${FILES[@]}"; do
    aid="$(fm "$f" id)"
    while IFS= read -r cid; do
      [[ -n "$cid" ]] || continue
      if [[ -z "${DECLARED[$cid]:-}" ]]; then
        err "$f: enforced_by names '$cid', absent from $MANIFEST — unresolved id, so nothing enforces it (ADR-0014)"
        unresolved=$((unresolved+1))
      fi
      CITED_BY[$cid]="${CITED_BY[$cid]:-} $aid"
    done < <(frontmatter_check_ids "$f")
  done
  [[ $unresolved -eq 0 ]] && printf '%s✔%s every enforced_by check id resolves against %s\n' "$GRN" "$RST" "$MANIFEST"
fi

# --- 7. orphan declarations (knowledge.orphan_check_declarations) -----------
# A declaration nothing cites is a commitment to enforce nothing, and a `blocks`
# list that disagrees with reality stops being a usable priority ordering.
# An artifact that is `locked` or `superseded` CANNOT cite a check: adding an `enforced_by`
# entry is a content change, which the immutability rule forbids. But ADR-0014's whole model is
# that the gate hardens over time — a check written for an already-locked decision is the NORMAL
# path, not an exception. Comparing `blocks` to citers alone therefore produced a permanent
# warning for every such check, which is how a gate teaches people to ignore it.
#
# `blocks` means "artifact ids whose enforcement waits on this check", which is satisfied by a
# locked artifact naming nothing. So an entry is accounted for when it either cites the check or
# is immutable. Surfaced by writing repo.entrypoint_does_not_restate_standards for the
# already-locked ADR-0026 and watching this warn with no legal way to silence it.
declare -A ART_STATUS=()
for f in "${FILES[@]}"; do
  aid="$(fm "$f" id)"; [[ -n "$aid" ]] && ART_STATUS[$aid]="$(fm "$f" status)"
done
immutable_artifact() {
  case "${ART_STATUS[$1]:-}" in locked|superseded) return 0 ;; *) return 1 ;; esac
}

# --- knowledge.check_specs_cite_live_artifacts -------------------------------
# A spec is the falsifiable rule an implementer reads BEFORE writing the check, so a spec
# naming a superseded artifact sends them to a rule that was deliberately replaced.
# `notes` is deliberately NOT scanned: notes cite history on purpose, and 22 of the 27
# references in the first pass were notes recording why a rule changed. Scanning them would
# have made this check fire on the registry working correctly.
# The instance that motivated it: repo.name_matches_scheme cited C-0002's prefix table, which
# holds 7 of the 10 classes — an implementation obeying the spec would reject every web-,
# sdk- and mobile- repository.
spec_stale=0
while IFS=$'\t' read -r cid ref; do
  [[ -n "$cid" ]] || continue
  case "${ART_STATUS[$ref]:-}" in
    superseded|deprecated)
      err "$MANIFEST: $cid spec cites $ref, which is ${ART_STATUS[$ref]} — a spec is the rule an implementer reads, so it must name the live artifact (ADR-0014)"
      spec_stale=$((spec_stale+1))
      ;;
  esac
done < <(awk '
  /^  - check: /   { cid = $3; inspec = 0; next }
  /^    spec: >/   { inspec = 1; next }
  /^    [a-z_]+:/  { inspec = 0 }
  inspec {
    line = $0
    while (match(line, /(ADR|ANTI|BET|C|GLO|PAT|REQ|RFC|RUN|STD)-[0-9][0-9][0-9][0-9]/)) {
      print cid "\t" substr(line, RSTART, RLENGTH)
      line = substr(line, RSTART + RLENGTH)
    }
  }
' "$MANIFEST" | sort -u)
[[ "$spec_stale" -gt 0 ]] \
  || printf '%s✔%s every check spec names a live artifact\n' "$GRN" "$RST"
for id in $(printf '%s\n' "${!DECLARED[@]}" | sort); do
  cited="$(norm "${CITED_BY[$id]:-}")"
  blocks="${DECL_BLOCKS[$id]:-}"
  if [[ -z "$cited" ]]; then
    # Every blocked artifact immutable => the check is attributable and unciteable, not orphaned.
    all_immutable=true
    for b in $blocks; do immutable_artifact "$b" || all_immutable=false; done
    if [[ -n "${blocks// /}" && "$all_immutable" == true ]]; then
      printf '    %s: declared for immutable [%s] — enforced, not citeable\n' "$id" "${blocks% }"
    else
      wrn "$id is declared in $MANIFEST but no artifact cites it"
    fi
  elif [[ "$cited" != "$blocks" ]]; then
    unaccounted=""
    for b in $blocks; do
      [[ " $cited " == *" $b "* ]] && continue
      immutable_artifact "$b" && continue
      unaccounted+="$b "
    done
    for c in $cited; do
      [[ " $blocks " == *" $c "* ]] || unaccounted+="$c "
    done
    [[ -z "${unaccounted// /}" ]] \
      || wrn "$id declares blocks [${blocks% }] but is cited by [$cited] — unaccounted: ${unaccounted% }"
  fi
done

# --- 7b. manifest / grant agreement (domain.grants_match_declared_ownership) --
# manifest.yaml `owns.tables` is a declaration; db/grants.sql is the enforcement
# (ANTI-0001). If they drift, the declaration is fiction and the grants are the truth —
# so the drift itself is the thing worth failing on, in either direction.
#
# Deliberately a text comparison rather than SQL parsing. A real parser would be more
# precise and is tool-forge's job; this catches the case that actually happens, which is
# adding a table to one file and forgetting the other.
while IFS= read -r mf; do
  # docs.manifest_schema_valid — real validation against manifest.schema.json, same
  # validator as the frontmatter. The bash checks below are retained as the fallback
  # for when it cannot run, not as the primary.
  if [[ -f scripts/lib/validate-schema.py && -f manifest.schema.json ]]; then
    mout="$(python3 scripts/lib/validate-schema.py document manifest.schema.json "$mf" 2>&1)"
    case $? in
      0) ;;
      1) while IFS= read -r line; do [[ -n "$line" ]] && err "${line#✘ }"; done \
           < <(grep '^✘' <<<"$mout") ;;
      *) wrn "$mf: schema validation unavailable — field checks only" ;;
    esac
  fi
  # Fallback / belt-and-braces: required fields, closed enums, and the one cross-field
  # invariant the schema cannot express (exposes.events must be a subset of owns.events).
  for field in name class owner lifecycle; do
    grep -qE "^${field}:[[:space:]]*[^[:space:]]" "$mf" \
      || err "$mf: missing required field '$field' (STD-0002)"
  done
  cls="$(awk -F': *' '/^class:/{sub(/ *#.*/,"",$2);print $2;exit}' "$mf")"
  case "$cls" in
    infra|library|mobile|platform|sdk|service|spec|template|tool|web|"") ;;
    *) err "$mf: class '$cls' is not one of service|platform|library|tool|infra|spec|template" ;;
  esac
  lc="$(awk -F': *' '/^lifecycle:/{sub(/ *#.*/,"",$2);print $2;exit}' "$mf")"
  case "$lc" in
    experimental|beta|production|deprecated|"") ;;
    *) err "$mf: lifecycle '$lc' is not one of experimental|beta|production|deprecated" ;;
  esac
  # You cannot publish an event you do not own. Cheap to check, and it is the invariant
  # that keeps `exposes` from drifting into claiming other services' events.
  own_ev="$(awk '/^owns:/{o=1} o&&/^  events:/{e=1;next} e&&/^    - /{sub(/^    - /,"");print;next} e&&!/^    - /{exit}' "$mf" | sort -u)"
  exp_ev="$(awk '/^exposes:/{x=1} x&&/^  events:/{e=1;next} e&&/^    - /{sub(/^    - /,"");print;next} e&&!/^    - /{exit}' "$mf" | sort -u)"
  while IFS= read -r ev; do
    [[ -n "$ev" ]] || continue
    grep -qx "$ev" <<<"$own_ev" \
      || err "$mf: exposes.events lists '$ev' which is not in owns.events — a service cannot publish an event it does not own"
  done <<<"$exp_ev"

  # A repository's stack must be one its class permits. The class says what the
  # repository is; the stack says which language standards bind it. A mismatch means a
  # repository claiming rules that were never written for it — a `mobile` repo declaring
  # `rust` would inherit cargo checks and satisfy none.
  cls="$(awk -F': *' '/^class:/{sub(/ *#.*/,"",$2);print $2;exit}' "$mf")"
  stk="$(awk -F': *' '/^stack:/{sub(/ *#.*/,"",$2);print $2;exit}' "$mf")"
  if [[ -n "$cls" && -n "$stk" && -f "$CLASSREG" ]]; then
    allowed="$(awk -v c="  - class: $cls" '$0==c{f=1;next} f&&/^    stacks: /{gsub(/.*\[|\]/,"");gsub(/,/," ");print;exit} f&&/^  - class: /{exit}' "$CLASSREG")"
    grep -qw "$stk" <<<"$allowed" \
      || err "$mf: stack '$stk' is not permitted for class '$cls' — class-registry allows [${allowed// /, }] (C-0005)"
  fi

  d="$(dirname "$mf")"
  g="$d/db/grants.sql"
  [[ -f "$g" ]] || continue
  declared="$(awk '/^owns:/{o=1;next} o&&/^  tables:/{t=1;next} t&&/^    - /{sub(/^    - /,"");print;next} t&&!/^    - /{exit}' "$mf" | sort -u)"
  [[ -n "$declared" ]] || continue
  granted="$(grep -oiE 'GRANT [A-Z, ]*(INSERT|UPDATE|DELETE)[A-Z, ]* ON ([a-z_]+)' "$g" \
             | awk '{print $NF}' | sort -u)"
  while IFS= read -r t; do
    [[ -n "$t" ]] || continue
    grep -qx "$t" <<<"$granted" \
      || err "$mf declares owns.tables '$t' but $g grants it no write access — the declaration is not enforced (ANTI-0001)"
  done <<<"$declared"
  while IFS= read -r t; do
    [[ -n "$t" ]] || continue
    grep -qx "$t" <<<"$declared" \
      || err "$g grants write access on '$t' but $mf does not declare owning it — undeclared ownership (ANTI-0001)"
  done <<<"$granted"
done < <(find . -name manifest.yaml -not -path './.git/*' 2>/dev/null)

# --- 7c. cross-repository ownership (domain.entity_has_single_owner) ---------
# The other half of the check. Two repositories declaring the same table is ANTI-0001 —
# the one boundary violation that is mechanically detectable and effectively permanent
# once shipped. Vacuous while only one manifest exists, and it must stay implemented
# rather than deferred so it is armed the moment a second appears.
declare -A TABLE_OWNER=()
while IFS= read -r mf; do
  who="$(awk -F': *' '/^name:/{sub(/ *#.*/,"",$2);print $2;exit}' "$mf")"
  who="${who:-$mf}"
  while IFS= read -r tbl; do
    [[ -n "$tbl" ]] || continue
    prev="${TABLE_OWNER[$tbl]:-}"
    if [[ -n "$prev" && "$prev" != "$who" ]]; then
      err "table '$tbl' is declared owned by both '$prev' and '$who' — two writers is ANTI-0001, and it is effectively permanent once shipped"
    fi
    TABLE_OWNER[$tbl]="$who"
  done < <(awk '/^owns:/{o=1;next} o&&/^  tables:/{t=1;next} t&&/^    - /{sub(/^    - /,"");print;next} t&&!/^    - /{exit}' "$mf")
done < <(find . -name manifest.yaml -not -path './.git/*' 2>/dev/null | sort)

# --- 8. class registry (knowledge.class_registry_valid + scope coverage) -----
# C-0005 moved the repository class list out of a locked prose table into
# class-registry.yaml. That made it easier to change, which is both the intent and the
# risk: a table inside a constitution was guarded by supersession, so data needs a check
# or it is less protected than what it replaced.
#
# The prefix-to-scope map used to be hardcoded here. ADR-0015 flagged that as a hazard and
# it is now read from the registry, so there is one source of truth rather than two.
if [[ ! -f "$CLASSREG" ]]; then
  err "$CLASSREG is missing — the repository class list lives there (C-0005)"
else
  if [[ -f scripts/lib/validate-schema.py && -f class-registry.schema.json ]]; then
    cout="$(python3 scripts/lib/validate-schema.py document class-registry.schema.json "$CLASSREG" 2>&1)"
    case $? in
      0) ;;
      1) while IFS= read -r line; do [[ -n "$line" ]] && err "${line#✘ }"; done < <(grep '^✘' <<<"$cout") ;;
      *) wrn "$CLASSREG: schema validation unavailable — uniqueness checked, shape not" ;;
    esac
  fi

  # Uniqueness across all three generated fields. A collision on any one silently shadows
  # an entry: `class` generates the manifest enum, `prefix` the naming rule, `scope` the
  # value artifacts bind against.
  # `class` is the list-item key (`  - class: x`); prefix and scope are nested one level
  # deeper. Matching only the deeper indent silently skipped `class` entirely, which a
  # planted duplicate caught.
  for field in class prefix scope; do
    dupe="$(grep -oE "^(  - |    )${field}: [a-z-]+" "$CLASSREG" | awk '{print $NF}' | sort | uniq -d | tr '\n' ' ')"
    [[ -z "${dupe// /}" ]] || err "$CLASSREG: duplicate ${field} value(s): ${dupe% }"
  done

  # Every registered class must have a scope value in the schema, or no artifact could
  # ever target it. Under C-0002 this compared a prose table to the schema; the registry
  # is now the source.
  if [[ -f "$SCHEMA" ]]; then
    enum="$(awk '/"scope": \{/,/^    \},?$/' "$SCHEMA" \
            | awk '/"enum": \[/,/\]/' | grep -oE '^[[:space:]]+"[a-z]+",?$' \
            | sed 's/[[:space:]",]//g' | sort -u)"
    missing=0
    while IFS= read -r sc; do
      [[ -n "$sc" ]] || continue
      grep -qx "$sc" <<<"$enum" || {
        err "$SCHEMA: class-registry declares scope '$sc' with no matching value in the scope enum — no artifact could target that class (C-0005)"
        missing=$((missing+1)); }
    done < <(grep -oE '^    scope: [a-z]+' "$CLASSREG" | awk '{print $2}' | sort -u)
    [[ $missing -eq 0 ]] && printf '%s✔%s every registered class has a scope value\n' "$GRN" "$RST"
  fi
fi

# --- 8b. artifact-type registry (knowledge.artifact_registry_valid) ----------
# ADR-0022. The registry claims to GENERATE knowledge.schema.json's `type` enum and the
# prefix alternation in its four id patterns. A "generates" claim that nothing verifies is
# the exact pattern this repository has been caught by repeatedly — class-registry.yaml's
# "Sorted by id" header, the secret scan's four defects, the commit-identity gate that
# never examined a commit. So the agreement is checked in BOTH directions: a prefix in the
# registry but not the schema means artifacts of that type cannot validate, and a prefix in
# the schema but not the registry means ids nothing scans or resolves.
printf '\n'
if [[ -f scripts/lib/validate-schema.py && -f artifact-registry.schema.json ]]; then
  aout="$(python3 scripts/lib/validate-schema.py document artifact-registry.schema.json "$ARTREG" 2>&1)"
  case $? in
    0) ;;
    1) while IFS= read -r line; do [[ -n "$line" ]] && err "${line#✘ }"; done < <(grep '^✘' <<<"$aout") ;;
    *) wrn "$ARTREG: schema validation unavailable — uniqueness and agreement checked, shape not" ;;
  esac
fi

# Uniqueness across all three generated fields, for the same reason class-registry checks
# its three: a collision silently shadows an entry. Two types sharing a `directory` would
# make one of them unscannable while both appear declared.
for field in type prefix directory; do
  dupe="$(grep -oE "^(  - |    )${field}: [A-Za-z-]+" "$ARTREG" | awk '{print $NF}' | sort | uniq -d | tr '\n' ' ')"
  [[ -z "${dupe// /}" ]] || err "$ARTREG: duplicate ${field} value(s): ${dupe% }"
done

# `requires_trigger: always` without a `review_date` requirement would let a bet sit
# unjudged forever, so the registry may not declare a type that demands one without the
# other. Stated here because it is a property of the registry, not of any one artifact.
while IFS=$'\t' read -r t pfx dir lockable deferrable rt rs imm; do
  [[ -n "$t" ]] || continue
  [[ "$rt" == "always" && "$lockable" == "true" ]] \
    && err "$ARTREG: type '$t' declares requires_trigger: always with lockable: true — an artifact that must state what would falsify it cannot also be immune to contradiction (ADR-0022)"
  [[ "$deferrable" == "true" && "$rt" == "never" ]] \
    && err "$ARTREG: type '$t' is deferrable but requires_trigger is 'never' — a deferred artifact with no trigger is the failure ADR-0018 exists to prevent"
  if [[ ! -d "$dir" ]]; then
    wrn "$ARTREG: type '$t' declares directory '$dir/', which does not exist — legitimate for a declared-but-unpopulated type, and it means nothing of this type is scanned"
  fi
done < <(artreg_entries "$ARTREG")

if [[ -f "$SCHEMA" ]]; then
  # json is stdlib, so this needs no PyYAML and cannot hit the fallback path the
  # frontmatter validator has. Compares the two directions separately so the message says
  # which way the drift went.
  agree="$(python3 - "$SCHEMA" <<'PY' 2>&1
import json, re, sys
schema = json.load(open(sys.argv[1]))
print("TYPES", " ".join(sorted(schema["properties"]["type"]["enum"])))
# A list, not a set. Counting distinct pattern strings reported 1 when all four agreed,
# which is the healthy case — the count must be of OCCURRENCES so a dropped or renamed
# pattern is visible, while the emitted PREFIXES lines stay deduplicated so that exactly
# one line means all four agree.
occurrences = []
def walk(node):
    if isinstance(node, dict):
        m = re.match(r"^\^\(([A-Z|]+)\)-\[0-9\]\{4\}\$$", node.get("pattern", ""))
        if m:
            occurrences.append(m.group(1))
        for v in node.values():
            walk(v)
    elif isinstance(node, list):
        for v in node:
            walk(v)
walk(schema)
print("PATTERNS", len(occurrences))
for p in sorted(set(occurrences)):
    print("PREFIXES", " ".join(sorted(p.split("|"))))
PY
)"
  if [[ -z "$agree" || "$agree" != TYPES* ]]; then
    err "$SCHEMA: could not read the type enum or id patterns — ${agree%%$'\n'*}"
  else
    schema_types="$(awk '/^TYPES /{$1=""; sub(/^ /,""); print}' <<<"$agree")"
    npat="$(awk '/^PATTERNS /{print $2}' <<<"$agree")"
    schema_prefixes="$(awk '/^PREFIXES /{$1=""; sub(/^ /,""); print}' <<<"$agree" | sort -u)"
    reg_types="$(printf '%s\n' "${!T_DIR[@]}" | sort | tr '\n' ' ')"
    reg_prefixes="$(printf '%s\n' "${ID_PREFIXES[@]}" | sort | tr '\n' ' ')"

    [[ "$schema_types" == "${reg_types% }" ]] \
      || err "$SCHEMA type enum [$schema_types] disagrees with $ARTREG [${reg_types% }] — the enum is generated from the registry (ADR-0022)"

    # All four patterns must carry the SAME prefix set. They diverged silently before this
    # check existed: `id` could accept a prefix that `supersedes` rejected, so an artifact
    # would validate but nothing could ever supersede it.
    if [[ "$(wc -l <<<"$schema_prefixes" | tr -d ' ')" != "1" ]]; then
      err "$SCHEMA: its id patterns do not all carry the same prefix set — a prefix valid in 'id' but not 'supersedes' produces an artifact nothing can ever supersede"
    elif [[ "$schema_prefixes" != "${reg_prefixes% }" ]]; then
      err "$SCHEMA id patterns allow [$schema_prefixes] but $ARTREG declares [${reg_prefixes% }] — a prefix in only one of the two is either unvalidatable or unresolvable (ADR-0022)"
    elif [[ "${npat:-0}" -lt 4 ]]; then
      err "$SCHEMA: found only ${npat:-0} artifact-id pattern(s), expected 4 (id, supersedes.items, superseded_by, resolved_by) — one was renamed or dropped"
    else
      printf '%s✔%s %s agrees with %s on %d types and %d prefixes\n' \
        "$GRN" "$RST" "$SCHEMA" "$ARTREG" "$(wc -w <<<"$schema_types" | tr -d ' ')" "$(wc -w <<<"$schema_prefixes" | tr -d ' ')"
    fi
  fi
fi

# --- 8c. the sites that stay hardcoded, verified anyway ---------------------
# ADR-0022 derives ten of fifteen sites. Three of the remainder cannot reasonably be
# derived — two hooks that must stay fast and dependency-free, and a schema this gate does
# not generate — so they are hardcoded AND CHECKED, which is the same bargain ADR-0014 made
# for check ids. Hardcoding is acceptable; hardcoding with nothing comparing it is not.
#
# All three were found carrying a STALE list, which is why this section exists rather than a
# comment asking people to remember. One of them was a real hole in ADR-0007's layer 1: the
# PreToolUse guard omitted requirements/, so a locked REQ- could be rewritten in place
# in-session while check-knowledge.sh section 3 forbade exactly that.
prefix_sites=(
  "check-registry.schema.json"
  "plugins/paridyum-platform/hooks/capture-open-loops.sh"
)
for site in "${prefix_sites[@]}"; do
  [[ -f "$site" ]] || continue
  found="$(grep -oE '\((ADR|ANTI|BET|C|PAT|REQ|RFC|RUN|STD)(\|[A-Z]+)+\)' "$site" | head -1 \
           | tr -d '()' | tr '|' '\n' | sort -u | tr '\n' ' ')"
  if [[ -z "${found// /}" ]]; then
    wrn "$site: no artifact-id prefix alternation found — it may have been rewritten in a shape this check cannot read, which is indistinguishable from removal"
  elif [[ "${found% }" != "${reg_prefixes:-}" && "${found% }" != "$(printf '%s\n' "${ID_PREFIXES[@]}" | sort | tr '\n' ' ' | sed 's/ $//')" ]]; then
    err "$site: artifact-id prefixes [${found% }] disagree with $ARTREG — a stale copy here fails silently (ADR-0022)"
  fi
done

# The PreToolUse guard's directory list must equal the types declaring
# immutable_when_locked: true. It is hardcoded there on purpose — the hook runs on every
# tool call and a YAML parse per keystroke is a hook people disable — so the agreement is
# checked here instead.
GUARD="plugins/paridyum-platform/hooks/guard-locked-decisions.sh"
if [[ -f "$GUARD" ]]; then
  guard_dirs="$(grep -oE '\*"/[a-z-]+/"\*\.md' "$GUARD" | sed -E 's#.*/([a-z-]+)/.*#\1#' | sort -u | tr '\n' ' ')"
  want_dirs="$(printf '%s\n' "${IMMUTABLE_DIRS[@]}" | sort -u | tr '\n' ' ')"
  if [[ -z "${guard_dirs// /}" ]]; then
    wrn "$GUARD: no artifact directory list found — cannot verify it matches $ARTREG"
  elif [[ "${guard_dirs% }" != "${want_dirs% }" ]]; then
    err "$GUARD guards [${guard_dirs% }] but $ARTREG declares immutable_when_locked for [${want_dirs% }] — ADR-0007 layer 1 and check-knowledge.sh section 3 must agree on which artifacts are immutable, and layer 1 is the one that cannot be reasoned around (ADR-0022)"
  else
    printf '%s✔%s the PreToolUse guard and section 3 agree on %d immutable directories\n' \
      "$GRN" "$RST" "$(wc -w <<<"$want_dirs" | tr -d ' ')"
  fi
fi

# The Stop hook's ART_DIRS must equal every directory the registry declares. It is hardcoded
# there for the same reason the guard's list is — the hook runs on every Stop and a YAML parse
# it does not need is a hook people disable — so the agreement is checked here instead.
#
# THE SIXTEENTH SITE. ADR-0022 enumerated fifteen places the type list was hardcoded and this
# was not one of them, because the entry for capture-open-loops.sh named its ID-PREFIX scanner,
# which the loop above does compare. Its DIRECTORY list sat beside that, carrying a comment
# asserting it "MUST stay identical to ART_DIRS in scripts/check-knowledge.sh" — an invariant
# stated in prose with nothing enforcing it, in the file whose own argument is that unenforced
# prose is a suggestion.
#
# The cost was not hypothetical. It held seven directories against the registry's eleven, so
# `bets`, `glossary`, `requirements` and `markets` were never scanned and four `proposed`
# artifacts were invisible to the ADR-0011 ledger — a mechanism reporting a smaller backlog
# than it had, which is indistinguishable from having no backlog. Found on 2026-08-17, three
# artifact types after it first went stale.
LEDGER_HOOK="plugins/paridyum-platform/hooks/capture-open-loops.sh"
if [[ -f "$LEDGER_HOOK" ]]; then
  ledger_dirs="$(grep -oE '^ART_DIRS=\([a-z? -]+\)' "$LEDGER_HOOK" | head -1 \
                 | sed -E 's/^ART_DIRS=\(//; s/\)$//' | tr ' ' '\n' | sort -u | tr '\n' ' ')"
  want_all="$(printf '%s\n' "${ART_DIRS[@]}" | sort -u | tr '\n' ' ')"
  if [[ -z "${ledger_dirs// /}" ]]; then
    wrn "$LEDGER_HOOK: no ART_DIRS list found — it may have been rewritten in a shape this check cannot read, which is indistinguishable from removal"
  elif [[ "${ledger_dirs% }" != "${want_all% }" ]]; then
    err "$LEDGER_HOOK scans [${ledger_dirs% }] but $ARTREG declares [${want_all% }] — a directory missing here is an artifact class the ADR-0011 ledger never reports, so the backlog reads shorter than it is (ADR-0022, ADR-0050)"
  else
    printf '%s✔%s the open-loop ledger and %s agree on %d scanned directories\n' \
      "$GRN" "$RST" "$ARTREG" "$(wc -w <<<"$want_all" | tr -d ' ')"
  fi
fi

# --- 8d. framework mappings (knowledge.framework_ids_resolve) ---------------
# A control may map outward to external framework ids (SSDF task, CIS safeguard, CSF
# category). The mapping is only worth having if a wrong one fails, because its whole
# purpose is to answer "what covers this obligation" — and an id that does not exist
# answers that question confidently and wrongly. Nobody reviewing a diff has the
# frameworks memorised, so review is not a control here; this is.
#
# Two things are checked, and neither can be expressed in check-registry.schema.json: the
# framework key must be registered, and each id must match that framework's own
# `id_pattern`. Keeping the pattern in framework-registry.yaml rather than restating it in
# the schema is what stops the two drifting.
FWREG="framework-registry.yaml"
if [[ ! -f "$FWREG" ]]; then
  wrn "$FWREG is missing — framework mappings in $MANIFEST cannot be validated"
elif ! grep -q '^    frameworks:' "$MANIFEST" 2>/dev/null; then
  : # nothing mapped yet; nothing to verify
elif ! command -v python3 >/dev/null; then
  wrn "python3 unavailable — framework ids in $MANIFEST are unverified"
else
  fwout="$(python3 - "$FWREG" "$MANIFEST" <<'PY'
import re, sys
try:
    import yaml
except ImportError:
    sys.exit(2)

reg, man = (yaml.safe_load(open(p, encoding="utf-8")) for p in sys.argv[1:3])
pats, errs = {}, []
for f in reg.get("frameworks", []):
    name, pat = f.get("framework"), f.get("id_pattern")
    if not name or not pat:
        errs.append(f"{sys.argv[1]}: entry missing framework or id_pattern")
        continue
    try:
        pats[name] = re.compile(pat)
    except re.error as e:
        errs.append(f"{sys.argv[1]}: {name} id_pattern does not compile — {e}")

used = set()
for c in man.get("checks", []):
    fw = c.get("frameworks")
    if not fw:
        continue
    cid = c.get("check", "<unnamed>")
    if not isinstance(fw, dict):
        errs.append(f"{cid}: frameworks must be a mapping of framework -> [ids]")
        continue
    for name, ids in fw.items():
        used.add(name)
        if name not in pats:
            errs.append(f"{cid}: framework '{name}' is not registered in {sys.argv[1]}")
            continue
        for i in ids or []:
            if not pats[name].fullmatch(str(i)):
                errs.append(f"{cid}: '{i}' is not a valid {name} id (expected {pats[name].pattern})")

# A registered framework nobody maps to is the mirror of an orphan check declaration:
# harmless, but it reads as coverage in a file whose only job is to describe coverage.
# `mapping_deferred_to` names the requirement that will close it, which is the difference
# between a deliberate gap and a forgotten one — and it keeps a permanent warning off the
# run, because a warning that is always there is one people stop reading.
deferred = {f["framework"]: f.get("mapping_deferred_to")
            for f in reg.get("frameworks", []) if f.get("framework")}
for name in sorted(set(pats) - used):
    if not deferred.get(name):
        print(f"WARN|{name} is registered but no control maps to it, and no "
              f"mapping_deferred_to says why")
for e in errs:
    print(f"ERR|{e}")
mapped = sum(1 for c in man.get("checks", []) if c.get("frameworks"))
print(f"OK|{mapped} of {len(man.get('checks', []))} controls carry a framework mapping")
PY
  )"
  case $? in
    0)
      while IFS='|' read -r kind msg; do
        [[ -n "$msg" ]] || continue
        case "$kind" in
          ERR)  err "$MANIFEST: $msg" ;;
          WARN) wrn "$FWREG: $msg" ;;
          OK)   printf '%s✔%s %s\n' "$GRN" "$RST" "$msg" ;;
        esac
      done <<<"$fwout"
      ;;
    2) wrn "PyYAML unavailable — framework ids in $MANIFEST are unverified" ;;
    *) err "$FWREG: framework mapping validation failed to run" ;;
  esac
fi

# --- 8e. test-type registry + verification mapping (ADR-0029, ADR-0030) -----
# Three checks live here because they read the same two files and splitting them would mean
# parsing test-registry.yaml three times in a gate that already runs on every commit:
#
#   knowledge.test_registry_valid              the registry means what it says
#   knowledge.verification_methods_resolve     a TST- id in an artifact exists
#   knowledge.verification_mapping_is_symmetric the two directions agree          (warn)
#
# The symmetry pass is the sibling of section 7, and it inherits section 7's hard-won escape:
# a `locked` or `superseded` artifact CANNOT gain a `verified_by` entry, because that is a
# content edit, so a test type naming one unilaterally is the normal path rather than drift.
# `deferred` is excused for the adjacent reason — its verification method does not exist yet
# by construction. Without both escapes this would warn permanently with no legal remedy,
# which is how a gate teaches people to ignore it.
#
# The unverified report at the end is deliberately informational, exactly like section 4's
# enforcement backlog. No check can decide whether an artifact states a runtime obligation,
# and a gate that guessed would be satisfiable by adding an irrelevant id.
printf '\n'
TESTREG="test-registry.yaml"
if [[ ! -f "$TESTREG" ]]; then
  err "$TESTREG is missing — the test-type id-space lives there (ADR-0029)"
elif ! command -v python3 >/dev/null; then
  wrn "python3 unavailable — $TESTREG and every verified_by are unverified"
else
  if [[ -f scripts/lib/validate-schema.py && -f test-registry.schema.json ]]; then
    tsout="$(python3 scripts/lib/validate-schema.py document test-registry.schema.json "$TESTREG" 2>&1)"
    case $? in
      0) ;;
      1) while IFS= read -r line; do [[ -n "$line" ]] && err "${line#✘ }"; done < <(grep '^✘' <<<"$tsout") ;;
      *) wrn "$TESTREG: schema validation unavailable — cross-references checked, shape not" ;;
    esac
  fi

  tregout="$(python3 - "$TESTREG" "$FWREG" "$CLASSREG" "$MANIFEST" "${FILES[@]}" <<'PY'
import re, sys

try:
    import yaml
except ImportError:
    sys.exit(2)

treg_p, fwreg_p, classreg_p, manifest_p = sys.argv[1:5]
artifacts = sys.argv[5:]


def load(p):
    with open(p, encoding="utf-8") as fh:
        return yaml.safe_load(fh) or {}


treg = load(treg_p)
axes = {a.get("axis") for a in treg.get("axes", []) if isinstance(a, dict)}
tiers = {t.get("tier") for t in treg.get("tiers", []) if isinstance(t, dict)}
types = [t for t in treg.get("test_types", []) if isinstance(t, dict)]

errs, warns = [], []

# Declared elsewhere, resolved here. Each of these three is a list this registry does not own,
# and a value outside one silently binds nothing.
stacks = {c.get("stack") for c in []}
classreg = load(classreg_p)
stacks = {"any"}
for c in classreg.get("classes", []):
    for s in c.get("stacks", []) or []:
        stacks.add(s)
check_ids = {c.get("check") for c in load(manifest_p).get("checks", []) if isinstance(c, dict)}

seen_ids, seen_names, last = set(), set(), None
for t in types:
    tid = t.get("test_type", "<unnamed>")
    if tid in seen_ids:
        errs.append(f"duplicate entry for '{tid}' — the second silently shadows the first")
        continue
    seen_ids.add(tid)
    name = t.get("name")
    if name in seen_names:
        errs.append(f"{tid}: duplicate name '{name}' — names appear in CI job ids, so a "
                    f"collision makes two suites indistinguishable")
    seen_names.add(name)
    # The header claims "Sorted by test_type" and every other registry here has been caught
    # by an unverified claim in its own header at least once.
    if last is not None and tid < last:
        errs.append(f"'{tid}' appears after '{last}' — entries must be sorted by test_type")
    last = tid

    if t.get("axis") not in axes:
        errs.append(f"{tid}: axis '{t.get('axis')}' is not declared in axes:")
    if t.get("tier") not in tiers:
        errs.append(f"{tid}: tier '{t.get('tier')}' is not declared in tiers:")
    if t.get("status") == "deferred" and not t.get("trigger"):
        errs.append(f"{tid}: status is deferred with no trigger — a deferred entry without "
                    f"one is a TODO with an id (ADR-0018)")
    if t.get("status") != "deferred" and t.get("trigger"):
        errs.append(f"{tid}: carries a trigger while not deferred — the trigger has already "
                    f"fired or the status is wrong, and neither is readable as written")
    for s in t.get("stacks", []) or []:
        if s not in stacks:
            errs.append(f"{tid}: stack '{s}' is not declared in {classreg_p}")
    for cid in t.get("checks", []) or []:
        if cid not in check_ids:
            errs.append(f"{tid}: names check '{cid}', absent from {manifest_p} — unresolved "
                        f"id, so nothing enforces it (ADR-0014)")

# Same rule as check-registry.yaml's mappings, same registry, same reason (ADR-0028).
fwreg = load(fwreg_p)
pats = {}
for f in fwreg.get("frameworks", []):
    if f.get("framework") and f.get("id_pattern"):
        try:
            pats[f["framework"]] = re.compile(f["id_pattern"])
        except re.error as e:
            errs.append(f"{fwreg_p}: {f['framework']} id_pattern does not compile — {e}")
for t in types:
    tid = t.get("test_type", "<unnamed>")
    for fname, ids in (t.get("frameworks") or {}).items():
        if fname not in pats:
            errs.append(f"{tid}: framework '{fname}' is not registered in {fwreg_p}")
            continue
        for i in ids or []:
            if not pats[fname].fullmatch(str(i)):
                errs.append(f"{tid}: '{i}' is not a valid {fname} id "
                            f"(expected {pats[fname].pattern})")

# --- the verification mapping, both directions ------------------------------
FM = re.compile(r"\A---\n(.*?)\n---\n", re.S)
art_status, art_verified = {}, {}
for path in artifacts:
    with open(path, encoding="utf-8") as fh:
        m = FM.match(fh.read())
    if not m:
        continue
    try:
        fm = yaml.safe_load(m.group(1)) or {}
    except yaml.YAMLError:
        continue          # section 1 owns malformed frontmatter; do not report it twice
    aid = fm.get("id")
    if not aid:
        continue
    art_status[aid] = fm.get("status")
    art_verified[aid] = list(fm.get("verified_by") or [])

governed = {}
for t in types:
    for aid in t.get("governed_by", []) or []:
        governed.setdefault(aid, set()).add(t.get("test_type"))

for aid, vb in sorted(art_verified.items()):
    for tid in vb:
        if tid not in seen_ids:
            errs.append(f"{aid}: verified_by names '{tid}', absent from {treg_p} — "
                        f"unresolved id, so nothing demonstrates it (ADR-0030)")
        elif tid not in governed.get(aid, set()):
            warns.append(f"{aid} names {tid} in verified_by, but that type's governed_by "
                         f"omits {aid} — a claim written in one place only")

# A type may name an immutable or deferred artifact unilaterally. Anything else is drift.
EXCUSED = {"locked", "superseded", "deferred", "draft"}
for aid, tids in sorted(governed.items()):
    if aid not in art_status:
        continue          # section 2 owns dangling references
    if art_status[aid] in EXCUSED:
        continue
    missing = sorted(t for t in tids if t not in art_verified.get(aid, []))
    if missing:
        warns.append(f"{aid} is governed by [{' '.join(missing)}] but its verified_by omits "
                     f"them — and it is neither immutable nor deferred, so it could say so")

for e in errs:
    print(f"ERR|{e}")
for w in warns:
    print(f"WARN|{w}")

req = sum(1 for t in types if t.get("status") == "required")
rec = sum(1 for t in types if t.get("status") == "recommended")
dfr = sum(1 for t in types if t.get("status") == "deferred")
print(f"OK|{len(types)} test types across {len(axes)} axes and {len(tiers)} tiers "
      f"— {req} required, {rec} recommended, {dfr} deferred")

# The companion to section 4's enforcement backlog: what is defended at the diff and
# unproven at runtime. Informational, never fatal (ADR-0030 decision 6).
unverified = sorted(a for a, s in art_status.items()
                    if s in ("locked", "accepted") and not art_verified.get(a)
                    and a in governed)
if unverified:
    print(f"INFO|{len(unverified)} artifact(s) governed by a test type name no verification "
          f"method of their own — immutable, so this is a reading list: "
          f"{' '.join(unverified)}")
PY
  )"
  case $? in
    0)
      while IFS='|' read -r kind msg; do
        [[ -n "$msg" ]] || continue
        case "$kind" in
          ERR)  err "$TESTREG: $msg" ;;
          WARN) wrn "$msg" ;;
          OK)   printf '%s✔%s %s\n' "$GRN" "$RST" "$msg" ;;
          INFO) printf '    %s\n' "$msg" ;;
        esac
      done <<<"$tregout"
      ;;
    2) wrn "PyYAML unavailable — $TESTREG and every verified_by are unverified" ;;
    *) err "$TESTREG: test-registry validation failed to run" ;;
  esac
fi

# --- 8f. product registry (ADR-0050) ----------------------------------------
# Two checks, one parse, for the reason 8e gives: the file would otherwise be read twice in a
# gate that runs on every commit.
#
#   knowledge.product_registry_valid   the registry means what it says
#   knowledge.product_slugs_resolve    every `product:` written anywhere is registered
#
# THE FIFTH INSTANCE of the argument sections 8 and 8b each make. Moving a list out of code
# makes it easier to change, which is both the intent and the risk — registry data needs a
# check or it is less protected than the hardcoded list it replaced. The half that will
# actually fire is the dossier prefix: a `dossier` entry naming an artifact prefix that does
# not exist is an obligation nothing can resolve, which reads as governance and enforces
# nothing, and that is the silent shape ADR-0014's audit found four times.
#
# The KIND vocabulary is read from product-registry.schema.json rather than restated here,
# because the registry carries no `kinds:` list of its own — `stages` is data and `kind` is an
# enum, and the set of products' kinds could not serve as the vocabulary because no catalogue
# product is registered, deliberately (C-0006 anti-goal 6). Reading the schema keeps one source
# of truth, the same bargain 8b strikes in the opposite direction.
#
# BOTH CHECKS ARE VACUOUS TODAY on their most interesting half — no artifact and no manifest
# carries a `product:` — and they must stay implemented rather than deferred so they are armed
# the moment one does. Section 7c takes the same position while only one manifest exists. The
# count is printed rather than a bare tick, because "every entry resolves" over zero entries is
# the green check nobody has watched fail.
printf '\n'
PRODREG="product-registry.yaml"
PRODSCHEMA="product-registry.schema.json"
if [[ ! -f "$PRODREG" ]]; then
  err "$PRODREG is missing — the product list lives there (ADR-0050)"
else
  if [[ -f scripts/lib/validate-schema.py && -f "$PRODSCHEMA" ]]; then
    prout="$(python3 scripts/lib/validate-schema.py document "$PRODSCHEMA" "$PRODREG" 2>&1)"
    case $? in
      0) ;;
      1) while IFS= read -r line; do [[ -n "$line" ]] && err "${line#✘ }"; done < <(grep '^✘' <<<"$prout") ;;
      *) wrn "$PRODREG: schema validation unavailable — cross-references checked, shape not" ;;
    esac
  fi

  # Duplicate slugs in bash as well as in the python pass below, and not redundantly: this is
  # the one invariant that must survive PyYAML being absent, because a duplicate slug is the
  # failure whose consequence is permanent. A slug is the handle for a product in frontmatter,
  # in manifests and in git history, so the second entry does not merely shadow the first — it
  # makes every reference written since ambiguous, with nothing recording which was meant.
  dupe="$(grep -oE '^  - product: [a-z0-9-]+' "$PRODREG" | awk '{print $NF}' | sort | uniq -d | tr '\n' ' ')"
  [[ -z "${dupe// /}" ]] || err "$PRODREG: duplicate product slug(s): ${dupe% } — the second entry silently shadows the first, and a slug is the permanent handle for a product (ADR-0050)"

  if ! command -v python3 >/dev/null; then
    wrn "python3 unavailable — $PRODREG cross-references and every product: entry are unverified"
  else
    mapfile -t REPO_MANIFESTS < <(find . -name manifest.yaml -not -path './.git/*' 2>/dev/null | sort)
    predout="$(python3 - "$PRODREG" "$PRODSCHEMA" "$ARTREG" "${REPO_MANIFESTS[@]}" -- "${FILES[@]}" <<'PY'
import json, re, sys

try:
    import yaml
except ImportError:
    sys.exit(2)

prodreg_p, prodschema_p, artreg_p = sys.argv[1:4]
rest = sys.argv[4:]
cut = rest.index("--")
manifests, artifacts = rest[:cut], rest[cut + 1:]


def load(p):
    with open(p, encoding="utf-8") as fh:
        return yaml.safe_load(fh) or {}


reg = load(prodreg_p)
errs, slugerrs = [], []

stages = [s.get("stage") for s in reg.get("stages", []) or [] if isinstance(s, dict)]
stage_set = {s for s in stages if s}

# The kind vocabulary. Read from the schema because the registry has no list of its own; an
# unreadable enum is reported rather than defaulted, since a silently empty vocabulary would
# accept every kind and this check would stop being one.
kinds = set()
try:
    schema = json.load(open(prodschema_p, encoding="utf-8"))
    kinds = set(schema["properties"]["dossier"]["items"]["properties"]["kind"]["enum"])
except (OSError, ValueError, KeyError, TypeError) as e:
    errs.append(f"could not read the kind enum from {prodschema_p} — {e}. Every `kind` in "
                f"this file is therefore unchecked")

prefixes = {t.get("prefix") for t in load(artreg_p).get("types", []) or []
            if isinstance(t, dict)}

registered = set()
for p in reg.get("products", []) or []:
    if not isinstance(p, dict):
        errs.append("a products: entry is not a mapping")
        continue
    slug = p.get("product") or "<unnamed>"
    if slug in registered:
        errs.append(f"duplicate entry for '{slug}' — the second silently shadows the first")
        continue
    registered.add(slug)
    if p.get("stage") not in stage_set:
        errs.append(f"{slug}: stage '{p.get('stage')}' is not declared in stages: — the "
                    f"lifecycle is ordered and a stage outside it has no position, so nothing "
                    f"can say whether the product moved forward or which dossier it owes")
    if kinds and p.get("kind") not in kinds:
        errs.append(f"{slug}: kind '{p.get('kind')}' is not a declared kind — `kind` is what "
                    f"selects the dossier, so a product with an unknown one owes nothing")

seen_pairs = set()
for d in reg.get("dossier", []) or []:
    if not isinstance(d, dict):
        errs.append("a dossier: entry is not a mapping")
        continue
    kind, stage = d.get("kind"), d.get("stage")
    where = f"dossier[{kind}/{stage}]"
    if kinds and kind not in kinds:
        errs.append(f"{where}: kind '{kind}' does not exist — this obligation binds no product")
    if stage not in stage_set:
        errs.append(f"{where}: stage '{stage}' is not declared in stages: — an obligation at "
                    f"a stage no product can reach binds nobody")
    if (kind, stage) in seen_pairs:
        errs.append(f"{where}: duplicate kind/stage pair — the second entry silently shadows "
                    f"the first, so half the dossier would be owed and never asked for")
    seen_pairs.add((kind, stage))
    # THE HALF THAT WILL ACTUALLY FIRE (ADR-0050's own note). A prefix nothing declares is a
    # requirement no gate can count, which is ADR-0014's unresolved-check-id one axis over.
    for pref in d.get("artifacts", []) or []:
        if pref not in prefixes:
            errs.append(f"{where}: names artifact prefix '{pref}', absent from {artreg_p} — a "
                        f"dossier obligation nothing can resolve, which reads as governance "
                        f"and enforces nothing (ADR-0014)")

# --- knowledge.product_slugs_resolve ----------------------------------------
# A typo'd slug produces an artifact that validates, binds nothing, and reads to a human as
# though it governs the product whose name it nearly spells. Deliberately NOT an enum in
# knowledge.schema.json: that would put the product list in a second place, which is the whole
# of RFC-0015's argument.
FM = re.compile(r"\A---\n(.*?)\n---\n", re.S)
cited = 0


def check_products(where, value):
    global cited
    if value is None:
        return                # absent means EVERY product (ADR-0050 §1), never an error
    if not isinstance(value, list):
        slugerrs.append(f"{where}: `product:` must be a list of slugs")
        return
    for slug in value:
        cited += 1
        if slug not in registered:
            slugerrs.append(f"{where}: names product '{slug}', absent from {prodreg_p} — an "
                            f"unregistered slug binds nothing while reading as though it "
                            f"governs the product it nearly spells (ADR-0050)")


for path in artifacts:
    with open(path, encoding="utf-8") as fh:
        m = FM.match(fh.read())
    if not m:
        continue              # section 1 owns a missing frontmatter block
    try:
        fm = yaml.safe_load(m.group(1)) or {}
    except yaml.YAMLError:
        continue              # section 1 owns malformed YAML; do not report it twice
    check_products(fm.get("id") or path, fm.get("product"))

for mf in manifests:
    try:
        check_products(mf, load(mf).get("product"))
    except yaml.YAMLError:
        continue              # section 7b owns an unparseable manifest

for e in errs:
    print(f"REG|{e}")
for e in slugerrs:
    print(f"SLUG|{e}")
print(f"OK|{len(registered)} product(s) across {len(stage_set)} stages, {len(seen_pairs)} "
      f"dossier entries — {cited} product: reference(s) checked")
PY
    )"
    case $? in
      0)
        while IFS='|' read -r kind msg; do
          [[ -n "$msg" ]] || continue
          case "$kind" in
            REG)  err "$PRODREG: $msg" ;;
            SLUG) err "$msg" ;;
            OK)   printf '%s✔%s %s\n' "$GRN" "$RST" "$msg" ;;
          esac
        done <<<"$predout"
        ;;
      2) wrn "PyYAML unavailable — $PRODREG cross-references and every product: entry are unverified" ;;
      *) err "$PRODREG: product-registry validation failed to run" ;;
    esac
  fi
fi

# --- 9. enforcement inventory (ADR-0014: reported at every run) -------------
# Not gated on --audit. A backlog is worth nothing if seeing it requires
# choosing to look.
if [[ ${#DECLARED[@]} -gt 0 ]]; then
  printf '\n'
  impl=0
  for id in "${!DECLARED[@]}"; do
    [[ "${DECL_OWNER[$id]:-tool-forge}" == "tool-forge" ]] || impl=$((impl+1))
  done
  # `|| echo 0` would append a second zero to grep's own "0" and break printf.
  part=$(grep -c '^    partial: true' "$MANIFEST" 2>/dev/null || true)
  printf '%d check ids declared — %d implemented (%d only partly), %d awaiting tool-forge\n' \
    "${#DECLARED[@]}" "$impl" "$part" "$(( ${#DECLARED[@]} - impl ))"
  printf '  by owner: '
  for id in "${!DECL_OWNER[@]}"; do printf '%s\n' "${DECL_OWNER[$id]}"; done \
    | sort | uniq -c | sort -rn | awk '{printf "%s=%s  ", $2, $1}'
  printf '\n'
fi

# --- summary ----------------------------------------------------------------
printf '\n'
if [[ $ERRORS -gt 0 ]]; then
  printf '%s%d error(s)%s, %d warning(s)\n' "$RED" "$ERRORS" "$RST" "$WARNS"
  exit 1
fi
printf '%sKnowledge check passed%s (%d warning(s))\n' "$GRN" "$RST" "$WARNS"
