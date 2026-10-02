# Bop hook for Windows without Git Bash (Claude Code then runs hooks with PowerShell).
#
# Usage: bop-hook.ps1 <app-folder> <event>
#
# Same behavior as bop-hook.sh: Claude Code passes the event as JSON on stdin. If the Bop app is
# installed in <app-folder>, it runs `bop.exe hook <event>`, which reads that JSON and writes the
# pet's state to ~/.bop/state.json. No network, no download, no other files.
#
# A hook must never break or slow down Claude Code, so this script always exits 0 without output.
# If the app is not installed, it only reads stdin and exits.

param(
    [string]$AppDir,
    [string]$HookEvent
)

$app = Join-Path $AppDir 'bop.exe'

if ($AppDir -and (Test-Path -LiteralPath $app -PathType Leaf)) {
    & $app hook $HookEvent *> $null
} else {
    $null = [Console]::In.ReadToEnd()
}

exit 0
