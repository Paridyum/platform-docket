# Phase 2b Substrate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A request from the public internet reaches a pod on the `staging` cluster and returns, having travelled git → build → OpenTofu → cluster → rented edge.

**Architecture:** Two OpenTofu layers on one machine. Layer 0 uses `dmacvicar/libvirt` against `qemu:///system` on Arch to create the Proxmox VE guest — a layer that does not exist in production and exists only to validate the one above it. Layer 1 uses `bpg/proxmox` against the Proxmox API to create five Talos VMs, and its endpoint is a variable, which is what makes acquiring metal a variable change rather than a migration. Longhorn provides storage, a Cloudflare Tunnel provides the only inbound path, and the origin dials out so the machine needs no static address.

**Tech Stack:** OpenTofu, `dmacvicar/libvirt`, `bpg/proxmox`, Proxmox VE 9.x, Talos Linux, Kubernetes, Longhorn, SOPS + age, Cloudflare Tunnel + Cloudflare for SaaS.

**Spec:** `rfc/RFC-0023-the-development-substrate-is-production-one-level-down.md` (decisions 1–7), with `rfc/RFC-0016-the-edge-is-rented.md` (decisions 1 and 4), `adr/ADR-0052-opentofu-is-the-provisioning-tool.md`, `adr/ADR-0020-kubernetes-deployment-target.md`, `glossary/GLO-0011-environment.md`.

---

## ⛔ Preconditions — do not start Task 1 until both are true

1. **RFC-0023 and RFC-0016 are `accepted`** (T-2b.2). Both are `proposed`, which the precedence order in `CLAUDE.md` makes *direction, not yet binding*. This entire plan implements RFC-0023; building it while the RFC can still be rejected inverts the change lifecycle ADR-0013 exists to impose. **The status transition is the operator's act, not an agent's.**
2. **A price exists for Cloudflare's Data Localization Suite** (RFC-0016 decision 4, cost 1). The three controls that answered the residency criterion are a paid add-on. Tasks 1–5 are unaffected; **Task 6 must not begin without it**, because if the suite is unaffordable the vendor choice reopens and Task 6 is the task that adopts the vendor.

Phase 2's own blockers (T-2.1 `rust-ci.yml` never executed, T-2.2 `forge` undistributed, T-2.3 locked-but-unenforced artifacts) bite at **Task 7 only** — that is the first task to invoke CI on a real service. Tasks 1–6 create no Rust and call no reusable workflow.

## Global Constraints

Every task's requirements implicitly include this section. Values are copied verbatim from the artifacts named.

- **OpenTofu, never Terraform.** ADR-0052. Terraform ≥ v1.6 is BUSL and fails REQ-0012 exitability. The binary is `tofu`.
- **`terraform/` as a directory name and `terraform.fmt_and_validate` as a check id both stand unchanged.** ADR-0052 decision 2 — the identifier is vocabulary, and `tofu fmt` / `tofu validate` are the two operations it names. Do not rename either; ADR-0020 is `locked`.
- **Repositories: `infra-terraform` and `infra-kubernetes`.** ADR-0020 is `locked` and names both: *"Cluster-level infrastructure lives in `infra-kubernetes` and `infra-terraform`, versioned separately from services."* Do not consolidate them into one substrate repository; that contradicts a locked artifact.
- **`class: infra` is `deployable: false` and `ci: false`** (`class-registry.yaml`, ADR-0037). These repositories get **no CI pipeline and no Helm chart**, and ADR-0020's probes and resource limits do not apply to them. Verification in Tasks 1–6 is therefore local and manual by design, not by omission.
- **Required files in every repository:** `README.md`, `ARCHITECTURE.md`, `CHANGELOG.md`, `manifest.yaml` — STD-0002, `docs.required_files_present`, severity `error`. `manifest.yaml` must validate against `manifest.schema.json`.
- **`owner` in `manifest.yaml` is a team, never a person.** STD-0002: *"A manifest naming an individual is wrong the day they change role."*
- **One environment, named `staging`.** GLO-0011. Never `dev`. Never described as HA — RFC-0023 decision 3: three control-plane VMs on one physical host buy *"no availability whatsoever."*
- **Cluster shape:** 3 control plane @ 4 GB, 2 workers @ 8 GB, Proxmox VE guest 40 GB, Arch host 16 GB, 8 GB left unallocated. RFC-0023 decision 3 as revised 2026-08-19.
- **Every Talos node gets a second virtual disk** for Longhorn, sized as a variable. RFC-0023 decision 7 — every replicated CSI driver requires storage separate from the Talos install disk.
- **Node count, replica count, disk size and the Proxmox endpoint are all variables.** RFC-0023 decision 4: *"A configuration difference that cannot be expressed as a variable is a design difference"* — if you reach for a conditional on "is this the dev box", stop and amend the RFC instead.
- **PR size:** ≤ 250 lines fine · 251–400 warn with a stated reason · **> 400 fails**. STD-0007, measured on the branch diff. One task is one PR.
- **Every PR body cites an artifact id.** `pr.slice_cites_requirement` accepts `REQ-`, `RFC-` or `ADR-`. Cite the ids listed on each task.
- **Commits are signed.** RFC-0013 decision 3. The signing key is usable on the authoring host and nowhere else (RFC-0023 decision 5, addition of 2026-08-15).
- **The substrate pulls; it never receives.** RFC-0023 decision 5: the authoring host commits and pushes, the substrate pulls with a **per-repository read-only deploy key generated on the box**, which never leaves it and signs nothing. No `scp`, no file sync, no remote-mount build.

### ☠️ The one constraint whose violation cannot be undone

**The ADR-0025 `Stop` hook runs `git add -A` and commits and pushes automatically, and a secret is the only thing that stops it.** C-0004: a credential in history cannot be removed by a later commit.

This plan introduces a state-encryption passphrase, an age private key, a Proxmox root password, a Talos machine secret and a Cloudflare tunnel token. **Before writing any of them anywhere, the ignore rules of Task 1 must already be committed.** Every task below that touches a credential states where it goes; if a step is ambiguous, stop rather than guess.

---

### Task 1: The secrets mechanism, before anything that needs one

**Why this is first, against `TASKS.md` numbering.** T-2b.6 places the secrets mechanism after T-2b.4. That ordering was written before ADR-0052 decision 3 made state encryption part of the first `tofu` invocation. State cannot be encrypted by a mechanism that does not exist, and writing plaintext state "just for the first apply" is the shortcut C-0010 refuses. **T-2b.6 moves ahead of T-2b.4.**

RFC-0023 decision 6 authorises this explicitly: REQ-0001 has not fired and stays `deferred`, and the mechanism is built anyway, because *"a development environment with a different secrets story than production fails at precisely the transition this RFC exists to make seamless."*

**Slice:** No credential this substrate needs exists in plaintext on disk.
**Cites:** RFC-0023 §6 · ADR-0052 §3 · C-0004
**Files:**
- Create: `infra-terraform/.sops.yaml`
- Create: `infra-terraform/.gitignore`
- Create: `infra-terraform/secrets/staging.enc.yaml`
- Create: `infra-terraform/README.md`, `ARCHITECTURE.md`, `CHANGELOG.md`, `manifest.yaml`
- Create: `infra-terraform/SOP.md` (RFC-0032 — every repository carries one)

**Interfaces:**
- Produces: an age recipient public key committed in `.sops.yaml`; an age identity file at `~/.config/sops/age/keys.txt` on the Arch host **only**; `secrets/staging.enc.yaml` holding keys `state_passphrase`, `proxmox_root_password`, `cloudflare_tunnel_token`. Every later task reads credentials by decrypting this file and by no other route.

- [ ] **Step 1: Create the repository and its required files, and prove the ignore rules work before any secret exists**

```bash
gh repo create Paridyum/infra-terraform --private --clone
cd infra-terraform
cat > .gitignore <<'EOF'
# OpenTofu state is encrypted at rest, and is still never committed.
*.tfstate
*.tfstate.*
*.tfplan
.terraform/
.terraform.lock.hcl.bak
# Decrypted secret material. Never committed, under any name.
*.dec.yaml
*.dec.json
secrets/*.plain*
# age identities live outside the repository entirely; this is belt and braces.
keys.txt
*.agekey
EOF
git add .gitignore && git commit -S -m "chore: ignore state and decrypted secret material before any exists"
```

- [ ] **Step 2: Verify the ignore rules actually match, by planting a file that must be refused**

```bash
echo 'state_passphrase: planted-should-never-be-tracked' > secrets/staging.plain.yaml
git check-ignore -v secrets/staging.plain.yaml   # must print the matching rule
git status --porcelain | grep -c 'staging.plain' # must print 0
rm secrets/staging.plain.yaml
```
Expected: `git check-ignore` names the rule, and `git status` does not list the file. **If `git status` lists it, stop.** The ADR-0025 hook would commit it on the next tool call. This is T-2.13's lesson applied before the fact: verify the guard, do not assume it.

- [ ] **Step 3: Generate the age identity on the Arch host and commit only the recipient**

```bash
age-keygen -o ~/.config/sops/age/keys.txt        # on sitar-as-in-01-ar, not the macOS host
chmod 600 ~/.config/sops/age/keys.txt
age-keygen -y ~/.config/sops/age/keys.txt        # the public recipient — safe to commit
```

```yaml
# .sops.yaml
creation_rules:
  - path_regex: secrets/.*\.enc\.yaml$
    age: "age1...."   # the recipient printed above
```

- [ ] **Step 4: Create the encrypted secret file with a passphrase of at least 16 characters**

The 16-character floor is OpenTofu's own requirement for the `pbkdf2` key provider, not a preference.

```bash
tofu_pass="$(openssl rand -base64 32)"
sops --encrypt --input-type yaml --output-type yaml /dev/stdin > secrets/staging.enc.yaml <<EOF
state_passphrase: ${tofu_pass}
EOF
unset tofu_pass
sops --decrypt secrets/staging.enc.yaml | head -c 20   # proves round-trip
grep -c 'state_passphrase:' secrets/staging.enc.yaml   # present, but as ciphertext
```

- [ ] **Step 5: Generate the substrate's read-only deploy key on the substrate**

RFC-0023 decision 5 is specific about this and it is easy to satisfy accidentally wrongly: the
substrate pulls with a **per-repository read-only deploy key generated on the box**, which never
leaves it, is in no vault, and signs nothing. The signing key stays on the authoring host, and the
substrate must not be able to reach it — *"a substrate that can reach the signing key can author
signed commits"*, and the property that every deployed artifact traces to a signed commit from a
human's machine dissolves at that point.

```bash
# On sitar-as-in-01-ar, NOT the macOS host. One key per repository, read-only.
ssh-keygen -t ed25519 -N '' -C 'infra-terraform deploy key (read-only, sitar-as-in-01-ar)' \
  -f ~/.ssh/deploy_infra_terraform
gh repo deploy-key add ~/.ssh/deploy_infra_terraform.pub \
  --repo Paridyum/infra-terraform --title 'sitar-as-in-01-ar (read-only)'
```

Then prove the boundary holds in both directions:

```bash
# The substrate can pull:
GIT_SSH_COMMAND='ssh -i ~/.ssh/deploy_infra_terraform -o IdentitiesOnly=yes' \
  git -C /srv/infra-terraform pull --ff-only
# The substrate cannot push:
GIT_SSH_COMMAND='ssh -i ~/.ssh/deploy_infra_terraform -o IdentitiesOnly=yes' \
  git -C /srv/infra-terraform push 2>&1 | grep -qi 'read-only\|denied' && echo "push refused — correct"
# The substrate cannot sign:
ssh-add -l 2>/dev/null | grep -qi 'sign' && echo "SIGNING AGENT REACHABLE — violates RFC-0023 D5" || echo "no signing key reachable — correct"
```
Expected: pull succeeds, push is refused, no signing key is reachable. **The refused push is the
assertion**, in the same way STD-0009 makes the denial the assertion for tenant isolation.

- [ ] **Step 6: Prove no plaintext credential is tracked anywhere, then commit**

```bash
git add -A
git diff --cached | grep -iE 'passphrase: [A-Za-z0-9+/]{16}|BEGIN.*PRIVATE KEY|AGE-SECRET-KEY' && { echo "PLAINTEXT SECRET STAGED — ABORT"; exit 1; }
git commit -S -m "feat: age-encrypted secret store for the staging substrate

Cites RFC-0023 §6, C-0004."
```
Expected: the `grep` finds nothing and the commit succeeds. A hit means stop and do not commit.

---

### Task 2: OpenTofu runs, and its state is unreadable without the passphrase

**Slice:** `infra-terraform` state at rest is ciphertext.
**Cites:** ADR-0052 §1, §3 · STD-0002
**Files:**
- Create: `infra-terraform/versions.tf`
- Create: `infra-terraform/encryption.tf`
- Create: `infra-terraform/variables.tf`
- Modify: `infra-terraform/manifest.yaml`

**Interfaces:**
- Consumes: `secrets/staging.enc.yaml` key `state_passphrase` from Task 1.
- Produces: a working root module; the convention that **the passphrase reaches `tofu` through the `TF_ENCRYPTION` environment variable and never through a `.tfvars` file**, because a `.tfvars` file is a file and the Stop hook commits files.

- [ ] **Step 1: Pin the tool and declare no providers yet**

```hcl
# versions.tf
terraform {
  required_version = ">= 1.7.0"   # 1.7 is the floor for state encryption (ADR-0052 §3)
}
```

- [ ] **Step 2: Write the encryption block**

The canonical shape, from OpenTofu's state-encryption documentation. Note there is **no `default`** on the variable — a default would be a committed credential.

```hcl
# encryption.tf
variable "state_passphrase" {
  type        = string
  sensitive   = true
  description = "At least 16 characters. Supplied via TF_VAR_state_passphrase, sourced from sops. Never a default, never a .tfvars file."
}

terraform {
  encryption {
    key_provider "pbkdf2" "staging" {
      passphrase = var.state_passphrase
    }
    method "aes_gcm" "staging" {
      keys = key_provider.pbkdf2.staging
    }
    state {
      method = method.aes_gcm.staging
    }
    plan {
      method = method.aes_gcm.staging
    }
  }
}
```

- [ ] **Step 3: Run `tofu init` and confirm it fails without the passphrase**

```bash
tofu init
unset TF_VAR_state_passphrase
tofu apply -auto-approve
```
Expected: **FAIL**, asking for `state_passphrase`. Watching this fail is the point — it is the same discipline T-2.1 applies to `rust-ci.yml`, and an encryption block that silently no-ops is exactly the "reports success while enforcing nothing" failure the 2026-08-06 audit found.

- [ ] **Step 4: Supply the passphrase from sops and apply, then prove the state is ciphertext**

```bash
export TF_VAR_state_passphrase="$(sops --decrypt --extract '["state_passphrase"]' secrets/staging.enc.yaml)"
tofu apply -auto-approve
grep -c '"resources"' terraform.tfstate || echo "not plaintext JSON — expected"
head -c 64 terraform.tfstate; echo
python3 -c "import json,sys; d=json.load(open('terraform.tfstate')); print('encrypted' if 'encrypted_data' in d or 'meta' in d else 'PLAINTEXT — ABORT')"
```
Expected: no `"resources"` key readable, and the file identifies as encrypted. **If the state parses as ordinary plaintext OpenTofu state, the encryption block is not in effect — stop.**

- [ ] **Step 5: Run `tofu fmt` and `tofu validate`, satisfying ADR-0020's check by its intent**

```bash
tofu fmt -check -recursive
tofu validate
```
Expected: both pass. These are the two operations `terraform.fmt_and_validate` names; ADR-0052 decision 2 is the record that running them as `tofu` satisfies the locked check.

- [ ] **Step 6: Commit**

```bash
git add versions.tf encryption.tf variables.tf manifest.yaml
git commit -S -m "feat: encrypted OpenTofu state for the staging substrate

Cites ADR-0052 §1 and §3."
```

---

### Task 3: The Proxmox VE guest is created and destroyed by OpenTofu, not by hand

**⚠️ This task corrects `TASKS.md` T-2b.4, which named a tool that cannot do the job.** T-2b.4 says *"Provision the Proxmox guest with Terraform"* and names `bpg/proxmox`. `bpg/proxmox` drives the **Proxmox API**, which does not exist until Proxmox VE is installed — it manages resources *inside* Proxmox and cannot create the Proxmox VE guest itself. RFC-0023 decision 2's stack is Arch KVM → QEMU → Proxmox KVM → Talos, and the first arrow needs a different provider.

**Two providers, deliberately, and the split is the RFC's own boundary.** Layer 0 is `dmacvicar/libvirt` against `qemu:///system` and is the layer RFC-0023 decision 2 cost 3 calls *"a layer that does not exist in production."* Layer 1 (Task 4) is `bpg/proxmox` and is production-shaped. Keeping them as separate root modules means the transition to metal **deletes layer 0** rather than editing it.

**Slice:** The Proxmox VE guest is reproducible from an empty Arch host.
**Cites:** RFC-0023 §2 · ADR-0052 · RFC-0019 §1
**Files:**
- Create: `infra-terraform/layer0-hypervisor/versions.tf`
- Create: `infra-terraform/layer0-hypervisor/main.tf`
- Create: `infra-terraform/layer0-hypervisor/variables.tf`
- Create: `infra-terraform/layer0-hypervisor/answer.toml.tftpl`
- Create: `infra-terraform/layer0-hypervisor/README.md`

**Interfaces:**
- Produces: a reachable Proxmox VE API endpoint and an API token, written into `secrets/staging.enc.yaml` as `proxmox_api_token`. Task 4 consumes both and nothing else from this task.

- [ ] **Step 1: Build an unattended Proxmox VE installer ISO**

Proxmox VE supports fully unattended installation from a **TOML answer file**, with `proxmox-auto-install-assistant` producing a custom ISO whose boot menu auto-selects "Automated Installation" after ten seconds. This is what makes "not by hand" achievable.

Template the answer file so the root password comes from sops and never from a literal:

```toml
# answer.toml.tftpl
[global]
keyboard = "en-us"
country = "in"
fqdn = "${fqdn}"
mailto = "${mailto}"
timezone = "Asia/Kolkata"
root_password = "${root_password}"

[network]
source = "from-dhcp"

[disk-setup]
filesystem = "ext4"
disk_list = ["${install_disk}"]
```

```bash
# Run on the Arch host. Verify flags against `proxmox-auto-install-assistant --help`
# for the installed version before relying on this invocation.
proxmox-auto-install-assistant prepare-iso ./proxmox-ve_9.x.iso \
  --fetch-from iso --answer-file ./answer.toml --output ./pve-auto.iso
```

- [ ] **Step 2: Declare the libvirt provider and the guest**

`dmacvicar/libvirt` v0.8.3 (published 2026-05-31) is maintained and published to the OpenTofu registry. **Verify the exact attribute nesting for `libvirt_domain` against the v0.8.3 documentation before applying** — the resource follows the libvirt domain XML schema closely and its nested blocks change between minor versions. The properties this task requires are fixed:

- `memory = 40960` and `vcpu` sized to the host, per the revised RFC-0023 decision 3 table
- CPU mode passing host features through, so nested KVM is available to the L2 guests — **this is the whole reason the guest exists**; without it Task 4 cannot boot a Talos VM
- one volume for Proxmox itself, booting `pve-auto.iso` on first start
- `qemu-agent` enabled so the endpoint can be read back as an output

```hcl
# versions.tf
terraform {
  required_version = ">= 1.7.0"
  required_providers {
    libvirt = {
      source  = "dmacvicar/libvirt"
      version = "~> 0.8.3"
    }
  }
}

provider "libvirt" {
  uri = var.libvirt_uri   # "qemu:///system" here; the variable is the seam, not a conditional
}
```

- [ ] **Step 3: Verify nested virtualisation is exposed to the guest before going further**

```bash
tofu -chdir=layer0-hypervisor apply -auto-approve
# then, inside the Proxmox VE guest:
ssh root@<pve-guest> 'grep -o vmx /proc/cpuinfo | head -1; ls /dev/kvm'
```
Expected: `vmx` (or `svm`) present and `/dev/kvm` exists. T-2b.1 verified nested KVM on the **host** on 2026-08-15; this verifies it reaches the **guest**, which is a different claim and the one Task 4 depends on.

- [ ] **Step 4: Prove the guest is destroyable and reproducible**

```bash
tofu -chdir=layer0-hypervisor destroy -auto-approve
tofu -chdir=layer0-hypervisor apply -auto-approve
```
Expected: the guest is recreated and reaches a login prompt with no manual step. T-2b.4's done-condition is *"created and destroyed by OpenTofu, not by hand"* — a single successful `apply` does not demonstrate it, and RFC-0025 stage B3 wants this rehearsed rather than assumed.

- [ ] **Step 5: Create a scoped Proxmox API token, store it encrypted, and commit**

Create a token limited to what Task 4 needs rather than reusing `root@pam` with a password — REQ-0012 item 3 is least privilege, and this is the first place it applies.

```bash
sops --set '["proxmox_api_token"] "PVEAPIToken=..."' secrets/staging.enc.yaml
git add layer0-hypervisor secrets/staging.enc.yaml
git commit -S -m "feat: Proxmox VE guest provisioned by OpenTofu via libvirt

Corrects T-2b.4: bpg/proxmox cannot create the hypervisor it talks to.
Cites RFC-0023 §2, ADR-0052."
```

---

### Task 4: Five Talos nodes, production topology, each with a second disk

**Slice:** `kubectl get nodes` shows five nodes on the `staging` cluster.
**Cites:** RFC-0023 §3, §7 · RFC-0019 §2 · GLO-0011
**Files:**
- Create: `infra-terraform/layer1-cluster/versions.tf`
- Create: `infra-terraform/layer1-cluster/main.tf`
- Create: `infra-terraform/layer1-cluster/variables.tf`
- Create: `infra-terraform/layer1-cluster/talos.tf`
- Create: `infra-terraform/layer1-cluster/outputs.tf`

**Interfaces:**
- Consumes: `proxmox_api_token` and the endpoint from Task 3.
- Produces: a `kubeconfig` and a `talosconfig`, both written to paths matched by `.gitignore`; the cluster name `staging`; node names assigned by the cluster tooling per RFC-0021 decision 9, **not** from `device-registry.yaml`.

- [ ] **Step 1: Declare both providers, with the endpoint as a variable**

```hcl
terraform {
  required_version = ">= 1.7.0"
  required_providers {
    proxmox = {
      source  = "bpg/proxmox"
      version = "~> 0.111"        # 0.111.1, 2026-07-03, targets Proxmox VE 9.x
    }
    talos = {
      source  = "siderolabs/talos"
      version = "~> 0.7"          # verify the current minor before pinning
    }
  }
}

provider "proxmox" {
  endpoint  = var.proxmox_endpoint   # RFC-0023 §4: metal and guest expose the same API
  api_token = var.proxmox_api_token
  insecure  = var.proxmox_insecure   # true only while the guest has a self-signed certificate
}
```

- [ ] **Step 2: Declare the nodes from a variable, at production topology**

Node counts and sizes are variables set to production topology now — RFC-0023 decision 4's rule for node count is *"A Terraform variable. Decision 3 sets it to production topology now."*

```hcl
variable "control_plane" {
  type = object({ count = number, cores = number, memory = number })
  default = { count = 3, cores = 2, memory = 4096 }
}

variable "workers" {
  type = object({ count = number, cores = number, memory = number })
  default = { count = 2, cores = 4, memory = 8192 }   # 8 GB: Longhorn's floor, RFC-0023 §7
}

variable "longhorn_disk_gb" {
  type        = number
  default     = 100
  description = "Second disk per node. Longhorn requires storage separate from the Talos install disk."
}
```

Each node resource declares **two** disks: the Talos install disk and the Longhorn disk. Verify `bpg/proxmox`'s `disk` block shape against the v0.111 documentation.

- [ ] **Step 3: Add the Talos system extensions Longhorn requires**

Longhorn needs `iscsi-tools` for `iscsid`, `iscsiadm` and the `iscsi_tcp` module, and `util-linux-tools` for `nsenter` and `fstrim` from v1.5. On Talos these are **system extensions baked into the boot image**, not packages installed later — so getting this wrong means rebuilding every node, which is why it belongs in this task and not Task 5.

Produce the image schematic with both extensions via the Talos image factory, and use the resulting image id for every node.

- [ ] **Step 4: Apply, then verify node count and quorum behaviour**

```bash
export TF_VAR_state_passphrase="$(sops -d --extract '["state_passphrase"]' ../secrets/staging.enc.yaml)"
tofu -chdir=layer1-cluster apply -auto-approve
export KUBECONFIG=./kubeconfig
kubectl get nodes -o wide          # expect exactly 5, three of them control-plane
kubectl get nodes -o json | python3 -c "import json,sys; n=json.load(sys.stdin)['items']; print(len(n))"
```
Expected: `5`.

- [ ] **Step 5: Drain and rejoin a control-plane node**

```bash
kubectl drain <cp-node> --ignore-daemonsets --delete-emptydir-data
kubectl get nodes | grep <cp-node>     # SchedulingDisabled
kubectl uncordon <cp-node>
kubectl get nodes                      # Ready again, five nodes
```
Expected: the cluster stays serving throughout. This is T-2b.5's done-condition and the specific thing three control-plane nodes buy — RFC-0023 decision 3 is explicit that they buy *"correct etcd quorum behaviour, realistic drain and upgrade semantics"* and **no availability at all**. Do not record this as an HA test.

- [ ] **Step 6: Commit**

```bash
git add layer1-cluster
git commit -S -m "feat: Talos staging cluster, 3 control plane and 2 workers

Second disk per node for Longhorn; iscsi-tools and util-linux-tools baked in.
Cites RFC-0023 §3 and §7, RFC-0019 §2."
```

---

### Task 5: Longhorn serves a PersistentVolumeClaim

**Slice:** A pod on `staging` writes to a Longhorn volume and the data survives the pod.
**Cites:** RFC-0023 §7 · ADR-0020
**Files:**
- Create: `infra-kubernetes/README.md`, `ARCHITECTURE.md`, `CHANGELOG.md`, `manifest.yaml`, `SOP.md`
- Create: `infra-kubernetes/longhorn/values.yaml`
- Create: `infra-kubernetes/longhorn/storageclass.yaml`
- Create: `infra-kubernetes/longhorn/README.md`

**Interfaces:**
- Consumes: the `kubeconfig` from Task 4 and the second disk on each worker.
- Produces: a StorageClass named `longhorn` with `numberOfReplicas: "2"`, which every later workload and every `platform-` service consumes by name.

**This is the second repository, and ADR-0020 requires it to be separate.** `infra-kubernetes` holds cluster-level Kubernetes configuration; `infra-terraform` holds provisioning. Both are `ci: false` and `deployable: false`.

- [ ] **Step 1: Set the replica count as the variable that satisfies RFC-0023 decision 4**

Longhorn's default is three replicas and it wants at least three nodes; this cluster has two workers. **Two on the substrate, three on metal, and it is a value rather than a conditional** — that distinction is the entire reason Longhorn was chosen over Rook-Ceph.

```yaml
# longhorn/storageclass.yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: longhorn
  annotations:
    storageclass.kubernetes.io/is-default-class: "true"
provisioner: driver.longhorn.io
allowVolumeExpansion: true
reclaimPolicy: Delete
volumeBindingMode: Immediate
parameters:
  numberOfReplicas: "2"        # metal sets 3; RFC-0023 §7
  staleReplicaTimeout: "30"
  fsType: "ext4"
```

- [ ] **Step 2: Install Longhorn and point it at the second disk**

Set `defaultSettings.defaultDataPath` to the mount of the second disk rather than the default, so Longhorn cannot fill the Talos install disk. Verify current values keys against the Longhorn chart version being installed.

```bash
helm repo add longhorn https://charts.longhorn.io
helm install longhorn longhorn/longhorn -n longhorn-system --create-namespace -f longhorn/values.yaml
kubectl -n longhorn-system rollout status daemonset/longhorn-manager --timeout=5m
```

- [ ] **Step 3: Write the failing test — a PVC and a pod that writes to it**

```yaml
# /tmp/pvc-test.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata: { name: probe, namespace: default }
spec:
  accessModes: ["ReadWriteOnce"]
  storageClassName: longhorn
  resources: { requests: { storage: 1Gi } }
---
apiVersion: v1
kind: Pod
metadata: { name: probe-writer, namespace: default }
spec:
  restartPolicy: Never
  containers:
    - name: w
      image: busybox:1.36
      command: ["sh","-c","echo substrate-proof > /data/proof && sync"]
      volumeMounts: [{ name: d, mountPath: /data }]
  volumes:
    - name: d
      persistentVolumeClaim: { claimName: probe }
```

- [ ] **Step 4: Apply it and confirm the claim binds and the data survives the pod**

```bash
kubectl apply -f /tmp/pvc-test.yaml
kubectl get pvc probe -w                       # must reach Bound
kubectl wait --for=jsonpath='{.status.phase}'=Succeeded pod/probe-writer --timeout=120s
kubectl delete pod probe-writer
kubectl run probe-reader --image=busybox:1.36 --restart=Never -i --rm \
  --overrides='{"spec":{"containers":[{"name":"r","image":"busybox:1.36","command":["cat","/data/proof"],"volumeMounts":[{"name":"d","mountPath":"/data"}]}],"volumes":[{"name":"d","persistentVolumeClaim":{"claimName":"probe"}}]}}'
```
Expected: `substrate-proof`. A `Bound` PVC alone does not prove the driver works; reading the data back from a *different* pod does.

- [ ] **Step 5: Record a latency observation, labelled**

```bash
kubectl run fio --image=busybox:1.36 --restart=Never -i --rm -- sh -c 'time dd if=/dev/zero of=/data/t bs=1M count=256 conv=fsync' || true
```
Whatever number this produces, **write it down with the substrate named**. RFC-0023 risk R5: Longhorn replicates synchronously over a cluster network under two levels of nested virtualisation, so this figure is pessimistic by an unknown factor and must never be quoted as a baseline. T-0.4's D2 benchmark belongs on metal.

- [ ] **Step 6: Clean up the probe and commit**

```bash
kubectl delete -f /tmp/pvc-test.yaml
git add -A && git commit -S -m "feat: Longhorn storage on staging, two replicas

Cites RFC-0023 §7."
```

---

### Task 6: A hostname on the public internet resolves to the cluster

**⛔ Do not start without the Data Localization Suite price.** See Preconditions 2 — this is the task that adopts the vendor.

**Slice:** A public hostname resolves through the rented edge to a service on `staging`.
**Cites:** RFC-0016 §1, §4 · RFC-0023 §5
**Files:**
- Create: `infra-kubernetes/edge/cloudflared-deployment.yaml`
- Create: `infra-kubernetes/edge/README.md`
- Modify: `infra-terraform/secrets/staging.enc.yaml` (add `cloudflare_tunnel_token`)

**Interfaces:**
- Produces: a `cloudflared` Deployment holding an outbound tunnel, and a documented fallback-origin hostname that Cloudflare for SaaS points customer domains at.

- [ ] **Step 1: Create the tunnel and store its token encrypted**

The origin dials **out**. RFC-0023 decision 5: no inbound ports, no dynamic DNS, no static IP. RFC-0016 decision 1 item 1: TLS terminates at the edge.

```bash
cloudflared tunnel create staging-sitar
sops --set '["cloudflare_tunnel_token"] "<token>"' infra-terraform/secrets/staging.enc.yaml
```

- [ ] **Step 2: Create the Kubernetes secret from sops, never from a literal**

```bash
kubectl create namespace edge
sops -d --extract '["cloudflare_tunnel_token"]' infra-terraform/secrets/staging.enc.yaml \
  | kubectl -n edge create secret generic tunnel-credentials --from-file=token=/dev/stdin
```

- [ ] **Step 3: Deploy `cloudflared` with two replicas**

Two replicas so the tunnel survives a node drain — the drain rehearsed in Task 4 would otherwise
sever the only inbound path. Resource limits are set here by choice, not by ADR-0020, which does not
bind `infra-*` (ADR-0037).

```yaml
# infra-kubernetes/edge/cloudflared-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cloudflared
  namespace: edge
  labels:
    app.kubernetes.io/name: cloudflared
    app.kubernetes.io/part-of: paridyum
    app.kubernetes.io/component: edge
spec:
  replicas: 2
  selector:
    matchLabels: { app.kubernetes.io/name: cloudflared }
  template:
    metadata:
      labels:
        app.kubernetes.io/name: cloudflared
        app.kubernetes.io/part-of: paridyum
        app.kubernetes.io/component: edge
    spec:
      affinity:                       # keep the two replicas off one node
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
            - labelSelector:
                matchLabels: { app.kubernetes.io/name: cloudflared }
              topologyKey: kubernetes.io/hostname
      containers:
        - name: cloudflared
          image: cloudflare/cloudflared:latest   # pin to a digest before this is load-bearing
          args: ["tunnel", "--no-autoupdate", "--metrics", "0.0.0.0:2000", "run"]
          env:
            - name: TUNNEL_TOKEN
              valueFrom:
                secretKeyRef: { name: tunnel-credentials, key: token }
          livenessProbe:
            httpGet: { path: /ready, port: 2000 }
            initialDelaySeconds: 10
          readinessProbe:
            httpGet: { path: /ready, port: 2000 }
          resources:
            requests: { cpu: 50m, memory: 64Mi }
            limits:   { cpu: 500m, memory: 256Mi }
```

⚠️ **`image: latest` is a placeholder here and must not survive the commit.** Pin it to a digest —
ADR-0021 already forbids mutable tags for third-party GitHub Actions on the grounds that *"a mutable
tag on a third-party action is remote code execution with extra steps."* The same argument applies to
the container that terminates your only inbound path.

- [ ] **Step 4: Verify the tunnel is registered and the route resolves end to end**

```bash
kubectl -n edge rollout status deploy/cloudflared --timeout=3m
kubectl -n edge logs deploy/cloudflared | grep -i 'registered tunnel connection'
curl -sS -o /dev/null -w '%{http_code} %{ssl_verify_result}\n' https://<hostname>/
```
Expected: registered connections in the log, and a response over TLS the client validates. **Run the `curl` from a network outside the house** — from the Arch host it can succeed for the wrong reason.

- [ ] **Step 5: Confirm no inbound port was opened, which is the actual claim**

```bash
sudo ss -ltnp | grep -vE '127.0.0.1|::1' || echo "no non-loopback listener — expected"
```
Expected: nothing listening publicly. The tunnel works *because* nothing is exposed; verifying the absence is what distinguishes this from having opened a port and forgotten.

- [ ] **Step 6: Commit**

```bash
git add -A && git commit -S -m "feat: Cloudflare tunnel from staging to the rented edge

Origin dials out; no inbound ports. Cites RFC-0016 §1 and §4, RFC-0023 §5."
```

---

### Task 7: The chain, proven by the smallest possible service

**⛔ This task is gated on two decisions that do not exist, and it cannot be written as steps until they do.** Everything in Tasks 1–6 is retrievable from an existing artifact. This is not, and inventing the missing pieces inside an implementation plan is the failure `CLAUDE.md` names first: *never infer architecture, retrieve it*.

**Slice:** A request from the public internet reaches a pod and returns.
**Cites:** RFC-0023 exit gate · ADR-0020 · ADR-0021 · GLO-0011

#### Decision owed 1 — the HTTP framework

ADR-0005 locks Rust and `templates/rust-service/` exists, but it holds toolchain, lints, `deny.toml`, `nextest.toml` and CI — **no `src/`, and no artifact anywhere names an HTTP framework.** Searched: no `axum`, `actix`, `warp` or `hyper` appears in any artifact, registry or template. So there is no way to write this service's handler without choosing one, and choosing one in a plan file makes an architectural decision in the least reviewable place in the repository.

**This is an ADR**, and it is cheap now and structural later: it is the shape every `platform-` service in ADR-0042's first wave inherits, and RFC-0009's Q5 is already open about how much `lib-` code they share.

#### Decision owed 2 — who owns the chart shape

ADR-0020 requires a `Dockerfile`, a `helm/` chart with Deployment, Service, Ingress, HPA and ServiceAccount, probes, resource limits, and a `terraform/` skeleton. **None of it exists anywhere in the estate** — no `Dockerfile`, `Chart.yaml`, `*.tf` or `values.yaml`, verified 2026-08-19 — and every `helm.*` check is still owed to `tool-forge`, with ADR-0020 itself one of the locked-but-unenforced artifacts T-2.3 is about.

Worse, ADR-0042 fixes the first wave at five contexts and omits `platform-deployment`, so per `ROADMAP.md` §4 *"Release, Certificate, and the act of running an artifact have no owner."*

**Whatever chart this task writes becomes the template every later service copies.** That is a template decision masquerading as a task, and it should be one deliberate slice — probably in `templates/rust-service/`, with an artifact recording the runtime contract — rather than a by-product of proving a network path.

#### What is already known, and survives either decision

Recorded here so it is not re-derived when the gates clear:

- **The build must be in the chain.** RFC-0023's exit gate says *"git → build → OpenTofu → cluster → rented edge"*. A pre-built public image such as `nginx` would prove the network path while skipping `build` and GHCR, so it does **not** satisfy the gate — worth stating, because it is the obvious shortcut and it looks equivalent.
- **The six recommended labels are the one irreversible detail.** `app.kubernetes.io/name`, `instance`, `version`, `component`, `part-of`, `managed-by`, with `part-of: paridyum` (`helm.declares_recommended_labels`). A label in `metadata` can be added later by a rolling update; **one belonging in `spec.selector` cannot be added at all**, because that field is immutable on an `apps/v1` Deployment.
- **The image is slim or distroless**, the build stage is stack-shaped and the runtime contract is not (ADR-0020).
- **The deploy is manually approved and that is correct.** Per GLO-0011 there is exactly one environment, `staging`, gated by a GitHub Environment; ADR-0021's automatic path targets a `dev` that deliberately does not exist. Approving by hand is the design, not a workaround.
- **Expect `rust-ci.yml` to be broken.** T-2.1: the checks it declares **have never executed**. This is the first caller. Do not repair it inside this task's PR — it lives in `infra-ci` and is its own slice.
- **The service must be stateless.** Do not consume the `longhorn` StorageClass here; Task 5 already proved the driver, and coupling the two means a storage failure reads as a chain failure.
- **Verification is from outside the house**, on both the request and the absence of a listener:

```bash
curl -sS -w '\n%{http_code} %{time_total}s\n' https://<hostname>/
kubectl -n <ns> get pods -l app.kubernetes.io/part-of=paridyum
```
Expected: the service's body and `200`. That single request is Phase 2b's exit gate.

- **Closing the phase means recording which parts were watched failing** versus merely observed succeeding, in `TASKS.md` and `ROADMAP.md`. `CLAUDE.md` closes on exactly this: *"treat a green check as unproven until you have watched it fail."*

#### Recommended sequencing

Take the two decisions as their own work before returning here — an ADR for the framework, and one deliberate slice for the template and chart shape. Then Task 7 becomes an ordinary TDD task: failing test for the handler, minimal handler, Dockerfile, chart, merge, approve, `curl` from outside. **Tasks 1–6 do not depend on either decision and can proceed to completion first**, which is the argument for not blocking the whole plan on them.

---

## Notes for whoever executes this

**Run `paridyum-platform:design-change` before Task 7's code**, per the `CLAUDE.md` router: significance is computed from the diff and never self-declared (ADR-0013). Tasks 1–6 are configuration in `ci: false` repositories; Task 7 is the first real service.

**The research-capture hook does not fire from a working directory above the repository** (T-2.15, found 2026-08-19). If any step here sends you to vendor documentation, `research/` will not record it unless you work from inside the repository. Several syntax details in this plan are marked *verify against the current documentation* for exactly that reason — they were not captured when this plan was written, so they are assertions rather than evidence.

**Five places this plan deliberately says "verify before applying"** rather than pinning syntax: `libvirt_domain` attribute nesting, `proxmox-auto-install-assistant` flags, the `bpg/proxmox` `disk` block, the `siderolabs/talos` provider minor version, and Longhorn's `values.yaml` keys. Each is vendor syntax that moves between minor versions, and `CLAUDE.md`'s rule applies to vendors too — retrieve it, do not infer it. Pinning a wrong attribute here would read as authoritative.
