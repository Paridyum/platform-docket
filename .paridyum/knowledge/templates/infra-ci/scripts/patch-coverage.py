#!/usr/bin/env python3
"""Enforce coverage on lines this PR added (STD-0001).

Patch coverage is stricter than total coverage on purpose: new code carries the
burden, not the inherited backlog. A repository with 40% total coverage can still
merge well-tested changes, and the total rises on its own.

Usage:  patch-coverage.py <base-sha> <threshold-pct> [lcov-path]
Exit:   0 pass or nothing to measure, 1 below threshold, 2 bad input.
"""

from __future__ import annotations

import collections
import re
import subprocess
import sys
from pathlib import Path


def added_lines(base: str) -> dict[str, set[int]]:
    """Line numbers added per file, from a zero-context diff against the merge base."""
    out = subprocess.run(
        ["git", "diff", "-U0", f"{base}...HEAD", "--", "*.rs"],
        capture_output=True, text=True, check=True,
    ).stdout
    added: dict[str, set[int]] = collections.defaultdict(set)
    current: str | None = None
    for line in out.splitlines():
        if line.startswith("+++ b/"):
            current = line[6:]
        elif line.startswith("@@") and current:
            m = re.search(r"\+(\d+)(?:,(\d+))?", line)
            if m:
                start, count = int(m.group(1)), int(m.group(2) or 1)
                added[current].update(range(start, start + count))
    return added


def lcov_hits(path: Path) -> dict[str, dict[int, int]]:
    """{source file: {line: execution count}} from an lcov tracefile."""
    hits: dict[str, dict[int, int]] = {}
    current: str | None = None
    for line in path.read_text().splitlines():
        if line.startswith("SF:"):
            current = line[3:].strip()
            hits.setdefault(current, {})
        elif line.startswith("DA:") and current:
            parts = line[3:].strip().split(",")
            if len(parts) >= 2:
                hits[current][int(parts[0])] = int(parts[1])
    return hits


def is_test_path(path: str) -> bool:
    return "/tests/" in path or path.endswith(("_test.rs", "/tests.rs"))


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__, file=sys.stderr)
        return 2
    base, threshold = sys.argv[1], float(sys.argv[2])
    lcov = Path(sys.argv[3] if len(sys.argv) > 3 else "lcov.info")

    if not lcov.exists():
        print(f"::error::{lcov} not found — did the coverage step run?", file=sys.stderr)
        return 2

    added = {f: l for f, l in added_lines(base).items() if not is_test_path(f)}
    if not added:
        print("No non-test Rust lines added; patch coverage not applicable.")
        return 0

    hits = lcov_hits(lcov)
    covered = uncovered = 0
    uncovered_detail: list[str] = []

    for file, lines in added.items():
        # lcov paths may be absolute or workspace-relative; match on suffix.
        key = next((k for k in hits if k == file or k.endswith("/" + file)), None)
        if key is None:
            continue  # not instrumented (e.g. a binary excluded from coverage)
        for line in sorted(lines):
            count = hits[key].get(line)
            if count is None:
                continue  # not an executable line: comment, blank, brace
            if count > 0:
                covered += 1
            else:
                uncovered += 1
                if len(uncovered_detail) < 20:
                    uncovered_detail.append(f"{file}:{line}")

    total = covered + uncovered
    if total == 0:
        print("No instrumented lines among the added lines.")
        return 0

    pct = 100.0 * covered / total
    print(f"Patch coverage: {pct:.1f}% ({covered}/{total} added lines), threshold {threshold}%")

    if uncovered_detail:
        print("\nUncovered added lines:")
        for loc in uncovered_detail:
            print(f"  {loc}")
        if uncovered > len(uncovered_detail):
            print(f"  … and {uncovered - len(uncovered_detail)} more")

    if pct < threshold:
        print(f"::error::patch coverage {pct:.1f}% is below the {threshold}% "
              f"threshold required by STD-0001")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
