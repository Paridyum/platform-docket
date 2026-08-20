#!/usr/bin/env bash
#
# PostToolUse hook — the capture half of RFC-0020 (RFC-0027).
#
# RFC-0020 governs where an external fact LANDS: a `sources` block on the artifact whose
# decision rests on it, carrying the date it was read. It governs nothing about how the fact
# gets there. Between reading a vendor's documented limit on a web page and writing it into
# frontmatter sits exactly one mechanism: an agent remembering. That is the same posture
# ADR-0011 exists because nobody trusts, and RFC-0020 concedes it — "the checks here catch the
# mechanical half only".
#
# This is the other half. Every web fetch is written to an append-only log at the moment it
# happens, before anyone decides whether it mattered. The Stop hook then reconciles the log
# against the `sources` blocks the session actually produced, and what went uncited becomes an
# open loop rather than nothing at all.
#
# TWO KINDS, AND THE DIFFERENCE IS LOAD-BEARING:
#
#   fetched    a page this session actually read. Reconciles. Uncited ones are reported.
#   surfaced   a URL a search RESULT offered and nothing opened. Logged, never reconciled.
#
# Without that split a single WebSearch deposits ten unclaimed URLs into DECISION-DEBT.md, and
# a ledger nobody finishes reading is a ledger that stops working — which is the one failure
# mode ADR-0011 cannot survive. Surfaced URLs are still recorded, because "what was looked at
# and not used" is the other half of an honest research trail.
#
# `.jsonl`, NOT `.log`. .gitignore excludes `*.log`, and ADR-0025 commits with `git add -A`, so
# a research log named that way would never be committed and would exist on one disk only —
# precisely the loss the checkpoint hook was written in response to. Do not "tidy" the
# extension.
#
# Exits 0 unconditionally. A PostToolUse hook that fails must never disrupt the session, and a
# provenance record is worth nothing if it can break the work it is recording.

set -uo pipefail

payload="$(cat 2>/dev/null || true)"
command -v jq >/dev/null 2>&1 || exit 0
jq -e . >/dev/null 2>&1 <<<"$payload" || exit 0

tool="$(jq -r '.tool_name // empty' <<<"$payload")"
case "$tool" in WebSearch|WebFetch) ;; *) exit 0 ;; esac

cwd="$(jq -r '.cwd // empty' <<<"$payload")"; [[ -z "$cwd" ]] && cwd="$PWD"
session="$(jq -r '.session_id // "unknown"' <<<"$payload")"

# Same repo detection the other three hooks use. RFC-0020 scoped itself to knowledge/ and
# deferred the reach-beyond question until a service exists that asserts a vendor capability;
# this stays inside that scope rather than widening a decision nobody has taken.
root="$cwd"
while [[ "$root" != "/" && -n "$root" ]]; do
  [[ -d "$root/constitution" && -d "$root/adr" ]] && break
  root="$(dirname "$root")"
done

# THE UPWARD WALK MISSES THE COMMONEST CASE, and it was found by using this hook rather than by
# reading it. Detection climbs from cwd, so a session started in the PARENT of the knowledge
# repository — ~/Developer, with the repo at ~/Developer/paridyum/knowledge — finds nothing and
# captures nothing, even when every fetch in the session is research about this repository. That
# is exactly what happened on 2026-08-19 while verifying RFC-0027's own open question 1: two
# pages were read, the decision rests on both, and neither was logged.
#
# Guessing downward is not the fix — a parent directory may hold several checkouts, and a hook
# that picks one is a hook that silently files evidence against the wrong repository. The root is
# named explicitly instead, or capture does not happen and says nothing.
if [[ ! -d "$root/adr" ]]; then
  root="${PARIDYUM_KNOWLEDGE_ROOT:-}"
  [[ -n "$root" && -d "$root/adr" && -d "$root/constitution" ]] || exit 0
fi

today="$(date -u +%Y-%m-%d)"
ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
mkdir -p "$root/research" 2>/dev/null || exit 0
LOG="$root/research/${today}-${session:0:8}.jsonl"

# One JSON object per line, built by jq so a URL containing a quote cannot corrupt the record.
#
# THERE IS NO `cited` FIELD. An earlier draft carried `cited:false` for the Stop hook to flip,
# and that is a contradiction: knowledge.research_log_is_append_only makes this file evidence,
# and evidence a later process rewrites is not evidence. Reconciliation APPENDS a `kind:session`
# record instead — what was fetched, what was cited, what was not, and which artifacts the
# session touched. The outcome is derived from the log rather than written back into it.
emit() { # emit <kind> <url> <title> <query>
  jq -cn --arg ts "$ts" --arg session "${session:0:8}" --arg tool "$tool" \
        --arg kind "$1" --arg url "$2" --arg title "$3" --arg query "$4" \
    '{ts:$ts, session:$session, tool:$tool, kind:$kind, url:$url,
      title:(if $title == "" then null else $title end),
      query:(if $query == "" then null else $query end)}' >> "$LOG" 2>/dev/null || true
}

case "$tool" in
  WebFetch)
    url="$(jq -r '.tool_input.url // empty' <<<"$payload")"
    [[ -n "$url" ]] || exit 0
    # `title` is deliberately left null. The hook sees a tool payload, not a parsed page, and a
    # synthesised title in a provenance record is worse than a blank the author must fill:
    # RFC-0020 requires the title to be readable WITHOUT following the link, which a guess is
    # not. The reconciler reports the blank; it does not invent one.
    emit fetched "$url" "" "$(jq -r '.tool_input.prompt // empty' <<<"$payload")"
    ;;
  WebSearch)
    query="$(jq -r '.tool_input.query // empty' <<<"$payload")"
    # tool_response has no stable shape across versions, so it is flattened to text and scanned
    # rather than addressed by path. A missed URL costs one unlogged surfaced link; a hard
    # dependency on an undocumented shape costs the whole record on the next upgrade.
    while read -r u; do
      [[ -n "$u" ]] && emit surfaced "$u" "" "$query"
    done < <(jq -r '.tool_response // empty | tostring' <<<"$payload" 2>/dev/null \
             | grep -oE 'https?://[^ ")>,\\]+' | sed 's/[.,;]$//' | sort -u | head -40)
    ;;
esac

# The nudge. The moment the agent knows WHY it fetched something is now, not at Stop — so the
# reminder is returned into context here, where `supports` is still cheap to write.
#
# additionalContext is best-effort: a harness that does not support it ignores this object, the
# log is written either way, and the Stop-hook reconciliation still reports what went uncited.
# The record never depends on the nudge landing.
if [[ "$tool" == WebFetch ]]; then
  jq -cn --arg m "Logged to research/ — if a decision rests on this page, cite it in the artifact's \`sources\` block with url, title and retrieved (RFC-0020). If it was a dead end, ignore this; the Stop hook will file it for dismissal." \
    '{hookSpecificOutput:{hookEventName:"PostToolUse", additionalContext:$m}}'
fi

exit 0
