#!/bin/sh
# Runs the Bop app with the given arguments (macOS, Linux and Windows with Git Bash).
#
# Usage: sh bop.sh <arguments>     for example: sh bop.sh toggle
#
# The plugin does not download or install the app. The user downloads it from the GitHub
# releases page and opens it once; it then copies itself to ~/.bop/bin/bop (bop.exe on
# Windows). This script only runs that copy and passes its output and exit code through.
# BOP_HOME replaces ~/.bop (for testing).

# The app uses %USERPROFILE% on Windows, which can differ from Git Bash's $HOME.
user_home=$HOME
if [ -n "${USERPROFILE:-}" ] && command -v cygpath >/dev/null 2>&1; then
  user_home=$(cygpath -u "$USERPROFILE")
fi
home=${BOP_HOME:-$user_home/.bop}
for app in "$home/bin/bop" "$home/bin/bop.exe"; do
  if [ -f "$app" ] && [ -x "$app" ]; then
    exec "$app" "$@"
  fi
done

echo "bop: the Bop app is not installed. Download it from https://github.com/AlperenGonuk/bop/releases/latest and open it once." >&2
exit 127
