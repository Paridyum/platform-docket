#!/usr/bin/env bash
# Fail when a dependency ships a build script nobody has reviewed.
#
# WHY THIS EXISTS
#
# pnpm refuses lifecycle scripts unless a package is named in `allowBuilds`, and at
# `strictDepBuilds: true` an install that skips one exits non-zero — so a NEW
# dependency shipping a postinstall stops the world and forces a decision. That is
# the property worth having.
#
# It is unusable in practice for a repository with even one reviewed-and-refused
# package: the refusal itself trips the check, so EVERY install exits 1 — fresh and
# already-satisfied alike — taking every `pnpm exec` and CI step with it. pnpm's
# documented escape hatches do not apply: an `allowBuilds` entry of `false` does not
# silence it (tested, pnpm 11.22.0), and the pnpm 10 key `ignoredBuiltDependencies`
# is no longer read. Only `strictDepBuilds: false` clears it, which turns the signal
# off globally.
#
# So callers set `strictDepBuilds: false` and get the signal back here instead.
# `pnpm ignored-builds` separates the two cases that matter:
#
#   Automatically ignored builds during installation:   <- nobody has reviewed these
#   Explicitly ignored package builds (via allowBuilds): <- reviewed and refused
#
# This asserts the first list is empty. A reviewed refusal stays silent; an
# unreviewed postinstall fails the build, which is what `strictDepBuilds: true`
# was for.
#
# Reviewing a package means deciding whether its script is needed at all. Most
# native packages ship platform binaries as optionalDependencies and the
# postinstall is a fallback for package managers that mishandle them; pnpm is not
# one, so the usual correct answer is `false` — refused — not `true`.
set -euo pipefail

out="$(pnpm ignored-builds 2>&1)"

# Take only the lines under the "Automatically ignored" heading, stopping at the
# next heading or blank line. Parsed rather than pattern-matched on the whole
# output, because "Explicitly ignored" packages are legitimate and matching the
# word `ignored` anywhere would fail on every reviewed refusal.
auto="$(printf '%s\n' "$out" | awk '
  /^Automatically ignored builds/ { inblock=1; next }
  inblock && (/^[A-Za-z]/ || /^$/) { inblock=0 }
  inblock { gsub(/^[[:space:]]+|[[:space:]]+$/, ""); if ($0 != "") print }
')"

if [[ -z "$auto" || "$auto" == "None" ]]; then
  echo "✓ no unreviewed build scripts"
  exit 0
fi

# pnpm cannot answer this without an install, and says so IN the list -- the line
# "Cannot identify as no node_modules found" appears exactly where a package name
# would. Reported as its own failure rather than as a package nobody reviewed,
# because the two need opposite responses: one is "run install first" and the other
# is "decide whether to trust this code". Left unhandled, a check run before
# install accuses a package that does not exist.
if [[ "$auto" == *"no node_modules found"* ]]; then
  # ...UNLESS node_modules is right there, in which case pnpm is describing an EMPTY
  # SET rather than a missing install.
  #
  # Observed 2026-08-23 on pnpm 11.22.0 in lib-ui, whose dependency graph contains no
  # package with a build script at all. pnpm prints the same "Cannot identify as no
  # node_modules found" line for "I could not look" and for "there was nothing to
  # find", and the two need opposite responses. console-docket never hit it because it
  # has a reviewed refusal on record, so pnpm had something to report and said "None".
  #
  # Deciding on the directory rather than on the message is what separates them: if
  # node_modules exists, the install ran and the honest answer is that nothing is
  # unreviewed. A caller with no build scripts anywhere must not fail a check that
  # exists to catch build scripts.
  if [[ -d node_modules ]]; then
    echo "✓ no unreviewed build scripts (none in the dependency graph)"
    exit 0
  fi
  echo "✘ cannot check build scripts: dependencies are not installed" >&2
  echo "    run \`pnpm install --frozen-lockfile\` first; pnpm cannot report" >&2
  echo "    ignored builds without a node_modules to inspect." >&2
  exit 1
fi

echo "✘ dependencies ship build scripts that nobody has reviewed:" >&2
printf '%s\n' "$auto" | sed 's/^/    /' >&2
cat >&2 <<'MSG'

A build script runs arbitrary code at install time with your credentials in the
environment. Decide, per package, and record the decision in pnpm-workspace.yaml:

  1. Check whether the script is needed at all. Most native packages ship their
     platform binaries as optionalDependencies and the postinstall is only a
     fallback for package managers that mishandle them:

       node -e "require('<pkg>'); console.log('loads without postinstall')"

  2. If it loads, record the refusal — this is the usual answer:

       allowBuilds:
         <pkg>: false

  3. Only if it genuinely cannot work, grant it, and say in a comment what you are
     trusting it to run:

       allowBuilds:
         <pkg>: true

Do NOT run `pnpm approve-builds` without recording why, and do NOT reach for
`dangerouslyAllowAllBuilds`. Its name is its documentation.
MSG
exit 1
