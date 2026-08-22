#!/usr/bin/env bash
#
# Build, then serve. In that order, every time.
#
# ## Why this exists
#
# `docket serve` run from a binary somebody built last week serves last week's
# wire contract, and it does so quietly and indefinitely. That happened: a console
# that had begun requiring two new fields refused every response, and the operator
# was shown a redacted React error code, because a browser never receives a Server
# Component's real message. The cause was a binary nobody had rebuilt, and the fix
# on offer was a sentence in a document telling somebody to remember.
#
# A sentence telling somebody to remember is not a fix. This is: there is no
# ordering left to get wrong, because building is not a step you can skip — it is
# how the server starts.
#
# ## Why it refuses an occupied port instead of trying
#
# The usual reason 8080 is taken is a `docket serve` from an earlier session,
# still serving — which is the exact fault this script exists to prevent, wearing
# a different hat. Without this check the script spends a minute building and then
# dies on a bind error, and worse, anything probing the port meanwhile gets an
# answer from the old server and concludes the new one is up. That is not
# hypothetical: it happened while this script was being tested.
#
# Usage:  scripts/serve.sh [--root DIR] [flags for `docket serve`, e.g. --port N]
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"

# `--root` is a global flag and everything else belongs to the `serve` subcommand,
# so they cannot be passed through as one list. Getting this wrong is how the
# first version of this script died with "unexpected argument '--port'".
root=""
serve_args=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --root) root="${2:-}"; shift 2 ;;
    --root=*) root="${1#--root=}"; shift ;;
    *) serve_args+=("$1"); shift ;;
  esac
done

# The port, read back out of the serve flags only so this can check it is free.
# `docket serve` still owns the default; this does not invent one.
port=8080
for ((i = 0; i < ${#serve_args[@]}; i++)); do
  case "${serve_args[i]}" in
    --port) port="${serve_args[i + 1]:-$port}" ;;
    --port=*) port="${serve_args[i]#--port=}" ;;
  esac
done

if nc -z 127.0.0.1 "$port" 2>/dev/null; then
  cat >&2 <<MSG
docket: something is already listening on 127.0.0.1:$port.

  Most likely a \`docket serve\` from an earlier session, still running and still
  serving whatever it was built from. That is the fault this script exists to
  prevent, and starting a second server would not replace it.

  Find it and stop it:
      lsof -nP -iTCP:$port -sTCP:LISTEN
      kill <pid>

  Then run this again.
MSG
  exit 2
fi

# Release, not debug. Issuing a stage's documents renders eleven PDFs through a
# typesetter, and the debug build makes that noticeably slower for the person
# waiting. The first build is minutes; every later one is seconds.
echo "docket: building (release) — this is what stops a stale binary serving" >&2
cargo build --release --quiet

binary="$repo/target/release/docket"
if [[ ! -x "$binary" ]]; then
  echo "docket: cargo build succeeded but $binary is missing" >&2
  exit 2
fi

# The contract this build speaks, printed before it serves, so the number that
# turns up in a console's mismatch screen is one somebody has already seen.
contract="$(grep -oE 'pub const CONTRACT: u32 = [0-9]+' src/adapters/http/dto.rs | grep -oE '[0-9]+$' || true)"
echo "docket: wire contract ${contract:-unknown}, serving on 127.0.0.1:$port" >&2

exec "$binary" ${root:+--root "$root"} serve "${serve_args[@]}"
