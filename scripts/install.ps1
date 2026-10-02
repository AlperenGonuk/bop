# Installs the Bop desktop app on Windows (Windows PowerShell 5.1 or PowerShell 7).
#
# Usage: powershell -NoProfile -ExecutionPolicy Bypass -File install.ps1 -DataDir <plugin-data-dir> [-Force]
#
# What it does, and nothing else:
#   1. Reads the plugin version and the GitHub repository from ../.claude-plugin/plugin.json.
#   2. If <DataDir>/bin/bop.exe already reports that version, stops (use -Force to reinstall).
#   3. Downloads SHA256SUMS and bop-<version>-windows-x64.exe from the GitHub release v<version>
#      (https://github.com/<repo>/releases/download/v<version>/). GitHub may redirect the download
#      to its file host (*.githubusercontent.com). No other address is contacted and nothing is sent.
#   4. Checks the file against SHA256SUMS; on a mismatch the file is deleted and nothing changes.
#   5. Closes a running pet (bop stop) and moves the new file to <DataDir>/bin/bop.exe.
#
# BOP_RELEASE_BASE_URL replaces the release address (for testing with a local server).
# Exit code: 0 installed or already up to date, 1 error.
# Keep this file ASCII only: Windows PowerShell 5.1 reads BOM-less files in the ANSI code page.

param(
    [Parameter(Mandatory = $true)][string]$DataDir,
    [switch]$Force
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

function Fail([string]$Message) {
    [Console]::Error.WriteLine("bop setup: $Message")
    exit 1
}

# Runs the app with arguments and returns its first output line, or $null when it fails or
# does not finish within $Seconds (an old app without --version would open the pet instead).
function Invoke-Bop([string]$Exe, [string]$Arguments, [int]$Seconds = 10) {
    try {
        $info = New-Object System.Diagnostics.ProcessStartInfo
        $info.FileName = $Exe
        $info.Arguments = $Arguments
        $info.UseShellExecute = $false
        $info.CreateNoWindow = $true
        $info.RedirectStandardOutput = $true
        $info.RedirectStandardError = $true
        $p = [System.Diagnostics.Process]::Start($info)
        $out = $p.StandardOutput.ReadToEndAsync()
        if (-not $p.WaitForExit($Seconds * 1000)) {
            try { $p.Kill() } catch {}
            return $null
        }
        if ($p.ExitCode -ne 0) { return $null }
        return ($out.Result -split "`r?`n")[0].Trim()
    } catch {
        return $null
    }
}

function Get-Download([string]$Url, [string]$OutFile) {
    try {
        Invoke-WebRequest -Uri $Url -OutFile $OutFile -UseBasicParsing -TimeoutSec 120
    } catch {
        Fail "download failed: $Url ($($_.Exception.Message))"
    }
}

try {
    $manifestPath = Join-Path $PSScriptRoot '..\.claude-plugin\plugin.json'
    $manifest = Get-Content -LiteralPath $manifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
} catch {
    Fail "cannot read the plugin manifest: $manifestPath"
}
$version = [string]$manifest.version
$repo = [string]$manifest.metadata.releaseRepo
if ($version -notmatch '^[0-9A-Za-z][0-9A-Za-z.+-]*$') { Fail "invalid plugin version: '$version'" }
if ($repo -notmatch '^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$') { Fail "invalid release repository: '$repo'" }

# Windows on ARM runs the x64 build through emulation.
$arch = $env:PROCESSOR_ARCHITEW6432
if (-not $arch) { $arch = $env:PROCESSOR_ARCHITECTURE }
if ($arch -ne 'AMD64' -and $arch -ne 'ARM64') { Fail "no prebuilt Bop app for Windows $arch (64-bit Windows is required)" }
$asset = "bop-$version-windows-x64.exe"

if (-not $DataDir) { Fail 'the data folder is empty' }
$bin = Join-Path $DataDir 'bin'
$exe = Join-Path $bin 'bop.exe'

if ((Test-Path -LiteralPath $exe -PathType Leaf) -and -not $Force) {
    if ((Invoke-Bop $exe '--version') -eq "bop $version") {
        Write-Output "Bop $version is already installed: $exe"
        exit 0
    }
}

if ($env:BOP_RELEASE_BASE_URL) {
    $base = $env:BOP_RELEASE_BASE_URL.TrimEnd('/')
} else {
    $base = "https://github.com/$repo/releases/download/v$version"
}

# Old Windows PowerShell may not offer TLS 1.2 by default.
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
} catch {}

New-Item -ItemType Directory -Force -Path $bin | Out-Null
$sumsFile = Join-Path $bin 'SHA256SUMS.download'
$newFile = Join-Path $bin 'bop.exe.download'

try {
    Get-Download "$base/SHA256SUMS" $sumsFile
    $expected = $null
    foreach ($line in Get-Content -LiteralPath $sumsFile) {
        if ($line -match '^([0-9A-Fa-f]{64}) [ *]?(\S+)\s*$' -and $Matches[2] -eq $asset) {
            $expected = $Matches[1].ToLowerInvariant()
            break
        }
    }
    if (-not $expected) { Fail "SHA256SUMS has no entry for $asset" }

    Get-Download "$base/$asset" $newFile
    $actual = (Get-FileHash -LiteralPath $newFile -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) {
        Remove-Item -LiteralPath $newFile -Force -ErrorAction SilentlyContinue
        Fail "checksum mismatch for $asset (expected $expected, got $actual); nothing was installed"
    }

    $wasRunning = $false
    if (Test-Path -LiteralPath $exe -PathType Leaf) {
        $wasRunning = (Invoke-Bop $exe 'stop') -eq 'pet closed'
        # A running exe cannot be overwritten on Windows, but it can be renamed.
        $old = "$exe.old"
        Remove-Item -LiteralPath $old -Force -ErrorAction SilentlyContinue
        if (Test-Path -LiteralPath $old) { $old = "$exe.old-$([DateTime]::UtcNow.Ticks)" }
        Move-Item -LiteralPath $exe -Destination $old
    }
    Move-Item -LiteralPath $newFile -Destination $exe
    # The closed pet needs a moment to exit; a leftover is removed on the next run.
    for ($i = 0; $i -lt 10; $i++) {
        $leftovers = @(Get-ChildItem -LiteralPath $bin -Filter 'bop.exe.old*' -ErrorAction SilentlyContinue)
        if ($leftovers.Count -eq 0) { break }
        $leftovers | Remove-Item -Force -ErrorAction SilentlyContinue
        Start-Sleep -Milliseconds 500
    }
} catch {
    Fail $_.Exception.Message
} finally {
    Remove-Item -LiteralPath $sumsFile, $newFile -Force -ErrorAction SilentlyContinue
}

$installed = Invoke-Bop $exe '--version'
if ($installed -ne "bop $version") { Fail "installed $exe, but it does not run ('$installed')" }
Write-Output "Installed Bop $version (SHA-256 verified): $exe"
if ($wasRunning) { Write-Output 'The pet was closed for the update. Run /bop to open it again.' }
exit 0
