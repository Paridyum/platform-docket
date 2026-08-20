#!/usr/bin/env bash
#
# Secret scan (C-0004) — the single implementation of repo.secret_scan_clean.
#
#   ./secret-scan.sh --staged     what is about to be committed   (pre-commit)
#   ./secret-scan.sh --history    the full commit history         (CI)
#   ./secret-scan.sh --dir [path] a working tree, no git required (ad hoc)
#
# WHY THIS EXISTS RATHER THAN gitleaks/gitleaks-action
#
# GitHub's own secret scanning is unavailable on Free private repositories
# (ADR-0004), so gitleaks is the substitute. The obvious way to run it in CI is
# gitleaks/gitleaks-action, and that was tried first. Three problems, any one of
# which is disqualifying:
#
#   1. It requires GITLEAKS_LICENSE for organisation accounts. The key is free, but
#      it is obtained by submitting name/email/company to a form, and the action
#      validates it against a remote service on every run. That is a network
#      dependency that can fail the build for reasons unrelated to the code.
#   2. gitleaks-action@v2 stops working on 2026-09-16, when Node 20 is removed from
#      GitHub-hosted runners. v3 exists but carries the same licence requirement.
#   3. It is a second implementation. The pre-commit hook ran the gitleaks binary
#      while CI ran the action, so "the secret scan" meant two different code paths
#      with two different configs and two different failure modes.
#
# gitleaks itself is MIT. Only the action wrapper is licensed. So this script runs
# the binary, and every layer of the enforcement ladder calls this script. Local and
# CI cannot disagree, because there is only one thing to disagree with.
#
# SUPPLY CHAIN
#
# The version is pinned and every download is checksum-verified before it executes.
# ADR-0010 requires third-party actions pinned to a SHA because "a mutable tag is
# remote code execution with extra steps"; an unverified tarball is the same hazard
# wearing a different hat. A platform with no pinned checksum below is a hard
# failure, never an unverified download.

set -uo pipefail

GITLEAKS_VERSION="${GITLEAKS_VERSION:-8.30.1}"

# sha256 of gitleaks_<version>_<platform>.tar.gz, from the release's checksums.txt.
# Update all four together when bumping GITLEAKS_VERSION.
sha_for() {
  case "$1" in
    linux_x64)    printf '551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb' ;;
    linux_arm64)  printf 'e4a487ee7ccd7d3a7f7ec08657610aa3606637dab924210b3aee62570fb4b080' ;;
    darwin_arm64) printf 'b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5' ;;
    darwin_x64)   printf 'dfe101a4db2255fc85120ac7f3d25e4342c3c20cf749f2c20a18081af1952709' ;;
    *)            return 1 ;;
  esac
}

if [[ -t 1 ]]; then RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; RST=$'\033[0m'
else RED=""; GRN=""; YLW=""; RST=""; fi
die() { printf '%s✘%s %s\n' "$RED" "$RST" "$1" >&2; exit 1; }

MODE="${1:---history}"
TARGET="${2:-.}"

# The config travels with the knowledge base, so resolve it relative to the repo
# root rather than the caller's cwd. GITLEAKS_CONFIG overrides, matching gitleaks'
# own precedence order.
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
CONFIG="${GITLEAKS_CONFIG:-$ROOT/.gitleaks.toml}"
[[ -f "$CONFIG" ]] || die "no gitleaks config at $CONFIG (C-0004)"

# --- resolve a gitleaks binary ----------------------------------------------
# A gitleaks already on PATH is preferred: it is what the developer sees locally,
# and honouring it keeps `brew upgrade gitleaks` from being silently ignored. Only
# when there is none do we download, which is the CI path.
resolve_gitleaks() {
  if command -v gitleaks >/dev/null 2>&1; then
    GITLEAKS="$(command -v gitleaks)"
    return 0
  fi

  case "$(uname -s)" in
    Darwin) os=darwin ;;
    Linux)  os=linux ;;
    *) die "unsupported OS $(uname -s) — install gitleaks manually and re-run" ;;
  esac
  case "$(uname -m)" in
    arm64|aarch64) arch=arm64 ;;
    x86_64|amd64)  arch=x64 ;;
    *) die "unsupported architecture $(uname -m) — install gitleaks manually and re-run" ;;
  esac
  plat="${os}_${arch}"

  want="$(sha_for "$plat")" \
    || die "no pinned checksum for $plat — refusing to run an unverified download (C-0004)"

  # Cached per version+platform so repeated CI steps and repeated local runs each
  # download once. Not in the repo: it must never become a tracked binary.
  cache="${GITLEAKS_CACHE_DIR:-${TMPDIR:-/tmp}/paridyum-gitleaks}/${GITLEAKS_VERSION}-${plat}"
  GITLEAKS="$cache/gitleaks"
  [[ -x "$GITLEAKS" ]] && return 0

  mkdir -p "$cache" || die "cannot create cache dir $cache"
  tar="$cache/gitleaks.tar.gz"
  url="https://github.com/gitleaks/gitleaks/releases/download/v${GITLEAKS_VERSION}/gitleaks_${GITLEAKS_VERSION}_${plat}.tar.gz"

  printf 'Fetching gitleaks %s (%s)\n' "$GITLEAKS_VERSION" "$plat"
  curl -sSfL --retry 3 --retry-delay 2 -o "$tar" "$url" \
    || die "download failed: $url"

  # Verify BEFORE extraction, so a tampered archive is never even unpacked.
  got="$(
    if command -v sha256sum >/dev/null 2>&1; then sha256sum "$tar" | cut -d' ' -f1
    else shasum -a 256 "$tar" | cut -d' ' -f1; fi
  )"
  if [[ "$got" != "$want" ]]; then
    rm -f "$tar"
    die "checksum mismatch for gitleaks_${GITLEAKS_VERSION}_${plat}.tar.gz
     expected $want
     got      $got
   Refusing to execute it. If you bumped GITLEAKS_VERSION, update sha_for()."
  fi

  tar -xzf "$tar" -C "$cache" gitleaks || die "extract failed"
  chmod +x "$GITLEAKS"
  rm -f "$tar"
}

resolve_gitleaks

# --- repo.no_inline_scan_suppressions (C-0004, ADR-0017) ---------------------
# --ignore-gitleaks-allow makes the comment ineffective; this makes it visible.
# Without it a stale `# gitleaks:allow` sits in the tree looking like protection
# someone relied on, and silently becomes a bypass again the day the flag is dropped.
# `infisical-scan:ignore` is included because Infisical's scanner honours it, so the
# same comment would survive a migration between engines.
#
# The exclusions are load-bearing, not tidiness: this script, .gitleaksignore and the
# ADRs must all name the strings in order to prohibit them, so scanning those files
# makes the check self-inflicting — permanently red and unfixable. Same trap as
# DECISION-DEBT.md in check-knowledge.sh section 2.
if git -C "$ROOT" rev-parse --git-dir >/dev/null 2>&1; then
  # `:!.gitleaksignore` matches the ROOT file only. Every repository vendors the knowledge
  # snapshot at `.paridyum/knowledge/` (ADR-0001), whose own `.gitleaksignore` DOCUMENTS the
  # `gitleaks:allow` string this check forbids — so every consuming repository failed its secret
  # scan on the documentation of the rule. The snapshot is a copy of a repository that scans
  # itself, so excluding it loses no coverage. Found on tool-forge's first CI run.
  sup="$(git -C "$ROOT" grep -nE 'gitleaks:allow|infisical-scan:ignore' -- \
          ':!*.md' ':!*.gitleaksignore' ':!*secret-scan.sh' ':!check-registry.yaml' \
          ':!.paridyum/knowledge/*' \
          2>/dev/null || true)"
  if [[ -n "$sup" ]]; then
    printf '%s✘%s inline scan suppression found — use a fingerprint in .gitleaksignore instead (C-0004, ADR-0017)\n' "$RED" "$RST" >&2
    printf '%s\n' "$sup" | sed 's/^/    /' >&2
    exit 1
  fi
fi

# --- scan --------------------------------------------------------------------
# `detect` and `protect` were deprecated in gitleaks v8.19.0 and are hidden from
# --help. They still function, which is exactly why they are easy to leave in place
# for years. The current commands are `git`, `dir` and `stdin`.
#
# --ignore-gitleaks-allow is load-bearing. Without it, appending `# gitleaks:allow`
# to a line makes the scan exit 0 — verified: the same planted password passes with
# the comment and fails without it. That is a one-line bypass of C-0004 available in
# every layer, and precisely the reflex an agent reaches for when a scan goes red.
#
# The escape hatch is not removed, it is moved somewhere reviewable: a fingerprint in
# .gitleaksignore, which is tracked, appears in a diff, and names the exact finding
# rather than blanket-permitting a line. Same reasoning as ADR-0002 on making the
# exception visible. See ADR-0017.
#
# Note for anyone evaluating a different engine: Infisical's scanner has the identical
# hatch under the name `infisical-scan:ignore`, so this is a property of the category,
# not of gitleaks.
COMMON=(--no-banner --redact --config "$CONFIG" --ignore-gitleaks-allow)

case "$MODE" in
  --staged)
    # Pre-commit. Scans the staged diff only, so it is fast enough to run on every
    # commit — which is the whole point of layer 2 of the ladder (ADR-0007).
    "$GITLEAKS" git --staged "${COMMON[@]}" "$ROOT" || exit 1
    ;;
  --history)
    # CI. Scans every reachable commit, so a secret introduced and then "removed"
    # in a later commit is still caught — it is still in the history, and history is
    # what an attacker clones. This requires a full clone: with fetch-depth 1 or 2
    # the scan silently covers only those commits and still reports success.
    depth="$(git -C "$ROOT" rev-list --count HEAD 2>/dev/null || echo 0)"
    if [[ -f "$ROOT/.git/shallow" ]]; then
      printf '%s!%s shallow clone — only %s commit(s) will be scanned. CI needs fetch-depth: 0 for a real history scan.\n' \
        "$YLW" "$RST" "$depth"
    fi
    "$GITLEAKS" git "${COMMON[@]}" "$ROOT" || exit 1
    ;;
  --dir)
    "$GITLEAKS" dir "${COMMON[@]}" "$TARGET" || exit 1
    ;;
  *)
    die "unknown mode '$MODE' (expected --staged, --history or --dir)"
    ;;
esac

printf '%s✔%s no secrets found (%s, gitleaks %s)\n' \
  "$GRN" "$RST" "${MODE#--}" "$("$GITLEAKS" version 2>/dev/null || echo "$GITLEAKS_VERSION")"
