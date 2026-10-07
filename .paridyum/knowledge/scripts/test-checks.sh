#!/usr/bin/env bash
#
# Proves the gates by breaking them.
#
# README.md's most useful sentence is "treat a green check as unproven until you have watched
# it fail", and until now that was a discipline rather than a script. The ADR-0014 audit found
# four checks reporting success while enforcing nothing; every one of them would have been
# caught by planting the defect it claimed to reject. This runs that plant on every push to
# main, nightly, and on every pull request that can reach it (scripts/check-suite-scope.sh).
#
# A check is counted as implemented only if it appears here. That is the rule that stops the
# implemented column in check-registry.yaml drifting back into a wish.
#
#   ./test-checks.sh          run every case
#   ./test-checks.sh -v       show the output of each case

set -uo pipefail
ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT" || exit 1
VERBOSE=false; [[ "${1:-}" == "-v" ]] && VERBOSE=true

if [[ -t 1 ]]; then RED=$'\033[31m'; GRN=$'\033[32m'; RST=$'\033[0m'; else RED=""; GRN=""; RST=""; fi
PASS=0; FAIL=0
# STD-0008: every blocking tier publishes a machine-readable artifact. This suite IS this
# repository's blocking tier, and until now it wrote only to stdout — so the honest state
# recorded in TESTING.md was that the repository failed its own evidence rule. A log answers
# "did it pass" for as long as GitHub keeps it; JUnit answers "which case, on which commit"
# afterwards, which is the question that actually gets asked.
#
# Written to junit.xml unless JUNIT_OUT says otherwise; set it empty to disable.
JUNIT_OUT="${JUNIT_OUT-junit.xml}"
CASES=""
record() { # record <status> <name> <detail>
  [[ -n "$JUNIT_OUT" ]] || return 0
  # XML-escape, and drop the ANSI colours the terminal branch may have added.
  local n d
  n="$(printf '%s' "$2" | sed $'s/\033\[[0-9;]*m//g; s/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g')"
  d="$(printf '%s' "${3:-}" | sed $'s/\033\[[0-9;]*m//g; s/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g' | head -c 2000)"
  if [[ "$1" == pass ]]; then
    CASES+="    <testcase name=\"$n\"/>"$'\n'
  else
    CASES+="    <testcase name=\"$n\"><failure message=\"$d\"/></testcase>"$'\n'
  fi
}

ok()   { printf '%s✔%s %s\n' "$GRN" "$RST" "$1"; PASS=$((PASS+1)); record pass "$1"; }
bad()  { printf '%s✘%s %s\n' "$RED" "$RST" "$1"; FAIL=$((FAIL+1)); record fail "$1" "${2:-}"; }

# expect_ok <label> <cmd...>   — the command must exit 0
#
# ON FAILURE IT SHOWS THE LAST LINES, NOT THE FIRST, AND THAT WAS A REAL DEFECT. It used
# `head -4`, and every suite invoked through here prints its PASSING cases before its failing
# one — so a failure report was four green ticks and no cause. It also piped into `head`, which
# closes the pipe and makes `sed` print "couldn't flush stdout: Broken pipe" underneath, so the
# one line of output that WAS about the failure was a message about plumbing.
#
# Found while diagnosing a CI failure this harness had already reported and rendered unreadable.
expect_ok() {
  local label="$1"; shift
  local out; out="$("$@" 2>&1)"; local rc=$?
  [[ "$VERBOSE" == true ]] && printf '%s\n' "$out"
  if [[ $rc -eq 0 ]]; then ok "$label"; else bad "$label — expected exit 0, got $rc" "$out"; printf '%s\n' "$out" | tail -15 | sed 's/^/      /'; fi
}
# expect_warns <label> <pattern> <cmd...>  — output must contain the pattern
# Needed because a `warn`-level check never changes the exit code, so expect_caught can never
# assert one. Two knowledge checks are warn-level by design — knowledge.review_dates_not_overdue
# and knowledge.resolved_by_names_live_decision — and were therefore unprovable until this existed.
expect_warns() {
  local label="$1" pat="$2"; shift 2
  local out; out="$("$@" 2>&1)"
  [[ "$VERBOSE" == true ]] && printf '%s\n' "$out"
  if grep -q "$pat" <<<"$out"; then ok "$label"; else bad "$label — no output matched '$pat'" "$out"; fi
}

# expect_caught <label> <cmd...>  — the command must exit non-zero (defect caught)
expect_caught() {
  local label="$1"; shift
  local out; out="$("$@" 2>&1)"; local rc=$?
  [[ "$VERBOSE" == true ]] && printf '%s\n' "$out"
  if [[ $rc -ne 0 ]]; then ok "$label"; else bad "$label — the planted defect was NOT caught (exit 0)" "$out"; fi
}

printf '\n── migration gate (ADR-0024) ──────────────────────────────────────\n'
F=tests/fixtures/migrations
expect_ok "clean schema passes" bash scripts/check-migrations.sh --root "$F/pass"
# The positive case for the region check specifically. `pass` has no `tenants` table, so it
# exercises the skip path rather than the check — and a check whose only green run is a skip has
# not been shown to pass anything.
expect_ok "a correctly regioned tenants table passes" bash scripts/check-migrations.sh --root "$F/pass-region"
# A consuming repository runs this gate with `--root .`, and its vendored snapshot — fixtures that
# exist to fail included — sits inside that root. The fixture below plants one under `.paridyum/`;
# before 2026-09-28 the gate reported it as the repository's own defect, on every commit.
expect_ok "a vendored snapshot's fixtures are never the repository's migrations" bash scripts/check-migrations.sh --root "$F/pass-vendored-snapshot"
for c in \
  "fail-undeclared|a table declaring no tenancy category" \
  "fail-account-without-tenant|account_id with no tenant_id" \
  "fail-no-force|RLS ENABLEd but not FORCEd" \
  "fail-outer-only-policy|account-scoped table with an outer-only policy" \
  "fail-null-widens|operator scope granted by a NULL account (GLO-0005)" \
  "fail-regulated|a regulated column with no control framework (ANTI-0007)" \
  "fail-money-no-currency|a monetary column with no currency beside it (ADR-0033)" \
  "fail-money-float|a stored amount in floating point (ADR-0033)" \
  "fail-money-no-minor-suffix|a monetary column with no _minor suffix (ADR-0033)" \
  "fail-locale-char2|a locale column too narrow for a BCP 47 tag (STD-0010)" \
  "fail-naive-timestamp|an _at column without a time zone (C-0005, STD-0010)" \
  "fail-non-unicode-encoding|a non-Unicode database encoding (STD-0010)" \
  "fail-no-plane|a table declaring no plane (RFC-0014)" \
  "fail-tenant-in-control-plane|tenant rows declared into the control plane (RFC-0014)" \
  "fail-region-missing|a tenants table with no region column (ADR-0041 D3)" \
  "fail-region-nullable|a nullable region — a tenant with no jurisdiction (ADR-0041 D3)" \
  "fail-region-default|a DEFAULTed region — a jurisdiction acquired by accident (ADR-0041 D3)" \
  "fail-region-unconstrained|a region constrained to nothing, so a typo is a jurisdiction (ADR-0041 D3)" \
  "fail-region-not-in-registry|a region constraint disagreeing with region-registry.yaml (ADR-0035)" \
  "fail-region-not-immutable|a region with no trigger refusing an UPDATE (ADR-0041 D3)" \
  "fail-org-without-org-id|an org-scoped table with no org_id (ADR-0115 D1)" \
  "fail-org-with-tenant-id|an org-scoped table carrying tenant_id (ADR-0115 D2)" \
  "fail-org-in-control-plane|an org's own rows declared into the control plane (ADR-0117 D3)" \
  "fail-org-policy-without-predicate|an org table whose policy names no org (ADR-0115 D3)" \
  "fail-org-column-on-business-data|business data naming its owner org — the third silo (ADR-0115 D9)" \
  "fail-platform-with-org-id|a platform table carrying a discriminator's name (STD-0023 rule 1)" \
  "fail-pii-in-control-plane|a pii column in the control plane (ADR-0117 D3)" \
  "fail-unclassified-in-control-plane|an unclassified control-plane column (ADR-0117 D3)" \
  "fail-literal-locale|one tenant's languages written as every tenant's (ADR-0122 D4)" \
  "fail-tenant-locale-fields-missing|a tenant registry with no declared locales (ADR-0122 D4)"
do
  d="${c%%|*}"; label="${c#*|}"
  expect_caught "$label" bash scripts/check-migrations.sh --root "$F/$d"
done

# Regression: the default scan must not pick up this suite's own broken fixtures. It did,
# and it failed CI and the pre-commit hook on every commit until the exclusion was added.
expect_ok "root scan excludes the broken fixtures" bash scripts/check-migrations.sh

printf '\n── checkpoint hook (ADR-0025, RFC-0011) ───────────────────────────\n'
# The hook had NO tests until 2026-08-08, which is why two defects survived in it: an absent
# gate recorded `green`, and a commit made without the one blocking gate left no trace in the
# permanent record. Both are the ADR-0014 defect class -- reporting success while enforcing
# nothing -- inside the component that writes the history everything else is audited from.
#
# Driven with synthetic hook payloads against throwaway repositories, the same way ADR-0007
# says a PreToolUse hook must be tested: by driving it, not by inspecting it.
HOOK="$ROOT/plugins/paridyum-platform/hooks/checkpoint-session.sh"
hk_repo() { local d; d="$(mktemp -d)"; ( cd "$d" && git init -q . \
    && git config user.email t@t && git config user.name T && echo hi > a.txt ); printf '%s' "$d"; }
hk_gates() { # hk_gates <dir> <exit-code-for-recorded-gates>
  mkdir -p "$1/scripts" "$1/templates/infra-ci/scripts"
  for g in check-knowledge check-repo check-migrations check-commits; do
    printf '#!/bin/sh\nexit %s\n' "$2" > "$1/scripts/$g.sh"; chmod +x "$1/scripts/$g.sh"
  done
  printf '#!/bin/sh\nexit %s\n' "$2" > "$1/templates/infra-ci/scripts/check-changelog.sh"
  printf '#!/bin/sh\nexit 0\n'     > "$1/templates/infra-ci/scripts/secret-scan.sh"
  chmod +x "$1/templates/infra-ci/scripts/check-changelog.sh" "$1/templates/infra-ci/scripts/secret-scan.sh"
}
hk_run() { printf '{"cwd":"%s","session_id":"sess-test0123"}' "$1" | bash "$HOOK" 2>/dev/null; }
hk_subject() { git -C "$1" log -1 --format='%s' 2>/dev/null; }
hk_body()    { git -C "$1" log -1 --format='%b' 2>/dev/null; }
HK_DIRS=()
hk_expect() { # hk_expect <label> <dir> <grep-target: subject|body> <pattern>
  local label="$1" d="$2" where="$3" pat="$4" out
  case "$where" in subject) out="$(hk_subject "$d")" ;; *) out="$(hk_body "$d")" ;; esac
  if grep -qF -- "$pat" <<<"$out"; then ok "$label"; else bad "$label — got: ${out:-<no commit>}" "$out"; fi
}

d="$(hk_repo)"; HK_DIRS+=("$d"); hk_run "$d"
hk_expect "an absent gate is NOT reported as green"        "$d" subject "unverified (6 gate(s) absent)"
hk_expect "an unscanned commit says so, permanently"       "$d" body    "secret-scan: ABSENT — THIS COMMIT WAS NOT SCANNED"
hk_expect "the trailer claims checkpointing, not authorship" "$d" body  "checkpointed-by: sess-test0123"

d="$(hk_repo)"; HK_DIRS+=("$d"); hk_gates "$d" 0; hk_run "$d"
hk_expect "every gate passing is reported green"           "$d" subject "preserve session work — green"

d="$(hk_repo)"; HK_DIRS+=("$d"); hk_gates "$d" 1; hk_run "$d"
hk_expect "failing gates are counted, not hidden"          "$d" subject "5 gate(s) failing"

d="$(hk_repo)"; HK_DIRS+=("$d"); hk_gates "$d" 0
printf '2026-01-01T00:00:00Z\n' > "$d/.git/paridyum-concurrent"; hk_run "$d"
hk_expect "a shared tree is not reported as green"         "$d" subject "unverified (shared tree)"
hk_expect "the shared-tree commit cites its decision"      "$d" body    "See ADR-0043 for the decision and RFC-0011 for the evidence"
if [[ ! -f "$d/.git/paridyum-concurrent" ]]; then ok "the concurrency breadcrumb is consumed once"
else bad "the concurrency breadcrumb is consumed once — it survived the checkpoint that read it"; fi

# ADR-0043. A linked worktree has its own index, so a peer's files cannot be staged there no
# matter what `git add -A` does — which is the residual RFC-0011 could not close. The breadcrumb
# lives in the COMMON dir for the same reason it matters: in a linked worktree `.git` is a FILE,
# and the original path silently resolved to nothing, so a peer observed from a worktree was
# never recorded. The main-worktree path worked, which is why it went unnoticed.
d="$(hk_repo)"; HK_DIRS+=("$d"); hk_gates "$d" 0
printf '2026-01-01T00:00:00Z\n' > "$d/.git/paridyum-concurrent"; hk_run "$d"
hk_expect "a shared index downgrades the verdict"          "$d" subject "unverified (shared tree)"
hk_expect "the commit says the index was shared"           "$d" body    "MAIN worktree"

wt_base="$(hk_repo)"; HK_DIRS+=("$wt_base"); hk_gates "$wt_base" 0
( cd "$wt_base" && git add -A >/dev/null 2>&1 && git commit -qm base >/dev/null 2>&1 \
  && git worktree add -q ../wt-$$ -b feat >/dev/null 2>&1 ) || true
WT="$(dirname "$wt_base")/wt-$$"
if [[ -d "$WT" ]]; then
  HK_DIRS+=("$WT"); printf 'x\n' > "$WT/b.txt"
  printf '2026-01-01T00:00:00Z\n' > "$wt_base/.git/paridyum-concurrent"; hk_run "$WT"
  hk_expect "a linked worktree is NOT downgraded"          "$WT" subject "preserve session work — green"
  hk_expect "the commit records the isolated index"        "$WT" body    "linked worktree with its own index"
else
  bad "git worktree add failed — the ADR-0043 isolation cases did not run"
fi

d="$(hk_repo)"; HK_DIRS+=("$d"); hk_gates "$d" 0; touch "$d/.git/index.lock"; hk_run "$d"
if [[ "$(git -C "$d" rev-list --count HEAD 2>/dev/null || echo 0)" == "0" ]]; then
  ok "a held index defers the checkpoint rather than racing it"
else bad "a held index defers the checkpoint rather than racing it — it committed anyway"; fi
if [[ -f "$d/.git/paridyum-concurrent" ]]; then ok "the deferring session records that a peer exists"
else bad "the deferring session records that a peer exists — the only positive detection was discarded"; fi

for d in "${HK_DIRS[@]}"; do rm -rf "$d"; done

printf '\n── organisation gate (ADR-0012) ───────────────────────────────────\n'
# Six checks that sat `owner: tool-forge` and therefore enforced nothing, while four of them
# were one `gh api` call away. Every one PASSED against the live organisation on the day they
# were implemented — which is why the failure branches are driven from fixtures instead. A
# green check is unproven until it has been watched failing, and misconfiguring the real
# organisation to prove it is not an option.
O=tests/fixtures/org
expect_ok "a compliant organisation passes" \
  bash scripts/check-org.sh --org-json "$O/pass/org.json" --workflow-root "$O/pass"
for c in \
  "fail-no-2fa|two-factor not required (ADR-0012)" \
  "fail-base-permission|base permission is read, not none" \
  "fail-no-forking|private forking disabled, so a read-only contributor cannot contribute"
do
  d="${c%%|*}"; label="${c#*|}"
  expect_caught "$label" bash scripts/check-org.sh --org-json "$O/$d/org.json" --workflow-root "$O/pass"
done
expect_caught "a workflow declaring pull_request_target" \
  bash scripts/check-org.sh --offline --workflow-root "$O/fail-prt"
expect_warns "secrets on a pull_request workflow with no fork guard" \
  "secret_jobs_guarded_on_fork:" bash scripts/check-org.sh --offline --workflow-root "$O/warn-unguarded-secret"
# Regression: the default scan must not pick up this suite's own broken workflow fixtures.
# check-migrations.sh records the identical bug; this script reproduced it on its first run.
expect_ok "root scan excludes the broken workflow fixtures" bash scripts/check-org.sh --offline

printf '\n── repository hygiene gate ────────────────────────────────────────\n'
expect_ok "this repository passes" bash scripts/check-repo.sh

# Synthetic repository, mutated one defect at a time. Built rather than copied so the cases
# stay readable and cannot accidentally depend on this repository's contents.
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
scaffold() {
  local d="$1"; rm -rf "$d"; mkdir -p "$d/.claude" "$d/.github/workflows" "$d/.git/hooks"
  : > "$d/README.md"; : > "$d/CHANGELOG.md"; : > "$d/CODEOWNERS"; : > "$d/DECISION-DEBT.md"
  printf 'x\n' > "$d/CLAUDE.md"
  printf '{ "extraKnownMarketplaces": { "paridyum": {} }, "enabledPlugins": { "paridyum-platform@paridyum": true } }\n' > "$d/.claude/settings.json"
  printf '* text=auto eol=lf\n*.sh text eol=lf\n' > "$d/.gitattributes"
  printf 'on:\n  pull_request:\n  push:\njobs:\n  a:\n    steps:\n      - run: ./scripts/check-knowledge.sh\n' > "$d/.github/workflows/ci.yml"
  printf '#!/bin/sh\ncheck-knowledge.sh\n' > "$d/.git/hooks/pre-commit"
}
B="$TMP/base"; scaffold "$B"
expect_ok "synthetic clean repository passes" bash scripts/check-repo.sh --root "$B"

plant() { # plant <label> <mutator>
  local label="$1" fn="$2" d="$TMP/case"
  scaffold "$d"; "$fn" "$d"
  expect_caught "$label" bash scripts/check-repo.sh --root "$d"
}
m_no_claude()      { rm -f "$1/CLAUDE.md"; }
m_long_claude()    { seq 1 300 > "$1/CLAUDE.md"; }
m_no_ledger()      { rm -f "$1/DECISION-DEBT.md"; }
m_no_market()      { printf '{ "enabledPlugins": {} }\n' > "$1/.claude/settings.json"; }
m_plugin_off()     { printf '{ "extraKnownMarketplaces": { "paridyum": {} }, "enabledPlugins": { "other@x": true } }\n' > "$1/.claude/settings.json"; }
m_no_eol()         { printf '# nothing\n' > "$1/.gitattributes"; }
m_sh_not_pinned()  { printf '* text=auto eol=lf\n' > "$1/.gitattributes"; }
m_no_codeowners()  { rm -f "$1/CODEOWNERS"; }
m_no_ci()          { rm -f "$1/.github/workflows/ci.yml"; }
m_ci_no_pr()       { printf 'on:\n  push:\njobs:\n  a:\n    steps:\n      - run: ./scripts/check-knowledge.sh\n' > "$1/.github/workflows/ci.yml"; }
m_hook_wrong()     { printf '#!/bin/sh\ntrue\n' > "$1/.git/hooks/pre-commit"; }
m_no_readme()      { rm -f "$1/README.md"; }
# ADR-0059. Two mutators, because the defect had two sites with different shapes: a harness hook
# under plugins/, and the pre-commit GENERATOR at scripts/install-hooks.sh. Testing only one would
# have left the other passing, which is how the original went twelve days unnoticed.
m_gate_muted_hook() { mkdir -p "$1/plugins/p/hooks"
                      printf '#!/usr/bin/env bash\n"$R/scripts/check-repo.sh" >/dev/null || exit 1\n' \
                        > "$1/plugins/p/hooks/checkpoint.sh"; }
m_gate_muted_gen()  { mkdir -p "$1/scripts"
                      printf '#!/usr/bin/env bash\ncat <<EOF\n"\\$K/scripts/check-knowledge.sh" >/dev/null || exit 1\nEOF\n' \
                        > "$1/scripts/install-hooks.sh"; }
m_class_no_arch()  { printf 'name: svc-x\nclass: service\nowner: y\nlifecycle: beta\n' > "$1/manifest.yaml"; }
# STD-0013. ARCHITECTURE.md, API.md and RUNBOOK.md present and SOP.md missing, so the case can
# only be caught by docs.sop_present -- a repository missing everything would be caught by
# docs.required_files_present and would prove nothing about this one.
m_class_no_sop()   { printf 'name: tool-x\nclass: tool\nowner: y\nlifecycle: beta\n' > "$1/manifest.yaml"
                     : > "$1/ARCHITECTURE.md"; rm -f "$1/SOP.md"; }

# repo.ci_runs_knowledge_check used `head -1` and asserted triggers on whichever workflow grep
# returned first, so the verdict depended on filesystem ordering. It broke on infra-ci, which
# holds a `workflow_call` reusable workflow (no triggers of its own, by definition) beside its
# own self-CI (both triggers). The correct repository was reported broken. ANY workflow with
# both triggers now satisfies it.
m_two_wf_one_reusable() {
  printf 'on:\n  workflow_call:\njobs:\n  a:\n    steps:\n      - run: ./scripts/check-knowledge.sh\n' \
    > "$1/.github/workflows/reusable.yml"
}
# ADR-0056. Both shapes infra-terraform used, one mutator each; and two near-misses that must pass,
# because a check that fires on `| ssh host sendmail` or on prose would be switched off within a week.
m_ssh_bash_s()    { printf 'install:\n\tcat setup.sh | ssh root@10.0.0.9 bash -s\n' > "$1/Makefile"; }
m_ssh_redirect()  { mkdir -p "$1/scripts"; printf '#!/bin/sh\nssh -J jump root@host < scripts/setup.sh\n' > "$1/scripts/apply.sh"; }
# The real recipe's shape (infra-terraform, 2026-09-28): continued over lines, script cat into ssh.
m_ssh_cat_multi() { printf 'install:\n\t{ printf "%%s\\n" "$$tok"; cat ci/install.sh; } | ssh -o X=y \\\n\t  root@$(H) \\\n\t  "bash -c '"'"'read -r T; bash -s'"'"'"\n' > "$1/Makefile"; }
m_ssh_sendmail()  { printf 'mail:\n\tprintf "x" | ssh -J jump root@pmg "sendmail -f a b"\n' > "$1/Makefile"; }
m_ssh_in_prose()  { printf 'Never run `ssh host bash -s < x.sh` (ADR-0056).\n' > "$1/README.md"; }
plant_ok() { # plant_ok <label> <mutator> — the mutated repo must still PASS
  local label="$1" fn="$2" d="$TMP/case-ok"
  scaffold "$d"; "$fn" "$d"
  expect_ok "$label" bash scripts/check-repo.sh --root "$d"
}
plant_ok "data piped to a remote command is not host configuration (ADR-0056)" m_ssh_sendmail
plant_ok "the shape described in prose is not caught (ADR-0056)"               m_ssh_in_prose
plant_ok "a reusable workflow beside a triggered one still passes" m_two_wf_one_reusable

plant "a script piped to ssh bash -s (ADR-0056)"      m_ssh_bash_s
plant "a script redirected into ssh (ADR-0056)"       m_ssh_redirect
plant "a script cat into ssh, over continued lines (ADR-0056)" m_ssh_cat_multi
plant "SOP.md absent from a classed repo (STD-0013)" m_class_no_sop
plant "CLAUDE.md absent"                       m_no_claude
plant "CLAUDE.md over the line budget"         m_long_claude
plant "DECISION-DEBT.md absent (ADR-0011)"     m_no_ledger
plant "marketplace not declared (ADR-0007)"    m_no_market
plant "marketplace declared, plugin disabled"  m_plugin_off
plant "line endings not normalised (ADR-0006)" m_no_eol
plant "shell scripts not pinned to LF"         m_sh_not_pinned
plant "no CODEOWNERS (ADR-0012)"               m_no_codeowners
plant "no CI workflow (ADR-0010)"              m_no_ci
plant "CI does not run on pull_request"        m_ci_no_pr
plant "pre-commit hook skips the gate"         m_hook_wrong
plant "README.md absent (STD-0002)"            m_no_readme
plant "classed repo without ARCHITECTURE.md"   m_class_no_arch
plant "a harness hook mutes a gate (ADR-0059)" m_gate_muted_hook
plant "install-hooks.sh mutes a gate (ADR-0059)" m_gate_muted_gen

# --- the four test.* checks moved off tool-forge (STD-0008) ------------------
# The scaffold has no tests, so every one of these reports n/a there — which is correct and is
# also exactly how a check comes to be counted as implemented while enforcing nothing. Each
# case therefore builds the condition that makes the check APPLY before planting the defect.
TESTING_OK=$'# Testing\n\n## Tiers\n\nEverything runs in `pr`. TST-0001 unit, TST-0010 regression.\n\n## Not tested\n\nThe generator.\n\n## Quarantine\n\nNone.\n\n## Evidence\n\njunit.xml, kept 30 days.\n'
withtests() { # a scaffold that has tests, so the STD-0008 checks are in scope
  local d="$1"; scaffold "$d"; mkdir -p "$d/tests" "$d/.config"; : > "$d/tests/it.rs"
  printf '%s' "$TESTING_OK" > "$d/TESTING.md"
  printf '[profile.default]\nretries = 0\n\n[profile.ci]\nretries = 2\n' > "$d/.config/nextest.toml"
  cp "$ROOT/test-registry.yaml" "$d/test-registry.yaml"
}
WT="$TMP/withtests"; withtests "$WT"
expect_ok "a repository with tests, TESTING.md and nextest.toml passes" \
  bash scripts/check-repo.sh --root "$WT"

plantt() { local label="$1" fn="$2" d="$TMP/ct"; withtests "$d"; "$fn" "$d"
  expect_caught "$label" bash scripts/check-repo.sh --root "$d"; }

m_no_testing_md() { rm -f "$1/TESTING.md"; }
m_no_quarantine() { python3 -c "
import pathlib,sys
p=pathlib.Path(sys.argv[1])/'TESTING.md'
p.write_text(p.read_text().replace('## Quarantine\n\nNone.\n',''))" "$1"; }
m_unknown_tst()   { python3 -c "
import pathlib,sys
p=pathlib.Path(sys.argv[1])/'TESTING.md'
p.write_text(p.read_text().replace('TST-0001','TST-9999'))" "$1"; }
m_flaky_thin()    { python3 -c "
import pathlib,sys
p=pathlib.Path(sys.argv[1])/'TESTING.md'
p.write_text(p.read_text().replace('## Quarantine\n\nNone.\n',
  '## Quarantine\n\n| Test | Owner | First seen | Rate | Tracking | Expires |\n| - | - | - | - | - | - |\n| flaky_login | | | | | |\n'))" "$1"; }
m_flaky_full()    { python3 -c "
import pathlib,sys
p=pathlib.Path(sys.argv[1])/'TESTING.md'
p.write_text(p.read_text().replace('## Quarantine\n\nNone.\n',
  '## Quarantine\n\n| Test | Owner | First seen | Rate | Tracking | Expires |\n| - | - | - | - | - | - |\n| flaky_login | platform | 2026-08-01 | 4% | PDY-142 | 2026-09-01 |\n'))" "$1"; }
# The boundary case, and the one that found the `-ge 5` defect. Everything present EXCEPT the
# expiry — five filled cells of six. FLAKY-003 is the rule that an entry expires and that
# "silence is not renewal", so an entry with no expiry is precisely the permanent quarantine
# the standard exists to prevent, and it was accepted. A reject-case one column short of the
# accept-case is also what makes the accept-case non-vacuous: if this check ever becomes a
# no-op again, this fails while `m_flaky_full` stays green.
m_flaky_no_expiry() { python3 -c "
import pathlib,sys
p=pathlib.Path(sys.argv[1])/'TESTING.md'
p.write_text(p.read_text().replace('## Quarantine\n\nNone.\n',
  '## Quarantine\n\n| Test | Owner | First seen | Rate | Tracking | Expires |\n| - | - | - | - | - | - |\n| flaky_login | platform | 2026-08-01 | 4% | PDY-142 | |\n'))" "$1"; }
m_default_retry() { printf '[profile.default]\nretries = 2\n' > "$1/.config/nextest.toml"; }

plantt "a repository with tests and no TESTING.md"        m_no_testing_md
plantt "TESTING.md missing a required section"            m_no_quarantine
plantt "TESTING.md naming a test type that does not exist" m_unknown_tst
# There is deliberately no "unknown tier" case. The check does not validate tier names: reading
# every backticked word in the section as a tier made `push` — a GitHub event in this
# repository's own TESTING.md — an undeclared tier, and took four unrelated fixtures down with it.
plantt "a quarantine entry with no owner or expiry"       m_flaky_thin
plantt "a quarantine entry with no expiry (FLAKY-003)"    m_flaky_no_expiry
plantt "retries set in the default nextest profile"       m_default_retry

# The other direction for the flaky check. A rule that rejects a COMPLETE entry would make
# quarantining impossible, which is worse than not checking it at all — the entry would simply
# never be written and the test would be deleted instead.
FQ="$TMP/flakyok"; withtests "$FQ"; m_flaky_full "$FQ"
expect_ok "a fully-filled quarantine entry is accepted" bash scripts/check-repo.sh --root "$FQ"

# ADR-0025. The scaffold has no plugin and no remote, so these two need their own fixture:
# a repository that looks complete except for the durability guarantee.
adr25() {
  local d="$1"; scaffold "$d"
  mkdir -p "$d/plugins/paridyum-platform/hooks"
  printf '#!/bin/sh\ntrue\n' > "$d/plugins/paridyum-platform/hooks/checkpoint-session.sh"
  chmod +x "$d/plugins/paridyum-platform/hooks/checkpoint-session.sh"
  printf '#!/bin/sh\ntrue\n' > "$d/plugins/paridyum-platform/hooks/capture-open-loops.sh"
  cat > "$d/plugins/paridyum-platform/hooks/hooks.json" <<'HJ'
{ "hooks": { "Stop": [ { "hooks": [
  { "type": "command", "command": "${CLAUDE_PLUGIN_ROOT}/hooks/capture-open-loops.sh" },
  { "type": "command", "command": "${CLAUDE_PLUGIN_ROOT}/hooks/checkpoint-session.sh" }
] } ] } }
HJ
}
d25="$TMP/adr25"; adr25 "$d25"
expect_ok "checkpoint hook registered in the right order" bash scripts/check-repo.sh --root "$d25"

plant25() { local label="$1" fn="$2" d="$TMP/c25"; adr25 "$d"; "$fn" "$d"
  expect_caught "$label" bash scripts/check-repo.sh --root "$d"; }
m_cp_unregistered() { python3 - "$1" <<'PY'
import json,sys,pathlib
p=pathlib.Path(sys.argv[1])/"plugins/paridyum-platform/hooks/hooks.json"
d=json.loads(p.read_text())
d["hooks"]["Stop"][0]["hooks"]=[h for h in d["hooks"]["Stop"][0]["hooks"] if "checkpoint" not in h["command"]]
p.write_text(json.dumps(d))
PY
}
m_cp_wrong_order() { python3 - "$1" <<'PY'
import json,sys,pathlib
p=pathlib.Path(sys.argv[1])/"plugins/paridyum-platform/hooks/hooks.json"
d=json.loads(p.read_text()); d["hooks"]["Stop"][0]["hooks"].reverse(); p.write_text(json.dumps(d))
PY
}
m_cp_not_exec()   { chmod -x "$1/plugins/paridyum-platform/hooks/checkpoint-session.sh"; }

plant25 "checkpoint hook not registered (ADR-0025)"        m_cp_unregistered
plant25 "checkpoint runs before capture-open-loops"        m_cp_wrong_order
plant25 "checkpoint hook registered but not executable"    m_cp_not_exec

# ADR-0026. The router's routes must resolve, or an agent follows a link, finds nothing, and
# infers — the one failure the entrypoint exists to prevent.
router() {
  local d="$1"; scaffold "$d"
  mkdir -p "$d/standards" "$d/plugins/paridyum-platform/skills/design-change"
  : > "$d/standards/STD-0001-testing.md"
  : > "$d/plugins/paridyum-platform/skills/design-change/SKILL.md"
  printf '# Entry\n\nRead [STD-0001](standards/STD-0001-testing.md) and the `design-change` skill.\n' > "$d/CLAUDE.md"
}
dr="$TMP/router"; router "$dr"
expect_ok "router with resolving routes passes" bash scripts/check-repo.sh --root "$dr"

plantr() { local label="$1" fn="$2" d="$TMP/cr"; router "$d"; "$fn" "$d"
  expect_caught "$label" bash scripts/check-repo.sh --root "$d"; }
m_dead_link()   { sed -i.bak 's|standards/STD-0001-testing.md|standards/no-such-standard.md|' "$1/CLAUDE.md"; rm -f "$1/CLAUDE.md.bak"; }
m_dead_skill()  { sed -i.bak 's|`design-change` skill|`no-such-skill` skill|' "$1/CLAUDE.md"; rm -f "$1/CLAUDE.md.bak"; }
m_router_bloat(){ { cat "$1/CLAUDE.md"; seq 1 200; } > "$1/c.tmp" && mv "$1/c.tmp" "$1/CLAUDE.md"; }

plantr "router links to a path that does not exist"  m_dead_link
plantr "router names a skill with no SKILL.md"       m_dead_skill
plantr "router grew back past its line budget"       m_router_bloat

# The failure the other two router checks miss: an inlined SUMMARY of a standard fits inside the
# budget and leaves every link resolving. Found two real instances in the router on its first run.
m_restates() {
  mkdir -p "$1/standards"
  printf 'Structure follows responsibility and never size, and there is no line limit anywhere in this standard.\n' \
    > "$1/standards/STD-0001-testing.md"
  printf 'Structure follows responsibility and never size, and there is no line limit anywhere in this standard.\n' \
    >> "$1/CLAUDE.md"
}
m_quoted()   {
  mkdir -p "$1/standards"
  printf 'Structure follows responsibility and never size, and there is no line limit anywhere in this standard.\n' \
    > "$1/standards/STD-0001-testing.md"
  printf '> Structure follows responsibility and never size, and there is no line limit anywhere in this standard.\n' \
    >> "$1/CLAUDE.md"
}
plantr "router restates a standard verbatim" m_restates
dq="$TMP/quoted"; router "$dq"; m_quoted "$dq"
expect_ok "the same passage is fine when marked as a quotation" bash scripts/check-repo.sh --root "$dq"

printf '\n── pull request gate (STD-0007) ───────────────────────────────────\n'
# A real git repository with a real branch: the budget is measured on a branch diff against a
# merge base, so a fixture directory cannot exercise it.
PR="$TMP/pr"; mkdir -p "$PR"
(
  cd "$PR" && git init -q -b main . \
    && git config user.email t@t.t && git config user.name T && git config commit.gpgsign false \
    && echo base > a.txt && git add -A && git commit -qm "chore: base" \
    && git checkout -q -b feature \
    && seq 1 500 > big.rs \
    && seq 1 900 > Cargo.lock \
    && seq 1 900 > pnpm-lock.yaml && mkdir -p web && seq 1 900 > web/package-lock.json \
    && mkdir -p tests/fixtures && seq 1 900 > tests/fixtures/f.txt \
    && git add -A && git commit -qm "feat: over budget"
) >/dev/null 2>&1
printf 'Implements REQ-NNNN\n' > "$PR/body-plain.md"
printf 'Implements REQ-NNNN\nReview-budget-exception: mechanical rename\n' > "$PR/body-exc.md"

expect_caught "500-line branch diff exceeds the 400-line budget" \
  bash scripts/check-pr.sh --root "$PR" --base main --body "$PR/body-plain.md"
expect_ok "the same diff passes with Review-budget-exception" \
  bash scripts/check-pr.sh --root "$PR" --base main --body "$PR/body-exc.md"

# Regression: a 900-line lockfile and a 900-line fixture must NOT count. If they did, every
# dependency bump would fail the budget and the check would be switched off within a week
# (ADR-0008). The count must read exactly 500, not 4100. pnpm-lock.yaml and package-lock.json are
# in the fixture because a `*.lock` pattern alone missed them while its comment said otherwise.
expect_ok "lockfiles and fixtures are excluded from the count" \
  bash -c "bash scripts/check-pr.sh --root '$PR' --base main --body '$PR/body-exc.md' 2>&1 | grep -q '500 reviewable line'"

(cd "$PR" && git checkout -q -b small main && seq 1 40 > small.rs && git add -A && git commit -qm "feat: small") >/dev/null 2>&1
expect_ok "a 40-line branch diff passes" \
  bash scripts/check-pr.sh --root "$PR" --base main --body "$PR/body-plain.md"

# n/a, not a silent pass. This is the normal state of a repository committing straight to main.
expect_ok "no branch diff reports n/a rather than passing" \
  bash -c "bash scripts/check-pr.sh --root '$PR' --base small 2>&1 | grep -q 'n/a'"

printf '\n── hook reachability (ADR-0027) ────────────────────────────────────\n'
# The failure these cover was invisible for a whole session: the plugin declared and enabled while
# the marketplace had never been added, so layer 1 never denied an edit and the ADR-0025 checkpoint
# never fired. A fake HOME is used so the machine's real install state cannot mask the case.
HR="$TMP/hr"; cp -R . "$HR" >/dev/null 2>&1; rm -rf "$HR/.git"
# `.claude/` present but empty — a WORKSTATION with neither channel satisfied, which is the
# condition this section is about. It was previously an empty directory, which is a CI runner, and
# the two were indistinguishable to the check: it errored on both, so main went red for three runs
# on a condition no runner can satisfy. Making the fixture explicitly a workstation is what lets
# the n/a case below assert the other half without either one weakening the other.
FAKE_HOME="$TMP/fakehome"; mkdir -p "$FAKE_HOME/.claude"

expect_caught "neither channel present on a workstation means no hooks load" \
  env HOME="$FAKE_HOME" bash "$HR/scripts/check-repo.sh" --root "$HR"

# The other half. An `error` for a condition the environment CANNOT meet leaves main permanently
# red with no legal remedy — the ADR-0014 trap that knowledge-check.yml's identity step already
# documents. Asserted next to its opposite deliberately: "make CI green" is one HOME guard away
# from "the check never fires anywhere", and that would silently drop ADR-0007 layer 1 entirely.
CI_HOME="$TMP/cihome"; mkdir -p "$CI_HOME"
expect_ok "the same repository is n/a on a runner with no ~/.claude" \
  env HOME="$CI_HOME" bash "$HR/scripts/check-repo.sh" --root "$HR"

# Registration is not enough. ADR-0022 already found a hook registered and not executable —
# configured, silent, indistinguishable from working — so this asserts the executable bit too.
python3 - "$HR" <<'PY'
import json, pathlib, sys
p = pathlib.Path(sys.argv[1])/".claude/settings.json"
d = json.loads(p.read_text())
H = "$CLAUDE_PROJECT_DIR/plugins/paridyum-platform/hooks"
d["hooks"] = {
  "PreToolUse": [{"matcher": "Write", "hooks": [{"type": "command", "command": f"{H}/guard-locked-decisions.sh"}]}],
  "Stop": [{"hooks": [{"type": "command", "command": f"{H}/checkpoint-session.sh"}]}],
}
p.write_text(json.dumps(d, indent=2))
PY
expect_ok "settings.json alone satisfies reachability" \
  env HOME="$FAKE_HOME" bash "$HR/scripts/check-repo.sh" --root "$HR"
chmod -x "$HR/plugins/paridyum-platform/hooks/checkpoint-session.sh"
expect_caught "a registered hook that is not executable" \
  env HOME="$FAKE_HOME" bash "$HR/scripts/check-repo.sh" --root "$HR"
chmod +x "$HR/plugins/paridyum-platform/hooks/checkpoint-session.sh"

# The pin is a silent staleness vector: correct in git, stale on disk. Asserted on a real HARNESS
# change, not on commit distance — the first version of this check warned on any gap, which fires
# on every commit that does not touch the plugin, and a check that fires constantly gets disabled.
SP="$TMP/pin"; cp -R . "$SP" >/dev/null 2>&1
# ITS OWN REPOSITORY, ALWAYS, before anything commits in it. `cp -R .` copies `.git` as it finds it,
# and in a git WORKTREE `.git` is a file pointing at the real repository — so this case once committed
# "feat: change a hook" onto the branch being tested, sweeping in every uncommitted change, and the
# next push published it. A fresh repository with the copied tree as its first commit is what the
# case needs (a HEAD~1 before the harness change) and cannot reach anything outside $TMP.
rm -rf "$SP/.git"
# A remote too, as every real checkout has: check-repo.sh errors on a repository with none (ADR-0025),
# and that error is not what this case tests. The URL is never contacted.
( cd "$SP" && git init -q -b main . && git remote add origin https://example.invalid/knowledge.git \
    && git add -A \
    && git -c user.email=t@t.t -c user.name=T -c commit.gpgsign=false \
         commit -q --no-verify -m "fixture: the checkout as copied" ) >/dev/null 2>&1
SH="$TMP/stalehome"; mkdir -p "$SH/.claude/plugins"
printf '{"paridyum":{}}' > "$SH/.claude/plugins/known_marketplaces.json"
(
  cd "$SP" || exit 1
  echo "# a real harness change" >> plugins/paridyum-platform/hooks/checkpoint-session.sh
  git add -A && git -c user.email=t@t.t -c user.name=T -c commit.gpgsign=false \
    commit -q --no-verify -m "feat: change a hook"
) >/dev/null 2>&1
python3 - "$SH" "$(git -C "$SP" rev-parse HEAD~1 2>/dev/null)" <<'PY2'
import json, pathlib, sys
pathlib.Path(sys.argv[1], ".claude/plugins/installed_plugins.json").write_text(
    json.dumps({"plugins": {"paridyum-platform@paridyum": [{"gitCommitSha": sys.argv[2]}]}}))
PY2
expect_warns "a pin behind a real harness change is reported" "harness file" \
  env HOME="$SH" bash "$SP/scripts/check-repo.sh" --root "$SP"

# The other direction, which is the one that made the first version noise: a gap that does not
# touch the harness must stay silent.
python3 - "$SH" "$(git -C "$SP" rev-parse HEAD 2>/dev/null)" <<'PY3'
import json, pathlib, sys
pathlib.Path(sys.argv[1], ".claude/plugins/installed_plugins.json").write_text(
    json.dumps({"plugins": {"paridyum-platform@paridyum": [{"gitCommitSha": sys.argv[2]}]}}))
PY3
expect_ok "a pin with no harness change stays silent" \
  env HOME="$SH" bash "$SP/scripts/check-repo.sh" --root "$SP"

printf '\n── checkpoint bulk guard (ADR-0025) ───────────────────────────────\n'
# `git add -A` stages everything, and a large file is permanent once pushed — git keeps every
# object and C-0004 denies the rewrite that would remove it. The guard must exclude the bulk and
# still preserve the work; refusing the whole commit would trade a bloated repo for a lost session.
CP="$TMP/ckpt"; mkdir -p "$CP/plugins/paridyum-platform/hooks"
cp plugins/paridyum-platform/hooks/checkpoint-session.sh "$CP/plugins/paridyum-platform/hooks/"
(
  cd "$CP" && git init -q -b main . \
    && git config user.email t@t.t && git config user.name T && git config commit.gpgsign false \
    && echo seed > seed.txt && git add -A && git commit -qm "chore: seed"
) >/dev/null 2>&1
head -c 6000000 /dev/urandom > "$CP/big.bin"
echo "the actual work" > "$CP/feature.txt"
printf '{"cwd":"%s","session_id":"bulk-regression"}' "$CP" \
  | bash "$CP/plugins/paridyum-platform/hooks/checkpoint-session.sh" >/dev/null 2>&1

expect_ok "a >5MB file is kept out of the checkpoint commit" \
  bash -c "! git -C '$CP' show --name-only --format= HEAD | grep -q 'big.bin'"
expect_ok "the real work in the same checkpoint is still committed" \
  bash -c "git -C '$CP' show --name-only --format= HEAD | grep -q 'feature.txt'"
expect_ok "the excluded file is left on disk, not deleted" test -f "$CP/big.bin"

# ADR-0133 decision 9. The guard measured the file on disk, so a Git LFS asset — full-size on disk,
# a small pointer in the index — was left out by the guard whose message recommends LFS. A clean
# filter stands in for git-lfs, which the suite cannot assume is installed; it is the mechanism LFS
# itself uses to put the pointer in the index. quotePath is pinned because a machine that turned it
# off would hide the second defect: git quoted a non-ASCII path, and the quoted name slipped past.
CL="$TMP/ckpt-lfs"; mkdir -p "$CL/plugins/paridyum-platform/hooks"
cp plugins/paridyum-platform/hooks/checkpoint-session.sh "$CL/plugins/paridyum-platform/hooks/"
(
  cd "$CL" && git init -q -b main . \
    && git config user.email t@t.t && git config user.name T && git config commit.gpgsign false \
    && git config core.quotePath true \
    && git config filter.pointer.clean 'cat >/dev/null; printf "pointer\n"' \
    && printf '*.asset filter=pointer\n' > .gitattributes \
    && echo seed > seed.txt && git add -A && git commit -qm "chore: seed"
) >/dev/null 2>&1
for f in level.asset scratch.bin scratch-ø.bin; do head -c 6000000 /dev/urandom > "$CL/$f"; done
printf '{"cwd":"%s","session_id":"bulk-lfs"}' "$CL" \
  | bash "$CL/plugins/paridyum-platform/hooks/checkpoint-session.sh" >/dev/null 2>&1

expect_ok "an LFS-style asset is committed as its pointer" \
  git -C "$CL" cat-file -e HEAD:level.asset
expect_ok "a large file with no filter is still left out" \
  bash -c "! git -C '$CL' cat-file -e HEAD:scratch.bin 2>/dev/null"
expect_ok "a large file is left out whatever its name" \
  bash -c "! git -C '$CL' cat-file -e 'HEAD:scratch-ø.bin' 2>/dev/null"

printf '\n── regression gate (STD-0008) ─────────────────────────────────────\n'
# A real git repository, because the rule is about a commit rather than about a tree: the
# question is whether THIS fix changed a test, and that is only answerable from a diff.
RG="$TMP/regression"; mkdir -p "$RG"
gitc() { git -C "$RG" -c user.email=t@t.t -c user.name=T -c commit.gpgsign=false "$@"; }
(
  cd "$RG" && git init -q -b main . && mkdir -p src tests
  echo "fn a() {}" > src/lib.rs && echo "fn t() {}" > tests/it.rs
) >/dev/null 2>&1
gitc add -A >/dev/null 2>&1; gitc commit -qm "chore: base" >/dev/null 2>&1

rgcommit() { # rgcommit <subject> <body-or-empty>  — mutates whatever the caller staged
  gitc add -A >/dev/null 2>&1
  if [[ -n "$2" ]]; then gitc commit -qm "$1" -m "$2" >/dev/null 2>&1
  else gitc commit -qm "$1" >/dev/null 2>&1; fi
}

echo "fn b() {}" >> "$RG/src/lib.rs"; rgcommit "fix: source only, no test" ""
expect_caught "a fix commit touching no test" \
  bash templates/infra-ci/scripts/check-regression.sh --root "$RG" HEAD~1..HEAD

echo "fn u() {}" >> "$RG/tests/it.rs"; echo "fn c() {}" >> "$RG/src/lib.rs"
rgcommit "fix: source and a test" ""
expect_ok "a fix commit that also changes a test" \
  bash templates/infra-ci/scripts/check-regression.sh --root "$RG" HEAD~1..HEAD

# The path heuristic cannot see a Rust unit test, which lives beside the code. Without the
# attribute scan every correctly-tested Rust fix would fail, and the check would be off in a week.
printf '#[cfg(test)]\nmod tests { #[test] fn r() {} }\n' >> "$RG/src/lib.rs"
rgcommit "fix: with an inline unit test" ""
expect_ok "a fix whose test is a #[cfg(test)] module beside the code" \
  bash templates/infra-ci/scripts/check-regression.sh --root "$RG" HEAD~1..HEAD

echo "fn d() {}" >> "$RG/src/lib.rs"
rgcommit "fix: genuinely untestable" "Regression-test-exception: reproduces only under a specific scheduler interleaving"
expect_ok "a fix with a recorded Regression-test-exception" \
  bash templates/infra-ci/scripts/check-regression.sh --root "$RG" HEAD~1..HEAD

# A trailer with no reason is the obvious way to defeat this, so it must not work.
echo "fn e() {}" >> "$RG/src/lib.rs"
rgcommit "fix: empty excuse" "Regression-test-exception:"
expect_caught "an exception trailer with no stated reason" \
  bash templates/infra-ci/scripts/check-regression.sh --root "$RG" HEAD~1..HEAD

echo "fn f() {}" >> "$RG/src/lib.rs"; rgcommit "feat: not a fix" ""
expect_ok "a feat commit is out of scope" \
  bash templates/infra-ci/scripts/check-regression.sh --root "$RG" HEAD~1..HEAD

# ADR-0016's lesson, inherited deliberately: an explicit range examining nothing reported
# success in CI on every push for the life of that workflow.
expect_caught "an explicit range that resolves to no commits" \
  bash templates/infra-ci/scripts/check-regression.sh --root "$RG" HEAD..HEAD

printf '\n── errata on a locked artifact (ADR-0031) ─────────────────────────\n'
# The only hole ever opened in the immutability control, so every way through it is planted.
# A real git repository: the rule is "byte-identical against the COMMITTED version", which a
# fixture directory cannot express.
ER="$TMP/errata"; mkdir -p "$ER/adr" "$ER/scripts" "$ER/plugins/paridyum-platform/hooks"
cp scripts/check-knowledge.sh "$ER/scripts/"
cp artifact-registry.yaml class-registry.yaml knowledge.schema.json check-registry.yaml \
   framework-registry.yaml test-registry.yaml test-registry.schema.json "$ER/" 2>/dev/null
cp -R scripts/lib "$ER/scripts/" 2>/dev/null
cp plugins/paridyum-platform/hooks/guard-locked-decisions.sh \
   plugins/paridyum-platform/hooks/capture-open-loops.sh "$ER/plugins/paridyum-platform/hooks/" 2>/dev/null
cat > "$ER/adr/ADR-0001-x.md" <<'AD'
---
id: ADR-0001
title: A locked artifact used as an errata fixture
type: adr
status: locked
created: 2026-08-08
locked_at: 2026-08-08
supersedes: null
superseded_by: null
scope: [org]
applies_to: ["*"]
tags: [fixture]
breaking: false
enforced_by: []
---

# ADR-0001 — fixture

## Decision

The body of this artifact is the thing that must stay byte-identical.
AD
egit() { git -C "$ER" -c user.email=t@t.t -c user.name=T -c commit.gpgsign=false "$@"; }
(cd "$ER" && git init -q -b main .) >/dev/null 2>&1
egit add -A >/dev/null 2>&1; egit commit -qm "chore: fixture" >/dev/null 2>&1

# The gate is run from the fixture, so ROOT resolves there and section 3 diffs against its HEAD.
# Asserted on the MESSAGE, not the exit code, and that is not a stylistic choice. The fixture
# copies this repository's registries, which cross-reference artifacts it does not contain, so
# the gate exits non-zero for unrelated reasons on EVERY case. A first version of this block
# used expect_caught and all four negative cases "passed" — they would have passed with section
# 3 deleted. Section 3 emits exactly one of the two lines per file, so matching one proves the
# other did not fire.
PERMIT='errata appended, body byte-identical'
REJECT='is LOCKED but has content changes'
eplant() { # eplant <label> <permit|reject> <mutator>
  local label="$1" mode="$2" fn="$3" pat
  [[ "$mode" == permit ]] && pat="$PERMIT" || pat="$REJECT"
  egit checkout -q -- adr/ADR-0001-x.md 2>/dev/null
  "$fn"
  expect_warns "$label" "$pat" bash "$ER/scripts/check-knowledge.sh"
}
ERRATUM=$'\n## Errata\n\n### 2026-08-08 — the body says fixture and means example\n\nSays: fixture. Wrong: it is an example. Intended: example.\n'
m_er_append()  { printf '%s' "$ERRATUM" >> "$ER/adr/ADR-0001-x.md"; }
m_er_body()    { printf '%s' "$ERRATUM" >> "$ER/adr/ADR-0001-x.md"
                 python3 -c "
import pathlib,sys
p=pathlib.Path(sys.argv[1]); p.write_text(p.read_text().replace('byte-identical','editable',1))" "$ER/adr/ADR-0001-x.md"; }
m_er_midfile() { python3 -c "
import pathlib,sys
p=pathlib.Path(sys.argv[1]); s=p.read_text()
p.write_text(s.replace('## Decision', '## Errata\n\n### 2026-08-08 — smuggled\n\nSays: x. Wrong: y. Intended: z.\n\n## Decision',1))" "$ER/adr/ADR-0001-x.md"; }
m_er_empty()   { printf '\n## Errata\n' >> "$ER/adr/ADR-0001-x.md"; }

eplant "an appended, dated erratum is permitted"        permit m_er_append
eplant "a body edit smuggled in alongside an erratum"   reject m_er_body
eplant "an errata section inserted mid-file"            reject m_er_midfile
eplant "an errata heading that adds no erratum"         reject m_er_empty

# Append-only. Once errata are committed, shortening or rewriting them must fail — otherwise
# the archive is back where it started, one level down (ADR-0031 §4).
egit checkout -q -- adr/ADR-0001-x.md 2>/dev/null
m_er_append; egit add -A >/dev/null 2>&1; egit commit -qm "docs: erratum" >/dev/null 2>&1
python3 -c "
import pathlib,sys
p=pathlib.Path(sys.argv[1]); p.write_text(p.read_text().replace('Wrong: it is an example.','',1))" "$ER/adr/ADR-0001-x.md"
expect_warns "an existing erratum rewritten after the fact" "$REJECT" bash "$ER/scripts/check-knowledge.sh"
egit checkout -q -- adr/ADR-0001-x.md 2>/dev/null
printf '\n### 2026-08-09 — a second erratum\n\nSays: a. Wrong: b. Intended: c.\n' >> "$ER/adr/ADR-0001-x.md"
expect_warns "a second erratum appended below the first" "$PERMIT" bash "$ER/scripts/check-knowledge.sh"

printf '\n── PreToolUse guard, errata branch (ADR-0031) ─────────────────────\n'
# ADR-0007 layer 1 is the enforcement CLAUDE.md calls impossible to reason around, and until now
# nothing in this suite drove it — TESTING.md lists these hooks under "not tested" for that
# reason. ADR-0031 added a branch that PERMITS an edit to a locked artifact, which is the one
# kind of hook change that can only fail open, so it is the wrong one to leave unexercised.
# Synthetic tool-call JSON, which is the only way to test a hook: it reads stdin, not a repo.
GUARD="plugins/paridyum-platform/hooks/guard-locked-decisions.sh"
if ! command -v jq >/dev/null 2>&1; then
  printf '! jq unavailable — guard cases skipped, and the guard itself is OFF without it\n'
else
  LOCKED_FIXTURE="$ROOT/adr/ADR-0029-test-types-are-registry-data.md"
  # A permitted call produces NO OUTPUT and exits 0 — silence is the allow signal. Piping
  # straight into jq cannot express that: jq with empty stdin emits nothing and its `//`
  # default never fires, so every allow case reads as the empty string. The first version of
  # this helper did exactly that and reported three false failures.
  guard_decision() { # guard_decision <old_string> <new_string>
    local out
    out="$(jq -n --arg p "$LOCKED_FIXTURE" --arg o "$1" --arg n "$2" \
      '{tool_name:"Edit",tool_input:{file_path:$p,old_string:$o,new_string:$n}}' | bash "$GUARD")"
    [[ -n "${out//[[:space:]]/}" ]] || { printf 'allow'; return; }
    jq -r '.hookSpecificOutput.permissionDecision // "allow"' <<<"$out"
  }
  expect_guard() { # expect_guard <label> <allow|deny> <old> <new>
    local label="$1" want="$2" got; got="$(guard_decision "$3" "$4")"
    [[ "$got" == "$want" ]] && ok "$label" || bad "$label — expected $want, got $got"
  }
  NEW_SECTION=$'## Related\n\n## Errata\n\n### 2026-08-09 — t\n\nSays: a. Wrong: b. Intended: c.'
  expect_guard "appending a new errata section is permitted" allow \
    "## Related" "$NEW_SECTION"
  expect_guard "appending below an existing erratum is permitted" allow \
    $'## Errata\n' $'## Errata\n\n### 2026-08-09 — t\n'
  expect_guard "editing the body is still denied" deny \
    "**LOCKED** as of 2026-08-08." "**LOCKED** as of 2026-08-08. Amended."
  # The case that makes the branch safe: "additive at the anchor" must mean the old text is
  # still there. Replacing a heading with an errata section is not an erratum, and a guard that
  # only looked for the word "Errata" would wave it through.
  expect_guard "replacing text while adding errata is denied" deny \
    "## Related" $'## Errata\n\n### 2026-08-09 — t'
  expect_guard "the supersession edit is still permitted" allow \
    "status: locked" "status: superseded"
  # Regression: a Write is a whole-file overwrite and can never be an erratum, whatever it says.
  wout="$(jq -n --arg p "$LOCKED_FIXTURE" '{tool_name:"Write",tool_input:{file_path:$p,content:"## Errata\n"}}' | bash "$GUARD")"
  wdec=allow; [[ -n "${wout//[[:space:]]/}" ]] && wdec="$(jq -r '.hookSpecificOutput.permissionDecision // "allow"' <<<"$wout")"
  [[ "$wdec" == "deny" ]] && ok "a Write to a locked artifact is denied even if it says Errata" \
    || bad "a Write to a locked artifact is denied even if it says Errata — got $wdec"
fi

printf '\n── knowledge gate ─────────────────────────────────────────────────\n'
expect_ok "this repository passes" bash scripts/check-knowledge.sh

# Until 2026-08-07 this section was that one line, and mutation testing showed what that was
# worth: disabling "a bet may never be locked" — the load-bearing rule of ADR-0022 — left the
# suite green. check-knowledge.sh owns 20 implemented checks, the largest single owner, and NONE
# of them was regression-protected. Every defect test run while building them was ad-hoc in a
# shell and never captured, which is the difference between having watched a check fail once and
# knowing it still fails.
#
# A copy of the repository, mutated one defect at a time. .git is removed: it makes the copy slow,
# and the two git-dependent checks (locked-artifact immutability, supersession detected in a diff)
# are guarded by `git rev-parse` and skip cleanly. Those two remain uncovered here — stated rather
# than left for the next mutation run to discover.
KB="$TMP/kb"
cp -R . "$KB" >/dev/null 2>&1
rm -rf "$KB/.git"

kplant() { # kplant <label> <file> <old> <new>
  local label="$1" f="$KB/$2" old="$3" new="$4"
  cp "$f" "$f.orig"
  if ! python3 - "$f" "$old" "$new" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1]); s = p.read_text()
if sys.argv[2] not in s:
    sys.exit(3)
p.write_text(s.replace(sys.argv[2], sys.argv[3], 1))
PY
  then bad "$label — could not plant (anchor missing)"; mv "$f.orig" "$f"; return; fi
  expect_caught "$label" bash "$KB/scripts/check-knowledge.sh"
  mv "$f.orig" "$f"
}

# THE STATUS ANCHORS MOVED TO A LIVE BET ON 2026-08-17, and the reason is worth keeping. These
# two cases anchored on `status: accepted` in BET-0001, which BET-0006 superseded — so from that
# commit onward the anchor was absent, kplant reported "could not plant", and the two cases had
# been FAILING SILENTLY in the sense that matters: the suite was red, and the redness said
# nothing about the rule under test. A fixture that names a mutable artifact inherits that
# artifact's lifecycle, and a bet is the type most certain to be superseded.
#
# BET-0001 keeps the two cases whose anchors are frozen with it: a superseded artifact is
# immutable (section 3), so `trigger:` and its `review_date` can never move again.
B1=bets/BET-0001-the-wedge-is-the-solo-saas-founder.md
B_LIVE=bets/BET-0005-paridyum-operates-what-it-franchises.md
kplant "a bet with status: locked (ADR-0022)"        "$B_LIVE" "status: accepted" "status: locked"
kplant "a bet with no trigger"                       "$B1" "trigger: >" "x_trigger: >"
kplant "a bet with no review_date"                   "$B1" "review_date: 2026-12-01" "x_review_date: 2026-12-01"
kplant "status: deferred on a non-requirement"       "$B_LIVE" "status: accepted" "status: deferred"
kplant "an RFC with no alternatives section"         rfc/RFC-0007-nested-tenancy.md "## Options" "## Mechanisms"
# The replacement id is BUILT AT RUNTIME so the literal never appears in this file. Fourth
# instance of the same trap today: the cross-reference scanner reads *.sh as well as *.md, so a
# plausible id anywhere — a template example, the prose explaining that example, a fixture path,
# and now a fixture VALUE — becomes a citation the gate then reports as dangling. An exclusion for
# this file was considered and rejected: it would stop a real typo'd citation here from being
# caught, and DECISION-DEBT.md's exclusion was justified only because that file is machine-written.
dead_id="ADR-$(printf '%04d' 199)"
kplant "a dangling artifact reference"               constitution/C-0006-market-posture.md "ADR-0024" "$dead_id"
kplant "registry/schema type-enum drift"             artifact-registry.yaml "  - type: bet" "  - type: wager"
kplant "registry/schema id-prefix drift"             knowledge.schema.json "ADR|ANTI|BET|C|GLO" "ADR|ANTI|C|GLO"
kplant "two artifact types sharing a directory"      artifact-registry.yaml "    directory: bets" "    directory: adr"
# Anchored on the LAST directory in the guard's case arm, which moves whenever a type is added —
# it was `glossary` until ADR-0050 appended `markets`. Adding a type therefore breaks this plant,
# and that is the correct failure mode: the anchor and the guard must be updated together, which
# is the same coupling the 8c comparison enforces between the guard and the registry.
kplant "the PreToolUse guard drifting from section 3" \
  plugins/paridyum-platform/hooks/guard-locked-decisions.sh '|*"/markets/"*.md) ;;' ') ;;'
kplant "a stale prefix set in check-registry.schema.json" \
  check-registry.schema.json "ADR|ANTI|BET|C|GLO" "ADR|ANTI|C|GLO"
# The sixteenth site, added 2026-08-17. The ledger hook's DIRECTORY list is separate from its
# id-prefix scanner above and was compared by nothing, so it held seven directories against the
# registry's eleven and four `proposed` artifacts were invisible to the ADR-0011 ledger. The
# plant drops `markets` specifically, because that is the class whose absence would be least
# visible: it is the newest, and an unnoticed MKT- is exactly the loop the ledger exists to
# surface.
kplant "the open-loop ledger drifting from the artifact registry" \
  plugins/paridyum-platform/hooks/capture-open-loops.sh \
  "glossary markets patterns" "glossary patterns"
kplant "enforced_by naming an unregistered check"    adr/ADR-0024-nested-tenancy-is-two-columns-in-one-database.md \
  "check: db.operator_scope_is_an_explicit_grant" "check: db.no_such_check_exists"
kplant "check-registry entries out of sorted order"  check-registry.yaml \
  "  - check: repo.claude_md_links_resolve" "  - check: zz.out_of_order_on_purpose"
# knowledge.check_specs_cite_live_artifacts. The plant is the real defect this check was
# written for: repo.name_matches_scheme's spec cited C-0002's prefix table, which holds seven
# of the ten classes, so an implementation obeying the spec would have rejected every web-,
# sdk- and mobile- repository. `notes` are deliberately NOT scanned — 22 of the 27 references
# found in the first sweep were notes recording why a rule changed, which is the registry
# working correctly, and a check firing on that would be worse than no check.
kplant "a check spec citing a superseded artifact" check-registry.yaml \
  "a prefix from \`class-registry.yaml\`" "a prefix from C-0002"
# knowledge.hardcoded_count_baseline_does_not_rise. The plant is the exact edit made on
# 2026-08-10: a registry-computed count added to TESTING.md, one of the two living documents
# ADR-0035's context credits with having stopped stating them. The warn-level check saw it as
# 48 -> 49 and said nothing, because it always fires and nobody reads a number that never
# changes meaningfully. The ratchet is what makes the regression visible.
kplant "a NEW hardcoded registry count above the baseline" TESTING.md \
  "**The checks whose owner is" "**The 71 checks whose owner is"
# ADR-0036. The plant removes the RESOLUTION rather than adding a conflict, because the conflict
# is real and still in the tree: ADR-0007 cites repo.claude_md_under_line_budget at `warn` with
# max_lines 200, ADR-0026 cites it at `error` with no args. Renaming the resolution key restores
# the state the repository was actually in this morning — two locked artifacts disagreeing, and
# check-repo.sh enforcing a third threshold that neither of them declared.
kplant "a conflicting check citation with no resolution" check-registry.yaml \
  "    resolution:" "    x_resolution:"
# ADR-0035 is `warn`, so expect_caught can never see it — the exit code does not move. Same
# reason expect_warns exists for the two warn-level knowledge checks.
expect_warns "numeric claims about registry-computed things are listed" \
  "numeric claim" bash scripts/check-knowledge.sh
kplant "supersession recorded on only one side"      adr/ADR-0023-tenancy-is-row-level-in-one-database.md \
  "superseded_by: ADR-0024" "superseded_by: null"
# knowledge.retired_terms_absent (ADR-0113 D5). The first plant is the defect itself: a retired
# phrase in a live artifact that carries no dated note. The second removes the list, and must fail
# rather than scan for nothing and print a tick — ABSENT IS NOT PASS, as the count baseline has it.
kplant "a retired phrase in an unnoted artifact (ADR-0113)" glossary/GLO-0011-environment.md \
  "## Definition" "## Definition — as held by the platform tenant"
kplant "the retired-terms list removed from ADR-0113" \
  adr/ADR-0113-org-tenant-and-account-are-three-words-for-three-levels-and-the-vendors-organization-is-named-as-the-vendors.md \
  "      terms: [" "      x_terms: ["

# knowledge.framework_ids_resolve (ADR-0028). A framework mapping is the easiest thing in this
# repository to fabricate — PW.4.1 and PW.4.2 are indistinguishable to a reviewer, and a wrong id
# reads as coverage forever. All four ways it can be wrong are planted, because "the check exists"
# and "the check rejects THIS" have been different answers here before.
kplant "a framework mapping naming an unregistered framework" check-registry.yaml \
  '      ssdf: ["PO.4.1"]' '      nist_666: ["PO.4.1"]'
kplant "an SSDF id in a shape SSDF does not use"     check-registry.yaml \
  '      ssdf: ["PO.4.1"]' '      ssdf: ["PO-4-1"]'
kplant "a CIS safeguard outside the 1-18 control range" check-registry.yaml \
  '      cis: ["16.12"]' '      cis: ["19.4"]'
kplant "a CSF function that does not exist"          check-registry.yaml \
  '      csf: ["PR.DS", "DE.CM"]' '      csf: ["XX.DS", "DE.CM"]'
# The id_pattern is the whole control, so a registry entry whose pattern stops compiling would
# silently accept anything. Anchored by the framework nobody maps to yet, so this cannot collide
# with a real mapping.
kplant "a framework whose id_pattern does not compile" framework-registry.yaml \
  "id_pattern: '^V[0-9]+(\\.[0-9]+)*\$'" "id_pattern: '^V[0-9+\$'"
# check-registry.schema.json existed for two days and was never executed against the file it
# describes — the bash checks below it were assumed to be a subset of it, and were in fact the
# only thing running. These two cases are chosen because ONLY the schema pass rejects them:
# `additionalProperties: false` and the minItems on a mapping array have no bash equivalent.
kplant "an unknown property in a check-registry entry" check-registry.yaml \
  '    blocks: [ADR-0028]' '    severity: error
    blocks: [ADR-0028]'
kplant "an empty framework mapping array"            check-registry.yaml \
  '      ssdf: ["PO.4.1"]
      csf: ["GV.OV"]' '      ssdf: []
      csf: ["GV.OV"]'

# knowledge.test_registry_valid (ADR-0029). Nine ways the registry can mean less than it says,
# each planted. The registry resolves against four other files — check ids, framework ids, stacks
# and artifact ids — and every one of those cross-references is a place where an entry can look
# governed and govern nothing, which is the defect class ADR-0014 was written for.
TR=test-registry.yaml
kplant "a duplicate test_type id"                    "$TR" \
  "  - test_type: TST-0002" "  - test_type: TST-0001"
kplant "test-registry entries out of sorted order"   "$TR" \
  "  - test_type: TST-0003" "  - test_type: TST-0099"
kplant "a test type on an undeclared axis"           "$TR" \
  "    axis: functional" "    axis: made-up-axis"
kplant "a test type in an undeclared tier"           "$TR" \
  "    tier: pr
    status: required" "    tier: whenever
    status: required"
# ADR-0018's rule, reused: a deferred entry with no trigger is a TODO with an id, and nothing
# would ever revisit it. The schema cannot express "required only when status is deferred".
kplant "a deferred test type with no trigger"        "$TR" \
  "    status: required" "    status: deferred"
# The opposite, which is likelier and reads as correct: a type whose trigger has fired and whose
# status was updated in only one of the two places.
kplant "a trigger left on a type that is no longer deferred" "$TR" \
  "    status: deferred
    trigger:" "    status: recommended
    trigger:"
kplant "a test type naming a stack no class declares" "$TR" \
  "    stacks: [rust, typescript]" "    stacks: [rust, cobol]"
kplant "a test type naming an unregistered check id"  "$TR" \
  "    checks: [test.integration_tests_use_testcontainers]" "    checks: [test.no_such_check_exists]"
# The same fabrication hazard ADR-0028 documents, one file over. The registry was added after
# the framework check existed and would have been outside it entirely without this.
kplant "an invalid framework id in the test registry" "$TR" \
  '      cis: ["16.12"]' '      cis: ["19.4"]'
# Only the schema pass rejects this one; there is no bash equivalent of additionalProperties.
kplant "an unknown property in a test-registry entry" "$TR" \
  "    name: unit" "    name: unit
    severity: error"

# knowledge.verification_methods_resolve (ADR-0030). Identical in shape to an unresolved check
# id, and identical in consequence: it answers "how is this verified" confidently and wrongly.
kplant "verified_by naming a test type that does not exist" \
  standards/STD-0009-tenant-isolation-testing.md "  - TST-0016" "  - TST-9999"

# RFC-0005 resolves to ADR-0022 and never mentions ADR-0024, so repointing it at the superseded
# ADR-0023 leaves the successor unnamed — which is exactly the condition. RFC-0006 was the wrong
# fixture: it cites ADR-0024 in several places, so replacing one left the others and the check
# was correct to stay quiet.
kwarn() { # kwarn <label> <pattern> <file> <old> <new>
  local label="$1" pat="$2" f="$KB/$3" old="$4" new="$5"
  cp "$f" "$f.orig"
  python3 - "$f" "$old" "$new" <<'PY2'
import sys, pathlib
p = pathlib.Path(sys.argv[1]); s = p.read_text()
assert sys.argv[2] in s
p.write_text(s.replace(sys.argv[2], sys.argv[3], 1))
PY2
  expect_warns "$label" "$pat" bash "$KB/scripts/check-knowledge.sh"
  mv "$f.orig" "$f"
}
kwarn "resolved_by pointing at a superseded ADR (warn)" "resolved_by names" \
  rfc/RFC-0005-commercial-strategy-as-knowledge.md "resolved_by: ADR-0022" "resolved_by: ADR-0023"
kwarn "a review_date that has already passed (warn)" "review_date .* has passed" \
  bets/BET-0002-price-is-not-infrastructure-resold.md "review_date: 2026-12-01" "review_date: 2020-01-01"
# ADR-0028 §5: an unmapped framework is legal only when it says which requirement will close the
# gap. Removing that field must surface it, or `asvs` and `slsa` become permanent silent padding
# in the one file whose entire job is describing coverage honestly.
kwarn "a registered framework with no mapping and no reason (warn)" "no control maps to it" \
  framework-registry.yaml "    mapping_deferred_to: REQ-0015" "    x_mapping_deferred_to: REQ-0015"

# knowledge.verification_mapping_is_symmetric (ADR-0030). Both directions, because a mapping
# written in one place only is one nothing can contradict — the same finding section 7 made about
# `blocks` against the artifacts citing a check. Warn rather than error, and the reason is in
# ADR-0030 §3: a locked artifact CANNOT gain a verified_by entry, so a hard failure here would be
# permanent with no legal remedy.
kwarn "an artifact naming a type that does not name it back (warn)" "governed_by omits" \
  test-registry.yaml "    governed_by: [STD-0009, ADR-0024, ADR-0114, ADR-0115, ADR-0117, ADR-0118, ADR-0128]" \
  "    governed_by: [ADR-0024, ADR-0114, ADR-0115, ADR-0117, ADR-0118, ADR-0128]"
# The other direction. C-0001 is `accepted` — mutable and not deferred — so it is exactly the case
# the excuse list does NOT cover, which is what makes it a usable fixture.
kwarn "a type claiming a mutable artifact that stays silent (warn)" "verified_by omits them" \
  test-registry.yaml "    governed_by: [STD-0001]" "    governed_by: [STD-0001, C-0001]"

# --- ADR-0050: the product axis, the dossier, and the MKT- type -------------
# THREE CHECKS THAT PASSED VACUOUSLY ON THE DAY THEY LANDED, and that is the condition this
# suite exists to tell apart from working. `markets/` holds no artifact and no frontmatter
# anywhere carries `product:`, so knowledge.market_artifacts_cite_sources and
# knowledge.product_slugs_resolve ran against empty inputs and printed success. A check with no
# subject is indistinguishable from a check that does nothing — and it is worse than an
# unimplemented one, because check-registry.yaml's owner column already counts it as coverage.
#
# The id is BUILT AT RUNTIME, for the reason the dangling-reference plant above records: the
# cross-reference scanner reads *.sh as well as *.md, so a literal MKT id written here would
# become a citation the gate then reports as dangling. Fifth instance of that same trap.
#
# AND IT IS 9001 RATHER THAN 0001, which this suite learned the hard way on its first run. The
# fixture originally took the next free id, and a concurrent session created a real MKT-0001
# between the test being written and being run — so the positive control failed on `duplicate
# id`, reporting a defect in the gate that was really a collision with someone else's work.
# A fixture must not compete for the id space it is testing, so the high range is reserved for
# fixtures — the same move the dangling-reference plant above makes, and for the same reason.
# (Writing that plant's id literally in this comment made the gate report it as a dangling
# citation, on the run that was verifying this very block. Sixth instance.)
mkt_id="MKT-$(printf '%04d' 9001)"
MKTF="markets/$mkt_id-a-planted-market-observation.md"
mkdir -p "$KB/markets"
mkt_write() { # mkt_write <trailing-frontmatter-block>
  cat > "$KB/$MKTF" <<EOF
---
id: $mkt_id
title: A planted market observation, written and removed by the gate test suite
type: market
status: accepted
created: 2026-08-17
scope: [org]
product: [paridyum]
$1
---

# $mkt_id — A planted market observation

Fixture. Written by test-checks.sh and removed by it.
EOF
}

# THE POSITIVE CONTROL COMES FIRST AND IT IS NOT CEREMONY. If the gate rejects a well-formed
# MKT-, every negative case below "catches" its defect for the wrong reason and the whole block
# reports success while proving nothing. This is the one case whose failure invalidates the rest.
mkt_write 'sources:
  - url: https://example.org/fixture-report
    title: A source that exists only inside this fixture
    retrieved: 2026-08-17'
expect_ok "a well-formed MKT- with sources passes, and product: paridyum resolves" \
  bash "$KB/scripts/check-knowledge.sh"

# The rule that makes MKT- an evidence type rather than a bet with a different prefix. It cannot
# live in knowledge.schema.json: validate-schema.py implements no conditionals and reports them
# as unenforced, so a schema-side `if/then` would advertise a constraint nothing applied.
mkt_write 'tags: [fixture]'
expect_caught "an MKT- carrying no sources at all (ADR-0050)" bash "$KB/scripts/check-knowledge.sh"

# `retrieved` is the load-bearing field of the three: it converts a claim into a dated
# observation, which is the only honest form for a fact that changes somewhere else (RFC-0020).
mkt_write 'sources:
  - url: https://example.org/fixture-report
    title: A source with no retrieval date'
expect_caught "an MKT- source with no retrieved date" bash "$KB/scripts/check-knowledge.sh"

# A typo'd slug produces an artifact that validates, binds nothing, and reads to a human as
# though it governs the product whose name it nearly spells — the ADR-0014 defect, one axis over.
mkt_write 'sources:
  - url: https://example.org/fixture-report
    title: A source that exists only inside this fixture
    retrieved: 2026-08-17'
kplant "an artifact naming an unregistered product" "$MKTF" \
  "product: [paridyum]" "product: [no-such-product]"

rm -f "$KB/$MKTF"
expect_ok "the gate is clean again once the fixture is removed" bash "$KB/scripts/check-knowledge.sh"

printf '\n── check-suite scope (ADR-0124 addendum) ──────────────────────────\n'
# The suite skips itself on a pull request that cannot reach it. A scope rule that is too wide only
# costs minutes; one that is too narrow lets a broken gate merge. So the cases that matter are the
# ones that must RUN, and each is a path that looks like documentation and is not.
scope() { printf '%s\n' "$@" | bash scripts/check-suite-scope.sh 2>/dev/null; }
# The unplanted path is assembled, never written whole: this file is what the scope rule searches,
# so a literal path here would be "named in the suite" and the case would test itself.
unplanted="docs/notes/an-unplanted"; unplanted="$unplanted-page.md"
expect_ok "a docs-only change skips the suite" \
  test "$(scope CHANGELOG.md "$unplanted" DECISION-DEBT.md README.md)" = run=false
expect_ok "a checker change runs it" \
  test "$(scope README.md scripts/check-knowledge.sh)" = run=true
expect_ok "a registry change runs it" \
  test "$(scope check-registry.yaml)" = run=true
expect_ok "an artifact the suite plants into runs it, though it looks like documentation" \
  test "$(scope bets/BET-0005-paridyum-operates-what-it-franchises.md)" = run=true
expect_ok "TESTING.md runs it" \
  test "$(scope TESTING.md)" = run=true
expect_ok "no input runs it: skipping has to be earned" \
  test "$(printf '' | bash scripts/check-suite-scope.sh 2>/dev/null)" = run=true

printf '\n── research capture (RFC-0027) ────────────────────────────────────\n'
# Two halves, tested two ways. The hook is DRIVEN with synthetic payloads, the way ADR-0007 says
# a hook must be tested — by running it, not by reading it. The gate is tested by planting logs
# in the $KB copy.
#
# knowledge.research_log_is_append_only's git half is NOT covered here: $KB has no .git, so the
# `git diff HEAD` branch is guarded off, exactly as the locked-artifact immutability check is.
# Stated rather than left for the next mutation run to find. The malformed-line half IS covered.

WHOOK="$ROOT/plugins/paridyum-platform/hooks/log-web-sources.sh"
SHOOK="$ROOT/plugins/paridyum-platform/hooks/capture-open-loops.sh"
wr="$TMP/webhook"; mkdir -p "$wr/constitution" "$wr/adr" "$wr/rfc"
# THE ID IS BUILT AT RUNTIME, never written as a literal — the same trick the MKT- fixture uses
# two blocks up, and for a reason this suite discovered by tripping over it: a fixture id spelled
# out in this file is an id the repository REFERENCES and does not define, so
# knowledge.dangling_references reports the test suite itself. The positive control below then
# fails for a reason that has nothing to do with the rule under test.
rfc_id="RFC-$(printf '%04d' 9001)"
cat > "$wr/rfc/$rfc_id-fixture.md" <<MD
---
id: $rfc_id
title: A fixture that cites one page
type: rfc
status: proposed
sources:
  - url: https://example.org/cited-page
    title: The page this fixture cites
    retrieved: 2026-08-18
---
Fixture. Written by test-checks.sh.
MD
wfetch() { printf '{"cwd":"%s","session_id":"fixture0-aaaa","tool_name":"WebFetch","tool_input":{"url":"%s","prompt":"p"}}' "$wr" "$1" | bash "$WHOOK" >/dev/null 2>&1; }
wfetch "https://example.org/cited-page"
wfetch "https://example.org/never-cited"
printf '{"cwd":"%s","session_id":"fixture0-aaaa","tool_name":"WebSearch","tool_input":{"query":"q"},"tool_response":"see https://example.org/only-surfaced"}' "$wr" | bash "$WHOOK" >/dev/null 2>&1
printf '{"cwd":"%s","session_id":"fixture0-aaaa","tool_name":"Bash","tool_input":{"command":"ls"}}' "$wr" | bash "$WHOOK" >/dev/null 2>&1
WLOG="$(find "$wr/research" -name '*.jsonl' 2>/dev/null | head -1)"

wcount() { jq -r "select(.kind == \"$1\") | .url" "$WLOG" 2>/dev/null | grep -c . ; }
if [[ -n "$WLOG" ]] && [[ "$(wcount fetched)" == 2 && "$(wcount surfaced)" == 1 ]]; then
  ok "the hook logs a fetched page and a surfaced result as different kinds"
else
  bad "the hook logs a fetched page and a surfaced result as different kinds" "$(cat "$WLOG" 2>/dev/null)"
fi
# A single WebSearch offers ten results. If those reconciled, one search would deposit ten
# entries into DECISION-DEBT.md and the ledger would stop being read — the one failure mode
# ADR-0011 cannot survive. This is the case that guards the split.
if ! grep -q '"tool":"Bash"' "$WLOG" 2>/dev/null; then
  ok "a non-web tool call is not logged"
else
  bad "a non-web tool call is not logged" "$(cat "$WLOG")"
fi

# Consequence 5: the upward walk from cwd misses a session started in the PARENT of the
# repository, which is the commonest way this one is worked on. Found by using the hook, not by
# reading it — two pages were read on 2026-08-19 while verifying RFC-0027's own open question 1
# and neither was logged. The root is named explicitly or capture does not happen; guessing
# downward would file evidence against whichever checkout a parent happened to hold first.
pcwd="$TMP/parent"; mkdir -p "$pcwd/kb/constitution" "$pcwd/kb/adr"
# THE URL IS UNIQUE PER RUN ($$). A fixed one made this assertion match the leak a PREVIOUS run
# left in the real research/ log — which decision 6 voids but never deletes, so it is there
# forever. An assertion that reads a permanent record must scope itself to this run.
purl="https://example.org/from-a-parent-cwd-$$"
ppay='{"cwd":"'"$pcwd"'","session_id":"parent00-a","tool_name":"WebFetch","tool_input":{"url":"'"$purl"'","prompt":"p"}}'
# `env -u` IS NOT OPTIONAL HERE, and this case proved it by failing open. The variable is set
# in the real environment, so without unsetting it the hook fell back to THE ACTUAL REPOSITORY
# and wrote a fixture URL into its research log — while this assertion looked at $pcwd/kb and
# reported success. A test that passes because it inspected the wrong directory is the ADR-0014
# defect inside the suite whose entire purpose is to prevent it. Found 2026-08-19 by noticing a
# tracked research/ file in `git status` that no session had produced.
printf '%s' "$ppay" | env -u PARIDYUM_KNOWLEDGE_ROOT bash "$WHOOK" >/dev/null 2>&1
if [[ ! -d "$pcwd/kb/research" ]] && ! grep -rqF "$purl" "$ROOT/research" 2>/dev/null; then
  ok "a session started above the repository captures nothing rather than guessing a root"
else
  bad "a session started above the repository captures nothing rather than guessing a root" \
      "wrote somewhere it should not have — check $ROOT/research"
fi
printf '%s' "$ppay" | PARIDYUM_KNOWLEDGE_ROOT="$pcwd/kb" bash "$WHOOK" >/dev/null 2>&1
if grep -qF "$purl" "$pcwd"/kb/research/*.jsonl 2>/dev/null; then
  ok "PARIDYUM_KNOWLEDGE_ROOT names the root when the upward walk cannot find it"
else
  bad "PARIDYUM_KNOWLEDGE_ROOT names the root when the upward walk cannot find it"
fi

# RFC-0027 open question 2. SessionStart resolves the root through the variable and SAYS where
# capture is going, so an empty research/ means "nothing fetched" rather than "never running".
# The genuinely unarmed case stays silent by design — see the RFC; warning from an unrelated
# directory would fire in every project on the machine.
SCHOOK="$ROOT/plugins/paridyum-platform/hooks/session-context.sh"
sc_out() { (cd "$pcwd" && printf '{"cwd":"%s"}' "$pcwd" | env "$@" bash "$SCHOOK" 2>/dev/null); }
# CAPTURE, THEN MATCH — never `sc_out ... | grep -q`. This suite runs under `set -o pipefail`,
# and `grep -q` exits on its FIRST match, so the producer takes SIGPIPE and the pipeline reports
# 141. The assertion then fails on a hook that printed exactly what was asked for. Cost one
# debugging round on 2026-08-19; the same trap is waiting in every `... | grep -q` written here.
sc_armed="$(sc_out PARIDYUM_KNOWLEDGE_ROOT="$ROOT")"
if grep -q 'Research capture' <<<"$sc_armed"; then
  ok "SessionStart reports where research capture is going when the root is named"
else
  bad "SessionStart reports where research capture is going when the root is named" "$sc_armed"
fi
if [[ -z "$(sc_out -u PARIDYUM_KNOWLEDGE_ROOT)" ]]; then
  ok "SessionStart stays silent in a directory with no connection to this repository"
else
  bad "SessionStart stays silent in a directory with no connection to this repository" "$(sc_out -u PARIDYUM_KNOWLEDGE_ROOT | head -3)"
fi

# The class comes from class-registry.yaml. It came from a case statement holding C-0002's
# prefixes, so a web- repository was told its name broke the scheme and shown every locked
# artifact unfiltered — found while adding game- (ADR-0133). Each case is a directory with the
# repository's name, vendoring this checkout the way a consuming repository does.
sc_named() {
  local p; p="$(mktemp -d)/$1"; mkdir -p "$p/.paridyum"; ln -s "$ROOT" "$p/.paridyum/knowledge"
  printf '{"cwd":"%s"}' "$p" | bash "$SCHOOK" 2>/dev/null; rm -rf "$(dirname "$p")"
}
for spec in "web-example:web" "game-lantern:game" "svc-billing:service" "lib-auth-rust:library"; do
  name="${spec%%:*}"; want="${spec#*:}"; sc_class_out="$(sc_named "$name")"
  if [[ "$sc_class_out" == *"class \`$want\`"* && "$sc_class_out" != *"does not match the naming scheme"* ]]; then
    ok "SessionStart reads $name's class from the registry: $want"
  else
    bad "SessionStart reads $name's class from the registry: $want" "$(head -4 <<<"$sc_class_out")"
  fi
done
sc_class_out="$(sc_named "widget-example")"
if [[ "$sc_class_out" == *"does not match the naming scheme in C-0007"* && "$sc_class_out" == *'`game-`'* ]]; then
  ok "SessionStart lists the registry's prefixes for a name that matches none"
else
  bad "SessionStart lists the registry's prefixes for a name that matches none" "$(head -4 <<<"$sc_class_out")"
fi

# Ages in the ledger. The hook used to compute "now minus created", which straddled a second and
# floored to a day short, and showed "?" for anything created today. Three proposed fixtures, created
# today, 46 days ago and in the future; ids built at runtime for the reason given above.
day_ago() { date -v-"$1"d +%Y-%m-%d 2>/dev/null || date -d "$1 days ago" +%Y-%m-%d; }
day_ahead() { date -v+"$1"d +%Y-%m-%d 2>/dev/null || date -d "$1 days" +%Y-%m-%d; }
for spec in "9101:$(day_ago 0)" "9102:$(day_ago 46)" "9103:$(day_ahead 3)"; do
  n="${spec%%:*}"; created="${spec#*:}"; aid="RFC-$(printf '%04d' "$n")"
  printf -- '---\nid: %s\ntitle: Age fixture %s\ntype: rfc\nstatus: proposed\ncreated: %s\n---\n\n# %s\n' \
    "$aid" "$n" "$created" "$aid" > "$wr/rfc/$aid-age-fixture.md"
done
# Reconciliation. Uncited fetches reach the ledger; cited and surfaced ones do not.
printf '{"cwd":"%s","session_id":"fixture0-aaaa","transcript_path":"/nonexistent"}' "$wr" | bash "$SHOOK" >/dev/null 2>&1
for spec in "9101:0d" "9102:46d" "9103:?"; do
  n="${spec%%:*}"; want="${spec#*:}"; aid="RFC-$(printf '%04d' "$n")"
  line="$(grep -F "**$aid**" "$wr/DECISION-DEBT.md" 2>/dev/null)"
  if [[ "$line" == *"for $want —"* ]]; then
    ok "the ledger ages $aid as $want"
  else
    bad "the ledger ages $aid as $want" "${line:-no ledger line for $aid}"
  fi
done
if grep -q 'never-cited' "$wr/DECISION-DEBT.md" 2>/dev/null; then
  ok "a page read and never cited is filed as an open loop"
else
  bad "a page read and never cited is filed as an open loop" "$(cat "$wr/DECISION-DEBT.md" 2>/dev/null)"
fi
for u in cited-page only-surfaced; do
  if grep -q "$u" "$wr/DECISION-DEBT.md" 2>/dev/null; then
    bad "$u is not filed as an open loop" "$(cat "$wr/DECISION-DEBT.md")"
  else ok "$u is not filed as an open loop"; fi
done
# The session record is what the gate reads, and it must be appended rather than written back
# into the fetch lines: a log the reconciler can edit is not evidence.
if [[ "$(jq -rs '[.[] | select(.kind == "session")] | length' "$WLOG" 2>/dev/null)" == 1 ]] \
   && ! grep -q '"cited":false' "$WLOG"; then
  ok "reconciliation appends a session record and rewrites no fetch line"
else
  bad "reconciliation appends a session record and rewrites no fetch line" "$(cat "$WLOG")"
fi

# knowledge.research_log_is_append_only, the git half. $KB has no .git so the removal branch is
# unreachable there; this uses a real repository, the same way the errata block does, because
# "a line was removed" is a claim about the COMMITTED version that a fixture directory cannot
# express.
#
# THE DELETION CASE IS THE POINT. The check originally lived inside the "research logs exist"
# branch, so removing the whole directory deleted the evidence AND the check that guards it, and
# the gate reported success. A control whose subject can be emptied by the defect it detects is
# not a control. Found 2026-08-19 while removing a leaked fixture.
# A FULL COPY, NOT A HAND-BUILT SKELETON. The first version assembled a few registries into an
# empty tree, so the gate was ALREADY RED there (dangling C-0006, ADR-0035 — the registries cite
# artifacts that were not copied) and every expect_caught below reported success on a failure
# that had nothing to do with the planted defect. An invalid negative: the check under test was
# never what made the gate red. The positive control is what caught it, which is why it is first.
RL="$TMP/rlog"
cp -R . "$RL" >/dev/null 2>&1
rm -rf "$RL/.git" "$RL/research"
mkdir -p "$RL/research"
printf '{"ts":"2026-08-18T00:00:00Z","session":"gitfix0","tool":"WebFetch","kind":"fetched","url":"https://example.org/committed","title":null,"query":null}\n' \
  > "$RL/research/2026-08-18-gitfix0.jsonl"
(cd "$RL" && git init -q -b main . && git config user.email t@t && git config user.name T \
  && git add -A && git commit -qm "fixture") >/dev/null 2>&1
expect_ok "a committed, untouched research log passes" bash "$RL/scripts/check-knowledge.sh"
rm -rf "$RL/research"
expect_caught "deleting the entire research directory" bash "$RL/scripts/check-knowledge.sh"
(cd "$RL" && git checkout -q -- research) >/dev/null 2>&1
printf 'REWRITTEN\n' > "$RL/research/2026-08-18-gitfix0.jsonl"
expect_caught "rewriting a committed research log line" bash "$RL/scripts/check-knowledge.sh"
(cd "$RL" && git checkout -q -- research) >/dev/null 2>&1
printf '{"ts":"2026-08-18T00:00:01Z","session":"gitfix0","tool":"WebFetch","kind":"fetched","url":"https://example.org/appended","title":null,"query":null}\n' \
  >> "$RL/research/2026-08-18-gitfix0.jsonl"
expect_ok "appending to a committed research log is the one legal edit" bash "$RL/scripts/check-knowledge.sh"

# --- the gate, against planted logs ----------------------------------------
mkdir -p "$KB/research"
KLOG="$KB/research/2026-08-18-fixture0.jsonl"
klog() { printf '%s\n' "$@" > "$KLOG"; }

# POSITIVE CONTROL FIRST, for the reason the MKT- block states: if the gate rejects a well-formed
# log, every negative case below catches its defect for the wrong reason.
klog '{"ts":"2026-08-18T00:00:00Z","session":"fixture0","tool":"WebFetch","kind":"fetched","url":"https://pve.proxmox.com/wiki/Cluster_Manager","title":null,"query":null}' \
     '{"ts":"2026-08-18T00:00:01Z","session":"fixture0","tool":"WebSearch","kind":"surfaced","url":"https://example.org/only-surfaced","title":null,"query":"q"}' \
     '{"ts":"2026-08-18T00:00:02Z","session":"fixture0","kind":"session","fetched":1,"cited":1,"uncited":[],"artifacts":[]}'
expect_ok "a reconciled research log passes" bash "$KB/scripts/check-knowledge.sh"

klog '{"kind":"session","session":"fixture0","fetched":1,"cited":0,"uncited":["https://example.org/never-cited"],"artifacts":[]}'
expect_warns "a page fetched and never cited is reported" "never-cited" bash "$KB/scripts/check-knowledge.sh"

klog 'this is not json'
expect_caught "a malformed line in the research log" bash "$KB/scripts/check-knowledge.sh"

# knowledge.researched_artifacts_cite_sources. No type sets when_researched in the repository, so
# the fixture sets one — which is the point of the axis being registry data rather than a literal
# in the gate: the rule can be exercised without any artifact opting in.
WRT="$KB/adr/ADR-0011-anti-forgetting-ledger.md"
if [[ -f "$WRT" ]] && ! grep -q '^sources:' "$WRT"; then
  cp "$KB/artifact-registry.yaml" "$KB/artifact-registry.yaml.orig"
  python3 - "$KB/artifact-registry.yaml" <<'PY'
import sys, pathlib, re
p = pathlib.Path(sys.argv[1]); s = p.read_text()
s = re.sub(r"(  - type: adr\n(?:.*\n)*?    requires_sources: )never", r"\1when_researched", s, count=1)
p.write_text(s)
PY
  klog '{"kind":"session","session":"fixture0","fetched":1,"cited":1,"uncited":[],"artifacts":["adr/ADR-0011-anti-forgetting-ledger.md"]}'
  expect_caught "an artifact of a when_researched type written after a fetch, citing nothing" \
    bash "$KB/scripts/check-knowledge.sh"
  mv "$KB/artifact-registry.yaml.orig" "$KB/artifact-registry.yaml"
else
  bad "an artifact of a when_researched type written after a fetch, citing nothing — fixture anchor missing"
fi

rm -rf "$KB/research"
expect_ok "the gate is clean again once the research fixture is removed" bash "$KB/scripts/check-knowledge.sh"


# knowledge.product_registry_valid. Same argument as class_registry_valid and
# artifact_registry_valid: moving a list out of code makes it easier to change, which is both
# the intent and the risk, so the data needs a check or it is less protected than the hardcoded
# list it replaced.
PRG=product-registry.yaml
kplant "two products sharing a slug"                 "$PRG" \
  "  - product: tenancy" "  - product: identity"
# Anchored on kind+stage together, because `    stage: building` alone also matches a dossier
# entry and would silently test a different defect than the label claims.
kplant "a product at a stage the lifecycle does not declare" "$PRG" \
  "    kind: platform
    stage: building" "    kind: platform
    stage: shipping"
# The half that will actually fire in practice: a dossier requirement naming a prefix nothing
# can resolve reads as governance and enforces nothing.
kplant "a dossier entry naming an unregistered artifact prefix" "$PRG" \
  "    artifacts: [MKT, REQ, BET, RUN]" "    artifacts: [MKT, REQ, ZZZ, RUN]"
kplant "a dossier entry for a kind the registry does not declare" "$PRG" \
  "  - kind: catalogue
    stage: proposed" "  - kind: franchise
    stage: proposed"


printf '\n── diagrams (ADR-0065, STD-0020) ──────────────────────────────────\n'
# docs.diagram_source_is_present, docs.diagram_is_self_contained,
# docs.architecture_ir_is_hand_authored, and the filename-kind rule.
#
# THE IR-VALIDITY CASE IS NOT HERE, deliberately. It needs archify installed at its pinned sha,
# and a suite case that silently passes on a machine without the tool is the green check that
# proves nothing -- the exact failure this file exists to prevent. The gate warns rather than
# skipping silently; that warning is the honest surface, and this is the record of the gap.
#
# `withdiag` writes a MINIMAL render rather than a real one: these three checks read the
# filename and grep for an external origin, and none of them parses the HTML. A 700KB fixture
# per case would make the suite slower and prove nothing extra.
withdiag() { # a scaffold carrying one valid IR and its render
  local d="$1"; withtests "$d"; mkdir -p "$d/docs/diagrams"
  printf '{"schema_version":1,"diagram_type":"lifecycle","meta":{"title":"t"},"lanes":[],"states":[],"transitions":[]}\n' \
    > "$d/docs/diagrams/sample.lifecycle.json"
  printf '<!doctype html><html><body><svg xmlns="http://www.w3.org/2000/svg"></svg></body></html>\n' \
    > "$d/docs/diagrams/sample.lifecycle.html"
}
WD="$TMP/withdiag"; withdiag "$WD"
expect_ok "a diagram with its IR beside it and no external origin passes" \
  bash scripts/check-repo.sh --root "$WD"

plantd() { local label="$1" fn="$2" d="$TMP/cd"; rm -rf "$d"; withdiag "$d"; "$fn" "$d"
  expect_caught "$label" bash scripts/check-repo.sh --root "$d"; }

m_diag_no_ir()    { rm -f "$1/docs/diagrams/sample.lifecycle.json"; }
m_diag_fetches()  { printf '<!doctype html><html><head><link href="https://fonts.googleapis.com/css2?family=X" rel="stylesheet"></head><body></body></html>\n' \
                      > "$1/docs/diagrams/sample.lifecycle.html"; }
m_diag_arch()     { printf '{"schema_version":1,"diagram_type":"architecture","meta":{"title":"t"}}\n' \
                      > "$1/docs/diagrams/estate.architecture.json"; }
m_diag_no_kind()  { mv "$1/docs/diagrams/sample.lifecycle.json" "$1/docs/diagrams/sample.json"; }

plantd "a rendered diagram with no IR beside it"        m_diag_no_ir
plantd "a diagram that fetches a font when opened"      m_diag_fetches
plantd "a hand-authored architecture IR (ADR-0040)"     m_diag_arch
plantd "an IR whose filename does not name its kind"    m_diag_no_kind

# docs.diagram_is_current, planted against THIS repository rather than a fixture. It needs a
# real render to compare against, and `withdiag`'s minimal HTML is not one -- a fixture would
# report stale for the trivial reason that it was never rendered, which proves nothing about
# the comparison. Skipped rather than faked where archify is absent, for the reason the section
# comment in check-repo.sh gives.
if A_BIN="$(bash "$ROOT/scripts/tool-path.sh" archify 2>/dev/null)" && [[ -x "$A_BIN" ]]; then
  CURIR="$ROOT/docs/diagrams/tenancy-chain.dataflow.json"
  CURHTML="${CURIR%.json}.html"
  cp "$CURHTML" "$CURHTML.orig"
  python3 -c "
import pathlib,sys
p=pathlib.Path(sys.argv[1]); p.write_text(p.read_text().replace('append-only','append-onlyy',1))" "$CURHTML"
  expect_caught "a committed render that no longer matches its IR" \
    bash "$ROOT/scripts/check-repo.sh" --root "$ROOT"
  mv "$CURHTML.orig" "$CURHTML"
  expect_ok "the repository is clean again once the render is restored" \
    bash "$ROOT/scripts/check-repo.sh" --root "$ROOT"
else
  printf '  skipped: docs.diagram_is_current — archify is not installed at its pinned ref\n'
fi


printf '\n── tool registry (ADR-0066) ───────────────────────────────────────\n'
# knowledge.tool_registry_valid and knowledge.tool_registry_holds_no_version. Five cases,
# because the registry makes five distinct claims and a schema call proves only one of them.
#
# The pin-home pair is the interesting half. A registry whose entries merely VALIDATE is a
# registry that can still carry a version in two places, which is the exact defect it was
# created to close — RFC-0015, and the mechanism that put an end-of-life Node runtime into
# three repositories. Both directions are planted: two homes, and none.
TRG=tool-registry.yaml

kplant "a tool carrying both install.ref and pin_home" "$TRG" \
  "    pin_home: os
    network: always
    declared: 2026-09-05
    exit: >
      Loss costs nothing but a slower feedback loop" "    pin_home: os
    install:
      method: os-package
      ref: 0.2.82
    network: always
    declared: 2026-09-05
    exit: >
      Loss costs nothing but a slower feedback loop"

kplant "a tool with neither a pin nor a pin_home" "$TRG" \
  "    adopted_by: [ADR-0004, RUN-0001]
    pin_home: os" "    adopted_by: [ADR-0004, RUN-0001]"

# Two checks fire on this one — the cross-reference scanner in section 5 also walks this file
# — and that is deliberate rather than sloppy: adopted_by resolution must hold even if the
# generic scanner is ever narrowed to artifact directories, which is what section 8g asserts
# separately.
# The unresolvable id is ASSEMBLED rather than written, because section 5's cross-reference
# scanner walks this file too, and a literal four-digit id in the 9000 range here is itself a
# dangling reference — the
# gate caught exactly that on this case's first run.
NOSUCH="ADR-99$(printf '99')"
kplant "a tool whose adopted_by does not resolve" "$TRG" \
  "    adopted_by: [ADR-0065, RFC-0040, STD-0020]" "    adopted_by: [$NOSUCH]"

kplant "tool entries out of sorted order" "$TRG" \
  "  - id: act
" "  - id: zzz-act
"

kplant "a tool entry with no exit plan (REQ-0012 §5)" "$TRG" \
  "    exit: >
      No exit. ADR-0031's errata model" "    x_exit: >
      No exit. ADR-0031's errata model"

if [[ -n "$JUNIT_OUT" ]]; then
  {
    printf '<?xml version="1.0" encoding="UTF-8"?>\n'
    printf '<testsuites><testsuite name="paridyum-gates" tests="%d" failures="%d">\n' \
      "$((PASS + FAIL))" "$FAIL"
    printf '%s' "$CASES"
    printf '  </testsuite></testsuites>\n'
  } > "$JUNIT_OUT"
  printf '\nevidence: %s (%d cases)\n' "$JUNIT_OUT" "$((PASS + FAIL))"
fi

# ---------------------------------------------------------------------------
# The STD-0014 converter has its own suite, and it runs here so it is in the
# blocking tier rather than being something someone remembers to run.
#
# Its most valuable cases prove a claim this repository makes about itself: that an
# envelope satisfying test-envelope.schema.json is NOT thereby valid, because three
# cross-field rules are unenforceable by the vendored validator. They corrupt an
# envelope between build and write, assert the validator passes it, and assert the
# converter refuses it anyway.
# ---------------------------------------------------------------------------
expect_ok "junit-to-envelope converter (STD-0014)" \
  bash "$KB/templates/infra-ci/scripts/test-junit-to-envelope.sh"

# The ledger's own suite, in the blocking tier for the same reason. Its most
# valuable cases are the tamper ones: a mutated line, a deleted middle line, and an
# entry re-sealed correctly by the documented rule that STILL breaks the next entry —
# which is what makes the chain rather than the digest the load-bearing part.
expect_ok "evidence ledger append and verify (RFC-0034 §4)" \
  bash "$KB/templates/infra-ci/scripts/test-evidence-ledger.sh"

printf '\n'
if [[ $FAIL -gt 0 ]]; then
  printf '%s%d case(s) failed%s, %d passed\n' "$RED" "$FAIL" "$RST" "$PASS"; exit 1
fi
printf '%sAll %d check cases behaved as specified%s\n' "$GRN" "$PASS" "$RST"
