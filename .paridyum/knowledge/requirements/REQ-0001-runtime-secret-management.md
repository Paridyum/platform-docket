---
id: REQ-0001
title: Runtime secrets are managed outside the repository and outside GitHub
type: requirement
status: deferred
created: 2026-08-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, service, infra]
applies_to: ["*"]
tags: [secrets, runtime, infrastructure, kubernetes, github-free]
breaking: false
trigger: >
  The first service needs a credential at runtime — a database URL, an API key, or a
  signing key — in any environment including dev.
review_date: 2027-02-01
enforced_by: []
---

# REQ-0001 — Runtime secrets are managed outside the repository and outside GitHub

## Status

**DEFERRED** as of 2026-08-07. Real, understood, deliberately not scheduled.

Not deferred because it is unimportant. Deferred because there is nothing to protect
yet: zero services, zero deployments, zero runtime credentials. Building secret
infrastructure now would be provisioning for a load that does not exist.

`enforced_by` is empty and that is correct — ADR-0002 reserves `locked` for what can be
mechanically rejected, and there is nothing to check until the trigger fires. This is
ADR-0008's honest-inventory position applied to a requirement.

## Requirement

**The platform shall store and deliver runtime secrets from a system outside both the
git repository and GitHub's own secret storage, such that a repository can obtain the
credentials it needs without any secret being committed and without per-repository
manual configuration.**

Deliberately technology-neutral. Infisical is the current best candidate and the
evidence for it is recorded below, but the requirement is satisfied by anything meeting
the acceptance criteria. Naming the tool here would make this a design document.

## Acceptance criteria

1. No credential is committed to any repository, in history or working tree — already
   enforced by `repo.secret_scan_clean` and `repo.no_credential_files_committed`.
2. A new repository obtains its runtime secrets **without** an operator manually adding
   N secrets to that repository's settings.
3. A credential can be rotated without editing any repository.
4. The mechanism works for a **private** repository on the **GitHub Free** plan, or its
   adoption is explicitly paired with the plan upgrade ADR-0004 tracks.
5. Local development uses the same source of truth as CI and production, so the three
   cannot drift — the property that took four attempts to achieve for the secret scan.

## Out of scope

- Secret **scanning**, which is C-0004, ADR-0017 and already implemented. This
  requirement is about delivering secrets, not detecting leaked ones.
- The `GITLEAKS_LICENSE` key. Deliberately stored nowhere in GitHub; the MIT gitleaks
  binary needs no licence (ADR-0017). It is not a runtime secret.
- Certificate and PKI management, which is a separate concern even though some
  candidate tools cover it.

## Background — why this is not obvious

The constraint that makes this a real requirement rather than a preference was
established by testing against the live organisation on 2026-08-07:

**Organisation-level GitHub Actions secrets do not work for private repositories on the
Free plan.** They exist, but only public repositories can read them; private
repositories need Team or Enterprise. Paridyum is Free with private repositories
(ADR-0004, ADR-0012).

The consequence is the thing worth recording: with GitHub alone, **every repository's
secrets must be set by hand, in that repository, forever.** For a polyrepo platform
(ADR-0001) that is per-repository drift of exactly the kind vendored knowledge and the
plugin harness (ADR-0007) exist to eliminate — and it is invisible, because a repository
with a missing or stale secret fails at deploy time rather than at gate time.

This was discovered while deciding where to put a gitleaks licence key, and it would
otherwise have been rediscovered at the first service deploy.

## Evidence gathered so far

Recorded because the research is done and should not be repeated, not because the choice
is made.

**Infisical** — MIT-licensed core, enterprise features under a separate licence in
`backend/src/ee/`. Self-hostable. Free tier at $0 for up to five identities, including
the CLI, SDKs, Kubernetes operator, agent and all integrations. Relevant pieces:

- The **Kubernetes operator** is the piece that matters for ADR-0003, syncing secrets
  into the cluster rather than into a workflow.
- `Infisical/secrets-action` injects secrets into GitHub Actions and supports universal
  auth, AWS IAM and **OIDC**.
- With universal auth, criterion 2 is only partly met: a client id and client secret
  must still be stored per repository — two secrets instead of many, which is better but
  not zero.
- **With OIDC there is no long-lived credential in GitHub at all**, which satisfies
  criterion 2 fully. This is the pattern to aim for.

## Open questions

1. **Is OIDC machine-identity auth available on the free tier?** The tier list says "all
   integrations" but does not enumerate auth methods, and the enterprise list does not
   mention OIDC. This must be confirmed before OIDC is assumed, because criterion 2
   depends on it.
2. Self-host or Infisical Cloud? Self-hosting adds an operational dependency that must
   itself be deployed and backed up — which for a platform with no cluster yet is a
   chicken-and-egg problem worth thinking about before the trigger fires.
3. Does the plan upgrade ADR-0004 tracks change the calculus? Team-plan org secrets
   would satisfy criteria 2 and 3 for CI without a new system, though not for the
   cluster.
4. Alternatives not yet evaluated: HashiCorp Vault, Doppler, SOPS with age, and the
   cloud-native KMS options. Infisical was researched first because it came up first,
   which is not a reason to choose it.

## What this does not cover

**A read-only CI credential is out of scope.** Decided 2026-08-21 while accepting
RFC-0034, which needs one fine-grained read-only token to collect test evidence
across the organisation.

The boundary is what the secret gives access to, not where it is stored:

- **In scope:** anything a *running service* needs to function — database
  credentials, the Zitadel masterkey, service-account keys, signing keys. These are
  the ones whose absence is discovered at deploy time, which is the failure this
  requirement is named after, and whose leak grants powers the holder did not have.
- **Out of scope:** a credential that reads only what the person holding it can
  already read, used by CI rather than by a service. A read-only token over
  repositories the operator already owns adds no capability; it automates an
  existing one.

This is written here rather than left in RFC-0034 so the boundary is not re-argued
from a design document every time it comes up. It does not make such a token free:
it is still one secret set by hand in one repository, which is exactly the drift
this requirement describes — the argument is only that the drift is bounded at one,
and that read-only is a materially smaller blast radius than write.

If a CI credential ever needs write access to anything outside its own repository,
it is in scope and this section does not apply to it.

## Related

- ADR-0004 — GitHub Free plan limitations, and the upgrade trigger
- ADR-0003 — Kubernetes from day one; the operator is the integration point
- ADR-0010 — CI/CD architecture; the reusable workflow is where injection lands
- ADR-0012 — private repositories and contributor access
- ADR-0017 — secret **scanning**, the adjacent and already-solved problem
- C-0004 — credentials are never committed
