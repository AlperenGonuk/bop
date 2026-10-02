# Decision log

Newest decision on top. Format: date, decision, reason, source.

## 2026-10-02

36. **The plugin is the repository root; hooks call two readable scripts** (changes 29). Source:
    Alperen ("A, make it readable"), after checking the repository against the directory's
    pre-submission checklist (claude.com/docs/plugins/pre-submission-checklist).
    - Why: for a plugin in a subfolder, a hook command may only use `${CLAUDE_PLUGIN_ROOT}` paths,
      with no other variable or command substitution (**Blocks**), and any non-shell program a
      hook runs is held. The checklist recommends keeping the plugin at the root of its own
      repository. At the root the app's Rust source is also inside the scanned folder.
    - Layout: `.claude-plugin/plugin.json` + `marketplace.json` (`source: "./"`), `hooks/`,
      `skills/`, `scripts/`, `assets/icon.png`. `plugin/README.md` merged into `README.md` (the
      listing text). `CLAUDE.md` moved to `.claude/CLAUDE.md` (a root CLAUDE.md is a
      `validate --strict` warning). The whole repository (~2.5 MB) is now copied into the plugin
      cache; accepted.
    - Hooks: logic in `hooks/bop-hook.sh` and `hooks/bop-hook.ps1` (args: app folder, event).
      `hooks.json` keeps a 4-line sh/PowerShell command that only starts the matching script;
      paths come from `${CLAUDE_PLUGIN_ROOT}` / `${CLAUDE_PLUGIN_DATA}` placeholders. PowerShell
      starts the script with `-NoProfile -ExecutionPolicy Bypass -File` (the documented pattern).
      The repo's dev hooks in `.claude/settings.json` use the same scripts with the debug exe.
    - README: Markdown image syntax (HTML `<img>` to a bundled image is held), a "How the hooks
      work" section, and a note that the plugin does nothing in claude.ai chat and Cowork.
    - Tested: bash and PowerShell, with and without the app (writes the state / exits 0 silently).
    - Expected reviewer holds that stay: `app/src-tauri/icons/icon.ico` (binary other than an
      image; Tauri needs it), the downloaded executable, possibly the short name `bop`.
35. **Everything in the repository is in English** (changes 21: internal docs were Turkish).
    Conversation with Alperen stays Turkish. Source: Alperen (2026-10-02).
    - Docs renamed: `PROJE.md` → `PROJECT.md`, `docs/DURUM.md` → `docs/STATUS.md`,
      `docs/KARARLAR.md` → `docs/DECISIONS.md`, `docs/arastirma/` → `docs/research/`. Code
      comments, commit messages and logs are English too.
34. **Pre-release cleanup and new history.** Source: Alperen (choices after the audit).
    - The release branch starts with a single "Initial commit"; the old history stays only locally
      in a backup branch. Commit e-mail is the GitHub noreply address.
    - Internal docs (STATUS, PLAN, DECISIONS, research) stay in the repository; personal paths and
      internal notes were cleaned up, the old C# prototype (`pitir-deneme/`) was removed, the link
      to the Johnny repository was removed.
    - The app icon was generated from Pitir's neutral frame (the old orange icon resembled Clawd).
    - Repo `AlperenGonuk/bop`; marketplace name `bop`, plugin `bop@bop`.
33. **The exe is installed with `/bop setup`, not by a hook** (makes decision 24 concrete). Source:
    Claude (Alperen "don't ask until it's done"); reason: hooks that download are held up in the
    directory review.
    - `plugin/scripts/install.ps1` / `install.sh`: `plugin.json` `version` → release `v<version>`,
      the asset and `SHA256SUMS` are downloaded, sha256 is verified, placed into
      `<CLAUDE_PLUGIN_DATA>/bin/`; skipped if the version matches (`bop --version`), `--force`
      reinstalls. The repo is defined in one place: `plugin.json` `metadata.releaseRepo`. For
      testing: `BOP_RELEASE_BASE_URL`.
    - Actions (`.github/workflows/release.yml`): on a `v*` tag, Windows x64, macOS arm64/x64,
      Linux x64; `SHA256SUMS`; the release is created as a **draft**, downloads do not work until
      Alperen publishes it.
    - Tauri 2.12.0 → 2.12.1 (macOS transparency). Pet images have the same license as the code (MIT).

31. **Sprite rule clarified:** `dev-pets/` and sprites from elsewhere are not modified;
    `app/src-tauri/default-pet/` is engine output, regenerated from `pitir.json` (CLAUDE.md
    updated). Source: Alperen (code review finding 8). Same day: UI strings were translated to
    English (decision 21, Alperen "make the settings part and so on English too").
    Terms: Yeni sohbet → New chat, Terminalde aç → Open in terminal, "Bu klasöre güveniyor
    musun?" → "Trust this folder?", Güven/Güvenme → Trust/Don't trust, Kaynaklar → Sources, Pet
    değiştir → Switch pet, Yeni pet yap → Make a new pet, Kapat → Quit. The Turkish names in
    decisions 18–20 and 28 should now be read with these English equivalents. Log lines and
    comments stay Turkish (changed by decision 35).
32. **The exe is built with the console subsystem** (`windows_subsystem = "windows"` removed).
    Source: Claude (code review finding 5; Alperen "don't ask until it's done"). Reason: PowerShell
    5.1 does not wait for a GUI-subsystem exe, so the output read by the skill and
    `$LASTEXITCODE` were lost; `AttachConsole` was not enough. The pet window is opened by
    `toggle` with CREATE_NO_WINDOW; if launched from Explorer it releases its own console with
    `FreeConsole` (a brief flash is possible). Tested with the release exe: output is captured,
    exit codes 0/1 are correct, `toggle`/`stop` work.


30. **`hatch` engine and spec format** (Phase 6; Alperen said "don't ask until it's done", the
    defaults are the agent's choice, to be reviewed). `bop hatch <spec.json> <folder>` → `pet.json`
    (`spriteVersionNumber: 2` always; the "only in the community" wording in decision 3 is
    outdated, the current Codex skill treats it as required), `spritesheet.png`,
    `contact-sheet.png`, `report.json`.
    - Spec has three levels: archetype (`blob`, `critter`, `floaty`, `custom`) + palette + options;
      parts merged by name; `pixels` parts and per-frame `overrides`. Unknown fields are errors.
    - Default grid 48×52 @4 (as in the Pitir prototype); 96×104 @2 optional. Pixel style only.
    - No detached effects (Codex rule; Pitir's "...", thought bubble and sparks were removed). A
      sweat drop touches the pet. A detached part is only a warning.
    - `running-left` is not a mirror but redrawn facing left; 180° = looking down.
    - (0,6) neutral = idle 0; the app (sprites.js) removes this cell from the idle loop in v2.
    - An overflowing pet is automatically shrunk down to 80% (warning). Exit codes 0/2
      (validation)/1 (spec).
    - Example specs in `plugin/skills/hatch/examples/` (embedded in the exe, `--example`); the
      embedded Pitir is generated from `pitir.json`, a test checks it matches the engine output.
    - The skill can be invoked by the model, `allowed-tools` is the exe only; `Write` (spec) asks
      the user.
    - (Review, Claude) References are embedded in the exe: `bop hatch --docs <name>`; Claude does
      not ask for permission to read the plugin folder. In both skills the PowerShell rules use the
      command name (without `&`; docs: PowerShell rules use the same format as Bash, matched from
      the AST) and an instruction "if a Bash tool is available use it, run the command on its own".
      In a real `claude -p` test ("A small sleepy blue ghost") the flow finished without a
      permission denial: spec, hatch, QA, install.

29. **(Changed by 36: plugin at the root, hooks call scripts.)** **Hooks are shell form, a single "polyglot" command; plugin in the `plugin/` folder** (changes
    decisions 15 and 17). Source: Alperen ("go on"), research `docs/research/4-cross-platform.md`.
    - The command is given without `args`: on macOS/Linux `sh -c`, on Windows Git Bash, or
      PowerShell if Git is missing, runs it. The first half of the same text is sh, the second half
      PowerShell (a `<# ... #>` block comment hides the sh part). The path is read from an
      environment variable (`$CLAUDE_PLUGIN_DATA`, `$env:` in PowerShell), no `${...}` placeholder
      is used. If the exe is missing stdin is drained; it always exits with 0. The `shell` field is
      not used (its behavior on macOS/Linux is undocumented).
    - Plugin: `plugin/hooks/hooks.json` (`$CLAUDE_PLUGIN_DATA/bin/bop[.exe]`). Repo development
      hooks in `.claude/settings.json` use the same command with
      `$CLAUDE_PROJECT_DIR/app/src-tauri/target/debug`. `hooks/claude-pet-hook.cmd` was removed.
    - The plugin root is `plugin/`, not the repo root: the CLAUDE.md at the root fails
      `validate --strict` with a warning, and the app source should not be copied into the plugin
      cache. The marketplace entry will be `source: "./plugin"`.
    - Tested: a real Claude session in this repo (Windows, Git Bash) writes to
      `~/.bop/state.json`; `claude plugin validate --strict plugin` passed. The PowerShell path and
      macOS were verified only by the manual tests in the research.

28. **Active pet selection and new pets** (closes the pending decision). Source: Alperen.
    - Installed pets in `~/.bop/pets/<id>/`; the selected pet in `~/.bop/config.json`
      (`{"activePet": "<id>"}`).
    - Right-click menu: **Switch pet ▸** (installed pets, the menu is rescanned on every open, ✓ on
      the selected one) and **Make a new pet** (`claude` starts in a new terminal with the `hatch`
      skill, empty working folder `~/.bop/hatch/`; at the end the skill asks "shall I switch to
      this pet?"). The pet chat's tools do not change: work that writes is done in the terminal.
    - `/bop use <id>` writes the same file; the pet watches the file and switches without a restart
      (asset permission is added for the new folder, `claude-pet.json` and animations are reloaded).
    - If there is no pet or the selected pet is not found, the Pitir embedded in the exe opens.
      `--pet` is for development only.
    - (Same day, in the app) The first message of "Make a new pet" is a plain sentence, not a
      command: `claude "Let's make a new Bop pet!"` (CLI docs: `claude "query"` opens an
      interactive session with a first message; there is no way to fill the input box without
      sending). It looks like the user's message; the `hatch` skill's description is written to
      recognize this request, it stays model-invocable. Source: Alperen.

27. **The pet drawing engine is in Rust, an exe subcommand** (`<exe> hatch ...`), the `png` crate
    as a direct dependency (already in the Tauri tree). Source: Alperen (approved). Reason: the
    same code on three platforms, no extra install for the user. The C# from the Pitir prototype
    (scratchpad) will be ported to Rust.
    Testing: Windows on a real device, Linux in WSL, macOS only through GitHub Actions builds +
    automated tests.
26. **Plugin name `bop`.** Tagline: "Bop — a desktop pet for Claude Code" (research: "for Claude
    Code" is fine in plain text, not as part of the name). Source: Alperen. Note: being a single,
    common word, there is a risk of a "Name is taken" or "generic" hold on the portal; to be
    checked before submission. `~/.claude-pet`, the exe name and the repo name will change
    accordingly.

25. **The published default pet is Pitir** (a mandarin drawn with code by the pet creation skill).
    Source: Alperen. Reason: original, no license issue; no permission for Clawd
    (`docs/research/1-directory-and-brand.md`). Closes the default pet question of Phase 6.
24. **The exe is downloaded from GitHub Releases on first run**, verified with sha256, stated
    clearly in the README. Source: Alperen. Reason: putting the exe inside the plugin exceeds the
    size limit and gets held up in review; there is no official download mechanism
    (`docs/research/2-technical.md`).
23. **The first release targets Windows, macOS and Linux** (changes decision 16). Source: Alperen
    ("let's target macOS and Linux from the first release"). Reason: acceptance into the official
    or community directory. Consequence: platform-dependent parts (hook wrapper, drawing engine,
    "Open in terminal", Actions) are reworked to run on three platforms. The plugin name cannot
    start with `claude-` (`claude plugin validate` reports an error); a new name will be chosen.

22. **The pet creation skill works without an image generation API, by drawing with code.** The
    character is defined once as data (grid + palette + parts), frames are derived from code;
    Claude looks at the output and fixes it. Goal: a good result for simple requests, a result
    close to what was asked for detailed ones. Reference: Codex `hatch-pet` (idea only; its
    `$imagegen` path does not exist in Claude Code). Source: Alperen ("I don't want to use an
    API"). Reason: the scratchpad prototype ("Pitir", full v2 atlas, PowerShell + .NET
    System.Drawing, no extra dependency) was liked by Alperen.
21. **Everything that goes into the plugin is English:** skill files, manifest, hook and
    user-facing strings, README. Internal project docs (`docs/`, CLAUDE.md) stay Turkish. Source:
    Alperen ("we'll offer it globally"). The Turkish strings in the app UI will also be translated
    in Phases 5–6. (Changed by decision 35: internal docs are English too.)

## 2026-10-01

20. **Files or web, never both in the same chat** (changes decision 19). Source: Alperen (option
    A). Reason: third code review.
    - Trusted folder: `--tools "Read,Glob,Grep"`, no `--allowedTools`. A file that was read cannot
      be put into the query of an unapproved web search and carried out. Links in the reply are not
      made clickable. For a user who wants the web, the instructions suggest "Don't trust" in a
      new chat.
    - Untrusted chat: only `WebSearch`, and the process runs in the empty `~/.claude-pet/chat`
      folder. Previously only the tool list was narrowed; `claude -p` started in the untrusted
      folder and that folder's `.claude/settings.json` hooks and CLAUDE.md (including `@` imports)
      were loaded.
    - The trust question is answered once per session; a second answer is ignored (double click).
    - The home directory warning is also shown for folders that contain the home directory (e.g.
      `C:\`).
19. **File reading only in a trusted folder; a new chat asks in the bubble** (narrows decision 18).
    Source: Alperen ("like trust this folder"). Reason: code review; when the project folder was
    unknown the chat started in the home directory, and `Read` needs no permission there (`.ssh`,
    `.claude/.credentials.json`). An instruction embedded in a search could make it read a secret
    and put it into a "source" link; a click would leak it.
    - Before the first message of a new chat the bubble asks "Trust this folder?" (with an extra
      warning in the home directory). Trust → `Read,Glob,Grep,WebSearch` and the folder is written
      to `~/.claude-pet/trusted.json`, never asked again. Don't trust → only `WebSearch` (for that
      chat).
    - If the project folder is unknown the chat starts in the empty `~/.claude-pet/chat`, no
      question is asked (only `WebSearch`). The home directory never becomes the working folder by
      itself.
    - The real domain is always shown next to the title of a source link.
    - The system prompt changes with the tools; since it is saved on the first request, no message
      is sent until the question is answered.
18. **Only read-only tools are enabled in the pet chat** (changes the "no tools" part of decision
    7): `--tools "Read,Glob,Grep,WebSearch" --allowedTools "WebSearch" --permission-prompts none`,
    plus `--append-system-prompt` tells Claude it is in a bubble and what its limits are. Source:
    Alperen. Reason: without tools Claude wrote a tool call as plain text for a weather question
    (`totalToolDuration: 0` in the transcript). According to the docs (tools-reference,
    2026-10-01): `Read`/`Glob`/`Grep` need no permission in the working folder but do outside it
    (the prompt is denied by `none`); `WebSearch` and `WebFetch` need permission. `WebFetch` is
    deliberately off: a session that reads files could send a request to an arbitrary address and
    leak content. The instructions apply only to new chats (CLI docs: the system prompt is saved
    on the first request).
    - The bubble is narrow: the instructions ask for 1–3 short sentences of plain text; no
      unsolicited suggestions/advice (Alperen: suggestions can lead to unexpected results). Sources
      are not written in the text; if they come as a trailing line of links they are shown in the
      bubble under a collapsed "Sources (n)", and on click they open in the browser with
      `rundll32 url.dll,FileProtocolHandler` (http/https only; `explorer.exe` split the address at
      commas). Source: Alperen.
17. **The hook path is given with delayed expansion, not as text** (changes the format of decision
    15): `"args": ["/d", "/v:on", "/c", "!CLAUDE_PLUGIN_ROOT!/hooks/claude-pet-hook.cmd", "<Event>"]`
    (`!CLAUDE_PROJECT_DIR!` in the repo settings). The wrapper starts with
    `setlocal DisableDelayedExpansion`. Reason: code review; the `${...}` placeholder wrote the path
    as plain text into the `cmd /c` line, and it stayed unquoted if the path had no spaces. All
    hooks broke on paths containing `&`, `%`, `^`, `(` (tested). `!VAR!` expands after parsing; in
    the folders `plain`, `R&D`, `a b`, `R&D b`, `x%PATH%y`, `c^d`, `e!f`, `(g)` the exe was called
    with the correct argument every time, exit 0. Environment variables are passed to the hook
    process (hooks docs: "both export them as the environment variables
    `CLAUDE_PROJECT_DIR`, `CLAUDE_PLUGIN_ROOT`, and `CLAUDE_PLUGIN_DATA`", read on 2026-10-01).
    Not yet tested with a real Claude session.
16. **The first release is Windows only.** The README says so clearly. GitHub Actions builds only
    Windows in the first release. Mac/Linux support in a separate later phase (PLAN.md, Phase 7);
    per-platform hook setup will be solved there. Source: Alperen. (Changed by decision 23.)
15. **Hooks call the `.cmd` wrapper with `cmd.exe` in exec form** (async, short timeout):
    `"command": "cmd.exe", "args": ["/d", "/c", "${CLAUDE_PLUGIN_ROOT}/hooks/claude-pet-hook.cmd", "<Event>"]`.
    Source: Alperen. Reason and measurements (2026-10-01):
    - Git Bash is not required on Windows (setup docs: "Installing Git for Windows is optional"),
      so bash was ruled out. `cmd.exe` exists on every Windows.
    - Time: `cmd` wrapper ~31 ms, PowerShell ~271 ms (average of 10 measurements). Since the hook
      is async neither blocks Claude ("runs in the background without blocking", hooks docs), but
      `cmd` puts much less load in the background.
    - `cmd` passes stdin through without re-encoding; Turkish paths were passed intact.
    - On Windows `${CLAUDE_PLUGIN_*}` is substituted with forward slashes (plugins-reference);
      tested with a path with spaces and mixed separators, it works.
    - `.cmd` files must be ASCII only: `cmd` reads the file with the OEM code page, and a Turkish
      comment line broke command parsing (tested). The rule is in CLAUDE.md.
    - There is no `cmd.exe` on Mac/Linux; a hook that cannot start produces a "Failed with
      non-blocking status code" notice, and hooks have no platform field. Hence decision 16.
14. **Location of the exe: `${CLAUDE_PLUGIN_DATA}/bin/`.** If the exe version does not match the
    plugin version, a new one is downloaded. Source: Alperen.
    - Source of behavior: Claude Code, *Plugin manifest reference → Environment variables*
      (https://code.claude.com/docs/en/plugins-reference#environment-variables, read on 2026-10-01):
      "`${CLAUDE_PLUGIN_DATA}` | `~/.claude/plugins/data/<id>/`, created on first reference and kept
      across plugin updates." and "By default, Claude Code deletes the `${CLAUDE_PLUGIN_DATA}` directory
      when you uninstall the plugin from the last place it's installed." (`--keep-data` exception:
      plugin uninstall docs.)
    - According to the same docs these variables are passed to hook processes, but not to commands
      Claude runs with the Bash tool. So the `/claude-pet` skill takes the path not from the
      environment but from `${CLAUDE_PLUGIN_DATA}` written in the skill text (Claude Code
      substitutes it on load).
    - Pet data (`state.json`, `chat.json`, `window.json`, pets) stays in `~/.claude-pet`; when the
      plugin is removed only the exe goes, the user's pets are not deleted.

## 2026-09-30 (scope session)

The project will be an open-source Claude Code plugin and will not be tied to a single pet.
Detailed reasoning: `PROJECT.md`.

1. **Distribution: Claude Code plugin + builds with GitHub Actions.** The plugin carries the
   `/claude-pet` skill, the hooks and the launcher. Actions builds the app for each platform and
   puts it on Releases; the plugin downloads it on first run. Reason: no Python/Node needed on the
   user's machine, a one-command install. Source: Alperen.
2. **Technology: Tauri (Rust + web UI).** Replaces the earlier Python + tkinter decision.
   Reason: real per-pixel transparency, bubble and animation are easy with HTML/CSS, small output,
   ready Actions support. Rust and VS C++ Build Tools will be installed. Source: Alperen's approval.
3. **Pet format: Codex format + optional `claude-pet.json`.** Codex has no official pet.json
   schema. The official source (openai/skills `hatch-pet`) defines 4 fields: `id`, `displayName`,
   `description`, `spritesheetPath`. `spriteVersionNumber` appears only in the community. No field
   outside these 5 is read. v1/v2 is told apart by image size (1536×1872 → 8×9, 1536×2288 → 8×11).
   The frame count is found by skipping empty cells (alpha = 0). Extra settings live in a separate
   `claude-pet.json`, so the same folder works both in Codex and in our app. Source: Alperen.
   - **v1 pets have no look rows:** following the mouse is silently disabled, with no error or
     warning. Source: Alperen.
4. **Commands:** `/claude-pet` (toggle), `list`, `use <id>`, `install <local-folder>`. Installing
   from a GitHub URL is postponed to the next phase. Source: Alperen.
5. **State bridge: `~/.claude-pet/state.json`.** Writes are atomic (temporary file first, then
   rename, with a short retry on Windows). In the first release the last writer wins, but every
   record keeps the `session_id`. Source: Alperen.
6. **The hook is run by the exe's own subcommand:** `claude-pet hook <event>`. It opens no window,
   the exit code is always 0. If the exe is missing the hook exits silently and never breaks or
   slows down Claude Code (a small wrapper is needed for the existence check). Source: Alperen.
7. **Pet messages go to a separate pet session:** the first message is sent with
   `claude -p --output-format json`, later ones continue with `--resume <id>`. No tool permissions
   are given. The "Open in terminal" button in the bubble runs `claude --resume <id>`. The pet
   never writes to a session handed off to the terminal again; it opens a new session. The
   session's folder is the last active Claude `cwd` and is fixed once the session opens. A folder
   label is shown in the bubble. Verified from the docs: `-p` sessions can be opened with
   `claude --resume <session-id>`. The bubble has an explicit **"New chat"** button that resets
   the stored `session_id`. On Windows, calling `claude -p` **does not flash a console window**
   (started as a hidden process). Source: Alperen.
8. **Movement: the pet stays in place,** it only moves when dragged. While idle it makes short
   small moves. The state → animation mapping is per pet and defined in `claude-pet.json`;
   otherwise the Codex defaults are used. Source: Alperen.
   - **Extra spritesheet:** same grid (192×208, at most 8 columns, any number of rows), rows are
     called by name. Invalid entries are skipped, the pet does not break. There is a
     `formatVersion` field. Source: Alperen.
9. **Default pet** (updated 2026-09-30): in development the default pet is `clawd-fan`
   (`dev-pets/clawd-fan/`, gitignored). Since it is a fan work inspired by Anthropic's design it
   does not enter the repository. **The published default pet will be decided in Phase 6.**
   Johnny is not embedded in the repository either; it stays an example pet. Source: Alperen.
10. **License: MIT (code).** (updated 2026-09-30) clawd-fan is a fan work, so it cannot be given
    CC BY. The license of pet images and whether a `LICENSE-ASSETS` is needed will be decided when
    the published default pet is chosen (Phase 6). Source: Alperen.
11. **Johnny is removed from the repository:** moved to the `dev-pets/johnny/` folder in
    `.gitignore` for local development, and the first commit is recreated without Johnny.
    Source: Alperen.
12. **Before submitting to the directory**, Anthropic's plugin directory submission criteria and
    brand guidelines are read. Source: Alperen.
13. **The spritesheet is loaded into the UI through the Tauri asset protocol,** instead of a byte
    array (JSON) over IPC. Scope is only the pet folders: the active pet folder is added to the
    permission at runtime, the static scope is empty. Source: Alperen.

## 2026-09-30 (initial setup)

- **Separate git repository (`D:\claude-pet`, branch `main`).** When the project is done it will be
  made open source and shared. For now local only, no remote. Source: Alperen.
- **The Johnny sprite will be used as it is.** Reason: the Codex pet format already includes
  9 states and 16 look directions. Source: Alperen.
- ~~**Technology: Python + tkinter + Pillow.**~~ Replaced by Tauri (decision 2 above).
- **Chat first, then the hook bridge.** Reason: small, visible steps. Source: planning.
- **Coding and planning will be done in separate windows**, in a separate repository folder.
  Source: Alperen.
