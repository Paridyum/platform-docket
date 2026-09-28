#!/usr/bin/env bash
#
# Stop hook — the anti-forgetting mechanism (ADR-0011).
#
# Runs when Claude finishes responding. Recomputes the open-loop ledger and appends
# anything suspicious from this session. The SessionStart hook then surfaces it, so
# an unrecorded decision is put back in front of whoever starts the next session —
# whether or not either party remembers it exists.
#
# Two sections, deliberately different in kind:
#
#   REGENERATED  mechanical signals, recomputed every run. Self-healing: close the
#                loop and the entry disappears. Never hand-edited.
#   APPENDED     heuristic signals from the transcript. Cannot be auto-cleared
#                because only a human knows whether the decision was real, so these
#                accumulate until explicitly resolved. That is the point: an entry
#                nobody clears is itself the signal.
#
# Exits 0 unconditionally. A Stop hook that fails must never disrupt the session.

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


payload="$(cat 2>/dev/null || true)"
command -v jq >/dev/null 2>&1 || exit 0
jq -e . >/dev/null 2>&1 <<<"$payload" || exit 0

cwd="$(jq -r '.cwd // empty' <<<"$payload")"; [[ -z "$cwd" ]] && cwd="$PWD"
transcript="$(jq -r '.transcript_path // empty' <<<"$payload")"
session="$(jq -r '.session_id // "unknown"' <<<"$payload")"

# Only act inside the knowledge repo — that is where the ledger lives and where
# unrecorded decisions actually matter.
root="$cwd"
while [[ "$root" != "/" && -n "$root" ]]; do
  [[ -d "$root/constitution" && -d "$root/adr" ]] && break
  root="$(dirname "$root")"
done
[[ -d "$root/adr" ]] || exit 0

LEDGER="$root/DECISION-DEBT.md"
today="$(date +%Y-%m-%d)"
# Seconds at local midnight of a YYYY-MM-DD date (BSD date, then GNU date), or nothing.
midnight() {
  date -j -f "%Y-%m-%d %H:%M:%S" "$1 00:00:00" +%s 2>/dev/null || date -d "$1 00:00:00" +%s 2>/dev/null
}
today_s="$(midnight "$today")"

fm() { awk -v k="^$2:" 'NR==1 && $0!="---"{exit} NR>1 && $0=="---"{exit} $0~k{sub(/^[^:]*:[ \t]*/,""); print; exit}' "$1" 2>/dev/null; }

# Artifact directories. MUST stay identical to ART_DIRS in scripts/check-knowledge.sh.
# They cannot share a file: this hook ships inside the plugin (ADR-0007) and runs from
# the plugin install directory, while the script lives in the repository. Add a
# directory to both in the same commit.
#
# THIS LIST WAS STALE FOR THE LIFE OF FOUR ARTIFACT TYPES, and the comment above is why it
# went unnoticed: it states the invariant and nothing enforced it. check-knowledge.sh builds
# its ART_DIRS from artifact-registry.yaml, so "identical" silently meant "identical to
# whatever the registry said in August 2026" — seven directories against the registry's
# eleven. `bets`, `glossary`, `requirements` and `markets` were never scanned, so BET-0008,
# BET-0009, GLO-0007 and GLO-0008 sat `proposed` and INVISIBLE to the ledger whose whole
# purpose is that a forgotten loop gets put back in front of the next person. The reported
# open-loop count was eleven; the real one was fifteen.
#
# This is ADR-0022's own defect recurring inside ADR-0011's mechanism, and it is a SIXTEENTH
# site — ADR-0022 enumerated fifteen and this was not among them, because section 8c compares
# this file's id-prefix alternation and never looked at its directory list. That comparison
# now exists (check-knowledge.sh section 8c), which is the actual fix; extending the list
# below is only the symptom.
ART_DIRS=(adr anti-patterns bets constitution glossary markets patterns requirements rfc runbooks standards)
find_arts() {
  for d in "${ART_DIRS[@]}"; do
    [[ -d "$root/$d" ]] && find "$root/$d" -name '*.md' -type f
  done | sort
}

# The ledger cites every id it reports, so scanning it for references makes its own
# entries self-sustaining: an entry would regenerate forever even after the file that
# originally cited the id was fixed. Worse, check-knowledge.sh treats a dangling
# reference as a hard error, so a self-cited id turns the gate permanently red while
# the ledger is marked do-not-hand-edit. Excluding it is what makes the regenerated
# section actually self-healing, as ADR-0011 states it is.
LEDGER_NAME="DECISION-DEBT.md"

# ── regenerated section ────────────────────────────────────────────────────
regen="$(mktemp)"; trap 'rm -f "$regen"' EXIT

open_decisions=""
while IFS= read -r f; do
  st="$(fm "$f" status)"
  case "$st" in draft|proposed) ;; *) continue ;; esac
  created="$(fm "$f" created)"
  age="?"
  if [[ -n "$created" ]] && created_s=$(midnight "$created") && [[ -n "$created_s" ]]; then
    # Whole days between two MIDNIGHTS, rounded: `date -j -f %Y-%m-%d` alone fills in the current
    # time of day, so "now minus created" straddled a second boundary and floored to a day less
    # (RFC-0021 read 45d beside two same-day RFCs at 46d), and a same-day artifact computed 0 and
    # showed "?". Rounding absorbs a DST day of 23 or 25 hours.
    days=$(( (today_s - created_s + 43200) / 86400 ))
    # A `created` date in the future is a typo or a clock, not an age — show "?" so the ledger still
    # surfaces it as unreadable instead of passing it off as brand new.
    if (( today_s >= created_s )); then age="${days}d"; fi
  fi
  open_decisions+="- **$(fm "$f" id)** \`$st\` for $age — $(fm "$f" title)
  \`${f#"$root"/}\`
"
done < <(find_arts)

# Decisions cited in prose that were never written. A dangling id is almost always a
# decision that got discussed, referenced, and then dropped.
declare -A known
while IFS= read -r f; do
  id="$(fm "$f" id)"; [[ -n "$id" ]] && known["$id"]=1
done < <(find "$root" -name '*.md' -type f -not -path '*/.git/*' 2>/dev/null)

dangling=""
while read -r ref; do
  [[ -n "${known[$ref]:-}" ]] && continue
  src="$(grep -rlE --include='*.md' --exclude="$LEDGER_NAME" "\b$ref\b" "$root" 2>/dev/null | head -1)"
  [[ -n "$src" ]] || continue    # only the ledger cited it; the loop is already closed
  dangling+="- **$ref** referenced in \`${src#"$root"/}\` — no such artifact
"
done < <(grep -rhoE --include='*.md' --exclude="$LEDGER_NAME" '\b(ADR|ANTI|BET|C|GLO|MKT|PAT|REQ|RFC|RUN|STD)-[0-9]{4}\b' "$root" 2>/dev/null | sort -u)

# Locked but unenforced — ADR-0002's visible backlog.
unenforced=""
while IFS= read -r f; do
  [[ "$(fm "$f" status)" == "locked" ]] || continue
  grep -qE '^enforced_by:[[:space:]]*\[\][[:space:]]*$' "$f" || continue
  unenforced+="- **$(fm "$f" id)** is locked but names no check — advisory only
"
done < <(find_arts)

uncommitted=""
if git -C "$root" rev-parse --git-dir >/dev/null 2>&1; then
  # Presence, not a count. A count changes on every response and would churn the
  # ledger into every diff — noise that trains people to stop reading it.
  if git -C "$root" status --porcelain 2>/dev/null | grep -qv 'DECISION-DEBT.md'; then
    uncommitted="- Uncommitted changes in the knowledge repo — governance edits that exist on one disk only
"
  fi
  git -C "$root" remote 2>/dev/null | grep -q . || \
    uncommitted+="- **no git remote configured** — this repository exists on this machine only
"
fi

{
  printf '<!-- REGENERATED: %s — do not hand-edit below; close the loop and the entry disappears -->\n\n' "$today"
  printf '## Open loops\n\n'
  if [[ -z "$open_decisions$dangling$unenforced$uncommitted" ]]; then
    printf 'None. Every decision is recorded, enforced, and committed.\n'
  fi
  [[ -n "$open_decisions" ]] && printf '### Undecided — draft or proposed\n\n%s\n' "$open_decisions"
  [[ -n "$dangling"       ]] && printf '### Referenced but never written\n\n%s\n' "$dangling"
  [[ -n "$unenforced"     ]] && printf '### Locked but unenforced\n\n%s\n' "$unenforced"
  [[ -n "$uncommitted"    ]] && printf '### Durability\n\n%s\n' "$uncommitted"
} > "$regen"

# ── appended section: possible unrecorded decisions ────────────────────────
# Heuristic. Detects decision language in a session that produced no artifact.
# Deliberately noisy in one direction only: a false positive costs one line to
# dismiss, a false negative is exactly the failure this hook exists to prevent.
appended=""
if [[ -n "$transcript" && -f "$transcript" ]]; then
  if ! grep -q "session \`${session:0:8}\`" "$LEDGER" 2>/dev/null; then
    phrases='we (will|should|are going to) use|lets? (go with|use)|decided (on|to)|instead of|rather than|from now on|going forward|the rule is|always .* never'
    hits="$(grep -icE "$phrases" "$transcript" 2>/dev/null || echo 0)"
    # Did this session actually record anything?
    recorded=0
    art_re="$(IFS='|'; printf '%s' "${ART_DIRS[*]}")"
    if git -C "$root" rev-parse --git-dir >/dev/null 2>&1; then
      git -C "$root" log --since="6 hours ago" --name-only --pretty=format: 2>/dev/null \
        | grep -qE "^($art_re)/" && recorded=1
    fi
    find_arts | while IFS= read -r a; do
      [[ -n "$(find "$a" -newermt '-6 hours' 2>/dev/null)" ]] && { echo x; break; }
    done | grep -q . && recorded=1

    if [[ "$hits" -ge 8 && "$recorded" -eq 0 ]]; then
      appended="- [ ] ${today} session \`${session:0:8}\` — ${hits} decision-shaped statements, no artifact created or changed. Was something decided that is not written down?
"
    fi
  fi
fi

# ── research reconciliation (RFC-0027) ─────────────────────────────────────
# The capture half is log-web-sources.sh, which writes every web fetch to research/ as it
# happens. This is the half that makes the log matter: a page this session READ and no
# artifact CITED is an external fact that entered a decision with no provenance, which is
# exactly what RFC-0020 exists to prevent and exactly what RFC-0020 alone cannot detect.
#
# It appends rather than rewrites. knowledge.research_log_is_append_only makes the log
# evidence, and evidence a later process edits is not evidence — so the outcome is a new
# `kind:session` record, never a flipped flag on an existing line.
#
# Uncited fetches land in the APPENDED half of the ledger, never the regenerated half. Only a
# human knows whether a fetch was a dead end or a dropped fact, so these cannot self-heal —
# which ADR-0011 already says is the point of that section.
research=""
RLOG="$root/research/$(date -u +%Y-%m-%d)-${session:0:8}.jsonl"
if [[ -f "$RLOG" ]] && command -v jq >/dev/null 2>&1; then
  # Which artifacts did this session actually touch? Reused verbatim from the recorded-check
  # above rather than recomputed: two definitions of "this session wrote an artifact" would
  # drift, and ADR-0022 is the standing argument against a second site.
  touched=()
  while IFS= read -r a; do
    [[ -n "$(find "$a" -newermt '-6 hours' 2>/dev/null)" ]] && touched+=("${a#"$root"/}")
  done < <(find_arts)

  # Every url already declared in SOME artifact's frontmatter. Substring containment, matching
  # knowledge.external_url_is_a_declared_source at check-knowledge.sh — a logged url and the
  # one written into `sources` differ by a trailing slash often enough that exact equality
  # would report a source that is plainly there.
  declared="$(for a in $(find_arts); do
      awk 'NR==1 && $0!="---"{exit} NR>1 && $0=="---"{exit} {print}' "$a"
    done | grep -oE 'https?://[^ )>"]+' | sed 's/[.,;]$//' | sort -u)"

  fetched_n=0 cited_n=0 uncited=""
  while IFS= read -r u; do
    [[ -n "$u" ]] || continue
    fetched_n=$((fetched_n+1))
    if grep -qF -- "$u" <<<"$declared"; then cited_n=$((cited_n+1)); else uncited+="$u"$'\n'; fi
  done < <(jq -r 'select(.kind == "fetched") | .url' "$RLOG" 2>/dev/null | sort -u)

  # The session record. This is what knowledge.researched_artifacts_cite_sources reads, so it
  # is written whether or not anything went uncited: absence of a record must mean "no
  # reconciliation ran", never "nothing was fetched".
  if (( fetched_n > 0 )); then
    jq -cn --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --arg session "${session:0:8}" \
          --argjson fetched "$fetched_n" --argjson cited "$cited_n" \
          --arg uncited "$uncited" --arg touched "$(printf '%s\n' "${touched[@]:-}")" \
      '{ts:$ts, session:$session, kind:"session", fetched:$fetched, cited:$cited,
        uncited:($uncited | split("\n") | map(select(length > 0))),
        artifacts:($touched | split("\n") | map(select(length > 0)))}' >> "$RLOG" 2>/dev/null || true
  fi

  while IFS= read -r u; do
    [[ -n "$u" ]] || continue
    grep -qF -- "$u" "$LEDGER" 2>/dev/null && continue   # already filed; do not re-file each Stop
    research+="- [ ] ${today} read \`$u\` (session \`${session:0:8}\`) — no artifact declares it in \`sources\`. Cite it under RFC-0020, or tick this if it was a dead end.
"
  done <<<"$uncited"
fi

# ── assemble ───────────────────────────────────────────────────────────────
existing_appended=""
if [[ -f "$LEDGER" ]]; then
  # `grep -E '^- \['` used to live here, and it kept only the FIRST line of each entry --
  # every continuation line was dropped on the next Stop hook. ADR-0011 says this half is
  # "deliberately never auto-cleared", and the hook was not clearing it, it was truncating it.
  # Worse, ADR-0025's `git add -A` commits the human's edit and this regeneration in the same
  # checkpoint, so the loss could never appear as a diff. Found 2026-08-08, after it amputated
  # a seven-line entry into a sentence ending on a comma.
  #
  # Now: start at the first list item, keep everything (continuations included) until the
  # trailing instruction line, then drop trailing blanks so the block re-emits cleanly.
  existing_appended="$(awk '
      /^<!-- APPENDED/ { f = 1; next }
      !f               { next }
      /^Tick an item/  { exit }
      /^- \[/          { p = 1 }
      p                { buf[++n] = $0; if (NF) last = n }
      END              { for (i = 1; i <= last; i++) print buf[i] }
    ' "$LEDGER" || true)"
fi

{
  cat <<'HEADER'
# Decision debt

Open loops in Paridyum's knowledge base. Maintained by the `capture-open-loops` Stop
hook (ADR-0011) and surfaced at the start of every session, so that a decision left
unrecorded is put back in front of the next person to work here — whether or not
anyone remembers it exists.

HEADER
  cat "$regen"
  printf '\n<!-- APPENDED: reviewed and cleared by a human; the hook never removes these -->\n\n'
  printf '## Possible unrecorded decisions\n\n'
  if [[ -z "$existing_appended$appended$research" ]]; then
    printf 'None flagged.\n'
  else
    [[ -n "$existing_appended" ]] && printf '%s\n' "$existing_appended"
    [[ -n "$appended"          ]] && printf '%s' "$appended"
    [[ -n "$research"          ]] && printf '%s' "$research"
    printf '\nTick an item to dismiss it, or record the decision as an ADR and delete the line.\n'
  fi
} > "$LEDGER"

exit 0
