---
name: bop
description: Opens or closes the Bop desktop pet, lists installed pets, switches the active pet, installs a pet from a local folder, or downloads the Bop app itself (setup).
argument-hint: "[setup | list | use <id> | install <folder> | stop]"
disable-model-invocation: true
allowed-tools: Bash(${CLAUDE_PLUGIN_DATA}/bin/bop *), Bash(${CLAUDE_PLUGIN_DATA}/bin/bop.exe *), Bash("${CLAUDE_PLUGIN_DATA}/bin/bop" *), Bash("${CLAUDE_PLUGIN_DATA}/bin/bop.exe" *), PowerShell(${CLAUDE_PLUGIN_DATA}/bin/bop.exe *), PowerShell("${CLAUDE_PLUGIN_DATA}/bin/bop.exe" *), PowerShell(& "${CLAUDE_PLUGIN_DATA}/bin/bop.exe" *), Bash(powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/install.ps1" *), PowerShell(powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/install.ps1" *), Bash(sh "${CLAUDE_PLUGIN_ROOT}/scripts/install.sh" *)
---

# Bop

Bop is a desktop pet that shows what Claude Code is doing. This skill only forwards the user's
request to the Bop app. Do not do anything else, and do not read or change other files.

## The app

- Windows: `${CLAUDE_PLUGIN_DATA}/bin/bop.exe`
- macOS and Linux: `${CLAUDE_PLUGIN_DATA}/bin/bop`

Use the one for the current platform. Use the Bash tool when it is available. Always put the
path in double quotes. In PowerShell,
start the command with `& ` (for example `& "<path>" list`).

Run the command on its own: no `cd`, no existence check, no pipes or `;`, so it matches the
permissions this skill grants. If it fails because the file is not found, tell the user in one
sentence that the Bop app is not installed yet and that `/bop setup` downloads it, then stop.
Do not try to download, build or find it any other way.

## Request

The user typed: `$ARGUMENTS`

Run exactly one command:

| Request | Command | Meaning |
| --- | --- | --- |
| empty | `"<app>" toggle` | open the pet, or close it if it is open |
| `setup` | see [Setup](#setup) | download and install the Bop app |
| `stop` | `"<app>" stop` | close the pet |
| `list` | `"<app>" list` | installed pets; `*` marks the active one |
| `use <id>` | `"<app>" use <id>` | switch to that pet; an open pet changes at once |
| `install <folder>` | `"<app>" install "<folder>"` | copy a pet folder (`pet.json` + spritesheet) into Bop |

Anything else: show the table above as a short usage note and run nothing.

After `install`, offer to switch to the new pet with `use <id>`; run it only if the user agrees.

## Setup

`setup` runs the plugin's install script instead of the app. It downloads the Bop app that
matches this plugin version from the plugin's GitHub release, checks it against the release's
`SHA256SUMS` file and puts it in the plugin data folder. It skips the download when that version
is already installed. Run exactly one command, on its own, as written:

- Windows: `powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/install.ps1" -DataDir "${CLAUDE_PLUGIN_DATA}"`
- macOS and Linux: `sh "${CLAUDE_PLUGIN_ROOT}/scripts/install.sh" "${CLAUDE_PLUGIN_DATA}"`

Only if the user typed `setup --force` (reinstall the same version), add ` -Force` (Windows) or
` --force` (macOS and Linux) at the end. After a successful setup, tell the user that `/bop`
opens the pet. If it fails, show the error line as it is; do not try another way to get the app.

## Reply

Answer in one or two short sentences in the user's language, based on the command output. If the
command fails, show its error line as it is. For `list`, show the pets as a short list with the
active one marked.
