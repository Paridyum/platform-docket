---
id: REQ-0018
title: Running software is tested from outside, by something that was not told how it works
type: requirement
status: deferred
created: 2026-08-08
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, web]
applies_to: ["svc-*", "platform-*", "web-*"]
tags: [security, dast, penetration-testing, asvs, vulnerability-management, testing]
breaking: false
trigger: >
  The first HTTP surface is reachable from anything other than a developer machine — a
  deployed environment, a preview URL, or a webhook endpoint an external service can call.
review_date: 2027-02-01
verified_by:
  - TST-0025
  - TST-0026
enforced_by: []
---

# REQ-0018 — Running software is tested from outside

## Status

**DEFERRED** as of 2026-08-08. Nothing is deployed and nothing is reachable. This repository
contains markdown, shell and YAML, and accepts input from one person on one machine.

## Requirement

**Software that is reachable shall be tested against its running form by something that does not
have the source — automatically and continuously against every deployed environment, and
periodically by a person whose job is to defeat it.**

## What already exists

The inward-facing half is real and running. `clippy --all-targets --all-features -D warnings`
and `cargo-deny check bans licenses sources advisories` run on every change in the shared
workflow; `gitleaks` scans full history in CI and in the pre-commit hook. ADR-0005's error and
panic rules make several lint categories security controls rather than style ones.

REQ-0015 owns threat modelling — what an attacker would try, decided at design time — and names
multi-tenancy as the standing threat. It also registers OWASP ASVS in `framework-registry.yaml`
with `mapping_deferred_to: REQ-0015`, for adoption as the requirement catalogue for the first
service.

REQ-0014 owns provenance of what ships. STD-0009 owns tenant isolation from inside the test
suite.

The gap is that **every one of those reads source or metadata.** None of them looks at what the
service actually serves, with its real configuration, in the environment it was deployed to.

## Numbered items

1. **The scanner runs against a deployed build, never a developer machine.** A dynamic scan
   against localhost reports the developer's machine. Its entire value is the difference between
   the source and the running form — a header absent at runtime, a debug route left routable, an
   error page returning a stack trace, a CORS policy that is permissive only in the environment
   nobody checked.

2. **The scan is authenticated.** An unauthenticated scan reaches the login page. Everything
   worth finding is behind it, which means the scanner needs a credential, which means a
   REQ-0001 dependency — and it means the scanner must be pointed at a tenant that exists for
   this purpose and is never a real one.

3. **Findings are triaged with a severity scale that already exists, and the noise is
   acknowledged.** Dynamic scanners produce false positives at a rate that will exhaust a
   one-person team within two runs. A suppression is an ADR-0017 exception — reviewable, with an
   owner and an expiry — never an inline comment, which `repo.no_inline_scan_suppressions`
   already forbids.

4. **A person tests it too, and not on a schedule.** TST-0026 is `on-demand` deliberately: a
   penetration test announced a quarter ahead becomes a rehearsal of the announcement. What a
   person finds and a scanner does not is the chain — a sequence of individually-permitted
   actions that crosses a tenant boundary — which is precisely the defect class this platform
   cannot afford (ADR-0024 cost 2).

5. **Every finding becomes a regression test.** STD-0008's rule for defects, applied to security
   findings. Without it an engagement report is a document that expires, and the next engagement
   finds the same class of issue.

6. **Production is in scope, carefully, and the constraint is on the test rather than the
   policy.** TST-0056 production smoke must be safe to run against production: read-only where
   possible, and where a write is unavoidable, scoped to a tenant that exists for this purpose.
   A "never touch production" rule produces a system nobody has ever verified in the only
   environment that matters.

7. **ASVS is the catalogue, and the level is stated.** REQ-0015 §6 already commits to adopting
   it. This requirement is where the verification half lands: an ASVS level is a list of things
   to test, and the mapping in `framework-registry.yaml` becomes writable at the same moment.

8. **Vulnerability response is the other half and is currently absent.** The ADR-0028 mapping
   found **nothing at all** under SSDF `RV.*`. Finding a vulnerability is worth little without a
   stated path from finding to fix to verification, and `SECURITY.md` already names the
   disclosure route without naming what happens after.

## Why this is not startable today

Items 1, 2 and 6 need a deployment. Item 4 needs something worth attacking and a budget. Items
3, 5 and 8 need findings.

Item 7 is decidable now — an ASVS level could be chosen today — and is deferred with the rest
for the reason `framework-registry.yaml` already records: ASVS governs running applications and
there are none, so a level chosen now would be chosen from a document rather than from a system.

## Framework position

Related: NIST SSDF `PW.8.1` (determine whether executable code testing should be performed and
which types) and `PW.8.2` (scope, design, perform and document the testing), `RV.1.1` (gather
vulnerability information from consumers and public sources), `RV.1.2` (review, analyse and/or
test code to identify or confirm previously undetected vulnerabilities), `RV.2.1` and `RV.2.2`
(analyse and remediate), `RV.3.3` and `RV.3.4` (eradicate the class, then fix the process);
CIS v8.1 `7.6` (automated vulnerability scans of externally-exposed assets), `7.7` (remediate
detected vulnerabilities), `16.2` (a process to accept and address software vulnerabilities),
`16.6` (a severity rating system), `16.13` (conduct application penetration testing); NIST CSF
2.0 `ID.IM-02` (improvements identified from security tests and exercises) and `RS.MA`.

This requirement is where the `RV.*` gap closes. It was identified by the ADR-0028 mapping and
recorded in the changelog at the time: this repository governs how code is written far better
than what happens after a vulnerability is found.

## Related

- [REQ-0015](REQ-0015-threat-modelling-and-risk-register.md) — the design half; that decides what to look for, this looks
- [REQ-0014](REQ-0014-supply-chain-integrity-and-provenance.md) — what shipped; this is what it does once running
- [REQ-0001](REQ-0001-runtime-secret-management.md) — item 2's dependency: an authenticated scan needs a credential
- [STD-0009](../standards/STD-0009-tenant-isolation-testing.md) — the same boundary, tested from inside the suite
- [STD-0008](../standards/STD-0008-test-tiers-and-gates.md) — tiers, and the finding-to-regression rule
- [ADR-0017](../adr/ADR-0017-scan-exceptions-are-reviewable.md) — the suppression mechanic item 3 reuses
- [ADR-0028](../adr/ADR-0028-controls-map-outward-to-frameworks.md) — the mapping that found the `RV.*` gap
- `SECURITY.md` — the disclosure path item 8 continues
