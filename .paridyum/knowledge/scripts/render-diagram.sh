#!/usr/bin/env bash
#
# Renders one archify diagram, then removes the only thing in its output that reaches the
# network. This is the generator, not a post-edit: `deliver` alone does not produce an artifact
# this estate may commit, so the two steps together are what "generated" means here, and running
# it twice on the same IR produces the same bytes.
#
#   ./scripts/render-diagram.sh docs/diagrams/change-lifecycle.workflow.json
#   ./scripts/render-diagram.sh --check docs/diagrams/*.json   # is the committed render current?
#
# `--check` renders to a temporary file and compares, exiting non-zero on a mismatch. It lives
# here rather than in the gate so that `strip_external` has ONE home: a second copy of it in
# check-repo.sh would drift, and the day it drifted every diagram would report as stale for a
# reason nobody could find.
#
# WHY THE STRIP EXISTS. archify's template loads JetBrains Mono from fonts.googleapis.com, with
# a `preconnect` to fonts.gstatic.com. It is loaded asynchronously behind `media="print"` and the
# stack falls back to system monospace, so the diagram RENDERS correctly with no network at all —
# the vendor's offline story is sound. What it is not is silent: opening a committed governance
# diagram issues a request to a third party and discloses that someone read it. REQ-0012 §4 asks
# for egress control, ADR-0001 vendors knowledge so a checkout is readable offline, and ADR-0065
# decision 7 requires the output not to reach the network when opened.
#
# THE COST IS THE TYPEFACE. Diagrams render in the system monospace stack instead of JetBrains
# Mono. That is a visible difference and it is worth less than the property it buys.
#
# THE HONEST ALTERNATIVE, NOT TAKEN YET: embed the font as a data URI, or upstream a flag. MIT
# permits a fork and the change is four lines in assets/template.html. Recorded in TASKS T-2.19
# rather than done here, because a forked renderer is a second thing to pin.
#
# THE OUTPUT IS PATH-INDEPENDENT, which is what makes `--check` valid rather than approximately
# valid. Verified 2026-09-05 by delivering the same IR to two different filenames and comparing:
# byte-identical. If that ever stops being true, `--check` starts reporting false staleness and
# this comment is where to look.

set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CHECK=false
if [[ "${1:-}" == "--check" ]]; then CHECK=true; shift; fi
[[ $# -ge 1 ]] || { printf 'usage: render-diagram.sh [--check] <ir.json> [more.json ...]\n' >&2; exit 2; }
A="$("$ROOT/scripts/tool-path.sh" archify)"

strip_external() { # strip_external <html-path>
  # Anchored on the two font hosts BY NAME rather than on `<link>`, so a future template change
  # that adds a DIFFERENT external reference is left in place and caught by the verification
  # below, instead of being silently swallowed by a broad rule.
  python3 -c '
import re, sys, pathlib
p = pathlib.Path(sys.argv[1])
html = p.read_text(encoding="utf-8")
out = re.sub(r"[ \t]*<link[^>]*fonts\.(?:googleapis|gstatic)\.com[^>]*>\n?", "", html)
out = re.sub(r"[ \t]*<noscript>\s*</noscript>\n?", "", out)
# Not an error when nothing matches: it means the vendor stopped shipping the font link, which
# is the outcome this step exists to make unnecessary.
if out != html:
    p.write_text(out, encoding="utf-8")
' "$1"
}

STALE=0
for IR in "$@"; do
  [[ -f "$IR" ]] || { printf 'render-diagram: %s does not exist\n' "$IR" >&2; exit 1; }
  BASE="${IR%.json}"
  KIND="${BASE##*.}"
  OUT="$BASE.html"

  case "$KIND" in
    workflow|sequence|dataflow|lifecycle) ;;
    architecture)
      printf 'render-diagram: %s is an architecture IR. Those are generated from manifests and may not be hand-authored (ADR-0065 decision 2, ADR-0040 decision 2).\n' "$IR" >&2
      exit 1 ;;
    *)
      printf 'render-diagram: cannot infer the kind from %s — the filename must end .<kind>.json (STD-0020 §3)\n' "$IR" >&2
      exit 1 ;;
  esac

  if [[ "$CHECK" == true ]]; then
    if [[ ! -f "$OUT" ]]; then
      printf 'STALE    %s — no committed render to compare against\n' "$OUT" >&2
      STALE=1; continue
    fi
    TMPOUT="$(mktemp -t archify-check-XXXXXX).html"
    "$A" deliver "$KIND" "$IR" "$TMPOUT" --json >/dev/null
    strip_external "$TMPOUT"
    if cmp -s "$TMPOUT" "$OUT"; then
      printf 'current  %s\n' "$OUT"
    else
      printf 'STALE    %s — the committed render does not match its IR; re-run render-diagram.sh on it\n' "$OUT" >&2
      STALE=1
    fi
    rm -f "$TMPOUT"
    continue
  fi

  "$A" deliver "$KIND" "$IR" "$OUT" --json >/dev/null
  strip_external "$OUT"

  # Verified, not assumed. A render step that claims a property and does not check it is the
  # green check this repository has been caught by before.
  # THREE PATTERNS, BECAUSE ONE OF THEM MATCHED WHAT A BROWSER DOES AND THE OTHER TWO DID NOT.
  #
  # The original check was `(src|href)="https?://`, which reads like it covers external origins
  # and covers only the shape an author would type by hand. A browser resolves two more:
  #
  #   url(https://…) or url('//…')   inside a style attribute or a <style> block — an SVG can
  #                                  fetch a background, a font or a filter this way, and none
  #                                  of it involves src= or href=
  #   //host/path                    protocol-relative, which inherits the page's scheme and is
  #                                  a perfectly ordinary external fetch
  #
  # Single quotes and whitespace around `=` are the same class of miss and are covered below.
  # This is a parser differential: the checker and the consumer disagreed about what the
  # document says, and the checker was the one that was wrong.
  EXTERNAL='(src|href|xlink:href)[[:space:]]*=[[:space:]]*.?(https?:)?//|url\([[:space:]]*.?(https?:)?//'
  if grep -qiE "$EXTERNAL" "$OUT"; then
    printf 'render-diagram: %s still reaches an external origin after the strip:\n' "$OUT" >&2
    grep -oiE "$EXTERNAL[^\"'\)[:space:]]*" "$OUT" | sort -u | sed 's/^/  /' >&2
    exit 1
  fi
  printf 'rendered %s (%s, self-contained)\n' "$OUT" "$KIND"
done

exit $STALE
