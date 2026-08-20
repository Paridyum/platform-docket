---
id: RFC-0021
title: A device hostname is family, country and sequence, and it is not a database
type: rfc
status: proposed
created: 2026-08-12
locked_at: null
supersedes: null
superseded_by: null
scope: [org, infra, platform]
applies_to: ["*"]
tags: [naming, hostnames, dns, devices, infrastructure, proxmox, c-0007, rfc-0014, rfc-0019]
breaking: false
enforced_by: []
sources:
  - url: https://www.rfc-editor.org/rfc/rfc1178.txt
    title: RFC 1178 — Choosing a Name for Your Computer
    retrieved: 2026-08-12
    supports: >
      Theme names are recommended and should be expandable rather than a finite set; names
      over eight characters "simply annoy people"; case is not to be relied on; sequential
      numbering is appropriate only where humans need not distinguish machines; and renaming
      an established host brings "an amazing amount of pain". Decision 4 and decision 6 rest
      on this, and decision 3 deviates from its warning about geographical names.
  - url: https://www.rfc-editor.org/rfc/rfc1123
    title: RFC 1123 — Requirements for Internet Hosts, Application and Support
    retrieved: 2026-08-12
    supports: >
      Host labels are letters, digits and hyphen only; the leading-digit restriction of
      RFC 952 is relaxed; a label must not end in a hyphen. Decision 1's character set.
  - url: https://www.rfc-editor.org/rfc/rfc952
    title: RFC 952 — DOD Internet Host Table Specification
    retrieved: 2026-08-12
    supports: >
      The original host-name syntax that RFC 1123 relaxes, and the source of the reserved
      names GATEWAY, GW and TAC which decision 4's family registry must not collide with.
  - url: https://www.rfc-editor.org/rfc/rfc1035
    title: RFC 1035 — Domain Names, Implementation and Specification
    retrieved: 2026-08-12
    supports: >
      A DNS label is at most 63 octets and a full domain name at most 255. Decision 7's
      headroom argument — the canonical form is nowhere near this limit, and the binding
      constraint is Windows rather than DNS.
  - url: https://www.rfc-editor.org/rfc/rfc4343
    title: RFC 4343 — Domain Name System (DNS) Case Insensitivity Clarification
    retrieved: 2026-08-12
    supports: >
      DNS comparison is case-insensitive over ASCII. Decision 1's lowercase rule — case
      cannot distinguish two devices, so permitting it in the canonical form buys nothing
      and costs comparison bugs in the case-sensitive tools above DNS.
  - url: https://www.rfc-editor.org/rfc/rfc2119
    title: RFC 2119 — Key words for use in RFCs to Indicate Requirement Levels
    retrieved: 2026-08-12
    supports: The MUST/SHOULD/MAY vocabulary this RFC's validation rules use.
  - url: https://www.rfc-editor.org/rfc/rfc8174
    title: RFC 8174 — Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words
    retrieved: 2026-08-12
    supports: >
      Only uppercase key words carry RFC 2119 force. Cited so the validation section's
      lowercase prose is not read as normative by accident.
  - url: https://www.rfc-editor.org/rfc/rfc6761
    title: RFC 6761 — Special-Use Domain Names
    retrieved: 2026-08-12
    supports: >
      Establishes the IANA Special-Use Domain Names registry. Decision 7's argument that a
      private namespace must be either owned or formally reserved, never invented.
  - url: https://www.rfc-editor.org/rfc/rfc6762
    title: RFC 6762 — Multicast DNS
    retrieved: 2026-08-12
    supports: >
      `.local` is reserved for link-local multicast DNS resolution. Decision 7 forbids it
      outright; an estate that used it would collide with mDNS on every link.
  - url: https://www.rfc-editor.org/rfc/rfc8375.html
    title: RFC 8375 — Special-Use Domain 'home.arpa.'
    retrieved: 2026-08-12
    supports: >
      `home.arpa` is reserved for non-unique use in residential home networks and cannot be
      DNSSEC-validated from the root trust anchor. Considered and rejected in decision 7.
  - url: https://www.rfc-editor.org/rfc/rfc4193
    title: RFC 4193 — Unique Local IPv6 Unicast Addresses
    retrieved: 2026-08-12
    supports: >
      ULA global IDs MUST be generated pseudo-randomly and MUST NOT be assigned sequentially
      or with well-known numbers. Decision 9's reason for refusing to carry the draft's
      sequential 0–255 country allocation into anything IPv6-shaped.
  - url: https://maxmind.github.io/GeoIP2-node/interfaces/ContinentRecord.html
    title: MaxMind GeoIP2 — ContinentRecord
    retrieved: 2026-08-12
    supports: >
      The two-letter continent code set AF, AN, AS, EU, NA, OC, SA. Decision 3 adopts it
      rather than inventing letters, which is what produced the draft's collisions.
  - url: https://learn.microsoft.com/en-us/troubleshoot/windows-server/active-directory/naming-conventions-for-computer-domain-site-ou
    title: Microsoft Learn — Name computers, domains, sites, and OUs (KB 909264)
    retrieved: 2026-08-12
    supports: >
      "Windows doesn't permit computer names that exceed 15 characters, and you can't specify
      a DNS host name that differs from the NetBIOS host name." Also: use ASCII, use only
      RFC 1123 characters, and "don't use character case to indicate the owner or the purpose
      of a computer". Decision 7's compact form exists entirely because of the 15-character
      limit. Page states it was updated 2026-08-04.
  - url: https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-nbte/6f06fa0e-1dc4-4c41-accb-355aaf20546d
    title: "[MS-NBTE] NetBIOS name syntax"
    retrieved: 2026-08-12
    supports: >
      A NetBIOS name is 16 characters with the 16th reserved for the service-type suffix,
      which is why the usable limit is 15 rather than 16.
  - url: https://kubernetes.io/docs/concepts/overview/working-with-objects/names/
    title: Kubernetes — Object Names and IDs
    retrieved: 2026-08-12
    supports: >
      RFC 1123 label names must be at most 63 characters and contain only lowercase
      alphanumerics and '-'. Decision 1's lowercase rule — an identity that cannot be used
      as a Kubernetes object name is an identity that stops at the cluster edge.
  - url: https://docs.aws.amazon.com/whitepapers/latest/tagging-best-practices/tagging-best-practices.html
    title: AWS Whitepaper — Best Practices for Tagging AWS Resources
    retrieved: 2026-08-12
    supports: >
      Operational metadata belongs in a tagging schema rather than in resource names.
      Decision 8's separation of identity from metadata.
  - url: https://docs.aws.amazon.com/whitepapers/latest/tagging-best-practices/best-practices-for-naming-tags-and-resources.html
    title: AWS Whitepaper — Best Practices for Naming Tags and Resources
    retrieved: 2026-08-12
    supports: The naming half of the same whitepaper, cited alongside the tagging half.
  - url: https://www.iso.org/iso-3166-country-codes.html
    title: ISO 3166 — Country Codes
    retrieved: 2026-08-12
    supports: >
      The alpha-2 country code set decision 2 uses for the country field, and which
      region-registry.yaml already uses for jurisdiction codes.
  - url: https://unstats.un.org/unsd/methodology/m49/
    title: UN M49 — Standard Country or Area Codes for Statistical Use
    retrieved: 2026-08-12
    supports: >
      Numeric area codes that are more change-resilient than ISO alpha-2 but not
      human-readable. Considered and rejected in decision 3.
  - url: https://www.icann.org/en/announcements/details/icann-seeks-feedback-on-proposed-top-level-domain-string-for-private-use-24-01-2024-en
    title: ICANN — Proposed Top-Level Domain String for Private Use
    retrieved: 2026-08-12
    supports: >
      `.internal` was identified as the reserved private-use TLD following SAC113.
      Considered and rejected in decision 7 in favour of a domain we own.
  - url: https://www.iana.org/news/2024/proposed-private-use-tld
    title: IANA — Proposed private-use TLD
    retrieved: 2026-08-12
    supports: The IANA-side record of the same `.internal` reservation.
---

# RFC-0021 — A device hostname is family, country and sequence

## Status

**PROPOSED** as of 2026-08-12. Feedback deadline **2026-09-11**.

Sixth in the region series and the first artifact to govern **physical hardware**. C-0007 names
repositories, crates, HTTP paths, tables, events, environment variables, container images and
Kubernetes labels — eight naming domains, and every one of them is software. It stops at the edge
of the machine.

RFC-0019 committed this platform to owning metal. That estate needs names **before** it has
machines, because renaming is the expensive direction and RFC 1178 has said so since 1990.

## The question

A draft device-naming standard exists, written independently of this repository. It proposes
musical family names (`Sitar` for servers, `Dhun` for routers, `Murli` for mobile devices), a
geography built from continent and country initials, a chronological sequence, and an optional
platform suffix.

**The instinct is right and most of the reasoning is right.** Identity separated from metadata,
sequence numbers never reused, IP addresses decoupled from names, physical site excluded — all
correct, all argued well, and all consistent with the sources the draft cites.

The draft is nonetheless not adoptable as written. It contains **collisions that are demonstrable
from its own table**; it contains **internal contradictions** where sections cannot all be true;
and it was written **without knowledge of `region-registry.yaml`**, so it invents a geography
beside one that already exists.

This RFC takes the draft seriously, states what breaks, and proposes the corrected form.

## What is wrong with the draft, stated first

Put here rather than in an appendix, because a reader who only skims should still leave knowing
that the original scheme cannot be deployed.

### The geographic codes collide, and the draft's own table proves it

The draft builds a two-letter code from the region's initial and the country's initial. Applied
to the eighteen rows the draft itself publishes:

| Code | Claimed by |
| --- | --- |
| `AS` | **A**sia/**S**ingapore · **A**sia/**S**outh Korea · **A**frica/**S**outh Africa |
| `AI` | **A**sia/**I**ndia · **A**sia/**I**ndonesia |
| `ES` | **E**urope/**S**weden · **E**urope/**S**pain |

Three regions — Asia, Africa, Australia — collapse to `A`. The draft's §21 uses `Sitar-AS-1P` for
Singapore while its own §8 assigns `AS` to two other countries. The scheme also silently
appropriates real ISO 3166-1 codes with different meanings: `AI` is Anguilla, `AS` is American
Samoa, `NC` is New Caledonia.

**One-letter abbreviations of an open-ended list collide. This is not a bug in the table; it is a
property of the construction**, and it is the reason decision 3 adopts a published code set rather
than deriving letters.

### Three sections cannot all be true

The draft's §12 puts the host platform in the hostname. Its §22 says the platform suffix *"does
not create a second identity"*. Its §23 says *"do not rename a device… its OS changes"*.

Reinstalling `Sitar-AI-1P` as Windows must then either rename it — violating §23 — or leave the
name asserting a platform the machine no longer runs. Additionally the draft's Rule 5 permits only
`W` and `P`, so a Debian, TrueNAS or OPNsense host has **no legal name** under the draft's own
validation rules.

### Encoding geography in a *mobile* device is self-defeating

`Murli` is defined as phones and tablets. §23 forbids renaming on relocation. A phone registered
in India therefore remains `Murli-AI-1` in Japan — the field becomes false exactly when someone
might rely on it.

### The draft cites RFC 1178 in support of a rule RFC 1178 warns against

The draft leans on RFC 1178 repeatedly and correctly for sequence stability. But that RFC's
guidance on geography is:

> Domain names are either organizational (e.g., cia.gov) or geographical (e.g., dallas.tx.us).
> Using anything like these tends to imply some connection.

Encoding geography is the thing RFC 1178 discourages. That does not make it wrong — 1990 predates
infrastructure-as-code, fleets and residency law — but a standard must **say it is deviating**
rather than cite the deviation's critic as its authority. Decision 3 states the deviation.

### Two more, briefly

- **Mixed case is declared canonical** (§17), which RFC 4343, Kubernetes and Microsoft all
  independently argue against.
- **§19 misuses "Region"** for two sites in one country. `region-registry.yaml` FIX 5 has already
  settled this: two locations in one jurisdiction are two **datacentres**, and *"a region code
  never carries a number. If you are reaching for one, you want a datacentre."*

## Decision 1 — the grammar, and it is lowercase

Two productions. Fixed-location devices carry geography; mobile devices do not.

```
<family>-<continent>-<country>-<sequence>[-<platform>]
<family>-<sequence>[-<platform>]
```

```abnf
fixed-host  = family "-" continent "-" country "-" sequence [ "-" platform ]
mobile-host = family "-" sequence [ "-" platform ]

family      = 3*6(%x61-7A)          ; from device-registry.yaml
continent   = "af" / "an" / "as" / "eu" / "na" / "oc" / "sa"
country     = 2(%x61-7A)            ; ISO 3166-1 alpha-2, lowercased
sequence    = 2*3DIGIT              ; zero-padded, never reused
platform    = 2(%x61-7A)            ; from device-registry.yaml
```

Worked:

```
sitar-as-in-01-px     server · Asia · India · #1 · Proxmox
sitar-as-in-03-wn     server · Asia · India · #3 · Windows
sitar-eu-es-01-px     server · Europe · Spain · #1 · Proxmox
sitar-eu-se-02-db     server · Europe · Sweden · #2 · Debian
dhun-as-in-01-op      router · Asia · India · #1 · OPNsense
murli-01              mobile device · #1
```

**Canonical case is lowercase.** Three independent authorities converge: RFC 4343 makes DNS
comparison case-insensitive, so case can never distinguish two devices; Kubernetes *requires*
lowercase for RFC 1123 label names, so a mixed-case identity cannot be used as an object name;
and Microsoft's guidance is explicit — *"don't use character case to indicate the owner or the
purpose of a computer… Windows and Windows applications don't preserve case in all situations."*

The tools **above** DNS are the problem, not DNS. Ansible inventories, Prometheus labels, Grafana
variables and shell comparisons are all case-sensitive. A canonical form that tolerates
`Sitar-AS-IN-01-PX` and `sitar-as-in-01-px` guarantees that two of them eventually fail to join.

Title case survives **only** as a documentation rendering: `Sitar-Asia-India-01-Proxmox`.

## Decision 2 — field three is the country, and it is never the jurisdiction

**This is the decision most likely to be misread, so it is stated before the ones that depend on
it.**

`region-registry.yaml` already defines a geographic identifier: the **region**, which is a
residency boundary and whose code is a jurisdiction — `in`, `sg`, `us`, or supranational `eu`.
That file is emphatic that a region is a *legal* construct and **not a building**.

A hostname's geography is a **different axis**, and the registry itself says so:

> address space is allocated per COUNTRY, a region is per JURISDICTION, and a country routinely
> has the first without the second.

So:

| Axis | Answers | Example | Lives in |
| --- | --- | --- | --- |
| **Country** | Where is this hardware physically? | `es` — Spain | the hostname |
| **Jurisdiction** | Which residency boundary governs this tenant's data? | `eu` — EEA | `region-registry.yaml` |
| **Datacentre** | Which building and failure domain? | `in-mum-1` | inventory |

A server in Madrid **is in Spain**. That is a fact about metal. "It is in the EEA" is a fact about
data. Both are true, neither substitutes, and the hostname records the physical one because the
hostname names a physical thing.

**Residency remains a lookup, and that is correct.** `es` and `se` both resolve to the `eu` region
through the registry. Adding a Swedish machine changes no region, because Sweden is not a region —
GDPR Art. 1(3) put it in the same residency boundary as Spain, and `region-registry.yaml` records
that reasoning already.

### The trap, written down because the common case hides it

For India the country code and the jurisdiction code are **the same string**: `in`.

So `sitar-as-in-01-px` looks exactly as it would if field three were the region — and for the one
region that currently exists, **nothing distinguishes the two readings.** Spain makes the
distinction obvious (`es` ≠ `eu`); India makes it invisible.

An implementer who infers the rule from the live estate will infer the wrong rule. Hence this
decision, stated normatively: **field three is the country in which the device physically sits. It
is never the region and never the jurisdiction.**

## Decision 3 — the continent is denormalized, published rather than derived, and checked

The continent field is **redundant**: it is derivable from the country, and
`region-registry.yaml` already carries `continent:` on every jurisdiction. It is kept anyway, for
grouping and legibility — `sitar-as-*` selects the Asian fleet, and a reader who does not know
that `cl` is Chile can still place `sa-cl`.

**Redundancy is only dangerous when unverified.** A check that validates the hostname's continent
against the registry converts this from drift risk into machine-enforced denormalization, which
is the trade this repository makes everywhere else.

**The code set is MaxMind's** — `af`, `an`, `as`, `eu`, `na`, `oc`, `sa` — adopted rather than
derived, because deriving letters from names is precisely what produced the draft's three-way `AS`
collision. Two consequences of adopting it:

- **Australia is a country, not a continent.** The draft's §8 lists "Australia" as a region. The
  continent is Oceania, `oc`, and Australia is `au` within it.
- **`eu` is overloaded across axes.** It is the continent code for Europe *and* the jurisdiction
  code for the EEA. In `sitar-eu-es-01-px` the leading `eu` is the continent; in
  `region-registry.yaml` the bare `eu` is a jurisdiction. Positional, unambiguous to a parser,
  and a genuine hazard to a human. Recorded rather than smoothed over.

### The honest limit of this field

The strongest available argument for a continent prefix is error prevention — ISO alpha-2 codes
are confusable, and a continent disambiguates a misread. **Tested against this estate's own
country list, that argument mostly fails.**

| Confusable pair | With continent | Helps? |
| --- | --- | --- |
| `in` India · `id` Indonesia | `as-in` · `as-id` | **No** — same continent |
| `es` Spain · `se` Sweden | both in the `eu` jurisdiction | **No** |
| `ae` UAE · `au` Australia | `as-ae` · `oc-au` | Yes |
| `ca` Canada · `cl` Chile | `na-ca` · `sa-cl` | Yes |
| `kr` Korea · `br` Brazil | `as-kr` · `sa-br` | Yes |

The prefix fails on **India versus Indonesia** — two adjacent jurisdictions, both Asia, both on
the market list, and the single most confusable pair in the estate. It succeeds only on pairs
already separated by obvious context.

**So the continent buys grouping and readability. It does not buy safety, and this RFC does not
claim that it does.** Anyone later tempted to justify the field on error-prevention grounds should
read this table first.

## Decision 4 — device families are registry data, not prose in this artifact

The draft states that *"no additional musical naming families are introduced by this standard"*.
That forecloses switches, access points, storage, PDUs and cameras — every one of which is
foreseeable, and none of which currently has a legal name.

**C-0007 has already solved this exact problem and its solution is adopted verbatim:**

> The grammar is constitutional. The list is data.

C-0002 held repository classes in locked prose, so adding one meant superseding a constitution;
C-0005 moved the list to `class-registry.yaml`. A closed family list in this artifact would
recreate that defect one layer down.

Families therefore live in **`device-registry.yaml`**, proposed alongside this RFC in the same
shape and with the same non-binding banner `region-registry.yaml` carries.

Two constraints on admitting a family, both mechanical:

1. **A family token is at most six characters.** The compact form of decision 7 is
   `family + 9` characters at a three-digit sequence, so six keeps every name inside the Windows
   15-character limit **by construction** rather than by review.
2. **No family token may be a prefix of another.** The compact form removes the delimiters, and
   two grammars share the namespace, so a `mur` admitted beside `murli` would make
   `murli01` ambiguous. `sitar`, `dhun` and `murli` are prefix-free.

A family token must also avoid `gateway`, `gw` and `tac`, reserved by RFC 952.

**The musical theme is kept and is good practice**, not merely tolerated. RFC 1178 recommends
theme names explicitly and asks for themes that expand rather than finite sets — Indian
instruments expand.

## Decision 5 — the sequence is chronological, zero-padded, and never reused

Two digits, zero-padded, widening to three at 99. Unpadded sequences sort wrong in every tool in
this estate, and `01` before `10` is worth two characters.

**A retired number stays retired.** If `sitar-as-in-02-px` is decommissioned, the next server is
`sitar-as-in-04-px`. RFC 1178's closing warning is the argument, and it applies to reuse as
sharply as to renaming: logs, monitoring series, backup catalogues, firewall rules, DNS, git
history and incident records all outlive the hardware, and a reused number silently merges two
machines' histories.

The sequence is **per family per country**. `sitar-as-in-01-px`, `dhun-as-in-01-op` and `murli-01`
coexist and are three different devices.

## Decision 6 — the platform suffix stays in the hostname, and a rebuild is therefore a rename

**This decision was taken against the recommendation recorded in this RFC, and is written as
chosen rather than as advised.**

The alternative — platform as metadata, hostname permanently stable — resolves the draft's
three-way contradiction outright and costs nothing but at-a-glance visibility. It was declined in
favour of keeping the platform visible in shell prompts and dashboards.

What follows from that choice, stated so it is not discovered later:

- **The draft's §23 clause "do not rename because its OS changes" is deleted.** It cannot survive
  alongside a platform-bearing hostname.
- **The permanent identity is `family-continent-country-sequence`.** The platform suffix is a
  **declared-mutable trailing component**. The draft's §22 is reworded accordingly: the base is
  unique and permanent; the suffix describes and may change.
- **A platform change is a rename event with a runbook**, not an edit. It touches DNS, IPAM,
  monitoring targets and series, backup catalogues, firewall rules, configuration management
  inventories and any TLS certificate carrying the name. This runbook does not yet exist and is
  open question 3.
- **Platform codes are two characters**, from `device-registry.yaml`. Single letters were rejected
  for the reason the geography section already demonstrates: `o` cannot be OPNsense, OpenWrt and
  OpenBSD simultaneously.

The suffix is **optional on every family**. A machine whose platform is not operationally
interesting omits it.

**A note on how much this will actually fire.** RFC-0019 decision 1 makes every datacentre one
Proxmox cluster and decision 2 puts Kubernetes in Talos virtual machines above it. In that estate
nearly every `sitar-*` is `-px`, and the suffix carries little information precisely where it is
most used. That is an argument for the rejected alternative, and it is recorded here rather than
lost.

## Decision 7 — one canonical name, and a compact form only where Windows forces it

Canonical form is 17 characters at a five-character family. **Windows permits 15.** Microsoft is
unambiguous, and the constraint is the operating system rather than Active Directory:

> Windows doesn't permit computer names that exceed 15 characters, and you can't specify a DNS
> host name that differs from the NetBIOS host name.

The limit is 15 rather than 16 because a NetBIOS name is sixteen characters with the last reserved
for the service-type suffix.

**The compact form is the canonical form with the hyphens removed.**

| Canonical | Compact | Length |
| --- | --- | --- |
| `sitar-as-in-01-px` | `sitarasin01px` | 13 |
| `sitar-as-in-003-px` | `sitarasin003px` | 14 |
| `dhun-as-in-01-op` | `dhunasin01op` | 12 |
| `murli-01` | `murli01` | 7 |

Two properties make this safe rather than a second naming scheme:

1. **It is a total function, not a judgement.** `compact(n) = n.replace("-", "")`. There is no
   case where a human chooses the short name.
2. **Identity is defined on the normalized form.** Two names denote the same device **iff**
   `lowercase(name).replace("-", "")` are equal. This is the single function the tooling owes,
   and it is what stops an Active Directory computer object from failing to join a Prometheus
   target label.

DNS itself is nowhere near its limits — RFC 1035 allows 63 octets per label and 255 per name — so
this is a Windows accommodation and nothing else.

### The zone

**`inf.paridyum.com`.** `sitar-as-in-01-px.inf.paridyum.com`.

A domain we own is the only option that cannot collide, can be DNSSEC-signed from the root, and
works unchanged for split-horizon resolution, VPN clients and TLS certificates.

- **`.local` is forbidden.** RFC 6762 reserves it for link-local multicast DNS. Microsoft's own
  guidance says to avoid *"names that are used in internet-standard special features, such as
  `.local`"*. An estate using it collides with mDNS on every link it touches.
- `home.arpa` was considered. RFC 8375 reserves it properly, but it is scoped to a single home
  network and cannot be DNSSEC-validated from the root trust anchor.
- `.internal` was considered. ICANN reserved it for private use following SAC113, so it is
  collision-safe — but it offers nothing a domain we already own does not.
- An invented TLD was never on the table. RFC 6761 exists because that practice broke.

## Decision 8 — the hostname is not a database, and the datacentre is not in it

Carried forward from the draft, which argues this well and needs no correction.

The hostname records family, continent, country, sequence and optionally platform. **Everything
else is inventory**: hardware vendor and model, serial, CPU, memory, storage, MAC, IP, rack,
firmware, owner, purchase record, workload, environment — and, specifically, the **datacentre**.

`in-mum-1` is a real identifier from `region-registry.yaml` and it is an **inventory field, never
a hostname component**. A machine that moves from `in-mum-1` to `in-hyd-1` stays
`sitar-as-in-01-px`, because it is still in India and still the same machine. This is the draft's
strongest section and its rejection of `Sitar-Bidholi-1` was correct.

The AWS tagging whitepaper makes the general form of this argument: operational metadata belongs
in a tagging schema, not in resource names.

**The country is the one geographic fact stable enough to name.** Hardware crosses buildings
routinely, racks constantly, and borders almost never — and when it does cross a border, the
residency question that changes is significant enough to deserve an explicit review rather than a
silent rename.

## Decision 9 — what this artifact does not govern

Stated positively so the boundaries are not inferred.

- **Virtual machines and Kubernetes nodes.** RFC-0019 decision 2 makes cluster nodes Talos VMs
  provisioned declaratively; their names come from that tooling and C-0007's Kubernetes section.
  The draft's §14 correctly refuses to conflate host and workload but never says what a VM is
  called — this RFC does not fill that gap, it assigns it elsewhere.
- **Address allocation.** The draft's §8 carries a 0–255 per-country range table.
  `region-registry.yaml` has already identified it: *"the archive's IPAM address-space
  reservation (X octets 208–255)… a different axis entirely."* It is an IPv4 plan inside a naming
  standard, and the draft's own §9 argues names and addresses must be decoupled. It belongs in a
  separate addressing RFC. Note for whoever writes it: RFC 4193 requires ULA global IDs be
  **pseudo-random**, so a sequential country allocation does not survive contact with IPv6.
- **Personal and non-Paridyum devices.** Open question 1.

## Validation rules

A hostname is valid when all hold. RFC 2119 keywords; per RFC 8174 only the uppercase forms are
normative.

1. The family MUST appear in `device-registry.yaml`.
2. The continent MUST be one of `af an as eu na oc sa`.
3. The country MUST be a valid ISO 3166-1 alpha-2 code, lowercased. It need **not** have a
   jurisdiction entry — hardware may exist in a country before a region does.
4. The continent MUST agree with the country's continent as recorded in `region-registry.yaml`.
5. The sequence MUST be zero-padded, MUST NOT be `00`, and MUST NOT already be assigned or
   retired within its family and country.
6. The platform, when present, MUST appear in `device-registry.yaml`.
7. The name MUST match `^[a-z0-9-]+$`, MUST start with a letter and MUST NOT end with a hyphen.
8. The name MUST be lowercase in canonical form.
9. The compact form MUST NOT exceed 15 characters.
10. Arbitrary suffixes MUST NOT be appended — not `-dell`, `-prod`, `-dev`, `-gpu`.
11. No IP address component. `sitar-as-in-192-168-1-10` is invalid.

```
fixed:   ^(sitar|dhun)-(af|an|as|eu|na|oc|sa)-[a-z]{2}-[0-9]{2,3}(-[a-z]{2})?$
mobile:  ^murli-[0-9]{2,3}(-[a-z]{2})?$
```

The alternation is generated from `device-registry.yaml`, never hand-maintained — a hardcoded
list would pass its own tests while disagreeing with the registry, which is the failure ADR-0036
names.

## Consequences

**Accepted costs**

1. **A rebuild is a rename** (decision 6), against this RFC's own recommendation. Every platform
   change now touches DNS, IPAM, monitoring, backups, firewall rules and configuration
   management. Under RFC-0019's estate the suffix will also be `-px` on nearly every server, so
   the cost is paid where the information content is lowest.

2. **Two renderings of one identity** (decision 7). The normalization function is small but it is
   a real obligation, and until it exists a Windows host's Active Directory name and its
   canonical name are different strings.

3. **The continent field is redundant and does not prevent the errors it looks like it prevents.**
   Decision 3's table is the evidence. It is kept for grouping, and with one live region there is
   currently nothing to group.

4. **A denormalized field needs a check that does not exist.** Rule 4 is unenforced, joining the
   larger half of `check-registry.yaml`. Until `tool-forge` implements it, the continent is
   correct by compliance.

5. **`eu` names both a continent and a jurisdiction** (decision 3). Unambiguous to a parser, a
   genuine hazard to a human reading quickly.

6. **A third registry file.** `device-registry.yaml` joins `class-registry.yaml` and
   `region-registry.yaml`, each with its own schema and its own validity check.

**Gained**

- Physical hardware acquires a naming rule for the first time; C-0007's eight software domains
  gain a ninth that is not software.
- Every collision in the draft's published table is eliminated by construction, because the code
  sets are adopted rather than derived.
- The geography reconciles with `region-registry.yaml` instead of competing with it — country and
  jurisdiction are separate axes and each has exactly one home.
- Families are registry data, so admitting a switch class needs no supersession.
- The Windows 15-character limit is satisfied **by construction** via the six-character family
  rule, rather than discovered when the first Windows server is built.
- The zone is decided, and `.local` is forbidden with a citation rather than a preference.

## Alternatives considered

**Adopt the draft as written.** Not possible: `AS` names three countries in its own table.

**Platform as metadata, hostname permanently stable.** The recommended option. Declined; recorded
in decision 6 with its full consequence rather than omitted.

**Drop geography entirely** (`sitar-01`). Most compliant with RFC 1178, survives every relocation,
and needs no continent check. Rejected because country is the one geographic fact stable enough
to name, and a residency-shaped estate benefits from seeing it.

**Region codes carry a continent** (`as-in` as the *region*). Structurally impossible.
`region-registry.yaml`'s `eu` is supranational — there is no country to pair — and the
contemplated `ae-difc` is sub-national. Continent+country cannot express either boundary.

**UN M49 numeric codes.** More change-resilient than ISO alpha-2, since M49 codes survive a
country renaming. Rejected as unreadable in a hostname: `sitar-142-356-01-px` fails the only test
a hostname has to pass.

**One-letter platform codes.** Rejected for the reason the geography section demonstrates at
length.

**Single canonical form, no compact variant.** Achievable by shortening families to three
characters (`sit`, `dhu`, `mur`), which fits 15 with the continent retained. Rejected because it
discards the musical identity in the one place the name is actually read.

## Open questions

1. **Does this bind personal devices?** The `murli` family is phones and tablets, which are
   plausibly personal rather than Paridyum estate. The scope declared here is Paridyum
   infrastructure. Either `murli` is out of scope, or the estate boundary needs stating —
   the second is more useful, since an operator's device that reaches production is exactly the
   thing an inventory should know about.

2. **Which check enforces this, and who owns it?** Proposed: `infra.hostname_matches_scheme`
   (error) and `infra.hostname_continent_matches_registry` (error). Both would be owned by
   `tool-forge` and both would join the unimplemented majority. ADR-0002 requires a named check
   before anything here could ever be locked.

3. **The rename runbook does not exist.** Decision 6 makes platform changes routine and they have
   no procedure. This is the highest-value follow-up and belongs in `runbooks/`.

4. **How does the existing estate migrate?** RFC-0001 owns existing-estate reconciliation. The
   draft names `Bidholi` hosts explicitly; nothing here says whether they are renamed, dual-named
   during a transition, or left alone until replaced.

5. **Is `an` (Antarctica) worth carrying?** It comes free with the MaxMind set. Keeping an
   unreachable value costs nothing; the alternative is a subset that needs justifying at the
   first exception.

## Related

- **C-0007** — the naming constitution this extends into hardware; its "grammar is
  constitutional, list is data" rule is adopted in decision 4
- **RFC-0014** · `region-registry.yaml` — the region model decision 2 reconciles against; region
  is jurisdiction, and a region code never carries a number
- **RFC-0019** — a datacentre is one Proxmox cluster; the source of the `-px` ubiquity noted in
  decision 6 and of the VM boundary in decision 9
- **RFC-0020** — why every external fact above carries a `retrieved` date
- **RFC-0001** — existing estate reconciliation, which owns open question 4
- **ADR-0002** — why nothing here can be locked without a named check
- **ADR-0035** — counts in prose are snapshots; the argument decision 3 reuses for a
  denormalized field
- **ADR-0036** — a check's parameters come from a citing artifact, never from prose; why the
  validation regex is generated from the registry
