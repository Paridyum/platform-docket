---
id: ADR-0056
title: Configuration management is Ansible, and a host is described rather than commanded
type: adr
status: locked
created: 2026-08-23
locked_at: 2026-10-04
supersedes: null
superseded_by: null
scope: [org, infra]
applies_to: ["infra-*"]
tags: [ansible, configuration-management, opentofu, licence, req-0012, rfc-0035, reconcile]
breaking: false
enforced_by:
  - check: repo.hosts_are_configured_only_by_ansible
    severity: error
    rationale: >
      The shape this decision was broken by, twice (infra-terraform's runner scripts, 2026-09-28 to
      2026-10-03): a script fed to a remote shell over SSH from a repository. It rejects that in every
      repository's CI. It cannot see a person typing on a host — that remains the SOPs' and reviews' job.
sources:
  - url: https://pypi.org/pypi/ansible-core/json
    title: ansible-core — package metadata
    retrieved: 2026-08-23
    supports: >
      ansible-core declares its licence as GPL-3.0-or-later, an OSI-approved licence, at
      version 2.21.3. This is the fact REQ-0012's exitability test turns on, and it is the
      test ADR-0052 applied to Terraform and RFC-0025 applied to Omni.
---

# ADR-0056 — Configuration management is Ansible

## Status

**LOCKED** as of 2026-10-04. **ACCEPTED** as of 2026-08-23.

**Locked 2026-10-04,** with `repo.hosts_are_configured_only_by_ansible` in `enforced_by`. Unlocked, it
was broken twice without any gate noticing: infra-terraform's runner was configured by scripts piped
over SSH for five days, each change citing a decision that quoted only this ADR's OpenTofu half (the
investigation is `docs/2026-10-03-why-adr-0056-was-not-followed.md`). The check rejects that shape —
`ssh … bash -s`, `ssh … < x.sh`, `cat x.sh | ssh …` — in any repository, and was watched failing on the
real recipes before this lock. What it cannot do is the gap the paragraph below names: it reads
repositories, not hosts, so a command typed on a machine by hand is outside it.

The paragraph below is what stood here while this was accepted; it is kept because it was true then.

`enforced_by` is empty and that is honest rather than an omission. Locking this needs a check
that fails when a host is configured by something other than a playbook, and no such check
exists — nor can `tool-forge` inspect a host it does not run on. Under ADR-0014 `accepted` with
no check is a legitimate state; under ADR-0002 `locked` with no check is not.

## Context

RFC-0035 specifies what a substrate host must look like. Nothing applies it.

The default alternative is a runbook of shell commands, and its failure mode is known well
enough that this repository already rejected it one layer up. RFC-0025 decision 7 is
**"reconcile, do not command"**, and the argument holds identically here: a command changes a
machine once, at a moment nobody recorded, in a state nobody can reproduce. A host configured
that way cannot be rebuilt, and RFC-0023 accepted cost 3 requires precisely that it be
disposable — *"which is a discipline, not a property, until something enforces it."*

**The second reason is the review problem.** RFC-0035 changes a machine that is an operator's
daily working environment. What a reviewer needs to see is a **diff**, not a description of a
diff. A declarative tool with a dry-run mode makes the review artifact the same object as the
change; a runbook makes the review an act of trust.

## The licence test is applied, because skipping it is the drift this estate names

ADR-0052 chose OpenTofu over Terraform on licence grounds alone, and stated the general rule in
terms this decision is bound by:

> *Applying REQ-0012's exitability test to Omni and then not applying it to Terraform is
> precisely the drift `CLAUDE.md` names — never infer architecture, retrieve it — one artifact
> deciding by principle and the next deciding by which tutorial was open.*

So the test is applied again rather than assumed. **`ansible-core` is GPL-3.0-or-later**, an
OSI-approved licence, verified from the package's own metadata on 2026-08-23. It passes where
Terraform (BUSL 1.1) and Sidero Omni (BUSL) both failed.

That the answer is comfortable is not a reason to skip the step. The point of the test is that
it is run every time, including the times it passes.

## Decision

**Host configuration is Ansible. Infrastructure provisioning remains OpenTofu (ADR-0052).**

The boundary between them is stated so that neither grows into the other:

| Tool | Owns | The question it answers |
| --- | --- | --- |
| **OpenTofu** | Infrastructure that does not exist yet — VMs, networks, disks, via a provider API | *What machines exist?* |
| **Ansible** | The inside of a machine that already exists — packages, users, kernel parameters, services, firewall | *What is true on this machine?* |

**Nothing in an Ansible role may create a machine, and nothing in an OpenTofu module may
configure the inside of one.** Both tools can technically do the other's job, which is exactly
why the line is drawn here rather than discovered per repository.

**A host is a variable, not a name.** RFC-0023 D4 and RFC-0025 D4 both require that nothing
encode which host is which. In practice: hostnames live in the inventory, never in a role.
Moving to rented metal is then an inventory edit rather than a rewrite — which is the property
that makes RFC-0025's stage B1 a change of address rather than a project.

**No privileged credential is stored.** Privilege escalation prompts at run time. REQ-0001 has
no home for a secret and ADR-0045 risk R3 is open, so a vault password or a NOPASSWD entry
would be inventing a secret store in the repository least equipped to protect one.

## Alternatives considered

**NixOS, replacing the host OS.** The strongest alternative by a distance, and the only one
that makes RFC-0023's "disposable" a genuine property rather than a discipline: the machine is
a build artifact, rebuilt identically from a single expression, with rollback at the bootloader.

Rejected for this host and **not for the class**. RFC-0023 licensed Arch for `sitar-as-in-01-ar`
specifically because *"the machine is an operator's working environment first"*, and replacing
the OS costs exactly what that decision declined to pay. The moment a substrate host stops being
somebody's workstation — which RFC-0025 stage B1 does — this alternative becomes a live
question again, and it should be reopened then rather than treated as settled here.

**Shell scripts plus systemd units.** Zero dependencies, and every host in this estate could run
them today. Rejected because a script commands rather than reconciles: it has no dry run, no
idempotence guarantee, and no diff, so the review artifact would be prose about what the script
does. That is the state RFC-0025 D7 already rejected.

**Salt, Puppet, Chef.** All viable and all pass the licence test. Rejected on agent footprint
and on ecosystem: each wants a daemon or a client on the managed host, which is one more
privileged process on a machine whose whole problem is that it already runs too many. Ansible is
agentless and reaches the host over the transport that already exists.

**OpenTofu alone, with `cloud-init` and remote-exec.** Tempting because it removes a tool.
Rejected because `remote-exec` is a shell script wearing a provider's clothes — it inherits
every property rejected two paragraphs above — and because cloud-init runs once at first boot,
so it cannot converge a host that has drifted.

## Consequences

**Accepted costs**

1. **A second tool in the infrastructure story**, with its own vocabulary and its own upgrade
   surface. The boundary table above is the mitigation; it will be tested the first time
   somebody wants a role to create a VM.
2. **Nothing checks that this is followed.** `enforced_by` is empty and no check can inspect a
   host from CI. A machine configured by hand would look identical to one configured by a
   playbook until the next run produced a large diff — which is, weakly, the detection.
3. **The `infra` class permits `stacks: [terraform, none]`.** Ansible is neither, so
   `infra-substrate` declares `none` — the value `infra-ci` already uses for workflow YAML and
   shell. Noted as a real if minor mismatch: that list still names `terraform`, a tool ADR-0052
   decided against, so the stack vocabulary is stale in a way worth an erratum.
4. **`infra` is `ci: false`.** These playbooks are linted and syntax-checked by hand. The class
   registry already records this gap; this decision inherits it rather than fixing it.

**Gained**

- RFC-0035 acquires an executable form, so its rollout table is a playbook rather than a list.
- The review artifact becomes a diff. `--check --diff` is what an operator reads before consenting.
- A substrate host becomes reproducible, which is the precondition for RFC-0023's disposability
  claim being true rather than intended.

## Related

- **ADR-0052** — OpenTofu as the provisioning tool, the licence test this reapplies, and the
  boundary this decision draws against
- **RFC-0035** — the controls this exists to apply
- **RFC-0025** — D7 "reconcile, do not command", and D2's "no path in that is not the API"
- **RFC-0023** — the substrate, its disposability claim, and why NixOS is deferred rather than rejected
- **REQ-0012** — the exitability test; **REQ-0001**, **ADR-0045** — why no credential is stored
- **ADR-0014**, **ADR-0002** — why this is `accepted` rather than `locked`
