# Security policy

## Reporting a vulnerability

**Email `rohanbatra@exaflair.com`.** Please do not open a public issue for anything
security-relevant, and please do not disclose publicly until a fix is available.

Include whatever you have — a description, the affected file or workflow, and how you found
it. A rough report is more useful than a polished one that arrives later. If you are not sure
whether something counts, report it; deciding is our job, not yours.

**What to expect:** an acknowledgement within 3 working days, an assessment within 10, and a
fix or a stated position before public disclosure. This is a very small team, so those are
honest targets rather than an SLA. If you have not heard back, assume the mail was missed and
send it again.

We do not run a bug bounty and cannot offer payment. We will credit you by name in the
CHANGELOG unless you would rather we did not.

## What this repository is

`Paridyum/knowledge` contains the governance rules for the Paridyum platform: markdown
artifacts, JSON schemas, YAML registries, shell gates, and a Claude Code plugin. It ships no
runtime service and holds no customer data.

That makes the interesting attack surface narrower than usual, and worth naming precisely:

- **The shell gates** (`scripts/*.sh`, `templates/infra-ci/scripts/*.sh`) run in CI and in
  developer pre-commit hooks. They read repository contents. An injection through a crafted
  filename, artifact id or YAML value that reaches a shell expansion is in scope.
- **The Claude Code plugin** (`plugins/paridyum-platform/`) runs `PreToolUse` hooks that
  decide whether an agent's tool call is permitted. A way to make a guard hook approve a write
  it should deny — to a locked artifact, to generated output, or a protected git operation —
  is in scope and is the highest-severity class here.
- **The vendored snapshot mechanism.** Consuming repositories carry a copy of this content
  at `.paridyum/knowledge/`. A way to alter what a consumer receives is in scope.
- **The CI workflows.** A way to obtain write access or secrets through a pull request —
  `pull_request_target` misuse, an unpinned action, a fork job reaching a secret — is in
  scope. `ci.no_pull_request_target`, `ci.actions_pinned_to_sha` and
  `ci.secret_jobs_guarded_on_fork` exist for exactly these and may have gaps.

Out of scope: findings against GitHub itself, and anything requiring an attacker who already
has write access to this repository — that is a compromise, not a vulnerability.

## Known limitations, stated rather than discovered

This repository's own governance requires being honest about what is not enforced, so:

- **Secret scanning cannot stop a determined push.** GitHub's push protection is unavailable
  on Free private repositories (ADR-0004). `gitleaks` runs in pre-commit and CI as a
  substitute, and `git commit --no-verify` bypasses the first layer. CI reports afterwards; it
  does not block, because required status checks also need a paid plan.
- **A large share of the declared controls are not implemented.** `check-registry.yaml`
  declares the full id-space; `scripts/check-knowledge.sh` prints the implemented/declared
  split on every run, and that live figure is the only one to trust — no count is written
  down here, because every hand-maintained count in this repository has been found stale. An
  id declared and unimplemented enforces nothing. This is deliberate and tracked (ADR-0014),
  not an oversight — but do not read a control's presence in the registry as evidence it runs.
- **Commits are signed.** `commit.gpgsign=true` with `gpg.format=ssh`, and every recent commit
  verifies. This section previously read *"Commits are not signed… authorship here is asserted,
  not proven"*, which was true when written and had stopped being true without anyone noticing —
  a security policy understating its own posture, found by an audit rather than by a check.
  `git.commits_are_signed` remains declared and unimplemented, so nothing *enforces* it: signing
  is configured, not guaranteed, and a commit made with it disabled would pass silently.
- **There is no threat model.** REQ-0015 recorded why: this repository had no trust boundary to
  model. **That trigger fired on 2026-08-08** — RFC-0008 designs Server Actions as publicly
  callable, unauthenticated-by-default endpoints — so the requirement is now `accepted` and the
  model is owed. Until it exists this remains an honest gap rather than a deferred one.
- **No SBOM or build provenance.** REQ-0014, deferred until the first released artifact.

## Handling on our side

A confirmed vulnerability is fixed before it is discussed publicly. If the fix requires
changing a locked artifact, it goes through supersession like any other change — a security
finding does not license editing a locked file in place, because the record of what was true
when the code was written is itself a security property.

A secret exposed in git history is treated as compromised and rotated, regardless of how
briefly it was present or whether the commit was amended away. History is what an attacker
clones.

## Related

- [C-0004](constitution/C-0004-protected-operations.md) — protected operations and why a
  leaked credential is irreversible.
- [ADR-0004](adr/ADR-0004-github-org-on-free-plan.md) — what the plan does not enforce.
- [ADR-0017](adr/ADR-0017-scan-exceptions-are-reviewable.md) — the only legitimate way to
  suppress a scan finding, and why it expires.
- [REQ-0001](requirements/REQ-0001-runtime-secret-management.md) — runtime secret handling,
  deferred.
- [REQ-0015](requirements/REQ-0015-threat-modelling-and-risk-register.md) — threat modelling
  and the risk register.
