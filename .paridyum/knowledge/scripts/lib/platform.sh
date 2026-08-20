#!/usr/bin/env bash
#
# Platform detection and package-manager abstraction.
# Sourced by bootstrap.sh. Not executable on its own.
#
# Supported: macOS, Debian/Ubuntu, Fedora/RHEL, Arch, and WSL2 (which reports as
# whichever Linux distribution is installed). See ADR-0006.

# PLATFORM  darwin | linux
# DISTRO    macos | debian | fedora | arch | unknown
# PKG       brew | apt | dnf | pacman
# IS_WSL    true | false

detect_platform() {
  case "$(uname -s)" in
    Darwin)
      PLATFORM=darwin; DISTRO=macos; PKG=brew; IS_WSL=false
      ;;
    Linux)
      PLATFORM=linux
      # WSL2 reports Linux; /proc/version carries the marker. Worth knowing because
      # Docker and clock-skew behaviour differ from bare-metal Linux.
      if grep -qiE "microsoft|wsl" /proc/version 2>/dev/null; then IS_WSL=true; else IS_WSL=false; fi
      if [[ -r /etc/os-release ]]; then
        # shellcheck disable=SC1091
        . /etc/os-release
        case "${ID:-}${ID_LIKE:+ $ID_LIKE}" in
          *debian*|*ubuntu*) DISTRO=debian; PKG=apt    ;;
          *fedora*|*rhel*)   DISTRO=fedora; PKG=dnf    ;;
          *arch*)            DISTRO=arch;   PKG=pacman ;;
          *)                 DISTRO=unknown; PKG=""    ;;
        esac
      else
        DISTRO=unknown; PKG=""
      fi
      ;;
    MINGW*|MSYS*|CYGWIN*)
      # Native Windows bash. Not supported — ADR-0006 routes Windows through WSL2.
      printf 'This script does not support native Windows.\n' >&2
      printf 'Run scripts/bootstrap.ps1 from PowerShell first; it installs WSL2\n' >&2
      printf 'and hands off to this script inside the Linux environment.\n' >&2
      printf 'See runbooks/RUN-0001-workstation-setup.md.\n' >&2
      exit 1
      ;;
    *)
      printf 'Unsupported platform: %s\n' "$(uname -s)" >&2
      exit 1
      ;;
  esac
  export PLATFORM DISTRO PKG IS_WSL
}

platform_label() {
  case "$PLATFORM" in
    darwin) printf 'macOS %s (%s)' "$(sw_vers -productVersion 2>/dev/null || echo '?')" "$(uname -m)" ;;
    linux)
      local name; name="${PRETTY_NAME:-Linux}"
      if [[ "$IS_WSL" == true ]]; then printf '%s on WSL2 (%s)' "$name" "$(uname -m)"
      else printf '%s (%s)' "$name" "$(uname -m)"; fi
      ;;
  esac
}

# Map a logical tool name to this platform's package name.
# Empty result = not available from the package manager; install another way.
pkg_name_for() {
  local tool="$1"
  case "$PKG:$tool" in
    brew:kubectl)   echo "kubernetes-cli" ;;
    apt:kubectl)    echo "" ;;   # not in default apt repos; needs upstream repo
    dnf:kubectl)    echo "kubernetes-client" ;;
    pacman:kubectl) echo "kubectl" ;;

    brew:terraform) echo "terraform" ;;
    apt:terraform)  echo "" ;;   # HashiCorp apt repo required
    dnf:terraform)  echo "" ;;
    pacman:terraform) echo "terraform" ;;

    apt:just)       echo "just" ;;   # universe; older releases may lack it
    *:just)         echo "just" ;;

    *:*)            echo "${tool}" ;;
  esac
}

pkg_install() {
  local pkgs=("$@")
  [[ ${#pkgs[@]} -eq 0 ]] && return 0
  case "$PKG" in
    brew)   brew install "${pkgs[@]}" ;;
    apt)    sudo apt-get update -qq && sudo apt-get install -y "${pkgs[@]}" ;;
    dnf)    sudo dnf install -y "${pkgs[@]}" ;;
    pacman) sudo pacman -S --needed --noconfirm "${pkgs[@]}" ;;
    *)      return 1 ;;
  esac
}

pkg_manager_present() {
  case "$PKG" in
    brew)   command -v brew    >/dev/null 2>&1 ;;
    apt)    command -v apt-get >/dev/null 2>&1 ;;
    dnf)    command -v dnf     >/dev/null 2>&1 ;;
    pacman) command -v pacman  >/dev/null 2>&1 ;;
    *)      return 1 ;;
  esac
}

# Docker liveness. Never shell out to `docker info` — with the daemon stopped it
# blocks for minutes instead of failing. The socket check returns instantly.
docker_daemon_up() {
  [[ -S /var/run/docker.sock ]] && return 0
  [[ -S "$HOME/.docker/run/docker.sock" ]] && return 0
  [[ -n "${DOCKER_HOST:-}" && "$DOCKER_HOST" == unix://* && -S "${DOCKER_HOST#unix://}" ]] && return 0
  return 1
}

docker_install_hint() {
  case "$DISTRO" in
    macos)  echo "install Docker Desktop: brew install --cask docker" ;;
    debian) echo "install Docker Engine: https://docs.docker.com/engine/install/debian/" ;;
    fedora) echo "install Docker Engine: https://docs.docker.com/engine/install/fedora/" ;;
    arch)   echo "sudo pacman -S docker && sudo systemctl enable --now docker" ;;
    *)      echo "install Docker for your distribution" ;;
  esac
}
