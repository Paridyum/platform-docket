---
id: ADR-0131
title: A second CI runner host is a Windows 11 Pro mini PC, configured only by Ansible
type: adr
status: proposed
created: 2026-10-03
locked_at: null
supersedes: null
superseded_by: null
scope: [infra]
applies_to: ["infra-terraform", "infra-ci"]
tags: [ci, runner, github-actions, self-hosted, hyper-v, wsl2, windows, adr-0124, throughput]
breaking: false
enforced_by: []
sources:
  - url: https://docs.github.com/en/actions/reference/runners/self-hosted-runners
    title: Requirements for self-hosted runner machines — GitHub Docs
    retrieved: 2026-10-03
    supports: >
      Windows 10/11 are supported runner hosts, but "If you want to run workflows that use Docker
      container actions or service containers, you must use a Linux machine and Docker must be
      installed." Our jobs are labelled linux and start Postgres in Docker, so the runner must be Linux.
  - url: https://learn.microsoft.com/en-us/windows/wsl/wsl-config
    title: Advanced settings configuration in WSL — Microsoft Learn
    retrieved: 2026-10-03
    supports: >
      WSL 2 supports systemd (/etc/wsl.conf [boot] systemd=true), but by default stops an idle distro
      after 15 s (instanceIdleTimeout) and the VM after 60 s (vmIdleTimeout), and takes 50% of memory
      and all processors unless .wslconfig says otherwise.
  - url: https://github.com/microsoft/WSL/discussions/9245
    title: Keep WSL2 instance with systemd alive — microsoft/WSL discussion
    retrieved: 2026-10-03
    supports: Keeping WSL running for background services needs both timeouts disabled and a start trigger.
  - url: https://www.jordansiaw.com/blog/challenges-in-making-wsl-2-run-services-at-windows-startup/
    title: Challenges in making WSL 2 run services at Windows startup
    retrieved: 2026-10-03
    supports: WSL runs per user; starting it at boot without a logged-in user needs a scheduled task or a service wrapper.
  - url: https://docs.ansible.com/ansible/latest/os_guide/windows_ssh.html
    title: Windows SSH — Ansible documentation
    retrieved: 2026-10-03
    supports: >
      SSH to Windows is officially supported from ansible-core 2.18, needs OpenSSH 7.9+,
      ansible_connection=ssh, ansible_shell_type matching the host's DefaultShell (cmd, OpenSSH's
      default, or powershell, which it recommends as better tested and faster), and an administrator's
      key in C:\ProgramData\ssh\administrators_authorized_keys. Re-read 2026-10-03: both are supported.
  - url: https://github.com/ansible-collections/community.general/blob/main/plugins/connection/wsl.py
    title: community.general.wsl connection plugin (in community.general 11.4.9, pinned here)
    retrieved: 2026-10-03
    supports: >
      Runs tasks inside a WSL distribution through the Windows host's SSH and wsl.exe, so the
      distribution needs no sshd. It quotes its command for CreateProcess (subprocess.list2cmdline),
      which survives cmd.exe as the SSH shell and does not survive PowerShell (verified on the host:
      PowerShell strips the backticks Ansible's temp-directory command uses).
  - url: https://search.opentofu.org/provider/taliesins/hyperv/latest
    title: taliesins/hyperv — OpenTofu Registry
    retrieved: 2026-10-03
    supports: >
      The one Hyper-V provider talks WinRM to the host. Its latest release is v1.2.1 (2024-02-11); the
      OpenTofu registry listed versions up to 1.0.2 when read.
  - url: https://zenn.dev/tenpa/articles/wsl-self-hosted-runner-cicd?locale=en
    title: Run GitHub Actions locally with a self-hosted runner on WSL2
    retrieved: 2026-10-03
    supports: A Linux x64 runner registered inside WSL2 and run as a systemd service works in practice.
  - url: https://docs.ansible.com/ansible/latest/os_guide/windows_ssh.html
    title: Windows SSH — Ansible documentation (source docs/docsite/rst/os_guide/windows_ssh.rst)
    retrieved: 2026-10-04
    supports: >
      "Ansible officially only supports the OpenSSH implementation shipped with Windows, not the upstream
      package. The OpenSSH version must be version 7.9.0.0 at a minimum." Addendum 2026-10-04, item 6.
  - url: https://learn.microsoft.com/en-us/windows-server/administration/openssh/openssh_install_firstuse
    title: Get started with OpenSSH Server for Windows — Microsoft Learn
    retrieved: 2026-10-04
    supports: >
      The in-box Feature on Demand: "Microsoft supports and services this version through Windows Update, so
      it's the recommended option for most users." Installing it "creates and enables a firewall rule named
      OpenSSH-Server-In-TCP". Item 6, and why item 7 does not depend on that rule.
  - url: https://github.com/PowerShell/Win32-OpenSSH/releases/tag/10.0.0.0p2-Preview
    title: Win32-OpenSSH 10.0.0.0p2-Preview release notes
    retrieved: 2026-10-04
    supports: >
      The MSI the bootstrap installed opens "This is a preview-release (non-production ready)". Item 6.
  - url: https://github.com/PowerShell/Win32-OpenSSH/issues/2299
    title: Win32-OpenSSH issue 2299 — service restart loop after a Windows update
    retrieved: 2026-10-04
    supports: >
      A maintainer: "if you have multiple versions of OpenSSH on the machine simultaneously, there may be
      conflicting libcrypto.dll files depending on the paths/order in the PATH environment variable". This
      host had both on the machine PATH, the MSI's first. Item 6.
  - url: https://learn.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/network-list-manager-policies
    title: Network List Manager policies — Microsoft Learn
    retrieved: 2026-10-04
    supports: >
      Network location type sets the firewall profile; policy can force a location type for unidentified
      or identifying networks. Considered and rejected for item 7.
---

# ADR-0131 — A second CI runner host is a Windows 11 Pro mini PC, configured only by Ansible

## Status

**Proposed** 2026-10-03, for the operator to accept. The operator has a fresh Windows 11 Pro mini PC
and asked whether it can add CI capacity. This records what was measured and researched, and the
recommendation. Until it is accepted, nothing is installed beyond the access bootstrap below.

## Context

**CI here is a queue, not a compute shortage (measured 2026-10-03).**

- **One host.** All CI runs on `ci-runner-01` (ADR-0124): one Proxmox guest with 6 vCPUs and 10 GB,
  running four runner processes.
- **Measured on platform-tenancy PR run 37074304054.** Its jobs did about 25 minutes of work in total,
  after about 5 hours queued each. The image build on main waited 2.6 hours and then ran for 17
  minutes.
- **Demand.** The estate ran 183 runs in 24 hours, at about 12 jobs per Rust run.
- **Compile time is being fixed separately.** A compile cache per runner process landed the same day
  (infra-terraform#13, infra-ci#13). It shortens each job, but every job still waits for one of four
  slots.

**The host's constraint is Linux.** GitHub supports Windows 10 and 11 as runner hosts, but "if you want
to run workflows that use Docker container actions or service containers, you must use a Linux
machine". Every estate workflow asks for `[self-hosted, linux, x64, paridyum]` (ADR-0124 D2) and starts
Postgres in Docker. So the runner has to live in Linux on the mini PC, and there are two ways to keep
Windows underneath:

- **WSL 2.** Supports systemd and Docker; a Linux runner registered there works. But WSL stops an idle
  distro after 15 s and its VM after 60 s, runs per user, and takes half the memory and every
  processor by default. Running 24/7 needs both timeouts disabled, limits set, and a boot-time task
  that starts it with no one logged in.
- **A Hyper-V guest.** Windows 11 Pro includes Hyper-V. A Debian 13 guest boots with the host through
  its automatic start action, gets fixed cores and memory, and is the same OS `ci-runner-01` runs. So
  infra-terraform's `ci-runner/install.sh` and `ci-runner/sccache.sh` apply unchanged.

## Decision (proposed)

**Nothing on this host is hand-built (ADR-0056).** OpenTofu answers what machines exist and Ansible
what is true inside one. A script that configures a machine is rejected there by name. So:

1. **The only hand step is the door.** `infra-substrate/scripts/bootstrap-windows-host.ps1`, run once
   on the host as Administrator, sets up OpenSSH Server, the operator's key only, key-only logins,
   cmd as the SSH default shell and, optionally, NetBird. That is exactly what Ansible needs to
   manage Windows over SSH (ansible-core 2.18+). It is the Windows counterpart of the SOP's
   `netbird up --allow-server-ssh`.
2. **Everything after the door is infra-substrate roles, run against an inventory host.** Check mode
   first, then apply (STD-0016):
   - power that never sleeps;
   - Windows Update active hours, so a reboot does not cut a job short;
   - NetBird's settings;
   - the Linux environment the runner lives in;
   - the runner.
3. **The Linux environment: WSL 2, owned by Ansible (recommended), or a Hyper-V guest, owned by
   OpenTofu.** Governance decides the shape as much as reliability does:
   - **WSL 2** is a feature and a distribution *of the Windows host*, so it is host configuration, and
     Ansible owns all of it. That covers `.wslconfig` (`instanceIdleTimeout=-1`, `vmIdleTimeout=-1`,
     the memory and processor caps), systemd in `/etc/wsl.conf`, the start-at-boot task, and the
     distribution's packages and runner. **One condition:** the first apply must show the runners
     coming online after a reboot with no one logged in, without a stored password (ADR-0056: no
     privileged credential is stored). If that cannot be met, item 3 falls back to the guest.
     - **How the condition is met (operator decision, 2026-10-03).** WSL 2 starts a distribution
       only in a user context, and a *batch* logon (a scheduled task "run whether logged on or
       not") with a non-blank password would store that password as an LSA secret — a stored
       credential ADR-0056 does not want on the host. So instead the runner account's password is
       **blanked** and the host **autologins** it; a launcher in that session starts WSL at boot.
       Nothing is stored — the blank password is the whole point — so ADR-0056's "no privileged
       credential" holds. The condition's "no one logged in" is traded for an always-signed-in
       console, whose cost is accepted below. SSH stays key-only (the bootstrap set
       `PasswordAuthentication no`), so the blank password opens no network path, and Windows
       blocks blank-password network logons by default (`LimitBlankPasswordUse`). The autologin,
       the blanking and the launcher are all Ansible, not hand steps.
     - **How Ansible reaches inside the distribution (2026-10-03).** Through the Windows host's own
       SSH and `wsl.exe` (`community.general.wsl`), not an sshd in the distribution: no second
       listener on the LAN, no mirrored networking, no port rule. The plugin's command survives
       `cmd.exe` and not PowerShell, so the SSH default shell is **cmd** (OpenSSH's default) and the
       Windows host's inventory says `ansible_shell_type: cmd`. Ansible recommends PowerShell as
       better tested and faster; the cost lands on two light Windows roles, and is smaller than a
       second listener inside a machine that runs pull-request code.
   - **A Hyper-V guest** is a machine, so under ADR-0056 its existence belongs to OpenTofu. The one
     provider (`taliesins/hyperv`) talks WinRM. That means a second management listener on the host,
     with a credential prompted at run time. Its last release was 2024-02-11, and the OpenTofu
     registry lagged at 1.0.2 when read. It boots with the host by design, and the guest's inside is
     still Ansible's.
4. **The runner is one Ansible role for every runner host.** `ci-runner-01` included. The role takes
   the place of `infra-terraform/ci-runner/install.sh` and `sccache.sh`, which are scripts configuring
   a machine's inside, kept in the OpenTofu repository. Existing code contradicting ADR-0056 is the bug
   in the code, and the role is the fix. The runners keep ADR-0124's labels and D3 to D6: images only
   from CI, no estate credential, persistent with cleanup, and no fork pull requests. Names are
   `ci-runner-02`, `-02-2` and so on, one sccache server per process; infra-ci maps the new names to
   their ports.
5. **Addresses.** The host needs outbound internet only. If it takes an estate address, the address is
   allocated in `address-registry.yaml` first (RFC-0037).

## Consequences

**Accepted costs.**

- **A runner host outside the Proxmox estate.** With WSL 2 it is one Windows machine whose whole inside
  is Ansible's; with a Hyper-V guest, OpenTofu reaches it over WinRM through a stale provider.
- **Windows Update reboots the host,** stopping the guest's jobs mid-run unless the reboot falls
  outside active hours. A job cut short fails and is re-run.
- **A second machine runs pull-request code** from the organization's repositories. Fork pull requests
  stay off (ADR-0124 D6), and the runner holds no estate credential (D4). The Windows host holds the
  operator's admin access only.
- **The runner host autologins a passwordless account** (the reboot-survival decision above). A person
  at the physical console gets a signed-in desktop. Accepted for a dedicated, single-purpose CI mini PC
  on the operator's LAN: it holds no estate credential (D4), SSH is key-only, and nothing a reboot needs
  is stored. On a machine that was shared or off-LAN this would not be acceptable.

**Gained.**

- **More runner slots on separate hardware.** Queue time falls in proportion. Together with the
  compile cache, a Rust PR should wait minutes, not hours.
- **The runner install stops being a script.** One `ci_runner` role covers both hosts, so
  `ci-runner-01` gets reconciled configuration with a diff as well.

**Rejected alternatives.**

- **A Windows runner on the host itself.** Docker container actions and service containers need a
  Linux machine, and every estate job is labelled `linux`.
- **Configuring the host by hand, or by running scripts over SSH.** ADR-0056 rejects both: no dry
  run, no diff, no idempotence.
- **Wiping Windows for bare-metal Debian.** This gives the most capacity, but the operator keeps
  Windows on this machine.
- **A bigger `ci-runner-01`.** Declined by the operator on 2026-10-03: sitar also hosts the staging
  cluster.

## Bootstrap — the operator's one hands-on step

`infra-substrate/scripts/bootstrap-windows-host.ps1`, documented in infra-substrate's SOP under "A
Windows host". Run it once, in PowerShell as Administrator:

```powershell
Set-ExecutionPolicy -Scope Process Bypass -Force
.\bootstrap-windows-host.ps1 -JoinNetBird
```

It prints the host name, addresses and hardware the inventory needs. Everything after it is Ansible.

## Addendum — 2026-10-04: the door must survive Windows Update and a new network

The host stopped accepting SSH early on 2026-10-04 and stayed unreachable until the evening, so its
runners were taken out of rotation. The host's own logs, read once it was reachable again, give this
sequence. Each line is one log entry: DISM, Windows Firewall, Service Control Manager, CBS, NetworkProfile
and the sshd operational log.

| When (IST) | What the host recorded |
| --- | --- |
| 10-03 15:59, 16:04 | DISM: `Add-WindowsCapability OpenSSH.Server~~~~0.0.1.0`. This was the first, slow attempt, before the bootstrap switched to the MSI. It did not fail: it queued behind Windows Update. |
| 10-03 16:05:33 | MSI `OpenSSH 10.0.0.0` installed. Service `sshd` → `C:\Program Files\OpenSSH\sshd.exe`. |
| 10-03 16:05:45 | The bootstrap creates rule `OpenSSH-Server-In-TCP` ("OpenSSH Server (sshd)"), profiles **Domain, Private, Public**. |
| 10-03 onward | The LAN is profile "Airtel_sraj_6835 2", category **Public**: the " 2" is a second profile Windows made for a known network, and new profiles start Public. SSH works because the rule covers Public. |
| 10-04 03:12 | The rename restart (windows_baseline). Last accepted SSH login: 03:13:58. |
| **10-04 03:32:02** | Firewall events 2097/2099: rule `OpenSSH-Server-In-TCP` replaced by the inbox capability's definition ("OpenSSH **SSH** Server (sshd)"), profiles **Private only**. With the network Public, every connection is dropped from here on. sshd keeps running until 18:30. |
| **10-04 18:30** | Windows Update restarts the host itself (`MoUsoCoreWorker.exe`, KB5129195, build 26200.9457). CBS: `OpenSSH-Server-Package …9457` **Installed**. sshd terminates; afterwards **no service named `sshd` exists**, though the MSI is still registered as installed. |
| 10-04 19:44–19:49 | Recovery at the console: network marked Private, MSI uninstalled and installed. `sshd` registered again, first login 19:49:16. |

**One cause, two blows: two OpenSSH servers on one machine.** The inbox server that the slow first attempt
had queued finished installing on top of the MSI. It is the server Microsoft and Ansible support (item 6);
the fault was that both were present:

1. **It rewrote the door's firewall rule.** The inbox server ships a rule under the same name,
   `OpenSSH-Server-In-TCP`, limited to the Private profile. Servicing replaced the bootstrap's
   all-profile rule with it. On a network Windows classes Public, that closes the door while sshd is
   still running.
2. **It took the service name.** The inbox server and the MSI both register a service called `sshd`.
   When servicing completed the capability, the MSI's registration was gone and nothing replaced it.
   A forced reinstall of the same MSI version did not restore it (Windows Installer ran a
   *reconfigure*, which skips service registration). Only an uninstall followed by an install did.
   Exactly how CBS dropped the registration is inferred from this sequence. No log line states it.

Decided:

- **6. The inbox OpenSSH server is the only one, and the Win32-OpenSSH MSI is removed.** This reverses the
  bootstrap's choice of 2026-10-03, which took the MSI because the inbox install was slow. Ansible
  "officially only supports the OpenSSH implementation shipped with Windows, not the upstream package".
  Microsoft services the inbox server through Windows Update and recommends it. The MSI the bootstrap
  installed calls itself "a preview-release (non-production ready)", which C-0010 does not allow on a
  production runner. Two servers on one machine also share the service name `sshd`, the firewall rule
  name, and `PATH`, and that is where this outage came from. The inbox server here is OpenSSH 9.5p2
  (`C:\Windows\System32\OpenSSH`), above Ansible's 7.9 minimum, and its `sshd -t` accepts the existing
  `C:\ProgramData\ssh\sshd_config`. Host keys, the operator key and the config are in that shared folder,
  so the host's identity does not change.
  - **The bootstrap** removes the MSI if present and makes sure the inbox capability is installed with
    its service registered. On a fresh machine the inbox install waits behind Windows Update, so the SOP
    has Windows Update run to completion first. That waiting is the delay that pushed the bootstrap to
    the MSI, and it is now a documented step instead of a workaround.
  - **Ansible asserts and fails closed:** the capability is `Installed`, `sshd` runs
    `C:\Windows\System32\OpenSSH\sshd.exe`, and no MSI binary or `PATH` entry remains. It does not
    install or remove either server: both can delete the service its own connection runs through, so
    that change belongs to the operator at the console.
- **7. The door has its own firewall rule, scoped by address, on every profile.** The rule is
  `Paridyum-SSH-In`: TCP 22 from `LocalSubnet` and the NetBird range `100.64.0.0/16` (the address
  registry's NetBird entry), on the Domain, Private and Public profiles. It is not `OpenSSH-Server-In-TCP`.
  That name belongs to the inbox server's installer, and servicing rewrote it on this host (03:32 above).
  Ansible disables every other inbound OpenSSH allow rule, the vendor's included, once its own rule is
  correct. If servicing re-enables the vendor rule, the exposure widens only on Private networks, and only
  until the next Ansible run; logins stay key-only (item 1).
  - **Rejected:** forcing the network category to Private, by hand or through Network List Manager policy.
    It would mark whatever network the machine joins as trusted for file sharing and discovery as well,
    and on this host Windows created a new Public profile for a known Wi-Fi network without any
    configuration changing.
  - **Rejected too:** allowing all addresses on all profiles.
- **8. One uplink, wired.** The host has two unplugged 2.5 GbE ports and two Wi-Fi adapters on one
  LAN, each with a default route. A CI runner on Wi-Fi was already the leading suspect for the hung image
  builds of 2026-10-04. The host is moved to Ethernet (an operator step: a cable), then the Wi-Fi adapters
  are disabled by Ansible. Until the cable is in, nothing about the adapters changes: disabling the wrong
  one over SSH removes the door.

**Open, not decided here.** Item 2's "Windows Update active hours" was never implemented, and the 18:30
restart fell outside Windows' default active hours. Active hours cover at most 18 hours of the day, and a
runner works all 24, so active hours alone cannot protect a job. Choosing a restart policy is owed; until
then a Windows Update restart ends a job in flight, as this ADR's accepted costs already say.
