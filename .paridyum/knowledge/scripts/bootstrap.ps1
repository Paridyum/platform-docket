#Requires -Version 5.1
<#
.SYNOPSIS
  Paridyum workstation bootstrap for Windows.

.DESCRIPTION
  Paridyum development on Windows runs inside WSL2, not on native Windows (ADR-0006).
  This script prepares the Windows side only — WSL2, a distribution, Docker Desktop
  with WSL integration — then hands off to scripts/bootstrap.sh inside Linux, which
  is the single provisioning path shared with macOS and native Linux.

  It deliberately does NOT install Rust, gh, kubectl, helm or terraform on Windows.
  Those belong inside the WSL2 environment where the code is built and run.

.PARAMETER Check
  Verify only. Changes nothing. Exits non-zero if provisioning is incomplete.

.EXAMPLE
  .\bootstrap.ps1 -Check
#>

[CmdletBinding()]
param([switch]$Check)

$ErrorActionPreference = 'Stop'
$script:Failures = 0
$script:Warnings = 0

function Write-Ok    { param($m) Write-Host "  [ok]   $m" -ForegroundColor Green }
function Write-Warn  { param($m) Write-Host "  [warn] $m" -ForegroundColor Yellow; $script:Warnings++ }
function Write-Bad   { param($m) Write-Host "  [FAIL] $m" -ForegroundColor Red;    $script:Failures++ }
function Write-Step  { param($m) Write-Host "`n$m" }
function Write-Note  { param($m) Write-Host "         $m" -ForegroundColor DarkGray }

$Distro = 'Ubuntu'

# ---------------------------------------------------------------------------
Write-Step 'Platform'

$os = Get-CimInstance Win32_OperatingSystem
Write-Ok "$($os.Caption) (build $($os.BuildNumber))"

# WSL2 requires Windows 10 build 19041+. Below that, wsl --install does not exist.
if ([int]$os.BuildNumber -lt 19041) {
  Write-Bad "Windows build $($os.BuildNumber) is too old for WSL2 (need 19041+)"
  Write-Note 'Update Windows, or use a Linux or macOS workstation instead.'
  exit 1
}

if (-not ([Security.Principal.WindowsPrincipal] `
      [Security.Principal.WindowsIdentity]::GetCurrent()
    ).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  if (-not $Check) {
    Write-Bad 'Installing WSL2 requires an elevated PowerShell'
    Write-Note 'Re-run as Administrator, or use -Check to verify only.'
    exit 1
  }
  Write-Warn 'not elevated — install steps would be skipped'
}

# ---------------------------------------------------------------------------
Write-Step 'WSL2'

$wsl = Get-Command wsl.exe -ErrorAction SilentlyContinue
if (-not $wsl) {
  if ($Check) {
    Write-Bad 'wsl.exe not present'
  } else {
    Write-Host '    installing WSL2 with the default distribution'
    wsl.exe --install -d $Distro
    Write-Ok 'WSL2 installed'
    Write-Note 'REBOOT REQUIRED, then re-run this script to continue.'
    exit 0
  }
} else {
  Write-Ok 'wsl.exe present'

  # `wsl --list` emits UTF-16LE; decode it or every match silently fails.
  $prev = [Console]::OutputEncoding
  [Console]::OutputEncoding = [System.Text.Encoding]::Unicode
  $distros = (wsl.exe --list --quiet) 2>$null | ForEach-Object { $_.Trim() } | Where-Object { $_ }
  [Console]::OutputEncoding = $prev

  if (-not $distros) {
    if ($Check) {
      Write-Bad 'no WSL distribution installed'
    } else {
      Write-Host "    installing $Distro"
      wsl.exe --install -d $Distro
      Write-Ok "$Distro installed"
      Write-Note 'Complete the Linux user setup in the new window, then re-run.'
      exit 0
    }
  } else {
    Write-Ok "distributions: $($distros -join ', ')"

    $verLines = (wsl.exe --list --verbose) 2>$null
    if ($verLines -match '\s1\s*$') {
      Write-Warn 'a distribution is running on WSL1'
      Write-Note "wsl --set-version $Distro 2   (WSL1 lacks the kernel features Docker needs)"
    } else {
      Write-Ok 'distributions running WSL2'
    }
  }
}

# ---------------------------------------------------------------------------
Write-Step 'Docker Desktop'

$dockerCli = Get-Command docker.exe -ErrorAction SilentlyContinue
if ($dockerCli) {
  Write-Ok 'docker CLI present on Windows'
  Write-Note 'Enable Settings > Resources > WSL Integration for your distribution,'
  Write-Note 'otherwise docker is unavailable inside WSL where the build runs.'
} else {
  Write-Warn 'Docker Desktop not detected'
  Write-Note 'winget install Docker.DockerDesktop'
}

# ---------------------------------------------------------------------------
Write-Step 'Git line endings'

# Repos are shared with macOS and Linux. Windows checkouts must not rewrite LF to
# CRLF: shell scripts with CRLF fail inside WSL and in containers with an opaque
# "bad interpreter" error. See ADR-0006.
$autocrlf = (git config --global --get core.autocrlf) 2>$null
if ($autocrlf -eq 'false' -or $autocrlf -eq 'input') {
  Write-Ok "git core.autocrlf = $autocrlf"
} elseif ($Check) {
  Write-Warn "git core.autocrlf = $(if ($autocrlf) { $autocrlf } else { '<unset>' }) — should be false"
  Write-Note 'CRLF line endings break shell scripts inside WSL and containers.'
} else {
  git config --global core.autocrlf false
  Write-Ok 'git core.autocrlf = false (set)'
}

# ---------------------------------------------------------------------------
Write-Step 'Hand off to WSL2'

Write-Note 'Paridyum repos live on the Linux filesystem (~/Developer), never /mnt/c.'
Write-Note 'Cross-filesystem I/O is roughly an order of magnitude slower, and Linux'
Write-Note 'file permissions are not preserved on the Windows mount.'
Write-Host ''
Write-Host '  Continue inside WSL2:' -ForegroundColor Cyan
Write-Host ''
Write-Host '    wsl'
Write-Host '    git clone git@github.com:Paridyum/knowledge.git ~/Developer/paridyum/knowledge'
Write-Host '    ~/Developer/paridyum/knowledge/scripts/bootstrap.sh'
Write-Host ''

# ---------------------------------------------------------------------------
Write-Host ''
if ($script:Failures -gt 0) {
  Write-Host "$($script:Failures) failed, $($script:Warnings) warning(s)" -ForegroundColor Red
  exit 1
} elseif ($script:Warnings -gt 0) {
  Write-Host "Windows side complete with $($script:Warnings) warning(s)." -ForegroundColor Yellow
} else {
  Write-Host 'Windows side ready. Continue inside WSL2.' -ForegroundColor Green
}
