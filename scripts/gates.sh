#!/usr/bin/env bash
#
# Every gate CI runs, run here, with each exit code reported separately.
#
# ## Why this exists rather than a line in a document
#
# `SOP.md` Task 3 says: capture an exit code from the command itself, never after
# a pipe. That rule is there because it was learned the hard way. It was then
# broken twice more, on the following day, by the person who wrote it — both times
# as `cargo fmt --check 2>&1 | head -20` followed by reading `$?`, which is the
# exit code of `head`. Both times a visible diff was recorded as clean and CI
# caught it on the push.
#
# A lesson in a document does not install itself. This is the same rule expressed
# as something that runs: there is no pipe to read through, because nothing here
# is piped.
#
# ## What it does not do
#
# It does not replace CI. CI is the gate; this is the same set of commands in the
# same order so that a red run is discovered before a push rather than after one.
# If they ever disagree, CI is right and this file is stale — which is a defect in
# this file.
set -uo pipefail   # NOT -e: a failing gate must be recorded, not abort the run.

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

log="$(mktemp -d)"
names=()
codes=()

run() {
  local name="$1"; shift
  printf '  %-14s ' "$name" >&2
  # Redirected to a file rather than piped, which is the entire point of this
  # script: `$?` here belongs to the command and to nothing else.
  "$@" > "$log/$name.out" 2>&1
  local code=$?
  names+=("$name")
  codes+=("$code")
  if [[ $code -eq 0 ]]; then echo "ok" >&2; else echo "FAILED ($code)" >&2; fi
  return 0
}

echo "gates:" >&2
run fmt     cargo fmt --check
run clippy  cargo clippy --all-targets -- -D warnings
run test    cargo test
# ADDED 2026-09-20, because this script was missing a gate CI runs and a change
# passed here and failed there. CI's `docs` job sets RUSTDOCFLAGS and a broken
# intra-doc link — `[\`refuse_without_backups\`]`, a function that had been
# renamed — failed rustdoc while fmt, clippy and tests were all green locally.
#
# The point of this script is that green here means green there. A gate CI runs
# and this does not makes it a subset that reads like a superset, which is the
# same shape as a check that passes while enforcing nothing.
run docs    env RUSTDOCFLAGS="-D warnings -D rustdoc::broken_intra_doc_links" cargo doc --no-deps
run check   cargo run --quiet -- check

failed=0
for i in "${!names[@]}"; do
  if [[ "${codes[i]}" -ne 0 ]]; then
    failed=1
    echo >&2
    echo "--- ${names[i]} ---" >&2
    cat "$log/${names[i]}.out" >&2
  fi
done

# `docket check` exits 0 on warnings, so a warning count is reported rather than
# being invisible behind a zero. A registry that has started warning is something
# to look at even though nothing is broken yet.
warnings="$(grep -oE '[0-9]+ warnings' "$log/check.out" 2>/dev/null | tail -1 || true)"
if [[ -n "$warnings" && "$warnings" != "0 warnings" ]]; then
  echo >&2
  echo "note: docket check passed with $warnings — see the list above the summary" >&2
fi

rm -rf "$log"
if [[ $failed -ne 0 ]]; then
  echo >&2
  echo "gates: at least one gate failed. Nothing has been pushed." >&2
  exit 1
fi
echo "gates: all green" >&2
