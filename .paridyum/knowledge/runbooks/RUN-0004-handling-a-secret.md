---
id: RUN-0004
title: Handling a secret — one key in a password manager, everything else encrypted in place
type: runbook
status: accepted
created: 2026-08-23
locked_at: null
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [secrets, sops, age, bitwarden, adr-0057, req-0001, onboarding]
breaking: false
enforced_by: []
---

# RUN-0004 — Handling a secret

The answer to *"where do I put this password?"*, asked by someone who has one in their hand
and does not want to think about cryptography. ADR-0057 is the decision; this is the
procedure.

## The whole model in four lines

1. **One** age private key exists. It lives in the operator's password manager.
2. That key is on each machine at `~/.config/sops/age/keys.txt`, mode `600`.
3. Every other secret is encrypted **in the repository that uses it** and committed.
4. Nothing else is memorised, and no second mechanism exists.

**There is exactly one thing to lose and exactly one thing to back up.** That is the property
this design exists for. Adding a repository, a machine or a secret does not add a thing to
remember.

## The three kinds of secret, and where each goes

The mistake this section prevents is encrypting something into git that never belonged there.

| Kind | Example | Where |
| --- | --- | --- |
| **Automation** — something a playbook or a build reads unattended | a become password, a database password | **SOPS-encrypted in the consuming repository** |
| **Operator** — something a person uses through a tool | a GitHub PAT, a NetBird setup key, a recovery code | **The password manager.** Not git |
| **CI** — something a pipeline reads and a person never does | a package-read token | **GitHub Actions secrets** |

**An operator credential does not go in git even encrypted**, and the reason is not caution.
A committed file is permanent: every historical version stays readable to anyone who later
obtains the key, so a credential that could simply have lived in a password manager has been
given an immortal copy for no gain.

## Naming an operator credential in the password manager

Added 2026-08-31, because the table above says *where* an operator credential goes and said
nothing about *how it is named* — and an item nobody can find is an item that gets created a
second time.

**Nothing in `constitution/C-0007` binds this.** That artifact names repositories, crates, APIs,
databases, events, environment variables, images and Helm releases; a password-manager entry is
none of them, and inventing a rule inside a locked constitution to cover one would be the
opposite of what C-0005 did. So the convention lives here, in the runbook that gets read at the
moment it is needed, and it is a convention rather than a rule — nothing enforces it.

**The item name is `<what> — <where>`.** The thing the credential opens, then the identity it
opens it on, separated by an em dash:

```
sunshine — sitar-as-in-01-ar
```

`<where>` is the **RFC-0021 hostname**, not "my laptop" and not an IP address. That is the
whole point of the convention: the machine already has one canonical name, it appears in the
inventory, in `~/.ssh/config` and in every log line, and reusing it means a search for the
hostname finds every credential that machine holds. An address would go stale — the tunnel
address on this estate has already changed once.

Where a credential is not tied to a machine, `<where>` is the org-level thing it belongs to —
`age (SOPS) — paridyum`, `github pat — paridyum`.

**Fields, and the reason each one is there:**

| Field | Value | Why |
| --- | --- | --- |
| Name | `<what> — <where>` | above |
| Username | whatever the service was told | the half people forget, and the half that cannot be reset by guessing |
| Password | the credential | — |
| URI | the address it is typed into | **use `https://localhost:47990` only if the service is genuinely loopback-only.** Autofill matches on host and port, so a `localhost` URI offers this item to *every* page on that port |
| Folder | the estate, then the role — e.g. `Paridyum/substrate` | so the set is enumerable without remembering names |
| Custom field `governed_by` | the artifact id — e.g. `RFC-0036 D4` | the link back. An entry that cannot name why it exists is an entry nobody will dare delete |
| Note | how to rotate it, in one line | the question actually asked in an incident is *"can I change this without breaking something?"* |

**What this is not for.** The age key, and only the age key, is the thing whose loss is
unrecoverable (above). Everything named by this section is re-creatable by resetting it at the
service, and the note field is where you say how.

## Adding a secret to a repository

```sh
make secret            # opens $EDITOR on the DECRYPTED file; re-encrypts on save
make secret-show       # read-only
```

Add the key, save, quit, commit the encrypted file. That is the entire procedure and it does
not change per secret or per repository.

**Never pass a secret on a command line.** `sops set … '<value>'` works and puts the value in
shell history, which is a plaintext file on disk — usually the same disk the encryption was
protecting against.

## Enabling a repository that has none yet

Three files, copied from `infra-substrate`:

1. **`.sops.yaml`** — the `creation_rules` block, unchanged. It names the **public** key, which
   is meant to be committed and can only encrypt.
2. **`requirements.yml`** — add `community.sops` if the repository runs Ansible.
3. **`ansible.cfg`** — `vars_plugins_enabled = host_group_vars,community.sops.sops`.

Then `mkdir -p group_vars/<group>` and `make secret`.

**`encrypted_regex` is not decoration.** It encrypts values and leaves keys visible, which is
what keeps a secrets change reviewable as a diff — STD-0016 rule 2's premise. Without it every
change renders as one opaque blob and review silently stops working.

## Setting up a new machine

1. Retrieve the age key from the password manager.
2. `mkdir -p ~/.config/sops/age && chmod 700 ~/.config/sops ~/.config/sops/age`
3. Paste it into `~/.config/sops/age/keys.txt`, then `chmod 600` it.
4. **On macOS only**, symlink the location sops actually checks:
   ```sh
   mkdir -p "$HOME/Library/Application Support/sops/age"
   ln -sf "$HOME/.config/sops/age/keys.txt" "$HOME/Library/Application Support/sops/age/keys.txt"
   ```
   Without this every decrypt fails with `CouldNotRetrieveKey … exited with code 128`, an
   error that names no path and reads like a corrupt file.

## Adding a second operator

One line in each `.sops.yaml` naming their public key, then:

```sh
sops updatekeys group_vars/*/secrets.sops.yaml
```

No secret is re-typed and no secret is decrypted to disk. This cheapness is why ADR-0057 chose
recipient-based encryption over a shared passphrase — a passphrase cannot be granted to one
person without being changed for everyone.

## An agent's terminal output is not a safe place for a decrypted value

Found 2026-09-24. An agent decrypted `garage-access.sops.yaml` to inspect its structure and tried
to redact the values with `sed 's/=.*/=<redacted>/'` before printing — a pattern that matches
`KEY=value` shell-style assignment and **silently does nothing** to a YAML file, where the
separator is `:`, not `=`. Both Garage access-key pairs (`docket_*`, `cnpg_*`) printed in full into
the session's output, which is captured in the model's context and the session transcript exactly
like a terminal scrollback or a CI log — the same class of exposure CLAUDE.md already names for a
Talos bootstrap token reaching a log via an `grep -A2` that ran past its intended field.

**The fix is the same shape as everywhere else in this document: verify the redaction against the
file's actual separator before trusting it, or better, never print a decrypted value at all.**
Concretely, when an agent (or a person) needs to confirm a sops file's *structure* without seeing
its *values*:

```sh
sops --decrypt secrets/whatever.sops.yaml | grep -oE '^[a-zA-Z_]+:'   # keys only, values never leave sops
```

And when a decrypted value must be *used* (to grant a permission, populate a live Secret), redirect
it straight into a file or a variable inside one shell invocation and never let it pass through a
command whose output is displayed — `awk -F': ' '/^Key ID:/{print $2}' file > /dev/null`-shaped
plumbing, not `cat file | grep Key`.

**What this incident's rotation looked like**, because RUN-0004 says a leaked secret is rotated,
not re-encrypted, and this is the first time that rule fired for a Garage-issued key rather than
the age key itself: two new Garage keys were created (`garage key create <name>`), granted the same
bucket permissions the old ones held, the live Kubernetes Secrets consuming them (`postgres-backup-
creds`, `docket-s3`) were updated and the workloads restarted/verified against the new credential
(an on-demand CNPG `Backup` completed; `docket`'s deployment rolled out clean), **then** the old
keys had their bucket permissions stripped (`garage bucket deny --read --write <bucket> --key
<id>`) rather than being deleted outright — a same-effect, lower-blast-radius action that the
session's own permission gate allowed where an outright `key delete` did not. `garage-
access.sops.yaml` was rewritten with the new pair and re-encrypted. The old keys still exist in
Garage with zero authorized buckets; deleting them outright is a small cleanup owed to whoever next
touches `storage/garage`.

## Rotation, and the thing people get wrong

**Rotating the key is not rotating the secret.** Every historical version of an encrypted file
stays in git history. If the age key is ever exposed, re-encrypting the current files protects
nothing that was committed before — the old ciphertext is still there and now openable.

So a key compromise means **rotating every secret it ever protected**: change the password on
the host, issue a new token, and then re-encrypt. Treat the key as opening everything it has
ever opened, not everything it currently opens.

## If the key is lost

Every secret it opens is unrecoverable, and this is the accepted trade in ADR-0057: a lost
secret is re-created, a leaked one is not un-leaked. Recovery is re-creation — set a new
password on the host, generate a new key, re-encrypt.

That is survivable only because the secrets are few and re-creatable. It stops being
survivable the moment something irreplaceable is encrypted with it, which is a reason to keep
customer data and long-lived signing material out of this mechanism entirely.

## Related

- **ADR-0057** — the decision, and why SOPS with age rather than ansible-vault or a hosted manager
- **ADR-0056** — Ansible, the first consumer
- **REQ-0001** — runtime secrets for deployed workloads, still deferred and NOT solved by this
- **ADR-0045** — risk R3, narrowed by this and not closed
- **C-0004** — `repo.secret_scan_clean`, which is what actually catches a plaintext credential
- **STD-0016** rule 5 — no credential is stored in an infrastructure repository unencrypted
- **RUN-0001** — workstation provisioning, which this is a companion to
