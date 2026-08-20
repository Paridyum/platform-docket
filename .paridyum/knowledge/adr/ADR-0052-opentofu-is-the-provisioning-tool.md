---
id: ADR-0052
title: OpenTofu is the provisioning tool, because the licence test that rejected Omni applies here too
type: adr
status: accepted
created: 2026-08-19
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform, service]
applies_to: ["*"]
tags: [terraform, opentofu, provisioning, licence, exitability, req-0012, adr-0020, rfc-0019, rfc-0023]
breaking: false
enforced_by: []
---

# ADR-0052 — OpenTofu is the provisioning tool

## Status

**ACCEPTED** as of 2026-08-19.

`enforced_by` is empty and that is honest rather than an omission. Locking this requires a check
that fails when a repository's provisioning is driven by the BUSL binary, and no such check exists.
Under ADR-0014 `accepted` with no check is a legitimate state; under ADR-0002 `locked` with no check
is not. See *Owed* below.

## Context

**The tool was never chosen, and the artifacts that touch it assume different answers.**

RFC-0019 left it explicitly open — *"Terraform or OpenTofu against the Proxmox API, with the cluster
defined as code and no hand-built node."* `TASKS.md` T-2b.4 then reads *"Provision the Proxmox guest
with Terraform"* and names the `bpg/proxmox` provider. So the roadmap task assumes a tool its own
governing RFC declined to pick.

**ADR-0020 is locked and says `terraform` four times.** It requires a `terraform/` module skeleton
per deployable repository, declares the check `terraform.fmt_and_validate`, and names
`infra-terraform` as a cluster-level repository. None of that is a decision *about which binary*,
but all of it reads like one to somebody scaffolding a repository.

**The pressure the other way is real and worth stating.** Terraform is the better-documented tool,
every tutorial for Proxmox and Talos assumes it, and `terraform` is already the word written into a
locked artifact. Choosing against the vocabulary in a locked ADR is friction paid on every future
reading.

## The decisive argument is one this repository has already made

RFC-0025 rejected Sidero Omni — the obvious adopt-path for the fleet manager, and by its own
assessment the better product — **on licence grounds alone**: Omni is BUSL, requires a commercial
licence for production, and *"fail[s] REQ-0012's exitability test outright."*

Terraform is in the same category. HashiCorp relicensed it from MPL-2.0 to the Business Source
Licence 1.1 with v1.6 (announced August 2023); BUSL is not OSI-approved and restricts competitive
use. OpenTofu is the MPL-2.0 fork under Linux Foundation multi-vendor governance. As of July 2026
the practical differences are the licence, who sets the roadmap, and a set of CLI features OpenTofu
shipped that Terraform's open binary lacks.¹

Applying REQ-0012's exitability test to Omni and then not applying it to Terraform is precisely the
drift `CLAUDE.md` names — *never infer architecture, retrieve it* — one artifact deciding by
principle and the next deciding by which tutorial was open. **The test does not become optional
because the tool is popular.**

## Decision

**1. OpenTofu is the provisioning tool for every repository and every `infra-` stack.** Terraform's
BUSL releases are not adopted. This resolves RFC-0019's open choice and corrects T-2b.4's
assumption.

**2. ADR-0020 is satisfied without supersession, and this is the part to read before scaffolding.**
The locked artifact requires a directory named `terraform/` and a check named
`terraform.fmt_and_validate`. Both survive: the directory name is tool-neutral, and `tofu fmt` and
`tofu validate` are the same two operations the check names. **The check's identifier is
vocabulary, not a product requirement**, and ADR-0020 nowhere states which binary runs it. Nothing
here contradicts a locked artifact, so no supersession is required and none is taken.

**3. State is encrypted client-side by OpenTofu, which answers RFC-0023 open question 2.** OpenTofu
1.7 shipped native encryption of state and plan files through a key provider — passphrase, Vault, or
a cloud KMS — encrypting before the file reaches any backend. Terraform's open CLI has no
equivalent.¹ RFC-0023 open question 2 asked where state lives and how it is encrypted, given that
state contains the secrets decision 6 exists to protect; the passphrase provider answers it with the
SOPS/age mechanism decision 6 already commits to building, and adds no vendor.

**4. The provider is `bpg/proxmox`, which is published to both registries.** v0.111.1, 2026-07-03,
targeting Proxmox VE 9.x, and maintained — as against `telmate/proxmox`, which most tutorials use
and which is less actively maintained.²

## Consequences

**Accepted costs**

1. **The vocabulary and the tool now differ.** A locked ADR says `terraform/`, a locked check id says
   `terraform.`, and the binary is `tofu`. Decision 2 exists to stop that reading as a
   contradiction, and it is still a paper cut on every first encounter. The alternative — superseding
   ADR-0020 to rename a directory — costs more than the confusion does.
2. **State encryption is an OpenTofu-only feature, so using it deepens the commitment to OpenTofu.**
   Exiting to any other tool means decrypting state first. This is a smaller lock-in than a
   proprietary licence and it is a lock-in, and REQ-0012 asks for the exit plan while the
   integration is small: the exit is `tofu` with encryption disabled, producing ordinary plaintext
   state that any HCL-compatible tool reads.
3. **The HashiCorp commercial ecosystem is out of reach** — Terraform Cloud and anything keyed to
   the BUSL binary. Nothing currently depends on it, which is why this is cheap now and would not
   have been later.
4. **Some Proxmox and Talos material assumes `terraform` verbatim.** Command names differ by one
   word and HCL is shared, so the cost is transliteration rather than translation.

**Gained**

- REQ-0012's exitability test is applied consistently across the two vendor decisions this
  organisation has actually taken, rather than selectively.
- RFC-0023 open question 2 closes without adding a vendor or a secrets story.
- RFC-0019's open choice closes, and T-2b.4 stops resting on an assumption nothing decided.

**Rejected alternatives**

- **Terraform.** Better documented and more widely assumed. Rejected on the licence test that
  already rejected Omni. Adopting it would also require arguing, in writing, why the same test
  produces two answers — and no such argument exists.
- **Cluster API providers for Proxmox.** RFC-0019 already assessed these as *"the more ambitious
  path… worth revisiting once there is more than one cluster to manage, and are premature at one."*
  Unchanged; this decision does not touch it.
- **Pulumi, or a general-purpose language.** A different model, not a different licence, and it
  discards the HCL that ADR-0020's `terraform/` skeleton and every Proxmox example are written in.
  No exitability problem, and no reason to pay the switch.
- **Deciding per repository.** Two provisioning tools in one estate is the drift ADR-0021 rejected
  for CI pipelines, one layer down.

## Owed

**A check, before this can ever be `locked`.** The shape is a repository-level assertion that
provisioning is invoked as `tofu` and that no BUSL Terraform binary is pinned in a workflow or a
toolchain file. Note T-2.7: a check owed by a component that is not `tool-forge` cannot currently be
declared in `check-registry.yaml` without being counted as implemented, so declaring this honestly
is blocked behind that fix.

## Sources

1. OpenTofu versus Terraform as of July 2026 — licence (MPL-2.0 under Linux Foundation governance
   versus BUSL 1.1, not OSI-approved), governance, and the OpenTofu-only CLI features including
   state encryption shipped in v1.7 with passphrase, Vault and KMS key providers. Retrieved
   2026-08-19.
2. `bpg/proxmox` provider, v0.111.1, published 2026-07-03, Proxmox VE 9.x, published to both the
   Terraform and OpenTofu registries. Retrieved 2026-08-19.

## Related

- REQ-0012 — item 5, the exit plan, and the test this decision applies
- RFC-0025 — the Omni rejection this decision is consistent with
- RFC-0019 — the open choice this closes, and the Cluster API path it leaves open
- RFC-0023 — decision 6 and open question 2, which decision 3 answers
- ADR-0020 — the locked `terraform/` obligations decision 2 reconciles
- ADR-0005 — the locked stack, and the precedent for deciding a toolchain by argument
- ADR-0014 — declared versus implemented, which is why `enforced_by` is empty
- T-2b.4 · T-2.7 — the task this corrects, and the one blocking its check
