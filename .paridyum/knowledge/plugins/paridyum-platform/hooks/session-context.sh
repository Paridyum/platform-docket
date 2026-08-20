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
case "$repo" in
  knowledge)   class="org"      ;;
  platform-*)  class="platform" ;;
  svc-*)       class="service"  ;;
  lib-*)       class="library"  ;;
  tool-*)      class="tool"     ;;
  infra-*)     class="infra"    ;;
  spec-*)      class="spec"     ;;
  template-*)  class="template" ;;
  *)           class=""         ;;
esac

printf '## Paridyum knowledge (injected by paridyum-platform)\n\n'
printf 'Repository: `%s`' "$repo"
[[ -n "$class" ]] && printf ' — class `%s`' "$class"
printf '\nKnowledge base: `%s`\n\n' "${kb/#$HOME/~}"

if [[ -z "$class" ]]; then
  printf '**`%s` does not match the naming scheme in C-0002.** Repositories must be\n' "$repo"
  printf 'prefixed `platform-`, `svc-`, `lib-`, `tool-`, `infra-`, `spec-` or `template-`.\n\n'
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
if [[ "$kb" == "${PARIDYUM_KNOWLEDGE_ROOT:-}" && -n "${PARIDYUM_KNOWLEDGE_ROOT:-}" ]]; then
  printf '### Research capture\n\n'
  printf 'Armed via `PARIDYUM_KNOWLEDGE_ROOT`: web fetches log to `%s/research/` and are\n' "$kb"
  printf 'reconciled against `sources` blocks at Stop (RFC-0027). Cite what a decision rests on.\n\n'
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
