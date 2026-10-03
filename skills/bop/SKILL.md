---
name: bop
description: Opens or closes the Bop desktop pet, lists installed pets, switches the active pet, installs a pet from a local folder, or explains how to install the Bop app (setup).
argument-hint: "[setup | list | use <id> | install <folder> | stop]"
disable-model-invocation: true
allowed-tools: Bash(sh "${CLAUDE_PLUGIN_ROOT}/scripts/bop.sh" toggle), Bash(sh "${CLAUDE_PLUGIN_ROOT}/scripts/bop.sh" stop), Bash(sh "${CLAUDE_PLUGIN_ROOT}/scripts/bop.sh" list), Bash(sh "${CLAUDE_PLUGIN_ROOT}/scripts/bop.sh" use *), Bash(sh "${CLAUDE_PLUGIN_ROOT}/scripts/bop.sh" install *), PowerShell(powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/bop.ps1" toggle), PowerShell(powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/bop.ps1" stop), PowerShell(powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/bop.ps1" list), PowerShell(powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/bop.ps1" use *), PowerShell(powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/bop.ps1" install *)
---

# Bop

Bop is a desktop pet that shows what Claude Code is doing. This skill only forwards the user's
request to the Bop app. Do not do anything else, and do not read or change other files.

## The app

`<app>` below stands for the plugin's script that runs the Bop app:

- Bash tool (macOS, Linux, Windows with Git Bash): `sh "${CLAUDE_PLUGIN_ROOT}/scripts/bop.sh"`
- PowerShell tool (Windows without Git Bash): `powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/bop.ps1"`

Use the Bash tool when it is available. Write the prefix exactly as above, then the arguments.

Run the command on its own: no `cd`, no existence check, no pipes or `;`, so it matches the
permissions this skill grants. If the script reports that the Bop app is not installed, show its
message to the user as it is, then stop. Do not try to download, build or find the app any
other way.

## Request

The user typed: `$ARGUMENTS`

Run exactly one command:

| Request | Command | Meaning |
| --- | --- | --- |
| empty | `<app> toggle` | open the pet, or close it if it is open |
| `setup` | none, see [Setup](#setup) | how to install the Bop app |
| `stop` | `<app> stop` | close the pet |
| `list` | `<app> list` | installed pets; `*` marks the active one |
| `use <id>` | `<app> use <id>` | switch to that pet; an open pet changes at once |
| `install <folder>` | `<app> install "<folder>"` | copy a pet folder (`pet.json` + spritesheet) into Bop |

Anything else: show the table above as a short usage note and run nothing.

After `install`, offer to switch to the new pet with `use <id>`; run it only if the user agrees.

## Setup

Run nothing. The plugin does not download or install the app. Tell the user, in their language:

1. Download the file for their platform from https://github.com/AlperenGonuk/bop/releases/latest
   (Windows: `bop-<version>-windows-x64.exe`; macOS: `macos-arm64` for Apple silicon,
   `macos-x64` for Intel; Linux: `linux-x64`).
2. Open it once (double-click, or run it from a terminal). It copies itself to `~/.bop/bin/`,
   where the plugin finds it, and the pet appears. The downloaded file can then be deleted.
3. To update, download the new version and open it the same way.

macOS may block the unsigned app; the README explains how to allow it. On Linux the file must be
made executable first (`chmod +x`).

## Reply

Answer in one or two short sentences in the user's language, based on the command output. If the
command fails, show its error line as it is. For `list`, show the pets as a short list with the
active one marked.
