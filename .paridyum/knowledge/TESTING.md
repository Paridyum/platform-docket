# Testing — Paridyum knowledge

Required by [STD-0008](standards/STD-0008-test-tiers-and-gates.md). This repository's subject
is governance, so its "code" is five bash gates, three Python helpers and six YAML/JSON
registries, and its tests are the suite that **plants every defect those gates claim to
reject and watches each one fail.**

That inversion is the point. `README.md`'s rule is *treat a green check as unproven until you
have watched it fail*, and `scripts/test-checks.sh` is that rule as a build step. A check is
counted as implemented in `check-registry.yaml` only if it appears there.

## Tiers

| Tier | What runs | Where |
| ---- | --------- | ----- |
| `pr` | the whole suite — `check-knowledge.sh`, `check-repo.sh`, `check-migrations.sh`, `check-commits.sh`, `check-pr.sh`, secret scan, shellcheck, JSON validity — and `test-checks.sh` **when the change can reach it** (`scripts/check-suite-scope.sh`: a checker, a registry or schema, this file, or an artifact the suite plants into; ADR-0124 addendum) | `.github/workflows/knowledge-check.yml` |
| `merge` | the same set, minus the two pull-request-only gates | same workflow, `push` trigger |
| `nightly` | the suite on main's latest commit, `test-checks.sh` always, identity and regression excepted (they gate a change as it arrives) | same workflow, `schedule` trigger, 03:00 IST |
| `pre-release` | — | nothing released |
| `post-deploy` | — | nothing deployed |

**Every tier is `pr` here, deliberately.** The full suite is seconds of bash with no build, no
container and no network, so the 10-minute budget is not close to binding and splitting it
would add a scheduling decision with nothing to gain. If it ever stops fitting, the migration
fixtures are the first thing to move.

Two gates are `pull_request`-only and report `n/a` rather than passing on a push:
`check-pr.sh` (the review budget needs a base ref) and `check-regression.sh`. Both say so
out loud — a gate that examines nothing must never look like one that passed (ADR-0016).

Test types from [`test-registry.yaml`](test-registry.yaml) that apply here:
**TST-0010** regression, **TST-0024** secret-scan. The Rust functional types do not: there
is no Rust in this repository.

## Not tested

Stated rather than left as a silent gap. Each of these is a decision, and each can be argued
with.

- **The two git-dependent knowledge checks** — locked-artifact immutability and supersession
  detected in a diff. `scripts/test-checks.sh` copies the repository *without* `.git` for
  speed, and both are guarded by `git rev-parse` and skip cleanly. They are the largest
  untested surface in the gate and they protect the rule `CLAUDE.md` calls non-negotiable.
- **`guard-commands.sh`**, and every branch of `guard-locked-decisions.sh` except the errata
  one. Testable only by driving them with synthetic tool-call JSON; `check-registry.yaml`
  records this against `agent.dangerous_commands_escalate`.
  The errata branch (ADR-0031) **is** covered — six cases — because it is the one hook change
  that can only fail open: it exists to *permit* an edit to a locked artifact. Its first
  version reported three false failures, from reading a permitted call's silence through `jq`,
  which produces no output on empty stdin and never applies its default. The remaining branches
  all fail closed, which is why they are lower priority rather than fine.
- **`scripts/bootstrap.sh`.** It installs toolchains onto a real workstation; a test would
  need a disposable machine per platform, and ADR-0006 names three.
- **`checkpoint-session.sh`'s verdict logic**, until 2026-08-08. Its *registration* was covered
  (three cases via `check-repo.sh`) and so was its *bulk guard* (three cases), which made the
  component look tested — the gap was the part that decides what a commit says about itself.
  Two defects survived there: an absent gate recorded as `green`, and a commit made with no
  secret scan leaving no trace, both inside the component that writes the history every other
  audit reads its evidence from. Partial coverage that reads as full is the ADR-0014 defect
  class applied to a test suite. Ten cases now drive it with synthetic payloads against
  throwaway repositories. Still uncovered: the push path, which needs a remote.
- **The checks whose owner is `tool-forge`.** They are declared and unbuilt. Not a testing
  gap — an implementation gap, and `scripts/check-knowledge.sh` prints the split on every run.
- **The CI workflows themselves.** `knowledge-check.yml` and `rust-ci.yml` are validated as
  YAML and read by humans. `rust-ci.yml` **is** executed now — `tool-forge` calls it on every
  push and nine of its jobs run — which closed the hole this bullet was written against. What
  remains untested is the workflow itself: nothing plants a defect in `rust-ci.yml` and watches
  a caller fail, so its jobs are proven only by having run against one real repository.
- **Prose.** Every artifact's argument is unverifiable by construction. That is what
  `status: accepted` means, and why `locked` costs a named check.

## Quarantine

None. No test in this repository has ever failed nondeterministically.

If one does, [STD-0008](standards/STD-0008-test-tiers-and-gates.md) FLAKY-001 to FLAKY-004
apply: quarantined within one working day, with an owner, a first-seen date, an observed
failure rate, a tracking reference and an expiry — recorded in this section, not deleted.

| Test | Owner | First seen | Rate | Tracking | Expires |
| ---- | ----- | ---------- | ---- | -------- | ------- |
| — | | | | | |

## Evidence

`scripts/test-checks.sh` writes **`junit.xml`** — one `<testcase>` per planted defect, with the
failing case's stderr as the failure message. CI uploads it as the `gate-results` artifact on
every run including a red one, which is the run worth keeping.

```sh
./scripts/test-checks.sh              # writes ./junit.xml
JUNIT_OUT=/tmp/gates.xml ./scripts/test-checks.sh
JUNIT_OUT= ./scripts/test-checks.sh   # empty disables it
```

Gitignored: it is generated output, and the ADR-0025 checkpoint hook stages everything.

**Retention is GitHub's default, and that is the weak point.** Artifacts expire; the durable
record is the commit history, because the ADR-0025 `Stop` hook puts every gate's result into
the commit message:

```sh
git log --grep 'gates: .*FAIL'
```

That answers "did it pass" for the repository's lifetime and does **not** answer "which case" —
which is what `junit.xml` adds for as long as the artifact survives. Neither alone is a record;
this section exists to say which question each one answers.

The other gates — `check-knowledge.sh`, `check-repo.sh`, `check-migrations.sh` — still write only
to stdout. They are covered indirectly, since `test-checks.sh` runs each of them and records the
outcome as a case, but a direct failure of one on this repository appears in the suite as a single
failed case rather than as its own diagnosis.
