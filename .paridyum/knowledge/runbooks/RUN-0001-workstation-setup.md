---
id: RUN-0001
title: Provisioning a Paridyum development workstation
type: runbook
status: accepted
created: 2026-08-06
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [onboarding, workstation, bootstrap, macos, linux, windows, wsl2]
breaking: false
enforced_by:
  - check: workstation.doctor_passes
    severity: warn
    rationale: Enforced on the machine by bootstrap --check / forge doctor, not by repository CI.
---

# RUN-0001 — Provisioning a Paridyum development workstation

The answer to "how was this set up?" asked by someone — including future-you — with a
blank machine and no context. Paired with [`scripts/bootstrap.sh`](../scripts/bootstrap.sh),
which performs every step idempotently. The prose exists because the script says
*what* and only this says *why*.

**Platform support is fixed by ADR-0006**: macOS and Linux natively, Windows through
WSL2. Native Windows is not supported.

| Environment    | Path                                                         |
| -------------- | ------------------------------------------------------------ |
| macOS          | `scripts/bootstrap.sh`                                        |
| Linux          | `scripts/bootstrap.sh`                                        |
| Windows        | `scripts/bootstrap.ps1` (elevated) → reboot → `bootstrap.sh` inside WSL2 |

## Directory layout

Identical on every platform, including inside WSL2:

```
~/Developer/
├── paridyum/       all Paridyum work — a folder of clones, not a repo (ADR-0001)
│   ├── knowledge/  ← clone of Paridyum/knowledge. Everything starts here.
│   ├── platform/   clones of platform-*
│   ├── services/   clones of svc-*
│   ├── libs/       clones of lib-*
│   ├── tools/      clones of tool-*
│   ├── games/      clones of game-* (ADR-0133)
│   ├── web/        clones of web-*
│   ├── consoles/   clones of console-*
│   ├── mobile/     clones of mobile-*
│   ├── agents/     clones of agent-*
│   ├── sdks/       clones of sdk-*
│   ├── infra/      clones of infra-*
│   ├── specs/      clones of spec-*
│   └── templates/  clones of template-*
├── personal/       unrelated to Paridyum
├── experiments/    throwaway; never deployed, never depended on
├── archived/       dead projects kept for reference
└── scratch/        deleted without warning
```

The `paridyum/` subdirectories mirror the class prefixes in `class-registry.yaml` (C-0007), so a
repository's name determines where it is cloned with no lookup. A class added to the registry adds
its line here; until 2026-10-07 this tree still held only C-0002's classes, while `web/` and
`consoles/` already existed on disk.

**Why `~/Developer` and not `~/Documents` (macOS).** `~/Documents` is synced by iCloud
Drive on many Macs, and syncing a `.git` directory or a Cargo `target/` directory
produces corruption and multi-gigabyte uploads. *(Verified 2026-08-06: sync is off on
the current machine — but the layout should not depend on a setting that can be
toggled in System Settings.)*

**Why `~/Developer` inside WSL and never `/mnt/c` (Windows).** Repositories must live
on the Linux filesystem. Cross-filesystem I/O through the Windows mount is roughly an
order of magnitude slower, and Linux permissions are not preserved, so the executable
bit on every script silently disappears.

## Toolchain

**This table answers *how do I install this here*. `tool-registry.yaml` answers *why do we have it,
under what licence, and how would we leave* (ADR-0066).** Neither is generated from the other, so
they can disagree — TASKS T-2.17 — and where they do, the registry owns the adoption facts and this
table owns the per-platform command. Neither owns a version: those are in
`toolchain-registry.yaml`, or in the OS package manager, which is an admission rather than a pin.

Two tools are installed from the registry rather than from this table:

```sh
./scripts/install-tools.sh          # clones each pinned tool at its recorded commit sha
./scripts/install-tools.sh --check  # report what is missing, install nothing
```

`archify` (RUN-0006) is the only one today. It is deliberately **not** installed with the
`npx skills add` command its own README documents — that delegates the clone to a third-party CLI
at no pinned revision.

| Tool          | Purpose                                   | macOS               | Debian/Ubuntu            | Fedora              | Arch          |
| ------------- | ----------------------------------------- | ------------------- | ------------------------ | ------------------- | ------------- |
| `git`         | —                                         | `brew`              | `apt`                    | `dnf`               | `pacman`      |
| `gh`          | forge's GitHub API client                 | `brew`              | `apt`                    | `dnf`               | `pacman`      |
| rustup/cargo  | primary language (ADR-0005)               | rustup.rs           | rustup.rs                | rustup.rs           | rustup.rs     |
| `docker`      | local dependencies + image builds         | Desktop (cask)      | Docker Engine            | Docker Engine       | `pacman`      |
| `kubectl`     | cluster access (ADR-0020)                 | `kubernetes-cli`    | **k8s apt repo**         | `kubernetes-client` | `kubectl`     |
| `helm`        | chart authoring and linting (ADR-0020)    | `brew`              | `apt`                    | `dnf`               | `pacman`      |
| `tofu`        | provisioning and service modules (ADR-0052) | `opentofu`        | **OpenTofu apt repo** (`tofu`) | **OpenTofu RPM repo** | `opentofu` |
| `sops`        | secret encryption (RFC-0023 D6)           | `brew`              | **GitHub release**       | **GitHub release**  | `pacman`      |
| `age`         | key backend for `sops` (RFC-0023 D6)      | `brew`              | `apt`                    | `dnf`               | `pacman`      |
| `talosctl`    | Talos cluster management (RFC-0019 D2)    | **`siderolabs/tap`** | **install script**      | **install script**  | **install script** |
| `jq`          | required by the enforcement hooks         | `brew`              | `apt`                    | `dnf`               | `pacman`      |
| `just`        | task runner in generated repos            | `brew`              | `apt` (universe)         | `dnf`               | `pacman`      |
| `sqlx-cli`    | migrations, offline query metadata        | `cargo install sqlx-cli --no-default-features --features postgres,rustls` |||
| `cargo-chef`  | Docker dependency-layer caching           | `cargo install cargo-chef` |||
| `cargo-deny`  | licence and advisory policy in CI         | `cargo install cargo-deny` |||
| `cargo-audit` | RustSec advisories                        | `cargo install cargo-audit` |||

**Bold** entries are not in default repositories and need a vendor repository added
first. `bootstrap.sh` detects this and prints the specific instructions rather than
failing opaquely.

**`tofu` replaced `terraform` on 2026-08-20, and the replacement is not cosmetic.** ADR-0052
decides OpenTofu and declines the Terraform binary, because Terraform has shipped under BUSL
since v1.6 and fails REQ-0012's exitability test — the same test that made RFC-0025 reject
Sidero Omni. A workstation that installs the HashiCorp binary is provisioning the tool the
org has decided against, so this row was the decision's last unclosed edge.

**What did *not* change, deliberately:** ADR-0020 still requires a directory named `terraform/`
in every deployable repository, and `terraform.fmt_and_validate` is still the check id. ADR-0052
decision 2 explains why both stand — the identifier is vocabulary, and `tofu fmt` and
`tofu validate` are the two operations the check names. Do not "correct" either to match this row.

The `kubectl` and `helm` rows cited ADR-0003, which is `superseded`. They now cite ADR-0020,
which supersedes it. The citation was stale rather than wrong: ADR-0020 carries the same
Kubernetes obligations forward.

`sops` reads its identity from `SOPS_AGE_KEY_FILE`, conventionally
`~/.config/sops/age/keys.txt`. Under RFC-0023 decision 5 that identity is generated **on the
substrate** and the signing key stays on the authoring host — the two must not meet on one
machine.

**Install Rust via rustup on every platform, never from the system package manager.**
Distro and Homebrew Rust packages cannot switch toolchains, ignore
`rust-toolchain.toml`, and will silently disagree with CI about the compiler version.
Pinning the toolchain per repository is what makes builds reproducible. The bootstrap
detects a shadowing system package and tells you how to remove it.

## Machine state as found — 2026-08-06 (macOS)

Recorded so a later reader can tell what this runbook actually changed.

```
macOS 26.3 (arm64)           Homebrew 6.0.13
git        2.55.0            present
gh         2.89.0            present, authed as rohanbatrain, scopes: gist read:org repo
docker     29.1.3            present, daemon NOT running
kubectl / helm               present
claude code                  present
rustc / cargo                NOT INSTALLED
terraform / just / jq        NOT INSTALLED
~/Developer                  did not exist
ssh key                      id_rsa only (no ed25519)
iCloud Documents sync        off
Paridyum GitHub org          exists, Free plan, rohanbatrain is a member
```

## GitHub authentication

`forge` needs more than the default `gh` scopes. As found, the token carried
`gist, read:org, repo` — enough to clone and push, **not** enough to do forge's job.

| Scope       | Needed for                                                       |
| ----------- | ---------------------------------------------------------------- |
| `repo`      | create repositories, push, read/write contents                    |
| `workflow`  | **push files under `.github/workflows/`** — without this, every scaffold that includes CI fails at push with an opaque error |
| `admin:org` | create repositories in the org, manage teams referenced by CODEOWNERS, apply org settings |

```sh
gh auth refresh -h github.com -s workflow,admin:org
```

The `workflow` scope produces the most confusing failure when missing: the *push* is
rejected, not the file write, so the error surfaces long after the cause.

## Git configuration

Conventional Commits and signed commits are org standards. SSH signing rather than
GPG — the key already exists for push authentication, and there is no keyring to
manage:

```sh
git config --global user.name  "Rohan Batra"
git config --global user.email "<org email>"
git config --global init.defaultBranch main
git config --global pull.rebase true
git config --global commit.gpgsign true
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/id_ed25519.pub
git config --global core.autocrlf false      # Windows only — ADR-0006
```

The public key must also be registered on GitHub **as a signing key**, separately from
its registration as an authentication key. Same key, two entries — commits show
"Unverified" until the second one exists.

### Where the private key lives — prefer an agent

*Added 2026-08-15.*

**Recommended: serve the key from a password manager's SSH agent rather than a file.**
Bitwarden Desktop's SSH agent (2025 releases) exposes vault-held SSH keys over a socket,
so the private key is never written to disk, is synced across machines by the vault, and
is revocable in one place. macOS and Linux desktop builds both support it; on Arch the
desktop app is packaged in the AUR.

**Two things about this are not obvious, and both were found the hard way on 2026-08-15.**

**The socket is inside the app container on a sandboxed build.** The Mac App Store build of
Bitwarden is sandboxed, so the documented `~/.bitwarden-ssh-agent.sock` is a dead file — it
exists, and connecting to it returns `Connection refused`. The live socket is:

```
~/Library/Containers/com.bitwarden.desktop/Data/.bitwarden-ssh-agent.sock
```

Confirm with `lsof` against the Bitwarden process rather than trusting either path:

```sh
pgrep -f 'Bitwarden.app/Contents/MacOS/Bitwarden$' | xargs -I{} lsof -p {} | grep ssh-agent
```

**Export it from `~/.zshenv`, never `~/.zshrc`.** `.zshrc` is sourced only for *interactive*
shells. **ADR-0025's checkpoint hook runs a non-interactive shell**, so with the export in
`.zshrc` signing works every time you test it by hand and fails every time the hook fires — the
silent failure this section warns about, arriving through the one door nobody checks. `.zshenv` is
sourced for every zsh invocation.

```sh
# ~/.zshenv
export SSH_AUTH_SOCK="$HOME/Library/Containers/com.bitwarden.desktop/Data/.bitwarden-ssh-agent.sock"
```

```sh
git config --global gpg.format ssh
git config --global user.signingkey "key::$(ssh-add -L | head -1)"
git config --global gpg.ssh.allowedSignersFile ~/.config/git/allowed_signers
```

`user.signingkey` takes the literal public key with a `key::` prefix rather than a path, which is
what lets `ssh-keygen -Y sign` resolve the private half through the agent with nothing on disk.
`allowedSignersFile` is what makes `git log --show-signature` able to *verify* rather than only
sign — one line of `<email> <public key>`.

**Verify it the way the hook will exercise it**, not the way you would by hand:

```sh
env -u SSH_AUTH_SOCK zsh -c 'cd $(mktemp -d) && git init -q . && echo x > f && git add f &&
  git commit -q -m test && git log --show-signature -1 --format="%G?"'
# must print: G
```

`%G?` prints `G` for a good signature. Stripping `SSH_AUTH_SOCK` and using a non-interactive
`zsh -c` reproduces the checkpoint hook's environment; a setup that only passes without those two
things is the failing one.

`user.signingkey` takes the literal public key with a `key::` prefix rather than a path,
which is what lets `ssh-keygen -Y sign` resolve the private half through the agent. A
`.pub` file on disk works equally well as the value; only the private half stays in the
vault.

**⚠️ The agent must be unlocked when ADR-0025's checkpoint fires.** `commit.gpgsign true`
with no reachable signing key does not warn and continue — **`git commit` fails
outright**, so the Stop hook's commit fails and the mechanism that exists to stop work
being lost stops working. It fails at the moment you have walked away, which is the
moment it was written for.

Two things make this safe rather than theoretical:

- Set the vault's lock timeout to span a working session, not minutes.
- If the agent prompts for per-request approval, an unattended commit will hang rather
  than fail. Disable approval for the signing key, or accept that checkpoints need you
  present.

**Fallback — a key file on disk.** Legitimate, and what most setups run. The vault then
serves as backup rather than as runtime protection, and the key's actual protection is
file permissions plus full-disk encryption:

```sh
chmod 600 ~/.ssh/id_ed25519
ssh-keygen -y -f ~/.ssh/id_ed25519 > ~/.ssh/id_ed25519.pub   # derive, don't copy separately
ssh-add --apple-use-keychain ~/.ssh/id_ed25519               # macOS: cache the passphrase
git config --global user.signingkey ~/.ssh/id_ed25519.pub
```

`ssh-add --apple-use-keychain` is what makes unattended commits work here; without it
this fallback has the same failure as a locked vault.

### On a substrate host, use a deploy key instead

**Do not connect the substrate to your vault agent.** RFC-0023 decision 5 requires that
the signing key be usable only on the authoring host — a substrate that can reach it can
author signed commits, and the property that every deployed artifact traces back to a
human's machine dissolves quietly.

A build or deploy host needs to *pull*, not to author. Generate a read-only deploy key on
that host, register it per-repository on GitHub, and leave it out of the vault entirely.
It never needs to leave the machine, which is the point.

## Bootstrap

**macOS / Linux / inside WSL2**

```sh
git clone git@github.com:Paridyum/knowledge.git ~/Developer/paridyum/knowledge
~/Developer/paridyum/knowledge/scripts/bootstrap.sh
```

**Windows, first**

```powershell
# elevated PowerShell
.\bootstrap.ps1        # installs WSL2 + Ubuntu, checks Docker integration, fixes autocrlf
# reboot, then run the Linux commands above inside `wsl`
```

Both scripts are idempotent and safe to re-run. Neither overwrites existing git config.

## Agent harness

Per ADR-0007 the Claude Code harness is a **plugin**, not vendored config. Install it
once per machine:

```
/plugin marketplace add Paridyum/knowledge
/plugin install paridyum-platform@paridyum
```

Repositories scaffolded by `forge` declare the marketplace in `.claude/settings.json`,
so this is normally offered automatically when you trust the folder.

This is what gives you the `PreToolUse` guard that blocks edits contradicting locked
decisions, and the `SessionStart` injection of the rules binding the current
repository. Without it an agent still reads the vendored knowledge, but nothing
enforces it.

## Verification

`scripts/bootstrap.sh --check` verifies without changing anything. Once `tool-forge`
ships, `forge doctor` supersedes it and this section is updated to say so.

Known limitation: on macOS, `gh` stores its token in the Keychain, which a detached or
non-TTY process cannot unlock. Running `--check` outside a terminal reports gh as
unauthenticated. The script detects this and warns rather than failing.

## Rebuilding from nothing

1. Install a package manager (Homebrew on macOS; the distro's own on Linux).
   On Windows: run `bootstrap.ps1` elevated, reboot, then continue inside WSL2.
2. Generate an SSH key; register it on GitHub **twice** — as an auth key and as a
   signing key.
3. `gh auth login`, then the `gh auth refresh` above.
4. Clone `knowledge`, run `bootstrap.sh`.
5. Install the `paridyum-platform` plugin.
6. `forge doctor`.

Steps 1–3 cannot be scripted from a truly blank machine because they require browser
interaction. Everything after step 3 can.

---

## Machine state as found — 2026-08-23 (macOS, `Darwin 25.3.0`, arm64)

**Appended rather than replacing the 2026-08-06 section**, per ADR-0031's additive-and-dated
rule. This is a second observation of the same machine, not a correction of the first.

**Why this section exists.** The operator's standing rule is that governance precedes
implementation, so that work is reproducible rather than remembered. A record of what is
actually installed is the first half of that; the second half is making it declarative, which
is owed below and not yet done.

### Toolchains observed

| | Installed | Declared in `toolchain-registry.yaml` |
| --- | --- | --- |
| Rust | `1.97.1` | ✅ `channel: 1.97.1`, floor `1.94` |
| Node | `v25.9.0` | ❌ no entry |
| pnpm | `11.22.0` | ❌ no entry |
| Python | `3.14.6` (plus `pyenv`, `python@3.11`, `python@3.12`) | ❌ no entry |
| Ansible | `ansible-core 2.21.1` via pipx | ✅ added 2026-08-23, floor `2.16.0` |
| Go, Java | `openjdk 17.0.18` | ❌ no entry |

Node and pnpm happen to satisfy what `console-docket` and `console-testing` require
(`node >=25.9.0`, `pnpm@11.22.0`) — by coincidence rather than by a pin, since no entry exists.

### Three drift findings

**1 · Terraform is installed and OpenTofu is not.** `hashicorp/tap/terraform` v1.15.8 is
present; `tofu` is absent. **ADR-0052 chose OpenTofu**, and chose it on licence grounds — BUSL
1.1 fails REQ-0012's exitability test. The workstation therefore carries the tool the estate
decided against and lacks the one it decided on. Nothing has been provisioned with either, so
this is drift rather than damage, and the fix is `brew install opentofu` plus removing the tap.

**2 · `toolchain-registry.yaml` covers one stack.** Only `rust` has an entry. RFC-0028 already
recorded this for TypeScript — *"`toolchain-registry.yaml` has no TypeScript entry at all"* —
and ADR-0048 open question 3, when a pin advances, is unanswered. Ansible now joins the list of
undeclared toolchains, added by ADR-0056.

**3 · A minor Ansible skew.** `ansible-core` is 2.21.1 while `ansible-lint` bundles 2.21.2.
Harmless today; recorded because it is the kind of difference that explains a lint result that
cannot be reproduced.

### Fonts

`font-inter`, `font-jetbrains-mono` and `font-eb-garamond` are installed. Worth noting against
the design system: `brand.css` currently sets **system stacks**, and neither IBM Plex nor
Archivo — proposed but unratified for `lib-ui` — is present. Type remains an open decision, and
the installed fonts do not settle it.

### What changed on this machine on 2026-08-23

Recorded because every one of these was done **by hand**, which is the debt this section exists
to make visible.

| Change | Why | Reproducible? |
| --- | --- | --- |
| `ansible-core`, `ansible-lint` via pipx | ADR-0056 | ❌ by hand |
| Collections `community.general`, `community.libvirt`, `ansible.posix` | `infra-substrate` | ✅ `requirements.yml` |
| `~/.ssh/config`: `Host sitar` with `ControlPersist 8h` | NetBird's SSH server is JWT-gated and prompts for SSO per connection; multiplexing makes one login cover a run | ❌ by hand |
| NetBird profile switched `default` → `Production-Devices`; peer deregistered and re-registered | The peer key was registered under a different account, so no SSO could claim it | ❌ by hand |
| ~~`paridyum/.secrets`~~ — **deleted 2026-08-23** | Held the GitHub PAT and NetBird setup key in plaintext. Retired under ADR-0057: both are *operator* credentials and now live in the operator's password manager, which RUN-0004 makes the rule rather than a preference | ✅ superseded |
| SOPS + age installed; key at `~/.config/sops/age/keys.txt` | ADR-0057. The key is backed up in the password manager and is the only secret a new machine needs | ⚠️ by hand — RUN-0004 §"new machine" is the procedure until a workstation play exists |

⚠️ **The GitHub PAT is a classic token carrying `admin:enterprise`, `admin:org`, `delete_repo`
and `delete:packages`.** Publishing a package needs `write:packages` alone. This is ADR-0045
risk R3 and REQ-0001 in concrete form; a fine-grained token scoped to packages on the `Paridyum`
organisation is the least-privilege replacement.

**It is now also a CI secret.** `web-paridyum` carries it as `PARIDYUM_PACKAGES_TOKEN`, because
on the Free plan `GITHUB_TOKEN` cannot read another repository's private package. That widens
the same risk rather than creating a new one, and it is the strongest argument for the
fine-grained replacement above.

### What is owed

1. **A `workstation` play in `infra-substrate`.** Everything marked ❌ above should be
   declarative. STD-0016 rule 4 already permits it — a workstation is another inventory group,
   not another repository — and the operator's own rule makes it required rather than optional.
2. ~~`toolchain-registry.yaml` entries for typescript and ansible~~ — **both done
   2026-08-23**. The typescript one found something worse than drift; see the addendum below.
   Three stacks are now pinned and none has an advance policy, which is ADR-0048 open
   question 3 and is unchanged by this.
3. ~~OpenTofu installed and Terraform removed~~ — **done 2026-08-23**, closing finding 1.
   OpenTofu `v1.12.6` installed; `hashicorp/tap/terraform` 1.15.8 uninstalled and the tap
   untapped, on the operator's explicit instruction. The workstation now carries the tool
   ADR-0052 chose and not the one it decided against.
4. **A fine-grained GitHub token**, closing the R3 note above.

### Addendum — 2026-08-23: the missing pin was hiding an end-of-life runtime

Writing the toolchain entry required checking what actually forced the Node floor, and the
answer was nothing. `next` 16.3.1 asks for `>=20.9.0`; `vitest` 4.1.11 accepts
`^20 || ^22 || >=24`. No dependency asked for 25.

**Node 25 reached end-of-life on 2026-06-01**, and its final release was 25.9.0 on
2026-03-31. All three TypeScript repositories were pinned to it, so the estate had been
running a runtime with no security patches for nearly three months. Verified against
`nodejs/Release` `schedule.json` and `nodejs.org/dist/index.json`.

**Node's odd-numbered majors never become LTS.** A pin taken from whatever `node --version`
printed has an even chance of landing on one, and that is exactly what happened: the first
repository copied the workstation, and the next two copied the first — the mechanism this
section had described as "drift" a few hours earlier, with a consequence nobody had checked.

All three now pin **24.19.0** (Active LTS, released 2026-08-03, supported to 2028-04-30) and
their CI is green on it. Revisit after 2026-10-28, when Node 26 becomes LTS and 24 enters
maintenance.

**The general lesson, which is ADR-0048's own thesis arriving in a second stack:** a value
with no registry entry does not stay unpinned. It gets copied from whatever was nearby, and
nobody checks what that was.

Until item 1 lands, this section **is** the reproducibility mechanism, and it decays the moment
somebody installs something without appending to it. That is a discipline rather than a
property — the same sentence RFC-0023 uses about the substrate, and it is true here for the
same reason.

---

## Addendum — 2026-08-23: NetBird rewrites this workstation's SSH configuration

**A vendor owns a file in `/etc/ssh/` on every machine running its client**, and it says so:

```
/etc/ssh/ssh_config.d/99-netbird.conf
  # NetBird SSH client configuration
  # Generated automatically - do not edit manually
```

Two things it does that changed behaviour here, neither of them announced at install time.

**1 · It routes every NetBird address through its own proxy.** The file matches the peer's
tunnel IP and all of its hostnames, then runs `netbird ssh proxy %h %p`. When that proxy
hangs, `ssh` reports:

```
Connection timed out during banner exchange
```

**which names the remote sshd — and the remote sshd is healthy.** A raw TCP read of port 22
returns a banner in ~30 ms because it bypasses the proxy entirely, so the two pieces of
evidence appear to contradict each other. Diagnosing this cost most of a session.

**2 · It prefers passwords over keys.** The generated block sets
`PreferredAuthentications password,publickey,keyboard-interactive`. A host with a loaded agent
still gets a password prompt, which reads like a missing key rather than an ordering choice.

**The fix belongs in our config, not theirs.** `~/.ssh/config` pins `ProxyCommand none` for
substrate hosts, with the reasoning inline so it is not removed later as redundant. Editing
the generated file would be undone on upgrade; NetBird offers
`netbird service reconfigure --service-env NB_DISABLE_SSH_CONFIG=true` to stop generating it
at all, which is the heavier option and is not taken — the file is useful for interactive
sessions and only wrong for automation.

**The general lesson, which outlives NetBird:** a tool that manages system SSH configuration
can break automation's transport with no change on our side, and its errors will name
something else. ADR-0058 is the decision this produced.

### How it was resolved, later the same day

`ProxyCommand none` fixed the hang, and immediately exposed a second problem: what answered
on the far side was NetBird's own SSH server, which is **JWT-mandatory and offers no key
path**. Negotiating `PreferredAuthentications=none` returns `password` and never `publickey`,
and that "password" is the front of an SSO flow — it does not check a Unix password at all.

**A correct password was rejected three times before that was diagnosed**, and the check that
would have settled it takes one command. Recorded because the failure is entirely
non-obvious from the prompt, and the next person will read it as a wrong password too.

The resolution is ADR-0058 as revised: **NetBird's SSH server is disabled and OpenSSH owns
the tunnel.** NetBird remains the network — what was switched off is an optional feature
bolted onto the tunnel, not the tunnel itself.

**On this workstation that means `~/.ssh/config` keeps `ProxyCommand none` permanently**, and
the NetBird-generated file becomes irrelevant rather than fought. Removing the pin later,
believing it redundant, would route SSH back through a proxy for a service that is no longer
running.

### Also on this workstation, 2026-08-23

| Change | Why | Reproducible? |
| --- | --- | --- |
| `sops` 3.13.3, `age` 1.3.1 installed | ADR-0057 | ❌ by hand |
| age key at `~/.config/sops/age/keys.txt`, backed up in the password manager | ADR-0057; RUN-0004 is the procedure | ⚠️ manual by design |
| macOS symlink `~/Library/Application Support/sops/age/keys.txt` | sops does not use the XDG path on macOS; without it every decrypt fails with `CouldNotRetrieveKey` and names no path | ❌ by hand |
| `~/.ssh/config`: `ProxyCommand none` for sitar | the addendum above | ❌ by hand |
| `~/.npmrc`: GitHub Packages credential | publishing `@paridyum/ui` | ❌ by hand |
| OpenTofu installed, Terraform removed | ADR-0052 | ❌ by hand |

Every ❌ is a reason the workstation play in `infra-substrate` is owed. The list is longer than
it was this morning, which is the argument making itself.

## Addendum, 2026-08-31 — why the tunnel drops, and it is not flakiness

The workstation's NetBird connection failed repeatedly across a working day, each time appearing
as `ssh: Operation timed out` to `sitar-as-in-01-ar` and each time "fixed" by `netbird up`. It is
three faults stacked, and `netbird up` addresses none of them — it re-authenticates a session
that expires again.

**1. The daemon reports `LoginFailed`, which is SSO session expiry.** This workstation is
SSO-registered, and NetBird expires SSO peer sessions; the daemon cannot renew one by itself and
requires an interactive browser login. **Peers registered with a setup key are exempt.** That is
the entire reason `sitar-as-in-01-ar` has been stable all day and this machine has not — sitar
was registered with a setup key and this was not.

**2. Every SSO re-login mints a NEW peer, and the address changes with it.** Observed directly:
the peer count moved from `0/4` to `1/5` in the instant `netbird up` completed. This workstation
now exists in the tenant three times, at three addresses:

| Peer | Address | Status when observed |
| --- | --- | --- |
| `sitar-as-in-01-ar` | `100.64.78.110` | Connected — the substrate host |
| `sitar-asia-india-1w-162-127` | `100.64.162.127` | Idle — this workstation, oldest |
| `unknown-3e-af-60-bf-84-51` | `100.64.105.163` | Connecting — this workstation |
| *this workstation, current* | `100.64.195.71` | — |
| `sitar-1-router` | `100.64.26.105` | Connecting — stale |
| `sitar-1-router-98-249` | `100.64.98.249` | Connecting — stale duplicate |

**Anything pinned to this workstation's tunnel address breaks on every re-login.** Nothing is,
today, and only because RFC-0035 D4's ruleset admits `iifname wt0` rather than an address — a
decision made for a different reason that happens to survive this one.

**The two `sitar-1-router` peers matter more than they look.** They are the routing peers that
would advertise `10.1.1.0/24` (RFC-0037), and both read `Connecting`. The estate network will not
be reachable from a workstation until one of them is actually up, so this must be resolved before
the guest is reachable from anywhere but sitar itself.

**3. The workstation has no usable hostname.**

```
hostname             → Unknown_3e:af:60:bf:84:51
scutil --get HostName → (unset)
scutil --get LocalHostName → Rohans-MacBook-Air-4796
```

`hostname` returns a MAC-address placeholder because `HostName` is unset, which is why NetBird
names the peer `unknown-…` and why the duplicates above are indistinguishable in the console.

**It could not be fixed correctly when this was written, and that gap is now closed.** RFC-0021's
grammar is `family-continent-country-sequence[-platform]`, and `device-registry.yaml` declared
three families — `sitar` (server), `dhun` (router), `murli` (mobile) — and six platforms, none of
which was macOS. An operator workstation had no family and macOS had no platform code, so **no
conforming name existed to set**: the machine that administers the estate was the one machine the
estate could not name. That was RFC-0021 **open question 1**.

Resolved the same day, for the operator half only: `device-registry.yaml` admits the **`tabla`**
family — operator workstations, `geography: false` for `murli`'s reason (a laptop crosses borders
exactly as a phone does), `platform: optional` because ADR-0006 makes macOS, Linux and WSL2 all
first-class. Platform code **`mc`** is added for macOS. The purely personal device — one that
never reaches production — is still open and is not answered by extension.

**This workstation is therefore `tabla-01-mc`.** Set it BEFORE re-registering with a setup key,
or the new peer inherits the `unknown-…` name and the whole exercise repeats:

```sh
sudo scutil --set HostName tabla-01-mc
sudo scutil --set LocalHostName tabla-01-mc    # the Bonjour/mDNS name
sudo scutil --set ComputerName tabla-01-mc     # the name in Sharing preferences
```

`HostName` is the one that matters here — it is unset today, which is why `hostname` falls back
to a MAC-address placeholder. The other two are set for consistency so the machine does not
answer to three different names.

### What actually fixes it

| | Where | |
| --- | --- | --- |
| Disable login expiration for this peer | NetBird console | stops the `LoginFailed` cycle |
| Re-register with a **reusable setup key** | workstation + console | the durable fix; setup-key peers do not expire |
| Delete the four stale peers | NetBird console | removes the ambiguity |
| Resolve RFC-0021 OQ1, then set `HostName` | `device-registry.yaml` | so the new registration is not `unknown-…` |
| Upgrade `0.67.4` → `0.77.1` | workstation | ten minor versions of reconnection fixes |

**Order matters, and the upgrade goes last.** Upgrading restarts the daemon and forces a fresh
login, which under fault 2 mints yet another peer. Doing it before the setup key is in place
makes the problem it is meant to help with measurably worse.

## Addendum — 2026-09-16: the default `kubectl` context is not the estate

`kubectl` on this workstation defaults to a **dead local `kind-pd-dev` context** whose API server
refuses connections. The estate's cluster is elsewhere entirely, and this was written down
nowhere `RUN-0001` reaches. A planning document read the default context on 2026-09-16, saw
`connection refused`, and concluded *"nothing in the estate is deployed"* — a conclusion about the
whole estate drawn from a context that has never been it.

**The estate is a Talos cluster reached over NetBird.** `go-live.sh` in the `knowledge` checkout
says so in a comment, which is currently the only place it is recorded:

```sh
export KUBECONFIG=~/Developer/paridyum/infra/infra-terraform/kubeconfig
kubectl config current-context     # expect: the staging cluster, NOT kind-pd-dev
kubectl get nodes                  # expect: five staging-* nodes
netbird status                     # must be Connected, or the cluster is unreachable
```

**Three things fail together and only the third is loud.** The NetBird peer must be up; the
kubeconfig must be the file above and not the ambient context; and `kind-pd-dev` must not be what
answers — it fails to connect rather than failing to exist, so it reads as *"the cluster is down"*
rather than *"you are asking the wrong cluster."*

| Symptom | Actual cause | |
| --- | --- | --- |
| `dial tcp 127.0.0.1:5xxxx: connect: connection refused` | `kind-pd-dev` is the current context | `kind` cluster stopped; the estate is elsewhere |
| Cluster silent, NetBird `Management: Connected` | kubeconfig not set | `go-live.sh` sets it, an ad-hoc `kubectl` does not |
| `netbird status` shows `Profile: default` | wrong profile | T-2b-era notes switch to `Production-Devices` |

**The lesson, which is the reason this is an addendum and not a note:** `kubectl` answers whatever
context it is pointed at, and it answers *confidently*. A tool that cannot tell you it is looking
at the wrong thing will be believed. Before drawing any conclusion about the estate from
`kubectl`, print `kubectl config current-context` — and treat a refused connection as a question
about which cluster you asked, before a fact about whether anything is running.

**Verified 2026-09-16** with the commands above: five `staging-*` nodes `Ready` for 14 days, and
`helm list -A` returning nine `deployed` releases across `paridyum-web`, `platform`, `zitadel`,
`longhorn-system`, `cnpg-system` and `kube-system`.
