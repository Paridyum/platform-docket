#!/usr/bin/env python3
"""Append test evidence envelopes to a hash-chained, append-only ledger (RFC-0034 §4).

WHY THIS EXISTS
---------------
Envelopes are CI artifacts, and CI artifacts expire — RFC-0034's Problem section names
the 90-day retention as the first of four observable consequences: "after 90 days the
answer to 'did that pass on that commit' is gone, which is precisely the question
STD-0008's Evidence section says a log cannot answer."

This is the durable half. It takes envelopes produced by `junit-to-envelope.py` (or
pulled from the API by the scheduled workflow RFC-0034 §4 describes) and writes each
one into `<ledger-dir>/YYYY-MM.jsonl`, one JSON object per line, each object carrying
`prev_sha256` and `entry_sha256` so that — in the RFC's words — "a rewritten history is
detectable rather than merely discouraged". The detecting is done by
`verify-evidence.py`, which this imports rather than reimplements.

WHAT IT REFUSES TO DO
---------------------
1. Append an envelope it has not validated, against the schema AND against the three
   cross-field rules STD-0014 §"Three rules the schema cannot enforce" names: totals
   summing to `tests`; `run` required when `provenance: ci` and forbidden when `local`;
   `submitted_by` required when `local`; `skipped_reason` required when the outcome is
   `skipped`. STD-0014 makes this a rule and not a caveat — "a producer validates
   against the schema and then checks all three by hand, and a consumer does the same."
   A ledger is a consumer, and it is the last one that can still refuse.
2. Extend a chain that is already broken. If `verify-evidence.py` finds a fault in the
   target month, nothing is appended to it. Appending onto a rewritten history seals
   the rewrite under a valid-looking tail and converts a detected forgery into an
   undetectable one.
3. Rewrite, reorder or reformat an existing line. The file is opened for append and one
   line is written; nothing else in it is ever read for anything but hashing. After the
   write the last line is read back and compared, so a second appender racing this one
   is reported rather than silently interleaved.
4. File an envelope by the wall clock. The month comes from the envelope's `started_at`,
   so a run from last month lands in last month's file. Filing by `now()` would put the
   run in the month the pull happened to notice it, and the ledger's whole purpose is
   answering "when did this last pass".
5. Append the same envelope twice. See "IDENTITY" below.
6. Invent a `run` or an operator. Both come from the envelope; an envelope that has
   neither where STD-0014 requires one is refused, not patched.

THE DIGEST, EXACTLY — because an ambiguous one cannot be verified later
-----------------------------------------------------------------------
RFC-0034 §4 says entries carry `prev_sha256` and `entry_sha256` and does not say what
bytes go in. That is the decision, stated here so a second implementation can agree:

    entry_sha256 = SHA-256( canonical( entry without the `entry_sha256` key ) )
    envelope_sha256 = SHA-256( canonical( the envelope object ) )
    canonical(v) = json.dumps(v, sort_keys=True, separators=(",", ":"),
                              ensure_ascii=False).encode("utf-8")

So the digest covers `schema`, `prev_sha256`, `appended_at`, `envelope_sha256` and the
entire `envelope`. Only the field holding the digest is outside it. Three choices in
there are the ones that matter:

- **Over the value, not the line's bytes.** The entry is parsed and re-canonicalised
  before hashing, so agreeing with this file means canonicalising the same way rather
  than reproducing a particular writer's whitespace.
- **Sorted keys, compact separators, UTF-8, non-ASCII unescaped.** This is what
  `tool-docket`'s `chain::digest_of` already produces: `serde_json::Value`'s map is a
  `BTreeMap`, so its keys come out sorted, and `to_string` is compact and leaves
  non-ASCII alone. The two ledgers hash the same data to the same digest.
- **The key is removed, not blanked.** `{"a":1}` and `{"a":1,"entry_sha256":""}` are
  different objects and hash differently, so which one is meant has to be written down.

`prev_sha256` on the first entry of a file is GENESIS — sixty-four `0`s, the same
constant `chain.rs` uses. It is an explicit value rather than an absent field, so a
head that was deleted is distinguishable from a head that never existed.

The chain is **per month file**, and `verify-evidence.py`'s header states what that
costs and why nothing else works while envelopes arrive out of order.

IDENTITY, AND WHY IT IS THE ENVELOPE ITSELF
--------------------------------------------
Two envelopes are the same evidence when `envelope_sha256` matches — that is, when
their canonical bytes are identical. Re-appending one already in the target month is a
no-op that reports where it already is, and exits 0.

The alternative considered was a composite key: repository + run id + attempt + the
artifact path. It is rejected because **`local` envelopes have no run id at all** —
STD-0014 forbids `run` when `provenance: local`, calling a local envelope that carries
one "a trace nobody can follow" — so the composite cannot identify the very evidence
RFC-0034 §4 says is most worth having: "the write path — the most valuable suite in
the estate". A key that only works for CI would silently degrade to "no key" exactly
where duplicates are most likely, because a local submission is a person running a
command twice.

The content digest has one consequence, and it is the right one: a *different*
envelope for the same run appends a SECOND entry rather than replacing the first. Two
differing reports of one run are two claims, and an append-only ledger records both.
Suppressing the later one would be a rewrite performed by the appender. A warning is
printed when this happens, because a run re-reporting different numbers is a fact
worth seeing.

Its other consequence is stated rather than hidden: two genuinely distinct runs whose
envelopes are byte-identical would be treated as one. That needs the same repository,
commit, ref, tier, stack, totals, duration, artifact digests and the same `started_at`
to the second — and if all of that matches, no reader could have told them apart
either. The ledger is not claiming they are one event; it is declining to invent a
distinction its inputs do not carry.

USAGE
-----
    append-evidence.py --ledger-dir ledger reports/envelope-unit.json [more.json ...]
    append-evidence.py --ledger-dir ledger --dry-run envelope.json
    gh api ... | append-evidence.py --ledger-dir ledger -      # one envelope on stdin

`--ledger` is accepted as a spelling of `--ledger-dir`, because `collect-evidence.py`
— the scheduled pull RFC-0034 §4 describes — calls it that way, and a flag name is not
worth a second implementation of anything.

Exit: 0 every envelope appended or already present, 1 at least one refused (each
named), 2 the appender could not run at all. Never 0 with an unvalidated entry on disk.
"""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

# Same root as junit-to-envelope.py, for the same reason: the script runs both from the
# vendored snapshot in CI (`.paridyum/knowledge/templates/infra-ci/scripts/`) and from a
# checkout of the knowledge repository in tests, and both put the schema and the
# validator three levels up. One relative root serves both; neither needs an env var.
ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent

if sys.stderr.isatty():
    RED, YLW, GRN, DIM, RST = "\033[31m", "\033[33m", "\033[32m", "\033[2m", "\033[0m"
else:
    RED = YLW = GRN = DIM = RST = ""

WARNINGS = 0


def warn(msg: str) -> None:
    global WARNINGS
    WARNINGS += 1
    print(f"{YLW}!{RST} {msg}", file=sys.stderr)


class Refused(Exception):
    """A defect in an envelope, or in the ledger it would go into. Exit 1."""


class Unusable(Exception):
    """The appender cannot do its job at all — bad usage, missing pieces. Exit 2."""


# ── the three pieces this borrows rather than rewrites ───────────────────────
def load_module(path: Path, name: str):
    """Import a sibling script by path. Their filenames contain hyphens, so `import`
    cannot reach them and importlib is the only route."""
    if not path.is_file():
        raise Unusable(f"{path} not found")
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise Unusable(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    try:
        spec.loader.exec_module(mod)
    except SystemExit as e:  # an import-time guard, surfaced rather than swallowed
        raise Unusable(f"{path} refused to load (exit {e.code})") from e
    return mod


def load_rules(converter: Path):
    """junit-to-envelope.py, imported for its cross-field checks and its validator loader.

    Reused rather than reimplemented, on the same argument STD-0014 makes for having one
    converter: "four converters is four places for the totals to be computed
    differently". Two implementations of the three cross-field rules would be two places
    they can drift, and drift there means a producer and a consumer disagreeing about
    what a valid envelope is — which is the disagreement the standard exists to prevent.

    The reuse is a hard dependency on purpose. If the converter is absent this exits 2
    rather than falling back to a weaker check, because an appender that quietly
    verifies less than it claims is the exact defect `validate-schema.py`'s own header
    was written about.
    """
    mod = load_module(converter, "paridyum_junit_to_envelope")
    for attr in ("cross_field_errors", "load_validator"):
        if not hasattr(mod, attr):
            raise Unusable(
                f"{converter} has no {attr}(); the cross-field rules STD-0014 names "
                "cannot be checked and this refuses to append without them"
            )
    return mod


# ── one month file's state, read once ────────────────────────────────────────
class MonthLedger:
    """Everything a batch needs to know about one `YYYY-MM.jsonl`, from one pass.

    Held in memory across a batch because a scheduled pull appends many envelopes at
    once, and re-verifying the whole month per envelope is quadratic in a file that
    only grows. The in-memory head is advanced as lines are written, so the chain is
    correct within a batch without re-reading.
    """

    def __init__(self, path: Path, verifier):
        self.path = path
        self.faults: list = []
        self.head = verifier.GENESIS
        self.index: dict[str, int] = {}   # envelope_sha256 -> line number
        self.coords: dict[tuple, str] = {}  # job coordinates -> envelope_sha256
        self.lines = 0

        if not path.exists():
            return

        # Refusal 2: verify before extending. The whole file, every time — a chain
        # is not "probably intact from here on", and checking only the tail is how a
        # rewrite in the middle survives.
        entries, faults, _notes = verifier.verify_file(path)
        self.faults = faults
        self.lines = len(entries)
        if entries:
            self.head = entries[-1].get("entry_sha256", verifier.GENESIS)
        for lineno, entry in enumerate(entries, 1):
            key = entry.get("envelope_sha256")
            if isinstance(key, str):
                self.index.setdefault(key, lineno)
                coord = job_coordinates(entry.get("envelope"))
                if coord is not None:
                    self.coords.setdefault(coord, key)


def job_coordinates(envelope) -> tuple | None:
    """What identifies the JOB an envelope reports on, as distinct from the envelope.

    Not the idempotency key — see the header on why that is the content digest. This is
    used only to notice the interesting case the content digest cannot: the same job
    reported twice with DIFFERENT numbers. Both entries are kept; the operator is told.
    """
    if not isinstance(envelope, dict):
        return None
    run = envelope.get("run")
    if isinstance(run, dict):
        where = ("ci", str(run.get("id")), str(run.get("attempt")))
    else:
        # A local envelope has no run by STD-0014, so the closest thing to a job
        # identity is who ran it and when they say it started.
        where = ("local", str(envelope.get("submitted_by")), str(envelope.get("started_at")))
    artifacts = envelope.get("artifacts")
    paths = tuple(sorted(
        str(a.get("path")) for a in artifacts if isinstance(a, dict)
    )) if isinstance(artifacts, list) else ()
    return (str(envelope.get("repository")), *where, paths)


# ── validation ───────────────────────────────────────────────────────────────
def month_of(envelope: dict) -> str:
    """`YYYY-MM` from the envelope's own `started_at`. Refusal 4.

    Parsed rather than sliced. The schema's pattern is `[0-9]{2}` for the month, which
    `2026-99-01T00:00:00Z` satisfies — so the contract admits a date that is not a date,
    and slicing would create `ledger/2026-99.jsonl`, a file `verify-evidence.py` would
    then report as not being a ledger file at all.
    """
    started = envelope.get("started_at")
    if not isinstance(started, str):
        raise Refused("no `started_at`, so there is no month to file this under")
    try:
        when = datetime.strptime(started, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    except ValueError as e:
        raise Refused(
            f"started_at {started!r} is not a real RFC 3339 UTC instant — {e}. The "
            "schema's pattern checks the shape and not the calendar, so this is checked "
            "here rather than assumed"
        ) from e
    return f"{when.year:04d}-{when.month:02d}"


def validate(envelope, validator, schema, rules, source: str) -> None:
    """Refusal 1: the schema, then the three rules it cannot express."""
    if not isinstance(envelope, dict):
        raise Refused(f"{source}: the file is a JSON {type(envelope).__name__}, not an envelope object")

    errs = validator.validate(envelope, schema, "$") + rules.cross_field_errors(envelope)
    if errs:
        detail = "\n".join(f"      {e}" for e in errs)
        raise Refused(
            f"{source}: refusing to append an invalid envelope "
            f"({len(errs)} violation(s)):\n{detail}"
        )


# ── the append itself ────────────────────────────────────────────────────────
def build_entry(envelope: dict, prev: str, verifier) -> dict:
    """One ledger line, sealed. The Python of `chain::seal`.

    `appended_at` is the one value here that comes from the clock, and it is a
    different fact from `started_at`: when the ledger accepted the claim, not when the
    run happened. Both are kept because a large gap between them is itself information
    — a pull that stopped working for a fortnight looks like nothing else.
    """
    entry = {
        "schema": verifier.LEDGER_SCHEMA,
        "prev_sha256": prev,
        "appended_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "envelope_sha256": verifier.envelope_digest(envelope),
        "envelope": envelope,
    }
    entry["entry_sha256"] = verifier.entry_digest(entry)
    return entry


def append_line(path: Path, entry: dict, verifier) -> int:
    """Write one line and prove it is the line that landed. Refusal 3.

    Opened `"a"`, so the write is at the end by the operating system's rules rather
    than by a seek this program computes — a computed offset is how an append-only file
    comes to overwrite its own tail. Flushed and fsynced because a ledger entry that
    exists only in a page cache is an entry a power cut removes from history.

    The read-back is not paranoia about the write; it is the only cheap way to notice a
    second appender that wrote between this one's verification and its write. Locking
    was considered and rejected: `fcntl` is POSIX-only and RFC-0034 §4 describes a
    single scheduled workflow, so the honest posture is to detect the race loudly
    rather than to advertise a mutual exclusion that does not hold across machines.
    """
    line = verifier.canonical(entry).decode("utf-8")
    if "\n" in line:  # canonical() escapes newlines, so this cannot happen — checked
        raise Unusable("the sealed entry contains a newline; one entry is one line")

    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as fh:
        fh.write(line + "\n")
        fh.flush()
        os.fsync(fh.fileno())

    with path.open(encoding="utf-8") as fh:
        written = fh.readlines()
    if not written or written[-1].rstrip("\n") != line:
        raise Refused(
            f"{path}: the line just appended is not the last line of the file. Another "
            "process is writing to this ledger at the same time; verify it before "
            "trusting anything appended in this run"
        )
    return len(written)


def process(source: str, text: str, args, validator, schema, rules, verifier, cache) -> bool:
    """One envelope, already read. Returns True when something was written.

    Takes the text rather than a path because an envelope does not have to be a file:
    `collect-evidence.py` pulls them from the GitHub API and pipes them in, and making
    it write a temporary file first would put unvalidated evidence on disk in order to
    validate it.
    """
    try:
        envelope = json.loads(text)
    except json.JSONDecodeError as e:
        raise Refused(f"{source}: not parseable JSON — {e}") from e

    validate(envelope, validator, schema, rules, source)
    month = month_of(envelope)
    target = args.ledger_dir / f"{month}.jsonl"

    if target not in cache:
        cache[target] = MonthLedger(target, verifier)
    ledger = cache[target]

    # Refusal 2, and it is deliberately fatal for this file rather than for the batch:
    # envelopes bound for an intact month still get in.
    if ledger.faults:
        for fault in ledger.faults:
            print(fault.render(), file=sys.stderr)
        raise Refused(
            f"{source}: {target} already has {len(ledger.faults)} chain fault(s), listed "
            "above. Appending would seal a rewritten history under a valid-looking tail. "
            "Restore the file from the last copy that verified, then append again"
        )

    key = verifier.envelope_digest(envelope)
    if (line := ledger.index.get(key)) is not None:
        # Refusal 5. Exit status stays 0: the caller asked for this envelope to be in
        # the ledger and it is. A retried pull must be able to run to completion.
        print(f"{DIM}={RST} {source}: already in {target} at line {line}; nothing appended")
        return False

    coord = job_coordinates(envelope)
    if coord is not None and (other := ledger.coords.get(coord)) is not None and other != key:
        warn(
            f"{source}: {target} already holds a DIFFERENT envelope for the same job "
            f"({verifier.identity(envelope)}). Both are kept — an append-only ledger does "
            "not replace a claim — but one run has now reported two different results"
        )

    entry = build_entry(envelope, ledger.head, verifier)

    if args.dry_run:
        print(f"{DIM}~{RST} {source}: would append to {target} as entry "
              f"{ledger.lines + 1}, prev {entry['prev_sha256'][:12]}…, "
              f"entry {entry['entry_sha256'][:12]}…")
        return False

    lineno = append_line(target, entry, verifier)

    ledger.head = entry["entry_sha256"]
    ledger.index[key] = lineno
    ledger.lines = lineno
    if coord is not None:
        ledger.coords.setdefault(coord, key)

    genesis = entry["prev_sha256"] == verifier.GENESIS
    print(
        f"{GRN}✔{RST} {source} → {target} line {lineno} "
        f"({'genesis' if genesis else 'prev ' + entry['prev_sha256'][:12] + '…'}, "
        f"entry {entry['entry_sha256'][:12]}…) — {verifier.identity(envelope)}, "
        f"{envelope.get('outcome')}, provenance {envelope.get('provenance')}"
    )
    return True


def main(argv=None) -> int:
    p = argparse.ArgumentParser(
        prog="append-evidence.py",
        description="Append STD-0014 test evidence envelopes to a hash-chained, "
                    "append-only ledger (RFC-0034 §4).",
    )
    p.add_argument("envelope", nargs="+",
                   help="one or more envelope JSON files, appended in the order given; "
                        "`-` reads one envelope from stdin")
    p.add_argument("--ledger-dir", "--ledger", required=True, type=Path, dest="ledger_dir",
                   help="directory holding YYYY-MM.jsonl; created if absent")
    p.add_argument("--dry-run", action="store_true",
                   help="validate, verify the target chain and report what would be "
                        "written, without writing")
    p.add_argument("--schema", type=Path, default=ROOT / "test-envelope.schema.json")
    p.add_argument("--validator", type=Path, default=ROOT / "scripts" / "lib" / "validate-schema.py")
    p.add_argument("--converter", type=Path, default=HERE / "junit-to-envelope.py",
                   help="junit-to-envelope.py, imported for the cross-field rules "
                        "STD-0014 says the schema cannot enforce")
    p.add_argument("--verifier", type=Path, default=HERE / "verify-evidence.py",
                   help="verify-evidence.py, imported so the chain has one definition")
    args = p.parse_args(argv)

    try:
        # Read everything first. An envelope that is not there is a usage error, not a
        # refusal, and finding it out after three lines have been appended would leave
        # a batch half-done with no way to tell which half.
        sources: list[tuple[str, str]] = []
        for spec in args.envelope:
            if spec == "-":
                if any(s == "<stdin>" for s, _ in sources):
                    raise Unusable("`-` given twice; stdin can only be read once")
                sources.append(("<stdin>", sys.stdin.read()))
                continue
            f = Path(spec)
            if not f.is_file():
                raise Unusable(f"{f} does not exist")
            sources.append((str(f), f.read_text(encoding="utf-8")))
        if not args.schema.is_file():
            raise Unusable(f"schema not found at {args.schema}")

        verifier = load_module(args.verifier, "paridyum_verify_evidence")
        rules = load_rules(args.converter)
        validator = rules.load_validator(args.validator)
        schema = json.loads(args.schema.read_text(encoding="utf-8"))

        # Relayed rather than hidden, exactly as the converter relays it: `allOf`
        # appearing here is why the cross-field checks are hand-written, and STD-0014
        # quotes this line as evidence that the estate's validator says what it does
        # not enforce instead of passing quietly.
        for gap in validator.unsupported(schema):
            print(f"append-evidence: NOTE unsupported keyword, not enforced — {gap}",
                  file=sys.stderr)

        cache: dict[Path, MonthLedger] = {}
        appended = refused = 0
        for source, text in sources:
            try:
                if process(source, text, args, validator, schema, rules, verifier, cache):
                    appended += 1
            except Refused as e:
                # One bad envelope does not stop a pull of two hundred. Counted and
                # reported at the end, so the exit status still says something went
                # wrong and the good evidence still got in.
                print(f"{RED}✘{RST} {e}", file=sys.stderr)
                refused += 1

        # stdout carries the per-envelope lines and stderr the summary; flushed so a
        # caller capturing both sees them in the order they happened.
        sys.stdout.flush()
        print(
            f"\n{appended} appended, {len(sources) - appended - refused} already "
            f"present or not written, {refused} refused"
            + (f", {WARNINGS} warning(s)" if WARNINGS else ""),
            file=sys.stderr,
        )
        return 1 if refused else 0

    except Unusable as e:
        print(f"append-evidence: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
