#!/usr/bin/env python3
"""Report which declared test types actually run in the estate, and where.

    report-test-coverage.py --root <knowledge-repo> [--estate DIR] [--only required] [--tsv]

Driven by scripts/report-test-coverage.sh, which explains why the report exists. This file
explains why it is built the way it is.

WHY EVIDENCE, AND NOT A MAPPING TABLE
-------------------------------------
There is no declared mapping from a test type to the thing that runs it. manifest.schema.json
has no test-type field, and a grep of the whole estate finds a TST- id in exactly two
pipeline sites. So a hardcoded "TST-0001 runs in tool-docket" table would be a fourth copy
of a list nothing compares against — the defect ADR-0029 was written to end, reintroduced by
the tool meant to report on it.

Instead every row is produced by a PROBE that names the artefact it read, and every finding
carries a confidence:

  declared   the pipeline names the TST- id itself. The only non-inferred evidence there is.
  direct     the pipeline's own step list and literal gates decide it, with no judgement
             left over: a step exists and its `if:` resolves against the caller's literal
             inputs, or the step list is fully enumerable and contains no such step.
  heuristic  inferred from a tool name or a file layout. Could be wrong in both directions.

A type with no probe is `cannot determine`. It is NOT `not found`, because nothing looked —
and reporting "not found" for something never searched for is the exact overclaim this
report exists to expose.

WHY THE PIPELINE IS RESOLVED ACROSS REPOSITORIES
------------------------------------------------
Caller repositories contain almost no pipeline: ci.yml calls a reusable workflow in infra-ci
(ADR-0021). Reading only the caller's own file finds no tests anywhere and would report the
entire estate as untested. So `uses: <org>/<repo>/.github/workflows/<file>@<ref>` is resolved
against the local sibling repository of that name, and the caller's `with:` inputs are bound
so gated jobs can be evaluated. That resolution is itself an assumption and is stated in the
output: the local checkout of infra-ci is not necessarily what the pinned tag points at.

WHY GATES ARE EVALUATED AT ALL
------------------------------
Because a job that exists and never fires is the failure mode most likely to be reported as
coverage. `mutants` and the integration step both exist in rust-ci.yml and both are gated;
one caller passes `integration: false` explicitly. A report that listed the step and stopped
would say mutation testing and integration testing run in this estate. They do not.

PyYAML is required. If it is missing this exits 2 rather than printing a thinner report, for
the reason validate-schema.py does: a report that could not read the registry must never be
mistaken for one that read it and found nothing.

READ-ONLY. This opens files for reading and writes only to stdout/stderr. No path is opened
for writing anywhere in this file.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys

try:
    import yaml
except ImportError:  # pragma: no cover - environment-dependent
    print("report-test-coverage: PyYAML not available; cannot read the registry",
          file=sys.stderr)
    print("  Reported as an inability, not skipped: an empty report reads as "
          "\"nothing runs\".", file=sys.stderr)
    sys.exit(2)


RUNS = "runs"
NOT_FOUND = "not found"
UNKNOWN = "cannot determine"

DECLARED = "declared"
DIRECT = "direct"
HEURISTIC = "heuristic"

# Directories that are never part of a repository's own pipeline. `.paridyum` holds the
# vendored knowledge snapshot (ADR-0001), which contains templates of the very workflows
# being searched for — counting those as evidence would report every repository as running
# every test type the templates mention.
SKIP_DIRS = {".git", "node_modules", "target", ".next", ".paridyum", "dist", "build",
             ".venv", "vendor"}


# --- estate discovery ------------------------------------------------------

class Repo:
    """One git repository in the estate, with the config files a probe may read.

    `stack` comes from manifest.yaml, which is the estate's own declaration of which
    language rules bind a repository (class-registry.yaml). A repository without a
    manifest gets stack `unknown` rather than a guess from its file extensions: guessing
    would silently put it in scope for stack-limited test types.
    """

    def __init__(self, path):
        self.path = path
        self.dirname = os.path.basename(path)
        self.name = self.dirname
        self.cls = "unknown"
        self.stack = "unknown"
        self.workflows = []          # entry workflows (have `on:` triggers)
        self.callable_workflows = {} # basename -> parsed, for cross-repo resolution
        self.steps = []              # resolved ExecStep list
        self.texts = {}              # label -> raw text, for signature probes
        self.pkg_scripts = {}
        self.unresolved = []         # `uses:` refs that could not be resolved locally

    def has(self, rel):
        return os.path.exists(os.path.join(self.path, rel))

    def read(self, rel):
        try:
            with open(os.path.join(self.path, rel), "r", encoding="utf-8",
                      errors="replace") as fh:
                return fh.read()
        except OSError:
            return ""


class ExecStep:
    """A step that the estate's pipelines would actually execute, with its gate resolved."""

    def __init__(self, repo, entry_wf, source_wf, job_id, job_name, name, run, uses, state,
                 gate):
        self.repo = repo
        self.entry_wf = entry_wf
        self.source_wf = source_wf
        self.job_id = job_id
        self.job_name = job_name
        self.name = name or ""
        self.run = run or ""
        self.uses = uses or ""
        self.state = state           # active | off | unknown
        self.gate = gate             # the `if:` text that decided it, for the output

    @property
    def blob(self):
        # The JOB name is part of what a probe may match on, because in this estate that is
        # where the identifying label lives: rust-ci.yml's msrv job passes its version
        # through `with:` and names nothing in any step, so a step-only search reported "no
        # msrv job" about a workflow that contains one.
        return "\n".join((self.job_name or self.job_id or "", self.name, self.run, self.uses))

    def where(self):
        wf = os.path.basename(self.source_wf)
        label = self.name or self.uses
        if not label:
            # An unnamed step is identified by its command. Cut on a word boundary: a label
            # sliced mid-token ("origin \"${{ github.bas") reads as a parsing bug and makes a
            # reader distrust the row rather than the step.
            first = self.run.strip().splitlines()[0] if self.run.strip() else "(no run)"
            label = first if len(first) <= 56 else first[:56].rsplit(" ", 1)[0] + " ..."
        return "%s: %s / %s: %s" % (self.repo.name, wf, self.job_name or self.job_id, label)


def load_yaml(path):
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            return yaml.safe_load(fh)
    except (OSError, yaml.YAMLError):
        return None


def wf_on(doc):
    """Return the workflow's trigger names.

    YAML 1.1 resolves the bare key `on` to the boolean True, so `doc["on"]` is a KeyError on
    every GitHub Actions workflow ever written. PyYAML is correct and the workflow schema is
    the odd one out; this looks under both keys rather than pretending the field is missing —
    which would make every job's `github.event_name` gate unevaluable.
    """
    node = doc.get("on", doc.get(True))
    if isinstance(node, dict):
        return set(node.keys())
    if isinstance(node, list):
        return set(node)
    if isinstance(node, str):
        return {node}
    return set()


def discover_repos(estate_root, max_depth=3):
    """Every git repository under the estate root, enumerated rather than listed.

    ADR-0035's rule applied to a list instead of a count: a hardcoded repository list goes
    stale the day a repository is added, and does it silently — the new repository is simply
    never scanned and the report keeps printing a confident total.
    """
    found = []
    root_depth = estate_root.rstrip(os.sep).count(os.sep)
    for dirpath, dirnames, _files in os.walk(estate_root):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS and not d.startswith(".git")]
        if dirpath.count(os.sep) - root_depth >= max_depth:
            dirnames[:] = []
        if os.path.isdir(os.path.join(dirpath, ".git")):
            found.append(dirpath)
            dirnames[:] = []   # a repository inside a repository is not part of the estate
    return sorted(found)


def load_repo(path):
    repo = Repo(path)
    man = load_yaml(os.path.join(path, "manifest.yaml"))
    if isinstance(man, dict):
        repo.name = man.get("name") or repo.dirname
        repo.cls = man.get("class") or "unknown"
        repo.stack = man.get("stack") or "unknown"

    wf_dir = os.path.join(path, ".github", "workflows")
    if os.path.isdir(wf_dir):
        for fn in sorted(os.listdir(wf_dir)):
            if not fn.endswith((".yml", ".yaml")):
                continue
            full = os.path.join(wf_dir, fn)
            doc = load_yaml(full)
            if not isinstance(doc, dict):
                continue
            triggers = wf_on(doc)
            repo.callable_workflows[fn] = (full, doc)
            if triggers - {"workflow_call"}:
                repo.workflows.append((full, doc, triggers))
                repo.texts["%s:.github/workflows/%s" % (repo.name, fn)] = repo.read(
                    os.path.join(".github", "workflows", fn))
            # A `workflow_call`-only file is a pipeline this repository PROVIDES, not one it
            # runs on itself. infra-ci hosts rust-ci.yml, which names TST-0022 in a comment
            # and a step name; searching it as infra-ci's own text reported infra-ci as
            # running static analysis on its own code, which it does not — its ci.yml runs
            # shellcheck and the knowledge gates and nothing else. Callers still see it,
            # because it is resolved into THEIR step list.

    # CONFIG ONLY, and TESTING.md is deliberately absent. It is the most tempting file in
    # the estate to read — it states which tiers a repository believes it runs — and it is
    # prose. A signature probe that read it would report tool-forge as running TST-0001
    # because its TESTING.md says so, which is a claim being counted as its own evidence.
    for rel in ("package.json", "Cargo.toml", "pnpm-workspace.yaml", "manifest.yaml",
                "playwright.config.ts", "vitest.config.ts", ".config/nextest.toml",
                "deny.toml"):
        if repo.has(rel):
            repo.texts["%s:%s" % (repo.name, rel)] = repo.read(rel)

    if repo.has("package.json"):
        try:
            repo.pkg_scripts = json.loads(repo.read("package.json")).get("scripts") or {}
        except (ValueError, AttributeError):
            repo.pkg_scripts = {}
    return repo


# --- pipeline resolution ---------------------------------------------------

USES_RE = re.compile(r"^[^/]+/([^/]+)/\.github/workflows/([^@]+)@(.+)$")


def truthy(val):
    if isinstance(val, bool):
        return val
    if val is None:
        return False
    return str(val).strip() not in ("", "false", "0")


def eval_gate(expr, inputs, triggers):
    """Resolve a job or step `if:` to active | off | unknown.

    Only the forms the estate's workflows actually use are evaluated, and anything else is
    `unknown` rather than assumed active. Assuming active is how a gated job comes to be
    reported as coverage; assuming off is how a running job comes to be reported as missing.
    Neither guess is available here, so the third answer has to exist.
    """
    if expr is None:
        return "active", ""
    text = str(expr).strip()
    shown = text
    m = re.fullmatch(r"\$\{\{(.*)\}\}", text, re.S)
    if m:
        text = m.group(1).strip()
    if text == "always()":
        return "active", shown

    m = re.fullmatch(r"inputs\.([A-Za-z0-9_-]+)", text)
    if m:
        return ("active" if truthy(inputs.get(m.group(1))) else "off"), shown

    m = re.fullmatch(r"inputs\.([A-Za-z0-9_-]+)\s*(==|!=)\s*'([^']*)'", text)
    if m:
        key, op, lit = m.groups()
        cur = inputs.get(key)
        cur = "" if cur is None else str(cur)
        same = cur == lit
        return ("active" if (same if op == "==" else not same) else "off"), shown

    m = re.fullmatch(r"github\.event_name\s*(==|!=)\s*'([^']+)'", text)
    if m:
        op, lit = m.groups()
        present = lit in triggers
        return ("active" if (present if op == "==" else not present) else "off"), shown

    return "unknown", shown


def combine(job_state, step_state):
    if "off" in (job_state, step_state):
        return "off"
    if "unknown" in (job_state, step_state):
        return "unknown"
    return "active"


def expand_jobs(repo, by_dirname):
    """Walk every entry workflow, following `uses:` into reusable workflows.

    Depth is capped at one hop because that is the shape ADR-0021 defines (a thin caller
    calling a shared pipeline) and because an uncapped walk over a cyclic `uses:` graph
    hangs. A second hop would be reported as unresolved rather than silently dropped.
    """
    for wf_path, doc, triggers in repo.workflows:
        jobs = doc.get("jobs") or {}
        if not isinstance(jobs, dict):
            continue
        for job_id, job in jobs.items():
            if not isinstance(job, dict):
                continue
            uses = job.get("uses")
            if isinstance(uses, str):
                m = USES_RE.match(uses.strip())
                target = by_dirname.get(m.group(1)) if m else None
                if not m or target is None or m.group(2) not in target.callable_workflows:
                    repo.unresolved.append((os.path.basename(wf_path), job_id, uses.strip()))
                    continue
                sub_path, sub_doc = target.callable_workflows[m.group(2)]
                inputs = defaults_of(sub_doc)
                passed = job.get("with") or {}
                if isinstance(passed, dict):
                    inputs.update(passed)
                _collect(repo, wf_path, sub_path, sub_doc, triggers, inputs)
            else:
                _collect_job(repo, wf_path, wf_path, job_id, job, triggers, {})


def defaults_of(doc):
    node = doc.get("on", doc.get(True))
    out = {}
    if isinstance(node, dict):
        call = node.get("workflow_call") or {}
        for key, spec in (call.get("inputs") or {}).items():
            out[key] = (spec or {}).get("default")
    return out


def _collect(repo, entry_wf, source_wf, doc, triggers, inputs):
    for job_id, job in (doc.get("jobs") or {}).items():
        if isinstance(job, dict):
            _collect_job(repo, entry_wf, source_wf, job_id, job, triggers, inputs)


def _collect_job(repo, entry_wf, source_wf, job_id, job, triggers, inputs):
    job_state, job_gate = eval_gate(job.get("if"), inputs, triggers)
    steps = job.get("steps")
    if not isinstance(steps, list):
        steps = []
    for step in steps:
        if not isinstance(step, dict):
            continue
        step_state, step_gate = eval_gate(step.get("if"), inputs, triggers)
        gate = " and ".join(g for g in (job_gate, step_gate) if g)
        repo.steps.append(ExecStep(
            repo, entry_wf, source_wf, job_id, job.get("name"), step.get("name"),
            step.get("run"), step.get("uses"),
            combine(job_state, step_state), gate))


# --- probe helpers ---------------------------------------------------------

def find_steps(repo, pattern, states=("active",)):
    """Matching steps, with step-level matches ordered ahead of job-name-only matches.

    Because the job name is part of `blob`, a pattern naming a job matches EVERY step in it —
    and the first is almost always `actions/checkout`. The row then cited a checkout as the
    evidence that a regression check runs, which is a true verdict with a useless basis, and
    a basis nobody can act on is the part of this report that has failed."""
    rx = re.compile(pattern, re.I)
    hits = [s for s in repo.steps if s.state in states and rx.search(s.blob)]
    precise = [s for s in hits
               if rx.search("\n".join((s.name, s.run, s.uses)))]
    return precise + [s for s in hits if s not in precise]


def grep_tree(repo, subdir, pattern, exts, limit=1):
    """Count files under `subdir` whose text matches. Used for file-layout probes only.

    Source code is read ONLY here and ONLY for patterns STD-0001 makes structural
    (`#[cfg(test)]` is the unit tier's definition, not a hint). Tool-name searches
    deliberately never touch source: a crate named in a comment is not a suite.
    """
    rx = re.compile(pattern)
    hits = []
    base = os.path.join(repo.path, subdir)
    for dirpath, dirnames, files in os.walk(base):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in files:
            if exts and not fn.endswith(exts):
                continue
            try:
                with open(os.path.join(dirpath, fn), "r", encoding="utf-8",
                          errors="replace") as fh:
                    if rx.search(fh.read()):
                        hits.append(os.path.relpath(os.path.join(dirpath, fn), repo.path))
            except OSError:
                continue
            if len(hits) >= limit:
                return hits
    return hits


def sig_hits(repo, pattern):
    """Where a tool signature appears among the repository's CONFIG files and pipeline.

    Config and pipeline only. Searching README prose or source comments would turn every
    mention of a tool into evidence that it runs, which is the overclaim this report is for.
    """
    rx = re.compile(pattern, re.I)
    out = []
    for label, text in repo.texts.items():
        if rx.search(text or ""):
            out.append(label)
    for s in repo.steps:
        if s.state == "active" and rx.search(s.blob):
            out.append(s.where())
    return sorted(set(out))


# --- probes ----------------------------------------------------------------
# A probe answers, for ONE repository, "does this test type run here" and returns
# (verdict, confidence, basis). Basis is printed verbatim: it is the whole value of this
# report, because it is what lets a reader disagree with a row.


class F:
    __slots__ = ("repo", "verdict", "conf", "basis")

    def __init__(self, repo, verdict, conf, basis):
        self.repo, self.verdict, self.conf, self.basis = repo, verdict, conf, basis


def probe_annotation(tst, repo):
    """The only non-inferred probe: the pipeline names the TST- id itself.

    Runs for every type before any other probe. It is also the migration path out of this
    script's heuristics — annotate a job with its TST- id and this row stops being a guess.
    """
    hits = sig_hits(repo, r"\b%s\b" % re.escape(tst["test_type"]))
    if hits:
        return F(repo, RUNS, DECLARED, "pipeline names %s: %s"
                 % (tst["test_type"], "; ".join(hits[:3])))
    return None


NEXTEST_RUN = r"cargo\s+nextest\s+run"
NEXTEST_INTEG = r"cargo\s+nextest\s+run[^\n]*--profile\s+integration"


def _ts_unit_runner(repo):
    """The one TypeScript test command in this estate, with its evidence path.

    Returns (step, state, note). `state` is the step's gate state, and the caller must not
    collapse `unknown` into absent. nextjs-ci.yml gates its Test step on
    `steps.has_tests.outputs.yes == 'true'` — a RUNTIME step output, which this report cannot
    evaluate. Treating that as "no pipeline step invokes it" was the first thing this script
    got wrong about its own estate: vitest plainly does run in console-docket's CI. An
    unevaluable gate is reported as such, with the gate text, and never as an absence.
    """
    body = (repo.pkg_scripts or {}).get("test", "")
    if "vitest" not in body:
        return None, "off", "no package.json `test` script running vitest"
    pattern = r"(pnpm|npm|yarn)\s+(run\s+)?test\b"
    active = find_steps(repo, pattern)
    if active:
        return active[0], "active", ""
    gated = find_steps(repo, pattern, states=("unknown", "off"))
    if gated:
        return gated[0], gated[0].state, gated[0].gate
    return None, "off", ("package.json has `test: %s` but no step in the resolved pipeline "
                         "invokes it" % body)


def probe_unit(tst, repo):
    if repo.stack == "rust":
        run = find_steps(repo, NEXTEST_RUN)
        if not run:
            return F(repo, NOT_FOUND, DIRECT, "no pipeline step runs `cargo nextest run`")
        junit = "[profile.ci.junit]" in repo.read(".config/nextest.toml")
        inline = grep_tree(repo, "src", r"#\[cfg\(test\)\]", (".rs",))
        if not inline:
            return F(repo, NOT_FOUND, HEURISTIC,
                     "`cargo nextest run` runs but no src/**.rs contains `#[cfg(test)]`, "
                     "which is STD-0001's definition of the unit tier")
        return F(repo, RUNS, HEURISTIC,
                 "%s + `#[cfg(test)]` in %s%s | HEURISTIC: one nextest invocation runs "
                 "every Rust tier at once, so this identifies the unit tier by STD-0001's "
                 "file layout, not by a distinct command"
                 % (run[0].where(), inline[0],
                    "" if junit else " (no [profile.ci.junit]; evidence `junit.xml` absent)"))
    if repo.stack == "typescript":
        step, state, note = _ts_unit_runner(repo)
        if step is None or state == "off":
            return F(repo, NOT_FOUND, DIRECT, note)
        junit = "junit" in repo.read("vitest.config.ts")
        gate = ("" if state == "active" else
                " | gate `%s` is a runtime value this report cannot evaluate; counted as "
                "running because package.json declares the `test` script that gate tests "
                "for — read the gate, not this verdict, if it matters" % note)
        return F(repo, RUNS, HEURISTIC,
                 "%s -> `%s` (vitest)%s%s | HEURISTIC: vitest runs unit and component under "
                 "one include pattern; nothing in the repository separates the two tiers"
                 % (step.where(), repo.pkg_scripts.get("test", ""),
                    "" if junit else " (no junit reporter; evidence `junit.xml` absent)",
                    gate))
    return None


def probe_component(tst, repo):
    if repo.stack == "rust":
        run = find_steps(repo, NEXTEST_RUN)
        if not run:
            return F(repo, NOT_FOUND, DIRECT, "no pipeline step runs `cargo nextest run`")
        files = [f for f in (grep_tree(repo, "tests", r".", (".rs",), limit=50) or [])
                 if not f.startswith(os.path.join("tests", "it"))]
        if not files:
            return F(repo, NOT_FOUND, HEURISTIC,
                     "nextest runs but tests/ holds no .rs outside tests/it/, which is "
                     "STD-0001's location for the component tier")
        return F(repo, RUNS, HEURISTIC,
                 "%s picks up %d file(s) under tests/ (e.g. %s) | HEURISTIC: STD-0001 places "
                 "the component tier in tests/; a file there could be any tier and nothing "
                 "in the repository labels it"
                 % (run[0].where(), len(files), files[0]))
    if repo.stack == "typescript":
        step, state, note = _ts_unit_runner(repo)
        if step is None or state == "off":
            return F(repo, NOT_FOUND, DIRECT, note)
        return F(repo, UNKNOWN, HEURISTIC,
                 "vitest runs (%s) but its include pattern does not distinguish component "
                 "from unit, and no second command, config or directory convention exists "
                 "for it. TESTING.md claims both tiers run; prose is a claim, not evidence, "
                 "and is not read by this report." % step.where())
    return None


def probe_integration(tst, repo):
    if repo.stack == "rust":
        active = find_steps(repo, NEXTEST_INTEG)
        gated = find_steps(repo, NEXTEST_INTEG, states=("off", "unknown"))
        it_dir = os.path.isdir(os.path.join(repo.path, "tests", "it"))
        if active:
            return F(repo, RUNS, DIRECT, "%s (gate: %s)"
                     % (active[0].where(), active[0].gate or "none"))
        if gated:
            g = gated[0]
            return F(repo, NOT_FOUND, DIRECT if g.state == "off" else HEURISTIC,
                     "the integration step exists (%s) but its gate `%s` resolves %s against "
                     "this caller's inputs; tests/it/ %s"
                     % (g.where(), g.gate, g.state,
                        "exists" if it_dir else "does not exist"))
        return F(repo, NOT_FOUND, DIRECT,
                 "no step in the resolved pipeline runs `--profile integration`")
    if repo.stack == "typescript":
        names = [n for n in repo.pkg_scripts if "integration" in n or n.endswith(":it")]
        if names:
            steps = find_steps(repo, r"(pnpm|npm|yarn)\s+(run\s+)?(%s)\b"
                               % "|".join(re.escape(n) for n in names))
            if steps:
                return F(repo, RUNS, HEURISTIC, steps[0].where())
            return F(repo, NOT_FOUND, DIRECT,
                     "package.json has %s but no pipeline step invokes it" % names)
        return F(repo, NOT_FOUND, DIRECT,
                 "package.json declares no integration script and no pipeline step runs one")
    return None


def probe_regression(tst, repo):
    steps = find_steps(repo, r"check-regression\.sh|regression test on a fix|"
                             r"Regression test on a fix")
    if steps:
        return F(repo, RUNS, DIRECT,
                 "%s (gate: %s) | CAVEAT, from the workflow's own comment: this is the "
                 "mechanical half only — it fails a `fix:` commit whose diff touches no test "
                 "file. It does not run a regression suite and does not claim to."
                 % (steps[0].where(), steps[0].gate or "none"))
    gated = find_steps(repo, r"check-regression\.sh", states=("off", "unknown"))
    if gated:
        return F(repo, NOT_FOUND, DIRECT, "the regression job exists (%s) but its gate `%s` "
                 "resolves %s here" % (gated[0].where(), gated[0].gate, gated[0].state))
    return F(repo, NOT_FOUND, DIRECT,
             "no step in the resolved pipeline runs the regression check")


def probe_static_analysis(tst, repo):
    clippy = find_steps(repo, r"cargo\s+clippy")
    if clippy:
        report = find_steps(repo, r"message-format=json|\.sarif|sarif")
        return F(repo, RUNS, DIRECT,
                 "%s%s | the registry's own note and check-registry's `partial: true` on "
                 "test.static_analysis_runs_on_pull_request both say the same thing: clippy "
                 "is the analysis, and nothing verifies a pipeline runs one AT ALL"
                 % (clippy[0].where(),
                    "; report step: " + report[0].where() if report else
                    "; no machine-readable report step (evidence `SARIF report` absent)"))
    # shellcheck belongs here and not in the `not found` branch. knowledge's pipeline runs
    # it over every script in the estate; calling that "no static analysis" would be a
    # negative overclaim, and calling it TST-0022 would be a positive one. The registry's
    # evidence field asks for a SARIF report and none of these produce one, so the only
    # answer this report is entitled to give is that it will not decide.
    lint = find_steps(repo, r"(pnpm|npm|yarn)\s+(run\s+)?lint\b|eslint|shellcheck")
    if lint:
        return F(repo, UNKNOWN, HEURISTIC,
                 "a linter runs (%s) but emits no SARIF and carries no TST- id. Whether a "
                 "lint pass is `static application security analysis` is a judgement this "
                 "report will not make on the estate's behalf" % lint[0].where())
    return F(repo, NOT_FOUND, DIRECT,
             "no clippy, eslint, shellcheck or SARIF-producing step in the resolved pipeline")


def probe_dependency_scan(tst, repo):
    # renovate is DELIBERATELY not a signature here. infra-ci runs it, and it updates
    # dependencies rather than scanning them for advisories — counting it would turn a
    # bot that opens pull requests into evidence of a security control.
    steps = find_steps(repo, r"cargo-deny|cargo\s+deny|cargo\s+audit|osv-scanner|"
                             r"(pnpm|npm|yarn)\s+audit|trivy\s+fs")
    if steps:
        return F(repo, RUNS, DIRECT, "%s%s" % (
            steps[0].where(),
            "" if repo.has("deny.toml") else " (no deny.toml in the repository)"))
    return F(repo, NOT_FOUND, DIRECT,
             "no advisory-scanning step (cargo-deny, cargo-audit, osv-scanner, "
             "`pnpm audit`) in the resolved pipeline; renovate is not counted, it updates "
             "dependencies rather than scanning them")


def probe_secret_scan(tst, repo):
    steps = find_steps(repo, r"secret-scan\.sh|gitleaks|trufflehog")
    if steps:
        return F(repo, RUNS, DIRECT, steps[0].where())
    return F(repo, NOT_FOUND, DIRECT, "no secret-scanning step in the resolved pipeline")


def _check_impl_sites(root, check_id):
    """Which knowledge script implements a check id, ignoring its own test suite.

    scripts/test-checks.sh names every check id it plants a defect for, so counting it as an
    implementation would report every declared check as implemented — the registry-versus-
    reality gap this estate has been caught by before.
    """
    sites = []
    sdir = os.path.join(root, "scripts")
    for dirpath, dirnames, files in os.walk(sdir):
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fn in files:
            if fn == "test-checks.sh" or not fn.endswith((".sh", ".py")):
                continue
            full = os.path.join(dirpath, fn)
            try:
                with open(full, "r", encoding="utf-8", errors="replace") as fh:
                    if check_id in fh.read():
                        sites.append(os.path.relpath(full, root))
            except OSError:
                continue
    return sorted(sites)


def probe_via_declared_checks(tst, repo, root):
    """For a type whose whole meaning is enforcement that already runs (TST-0061).

    The chain is type -> `checks:` -> a knowledge script implementing that id -> a pipeline
    step invoking that script. Every link is real; what makes it heuristic is the last
    inference, that a check running is the same thing as the test type running. For
    boundary-conformance the registry says exactly that, which is why the probe exists — and
    it reports how many of the declared checks it could account for, never just the first.
    """
    ids = tst.get("checks") or []
    if not ids:
        return None
    accounted = []
    for cid in ids:
        for script in _check_impl_sites(root, cid):
            base = os.path.basename(script)
            steps = find_steps(repo, re.escape(base))
            if steps:
                accounted.append((cid, base, steps[0].where()))
                break
    if not accounted:
        return F(repo, NOT_FOUND, HEURISTIC,
                 "none of its %d declared check(s) (%s) is implemented in a knowledge script "
                 "that this pipeline runs" % (len(ids), ", ".join(ids)))
    return F(repo, RUNS, HEURISTIC,
             "%d of %d declared checks accounted for: %s | HEURISTIC: a check running is not "
             "the same as a suite running. It is counted here because the registry entry says "
             "this type IS a name for enforcement that already runs; the unaccounted checks "
             "(%s) are not implemented in any script this pipeline invokes"
             % (len(accounted), len(ids),
                "; ".join("%s via %s in %s" % a for a in accounted),
                ", ".join(c for c in ids if c not in [a[0] for a in accounted]) or "none"))


# --- tool-signature probes -------------------------------------------------
# For a type whose implementation would necessarily name a known tool. These are the weakest
# probes in the file and are always reported `heuristic`: a bespoke suite that names no
# recognised tool is invisible to them. Their value is that a `not found` row states WHAT was
# searched for, so the reader can see the shape of the blind spot instead of trusting a
# verdict. A type with no entry here and no specific probe stays `cannot determine`.
#
# HARNESS, separately from the pipeline: a repository can carry a complete test harness that
# no pipeline step invokes. console-docket does — Playwright, three specs, a `test:e2e`
# script, and no CI job. Reporting that as plain `not found` would lose the only interesting
# thing about it, so the harness is detected and named in the basis.

SIGNATURES = {
    "TST-0006": (r"@playwright/test|playwright\s+test|cypress|webdriverio",
                 "Playwright, Cypress, WebdriverIO",
                 ("playwright.config.ts", "cypress.config.ts", "e2e")),
    "TST-0011": (r"\bproptest\b|\bquickcheck\b|fast-check",
                 "proptest, quickcheck, fast-check", ()),
    "TST-0012": (r"cargo-fuzz|cargo\s+fuzz|libfuzzer|honggfuzz|\bafl\b",
                 "cargo-fuzz, libFuzzer, honggfuzz, AFL", ("fuzz",)),
    "TST-0013": (r"cargo-mutants|cargo\s+mutants|\bmutmut\b|stryker",
                 "cargo-mutants, mutmut, Stryker", ()),
    "TST-0015": (r"\binsta\b|toMatchSnapshot|__snapshots__|\.snap\b",
                 "insta, vitest snapshots", ("__snapshots__",)),
    "TST-0025": (r"\bzap\b|nuclei|burp|dastardly",
                 "OWASP ZAP, nuclei, Burp", ()),
    "TST-0028": (r"\bcriterion\b|cargo\s+bench|\bdivan\b|\bhyperfine\b",
                 "criterion, divan, cargo bench, hyperfine", ("benches",)),
    "TST-0029": (r"\bk6\b|gatling|locust|\bvegeta\b|artillery|jmeter",
                 "k6, Gatling, Locust, vegeta, Artillery, JMeter", ()),
    "TST-0042": (r"\btfsec\b|checkov|terrascan|tflint|terraform\s+validate",
                 "tfsec, Checkov, terrascan, tflint, terraform validate", ()),
    "TST-0043": (r"\bconftest\b|\bopa\b|\bregula\b|kyverno|gatekeeper",
                 "conftest, OPA, regula, Kyverno, Gatekeeper", ()),
    "TST-0044": (r"\btrivy\b|\bgrype\b|\bsyft\b|docker\s+scout|dockle",
                 "trivy, grype, syft, docker scout, dockle", ()),
    "TST-0046": (r"oasdiff|openapi-diff|buf\s+breaking|\bcargo-semver-checks\b",
                 "oasdiff, openapi-diff, buf breaking, cargo-semver-checks", ()),
    "TST-0048": (r"@playwright/test|playwright\s+test|selenium|browserstack",
                 "Playwright, Selenium, BrowserStack",
                 ("playwright.config.ts", "e2e")),
    "TST-0049": (r"toHaveScreenshot|\bpercy\b|backstop|\bloki\b|reg-suit",
                 "Playwright toHaveScreenshot, Percy, BackstopJS, Loki", ()),
    "TST-0050": (r"\baxe-core\b|@axe-core|\bpa11y\b|lighthouse\s+ci|\blhci\b",
                 "axe-core, pa11y, Lighthouse CI", ()),
    "TST-0052": (r"\bcosign\b|slsa-verify|slsa-github-generator|in-toto|\bwitness\b",
                 "cosign, SLSA verifier, in-toto", ()),
    "TST-0053": (r"reprotest|diffoscope|SOURCE_DATE_EPOCH",
                 "reprotest, diffoscope, SOURCE_DATE_EPOCH", ()),
}


def probe_signature(tst, repo):
    entry = SIGNATURES.get(tst["test_type"])
    if entry is None:
        return None
    pattern, human, harness_paths = entry

    active = find_steps(repo, pattern)
    if active:
        return F(repo, RUNS, HEURISTIC,
                 "%s | HEURISTIC: matched on tool name (%s), not on a declared test type"
                 % (active[0].where(), human))

    gated = find_steps(repo, pattern, states=("off", "unknown"))
    harness = [p for p in harness_paths if repo.has(p)]
    scripts = sorted(n for n, b in repo.pkg_scripts.items()
                     if re.search(pattern, b or "", re.I))
    if scripts:
        harness.append("package.json script(s): " + ", ".join(scripts))
    cfg = [lbl for lbl, txt in repo.texts.items()
           if re.search(pattern, txt or "", re.I) and not lbl.endswith((".yml", ".yaml"))]

    if gated:
        g = gated[0]
        return F(repo, NOT_FOUND, DIRECT if g.state == "off" else HEURISTIC,
                 "the step exists (%s) but its gate `%s` resolves %s against this caller's "
                 "inputs, and no trigger in this repository sets it. Declared, never fired."
                 % (g.where(), g.gate, g.state))
    if harness or cfg:
        # cfg only adds a line when nothing more specific was found; otherwise it repeats
        # package.json as both "script(s): test:e2e" and "console-docket:package.json".
        return F(repo, NOT_FOUND, DIRECT,
                 "harness present (%s) and NO step in the resolved pipeline invokes it — a "
                 "suite that exists and never runs automatically"
                 % "; ".join(harness or cfg))
    return F(repo, NOT_FOUND, HEURISTIC,
             "searched the resolved pipeline and this repository's config files for %s — "
             "none present. HEURISTIC: a bespoke suite naming no known tool would not be "
             "seen by this probe." % human)


def probe_runtime_version(tst, repo):
    """TST-0047 runtime-version. rust-ci has an `msrv` job; nothing else matrixes a runtime."""
    active = find_steps(repo, r"msrv|matrix\.node|node-version:\s*\[")
    if active:
        return F(repo, RUNS, HEURISTIC, "%s | HEURISTIC: matched on an msrv/runtime-matrix "
                 "step name" % active[0].where())
    gated = find_steps(repo, r"msrv", states=("off", "unknown"))
    if gated:
        g = gated[0]
        return F(repo, NOT_FOUND, DIRECT if g.state == "off" else HEURISTIC,
                 "the msrv job exists (%s); its gate `%s` resolves %s because this caller "
                 "passes no msrv input" % (g.where(), g.gate, g.state))
    return F(repo, NOT_FOUND, HEURISTIC,
             "no msrv job and no runtime matrix in the resolved pipeline")


# Dispatch. Order matters only in that probe_annotation runs first for every type.
PROBES = {
    "TST-0001": probe_unit,
    "TST-0002": probe_component,
    "TST-0003": probe_integration,
    "TST-0010": probe_regression,
    "TST-0022": probe_static_analysis,
    "TST-0023": probe_dependency_scan,
    "TST-0024": probe_secret_scan,
    "TST-0047": probe_runtime_version,
}

# Types answered by walking their declared `checks:` into the knowledge scripts.
CHECK_CHAIN = {"TST-0061"}


# --- aggregation -----------------------------------------------------------

CONF_RANK = {DECLARED: 3, DIRECT: 2, HEURISTIC: 1}


def in_scope(tst, repos):
    """Repositories a type could possibly run in, per its `stacks:` field.

    A type limited to `rust` cannot be missing from a TypeScript repository, and counting it
    as missing there would inflate every `not found` row with places it was never owed.
    `stacks: null` in the registry means stack-agnostic, so every repository is in scope —
    including the two with no Rust or TypeScript at all, which is correct for a type like
    secret-scan.
    """
    stacks = tst.get("stacks")
    if not stacks or "any" in stacks:
        return list(repos)
    return [r for r in repos if r.stack in stacks]


def assess(tst, repos, root):
    scope = in_scope(tst, repos)
    if not scope:
        return (NOT_FOUND, DIRECT,
                [F(None, NOT_FOUND, DIRECT,
                   "no repository in the estate declares a stack this type applies to "
                   "(registry limits it to %s)" % tst.get("stacks"))])

    findings = []
    for repo in scope:
        f = probe_annotation(tst, repo)
        if f is None and tst["test_type"] in CHECK_CHAIN:
            f = probe_via_declared_checks(tst, repo, root)
        if f is None:
            probe = PROBES.get(tst["test_type"])
            if probe is not None:
                f = probe(tst, repo)
        if f is None:
            f = probe_signature(tst, repo)
        if f is None:
            f = F(repo, UNKNOWN, HEURISTIC,
                  "NOT PROBED. No probe in this report can distinguish this type from the "
                  "others sharing its runner, and no tool signature is defined for it. "
                  "Nothing was searched for, so nothing is claimed.")
        findings.append(f)

    positives = [f for f in findings if f.verdict == RUNS]
    if positives:
        # Strongest positive decides: the claim "it runs somewhere" needs one good witness.
        conf = max(positives, key=lambda f: CONF_RANK[f.conf]).conf
        return RUNS, conf, findings
    if all(f.verdict == NOT_FOUND for f in findings):
        # Weakest negative decides: the claim "it runs nowhere" is only as strong as the
        # flimsiest repository it rests on. One heuristic `not found` makes the row heuristic.
        conf = min(findings, key=lambda f: CONF_RANK[f.conf]).conf
        return NOT_FOUND, conf, findings
    return UNKNOWN, HEURISTIC, findings


# --- rendering -------------------------------------------------------------

def wrap(text, width, indent):
    out, line = [], indent
    for word in text.split():
        if len(line) + len(word) + 1 > width and line.strip():
            out.append(line.rstrip())
            line = indent + word + " "
        else:
            line += word + " "
    if line.strip():
        out.append(line.rstrip())
    return out


MARK = {RUNS: "RUN", NOT_FOUND: "NOT RUN", UNKNOWN: "??"}
CONF_TAG = {DECLARED: "declared", DIRECT: "direct", HEURISTIC: "HEURISTIC"}


def conf_tag(row):
    # A `cannot determine` row has no confidence to report: there is no finding to be
    # confident about. Printing HEURISTIC there suggests an inference was made.
    return "-" if row["verdict"] == UNKNOWN else CONF_TAG[row["conf"]]


def render(rows, repos, args, meta):
    w = args.width
    print("Paridyum test-type coverage — which declared test types actually run, and where")
    print("=" * min(w, 92))
    print("  registry   %s  (version %s, %d test types)"
          % (meta["registry"], meta["version"], meta["total"]))
    print("  estate     %s  (%d repositories, discovered not listed)"
          % (meta["estate"], len(repos)))
    print("  read-only  this script opens no file for writing")
    print()
    print("  A ROW IS ONLY AS GOOD AS ITS BASIS, WHICH IS PRINTED UNDER IT.")
    print("    RUN      evidence found that it executes in an automated pipeline")
    print("    NOT RUN  probed, and no automated execution found — read the basis for what")
    print("             was searched; a suite naming no known tool can hide from this")
    print("    ??       cannot determine. Nothing was probed, or the evidence is ambiguous.")
    print("    declared the pipeline names the TST- id itself — the only non-inferred proof")
    print("    direct   the pipeline's step list and literal gates settle it")
    print("    HEURISTIC inferred from a tool name or file layout; can be wrong either way")
    print()
    print("REPOSITORIES")
    for r in repos:
        print("  %-16s class=%-8s stack=%-11s entry-workflows=%-2d resolved-steps=%d"
              % (r.name, r.cls, r.stack, len(r.workflows), len(r.steps)))
    unres = [(r.name,) + u for r in repos for u in r.unresolved]
    if unres:
        print()
        print("  UNRESOLVED `uses:` — these pipelines were NOT read, so anything they run is")
        print("  invisible to every row below:")
        for name, wf, job, ref in unres:
            print("    %s %s / %s -> %s" % (name, wf, job, ref))
    print()
    print("  Reusable workflows were resolved against the LOCAL sibling checkout of the named")
    print("  repository. Callers pin a tag (ADR-0021); the local checkout is not necessarily")
    print("  what that tag points at, so a row can be right about this working tree and wrong")
    print("  about what GitHub ran.")

    for status in ("required", "recommended", "deferred"):
        group = [r for r in rows if r["status"] == status]
        if not group:
            continue
        print()
        print("%s — %d type(s)" % (status.upper(), len(group)))
        print("-" * min(w, 92))
        for row in group:
            print("  %-9s %-26s %-11s %-7s %-9s %s"
                  % (row["id"], row["name"], row["tier"],
                     MARK[row["verdict"]], conf_tag(row),
                     ", ".join(row["where"]) or "-"))
            groups = []
            for f in row["findings"]:
                who = f.repo.name if f.repo is not None else "estate"
                if groups and groups[-1][0] == (f.verdict, f.basis):
                    groups[-1][1].append(who)
                else:
                    groups.append(((f.verdict, f.basis), [who]))
            # Identical findings are merged. Thirty types are answered the same way in every
            # repository, and printing that paragraph five times per row hides the rows where
            # the repositories genuinely disagree — which are the only interesting ones.
            for (verdict, basis), whos in groups:
                head = whos[0] if len(whos) == 1 else "%s (%d repos)" % (", ".join(whos),
                                                                        len(whos))
                for line in wrap("%s [%s] %s" % (head, verdict, basis), w, " " * 8):
                    print(line)


def summarise(rows, w):
    print()
    print("SUMMARY")
    print("=" * min(w, 92))
    order = ("required", "recommended", "deferred")
    print("  %-13s %5s %8s %9s %6s" % ("status", "RUN", "NOT RUN", "cannot", "total"))
    for status in order:
        group = [r for r in rows if r["status"] == status]
        if not group:
            continue
        counts = {v: sum(1 for r in group if r["verdict"] == v)
                  for v in (RUNS, NOT_FOUND, UNKNOWN)}
        print("  %-13s %5d %8d %9d %6d"
              % (status, counts[RUNS], counts[NOT_FOUND], counts[UNKNOWN], len(group)))
    tot = {v: sum(1 for r in rows if r["verdict"] == v) for v in (RUNS, NOT_FOUND, UNKNOWN)}
    print("  %-13s %5d %8d %9d %6d"
          % ("all", tot[RUNS], tot[NOT_FOUND], tot[UNKNOWN], len(rows)))

    req = [r for r in rows if r["status"] == "required"]
    print()
    print("  REQUIRED, the only tally that binds anything today:")
    print("    running          %d of %d" % (sum(1 for r in req if r["verdict"] == RUNS),
                                             len(req)))
    print("    NOT running      %d of %d%s"
          % (sum(1 for r in req if r["verdict"] == NOT_FOUND), len(req),
             "  -> " + ", ".join("%s %s" % (r["id"], r["name"])
                                 for r in req if r["verdict"] == NOT_FOUND)
             if any(r["verdict"] == NOT_FOUND for r in req) else ""))
    print("    UNDETERMINABLE   %d of %d%s"
          % (sum(1 for r in req if r["verdict"] == UNKNOWN), len(req),
             "  -> " + ", ".join("%s %s" % (r["id"], r["name"])
                                 for r in req if r["verdict"] == UNKNOWN)
             if any(r["verdict"] == UNKNOWN for r in req) else ""))
    # Confidence is only meaningful where a verdict was reached. Counting the undetermined
    # rows as "heuristic" put 54 of 62 in that bucket and made the report look like it had
    # guessed at everything, when in fact it had declined to guess at most of it.
    decided = [r for r in rows if r["verdict"] != UNKNOWN]
    print()
    print("  HOW MUCH OF THIS REPORT IS INFERENCE:")
    print("    verdicts reached              %d of %d types" % (len(decided), len(rows)))
    for label, conf in (("of those, DECLARED by the pipeline", DECLARED),
                        ("of those, direct from the pipeline", DIRECT),
                        ("of those, a HEURISTIC             ", HEURISTIC)):
        print("      %-33s %d" % (label, sum(1 for r in decided if r["conf"] == conf)))
    print("    no verdict, nothing probed    %d of %d types"
          % (sum(1 for r in rows if r["verdict"] == UNKNOWN), len(rows)))
    print()
    print("  The undeterminable rows are the honest result, not a gap in the script. There is")
    print("  no declared mapping from a test type to the job that runs it, so most types can")
    print("  only be answered by annotating a job with its TST- id. Every annotation added")
    print("  moves one row from HEURISTIC or ?? to `declared`.")


# --- main ------------------------------------------------------------------

def main(argv):
    # prog is the wrapper, not this file: --root is supplied by the wrapper and a
    # usage line telling a reader to pass it themselves invites the wrong call.
    ap = argparse.ArgumentParser(prog="report-test-coverage.sh", add_help=True)
    ap.add_argument("--root", required=True, help="the knowledge repository")
    ap.add_argument("--estate", default=None,
                    help="estate root to scan (default: the knowledge repo's parent)")
    ap.add_argument("--only", choices=("required", "recommended", "deferred"), default=None)
    ap.add_argument("--tsv", action="store_true", help="one row per type, tab-separated")
    ap.add_argument("--width", type=int, default=92)
    args = ap.parse_args(argv)

    root = os.path.abspath(args.root)
    reg_path = os.path.join(root, "test-registry.yaml")
    if not os.path.isfile(reg_path):
        print("report-test-coverage: no test-registry.yaml under %s" % root, file=sys.stderr)
        return 2
    reg = load_yaml(reg_path)
    if not isinstance(reg, dict) or not isinstance(reg.get("test_types"), list):
        print("report-test-coverage: test-registry.yaml has no test_types list",
              file=sys.stderr)
        return 2

    estate = os.path.abspath(args.estate or os.path.dirname(root))
    repos = [load_repo(p) for p in discover_repos(estate)]
    if not repos:
        print("report-test-coverage: no git repositories found under %s" % estate,
              file=sys.stderr)
        return 2
    by_dirname = {r.dirname: r for r in repos}
    for r in repos:
        expand_jobs(r, by_dirname)

    types = sorted(reg["test_types"], key=lambda t: t.get("test_type", ""))
    rows = []
    for tst in types:
        if args.only and tst.get("status") != args.only:
            continue
        verdict, conf, findings = assess(tst, repos, root)
        rows.append({
            "id": tst.get("test_type"), "name": tst.get("name"),
            "tier": tst.get("tier"), "status": tst.get("status"),
            "stacks": tst.get("stacks") or ["any"],
            "verdict": verdict, "conf": conf, "findings": findings,
            "where": [f.repo.name for f in findings
                      if f.verdict == RUNS and f.repo is not None],
        })

    if args.tsv:
        print("\t".join(("test_type", "name", "tier", "status", "stacks", "verdict",
                         "confidence", "runs_in")))
        for r in rows:
            # `n/a`, not the internal conf value: the text renderer already refuses to put
            # a confidence on a `cannot determine` row, and the machine-readable form must
            # not disagree with it. A consumer reading `heuristic` there would think an
            # inference was attempted.
            print("\t".join((r["id"], r["name"], r["tier"], r["status"],
                             ",".join(r["stacks"]), r["verdict"],
                             "n/a" if r["verdict"] == UNKNOWN else r["conf"],
                             ",".join(r["where"]))))
        return 0

    render(rows, repos, args, {
        "registry": os.path.relpath(reg_path, root),
        "version": reg.get("version"),
        # Computed, never written down (ADR-0035). A literal here would be a fifth copy of a
        # number the file next to it already knows.
        "total": len(reg["test_types"]),
        "estate": estate,
    })
    summarise(rows, args.width)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
