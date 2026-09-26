#!/usr/bin/env python3
"""Fetch STD-0014 envelopes from the organisation's CI runs and append them.

WHY THIS PULLS RATHER THAN BEING PUSHED TO
-------------------------------------------
REQ-0001 established, by testing the live organisation, that organisation-level
Actions secrets do not work for private repositories on the Free plan — so every
repository's secrets are set by hand, in that repository, forever.

A push design needs one WRITE credential per repository, growing with the estate,
failing invisibly when one goes stale, and letting any repository write any other's
results. This needs one READ-ONLY credential in one place, and no repository can
write to the ledger at all.

WHAT IT REFUSES TO DO
---------------------
It does not judge. An envelope is appended exactly as the producing job wrote it, or
it is refused and said so. In particular it never infers `implements` from a job's
name — the whole point of RFC-0034 is that the job declares it, and a collector that
guessed would reintroduce the inference one layer down.

It appends only `provenance: ci` envelopes. A local envelope reaches the ledger by
being submitted deliberately (`append-evidence.py`), never by being scraped: a
collector that hoovered up laptop results would make the ledger's provenance field
meaningless.
"""

from __future__ import annotations

import argparse
import io
import json
import os
import subprocess
import sys
import urllib.error
import urllib.request
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
API = "https://api.github.com"

# The repositories collected from when none are named. Written down rather than
# discovered, because discovery needs a token permission this deliberately does not
# have (org read), and because a collector silently widening its own scope is worse
# than one that is edited.
DEFAULT_REPOSITORIES = [
    "Paridyum/tool-docket",
    "Paridyum/tool-forge",
    "Paridyum/console-docket",
    "Paridyum/console-testing",
]

# How many recent runs per repository to look through. The schedule is twice daily,
# so this is the number of runs a repository can have between collections before one
# is missed. Generous rather than tight: a missed envelope is a permanent hole in an
# append-only ledger.
RUNS_PER_REPOSITORY = 40


def token() -> str:
    for name in ("GH_TOKEN", "GITHUB_TOKEN", "EVIDENCE_TOKEN"):
        value = os.environ.get(name, "").strip()
        if value:
            return value
    sys.exit(
        "no token in GH_TOKEN, GITHUB_TOKEN or EVIDENCE_TOKEN. Refusing to run: a "
        "collector with no credential reports an empty ledger, which is "
        "indistinguishable from nothing having happened."
    )


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    """Stop at the redirect instead of following it."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):  # noqa: D102
        return None


def download(url: str, auth: str) -> bytes:
    """Fetch an artifact zip, which lives behind a redirect to storage.

    The redirect is handled by hand because following it automatically FAILS: urllib
    re-sends the `Authorization` header to the storage host, which rejects it with
    401. That produced a collector reporting `0 envelope(s)` from repositories whose
    artifacts plainly existed — success-shaped output, no evidence, no error.

    So: ask GitHub, take the `Location`, and fetch it with NO credential. The signed
    URL carries its own authorisation, which is why sending ours breaks it.
    """
    opener = urllib.request.build_opener(_NoRedirect)
    request = urllib.request.Request(
        url,
        headers={
            "Authorization": f"Bearer {auth}",
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "paridyum-collect-evidence",
        },
    )
    try:
        with opener.open(request, timeout=120) as response:
            # No redirect: some responses stream the zip directly.
            return response.read()
    except urllib.error.HTTPError as e:
        if e.code not in (301, 302, 303, 307, 308):
            raise
        location = e.headers.get("Location")
        if not location:
            raise
    with urllib.request.urlopen(
        urllib.request.Request(location, headers={"User-Agent": "paridyum-collect-evidence"}),
        timeout=120,
    ) as signed:
        return signed.read()


def api(path: str, auth: str) -> bytes:
    request = urllib.request.Request(
        f"{API}{path}",
        headers={
            "Authorization": f"Bearer {auth}",
            "Accept": "application/vnd.github+json",
            "X-GitHub-Api-Version": "2022-11-28",
            "User-Agent": "paridyum-collect-evidence",
        },
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        return response.read()


def envelopes_in(archive: bytes) -> list[dict]:
    """Every envelope inside one artifact zip."""
    found = []
    try:
        with zipfile.ZipFile(io.BytesIO(archive)) as zf:
            for name in zf.namelist():
                if not name.endswith(".json") or "envelope" not in name:
                    continue
                try:
                    found.append(json.loads(zf.read(name)))
                except json.JSONDecodeError:
                    # A malformed envelope is skipped and reported, never repaired.
                    print(f"  ! {name}: not JSON, skipped", file=sys.stderr)
    except zipfile.BadZipFile:
        print("  ! artifact is not a zip, skipped", file=sys.stderr)
    return found


def collect(repository: str, auth: str) -> list[dict]:
    """Envelopes from a repository's recent completed runs."""
    try:
        runs = json.loads(
            api(
                f"/repos/{repository}/actions/runs"
                f"?status=completed&per_page={RUNS_PER_REPOSITORY}",
                auth,
            )
        ).get("workflow_runs", [])
    except urllib.error.HTTPError as e:
        # 404 on a private repository means the token cannot see it. That is a
        # permissions fact, and reporting it as "no runs" would hide a misconfigured
        # token behind a plausible empty result.
        why = "not visible to this token" if e.code == 404 else f"HTTP {e.code}"
        print(f"  ! {repository}: {why}", file=sys.stderr)
        return []

    out: list[dict] = []
    for run in runs:
        try:
            artifacts = json.loads(
                api(f"/repos/{repository}/actions/runs/{run['id']}/artifacts", auth)
            ).get("artifacts", [])
        except urllib.error.HTTPError:
            continue
        for artifact in artifacts:
            if artifact.get("expired"):
                # Expired artifacts are exactly why this ledger exists. Nothing to
                # recover; the gap is permanent and worth counting.
                continue
            try:
                blob = download(
                    f"{API}/repos/{repository}/actions/artifacts/{artifact['id']}/zip",
                    auth,
                )
            except (urllib.error.HTTPError, urllib.error.URLError) as e:
                # Reported, never swallowed. A silent `continue` here is what made an
                # earlier version print `0 envelope(s)` and exit 0 while every
                # download was failing with 401.
                print(
                    f"  ! {repository} artifact {artifact['name']}: {e}", file=sys.stderr
                )
                continue
            out.extend(envelopes_in(blob))
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ledger", required=True, help="ledger directory")
    parser.add_argument(
        "--repositories", help="comma-separated owner/name; default is the list in this file"
    )
    parser.add_argument(
        "--dry-run", action="store_true", help="fetch and report, append nothing"
    )
    args = parser.parse_args()

    auth = token()
    repositories = (
        [r.strip() for r in args.repositories.split(",") if r.strip()]
        if args.repositories
        else DEFAULT_REPOSITORIES
    )

    appender = Path(__file__).resolve().parent / "append-evidence.py"
    if not appender.exists():
        sys.exit(f"{appender} is missing; nothing can be appended")

    total = skipped_local = 0
    for repository in repositories:
        found = collect(repository, auth)
        ci_only = []
        for envelope in found:
            if envelope.get("provenance") != "ci":
                # Local envelopes are submitted deliberately, never scraped. See the
                # module docstring.
                skipped_local += 1
                continue
            ci_only.append(envelope)
        print(f"  {repository}: {len(ci_only)} envelope(s)")

        for envelope in ci_only:
            if args.dry_run:
                total += 1
                continue
            # Appending goes through append-evidence.py rather than being reimplemented
            # here, so the chain and the refusal rules have exactly one implementation.
            result = subprocess.run(
                [sys.executable, str(appender), "--ledger", args.ledger, "-"],
                input=json.dumps(envelope),
                text=True,
                capture_output=True,
                check=False,
            )
            if result.returncode != 0:
                print(f"  ! refused: {result.stderr.strip()}", file=sys.stderr)
                continue
            total += 1

    verb = "would be appended" if args.dry_run else "appended"
    print(f"\n{total} envelope(s) {verb}", end="")
    if skipped_local:
        print(f"; {skipped_local} local envelope(s) not scraped", end="")
    print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
