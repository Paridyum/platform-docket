#!/usr/bin/env bash
#
# SessionStart hook — injects the rules that bind THIS repository into context
# before the first prompt. Anything printed to stdout is added to Claude's context.
#
# This is the retrieval half of ADR-0007. The guard hook stops violations; this
# stops them from being attempted, by making the applicable rules present rather
# than something the agent has to remember to go and read.
#
# Deliberately prints ids, titles and paths — not full artifact bodies. The point
# is a small, deterministic index the agent can read selectively, not a context dump.

set -uo pipefail

payload="$(cat 2>/dev/null || true)"
cwd="$(jq -r '.cwd // empty' <<<"$payload" 2>/dev/null || true)"
[[ -z "$cwd" ]] && cwd="$PWD"

# Locate the knowledge base: the vendored snapshot in a service repo, or the
# canonical repo when working in knowledge itself.
kb=""
for candidate in "$cwd/.paridyum/knowledge" "$cwd"; do
  if [[ -d "$candidate/constitution" && -d "$candidate/adr" ]]; then kb="$candidate"; break; fi
done
if [[ -z "$kb" ]]; then
  d="$cwd"
  while [[ "$d" != "/" && -n "$d" ]]; do
    if [[ -d "$d/.paridyum/knowledge/adr" ]]; then kb="$d/.paridyum/knowledge"; break; fi
    d="$(dirname "$d")"
  done
fi
# LAST RESORT: the root named explicitly. Without this a session started in a PARENT of the
# canonical repository — how this repository is usually worked on — finds nothing: the upward
# walk above looks only for a vendored `.paridyum/knowledge`, never for the canonical checkout.
# The same blind spot that made log-web-sources.sh capture nothing on 2026-08-19 (RFC-0027
# consequence 5), in the hook that would otherwise have been the place to notice.
if [[ -z "$kb" && -n "${PARIDYUM_KNOWLEDGE_ROOT:-}" ]]; then
  [[ -d "$PARIDYUM_KNOWLEDGE_ROOT/constitution" && -d "$PARIDYUM_KNOWLEDGE_ROOT/adr" ]] \
    && kb="$PARIDYUM_KNOWLEDGE_ROOT"
fi
[[ -z "$kb" ]] && exit 0   # not a Paridyum repo; stay silent

repo="$(basename "$cwd")"
# THE CLASS IS THE REGISTRY'S, read from the knowledge base this hook just resolved — never a list
# copied here (C-0007: the grammar is constitutional, the list is data). This was a case statement
# holding C-0002's prefixes, so every `web-`, `console-`, `mobile-`, `agent-` and `sdk-` repository
# was told at session start that its name broke the scheme, and shown every locked artifact
# unfiltered. Found while adding `game-` (ADR-0133). The value is the class's `scope`, because
# that is what an artifact's `scope` names.
class=""
if [[ "$repo" == "knowledge" ]]; then
  class="org"
elif [[ -f "$kb/class-registry.yaml" ]]; then
  class="$(awk -v r="$repo" '
    /^  - class: /  { prefix = ""; scope = "" }
    /^    prefix: / { prefix = $2 }
    /^    scope: /  { scope = $2 }
    prefix != "" && scope != "" && index(r, prefix) == 1 { print scope; exit }
  ' "$kb/class-registry.yaml")"
fi

printf '## Paridyum knowledge (injected by paridyum-platform)\n\n'
printf 'Repository: `%s`' "$repo"
[[ -n "$class" ]] && printf ' — class `%s`' "$class"
printf '\nKnowledge base: `%s`\n\n' "${kb/#$HOME/~}"

if [[ -z "$class" ]]; then
  prefixes="$(awk '/^    prefix: / { printf "%s`%s`", sep, $2; sep = ", " }' "$kb/class-registry.yaml" 2>/dev/null)"
  printf '**`%s` does not match the naming scheme in C-0007.** A repository name begins with a\n' "$repo"
  printf 'prefix from `class-registry.yaml`: %s.\n\n' "${prefixes:-none could be read from this snapshot}"
fi

# --- CI that cannot run says so (RFC-0060 D5) --------------------------------------------------
# On 2026-10-07 every self-hosted runner went offline and `main` sat queued for more than a day
# with nobody told. The channel is this summary — the place ADR-0059 decision 3 already makes the
# one the operator reads — chosen by the operator on 2026-10-08. GitHub is asked with the
# operator's own `gh` login, so no secret is added, and every call is bounded so a session never
# waits on the network: when `gh` or `perl` is absent, unauthorised, slow or offline the summary
# says nothing, because an unanswered question is not news. PARIDYUM_CI_HEALTH=0 turns it off —
# the check suite does, to stay offline.
bounded() { perl -e 'alarm shift; exec @ARGV' "$@" 2>/dev/null; }
if [[ "${PARIDYUM_CI_HEALTH:-1}" != "0" ]] && command -v gh >/dev/null 2>&1 \
   && command -v perl >/dev/null 2>&1; then
  read -r ci_online ci_total < <(bounded 4 gh api orgs/Paridyum/actions/runners \
    --jq '"\([.runners[] | select(.status == "online")] | length) \(.runners | length)"' || true)
  if [[ "${ci_total:-}" =~ ^[0-9]+$ && "${ci_online:-}" =~ ^[0-9]+$ && "$ci_total" -gt 0 ]]; then
    if [[ "$ci_online" -eq 0 ]]; then
      printf '**CI cannot run:** none of the organisation'"'"'s %s self-hosted runners is online, so every\n' "$ci_total"
      printf 'pull request and push waits in the queue unverified; until they are back, the gates run only\n'
      printf 'where someone runs them locally (RFC-0060 D5).\n\n'
    else
      # Runners online but `main` still queued: the queue itself is stuck. One hour, because a
      # healthy run here starts within minutes.
      queued_at="$( (cd "$cwd" && bounded 4 gh run list --branch main --status queued --limit 1 \
        --json createdAt --jq '.[0].createdAt // empty') || true)"
      if [[ -n "$queued_at" ]]; then
        queued_s="$(date -j -u -f '%Y-%m-%dT%H:%M:%SZ' "$queued_at" +%s 2>/dev/null \
                    || date -u -d "$queued_at" +%s 2>/dev/null)"
        if [[ "$queued_s" =~ ^[0-9]+$ ]] && (( $(date -u +%s) - queued_s > 3600 )); then
          printf '**CI is stuck:** this repository'"'"'s run on main has waited in the queue since %s\n' "$queued_at"
          printf 'while runners are online (RFC-0060 D5).\n\n'
        fi
      fi
    fi
  fi
fi

# Frontmatter reader: stops at the closing delimiter, so prose containing the word
# "locked" cannot produce a false match.
fm() { awk -v k="^$2:" 'NR==1 && $0!="---"{exit} NR>1 && $0=="---"{exit} $0~k{sub(/^[^:]*:[ \t]*/,""); print; exit}' "$1" 2>/dev/null; }

emit_section() {
  local dir="$1" heading="$2" found=0 out=""
  [[ -d "$kb/$dir" ]] || return 0
  while IFS= read -r f; do
    [[ -f "$f" ]] || continue
    local st id ti sc
    st="$(fm "$f" status)"
    [[ "$st" == "locked" ]] || continue
    sc="$(fm "$f" scope)"
    # scope is an inline YAML list: [org, service, ...]
    if [[ -n "$class" && -n "$sc" && "$sc" != *"$class"* && "$sc" != *"org"* ]]; then continue; fi
    id="$(fm "$f" id)"; ti="$(fm "$f" title)"
    out+="$(printf -- '- **%s** — %s\n  `%s`\n' "$id" "$ti" "${f#$kb/}")"$'\n'
    found=1
  done < <(find "$kb/$dir" -name '*.md' -type f 2>/dev/null | sort)
  [[ $found -eq 1 ]] && { printf '%s\n' "$heading"; printf '%s\n' "$out"; }
  return 0
}

emit_section constitution "### Constitution — highest precedence, binding"
emit_section adr          "### Locked decisions binding this repository"

# ACCEPTED DECISIONS FOR THIS REPOSITORY — direction, one line each, newest first.
#
# Until 2026-10-04 a session saw only locked artifacts, and ADR-0056 ("host configuration is
# Ansible") was accepted, not locked: no session in any infra repository was ever shown it, and
# infra-terraform's runner was configured by scripts for five days (docs/2026-10-03-why-adr-0056-was-
# not-followed.md, cause 1). Accepted is direction, not law (ADR-0002), but direction an agent cannot
# see is direction nobody follows.
#
# Narrowed twice so it stays readable: the ADR's `scope` must NAME this repository's class (org-wide
# ones would put every decision in every session), and its `applies_to`, when it is not "*", must
# match this repository's name. Newest first, because a recent decision is the one least likely to be
# known; capped, with the count of the rest and how to list them.
emit_accepted() {
  local cap=25 n=0 more=0 out="" f st sc ap id ti matched
  [[ -n "$class" && -d "$kb/adr" ]] || return 0
  while IFS= read -r f; do
    st="$(fm "$f" status)"; [[ "$st" == "accepted" ]] || continue
    sc="$(fm "$f" scope)";  [[ "$sc" == *"$class"* ]] || continue
    ap="$(fm "$f" applies_to)"
    if [[ -n "$ap" && "$ap" != *'"*"'* ]]; then
      matched=0
      for pat in $(tr -d '[]",' <<<"$ap"); do
        # shellcheck disable=SC2053  # pat is a glob on purpose ("infra-*")
        [[ "$repo" == $pat ]] && { matched=1; break; }
      done
      [[ $matched -eq 1 ]] || continue
    fi
    if (( n < cap )); then
      id="$(fm "$f" id)"; ti="$(fm "$f" title)"
      out+="- **${id}** — ${ti}"$'\n'; n=$((n+1))
    else
      more=$((more+1))
    fi
  done < <(find "$kb/adr" -name 'ADR-*.md' -type f 2>/dev/null | sort -r)
  (( n > 0 )) || return 0
  printf '### Accepted decisions for this repository — direction, newest first\n\n%s' "$out"
  (( more > 0 )) && printf -- '- … and %d more: `grep -l "status: accepted" %s/adr/*.md`\n' "$more" "${kb/#$HOME/~}"
  printf '\nRead the one that matches what you are about to do. Accepted is not locked, but code that\ncontradicts it needs a decision, not a pattern copied from a neighbouring file.\n\n'
}
emit_accepted

# Open loops carried over from previous sessions (ADR-0011). This is the half of the
# anti-forgetting mechanism that does the remembering: the Stop hook records, this
# puts it back in front of whoever starts next.
for ledger in "$cwd/DECISION-DEBT.md" "$kb/DECISION-DEBT.md"; do
  [[ -f "$ledger" ]] || continue
  loops="$(awk '/^## Open loops/{f=1;next} /^<!-- APPENDED/{f=0} f' "$ledger" | grep -E '^- ' || true)"
  flagged="$(grep -E '^- \[ \]' "$ledger" 2>/dev/null || true)"
  if [[ -n "$loops$flagged" ]]; then
    printf '### Carried over — open loops\n\n'
    [[ -n "$loops"   ]] && printf '%s\n\n' "$loops"
    if [[ -n "$flagged" ]]; then
      printf '**Possibly decided but never written down:**\n\n%s\n\n' "$flagged"
    fi
    printf 'Full ledger: `%s`. If one of these is what you are about to work on, close it.\n\n' "${ledger#"$cwd"/}"
  fi
  break
done

# ── research capture armed? (RFC-0027 open question 2) ─────────────────────
# Confirms, in the session summary, where web fetches will be logged — so "nothing appeared in
# research/" can be read as "nothing was fetched" rather than "capture was never running".
#
# WHAT THIS CANNOT DO, and open question 2 is narrowed rather than closed. The unarmed case is
# a session started outside the repository with PARIDYUM_KNOWLEDGE_ROOT unset — and there this
# hook has already exited at the guard above, because nothing identifies the session as a
# Paridyum one. Warning from a directory that has no connection to this repository would fire
# in every unrelated project on the machine, which is how a warning becomes noise and then
# becomes ignored. The hole is real and it is not closable here: the signal has to come from
# somewhere that knows the intent, and the only such thing is the variable itself.
#
# ARMED IS A FACT ABOUT THE MACHINE, NOT ABOUT A VARIABLE (ADR-0059 decision 3). This block used
# to report "armed" whenever PARIDYUM_KNOWLEDGE_ROOT pointed at the knowledge base — which is a
# statement of intent, and on 2026-08-31 it would have been false: the variable was set correctly,
# and the INSTALLED plugin cache had no log-web-sources.sh in it at all, so nothing was capturing.
# An assurance derived from configuration is the intent-versus-installation conflation ADR-0007's
# own note names. The hook script either exists in the cache or it does not.
capture_hook=""
if [[ -n "${CLAUDE_PLUGIN_ROOT:-}" && -f "${CLAUDE_PLUGIN_ROOT}/hooks/log-web-sources.sh" ]]; then
  capture_hook="${CLAUDE_PLUGIN_ROOT}/hooks/log-web-sources.sh"
fi
if [[ "$kb" == "${PARIDYUM_KNOWLEDGE_ROOT:-}" && -n "${PARIDYUM_KNOWLEDGE_ROOT:-}" ]]; then
  printf '### Research capture\n\n'
  if [[ -n "$capture_hook" ]]; then
    printf 'Armed: `log-web-sources.sh` is present in the INSTALLED plugin and the knowledge root\n'
    printf 'resolves, so web fetches log to `%s/research/` and are reconciled against\n' "$kb"
    printf '`sources` blocks at Stop (RFC-0027). Cite what a decision rests on.\n\n'
  else
    printf '**NOT armed.** `PARIDYUM_KNOWLEDGE_ROOT` resolves, but `log-web-sources.sh` is absent\n'
    printf 'from the installed plugin — so no web fetch this session will be logged, and an empty\n'
    printf '`research/` will mean nothing (RFC-0027, ADR-0059). The installed plugin is a pinned\n'
    printf 'snapshot; run `scripts/check-repo.sh` for the staleness reading, then:\n\n'
    printf '    /plugin update paridyum-platform@paridyum\n\n'
  fi
fi

cat <<'EOF'
### Rules of engagement

- **Never infer architecture. Retrieve it.** If a naming, structural, deployment or
  API question has a settled answer above, read it rather than deciding again.
- **Locked artifacts must not be contradicted.** Not temporarily, not "just here".
  If the task requires it, stop and say which artifact is in the way.
- **Existing code is the lowest authority.** Code contradicting a locked artifact is
  a bug in the code, not an amendment to the rule. Do not copy a neighbouring file
  and assume it is compliant.
- `.paridyum/knowledge/` is a read-only snapshot. Changes belong in `Paridyum/knowledge`.

Edits violating the first two are blocked by a PreToolUse hook, not merely discouraged.
EOF
