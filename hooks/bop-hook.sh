#!/bin/sh
# Bop hook for macOS, Linux and Windows with Git Bash.
#
# Usage: bop-hook.sh <event> [app-folder]
#
# Claude Code runs this on every hook event listed in hooks.json and passes the event as JSON on
# stdin. If the user has installed the Bop app (~/.bop/bin/bop, or bop.exe on Windows), it runs
# `bop hook <event>`, which reads that JSON and writes the pet's state to ~/.bop/state.json.
# Nothing else happens: no network, no download, no other files.
#
# A hook must never break or slow down Claude Code, so this script always exits 0 without output.
# If the app is not installed (or in Cowork), it only reads stdin and exits.
# BOP_HOME replaces ~/.bop (for testing). The optional app folder is for developing Bop: the
# repository's .claude/settings.json passes the debug build folder instead of ~/.bop/bin.

event=$1
# The app uses %USERPROFILE% on Windows, which can differ from Git Bash's $HOME.
user_home=$HOME
if [ -n "${USERPROFILE:-}" ] && command -v cygpath >/dev/null 2>&1; then
  user_home=$(cygpath -u "$USERPROFILE")
fi
home=${BOP_HOME:-$user_home/.bop}
app_dir=${2:-$home/bin}

for app in "$app_dir/bop" "$app_dir/bop.exe"; do
  if [ -f "$app" ] && [ -x "$app" ]; then
    "$app" hook "$event" >/dev/null 2>&1
    exit 0
  fi
done

cat >/dev/null 2>&1
exit 0
