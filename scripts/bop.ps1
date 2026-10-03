# Runs the Bop app with the given arguments (Windows without Git Bash).
#
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File bop.ps1 <arguments>
#
# Same behavior as bop.sh: the plugin does not download or install the app. The user downloads
# it from the GitHub releases page and opens it once; it then copies itself to
# ~/.bop/bin/bop.exe. This script only runs that copy and passes its output and exit code
# through. BOP_HOME replaces ~/.bop (for testing).

$bopHome = if ($env:BOP_HOME) { $env:BOP_HOME } else { Join-Path $env:USERPROFILE '.bop' }
$app = Join-Path (Join-Path $bopHome 'bin') 'bop.exe'

if (-not (Test-Path -LiteralPath $app -PathType Leaf)) {
    [Console]::Error.WriteLine('bop: the Bop app is not installed. Download it from https://github.com/AlperenGonuk/bop/releases/latest and open it once.')
    exit 127
}

& $app @args
exit $LASTEXITCODE
