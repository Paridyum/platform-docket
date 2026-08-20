#!/usr/bin/env python3
"""Detect an entrypoint restating a standard verbatim (ADR-0026).

The two guards written with ADR-0026 catch the wrong failure. `repo.claude_md_links_resolve`
catches a dead route and `repo.claude_md_under_line_budget` catches growth, but **the actual
way a router decays is somebody inlining a summary of a standard** — which fits inside 150
lines and leaves every link resolving. Nothing detected that, and it is the failure mode
ADR-0026 exists to prevent.

Duplication matters here for a reason specific to this repository: README.md's argument is that
knowledge is enforced rather than advisory, and a second uncheckable copy of a standard is
advisory by construction. When the two disagree there is no rule about which wins.

WHAT COUNTS AS DUPLICATION

A run of N consecutive words appearing in both the entrypoint and a standard or constitution.
Word-level shingles rather than line diffing, because a copy that has been rewrapped or lightly
edited is still a copy and line comparison misses it.

WHAT DOES NOT COUNT, AND WHY

- **Blockquoted lines.** A quotation marked as a quotation is honest attribution; an unmarked
  verbatim passage is a second authority pretending to be original. STD-0005 *requires* the
  changelog brief be reproduced in CLAUDE.md, and it is a blockquote there — so the rule
  "quote it or link it" makes the mandated case legal without an exception list.
- **Code fences, tables and headings.** A routing table legitimately contains the short phrases
  a standard uses; that is the row carrying the one-line version of its rule, which ADR-0026
  cost 1 explicitly endorses.
- **Inline code and link URLs.** Path and identifier overlap is not prose overlap.
- **`adr/` is not compared.** Measured, not assumed: ADR-0022, ADR-0025 and ADR-0026 all quote
  CLAUDE.md rather than the reverse — an ADR about the harness legitimately describes the
  entrypoint, so comparing against `adr/` produced three false positives on the first run. The
  layers at risk of being summarised are `standards/` (how to do things) and `constitution/`
  (principles).

WHY N=10

At the time of writing, the entrypoint was clean at N=8 as well, so 10 is deliberately loose
rather than the tightest available. A summary of a standard shares far more than ten consecutive
words, while ten leaves headroom for a legitimately shared phrase. Tightening it is a one-line
change with that evidence behind it.

  check-duplication.py <entrypoint> <dir> [dir...]
"""
import pathlib
import re
import sys

N = 10


def prose(path: pathlib.Path, drop_quotes: bool) -> str:
    """The comparable prose of a markdown file, with the non-prose stripped."""
    out, in_fence = [], False
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        s = line.strip()
        if s.startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        # frontmatter delimiters, tables, headings
        if s.startswith(("---", "|", "#")):
            continue
        if drop_quotes and s.startswith(">"):
            continue
        out.append(s)
    return " ".join(out)


def words(text: str) -> list[str]:
    text = re.sub(r"`[^`]*`", " ", text)                      # inline code
    text = re.sub(r"\[([^\]]*)\]\([^)]*\)", r"\1", text)      # keep link text, drop the URL
    text = re.sub(r"[*_>]", " ", text)
    return re.findall(r"[a-z0-9']+", text.lower())


def shingles(ws: list[str], n: int) -> set[tuple[str, ...]]:
    return {tuple(ws[i:i + n]) for i in range(len(ws) - n + 1)}


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__.strip().splitlines()[-1], file=sys.stderr)
        return 2
    entry = pathlib.Path(sys.argv[1])
    if not entry.is_file():
        return 0

    # drop_quotes on the entrypoint only: a quotation there is attribution. The standard's own
    # quotes are its content.
    entry_sh = shingles(words(prose(entry, drop_quotes=True)), N)
    if not entry_sh:
        return 0

    failures = 0
    for d in sys.argv[2:]:
        dp = pathlib.Path(d)
        if not dp.is_dir():
            continue
        for f in sorted(dp.glob("*.md")):
            overlap = entry_sh & shingles(words(prose(f, drop_quotes=False)), N)
            if not overlap:
                continue
            phrase = " ".join(sorted(overlap)[0])
            print(
                f"✘ {entry.name} restates {f} verbatim: \"...{phrase}...\" — "
                f"a second uncheckable copy of a rule is advisory by construction, and when the "
                f"two disagree nothing says which wins. Link to it, or mark it as a quotation "
                f"with '>' (ADR-0026)"
            )
            failures += 1
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
