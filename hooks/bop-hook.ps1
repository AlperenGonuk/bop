# Bop hook for Windows without Git Bash (Claude Code then runs hooks with PowerShell).
#
# Usage: bop-hook.ps1 <event> [app-folder]
#
# Same behavior as bop-hook.sh: Claude Code passes the event as JSON on stdin. If the user has
# installed the Bop app (~/.bop/bin/bop.exe), it runs `bop.exe hook <event>`, which reads that
# JSON and writes the pet's state to ~/.bop/state.json. No network, no download, no other files.
#
# A hook must never break or slow down Claude Code, so this script always exits 0 without output.
# If the app is not installed, it only reads stdin and exits. BOP_HOME replaces ~/.bop (for testing).
# The optional app folder is for developing Bop (the debug build folder instead of ~/.bop/bin).

param(
    [string]$HookEvent,
    [string]$AppDir
)

$bopHome = if ($env:BOP_HOME) { $env:BOP_HOME } else { Join-Path $env:USERPROFILE '.bop' }
if (-not $AppDir) { $AppDir = Join-Path $bopHome 'bin' }
$app = Join-Path $AppDir 'bop.exe'

if ($env:USERPROFILE -and (Test-Path -LiteralPath $app -PathType Leaf)) {
    & $app hook $HookEvent *> $null
} else {
    $null = [Console]::In.ReadToEnd()
}

exit 0
