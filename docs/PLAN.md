# Plan

Decisions: [DECISIONS.md](DECISIONS.md). Architecture: [../PROJECT.md](../PROJECT.md).

## Phase 0: Preparation

- [x] Scope and architecture decisions (2026-09-30)
- [x] Move Johnny out of the repo → `dev-pets/johnny/` (gitignored), recreate the first commit
- [x] Rust (`rustup`, user level); VS 2022 C++ tools were already installed
- [x] Development pet: clawd-fan → `dev-pets/clawd-fan/` (v2). Default pet decision pending, see STATUS.md

## Phase 1: Pet on screen (Tauri)

- [x] Tauri skeleton: transparent, borderless, always-on-top window
- [x] Pet loader: `pet.json` (5 fields), v1/v2 from image size, empty cells skipped
- [x] `idle` loop (tested with clawd-fan and Johnny)
- [x] Test with a v1 pet: look silently off, no error (temporary 1536×1872 test pet)
- [x] Drag and drop + `running-left/right` while dragging
- [x] Right-click menu: quit

## Phase 2: State bridge (hooks)

- [x] Read the hooks documentation: events, input fields (`session_id`, `cwd`, `tool_name`), Windows shell, `timeout`
- [x] State vocabulary and hook → state mapping (`state.rs`, tested)
- [x] `claude-pet hook <event>`: stdin → `state.json` (atomic write, exit code always 0, ~25 ms)
- [x] Wrapper that exits silently when the exe is missing (`hooks/claude-pet-hook.sh` + `.ps1`)
- [x] The pet watches `state.json` and switches animation
- [x] In-repo development hooks: `.claude/settings.json` (async, this repo only)

## Phase 3: Chat

- [x] Message box on click (window grows upward, bubble below if there is no room; the pet stays in place on screen)
- [x] `claude -p --output-format json` and `--resume <id>`, no tools (`--tools ""`, `mcp__*` off, `--permission-prompts none`)
- [x] Hidden process on Windows (CREATE_NO_WINDOW, no console window while waiting — tested)
- [x] Session folder: the user's last Claude `cwd` (the pet does not overwrite the session), fixed once the session opens, folder label in the bubble
- [x] Bubble: reply, "New chat" button (session id is reset)
- [x] "Open in terminal": `claude --resume <id>` in a new terminal; the pet never writes to a handed-off session
- [x] Animations while waiting and on error
- [x] Tested with real claude (2 calls): reply in 5-6 s, `--resume` keeps context

## Phase 4: Pet system

- [x] `claude-pet.json` reader: `formatVersion`, `frameMs`, `sheets`, `animations`, `states`, `idleExtras` (Rust, tested)
- [x] Extra sheet validation (192×208, at most 8 columns), invalid entries are skipped with a warning
- [x] State fallback chain (`writing-code` → `running` → `idle`)
- [x] Small idle moves (every 15-30 s), following the mouse (v2 only, front half circle; silently off in v1)
- [x] Remember position (`~/.claude-pet/window.json`, ignored if off screen)

## Phase 5: Plugin "bop" (Windows + macOS + Linux)

Decisions: 21 (English), 23 (three platforms), 24 (exe from Release), 26 (name `bop`), 27 (Rust
drawing engine). Research: [research/](research/) (1 directory and brand, 2 technical, 4 cross-platform).

- [x] Decisions: exe in `${CLAUDE_PLUGIN_DATA}/bin` (14), first release on three platforms (23, replaces 16)
- [x] Research on submission requirements and technical requirements (2026-10-02)
- [x] Manually test and commit the uncommitted 2026-10-01 work (2026-10-02)
- [x] Rename: `claude-pet` → `bop` (exe, `~/.claude-pet` → `~/.bop`, code and docs)
- [x] Hooks: shell form, a single command readable by both sh and PowerShell; if the exe is missing,
      stdin is drained and it exits with 0 (`research/4-cross-platform.md`). The `.cmd` wrapper is removed.
- [x] `.claude-plugin/plugin.json` (`name: bop`, `displayName`, MIT) + `hooks/hooks.json` (29, 36)
- [x] `/bop` skill (toggle, `list`, `use <id>`, `install <local-folder>`) + exe subcommands
- [x] Active pet selection decision (28)
- [x] `~/.bop/pets/` + `config.json`, embedded Pitir fallback, live pet switching, exe subcommands
      (`list`, `use`, `install`, `toggle`, `stop`), single instance
- [x] Right-click: "Switch pet ▸"
- [x] Right-click: "Make a new pet" (first message in the terminal is "Let's make a new Bop pet!"; skill in Phase 6)
- [x] Test with `claude --plugin-dir`, `claude plugin validate --strict` (`/bop`, `hatch` flow with `-p`)
- [x] Exe download: GitHub Releases + sha256, stated clearly in the README (`/bop setup`, 33; not tested with a real release)
- [x] Directory review fix: no download; the app copies itself to `~/.bop/bin/` on first open (37)
- [ ] Release v0.2.0 with the self-copying app, then resubmit to the directory
- [ ] Platform differences: "Open in terminal" (macOS/Linux), Tauri 2.12.1+ (macOS transparency),
      graceful fallback for Wayland limits (always on top, position, mouse position)
- [x] GitHub Actions: Windows, macOS, Linux builds → Releases (written, never run)
- [x] Translate the app's Turkish strings to English (21, 31)

## Phase 6: Pet creation skill (`hatch`)

Decisions: 22 (no API, drawing with code), 27. Design: [research/3-hatch-design.md](research/3-hatch-design.md).
The prototype (C#) was ported to Rust: `app/src-tauri/src/hatch/`; the prototype code was removed before publishing.

- [x] Align `SPRITE.md` with Codex's current contract ((0,6) neutral cell, `spriteVersionNumber`, 180° = down, frame durations) (2026-10-02)
- [x] Rust drawing engine: `<exe> hatch <spec.json> <output>` (`png` crate); three-level spec
      (archetype+palette → parts → pixel map), pseudo-3D body rotation (2026-10-02)
- [x] Deterministic validation (atlas, cell, direction semantics) + contact sheet, automatic fitting
- [x] English `skills/hatch/SKILL.md` (< 500 lines) + `references/`; draw-look-fix loop
- [x] Regenerate Pitir with the engine → default pet (25)
- [ ] Test in a real session: "Make a new pet" → skill → `hatch` → `install` → `use` (passed with `-p`; interactive: Alperen)
- [ ] See the new Pitir in the app by hand (idle is now 6 frames, (0,6) neutral separate)

## Phase 7: Open-source release and submission

- [x] `LICENSE` (MIT), README (installation, downloads, limit warning, "not official" note)
- [x] Privacy policy (`PRIVACY.md`) and contact (GitHub Issues); links in `plugin.json` (2026-10-02)
- [x] Check that no personal paths or information remain (audit + clean single commit, 34)
- [ ] Name check (is `bop` taken/generic on the portal)
- [x] Public GitHub repository (with Alperen's explicit approval) (2026-10-02, `AlperenGonuk/bop`)
- [ ] Submission via claude.ai/directory/manage (paid plan + public repo)
- [ ] Next: `install <github-url>`
