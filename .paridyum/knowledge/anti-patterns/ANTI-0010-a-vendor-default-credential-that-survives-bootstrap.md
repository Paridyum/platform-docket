---
id: ANTI-0010
title: A vendor default credential that survives bootstrap, and a provisioning path that runs through a human
type: anti-pattern
status: accepted
created: 2026-09-07
locked_at: null
supersedes: null
superseded_by: null
scope: [org, platform, infra]
applies_to: ["*"]
tags: [security, bootstrap, credentials, zitadel, identity, default-password, req-0001, adr-0047, adr-0057, c-0004]
breaking: false
sources:
  - url: https://github.com/zitadel/zitadel-charts/blob/zitadel-10.0.4/charts/zitadel/values.yaml
    title: "zitadel-charts 10.0.4 — values.yaml, FirstInstance block"
    retrieved: 2026-09-07
    supports: >
      That the chart creates an `iam-admin` machine user with IAM_OWNER at setup and stores its
      JWT key in a Kubernetes secret named after the username; and that when a `Pat` block is
      present it ALSO mints a PAT into a secret suffixed `-pat` (e.g. `iam-admin-pat`). The
      browserless provisioning credential this anti-pattern says to use is a chart default, not
      a thing that had to be built.
enforced_by: []
---

# ANTI-0010 — A vendor default credential that survives bootstrap

## The shape

A platform component is installed. It comes up with a **vendor default administrator** — a known
username and a documented default password — reachable at whatever address the component was
exposed on. The password change is "required on first login", so it is treated as handled. It is
not handled; it is deferred to whoever logs in first, and until they do, the default stands.

The second half of the same shape: **provisioning the component is done by a human clicking
through its console.** Creating the first API credential is written down as a manual step,
because "you need an admin to make a service account, and there is no admin until someone logs
in." So the manual login becomes load-bearing, the console session is the only way in, and the
setup is unreproducible by construction.

## Why it is not merely untidy

**The default password is public.** "Change required on first login" defends against a password
that is secret and weak. It does nothing against one that is printed in the vendor's quickstart,
because the attacker does not need to keep the password — they need one authenticated request,
and the change-on-login flow is reached only *after* authentication succeeds. Every minute the
component is reachable on that credential is a minute anyone who has read the vendor docs is an
administrator.

This is not hypothetical for this estate. It is written from the incident that produced it:
Zitadel was deployed to `auth.paridyum.com` and reachable for roughly two days with the default
`zitadel-admin@zitadel.<domain>` account on the documented default password, because the install
configured no `FirstInstance` human and nobody had logged in to trigger the change. The exposure
was found by reading the live user table while answering "what is my login", not by any alarm.

**The manual-login provisioning step is the same mistake wearing a process hat.** It is a
credential that exists only in a person's session, a setup that cannot be replayed, and — the
part that stings — it was unnecessary here. The chart had *already* minted a machine credential
with `IAM_OWNER` into a Kubernetes secret at install time. The SOP told an operator to log in and
create by hand a thing that was sitting in `kubectl get secret` the whole time. A provisioning
path that goes through a human is not just slower; it hides the automated path that already
exists.

## The rule

**No component reaches a state where it is both reachable and holding a vendor default
credential, and no provisioning path runs through a human console login.** Concretely:

1. **The default admin is deactivated or its password is set from a secret, as part of
   bootstrap** — in the same change that exposes the component, not in a follow-up ticket. If the
   installer cannot set the password, the component is not exposed until the default is dealt
   with.

2. **Provisioning authenticates as a machine credential minted at install and captured into the
   secret store** (REQ-0001, SOPS per ADR-0057). Where the installer already mints one — as this
   chart does — bootstrap *captures* it into the secret store rather than leaving it only in the
   cluster and rather than minting a second one by hand.

3. **The human step is reduced to the one thing that genuinely requires a human** (C-0004): a
   person setting *their own* administrative password into their own password manager, once,
   whenever they need interactive access — never as a prerequisite for automation to run.

## What it is not

It is not a rule against vendor defaults existing. A chart shipping a default admin so the thing
is usable out of the box is reasonable. The anti-pattern is the default **surviving bootstrap**
while the component is reachable, and the provisioning path **depending on** logging in as it.

## Errata

<!-- none yet -->
