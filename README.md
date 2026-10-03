# Bop

**Bop — a desktop pet for Claude Code.**

Bop is a small pixel pet that lives on your desktop and shows what Claude Code is doing: it
thinks while Claude thinks, types while Claude edits code, waves when Claude needs your
permission, cheers when a task is done and looks sad when something fails. Click it to chat with
Claude in a little terminal-style speech bubble. You can make new pets with Claude, and pets use
the same format as Codex pets.

![Bop, a desktop pet for Claude Code: Pitir the mandarin mascot at a computer](docs/images/bop-brand.png)

The default pet is Pitir, a small mandarin drawn entirely by Bop's own drawing engine.

> **Status:** early release (0.2.0). Windows is the most tested platform; macOS and Linux builds
> are produced by CI but have had little real-world use. Please report problems in
> [Issues](https://github.com/AlperenGonuk/bop/issues).

## Install

Bop works in Claude Code on Windows, macOS and Linux (the terminal, IDE extensions and the
desktop app's Code tab), because the pet is a desktop app on your own computer. In claude.ai
chat and in Cowork the plugin does nothing: chat does not load hooks, and the Bop app is not
installed there, so its hooks exit without doing anything.

You need Claude Code. In a terminal:

```sh
claude plugin marketplace add AlperenGonuk/bop
claude plugin install bop@bop
```

The plugin does not download anything. Install the Bop app yourself:

1. Download the file for your platform from the
   [latest release](https://github.com/AlperenGonuk/bop/releases/latest):
   `bop-<version>-windows-x64.exe`, `bop-<version>-macos-arm64` (Apple silicon),
   `bop-<version>-macos-x64` (Intel) or `bop-<version>-linux-x64`.
2. Open it once: double-click it, or run it from a terminal (on macOS and Linux, `chmod +x` it
   first; see [Platform notes](#platform-notes)). It copies itself to `~/.bop/bin/`, where the
   plugin finds it, and the pet appears. You can delete the downloaded file afterwards.

To update the app, download the new version and open it the same way.

Then, in a new Claude Code session (or after `/reload-plugins`), `/bop` opens the pet; run it
again to close it. `/bop setup` repeats the install steps above.

## Commands

| Command | What it does |
| --- | --- |
| `/bop` | Open the pet, or close it if it is open |
| `/bop setup` | Show how to install the Bop app (runs nothing) |
| `/bop list` | List installed pets (`*` marks the active one) |
| `/bop use <id>` | Switch to another pet; an open pet changes at once |
| `/bop install <folder>` | Install a pet from a local folder (`pet.json` + spritesheet) |
| `/bop stop` | Close the pet |

Right-click the pet for **New chat**, **Switch pet**, **Make a new pet** and **Quit**. In the chat
bubble, **Open in terminal** hands the conversation over to a full Claude Code session.

## Make your own pet

Ask Claude in any session, for example "make me a blue cat pet for Bop", or right-click the pet
and choose **Make a new pet**. The `hatch` skill describes the pet as a small JSON spec, and the
Bop app draws every animation frame from it with code (no image generation), checks the result,
and shows Claude a contact sheet to review. When you are happy, Claude installs the pet and asks
whether to switch to it.

## What the plugin runs

The plugin is this repository: the skills, the hooks, two small scripts and the source of the
Bop app. It contains no executables and never downloads anything. The Bop app is built from the
Rust/Tauri source in [`app/`](app/) by [GitHub Actions](.github/workflows/release.yml) and
attached to each [GitHub release](https://github.com/AlperenGonuk/bop/releases), together with a
`SHA256SUMS` file. You download and open it yourself (see [Install](#install)).

The skills run the app only through [`scripts/bop.sh`](scripts/bop.sh) (or
[`scripts/bop.ps1`](scripts/bop.ps1) on Windows without Git Bash). The script runs
`~/.bop/bin/bop` (`bop.exe` on Windows) with the skill's arguments, or prints how to install the
app if it is not there.

To check a downloaded file, compare it with `SHA256SUMS` from the release page:

```sh
sha256sum -c --ignore-missing SHA256SUMS        # Linux
shasum -a 256 bop-0.2.0-macos-arm64           # macOS: compare with the line in SHA256SUMS
```

```powershell
Get-FileHash .\bop-0.2.0-windows-x64.exe -Algorithm SHA256   # Windows
```

## How the hooks work

[`hooks/hooks.json`](hooks/hooks.json) registers one asynchronous hook for each Claude Code event
the pet shows (session start and end, prompt, tool use, permission request, notification, stop).
Each hook runs a short script with one argument, the event name:

- [`hooks/bop-hook.sh`](hooks/bop-hook.sh) on macOS, Linux and Windows with Git Bash;
- [`hooks/bop-hook.ps1`](hooks/bop-hook.ps1) on Windows without Git Bash, where Claude Code runs
  hooks with PowerShell.

If the Bop app is installed in `~/.bop/bin/`, the script runs `bop hook <event>`, which reads the event JSON from
stdin and writes the pet's state to `~/.bop/state.json`. If it is not, the script only reads
stdin. Either way it prints nothing and exits 0, so it can never block or break Claude Code.

Claude Code runs a hook command with whichever shell is available, so each command in
`hooks.json` is written to be valid in both: `sh` reads the first two lines and exits, while
PowerShell skips them as a `<# ... #>` block comment and runs the last line. Each half only
starts the matching script above.

## Privacy

- **Pet chat uses your own Claude Code.** Messages you type into the pet run through
  `claude -p` on your machine, under your own Claude account. They count toward your plan's
  usage limits, like any other Claude Code session.
- **The pet chat cannot change anything.** In a folder you trust, it can only read files (`Read`,
  `Glob`, `Grep`). Otherwise it runs in an empty folder and can only use Claude Code's built-in
  web search when you ask for it. It never gets both, and never gets tools that write files, run
  commands or open web pages; use **Open in terminal** for that.
- **Hooks stay local.** The hooks only run the local Bop app, which writes the current state to
  `~/.bop/state.json` for the pet. They make no network requests.
- **Network access** happens only when Claude uses web search in the pet chat because you
  asked. The plugin itself never downloads anything.
- **No credentials are read.** Before the pet starts `claude -p`, the app removes the
  `CLAUDE_CODE_*` session variables (such as `CLAUDE_CODE_MESSAGING_TOKEN`) it inherited from the
  Claude Code session that opened it, so the chat runs as a separate session. It never reads,
  stores or sends their values.
- No telemetry, no analytics, no server. Full details: [PRIVACY.md](PRIVACY.md).

Bop's own files, the app included (`~/.bop/bin/`), live in `~/.bop/`; close the pet and delete
that folder to remove them. Uninstalling the plugin does not remove it.

## Platform notes

- **Windows 10 (1809+) and 11:** x64; also runs on Windows on ARM through emulation. The app is
  not code-signed yet, so the first time you open the downloaded file, Microsoft Defender
  SmartScreen may say "Windows protected your PC". Choose **More info**, then **Run anyway**.
  This happens only once: the plugin then runs the copy in `~/.bop/bin/`.
- **macOS (Apple silicon and Intel):** the app is not signed or notarized. A file downloaded
  with a browser is quarantined and macOS blocks it. Before opening it, run
  `xattr -d com.apple.quarantine bop-<version>-macos-arm64` and `chmod +x` on the file.
- **Linux (x64):** needs WebKitGTK 4.1 (Debian/Ubuntu: `sudo apt install libwebkit2gtk-4.1-0`).
  On X11, a transparent pet needs a compositing window manager (GNOME and KDE have one);
  without it the pet has a black background. On Wayland, the window cannot stay on top, its
  position is not remembered and the pet cannot follow the mouse pointer. Running Claude Code
  with `GDK_BACKEND=x11` (XWayland) may help.
- **macOS and Linux, for now:** **Open in terminal**, **Make a new pet** from the right-click menu
  and source links in the chat work only on Windows. Use `claude --resume` or ask Claude in a
  terminal session instead.

## Build from source

```sh
cd app/src-tauri
cargo build --release
```

The executable is `app/src-tauri/target/release/bop` (`bop.exe` on Windows). Linux builds need
the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/).

## Support

Questions, bugs and security reports: [GitHub Issues](https://github.com/AlperenGonuk/bop/issues).

## License

[MIT](LICENSE). Pitir and the other bundled example pets are part of this project and use the
same license.

Bop is an independent project. It is not affiliated with, endorsed by or sponsored by
Anthropic. "Claude" and "Claude Code" are trademarks of Anthropic, PBC.
