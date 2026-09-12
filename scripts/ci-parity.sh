#!/usr/bin/env bash
# Run the suite the way a CI runner does: no typesetter, no `age`.
#
# D-45 and D-36. Twice now a test has passed on every developer machine and
# failed on a runner, because a fixture reached for a tool the test itself did
# not need -- and the only way to find that out was to push and read the log.
# This makes the difference reproducible locally, in seconds.
#
# It does NOT replace `gates.sh`. A green run here means "nothing depends on a
# tool the runner lacks"; a green run there means "the code is correct". Both
# are needed, and this one is the cheaper half of the pair.
set -euo pipefail

cd "$(dirname "$0")/.."
bare="${TMPDIR:-/tmp}/docket-ci-parity-path"
rm -rf "$bare" && mkdir -p "$bare"

# Only the toolchain. Anything else a test wants must be skipped or not needed.
for tool in cargo rustc rustup; do
  path="$(command -v "$tool" || true)"
  [ -n "$path" ] && ln -sf "$path" "$bare/$tool"
done

echo "running with PATH stripped to the Rust toolchain"
for tool in age age-keygen pdflatex tectonic; do
  if PATH="$bare:/usr/bin:/bin" command -v "$tool" >/dev/null 2>&1; then
    echo "  WARNING: $tool is still reachable; this run is not CI parity"
  fi
done

PATH="$bare:/usr/bin:/bin:/usr/sbin:/sbin" cargo test --quiet
