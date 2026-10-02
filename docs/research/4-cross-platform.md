# 4. Cross-platform: hook command, window, "Open in terminal"

Date: 2026-10-02. Local Claude Code: 2.1.287 (top of the CHANGELOG, the version that added mod support).
Docs were downloaded as raw markdown (`https://code.claude.com/docs/en/<page>.md`), copies are in the
scratchpad `docs/` folder. Experiments: scratchpad `hook-deneme/` (no real Claude session was opened,
Claude Code settings were not touched).

Earlier findings are not repeated: `docs/research/2-technical.md` §7 and DECISIONS 14–17, 23–24.

---

## Summary and recommendation

**Recommended: a single hooks.json, shell form (no `args`), an sh + PowerShell "polyglot" command.**
The same text runs under `sh -c` (macOS, Linux), Git Bash (Windows with Git) and PowerShell (Windows
without Git); if the exe exists it calls `<exe> hook <Event>` (stdin is passed through unchanged),
otherwise it drains stdin and **exits with 0**. No `cmd.exe` wrapper and no `.cmd` file are needed.
Tested on Windows (Git Bash, Git sh, PowerShell 5.1) and WSL Ubuntu (`dash`, `bash`);
`claude plugin validate --strict` passed. Not yet tested in a real Claude session (see
"Not verified" below).

Command (per event only the word `SessionStart` changes; a single string with `\n` inside JSON):

```text
echo `# <#` >/dev/null
d="$CLAUDE_PLUGIN_DATA/bin"; for e in "$d/bop" "$d/bop.exe"; do if [ -n "$CLAUDE_PLUGIN_DATA" ] && [ -f "$e" ] && [ -x "$e" ]; then "$e" hook SessionStart >/dev/null 2>&1; exit 0; fi; done; cat >/dev/null 2>&1; exit 0
#> > $null
$e = "$env:CLAUDE_PLUGIN_DATA/bin/bop.exe"; if ($env:CLAUDE_PLUGIN_DATA -and (Test-Path -LiteralPath $e -PathType Leaf)) { & $e hook SessionStart *> $null } else { $null = [Console]::In.ReadToEnd() }; exit 0
```

hooks.json entry:

```json
{ "type": "command", "async": true, "timeout": 5,
  "command": "echo `# <#` >/dev/null\nd=\"$CLAUDE_PLUGIN_DATA/bin\"; for e in ... exit 0\n#> > $null\n$e = ... exit 0" }
```

How it works:
- **sh/bash:** on line 1, `` `# <#` `` is an empty command substitution (its content is a comment),
  and `echo` writes an empty line to `/dev/null`. Line 2 looks for the exe, runs it and does `exit 0`.
  Because sh parses the script line by line, it never reaches the PowerShell syntax on lines 3–4.
- **PowerShell:** `` `# `` is an escaped `#` (the argument of echo), then `<#` starts a block comment
  that swallows the sh part up to `#>`; `> $null` discards the echo output. Line 4 runs the exe.
- The path is never written into the command as text; it is read from the environment variable
  (`$CLAUDE_PLUGIN_DATA` / `$env:CLAUDE_PLUGIN_DATA`). The `${...}` placeholder is **deliberately
  not used**: Claude Code rewrites it in PowerShell, and in shell form it may substitute it as text
  (the same problem as in DECISIONS 17). Environment variables are passed to the hook process in
  every form (source below, 1.3).

Test results (`hook-deneme/run.py`, `wsltest.sh`):

| Shell | Folder | exe present | exe missing |
| :- | :- | :- | :- |
| Git Bash `bash -c` | `plain`, `R&D (x) %PATH% a!b ç'q$y`z` | exe called with `hook SessionStart`, stdin (Turkish) byte-for-byte identical, rc 0 | rc 0, stderr empty |
| Git `sh -c` | same | same | same |
| `powershell.exe -NoProfile -NonInteractive -Command` (5.1) | same | same (stdin is inherited by the child process, not re-encoded) | rc 0, stderr empty |
| WSL Ubuntu `dash` (`/bin/sh`) and `bash` | `R&D (x) a b $y`z` | same | rc 0; rc 0 also when the variable is unset |

Duration (exe missing, average of 5 runs): Git Bash ~60 ms, PowerShell 5.1 ~223 ms. Since the hook
is async, neither makes Claude wait (1.4).

Why this, in short: the docs have no platform field (2c), `node` is not guaranteed (2d), exec form
needs a single exe that exists on every platform and there is no such exe (2b), and writing settings
at setup leaves a hook behind on uninstall (2e). The polyglot relies on the shell selection rule
stated in the docs (1.1) and works on all current versions. There is also a cleaner but very new
path: a **mod** (2f); not for the first release, an option to watch.

---

## 1. Hook `command`: shell form and exec form

### 1.1 Forms and shell selection

Source: https://code.claude.com/docs/en/hooks#exec-form-and-shell-form

- "A command hook runs as exec form when `args` is set, and shell form when `args` is omitted."
- **Exec form:** "Claude Code resolves `command` as an executable on `PATH` and spawns it directly
  with `args` as the argument vector. There is no shell ... path placeholders like
  `${CLAUDE_PLUGIN_ROOT}` are substituted into `command` and into each `args` element as plain
  strings. ... No shell tokenization happens on any platform."
- **Shell form:** "The `command` string is passed to a shell: `sh -c` on macOS and Linux, Git Bash
  on Windows, or PowerShell when Git Bash isn't installed. Set the `shell` field to choose
  explicitly."
- Windows note: "exec form requires `command` to resolve to a real executable such as a `.exe`.
  The `.cmd` and `.bat` shims ... can't be spawned without a shell."

### 1.2 The `shell` field

Source: https://code.claude.com/docs/en/hooks#command-hook-fields

> `shell` | no | Shell to use for this hook. Accepts `"bash"` or `"powershell"`. Defaults to
> `"bash"`, or to `"powershell"` on Windows when Git Bash isn't installed. Setting `"powershell"`
> runs the command via PowerShell on Windows. ... Ignored when `args` is set

- On Windows, `"shell": "powershell"`: first `pwsh.exe`, otherwise `powershell.exe`
  (https://code.claude.com/docs/en/hooks#windows-powershell-tool).
- **What `"shell": "powershell"` does on macOS/Linux is undocumented.** The docs only say
  "on Windows". The PowerShell tool requires `pwsh` on macOS/Linux
  (https://code.claude.com/docs/en/tools-reference#powershell-tool); for a hook without `pwsh` a
  failed-to-start error is to be expected. So the `shell` field **should not be used**; the polyglot
  works with the default selection.
- If Git Bash cannot be located, its path can be set (`CLAUDE_CODE_GIT_BASH_PATH`, setup docs:
  https://code.claude.com/docs/en/setup). Git for Windows is optional: "Installing Git for Windows
  is optional" (same page).

### 1.3 Variable expansion

- Both forms support placeholders and export them as environment variables: "Both forms support
  the same path placeholders, and both export them as the environment variables
  `CLAUDE_PROJECT_DIR`, `CLAUDE_PLUGIN_ROOT`, and `CLAUDE_PLUGIN_DATA` on the spawned process"
  (https://code.claude.com/docs/en/hooks#exec-form-and-shell-form).
- Plugin hook environment: "every hook process receives `CLAUDE_PLUGIN_ROOT` and `CLAUDE_PLUGIN_DATA`
  in its environment" (https://code.claude.com/docs/en/plugins/components, "Environment, quoting,
  and matching MCP tools").
- In shell form the placeholder must be double-quoted; `claude plugin validate` warns about an
  unquoted `${CLAUDE_PLUGIN_ROOT}`, but not with `shell: "powershell"`
  (https://code.claude.com/docs/en/plugins-reference, "Quoting and path separators").
- On Windows, substituted paths come with `/`: "On Windows, the substituted paths use forward
  slashes so a shell doesn't read backslashes as escapes." (same section).
- PowerShell shell form: "As of v2.1.198, Claude Code rewrites the `${CLAUDE_PROJECT_DIR}`,
  `${CLAUDE_PLUGIN_ROOT}`, and `${CLAUDE_PLUGIN_DATA}` placeholders in a PowerShell shell-form
  command to PowerShell's `${env:NAME}` form" and "Don't write the bare `$CLAUDE_PROJECT_DIR`
  spelling in a PowerShell hook" (https://code.claude.com/docs/en/hooks#windows-powershell-tool).
  That is why the PowerShell part of the polyglot uses `$env:CLAUDE_PLUGIN_DATA`; the bare
  `$CLAUDE_PLUGIN_DATA` in the sh part is inside the block comment, so PowerShell does not run it.
- `${user_config.*}` is substituted only in exec form; in shell form it errors (hooks docs,
  same section). Does not affect us.

### 1.4 async and timeout

Source: https://code.claude.com/docs/en/hooks#run-hooks-in-the-background

- "Once an async hook is running in the background, Claude Code doesn't enforce `timeout` on it."
  So `"timeout": 5` **has no effect** on an async hook (the common fields table says the same:
  https://code.claude.com/docs/en/hooks#common-fields). The exe itself must exit quickly.
- In a `-p` session, an async hook still running at shutdown is killed ("finalizes it with outcome `cancelled`").
- "Async hook completion notifications are suppressed by default." (Visible with `--verbose`/Ctrl+O.)

---

## 2. Running silently on three platforms with a single hooks.json: candidates

### (a) Plain `sh` script (treating Git Bash as required on Windows)

- Pro: natural on macOS/Linux, a single file.
- Con: Git is optional on Windows (setup docs). On Windows without Git, shell form falls back to
  PowerShell; `sh script.sh` or `"$CLAUDE_PLUGIN_ROOT/hooks/x.sh"` does not work in PowerShell, and
  an error notice appears on every event. **Rejected** (same reasoning as in DECISIONS 15).

### (b) Per-platform selection with exec form

- Exec form looks up `command` on `PATH` (1.1). There is no single exe that exists on all three
  platforms: `cmd.exe` does not exist on macOS/Linux; `sh` does not exist on Windows without Git
  (even with Git installed, `sh.exe` is usually not on `PATH`; this part comes not from the docs but
  from Git for Windows' default `PATH` setting, not verified).
- Calling the exe itself (`command: "${CLAUDE_PLUGIN_DATA}/bin/bop"`) fails to start when the exe
  is missing and produces an error notice (section 3).
- Putting a small launcher exe inside the plugin is possible, but a binary is a problem in plugin
  review (DECISIONS 24), and whether an extensionless path resolves to `.exe` on Windows is
  undocumented. **Rejected.**

### (c) Platform/OS condition

- Common hook fields: `type`, `if`, `timeout`, `statusMessage`, `once`
  (https://code.claude.com/docs/en/hooks#common-fields); extra command hook fields: `command`,
  `args`, `async`, `asyncRewake`, `shell` (1.2). **There is no platform/OS field.** `if` works only
  on tool events and with permission rule syntax; "On other events, a hook with `if` set never
  runs." The plugin manifest has no platform condition either (searched in plugins-reference, not
  found). **Not available.**

### (d) An interpreter that exists everywhere (node?)

- A native install has no Node: "The installed `claude` binary does not itself invoke Node." and
  the npm package also installs "the same native binary" (https://code.claude.com/docs/en/setup,
  npm section). The `node` example in the docs (hooks docs Windows note) assumes the user has node.
  **Rejected.**
- `sh` exists on macOS/Linux, PowerShell (5.1) on every Windows. The polyglot relies on exactly
  these two interpreters; on Windows with Git Bash the sh part runs too and finds the exe with the
  `.exe` extension.

### (e) Writing a per-platform hook at setup (`/bop setup`)

- Pro: the fastest command can be written for each platform.
- Con: plugin hooks are loaded automatically from `hooks/hooks.json` and removed together with the
  plugin ("When a plugin is enabled, its hooks merge with your user and project hooks",
  https://code.claude.com/docs/en/hooks#reference-scripts-by-path). A hook written into the user's
  `settings.json`, however, **stays behind** when the plugin is removed, and since the exe is gone
  (`${CLAUDE_PLUGIN_DATA}` is deleted on uninstall, DECISIONS 14) it produces an error on every
  event. Changing user settings would also look bad in directory review (this last part is a guess,
  no docs). **Not recommended.**

### (f) Mod (new, 2.1.287): running the hook as a JS function inside Claude Code

Sources: https://code.claude.com/docs/en/plugins/mods/overview,
https://code.claude.com/docs/en/plugins/mods/reference,
https://code.claude.com/docs/en/plugins/mods/api,
https://github.com/anthropics/claude-code/blob/main/mods/types/claude-code.d.ts,
CHANGELOG 2.1.287 ("Added Claude Mods").

- `"modules": ["./register.js"]` inside `hooks/hooks.json`; the module can subscribe to classic
  events by the name `classic.<Event>`, and "`e` is the hook's stdin JSON" (reference,
  "Settings hook events").
- `$.fs.exists(path)` and `$.process.run(argv, { stdin, timeoutMs })` exist; `run` "uses no shell"
  (api, "Reach files, processes, and the network"; the `stdin` field is in `ProcessRunInit` in the d.ts).
  So if the exe is missing, it can return without starting any process: **no shell, no platform
  differences, no notice.**
- A `vmod/` sample in the scratchpad passed `claude plugin validate --strict` (event names must be
  string literals; a loop with `on('classic.' + ev)` did not pass the validator).
- Why not for now:
  - **Very new:** mod support arrived with 2.1.287 (today's version). Whether older Claude Code
    ignores the `modules` key is undocumented; on an older version the pet silently does not work.
  - Enterprise environments: on Team/Enterprise a built-in guard mod is loaded; an admin can turn
    off user mods with `allowManagedModsOnly` or reject a mod that calls `process.run`
    (https://code.claude.com/docs/en/plugins/mods/admin). `disableAllHooks` also stops mods.
  - The d.ts says "CLI only" for `process`; what happens in the Desktop Code tab is unclear.
  - Whether `CLAUDE_PLUGIN_DATA` exists in the mod process environment (`$.env.get`) is
    undocumented; if not, the data folder path must be found elsewhere (`$.plugin.root` exists, no
    method for the data folder was seen).
  - In directory review a `calls: $.process.run` line is shown ("Starts programs as the user",
    admin docs); it may raise extra questions (guess).
- Conclusion: an option to watch for after Phase 7. If wanted, hooks.json can have both the polyglot
  hook and the mod, but both together call the exe twice per event; one must be chosen.

### (g) Recommended: polyglot shell form

Grounds verified against the docs:
- Shell form shell selection: sh (macOS/Linux), Git Bash or PowerShell (Windows) (1.1).
- Environment variables are in the hook process in every form (1.3).
- Exit 0 and empty stdout = success, nothing appears in the transcript: "Exit 0 means success" and
  "Stderr from a hook that exits 0 goes to the debug log only, never the transcript"
  (https://code.claude.com/docs/en/hooks#exit-code-0).
- On `SessionStart` and `UserPromptSubmit`, plain stdout is added to Claude as context (same section).
  The output of the polyglot and the exe goes to `/dev/null` / `*> $null`; the `echo` that writes
  an empty line is redirected too. That is why it matters that the exe writes nothing to stdout.

Pro: a single file, no extra dependency, the `.cmd`/ASCII rule and the `cmd` delayed expansion
trick (DECISIONS 15, 17) become unnecessary; special characters in the path are not a problem (tested).
Con: hard to read; on Windows without Git, PowerShell startup is ~220 ms (async, does not block);
10 copies in hooks.json (identical apart from the event name). If the exe reads the event name from
`hook_event_name` in stdin, the command becomes event-independent, but the existing
`<exe> hook <Event>` interface can be kept.

---

## 3. Error notice when the exe is missing

Source: https://code.claude.com/docs/en/hooks#exit-code-output

- Exit 0: success. stderr goes only to the debug log.
- Exit 2: blocking (depending on the event); on `SessionStart` stderr is shown in the transcript as
  a "hook error".
- Other codes (1, 127...): if stdout is empty or plain text, "non-blocking error ... the transcript
  shows a `<hook name> hook error` notice followed by the first line of stderr, prefixed with
  `Failed with non-blocking status code:`".
- A hook that fails to start: "A hook that can't start lands in the same non-blocking bucket. When the
  script path doesn't exist or isn't executable, the shell exits with a code like 127 and you see
  the same notice" (same section). So on macOS/Linux, where there is no `cmd.exe`, the current hook
  produces this notice on every event (consistent with DECISIONS 15).
- Stdout that looks like JSON (starts with `{`) but is malformed produces an error notice with any
  exit code ("when Claude Code tries to parse your stdout as JSON and can't, it reports a non-blocking
  error"). The exe must not write to stdout.

**The docs are not explicit about async hooks.** They say an async hook's results (`systemMessage`,
`additionalContext`) are delivered to Claude on the next turn and that "completion notifications are
suppressed by default"; they do not say whether a failed async hook shows a "hook error" notice
to the user. Indirect signs:
- Issue #79847: on Windows the same script produces `hook_non_blocking_error` every time in
  synchronous hooks, while "0 occurrences of this error for hooks marked `"async": true`"
  (https://github.com/anthropics/claude-code/issues/79847). This is specific to one error type, not
  a general rule.
- CHANGELOG 2.1.287: when an `asyncRewake` hook's script file was missing, the "found issues" notice
  used to repeat, now it is "reported once". `asyncRewake` is a different mechanism (wakes Claude on
  exit 2) and says nothing about plain `async`.
Conclusion: instead of relying on async, the command **must exit with 0 in every case**; the
polyglot does this. To be verified in a real session (below).

---

## 4. Tauri transparent, borderless, always-on-top window on macOS/Linux

Versions locked in the project: `tauri 2.12.0`, `tao 0.37.1`, `wry 0.57.0` (`app/src-tauri/Cargo.lock`).

- **macOS transparency:** Tauri 2.12.1 CHANGELOG: "The `macos-private-api` feature flag /
  `macOSPrivateAPI` tauri.conf.json value is no longer required to use transparency or fullscreen
  on macOS." (https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/CHANGELOG.md, 2.12.1).
  The schema also says "No-op in Tauri 2.12.1+" (https://schema.tauri.app/config/2,
  `AppConfig.macOSPrivateApi`). **The project is on 2.12.0**: either upgrade to 2.12.1+ or enable
  `"macOSPrivateApi": true` + the `macos-private-api` feature (the pre-2.12.1 rule; on older
  versions a private API that cannot get into the App Store). Upgrading is cleaner.
- **Shadow:** `shadow` is "Linux: Unsupported"; on Windows `true` gives a 1 px border on a
  borderless window (schema, `WindowConfig.shadow`).
- **Linux, Wayland** (tao `Window` docs, https://docs.rs/tao/0.37.1/tao/window/struct.Window.html):
  - `set_always_on_top`: "Linux(x11): Result depends on the system's window manager. Consider this
    setting a suggestion. Linux(Wayland) / iOS / Android: Unsupported."
  - `set_outer_position`: "Android / Linux(Wayland): Unsupported."; `outer_position`:
    "Linux(Wayland): Has no effect, since Wayland doesn't support a global coordinate system".
    Saving/restoring the window position (`window.json`) and the bubble that docks to the pet do
    not work on Wayland.
  - `cursor_position`: "iOS / Android / Linux(Wayland): Unsupported, returns 0,0". Looking at the
    cursor does not work on Wayland.
  - `set_ignore_cursor_events`: lists no restriction for Linux (only iOS/Android).
  - `drag_window`: "macOS: May prevent the button release event to be triggered."
  - `skipTaskbar`: "hides the window icon from the taskbar on Windows and Linux" (schema); on macOS
    the Dock icon needs a separate setting (activation policy; docs not researched).
  - Common workaround: launch the app through XWayland (`GDK_BACKEND=x11`). This is not in the Tauri
    docs, it is general GTK knowledge; not verified.
- **Linux, X11 transparency:** real per-pixel transparency needs a compositor (on a window manager
  without compositing the background looks black). This is general X11 knowledge; no explicit
  sentence was found in the Tauri docs. GNOME/KDE run a compositor by default.
- **Linux dependency:** WebKitGTK 4.1 (`libwebkit2gtk-4.1`) is required at runtime
  (https://v2.tauri.app/start/prerequisites/). Choosing between AppImage and .deb distribution is
  Phase 7 work.
- **macOS signing:** a downloaded unsigned binary may be blocked by Gatekeeper; the quarantine
  attribute is set by the downloading application (curl does not set it). This part is not verified
  against docs and should be handled in the download design.

---

## 5. "Open in terminal" (`claude --resume <id>` in a new terminal)

Common: passing the command not directly but through a temporary script file containing only
`cd <folder>` and `claude --resume <uuid>` avoids quoting problems (the id must contain only UUID
characters, to be validated).

- **macOS:** Terminal.app exists on every macOS.
  - `open -a Terminal /tmp/bop-resume.command` (file with `#!/bin/sh`, `chmod +x`). `open` is a
    system tool; it asks for no extra permission.
  - Alternative: `osascript -e 'tell application "Terminal" to do script "..."'`. May trigger an
    automation permission prompt (TCC "X wants to control Terminal"). These two items are general
    macOS knowledge, not verified against official docs.
- **Linux:** no standard, try in order:
  1. `$TERMINAL` (convention, not a standard).
  2. `xdg-terminal-exec <command> [arg...]`: proposed to freedesktop, still a draft specification
     ("while this spec is in proposed state, backwards compatibility ... is not guaranteed",
     https://github.com/Vladimir-csp/xdg-terminal-exec).
  3. `x-terminal-emulator -e <command> [arg...]` (Debian/Ubuntu family): "`-e command` ... creates a
     new terminal window and runs the specified command ... as though the arguments were passed
     directly to `execvp`, bypassing the shell" (Debian Policy 11.8.3,
     https://www.debian.org/doc/debian-policy/ch-customized-programs.html).
  4. Known ones: `gnome-terminal -- <command>`, `konsole -e`, `xfce4-terminal -x`, `kitty`,
     `alacritty -e`, `wezterm start --`, `xterm -e` (each has a different argument format; not verified).
  If none exists, showing "copy command" (`claude --resume <id>`) in the bubble is the safest fallback.

---

## Not verified (to be tested in a real session)

1. **With which arguments** Claude Code launches Git Bash and PowerShell on Windows
   (`bash -c`/`-lc`? `powershell -Command`/`-File` with a temp file?). The tests used `bash -c` and
   `powershell -NoProfile -NonInteractive -Command`. If a temporary `.ps1` is used with `-File`, the
   execution policy may kick in (`-ExecutionPolicy Bypass` is documented for the PowerShell **tool**,
   not for hooks).
2. Whether a notice is shown to the user when an async hook fails (section 3).
3. That Claude Code passes the multi-line `command` string to the shell as-is on all three platforms
   (validate passed; execution not observed).
4. The polyglot with `/bin/sh` on macOS (bash 3.2 in sh mode) (tested with Git sh and dash, not on
   macOS; could be tested on a GitHub Actions macOS runner).
5. `"shell": "powershell"` behavior on macOS/Linux (we do not use it).
6. For the mod path: the `modules` key on older versions, `CLAUDE_PLUGIN_DATA` in the mod environment,
   `$.process.run` in Desktop.

Suggested test order: (1) Windows with Git installed: load the plugin with `--plugin-dir`, read the
hook lines with `--debug`, with and without the exe; (2) Windows without Git (by removing Git from
PATH or pointing `CLAUDE_CODE_GIT_BASH_PATH` at an invalid location); (3) Linux Claude Code in WSL;
(4) macOS: `claude plugin validate` in Actions + a unit test of the polyglot with `/bin/sh -c`.

## Test files

temporary test folder `hook-deneme/`:
`poly.txt` (command), `run.py` (Windows shells), `wsltest.sh` (dash/bash), `fake.cs`/`fake.exe`
(fake exe that dumps its arguments and stdin), `vplug/` (polyglot hooks.json, validate passed), `vmod/` (mod
sample, validate passed).
