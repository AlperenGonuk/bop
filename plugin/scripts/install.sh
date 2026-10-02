#!/bin/sh
# Installs the Bop desktop app on macOS or Linux (also works in Git Bash on Windows).
#
# Usage: sh install.sh <plugin-data-dir> [--force]
#
# What it does, and nothing else:
#   1. Reads the plugin version and the GitHub repository from ../.claude-plugin/plugin.json.
#   2. If <data-dir>/bin/bop already reports that version, stops (use --force to reinstall).
#   3. Downloads SHA256SUMS and bop-<version>-<os>-<arch> from the GitHub release v<version>
#      (https://github.com/<repo>/releases/download/v<version>/) with curl or wget. GitHub may
#      redirect the download to its file host (*.githubusercontent.com). No other address is
#      contacted and nothing is sent.
#   4. Checks the file against SHA256SUMS; on a mismatch the file is deleted and nothing changes.
#   5. Closes a running pet (bop stop), marks the file executable and moves it to
#      <data-dir>/bin/bop in one step.
#
# BOP_RELEASE_BASE_URL replaces the release address (for testing with a local server).
# Exit code: 0 installed or already up to date, 1 error.

set -u

fail() {
    echo "bop setup: $*" >&2
    exit 1
}

[ $# -ge 1 ] && [ -n "$1" ] || fail "usage: sh install.sh <plugin-data-dir> [--force]"
data_dir=$1
force=0
[ "${2:-}" = "--force" ] && force=1

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd) || fail "cannot find the script folder"
manifest="$script_dir/../.claude-plugin/plugin.json"
[ -f "$manifest" ] || fail "cannot read the plugin manifest: $manifest"

json_string() {
    sed -n "s/^[[:space:]]*\"$1\"[[:space:]]*:[[:space:]]*\"\([^\"]*\)\".*/\1/p" "$manifest" | head -n 1
}
version=$(json_string version)
repo=$(json_string releaseRepo)
case $version in
    ''|[!0-9A-Za-z]*|*[!0-9A-Za-z.+-]*) fail "invalid plugin version: '$version'" ;;
esac
case $repo in
    */*/*|/*|*/|*[!A-Za-z0-9_./-]*|'') fail "invalid release repository: '$repo'" ;;
    */*) ;;
    *) fail "invalid release repository: '$repo'" ;;
esac

os=$(uname -s)
arch=$(uname -m)
ext=
case $os in
    Darwin) os=macos ;;
    Linux) os=linux ;;
    MINGW*|MSYS*|CYGWIN*) os=windows; ext=.exe ;;
    *) fail "no prebuilt Bop app for $os" ;;
esac
case $arch in
    x86_64|amd64) arch=x64 ;;
    arm64|aarch64) arch=arm64 ;;
    *) fail "no prebuilt Bop app for $os on $arch" ;;
esac
# Windows on ARM runs the x64 build through emulation.
[ "$os" = windows ] && arch=x64
case "$os-$arch" in
    windows-x64|macos-arm64|macos-x64|linux-x64) ;;
    *) fail "no prebuilt Bop app for $os on $arch" ;;
esac
asset="bop-$version-$os-$arch$ext"

bin="$data_dir/bin"
exe="$bin/bop$ext"

# Runs the app and prints its first output line; fails when the app fails or does not finish
# within about 10 seconds (an old app without --version would open the pet instead).
run_bop() {
    out_file="$bin/.bop-out.$$"
    "$exe" "$1" >"$out_file" 2>/dev/null &
    pid=$!
    n=0
    while kill -0 "$pid" 2>/dev/null; do
        n=$((n + 1))
        if [ "$n" -gt 50 ]; then
            kill "$pid" 2>/dev/null
            rm -f "$out_file"
            return 1
        fi
        sleep 0.2 2>/dev/null || sleep 1
    done
    wait "$pid"
    status=$?
    head -n 1 "$out_file" | tr -d '\r'
    rm -f "$out_file"
    return "$status"
}

if [ -f "$exe" ] && [ "$force" = 0 ]; then
    if [ "$(run_bop --version)" = "bop $version" ]; then
        echo "Bop $version is already installed: $exe"
        exit 0
    fi
fi

base=${BOP_RELEASE_BASE_URL:-"https://github.com/$repo/releases/download/v$version"}
base=${base%/}

if command -v curl >/dev/null 2>&1; then
    download() { curl -fsSL --retry 2 --connect-timeout 20 -o "$2" "$1"; }
elif command -v wget >/dev/null 2>&1; then
    download() { wget -q -T 60 -O "$2" "$1"; }
else
    fail "curl or wget is required"
fi

if command -v sha256sum >/dev/null 2>&1; then
    sha256() { sha256sum "$1" | cut -d ' ' -f 1; }
elif command -v shasum >/dev/null 2>&1; then
    sha256() { shasum -a 256 "$1" | cut -d ' ' -f 1; }
else
    fail "sha256sum or shasum is required"
fi

mkdir -p "$bin" || fail "cannot create $bin"
sums_file="$bin/.SHA256SUMS.download.$$"
new_file="$bin/.bop.download.$$"
trap 'rm -f "$sums_file" "$new_file"' EXIT
trap 'exit 1' INT TERM

download "$base/SHA256SUMS" "$sums_file" || fail "download failed: $base/SHA256SUMS"
expected=$(awk -v name="$asset" '{ f = $2; sub(/^\*/, "", f); sub(/\r$/, "", f); if (f == name) { print tolower($1); exit } }' "$sums_file")
case $expected in
    ????????????????????????????????????????????????????????????????) ;;
    *) fail "SHA256SUMS has no entry for $asset" ;;
esac

download "$base/$asset" "$new_file" || fail "download failed: $base/$asset"
actual=$(sha256 "$new_file" | tr 'A-F' 'a-f')
if [ "$actual" != "$expected" ]; then
    rm -f "$new_file"
    fail "checksum mismatch for $asset (expected $expected, got $actual); nothing was installed"
fi
chmod 755 "$new_file" || fail "cannot make $new_file executable"

was_running=0
if [ -f "$exe" ]; then
    [ "$(run_bop stop)" = "pet closed" ] && was_running=1
    if [ "$os" = windows ]; then
        # A running exe cannot be overwritten on Windows, but it can be renamed.
        rm -f "$exe.old" 2>/dev/null
        old="$exe.old"
        [ -e "$old" ] && old="$exe.old-$$"
        mv -f "$exe" "$old" || fail "cannot move the old app out of the way: $exe"
    fi
fi
# On macOS and Linux this replaces the old file in one step, even while it runs.
mv -f "$new_file" "$exe" || fail "cannot install $exe"
rm -f "$exe".old* 2>/dev/null

installed=$(run_bop --version)
if [ "$installed" != "bop $version" ]; then
    if [ "$os" = linux ]; then
        fail "installed $exe, but it does not run. Bop needs WebKitGTK 4.1 (Debian/Ubuntu: sudo apt install libwebkit2gtk-4.1-0)."
    fi
    fail "installed $exe, but it does not run ('$installed')"
fi
echo "Installed Bop $version (SHA-256 verified): $exe"
[ "$was_running" = 1 ] && echo "The pet was closed for the update. Run /bop to open it again."
exit 0
