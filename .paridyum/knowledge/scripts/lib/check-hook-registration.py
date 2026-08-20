#!/usr/bin/env python3
"""Report whether .claude/settings.json registers hooks that will actually load (ADR-0027).

The question that matters is not whether the plugin is *declared*. That check existed, passed
throughout, and proved nothing: an audit found `.claude/settings.json` declaring and enabling
`paridyum-platform` while the marketplace had never been added, so ADR-0007 layer 1 had never
denied an edit and the ADR-0025 Stop hook had never fired once. A config file is a statement of
intent.

So this asks a narrower question with a checkable answer: are `PreToolUse` and `Stop` registered,
and does every path they name exist and carry the executable bit? Registration alone would
reproduce the original defect one level in — ADR-0022 found a hook registered and not executable,
which is configured, silent, and indistinguishable from working.

Paths beginning with `$` after project-dir substitution are plugin-root paths
(`${CLAUDE_PLUGIN_ROOT}/...`) and belong to the other channel, which this cannot verify from disk.
They are skipped rather than failed.

  check-hook-registration.py <settings.json> <project-root>

  exit 0  registered, and every local path resolves and is executable
  exit 1  PreToolUse or Stop is not registered
  exit 2  registered, but a path is missing or not executable
"""
import json
import os
import sys

REQUIRED = {"PreToolUse", "Stop"}


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__.strip().splitlines()[-4], file=sys.stderr)
        return 1
    settings, root = sys.argv[1], sys.argv[2]
    try:
        with open(settings, encoding="utf-8") as fh:
            data = json.load(fh)
    except (OSError, json.JSONDecodeError):
        return 1

    hooks = data.get("hooks") or {}
    if not REQUIRED <= set(hooks):
        return 1

    for groups in hooks.values():
        for group in groups:
            for hook in group.get("hooks", []):
                path = hook.get("command", "")
                for var in ("$CLAUDE_PROJECT_DIR", "${CLAUDE_PROJECT_DIR}"):
                    path = path.replace(var, root)
                # A remaining leading `$` is the plugin channel's business, not this one's.
                if path.startswith("$"):
                    continue
                path = path.split()[0] if path else path
                if not (os.path.isfile(path) and os.access(path, os.X_OK)):
                    print(f"not executable or missing: {path}", file=sys.stderr)
                    return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
