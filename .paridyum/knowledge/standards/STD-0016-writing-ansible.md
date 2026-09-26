---
id: STD-0016
title: A playbook cites the artifact it applies, fails closed on what it cannot do, and is reviewed as a diff
type: standard
status: accepted
created: 2026-08-23
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra]
applies_to: ["infra-*"]
tags: [ansible, configuration-management, idempotence, check-mode, reproducibility, adr-0056, rfc-0035]
breaking: false
enforced_by: []
---

# STD-0016 — Writing Ansible

ADR-0056 chose the tool. This says how it is written.

## Status

**`accepted`, and not lockable today.** `enforced_by` is empty for a structural reason rather
than an oversight: `class-registry.yaml` gives `infra` **`ci: false`**, so there is no pipeline
in which a check could run. Every rule below is verified by a command an author runs and an
operator can re-run — named per rule — which under ADR-0030 is a real bridge, just not a CI one.

Locking this requires giving `infra` a CI story. That is a `class-registry.yaml` change and its
own decision.

## Why a standard rather than taste

The first Ansible repository in an estate becomes the template for every one after it, whether
or not anybody reviewed it as a template. ADR-0009 records that exact thing already happening
once here — an estate *"predominantly Python with TypeScript frontends, none matching C-0002"* —
and RFC-0028 names the mechanism: repository one establishes conventions by accident and
repository two inherits them by copying.

`infra-substrate` is repository one. This is written now, while there is one of them.

## The rules

### 1 · A role names the artifact it applies

Every role's tasks cite the decision that requires them, in a comment at the point the reader
needs it. **A role that cannot name its governing artifact does not belong in the repository.**

This is the same rule RFC-0028 applies to `template-web`'s twelve files — *"a file that cannot
name its obligation is a file that does not belong"* — and it is what stops an infrastructure
repository accreting settings copied from a benchmark that nobody can justify later.

*Verified at review.* Judgement, so no check; stated rather than smoothed over, on ADR-0030's
principle that a rule with no bridge says so.

### 2 · Check mode is safe, and the diff is the review artifact

`--check --diff` must run to completion without erroring, and the repository's default target
must be that dry run rather than an apply.

**The reason is not convenience.** These playbooks change machines that are somebody's working
environment. What a reviewer needs is a diff, not a description of a diff; making the default
target the dry run means consent is given to an observed change rather than to a promise.

*Verified by:* `make check` completes; the default `make` target is not `apply`.

### 3 · What cannot be automated asserts and fails closed. It never simulates

Some controls cannot be applied by configuration management — encrypting a mounted root, or
deciding that a host has crossed a lifecycle stage. For those, a role **asserts the state and
fails**, and says in the failure message what the operator must do.

The forbidden alternative is a task that appears to do the thing and does not. A play that
"handles" full-disk encryption by writing a note is worse than one that refuses, because the
refusal is visible and the note is not.

*Verified by:* the defect test in rule 7.

### 4 · No hostname appears in a role

Hosts live in the inventory. RFC-0023 D4 and RFC-0025 D4 both require the machine be a
variable — *"nothing may encode 'this is the hosting fleet'"* — and the payoff is concrete:
moving to rented metal becomes an inventory edit rather than a rewrite, which is what makes
RFC-0025 stage B1 a change of address instead of a project.

*Verified by:* `grep -r` for known hostnames across `roles/` returns nothing.

### 5 · No credential is stored, and privilege prompts at run time

No vault password file, no `NOPASSWD` entry created for automation's convenience, no token in
`group_vars`. Escalation prompts when a play runs.

REQ-0001 has no home for a runtime secret and ADR-0045 risk R3 is open, so a repository that
invented a secret store would be inventing it in the place least equipped to protect it.

*Verified by:* `repo.secret_scan_clean` (C-0004) and by the absence of `ansible-vault` files.

### 6 · Lint at the `production` profile, with variables role-prefixed

`ansible-lint` must report zero failures at its **`production`** profile — the strictest — not
at `basic` and not with rules disabled.

The rule this catches most often is `var-naming[no-role-prefix]`: a role's registered variables
and facts carry the role's name as a prefix. Two roles registering `mounts` is a collision that
produces a wrong value rather than an error, and it is invisible in review.

*Verified by:* `make lint`, which must exit zero.

### 7 · Every fail-closed guard is run against a violation and observed to fail

A guard nobody has seen fail is not a guard. For each assertion written under rule 3, the author
runs the playbook with a variable set to the violating value and records that it failed.

This is `lib-ui/brand/tools/check.py`'s discipline — it plants each violation and watches it
fail — applied to infrastructure. The commands belong in the repository's README so an operator
can repeat them.

*Verified by:* the commands are in the README and exit non-zero.

### 8 · A task that can break a boot, a disk or the network says so, takes a backup, and never reboots

Bootloader edits, partition operations and firewall rules are the three that can leave a machine
unreachable or unbootable. Each must take a backup where one is possible, be called out in the
SOP as needing careful reading, and **never reboot or reformat on the playbook's own initiative.**

Network rules additionally must be applied **atomically**. An nftables ruleset is loaded whole,
so the allow rules and the drop policy land in one transaction; building the same policy from a
sequence of `nft add` commands creates a window in which the policy is deny and the tunnel is not
yet permitted, and losing that window means losing the only route to the host.

*Verified at review*, and by the SOP naming the dangerous hunks.

### 9 · Collections are pinned to a major range

ADR-0053 — a pin is exact and current. A collection release must not change behaviour under a
machine that is an operator's daily environment.

*Verified by:* `requirements.yml` carries a bounded version for every collection.

### 10 · A deliberate omission is written down

Where a common hardening item is **not** applied, the role says so and why, at the place a reader
would look for it.

`kernel.unprivileged_userns_clone` is the worked example: most benchmarks disable it, and doing
so would break rootless containers and the browser's own sandbox on a machine somebody uses
daily. Recorded as a decision, it is reviewable. Absent, it looks like an oversight and the next
author "fixes" it.

*Verified at review.*

## What this standard does not cover

**Provisioning.** ADR-0056 draws the line: OpenTofu answers *what machines exist*, Ansible
answers *what is true on this machine*. Nothing in a role may create a machine.

**Testing against a throwaway host.** Molecule is the obvious answer and it is not required here,
because `infra` is `ci: false` and there is nowhere to run it. When that changes, this section is
where the requirement lands.

## Related

- **ADR-0056** — the tool, the licence test, and the OpenTofu boundary
- **RFC-0035** — the first thing written to this standard
- **RFC-0025 D7** — "reconcile, do not command", which rule 2 implements
- **RFC-0023 D4**, **RFC-0025 D4** — the host is a variable, which rule 4 implements
- **ADR-0053** — a pin is exact and current (rule 9)
- **ADR-0030** — why an unenforced rule says so rather than implying a check
- **ADR-0009**, **RFC-0028** — why repository one is written to a standard rather than to taste
- **C-0004**, **REQ-0001**, **ADR-0045** — why rule 5 stores nothing
