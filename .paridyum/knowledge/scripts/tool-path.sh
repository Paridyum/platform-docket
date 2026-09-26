#!/usr/bin/env bash
#
# Resolves a tool's entrypoint from tool-registry.yaml, never from PATH.
#
# ADR-0066 decision 4 pins an install to an immutable ref, and a tool found on PATH is whatever
# the workstation happened to install — which is the drift the registry exists to close, and the
# mechanism that put an end-of-life Node runtime into three repositories. Every script that
# invokes a pinned tool goes through here.
#
#   ./scripts/tool-path.sh archify        prints the entrypoint, exits 0
#   ./scripts/tool-path.sh archify --sha  prints the pinned ref instead
#
# A tool whose version lives elsewhere (`pin_home`) is not resolvable here BY DESIGN: the
# registry holds no pin for it, so there is nothing to resolve to and PATH is the honest answer.
# That is reported rather than silently falling through, because a resolver that quietly returns
# PATH is a resolver that makes the pin optional.

set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS_HOME="${PARIDYUM_TOOLS_HOME:-$HOME/.paridyum/tools}"

die() { printf 'tool-path: %s\n' "$1" >&2; exit 1; }

[[ $# -ge 1 ]] || die "usage: tool-path.sh <tool-id> [--sha]"
ID="$1"; WANT="${2:-}"

command -v python3 >/dev/null || die "python3 is required to read tool-registry.yaml"

read -r REF PATH_IN_TREE PIN_HOME < <(python3 - "$ROOT/tool-registry.yaml" "$ID" <<'PY'
import sys
try:
    import yaml
except ImportError:
    print("! ! !"); sys.exit(0)
reg_p, tid = sys.argv[1], sys.argv[2]
with open(reg_p, encoding="utf-8") as fh:
    reg = yaml.safe_load(fh) or {}
for t in reg.get("tools", []) or []:
    if isinstance(t, dict) and t.get("id") == tid:
        inst = t.get("install") or {}
        print(inst.get("ref", "-"), inst.get("path", "-"), t.get("pin_home", "-"))
        sys.exit(0)
print("? ? ?")
PY
)

[[ "$REF" == "!" ]] && die "pyyaml is unavailable, so no pin can be read — refusing to fall back to PATH"
[[ "$REF" == "?" ]] && die "$ID is not in tool-registry.yaml. Adopt it there first (ADR-0066); a tool with no entry is a decision nobody made"

if [[ "$REF" == "-" ]]; then
  [[ "$PIN_HOME" != "-" ]] \
    && die "$ID is pinned elsewhere (pin_home: $PIN_HOME) and has no install here. Use it from PATH; the version is that file's to state, not this one's"
  die "$ID declares neither install.ref nor pin_home — the registry is inconsistent (knowledge.tool_registry_holds_no_version)"
fi

if [[ "$WANT" == "--sha" ]]; then printf '%s\n' "$REF"; exit 0; fi

DEST="$TOOLS_HOME/$ID/$REF"
[[ -d "$DEST" ]] || die "$ID is not installed at its pinned ref. Run: ./scripts/install-tools.sh $ID"

if [[ "$PATH_IN_TREE" == "-" ]]; then ENTRY="$DEST/$ID"; else ENTRY="$DEST/$PATH_IN_TREE"; fi
[[ -e "$ENTRY" ]] || die "$ID is installed at $REF but its entrypoint is missing: $ENTRY"

# A .mjs entrypoint is not executable on its own, and printing `node <path>` as one string
# breaks the moment a caller's shell does not word-split unquoted expansions -- zsh does not,
# and RUN-0001 names macOS a supported workstation. The installer writes a shim instead, so what
# this prints is always a single executable path and `"$(tool-path.sh x)" args` works in every
# shell. Found by running the documented invocation under zsh and watching it fail.
SHIM="$DEST/.paridyum-shim/$ID"
if [[ -x "$SHIM" ]]; then printf '%s\n' "$SHIM"; exit 0; fi
case "$ENTRY" in
  *.mjs|*.js) die "$ID needs an interpreter shim and none was written. Run: ./scripts/install-tools.sh $ID" ;;
  *)          printf '%s\n' "$ENTRY" ;;
esac
