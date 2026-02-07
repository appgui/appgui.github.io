#!/usr/bin/env pwsh
# Windows Development Environment Setup
# Must be run as Administrator

$ErrorActionPreference = "Stop"
$ProgressPreference = 'SilentlyContinue'

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

  Write-Host "[1/5] Setting up Winget..." -ForegroundColor Green
  if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
    Write-Host "  Installing Winget..." -ForegroundColor Yellow
    irm https://raw.githubusercontent.com/asheroto/winget-install/master/winget-install.ps1 | iex
  }
  Write-Host "[DONE] Winget is ready." -ForegroundColor Gray

  Write-Host "[2/5] Setting up pwsh..." -ForegroundColor Green
  if (-not (Get-Command pwsh -ErrorAction SilentlyContinue)) {
    Write-Host "  Installing pwsh..." -ForegroundColor Yellow
    winget install --id Microsoft.PowerShell
  } else {
    Write-Host "  Upgrading pwsh..." -ForegroundColor Yellow
    winget upgrade --id Microsoft.PowerShell
  }
  # Add pwsh to PATH if not present
  $pwshPath = "C:\Program Files\PowerShell\7"
  $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
  if ($currentPath -notlike "*$pwshPath*") {
    [Environment]::SetEnvironmentVariable("Path", "$currentPath;$pwshPath", "Machine")
    $env:Path = [Environment]::GetEnvironmentVariable("Path", "Machine")
  }
  Write-Host "[DONE] pwsh is ready." -ForegroundColor Gray

  Write-Host "[3/5] Setting up Just..." -ForegroundColor Green
  if (-not (Get-Command just -ErrorAction SilentlyContinue)) {
    Write-Host "  Installing Just..." -ForegroundColor Yellow
    winget install --id casey.just -e --source winget
  } else {
    Write-Host "  Upgrading Just..." -ForegroundColor Yellow
    winget upgrade --id casey.just -e --source winget
  }
  Write-Host "[DONE] Just is ready." -ForegroundColor Gray

  Write-Host "[4/5] Setting up Git shell..." -ForegroundColor Green
  if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "  Installing Git..." -ForegroundColor Yellow
    winget install --id Git.Git -e --source winget
  } else {
    # Check if update is available
    $updateCheck = winget upgrade --id Git.Git -e --source winget
    if ($updateCheck -match "No applicable update found") {
      Write-Host "  Git is already up to date." -ForegroundColor Gray
    } else {
      Write-Host "  Upgrading Git..." -ForegroundColor Yellow
      # Check if Git processes are running
      $gitProcesses = Get-Process | Where-Object {$_.Path -like "*\Git\*"}
      if ($gitProcesses) {
        Write-Host "  Git processes are running and need to be terminated." -ForegroundColor Yellow
        Write-Host "  Press Enter to force terminate running processes..." -ForegroundColor Yellow
        $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
        $gitProcesses | Stop-Process -Force -ErrorAction SilentlyContinue
      }
      winget upgrade --id Git.Git -e --source winget
    }
  }
  # Add Git to PATH if not present
  $gitCmd = "C:\Program Files\Git\cmd"
  $gitUsrBin = "C:\Program Files\Git\usr\bin"
  $currentPath = [Environment]::GetEnvironmentVariable("Path", "Machine")
  if ($currentPath -notlike "*$gitCmd*") {
    $currentPath = "$currentPath;$gitCmd"
  }
  if ($currentPath -notlike "*$gitUsrBin*") {
    $currentPath = "$currentPath;$gitUsrBin"
  }
  [Environment]::SetEnvironmentVariable("Path", $currentPath, "Machine")
  $env:Path = [Environment]::GetEnvironmentVariable("Path", "Machine")
  Write-Host "[DONE] Git is ready." -ForegroundColor Gray

  Write-Host "[5/5] Setting up dependencies..." -ForegroundColor Green
  & "C:\\Program Files\\Git\\bin\\bash.exe" -c 'just windows_set_deps'
  Write-Host "[DONE] Tools are ready." -ForegroundColor Gray

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
