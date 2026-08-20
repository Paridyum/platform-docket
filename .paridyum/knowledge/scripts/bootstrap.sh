#!/usr/bin/env bash
#
# Paridyum workstation bootstrap — macOS, Linux, and WSL2.
# See runbooks/RUN-0001-workstation-setup.md. Windows: run bootstrap.ps1 first.
#
#   ./bootstrap.sh          provision (idempotent, safe to re-run)
#   ./bootstrap.sh --check  verify only, change nothing; non-zero exit if incomplete
#
# Superseded by `forge doctor` once tool-forge ships. Until then this is the verifier.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/platform.sh
source "$SCRIPT_DIR/lib/platform.sh"

CHECK_ONLY=false
[[ "${1:-}" == "--check" ]] && CHECK_ONLY=true

DEV_ROOT="$HOME/Developer"
PARIDYUM="$DEV_ROOT/paridyum"
GH_SCOPES_REQUIRED=(repo workflow admin:org)

if [[ -t 1 ]]; then
  RED=$'\033[31m'; GRN=$'\033[32m'; YLW=$'\033[33m'; DIM=$'\033[2m'; RST=$'\033[0m'
else
  RED=""; GRN=""; YLW=""; DIM=""; RST=""
fi
FAILURES=0
WARNINGS=0

ok()   { printf '  %s✔%s %s\n' "$GRN" "$RST" "$1"; }
warn() { printf '  %s!%s %s\n' "$YLW" "$RST" "$1"; WARNINGS=$((WARNINGS + 1)); }
bad()  { printf '  %s✘%s %s\n' "$RED" "$RST" "$1"; FAILURES=$((FAILURES + 1)); }
step() { printf '\n%s\n' "$1"; }
note() { printf '    %s%s%s\n' "$DIM" "$1" "$RST"; }

# --------------------------------------------------------------------------
detect_platform
step "Platform"
ok "$(platform_label)"

if [[ -z "$PKG" ]]; then
  bad "no supported package manager (need brew, apt, dnf, or pacman)"
  note "install tools manually per RUN-0001, then re-run with --check"
elif pkg_manager_present; then
  ok "package manager: $PKG"
else
  if [[ "$PKG" == brew ]]; then
    bad "homebrew missing — install from https://brew.sh, then re-run"
    exit 1
  fi
  bad "$PKG not found on PATH"
fi

[[ "$IS_WSL" == true ]] && note "WSL2: keep repos on the Linux filesystem (~/), never /mnt/c — see RUN-0001"

# --------------------------------------------------------------------------
step "Directory layout"

for d in paridyum personal experiments archived scratch; do
  if [[ -d "$DEV_ROOT/$d" ]]; then
    ok "~/Developer/$d"
  elif $CHECK_ONLY; then
    bad "~/Developer/$d missing"
  else
    mkdir -p "$DEV_ROOT/$d"; ok "~/Developer/$d (created)"
  fi
done

# Mirrors the repository class prefixes in constitution/C-0002-naming.md
missing_sub=0
for d in platform services libs tools infra specs templates; do
  if [[ ! -d "$PARIDYUM/$d" ]]; then
    if $CHECK_ONLY; then missing_sub=$((missing_sub + 1)); else mkdir -p "$PARIDYUM/$d"; fi
  fi
done
if [[ $missing_sub -gt 0 ]]; then
  bad "$missing_sub paridyum/ subdirectories missing"
else
  ok "~/Developer/paridyum/{platform,services,libs,tools,infra,specs,templates}"
fi

if [[ "$PLATFORM" == darwin && -d "$HOME/Library/Mobile Documents/com~apple~CloudDocs/Documents" ]]; then
  warn "iCloud Desktop & Documents sync appears enabled"
  note "harmless for ~/Developer, but never put repos in ~/Documents — see RUN-0001"
fi

# --------------------------------------------------------------------------
step "Toolchain"

need=()
for tool in git gh kubectl helm terraform just; do
  if command -v "$tool" >/dev/null 2>&1; then
    ok "$tool"
  else
    pkgname="$(pkg_name_for "$tool")"
    if [[ -z "$pkgname" ]]; then
      # Not packaged for this platform; needs a vendor repository.
      bad "$tool missing — not in $PKG default repos"
      case "$tool" in
        kubectl)   note "add the Kubernetes apt repo: https://kubernetes.io/docs/tasks/tools/" ;;
        terraform) note "add the HashiCorp repo: https://developer.hashicorp.com/terraform/install" ;;
      esac
    else
      need+=("$pkgname")
    fi
  fi
done

if [[ ${#need[@]} -gt 0 ]]; then
  if $CHECK_ONLY; then
    bad "missing: ${need[*]}"
  else
    printf '    installing: %s\n' "${need[*]}"
    pkg_install "${need[@]}"
    for p in "${need[@]}"; do ok "$p (installed)"; done
  fi
fi

if command -v docker >/dev/null 2>&1; then
  ok "docker $(docker --version 2>/dev/null | awk '{gsub(",","",$3); print $3}')"
  if docker_daemon_up; then
    ok "docker daemon responding"
  else
    warn "docker installed but daemon not running"
    [[ "$PLATFORM" == linux && "$IS_WSL" == false ]] && note "sudo systemctl start docker"
    [[ "$PLATFORM" == darwin ]] && note "start Docker Desktop"
    [[ "$IS_WSL" == true ]] && note "enable WSL integration in Docker Desktop settings"
  fi
else
  bad "docker missing — $(docker_install_hint)"
fi

# --------------------------------------------------------------------------
step "Rust toolchain"

# rustup, never a distro/brew `rust` package — those ignore rust-toolchain.toml
# and will silently disagree with CI about the compiler version. See ADR-0005.
if command -v rustup >/dev/null 2>&1; then
  ok "rustup $(rustup --version 2>/dev/null | head -1 | awk '{print $2}')"
  ok "rustc $(rustc --version | awk '{print $2}')"
elif $CHECK_ONLY; then
  bad "rustup missing"
else
  printf '    installing rustup (stable)\n'
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile default
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
  ok "rustup (installed)"
fi

case "$PKG" in
  brew)   brew list rust      >/dev/null 2>&1 && { warn "Homebrew 'rust' shadows rustup on PATH"; note "brew uninstall rust"; } ;;
  apt)    dpkg -s rustc       >/dev/null 2>&1 && { warn "apt 'rustc' shadows rustup on PATH"; note "sudo apt remove rustc cargo"; } ;;
  dnf)    rpm -q rust         >/dev/null 2>&1 && { warn "dnf 'rust' shadows rustup on PATH"; note "sudo dnf remove rust cargo"; } ;;
  pacman) pacman -Qi rust     >/dev/null 2>&1 && { warn "pacman 'rust' shadows rustup on PATH"; note "sudo pacman -R rust"; } ;;
esac

if command -v cargo >/dev/null 2>&1; then
  # sqlx-cli on Linux needs a linker and TLS headers; rustls avoids the openssl-dev dep.
  for spec in "sqlx-cli:sqlx" "cargo-chef:cargo-chef" "cargo-deny:cargo-deny" "cargo-audit:cargo-audit"; do
    crate="${spec%%:*}"; bin="${spec##*:}"
    if command -v "$bin" >/dev/null 2>&1 || cargo install --list 2>/dev/null | grep -q "^$crate "; then
      ok "$crate"
    elif $CHECK_ONLY; then
      bad "$crate missing"
    else
      printf '    cargo install %s\n' "$crate"
      if [[ "$crate" == "sqlx-cli" ]]; then
        cargo install sqlx-cli --no-default-features --features postgres,rustls
      else
        cargo install "$crate"
      fi
      ok "$crate (installed)"
    fi
  done
fi

# --------------------------------------------------------------------------
step "GitHub authentication"

if ! command -v gh >/dev/null 2>&1; then
  bad "gh missing"
elif gh auth status >/dev/null 2>&1; then
  ok "gh authenticated as $(gh api user --jq .login 2>/dev/null || echo '?')"

  scopes=$(gh auth status 2>&1 | sed -n "s/.*Token scopes: //p" | tr -d "'" | tr ',' ' ')
  missing_scopes=()
  for s in "${GH_SCOPES_REQUIRED[@]}"; do
    grep -qw -- "$s" <<<"$scopes" || missing_scopes+=("$s")
  done

  if [[ ${#missing_scopes[@]} -eq 0 ]]; then
    ok "token scopes sufficient for forge"
  else
    # Not auto-refreshed: `gh auth refresh` opens a browser and blocks.
    warn "token missing scope(s): ${missing_scopes[*]}"
    note "run: gh auth refresh -h github.com -s $(IFS=,; echo "${missing_scopes[*]}")"
    note "'workflow' is required to push .github/workflows/ — its absence fails at"
    note "push time with an error that does not mention scopes. See RUN-0001."
  fi

  if gh api orgs/Paridyum >/dev/null 2>&1; then
    plan=$(gh api orgs/Paridyum --jq '.plan.name // "unknown"' 2>/dev/null || echo unknown)
    ok "Paridyum org reachable (plan: $plan)"
    if [[ "$plan" == "free" ]]; then
      warn "Free plan: private repos cannot require status checks before merge"
      note "merge-blocking is unavailable; the agent hooks are the live gate — ADR-0004, ADR-0007"
    fi
  else
    warn "Paridyum org not reachable (needs read:org or admin:org)"
  fi
elif [[ ! -t 1 ]]; then
  # On macOS gh stores its token in the Keychain; a detached / non-TTY process cannot
  # unlock it and reports "not authenticated" even when the login is valid.
  warn "cannot read gh credentials (non-interactive session)"
  note "re-run in a terminal to verify authentication"
else
  bad "gh not authenticated — run: gh auth login"
fi

# --------------------------------------------------------------------------
step "Git configuration"

# Never overwritten: clobbering identity or signing config is destructive and
# unrecoverable from here. Reported only.
git_check() {
  local key="$1" want="$2" val
  val=$(git config --global --get "$key" 2>/dev/null || true)
  if [[ -z "$val" ]]; then
    warn "git config $key unset — expected: ${want:-<your value>}"
  elif [[ -n "$want" && "$val" != "$want" ]]; then
    warn "git config $key = '$val' (expected '$want')"
  else
    ok "git config $key = $val"
  fi
}

git_check user.name ""
git_check user.email ""
git_check init.defaultBranch main
git_check pull.rebase true
git_check commit.gpgsign true
git_check gpg.format ssh
git_check user.signingkey ""

shopt -s nullglob
ssh_keys=("$HOME"/.ssh/*.pub)
shopt -u nullglob

# An agent-served signing key has NO FILE ON DISK, which is the shape RUN-0001
# recommends. Checking only for ~/.ssh/id_ed25519.pub warned on the recommended
# configuration and told the reader to generate a second, redundant key — a check
# that reports a defect where none exists trains people to ignore it, which is worse
# than not checking. `user.signingkey` carrying a `key::` literal that a reachable
# agent actually serves is the positive signal. Added 2026-08-15.
signing_key="$(git config --global user.signingkey 2>/dev/null || true)"
agent_signing=0
if [[ "$signing_key" == key::* ]] && ssh-add -l >/dev/null 2>&1; then
  agent_signing=1
fi

if [[ $agent_signing -eq 1 ]]; then
  ok "signing key served by an ssh agent (no private key on disk)"
  note "must be registered on GitHub as a SIGNING key, separately from any auth key"
elif [[ ${#ssh_keys[@]} -eq 0 ]]; then
  warn "no ssh public key — ssh-keygen -t ed25519 -C '<email>'"
elif [[ -f "$HOME/.ssh/id_ed25519.pub" ]]; then
  ok "ssh key ~/.ssh/id_ed25519.pub present"
  note "must be registered on GitHub TWICE — as an auth key AND as a signing key"
else
  ok "ssh key(s) present: ${ssh_keys[*]##*/}"
  warn "no ed25519 key — recommended for SSH commit signing"
  note "ssh-keygen -t ed25519 -C '<email>' (keeps existing keys for auth)"
fi

# --------------------------------------------------------------------------
step "Knowledge base and agent harness"

if [[ -d "$PARIDYUM/knowledge/.git" ]]; then
  ok "knowledge repo cloned"
  behind=$(git -C "$PARIDYUM/knowledge" rev-list --count HEAD..@{u} 2>/dev/null || echo 0)
  [[ "$behind" -gt 0 ]] && warn "knowledge is $behind commit(s) behind origin — git pull"
elif [[ -d "$PARIDYUM/knowledge" ]]; then
  warn "knowledge/ exists but is not a git repo yet (pre-first-push)"
else
  bad "knowledge/ missing"
  note "git clone git@github.com:Paridyum/knowledge.git $PARIDYUM/knowledge"
fi

if command -v claude >/dev/null 2>&1; then
  ok "claude code installed"
  note "install the harness: /plugin marketplace add Paridyum/knowledge"
  note "                     /plugin install paridyum-platform@paridyum"
else
  warn "claude code not installed — the agent enforcement hooks (ADR-0007) cannot run"
fi

if command -v jq >/dev/null 2>&1; then
  ok "jq (required by the enforcement hooks)"
elif $CHECK_ONLY; then
  bad "jq missing — enforcement hooks parse their input with it"
else
  pkg_install "$(pkg_name_for jq)" && ok "jq (installed)"
fi

if command -v forge >/dev/null 2>&1; then
  forge_path=$(command -v forge)
  if [[ "$forge_path" == "$HOME/.cargo/bin/forge" ]]; then
    ok "forge installed"
  else
    # Foundry ships a `forge` binary. See constitution/C-0002-naming.md.
    warn "a 'forge' on PATH is not ours: $forge_path"
    note "likely Foundry — ours installs to ~/.cargo/bin/forge. See C-0002."
  fi
else
  note "forge not installed yet — tool-forge is not built (expected pre-v0.1)"
fi

# --------------------------------------------------------------------------
printf '\n'
if [[ $FAILURES -gt 0 ]]; then
  printf '%s%d failed%s, %d warning(s)\n' "$RED" "$FAILURES" "$RST" "$WARNINGS"
  exit 1
elif [[ $WARNINGS -gt 0 ]]; then
  printf '%sComplete with %d warning(s).%s\n' "$YLW" "$WARNINGS" "$RST"
else
  printf '%sWorkstation ready.%s\n' "$GRN" "$RST"
fi
