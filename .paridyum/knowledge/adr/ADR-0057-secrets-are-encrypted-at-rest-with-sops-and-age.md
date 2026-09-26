---
id: ADR-0057
title: A secret is encrypted at rest with SOPS and age, and the key that opens it never enters a repository
type: adr
status: accepted
created: 2026-08-23
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform, service, web]
applies_to: ["*"]
tags: [secrets, sops, age, ansible-vault, encryption-at-rest, req-0001, adr-0045, adr-0056]
breaking: false
enforced_by: []
sources:
  - url: https://github.com/getsops/sops
    title: SOPS — repository licence metadata
    retrieved: 2026-08-23
    supports: >
      SOPS is MPL-2.0, an OSI-approved licence and the same one ADR-0052 chose OpenTofu under.
      This is the fact REQ-0012's exitability test turns on.
  - url: https://github.com/FiloSottile/age
    title: age — repository licence metadata
    retrieved: 2026-08-23
    supports: >
      age is BSD-3-Clause, OSI-approved. Recorded because the key format is the part hardest to
      migrate away from, so its licence matters more than the tool's.
  - url: https://docs.ansible.com/ansible/latest/collections/community/sops/docsite/guide.html
    title: Protecting Ansible secrets with SOPS — Ansible Community Documentation
    retrieved: 2026-08-23
    supports: >
      Ansible has first-party support for SOPS through community.sops, including a vars plugin
      that decrypts group_vars transparently. So choosing SOPS over ansible-vault costs no
      Ansible ergonomics — which is the objection this decision would otherwise have to answer.
  - url: https://unixy.io/blog/secrets-management-2026/
    title: Secrets Management in 2026 — Vault vs Secrets Manager vs SOPS
    retrieved: 2026-08-23
    supports: >
      SOPS encrypts VALUES and leaves keys visible, so a diff remains readable; ansible-vault
      encrypts whole files, making diffs opaque. It also records that SOPS is not tied to one
      automation platform, working across Ansible, Terraform/OpenTofu, Kubernetes and shell.
      Both facts are load-bearing for the decision below.
---

# ADR-0057 — Secrets are encrypted at rest with SOPS and age

## Status

**ACCEPTED** as of 2026-08-23.

`enforced_by` is empty and that is honest. The check worth having — *no plaintext secret is
committed* — already exists as `repo.secret_scan_clean` and `repo.no_credential_files_committed`
under C-0004, and those catch the failure this decision exists to prevent. A check that a secret
was encrypted *with SOPS specifically* would be enforcing a mechanism rather than a property.

## Context

Ansible needs a privileged password to apply RFC-0035, and the estate has nowhere to put one.
REQ-0001 is `deferred`, ADR-0045 risk R3 is open and says so, and the only credential store in
existence today is `paridyum/.secrets` — a plaintext file, outside git by accident of directory
layout rather than by design.

The question is not "which encryption" but **which mechanism the estate adopts once**, because
the second one is what makes secrets unfindable.

## Decision

**A secret is encrypted at rest with SOPS, using age keys. The age private key lives in the
operator's home directory and never enters a repository.**

Consequences that are part of the decision rather than downstream of it:

- **Encrypted files are committed.** That is the point of encrypting them. An encrypted file in
  git is versioned, reviewable and restorable; a plaintext file outside git is none of those and
  survives only as long as the disk does.
- **`.sops.yaml` declares which paths are encrypted and to which public keys.** Adding a
  recipient is a config edit and a re-encrypt, not a re-architecture.
- **The private key is backed up outside the estate.** Losing it loses every secret it opens.
  This is the single point of failure and it is named here rather than discovered.
- **No secret is decrypted into a file on disk.** Ansible reads them through `community.sops`,
  which decrypts in memory.

## Why not ansible-vault

It is the obvious choice, it ships with the tool ADR-0056 already picked, and it needs no new
dependency. It loses on two counts, both of which are about this estate specifically.

**1 · It is Ansible-only, and the estate is not.** ADR-0052 chose OpenTofu and ADR-0020 makes
Kubernetes the deployment target; both will need secrets. `ansible-vault` cannot serve them, so
choosing it means a second mechanism arrives the first time OpenTofu needs a credential — and
then a third. SOPS works across Ansible, OpenTofu, Kubernetes and plain shell. **One fact, one
home** (RFC-0015), applied to mechanism rather than to data.

**2 · It encrypts whole files, so a diff is opaque.** This estate's entire review model is
reading the diff — STD-0016 rule 2 makes `--check --diff` the default target precisely so that
consent is given to an observed change. A secrets file whose every change renders as one
unreadable blob is the one place that model silently stops working. SOPS encrypts values and
leaves keys visible, so a reviewer can see *that* `ansible_become_password` changed without
seeing what it became.

The counter-argument is real and is not dismissed: `ansible-vault` is one fewer tool, one fewer
licence to check, and one fewer binary that must exist wherever a playbook runs. On an estate
that only ever ran Ansible it would be the right answer.

## Why not a hosted secret manager

HashiCorp Vault, AWS Secrets Manager and the rest are the enterprise answer and are correct at
a scale this estate does not have. Rejected on three counts: Vault's licence is BUSL and fails
REQ-0012's exitability test exactly as Terraform and Omni did; a hosted manager is a running
service that must exist before the first secret can be read, which inverts the bootstrap order
this estate needs; and C-0006 names this category as commodity infrastructure we depend on
rather than position on.

Revisit when there is more than one person holding keys, which is the point at which key
distribution stops being a filesystem problem.

## What this does not decide

**Runtime secrets for deployed workloads.** REQ-0001 owns those, and a Kubernetes workload
reading a secret at runtime is a different problem from a playbook reading one at apply time.
SOPS is a plausible half of that answer through `sops-secrets-operator`, and this decision does
not assume it.

**CI secrets.** GitHub Actions secrets remain what CI uses. The token `web-paridyum` carries is
recorded there and in its own workflow, and moving it here would mean CI decrypting an age key
it would then have to hold — trading one long-lived credential for one plus a mechanism.

## Consequences

**Accepted costs**

1. **A second tool and a new binary.** `sops` must exist wherever a secret is read. It is a
   single static binary and is in Homebrew and every distribution, which is the mitigation.
2. **The age private key is a single point of failure.** Lose it and every secret it opens is
   gone. Backup is an operational obligation stated in the SOP, and it is the reason to prefer
   losing a secret over leaking one — a lost secret is re-created, a leaked one is not un-leaked.
3. **A committed encrypted file is a permanent artifact.** If the key is ever compromised, every
   historical version in git history is readable. Rotation therefore means rotating the SECRET,
   not just the key.
4. **Nothing checks that this decision is followed.** C-0004's scanners catch a plaintext
   credential, which is the failure that matters; they do not care whether the encrypted one
   beside it used SOPS.

**Gained**

- The estate has one secret mechanism before it has two, which is the whole reason to decide now.
- Secrets become versioned and restorable rather than surviving only on one disk.
- A secrets change stays reviewable as a diff, which is the property STD-0016 rule 2 depends on.

## Related

- **ADR-0056** — Ansible, the first consumer; **ADR-0052** — OpenTofu, the second, and the
  licence test this reapplies
- **REQ-0001** — runtime secret management, still deferred and not resolved by this
- **ADR-0045** — risk R3, the open acceptance this narrows but does not close
- **C-0004** — `repo.secret_scan_clean` and `repo.no_credential_files_committed`, which are the
  checks that actually enforce the property
- **RFC-0015** — a fact has one home, applied here to mechanism
- **STD-0016** rule 2, **RFC-0025 D7** — why an opaque diff is a real cost
- **REQ-0012** — exitability; **C-0006** — why a hosted manager is commodity
