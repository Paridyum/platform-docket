---
id: ADR-0132
title: CI runners reach the internet dual-stack, because the uplink's IPv4 path to Fastly is unusable
type: adr
status: accepted
created: 2026-10-03
locked_at: null
supersedes: null
superseded_by: null
scope: [infra]
applies_to: ["infra-substrate", "infra-terraform", "infra-ci"]
tags: [ci, runner, network, ipv6, egress, mtu, fastly, adr-0124, adr-0131, throughput]
breaking: false
enforced_by: []
sources:
  - url: https://learn.microsoft.com/en-us/windows/wsl/networking
    title: Accessing network applications with WSL — Microsoft Learn
    retrieved: 2026-10-03
    supports: >
      Mirrored networking (Windows 11 22H2+, networkingMode=mirrored) lists "IPv6 support" among its
      benefits; the default NAT mode gives the distribution no IPv6. Inbound connections from the LAN
      need a Hyper-V firewall rule, so they stay blocked unless one is added.
  - url: https://docs.docker.com/engine/daemon/ipv6/
    title: Use IPv6 networking — Docker Docs
    retrieved: 2026-10-03
    supports: >
      IPv6 on the default bridge is daemon.json "ipv6": true with "fixed-cidr-v6"; ip6tables (IPv6
      filtering and NAT) is enabled by default.
  - url: https://developers.cloudflare.com/cloudflare-one/team-and-resources/devices/warp/troubleshooting/known-limitations/
    title: Known limitations — Cloudflare One client
    retrieved: 2026-10-03
    supports: >
      "Docker on Linux does not perform the underlying network tunnel MTU changes required by the
      Cloudflare One Client" — a WARP tunnel breaks Docker's default bridge until Docker's MTU is
      lowered by hand.
  - url: https://developers.cloudflare.com/cloudflare-one/team-and-resources/devices/warp/configure-warp/warp-modes/
    title: Client modes — Cloudflare One client
    retrieved: 2026-10-03
    supports: >
      "Local proxy mode has a timeout limit of 10 seconds for requests. If a request goes above the 10
      second limit, Cloudflare will drop the connection." Unusable for toolchain downloads and the
      runner's ~50 s long-poll.
  - url: https://www.cloudflare.com/application/terms/
    title: Cloudflare Application Terms of Service (1.1.1.1 / WARP)
    retrieved: 2026-10-03
    supports: >
      Grants "a non-exclusive, personal, revocable, non-transferable license"; no clause bars business
      or server use, and none permits it either.
  - url: https://developers.cloudflare.com/cloudflare-one/tutorials/deploy-client-headless-linux/
    title: Deploy the Cloudflare One Client on headless Linux machines
    retrieved: 2026-10-03
    supports: Headless Zero Trust enrollment uses a service token kept on the machine.
---

# ADR-0132 — CI runners reach the internet dual-stack, because the uplink's IPv4 path to Fastly is unusable

## Status

**Accepted**, 2026-10-03. The operator delegated the choice ("research and choose the best way") after
the measurements below; this records the choice and the evidence it rests on.

## Context

The second runner host (ADR-0131) was registered and then taken out of service the same hour: its
runners flapped offline and its downloads crawled. Measuring why found a problem that the estate's
existing runner, `ci-runner-01`, has too (infra-substrate LOGBOOK, 2026-10-03; curl, 10–25 s samples):

| From | static.rust-lang.org | static.crates.io | GitHub release CDN | codeload.github.com |
| --- | --- | --- | --- | --- |
| mini PC, IPv4 | 25–830 KB/s | 37–740 KB/s | 23–660 KB/s | 1.2–6.7 MB/s |
| **mini PC, IPv6** | **4.0–8.0 MB/s** | **18.7 MB/s** | (no IPv6) | (no IPv6) |
| ci-runner-01 (IPv4 only) | 105 KB/s | 101 KB/s | 108 KB/s | 8.9 MB/s |
| sitar, IPv4 | 55–550 KB/s | — | 104 KB/s | 5.5 MB/s |
| **sitar, IPv6** | **10.7 MB/s** | — | — | — |
| operator's Mac (via Cloudflare WARP) | 7.4 MB/s | 8.9 MB/s | 10.1 MB/s | 6.8 MB/s |

- **It is the uplink's IPv4 route to Fastly** (151.101.0.0/16, 185.199.108.0/22). The same servers
  over IPv6 (2a04:4e42::/32) run 10–30× faster from the same machines at the same moment, and
  Cloudflare-hosted traffic is fast over both. Not the radio, DNS, or the hosts' resolvers: both Wi-Fi
  adapters reach Cloudflare at 10–12 MB/s and every machine resolves the same CDN addresses.
- **The runners are IPv4-only.** WSL's default NAT mode gives the distribution no IPv6; `ci-runner-01`
  sits behind libvirt's IPv4 NAT on sitar. The hosts themselves are dual-stack (the router advertises
  a prefix). So every crate, toolchain, apt package and image layer a job fetches takes the bad path.
- **Separately, the path MTU is 1480**, not 1500: a 1472-byte don't-fragment ping is refused by the
  router ("needs to be fragmented"), 1452 passes. Lowering WSL's interface to 1480 took IPv4 throughput
  to Cloudflare from 2.1 to 13.6 MB/s and to codeload from 1.2 to 5.9 MB/s.
- **What IPv6 reaches.** Published AAAA records cover the crates index and files, Rust's toolchains,
  ghcr's blob store (pkg-containers.githubusercontent.com), Docker Hub, Debian and Ubuntu archives and
  Docker's repository. GitHub's own hosts do not: github.com, codeload, release assets,
  objects.githubusercontent.com, and the Actions broker and results service are IPv4-only.

## Decision

1. **Every runner host is dual-stack, using the uplink's IPv6 directly.** No tunnel, proxy or third
   party is added to the path, and ADR-0124 D4 is unchanged: the runner gains no credential.
   - **The Windows host (ADR-0131):** WSL runs with `networkingMode=mirrored`, so the distribution
     shares the host's IPv6 (and drops WSL's NAT). Inbound from the LAN stays blocked by the Hyper-V
     firewall; no inbound rule is added.
   - **`ci-runner-01`:** IPv6 reaches it through sitar, governed by RFC-0035's firewall: a static
     unique-local address on the bridge and on the runner only (`address-registry.yaml`, allocated by
     RFC-0037 decision 8's rule), and NAT66 on sitar for that one source. **No router advertisement on
     the bridge**: it is one layer-2 segment with every guest on it, and an advertisement would hand
     IPv6 to the cluster's nodes, which this decision does not change. **Second**, after the
     Windows host has proved the decision end to end, because it changes the substrate host.
   - **Docker on runners** gets IPv6 on its default bridge (a ULA `fixed-cidr-v6`, NAT66 by Docker's
     ip6tables), so a `RUN cargo fetch` inside an image build takes IPv6 too. Whether the networks a
     runner creates per job inherit it is verified on the host, not assumed.
2. **The egress firewall (ADR-0124) covers IPv6 with the same rule: CI reaches nothing inside the
   estate or the operator's LAN.** New TCP and UDP connections are dropped to unique-local (fc00::/7),
   link-local (fe80::/10), and **the LAN's own global prefixes** — the routes the host has on-link. The
   ISP's prefix is dynamic, so those are re-read from the routing table on a timer, not written down.
   ICMPv6 is not dropped: neighbour discovery and path-MTU discovery need it.
3. **IPv4 uses the measured path MTU, 1480, on the runner hosts' uplinks,** set by their roles.
4. **What this does not fix is accepted and measured, not assumed away.** GitHub's IPv4-only hosts keep
   the IPv4 path: codeload measured fast (1–9 MB/s); release-asset downloads are tool installs that
   persist per runner process. The broker long-poll timeouts on the Windows host are re-measured after
   mirrored mode removes WSL's NAT; if they persist, that is a new finding, not this decision failing.

## Consequences

**Accepted costs.**

- **The egress firewall grows an IPv6 half,** including a set refreshed from the routing table. A rule
  that silently stopped matching after a prefix change would let CI reach the LAN, so the refresh is
  verified by a denial test (a connection to a LAN device's global address must fail), not by reading
  the ruleset.
- **Mirrored networking is WSL's newer mode.** Its known issues are tracked upstream; this host is the
  first place the estate depends on it.
- **IPv6 on sitar's libvirt network touches the operator's machine** and RFC-0035's firewall. It is a
  separate, later change, reviewed on its own.
- **GitHub's IPv4-only hosts stay on the slow-to-Fastly path** for release assets.

**Gained.**

- Crates, toolchains, apt, Docker Hub and ghcr layers at 4–19 MB/s instead of 0.03–0.8 MB/s, on both
  runner hosts, with nothing new in the path and no new credential.
- The second runner host can enter service (ADR-0131), which was the point of buying it.

**Rejected alternatives.**

- **Cloudflare WARP (what the operator's Mac uses).** It fixes all of it, but:
  - its tunnel breaks Docker's default bridge on Linux until Docker's MTU is lowered by hand
    (Cloudflare's known limitations), and CI starts Postgres in Docker and builds images in it;
  - its local proxy mode drops any request over 10 seconds, which kills toolchain downloads and the
    runner's long-poll;
  - the business product enrolls a headless machine with a service token, a second credential on a
    machine that runs pull-request code, against ADR-0124 D4;
  - the free consumer licence is "personal", and neither permits nor bars a company's CI;
  - every byte of CI would pass one third party, to fix a route IPv6 already avoids.
  Cloudflare is already the estate's edge (ADR-0061), so this was weighed on merit, not on vendor.
- **A caching mirror for crates and toolchains.** Every first fetch of a new crate or toolchain still
  takes the bad path, it does nothing for ghcr, apt or Docker Hub, and it is another service to run.
  It may still be worth having later for repeat downloads; it is not the fix.
- **Lowering the MTU alone.** Measured: it helped IPv4 generally and did nothing for Fastly.
- **Doing nothing.** `ci-runner-01` fetches crates at ~100 KB/s today, and rust-cache misses on every
  run (infra-ci LOGBOOK), so jobs re-download on that path.

## Verification

On each runner host, after its change:

- inside the runner's Linux **and inside a Docker container**: crates and Rust toolchain downloads
  over IPv6 at MB/s, not KB/s;
- the runners stay online for at least 30 minutes with no broker `TimeoutException` in `_diag`;
- **the denials**: a new connection from the runner's Linux and from a container to the router's
  IPv4 address, to a LAN device's global IPv6 address, and to a unique-local address each fails.

## Addendum — 2026-10-04: the IPv4 problem is the evening, and two consequences the first trial found

**"Unusable" overstated it.** Re-measured at 19:45 UTC (01:15 IST), IPv4 and IPv6 to Fastly were the same,
9–15 MB/s, from the WSL host, sitar and ci-runner-01. Every slow IPv4 reading in the Context table was
taken between about 13:00 and 18:00 UTC (18:30–23:30 IST), and IPv6 measured fast at those same moments.
The ISP's IPv4 route to Fastly congests in the evening; its IPv6 route does not. The decision stands — CI
runs at all hours, and the evening is when the estate's operator works — but the title's "unusable" reads
as always, and it is not. (A test clamping TCP's MSS to 1400 made no difference off-peak, so path-MTU loss
does not explain the hosts' slowness.)

**A unique-local IPv6 address alone does not get used.** RFC 6724 rule 5 prefers a destination whose
label matches the source's; glibc labels fc00::/7 differently from the internet's ::/0, so a host whose
only IPv6 is unique-local resolves a global name's IPv4 first. Shown in a container on Docker's ULA
network. So a runner given a static ULA (ci-runner-01) also gets an `/etc/gai.conf` giving fc00::/7 the
global label (infra-substrate#16). Containers bring their own `gai.conf` and keep preferring IPv4 — an
accepted residual: their evening fetches take the congested path.

**Docker's MTU must match the uplink.** On the WSL host the mirrored uplink is the path MTU, 1480, and
Docker's bridges were 1500: a container's download stalled (0.95 MB/s, against 10.4 MB/s at 1480, the
same minute), and the first trial's cargo-deny container hung for 40 minutes. The runner role sets
Docker's MTU where the uplink is smaller (infra-substrate#16).

