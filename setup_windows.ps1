#!/usr/bin/env powershell
# Windows Development Environment Setup
# Must be run as Administrator

$ErrorActionPreference = "Stop"
$ProgressPreference = 'Continue'

function Pause-Script {
  Write-Host "`nPress Enter to continue..." -ForegroundColor Yellow
  while ($Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown").VirtualKeyCode -ne 13) {
    # Wait for Enter key (key code 13)
  }
}

try {
  $isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
  if (-not $isAdmin) {
    Write-Host "Elevating privileges..." -ForegroundColor Yellow
    # Preserve the working directory
    $workingDir = Get-Location
    $args = @("-NoProfile", "-ExecutionPolicy", "Bypass", "-WorkingDirectory", "`"$workingDir`"", "-File", "`"$PSCommandPath`"")
    Start-Process powershell -Verb RunAs -ArgumentList $args
    exit
  }

  Write-Host "`n=== Windows Development Environment Setup ===`n" -ForegroundColor Cyan

  Write-Host "[1/6] Setting up Winget..." -ForegroundColor Green
  if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    Write-Host "  Installing Winget..." -ForegroundColor Yellow
    irm https://raw.githubusercontent.com/asheroto/winget-install/master/winget-install.ps1 | iex
  }
  Write-Host "  Winget is ready." -ForegroundColor Gray

  Write-Host "[2/6] Setting up Dprint..." -ForegroundColor Green
  if (-not (Get-Command dprint -ErrorAction SilentlyContinue)) {
    Write-Host "  Installing Dprint..." -ForegroundColor Yellow
    iwr https://dprint.dev/install.ps1 -useb | iex
  }
  Write-Host "  Dprint is ready." -ForegroundColor Gray

  Write-Host "[3/6] Setting up Just..." -ForegroundColor Green
  if (-not (Get-Command just -ErrorAction SilentlyContinue)) {
    Write-Host "  Installing Just..." -ForegroundColor Yellow
    winget install --id casey.just -e --source winget --accept-package-agreements --accept-source-agreements
  }
  Write-Host "  Just is ready." -ForegroundColor Gray

  Write-Host "[4/6] Setting up Rustup..." -ForegroundColor Green
  if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    Write-Host "  Installing Rustup..." -ForegroundColor Yellow
    winget install -e --id Rustlang.Rustup
  }
  Write-Host "  Rustup is ready." -ForegroundColor Gray

  Write-Host "[4/6] Setting up Git..." -ForegroundColor Green
  if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "  Installing Git..." -ForegroundColor Yellow
    winget install --id Git.Git -e --source winget --accept-package-agreements
  }
  Write-Host "  Git is ready." -ForegroundColor Gray

  Write-Host "[4/6] Setting up tools..." -ForegroundColor Green
  just setup_host_tools
  Write-Host "  Tools are ready." -ForegroundColor Gray

  Write-Host ""
  Write-Host "=== Setup complete ===" -ForegroundColor Cyan
  Write-Host ""

  Pause-Script
}
catch {
  Write-Host "`n=== SETUP FAILED ===" -ForegroundColor Red
  Write-Host "Error: $_" -ForegroundColor Red
  Pause-Script
  exit 1
}
