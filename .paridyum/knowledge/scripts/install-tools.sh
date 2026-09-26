#!/usr/bin/env bash
#
# Installs pinned tools from tool-registry.yaml.
#
# The standing rule is that anything installed by hand is a reproducibility debt, and ADR-0066
# decision 4 makes the pin immutable so that "installed" means one thing. This script is the
# declarative half: it reads the registry, clones or fetches at the pinned ref, and verifies.
# It decides nothing -- a version changed here and not in the registry is a bug in this script.
#
#   ./scripts/install-tools.sh           install every tool this file can install
#   ./scripts/install-tools.sh archify   install one
#   ./scripts/install-tools.sh --check   report what is missing, install nothing
#
# TOOLS WITH `pin_home` ARE SKIPPED AND SAID SO, never silently. Their version belongs to
# another file (toolchain-registry.yaml) or to the OS package manager, and installing them from
# here would be the second home for a fact that RFC-0015 gives one. `pin_home: os` is an
# admission rather than a pin -- brew on two workstations on two days installs two versions and
# nothing compares them -- and printing the skip is what keeps that visible.

set -euo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
TOOLS_HOME="${PARIDYUM_TOOLS_HOME:-$HOME/.paridyum/tools}"
CHECK_ONLY=false
if [[ "${1:-}" == "--check" ]]; then CHECK_ONLY=true; shift; fi
ONLY="${1:-}"

if [[ -t 1 ]]; then RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; RST=$'\033[0m'
else RED=""; GRN=""; YLW=""; RST=""; fi
ok()  { printf '%s✔%s %s\n' "$GRN" "$RST" "$1"; }
skip(){ printf '%s·%s %s\n' "$YLW" "$RST" "$1"; }
die() { printf '%s✘%s %s\n' "$RED" "$RST" "$1" >&2; exit 1; }

command -v python3 >/dev/null || die "python3 is required to read tool-registry.yaml"
command -v git     >/dev/null || die "git is required"

ENTRIES="$(python3 - "$ROOT/tool-registry.yaml" <<'PY'
import sys
try:
    import yaml
except ImportError:
    sys.exit(2)
with open(sys.argv[1], encoding="utf-8") as fh:
    reg = yaml.safe_load(fh) or {}
for t in reg.get("tools", []) or []:
    if not isinstance(t, dict):
        continue
    inst = t.get("install") or {}
    print("\t".join([
        t.get("id", "?"),
        inst.get("method", "-"),
        inst.get("ref", "-"),
        inst.get("path", "-"),
        t.get("source", "-"),
        t.get("pin_home", "-"),
    ]))
PY
)" || die "pyyaml is unavailable — refusing to install from a registry that cannot be parsed"

# A .mjs entrypoint is not executable and `node <path>` cannot be passed around as one word --
# zsh does not word-split unquoted expansions, and RUN-0001 names macOS a supported workstation,
# so the "obvious" `$(tool-path.sh archify) validate ...` fails there and nowhere else. The shim
# makes every tool resolve to a single executable path regardless of what runs it.
write_shim() { # write_shim <id> <dest> <path-in-tree>
  local id="$1" dest="$2" rel="$3" entry shim
  [[ "$rel" == "-" ]] && return 0
  entry="$dest/$rel"
  case "$entry" in *.mjs|*.js) ;; *) return 0 ;; esac
  shim="$dest/.paridyum-shim/$id"
  mkdir -p "$(dirname "$shim")"
  printf '#!/usr/bin/env bash\nexec node %q "$@"\n' "$entry" > "$shim"
  chmod +x "$shim"
}

MISSING=0
while IFS=$'\t' read -r id method ref path source pin_home; do
  [[ -n "$id" ]] || continue
  [[ -z "$ONLY" || "$ONLY" == "$id" ]] || continue

  if [[ "$ref" == "-" ]]; then
    skip "$id — pinned elsewhere (pin_home: ${pin_home}); install it the way that file says"
    continue
  fi

  if [[ "$ref" == "latest-unpinned" ]]; then
    skip "$id — declares an unpinned install and is deliberately not installed from here (see DECISION-DEBT.md)"
    continue
  fi

  DEST="$TOOLS_HOME/$id/$ref"
  if [[ -d "$DEST/.git" ]]; then
    write_shim "$id" "$DEST" "$path"
    ok "$id already at $ref"
    ln -sfn "$DEST" "$TOOLS_HOME/$id/current"
    continue
  fi

  MISSING=$((MISSING+1))
  if [[ "$CHECK_ONLY" == true ]]; then
    printf '%s·%s %s missing at %s\n' "$YLW" "$RST" "$id" "$ref"
    continue
  fi

  case "$method" in
    git)
      [[ "$source" != "-" ]] || die "$id: install.method is git but no source is declared"
      mkdir -p "$TOOLS_HOME/$id"
      TMP="$DEST.partial.$$"
      rm -rf "$TMP"
      git clone --quiet "$source" "$TMP" || die "$id: clone failed from $source"
      git -C "$TMP" checkout --quiet "$ref" \
        || { rm -rf "$TMP"; die "$id: pinned ref $ref does not exist in $source"; }
      # Verified before it is visible: a half-installed tree under the pinned path is
      # indistinguishable from a good one to every later caller.
      got="$(git -C "$TMP" rev-parse HEAD)"
      [[ "$got" == "$ref" ]] || { rm -rf "$TMP"; die "$id: checked out $got, expected $ref"; }
      mv "$TMP" "$DEST"
      write_shim "$id" "$DEST" "$path"
      ln -sfn "$DEST" "$TOOLS_HOME/$id/current"
      ok "$id installed at $ref"
      ;;
    *)
      die "$id: install.method '$method' is declared in the registry and not implemented here — add it rather than installing by hand"
      ;;
  esac
done <<<"$ENTRIES"

# The verification step, run only for tools that carry one. `doctor` is archify's; a tool
# without a self-check gets none rather than a fabricated one.
if [[ -z "$ONLY" || "$ONLY" == "archify" ]]; then
  if A="$("$ROOT/scripts/tool-path.sh" archify 2>/dev/null)"; then
    if $A doctor >/dev/null 2>&1; then ok "archify doctor: ready"
    else die "archify is installed at its pin and its own doctor reports a problem"; fi
  fi
fi

[[ "$CHECK_ONLY" == true && $MISSING -gt 0 ]] && exit 1
exit 0
