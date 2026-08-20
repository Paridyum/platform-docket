#!/usr/bin/env bash
#
# Installs the git pre-commit hook — layer 2 of the enforcement ladder (ADR-0007).
#
# Layer 1 (the Claude Code PreToolUse guard) covers work done through an agent.
# This layer covers edits made in an editor, by a script, or by any other tool.
# It is bypassable with `git commit --no-verify`, which is why layers 3 and 4 exist.

set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
HOOK_DIR="$(git -C "$ROOT" rev-parse --git-path hooks)"
HOOK="$HOOK_DIR/pre-commit"

mkdir -p "$HOOK_DIR"

if [[ -e "$HOOK" ]] && ! grep -q 'paridyum knowledge gate' "$HOOK" 2>/dev/null; then
  # Never clobber a hook someone else installed.
  printf 'A pre-commit hook already exists and is not ours:\n  %s\n' "$HOOK" >&2
  printf 'Add this line to it manually:\n  "%s/scripts/check-knowledge.sh" --staged\n' "$ROOT" >&2
  exit 1
fi

cat > "$HOOK" <<EOF
#!/usr/bin/env bash
# paridyum knowledge gate — installed by scripts/install-hooks.sh (ADR-0007)
# test-checks.sh is deliberately absent here: it scaffolds temporary repositories and
# takes seconds, and a slow pre-commit hook is one people bypass with --no-verify. It
# runs in CI, where the cost is invisible.
R="\$(git rev-parse --show-toplevel)"
"\$R/scripts/check-commits.sh" --staged || exit 1
"\$R/scripts/check-knowledge.sh" --staged || exit 1
"\$R/scripts/check-repo.sh" >/dev/null || exit 1
"\$R/scripts/check-migrations.sh" >/dev/null || exit 1
"\$R/templates/infra-ci/scripts/check-changelog.sh" "\$R/CHANGELOG.md" >/dev/null || exit 1
# Was: \`command -v gitleaks >/dev/null && { gitleaks protect --staged ... }\` — which
# silently skipped the scan on any machine without gitleaks installed, and used a
# command deprecated in v8.19.0. secret-scan.sh resolves the binary itself (PATH
# first, else a pinned checksum-verified download), so this cannot no-op, and it is
# the same script CI runs. See C-0004 and ADR-0014 on silent non-enforcement.
"\$R/templates/infra-ci/scripts/secret-scan.sh" --staged >/dev/null || exit 1
exit 0
EOF

chmod +x "$HOOK"
printf 'Installed pre-commit hook: %s\n' "$HOOK"
