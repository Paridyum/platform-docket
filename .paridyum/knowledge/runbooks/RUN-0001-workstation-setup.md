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
│   ├── infra/      clones of infra-*
│   ├── specs/      clones of spec-*
│   └── templates/  clones of template-*
├── personal/       unrelated to Paridyum
├── experiments/    throwaway; never deployed, never depended on
├── archived/       dead projects kept for reference
└── scratch/        deleted without warning
```

The `paridyum/` subdirectories mirror the repository class prefixes from C-0002, so a
repository's name determines where it is cloned with no lookup.

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

| Tool          | Purpose                                   | macOS               | Debian/Ubuntu            | Fedora              | Arch          |
| ------------- | ----------------------------------------- | ------------------- | ------------------------ | ------------------- | ------------- |
| `git`         | —                                         | `brew`              | `apt`                    | `dnf`               | `pacman`      |
| `gh`          | forge's GitHub API client                 | `brew`              | `apt`                    | `dnf`               | `pacman`      |
| rustup/cargo  | primary language (ADR-0005)               | rustup.rs           | rustup.rs                | rustup.rs           | rustup.rs     |
| `docker`      | local dependencies + image builds         | Desktop (cask)      | Docker Engine            | Docker Engine       | `pacman`      |
| `kubectl`     | cluster access (ADR-0003)                 | `kubernetes-cli`    | **k8s apt repo**         | `kubernetes-client` | `kubectl`     |
| `helm`        | chart authoring and linting (ADR-0003)    | `brew`              | `apt`                    | `dnf`               | `pacman`      |
| `terraform`   | service infrastructure modules (ADR-0003) | `brew`              | **HashiCorp repo**       | **HashiCorp repo**  | `pacman`      |
| `jq`          | required by the enforcement hooks         | `brew`              | `apt`                    | `dnf`               | `pacman`      |
| `just`        | task runner in generated repos            | `brew`              | `apt` (universe)         | `dnf`               | `pacman`      |
| `sqlx-cli`    | migrations, offline query metadata        | `cargo install sqlx-cli --no-default-features --features postgres,rustls` |||
| `cargo-chef`  | Docker dependency-layer caching           | `cargo install cargo-chef` |||
| `cargo-deny`  | licence and advisory policy in CI         | `cargo install cargo-deny` |||
| `cargo-audit` | RustSec advisories                        | `cargo install cargo-audit` |||

**Bold** entries are not in default repositories and need a vendor repository added
first. `bootstrap.sh` detects this and prints the specific instructions rather than
failing opaquely.

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
