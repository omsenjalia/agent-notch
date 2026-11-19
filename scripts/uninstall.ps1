# Per-user uninstaller for agent-notch - the counterpart to scripts\install.ps1.
#
#   powershell -ExecutionPolicy Bypass -File scripts\uninstall.ps1
#   powershell -ExecutionPolicy Bypass -File scripts\uninstall.ps1 -PurgeConfig
#
# Removes the autostart entry, shortcuts and the installed files. Settings in
# %APPDATA%\agent-notch are kept unless -PurgeConfig is passed, so a reinstall comes
# back exactly as it was left.
[CmdletBinding()]
param(
    [switch]$PurgeConfig
)

$ErrorActionPreference = "SilentlyContinue"

Get-Process agent-notch | Stop-Process -Force
Start-Sleep -Milliseconds 500

# Autostart entry, plus the Task Manager approval leftover next to it.
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "agent-notch"
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run" -Name "agent-notch"

$group = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs\agent-notch"
Remove-Item (Join-Path $group "agent-notch.lnk") -Force
Remove-Item $group -Force
Remove-Item (Join-Path ([Environment]::GetFolderPath("Desktop")) "agent-notch.lnk") -Force

$dest = Join-Path $env:LOCALAPPDATA "Programs\agent-notch"
if (Test-Path $dest) { Remove-Item $dest -Recurse -Force }

Write-Host "agent-notch uninstalled."
if ($PurgeConfig) {
    Remove-Item (Join-Path $env:APPDATA "agent-notch") -Recurse -Force
    Write-Host "Configuration removed (%APPDATA%\agent-notch)."
} else {
    Write-Host "Configuration kept at %APPDATA%\agent-notch (use -PurgeConfig to remove)."
}
