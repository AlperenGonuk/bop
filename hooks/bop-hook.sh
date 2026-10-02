#!/bin/sh
# Bop hook for macOS, Linux and Windows with Git Bash.
#
# Usage: bop-hook.sh <app-folder> <event>
#
# Claude Code runs this on every hook event listed in hooks.json and passes the event as JSON on
# stdin. If the Bop app is installed in <app-folder>, it runs `bop hook <event>`, which reads that
# JSON and writes the pet's state to ~/.bop/state.json. Nothing else happens: no network, no
# download, no other files.
#
# A hook must never break or slow down Claude Code, so this script always exits 0 without output.
# If the app is not installed (before `/bop setup`, or in Cowork), it only reads stdin and exits.

app_dir=$1
event=$2

for app in "$app_dir/bop" "$app_dir/bop.exe"; do
  if [ -n "$app_dir" ] && [ -f "$app" ] && [ -x "$app" ]; then
    "$app" hook "$event" >/dev/null 2>&1
    exit 0
  fi
done

cat >/dev/null 2>&1
exit 0
