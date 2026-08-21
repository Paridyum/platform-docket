#!/usr/bin/env python3
"""JUnit XML in, one test evidence envelope out (STD-0014).

WHY THIS EXISTS
---------------
JUnit is the only report format every stack in the estate already produces, and it is
anonymous about everything a question about test evidence actually needs: which
repository, which commit, which STD-0008 tier, which declared test type from
`test-registry.yaml`, and whether a workflow or a laptop produced it. STD-0014 wraps
the JUnit in an envelope carrying those. This is the ONE converter RFC-0034 §2 calls
for — four per-stack converters would be four places for the totals to be computed
differently, which is how two dashboards come to disagree about the same run.

It reads nothing but structure: counts of `<testcase>` and their `<failure>`,
`<error>` and `<skipped>` children, plus the `timestamp` and `time` attributes. Test
names and failure messages are prose written for a person and branching on them is
ANTI-0008 by name. The one exception is `<skipped message>`, which is copied into
`skipped_reason` verbatim and checked only for being non-empty — carrying a sentence
is not the same as reading one.

WHAT IT REFUSES TO DO
---------------------
1. Emit an envelope it has not validated. Every run validates against
   `test-envelope.schema.json` via the estate's vendored validator, and then by hand
   against the cross-field rules that validator does not implement. STD-0014: "a
   producer that emits an invalid envelope has produced a lie about a test run."
2. Invent a `started_at` or a `duration_ms` it was not given. JUnit's `timestamp` is
   often absent and its `time` sometimes is too. A missing input is refused with the
   flag that supplies it, never filled with `now` or `0` — a ledger of test runs whose
   times are the times the converter happened to run is worse than one with holes.
3. Guess at `implements`. Absent means `[]`, which STD-0014 makes a claim — this job
   implements no declared type — and the claim is printed as a warning, because a job
   covering nothing declared is a real statement about coverage.
4. Collapse `errored` into `failed`. JUnit distinguishes them and so does the
   envelope; collapsing them hides infrastructure decay behind test decay.
5. Write an unattributed `local` envelope. That is the case STD-0014 §"Two rules the
   schema cannot enforce" names as the one that matters.
6. Digest an artifact it cannot read. A recorded sha256 of a file that is not there
   would be a claim nobody can check.

USAGE
-----
    junit-to-envelope.py --tier pr --stack typescript \
        --implements TST-0001,TST-0002 \
        --out reports/envelope-unit.json \
        reports/junit-unit.xml [more.xml ...]

Identity comes from GitHub Actions env vars when they are set and from `git` and flags
when they are not, because `console-docket`'s Playwright suite runs on a laptop only
(no runner has a typesetter, `tool-docket` D-35) and its envelope is the only evidence
that the write path is tested at all. Flags always win over the environment.

Exit: 0 valid envelope written, 1 refused (a defect, named), 2 unusable input or the
validator could not run. Never 0 without a validated envelope on disk.
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import re
import subprocess
import sys
import types
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from pathlib import Path

# The script is run from the vendored snapshot in CI
# (`.paridyum/knowledge/templates/infra-ci/scripts/`) and from a checkout of the
# knowledge repository in tests. Both put the schema, the validator and the registry
# three levels up, so one relative root serves both and neither needs an env var.
ROOT = Path(__file__).resolve().parents[3]

SCHEMA_VERSION = "paridyum.test-envelope/1"

if sys.stderr.isatty():
    RED, YLW, GRN, RST = "\033[31m", "\033[33m", "\033[32m", "\033[0m"
else:
    RED = YLW = GRN = RST = ""

WARNINGS = 0


def warn(msg: str) -> None:
    global WARNINGS
    WARNINGS += 1
    print(f"{YLW}!{RST} {msg}", file=sys.stderr)


class Refused(Exception):
    """A defect in the inputs or in the envelope built from them. Exit 1."""


class Unusable(Exception):
    """The converter cannot do its job at all — bad usage, no validator. Exit 2."""


# ── the vendored validator ───────────────────────────────────────────────────
def load_validator(path: Path):
    """Import scripts/lib/validate-schema.py and return its module.

    Reused rather than reimplemented: a second JSON Schema subset in the estate is a
    second set of keywords that quietly enforce nothing, which is the defect that file
    was written to end.
    """
    if not path.is_file():
        raise Unusable(f"validator not found at {path}; cannot validate the envelope")
    try:
        import yaml  # noqa: F401
    except ImportError:
        # validate-schema.py exits 2 at import time without PyYAML, because its own CLI
        # reads YAML. `validate()` and `unsupported()` — all this uses — touch neither
        # yaml nor the loader subclass built beside them. A stub carrying only what that
        # module-level line reads keeps the reuse available where PyYAML is absent, and
        # says so rather than looking like a full load.
        warn("PyYAML absent; validate-schema.py loaded for JSON validation only")
        stub = types.ModuleType("yaml")
        stub.SafeLoader = type("SafeLoader", (), {"yaml_implicit_resolvers": {}})
        stub.YAMLError = Exception
        sys.modules["yaml"] = stub

    spec = importlib.util.spec_from_file_location("paridyum_validate_schema", path)
    if spec is None or spec.loader is None:
        raise Unusable(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    try:
        spec.loader.exec_module(mod)
    except SystemExit as e:  # its import-time guard, surfaced rather than swallowed
        raise Unusable(f"{path} refused to load (exit {e.code})") from e
    return mod


# ── JUnit parsing ────────────────────────────────────────────────────────────
def _num(el, attr):
    """A float attribute, or None. An unparseable number is absent, not zero."""
    raw = el.get(attr)
    if raw is None or raw.strip() == "":
        return None
    try:
        return float(raw)
    except ValueError:
        warn(f"{attr}={raw!r} is not a number; treating it as absent")
        return None


def parse_timestamp(raw: str, source: str):
    """One JUnit `timestamp` as an aware UTC datetime, or None.

    Three shapes arrive in practice and all three are handled explicitly: Vitest writes
    `2026-08-21T16:11:44.529Z`, nextest writes an offset, and some writers emit a naive
    local time. A naive timestamp is a time whose zone is not stated; it is read in this
    machine's zone — the machine that ran the tests — and the assumption is announced,
    because it is only sound while the converter runs beside the run it describes.
    """
    text = raw.strip()
    if not text:
        return None
    normalised = text[:-1] + "+00:00" if text.endswith(("Z", "z")) else text
    try:
        dt = datetime.fromisoformat(normalised)
    except ValueError:
        warn(f"{source}: timestamp {raw!r} is not ISO 8601; ignoring it")
        return None
    if dt.tzinfo is None:
        warn(
            f"{source}: timestamp {raw!r} states no time zone; read as local "
            f"({datetime.now().astimezone().tzname()}) because this machine ran the tests"
        )
        dt = dt.astimezone()
    return dt.astimezone(timezone.utc)


def parse_junit(path: Path) -> dict:
    """Structure only: counts, the earliest start, and the wall time.

    `tests=` and `failures=` attributes are read for one purpose — comparing them with
    what was counted — and never trusted as the answer. ADR-0035's rule about counts in
    prose is the same rule here: compute it, do not copy a number someone else wrote.
    """
    try:
        root = ET.parse(path).getroot()
    except ET.ParseError as e:
        raise Refused(f"{path}: not parseable XML — {e}")

    if root.tag not in ("testsuites", "testsuite"):
        raise Refused(f"{path}: root element is <{root.tag}>, not JUnit")

    suites = [root] if root.tag == "testsuite" else root.findall(".//testsuite")

    counts = {"tests": 0, "passed": 0, "failed": 0, "skipped": 0, "errored": 0}
    skip_reasons: list[str] = []
    reasonless_skips = 0

    for case in root.iter("testcase"):
        counts["tests"] += 1
        # Precedence is deliberate: a case that both errored and failed is errored,
        # because the harness breaking is the larger fact and the one that decays
        # silently. Order here IS the outcome vocabulary of STD-0014.
        if case.find("error") is not None:
            counts["errored"] += 1
        elif case.find("failure") is not None:
            counts["failed"] += 1
        elif (skipped := case.find("skipped")) is not None:
            counts["skipped"] += 1
            reason = (skipped.get("message") or skipped.text or "").strip()
            if reason:
                skip_reasons.append(reason)
            else:
                reasonless_skips += 1
        else:
            counts["passed"] += 1

    declared = _num(root, "tests")
    if declared is not None and int(declared) != counts["tests"]:
        warn(
            f"{path}: declares tests={int(declared)} but carries {counts['tests']} "
            "<testcase> elements; the counted value is used"
        )

    starts = [
        ts for el in suites
        if (ts := parse_timestamp(el.get("timestamp") or "", f"{path}")) is not None
    ]
    if root.tag == "testsuites" and (ts := parse_timestamp(root.get("timestamp") or "", f"{path}")):
        starts.append(ts)

    # A root `time` is the run's wall clock. Summing suite times instead double-counts
    # whatever ran in parallel, so it is the fallback and not the first choice.
    seconds = _num(root, "time")
    if seconds is None:
        suite_times = [t for el in suites if (t := _num(el, "time")) is not None]
        seconds = sum(suite_times) if suite_times else None

    return {
        "counts": counts,
        "skip_reasons": skip_reasons,
        "reasonless_skips": reasonless_skips,
        "started_at": min(starts) if starts else None,
        "seconds": seconds,
    }


# ── identity ─────────────────────────────────────────────────────────────────
def git(*args) -> str | None:
    try:
        out = subprocess.run(["git", *args], capture_output=True, text=True, check=False)
    except FileNotFoundError:
        return None
    return out.stdout.strip() or None if out.returncode == 0 else None


def repository_from_git() -> str | None:
    """`owner/name` from the origin remote, in either URL spelling GitHub hands out."""
    url = git("remote", "get-url", "origin")
    if not url:
        return None
    m = re.search(r"[:/]([A-Za-z0-9._-]+)/([A-Za-z0-9._-]+?)(?:\.git)?/?$", url)
    return f"{m.group(1)}/{m.group(2)}" if m else None


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 16), b""):
            h.update(chunk)
    return h.hexdigest()


def artifact_path(path: Path) -> str:
    """The path as an envelope may spell it: relative to the working directory if it
    lives under it, which is what a CI upload step and a reader both expect."""
    resolved = path.resolve()
    try:
        return str(resolved.relative_to(Path.cwd()))
    except ValueError:
        return str(resolved)


# ── the two rules the schema cannot enforce, and the one it does not express ──
def cross_field_errors(env: dict) -> list[str]:
    """Checks no JSON Schema validator, or not ours, will make.

    STD-0014 names two. There is a third: the standard's table requires
    `skipped_reason` when `outcome` is `skipped`, and the schema's `allOf` has no clause
    for it — so unlike the other two, a full JSON Schema validator would not catch it
    either. It is checked here on the same reasoning: a skip with no reason is
    indistinguishable from a pass to everything downstream.
    """
    errs = []

    t = env.get("totals")
    if isinstance(t, dict) and all(isinstance(t.get(k), int) for k in
                                   ("tests", "passed", "failed", "skipped", "errored")):
        summed = t["passed"] + t["failed"] + t["skipped"] + t["errored"]
        if summed != t["tests"]:
            errs.append(
                f"totals: passed+failed+skipped+errored = {summed} but tests = {t['tests']} "
                "(STD-0014: not expressible in JSON Schema, the language has no arithmetic)"
            )

    prov = env.get("provenance")
    if prov == "ci" and "run" not in env:
        errs.append("provenance is 'ci' but there is no 'run' — the run cannot be traced to its logs")
    if prov == "local" and not str(env.get("submitted_by") or "").strip():
        errs.append("provenance is 'local' but there is no 'submitted_by' — an anonymous claim about a test run")
    if prov == "local" and "run" in env:
        errs.append("provenance is 'local' but a 'run' is present — a local envelope has no inspectable run")

    if env.get("outcome") == "skipped" and not str(env.get("skipped_reason") or "").strip():
        errs.append("outcome is 'skipped' with no 'skipped_reason' — indistinguishable from a pass downstream")

    return errs


def registered_test_types(registry: Path) -> set[str] | None:
    """The declared TST- ids, by line shape rather than by YAML parse.

    Read to refuse a job claiming a type that does not exist: `implements` is the whole
    mechanism turning a guess into a fact (RFC-0034 §1), and an unregistered id in it is
    the drift ADR-0029 made the registry to prevent. Returns None when the registry is
    not there — in that case the claim is not checkable and that is said out loud.
    """
    if not registry.is_file():
        return None
    return set(re.findall(r"^\s*-\s*test_type:\s*(TST-[0-9]{4})\s*$",
                          registry.read_text(encoding="utf-8"), re.MULTILINE))


# ── assembly ─────────────────────────────────────────────────────────────────
def build(args) -> dict:
    # The same file twice would double every total, and a shell glob overlapping an
    # explicit path is how that happens. Refused rather than deduplicated: the caller
    # meant something, and quietly dropping one of them decides which.
    seen: dict[Path, Path] = {}
    for p in args.junit:
        key = p.resolve()
        if key in seen:
            raise Refused(f"{p} and {seen[key]} are the same file; every total would be counted twice")
        seen[key] = p

    parsed = [parse_junit(p) for p in args.junit]

    counts = {k: sum(r["counts"][k] for r in parsed)
              for k in ("tests", "passed", "failed", "skipped", "errored")}

    # Outcome from counts, in this order. `errored` first because a harness that failed
    # to run the tests has said nothing about whether they pass.
    #
    # `timed_out` is in STD-0014's vocabulary and is never produced here, because JUnit
    # cannot express it: a job killed by a timeout writes no report at all, so its
    # envelope cannot come from a converter that reads one.
    if counts["errored"]:
        outcome = "errored"
    elif counts["failed"]:
        outcome = "failed"
    elif counts["tests"] == 0 or counts["skipped"] == counts["tests"]:
        outcome = "skipped"
    else:
        outcome = "passed"

    reasonless = sum(r["reasonless_skips"] for r in parsed)
    if reasonless:
        warn(f"{reasonless} skipped test(s) carry no message; a reasonless skip reads as a pass downstream")

    skipped_reason = None
    if outcome == "skipped":
        collected = args.skipped_reason or "; ".join(
            dict.fromkeys(r for p in parsed for r in p["skip_reasons"]))
        if not collected.strip():
            if counts["tests"] == 0:
                raise Refused(
                    "no <testcase> in any input, so the outcome is 'skipped' and nothing "
                    "says why; pass --skipped-reason to state it for a person"
                )
            raise Refused(
                "every test skipped and no <skipped message> anywhere; pass "
                "--skipped-reason (STD-0014: a skip with no reason is how coverage "
                "disappears quietly)"
            )
        skipped_reason = collected

    # started_at and duration_ms: taken from the JUnit, or from a flag, and never
    # fabricated. See the header, refusal 2.
    if args.started_at:
        started = parse_timestamp(args.started_at, "--started-at")
        if started is None:
            raise Unusable(f"--started-at {args.started_at!r} is not an ISO 8601 timestamp")
    else:
        stamps = [r["started_at"] for r in parsed if r["started_at"]]
        if stamps:
            started = min(stamps)
        elif args.started_at_from_mtime:
            # A fact about the file, not about the run: the mtime is when the report was
            # written, i.e. roughly when the run ENDED. Only ever on request.
            mtime = min(p.stat().st_mtime for p in args.junit)
            started = datetime.fromtimestamp(mtime, timezone.utc)
            warn("no timestamp in any input; using the JUnit file's mtime, which is when "
                 "the report was written rather than when the run began")
        else:
            raise Refused(
                "no `timestamp` attribute in any input, so the start of the run is an "
                "input this converter does not have; pass --started-at, or "
                "--started-at-from-mtime to substitute the report file's mtime"
            )

    if args.duration_ms is not None:
        duration_ms = args.duration_ms
    else:
        seconds = [r["seconds"] for r in parsed if r["seconds"] is not None]
        if not seconds:
            raise Refused(
                "no `time` attribute in any input, so the duration is an input this "
                "converter does not have; pass --duration-ms (0 would be a claim that "
                "the run took no time)"
            )
        duration_ms = round(sum(seconds) * 1000)

    repository = args.repository or os.environ.get("GITHUB_REPOSITORY") or repository_from_git()
    if not repository:
        raise Refused("no repository: GITHUB_REPOSITORY is unset and no `origin` remote resolves; pass --repository")

    commit = args.commit or os.environ.get("GITHUB_SHA") or git("rev-parse", "HEAD")
    if not commit:
        raise Refused("no commit: GITHUB_SHA is unset and `git rev-parse HEAD` failed; pass --commit")

    ref = args.ref or os.environ.get("GITHUB_REF") or git("symbolic-ref", "-q", "HEAD")
    if not ref:
        raise Refused("no ref: GITHUB_REF is unset and HEAD is detached; pass --ref")

    # `ci` is claimed only where a workflow run identifies itself. Defaulting the other
    # way would let a laptop run be reported as CI by omission, which is rule 1 of
    # STD-0014 §provenance inverted.
    provenance = args.provenance or ("ci" if os.environ.get("GITHUB_RUN_ID") else "local")

    implements = []
    for chunk in args.implements:
        implements.extend(part.strip() for part in chunk.split(",") if part.strip())
    implements = list(dict.fromkeys(implements))
    if not implements:
        warn("no --implements: writing `implements: []`, which STD-0014 makes a claim — "
             "this job implements NO declared test type from test-registry.yaml")
    else:
        known = registered_test_types(args.registry)
        if known is None:
            warn(f"{args.registry} not found; the TST- ids in --implements are unchecked")
        elif unknown := [i for i in implements if i not in known]:
            raise Refused(f"--implements names types absent from {args.registry}: {', '.join(unknown)}")

    envelope = {
        "schema": SCHEMA_VERSION,
        "repository": repository,
        "commit": commit,
        "ref": ref,
        "tier": args.tier,
        "implements": implements,
        "stack": args.stack,
        "provenance": provenance,
    }

    if provenance == "ci":
        run_id = args.run_id or os.environ.get("GITHUB_RUN_ID")
        attempt = args.run_attempt or os.environ.get("GITHUB_RUN_ATTEMPT") or "1"
        server = os.environ.get("GITHUB_SERVER_URL", "https://github.com").rstrip("/")
        if not run_id:
            raise Refused("provenance is 'ci' but GITHUB_RUN_ID is unset; pass --run-id")
        if not str(attempt).isdigit():
            raise Refused(f"run attempt {attempt!r} is not a number; the envelope would name a run nobody can find")
        envelope["run"] = {
            "id": str(run_id),
            "attempt": int(attempt),
            "url": args.run_url or f"{server}/{repository}/actions/runs/{run_id}/attempts/{attempt}",
        }
    else:
        submitted_by = args.submitted_by or os.environ.get("PARIDYUM_OPERATOR")
        if not submitted_by:
            raise Refused(
                "provenance is 'local' and no operator is named; pass --submitted-by "
                "(an id from registry/operators.yaml) or set PARIDYUM_OPERATOR. "
                "STD-0014: a local result with no author is an anonymous claim"
            )
        envelope["submitted_by"] = submitted_by

    envelope["started_at"] = started.strftime("%Y-%m-%dT%H:%M:%SZ")
    envelope["duration_ms"] = duration_ms
    envelope["outcome"] = outcome
    envelope["totals"] = counts

    artifacts = [{"kind": "junit", "path": artifact_path(p), "sha256": sha256(p)}
                 for p in args.junit]
    for spec in args.artifact:
        kind, _, raw = spec.partition(":")
        if not raw:
            raise Unusable(f"--artifact {spec!r} is not kind:path")
        extra = Path(raw)
        if not extra.is_file():
            raise Refused(f"--artifact {spec!r}: {raw} does not exist, so it has no digest to record")
        artifacts.append({"kind": kind, "path": artifact_path(extra), "sha256": sha256(extra)})
    envelope["artifacts"] = artifacts

    envelope["skipped_reason"] = skipped_reason
    return envelope


def main(argv=None) -> int:
    p = argparse.ArgumentParser(
        prog="junit-to-envelope.py",
        description="Convert JUnit XML into one STD-0014 test evidence envelope.",
    )
    p.add_argument("junit", nargs="+", type=Path, help="one or more JUnit XML files, merged into one envelope")
    p.add_argument("--out", required=True, help="envelope path, or - for stdout")
    # `tier` and `stack` are not enumerated here on purpose: the schema is the one
    # place that vocabulary lives, and a copy of it in this file is a copy that ages.
    p.add_argument("--tier", required=True, help="STD-0008 tier, as test-registry.yaml spells it")
    p.add_argument("--stack", required=True, help="class-registry.yaml stack, or `none`")
    p.add_argument("--implements", action="append", default=[], metavar="TST-NNNN",
                   help="declared test types; repeatable or comma-separated. Absent means [] and warns")
    p.add_argument("--provenance", choices=["ci", "local"], help="default: ci when GITHUB_RUN_ID is set, else local")
    p.add_argument("--submitted-by", help="operator id, required when provenance is local")
    p.add_argument("--repository", help="owner/name; default GITHUB_REPOSITORY, else the origin remote")
    p.add_argument("--commit", help="full 40-char SHA; default GITHUB_SHA, else git rev-parse HEAD")
    p.add_argument("--ref", help="full ref; default GITHUB_REF, else HEAD's symbolic ref")
    p.add_argument("--run-id", help="default GITHUB_RUN_ID")
    p.add_argument("--run-attempt", help="default GITHUB_RUN_ATTEMPT, else 1")
    p.add_argument("--run-url", help="default derived from GITHUB_SERVER_URL, the repository and the run")
    p.add_argument("--started-at", help="RFC 3339; overrides the JUnit timestamp")
    p.add_argument("--started-at-from-mtime", action="store_true",
                   help="when no input carries a timestamp, substitute the report file's mtime and say so")
    p.add_argument("--duration-ms", type=int, help="overrides the JUnit `time` attributes")
    p.add_argument("--skipped-reason", help="for a person; required when everything was skipped")
    p.add_argument("--artifact", action="append", default=[], metavar="KIND:PATH",
                   help="an additional artifact to reference and digest; repeatable")
    p.add_argument("--schema", type=Path, default=ROOT / "test-envelope.schema.json")
    p.add_argument("--validator", type=Path, default=ROOT / "scripts" / "lib" / "validate-schema.py")
    p.add_argument("--registry", type=Path, default=ROOT / "test-registry.yaml")
    args = p.parse_args(argv)

    try:
        for f in args.junit:
            if not f.is_file():
                raise Unusable(f"{f} does not exist")
        if not args.schema.is_file():
            raise Unusable(f"schema not found at {args.schema}")

        validator = load_validator(args.validator)
        schema = json.loads(args.schema.read_text(encoding="utf-8"))

        # The validator reports what it does not implement rather than passing quietly.
        # Relayed rather than hidden: `allOf` appearing here is exactly why the
        # cross-field checks below are hand-written, and STD-0014 quotes this line.
        for gap in validator.unsupported(schema):
            print(f"junit-to-envelope: NOTE unsupported keyword, not enforced — {gap}", file=sys.stderr)

        envelope = build(args)

        # Validate the bytes that will be written, not the object in memory: the file is
        # what a consumer reads, and a serialisation that loses a value would pass an
        # in-memory check and ship a different envelope.
        text = json.dumps(envelope, indent=2, ensure_ascii=False) + "\n"
        written = json.loads(text)

        errs = validator.validate(written, schema, "$") + cross_field_errors(written)
        if errs:
            print(f"{RED}✘{RST} refusing to write an invalid envelope ({len(errs)} violation(s)):", file=sys.stderr)
            for e in errs:
                print(f"    {e}", file=sys.stderr)
            return 1

        if args.out == "-":
            sys.stdout.write(text)
        else:
            out = Path(args.out)
            out.parent.mkdir(parents=True, exist_ok=True)
            out.write_text(text, encoding="utf-8")

        t = written["totals"]
        print(
            f"{GRN}✔{RST} {args.out}: {written['outcome']} — {t['tests']} test(s), "
            f"{t['passed']} passed, {t['failed']} failed, {t['errored']} errored, "
            f"{t['skipped']} skipped; provenance {written['provenance']}, "
            f"implements {written['implements'] or '[] (nothing declared)'}"
            + (f", {WARNINGS} warning(s)" if WARNINGS else ""),
            file=sys.stderr,
        )
        return 0

    except Refused as e:
        print(f"{RED}✘{RST} {e}", file=sys.stderr)
        return 1
    except Unusable as e:
        print(f"junit-to-envelope: {e}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
