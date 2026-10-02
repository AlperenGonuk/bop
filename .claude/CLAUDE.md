# Bop — session instructions

A desktop pet for Claude Code (like the Codex pet), an open-source plugin. Supports multiple pets.
User: Alperen. Short, concrete, one step at a time.

## Language

- Talk to the user (Alperen) in Turkish; everything written into the repository (docs, code
  comments, commit messages, logs, UI) is in English (DECISIONS 35).

## Session start

Read docs/STATUS.md, then docs/PLAN.md; PROJECT.md and docs/SPRITE.md when needed.

1. `docs/STATUS.md`: where we are, next step
2. `docs/PLAN.md`: active phase
3. When needed: `PROJECT.md` (goals, architecture) and `docs/SPRITE.md` (atlas)

## Rules

- Read the relevant file before writing code; never invent APIs or signatures from memory.
- Move in small steps: every step must be runnable and visible.
- Windows 11 first, but the first release targets Windows, macOS and Linux (DECISIONS 23, replaces
  16). Technology: Tauri (Rust + HTML/CSS/JS); ask before adding a dependency.
- Do not modify sprite files in `dev-pets/` or pets that come from elsewhere. Johnny lives only
  locally in `dev-pets/johnny/` and never enters the repository. `app/src-tauri/default-pet/` is
  engine output: it is regenerated from `skills/hatch/examples/pitir.json` and never edited
  by hand (DECISIONS 31).
- The pet chat's (`claude -p`) tools are only those in DECISIONS 18–20: in a trusted folder only
  file reading (`Read`, `Glob`, `Grep`), in an untrusted chat only `WebSearch` (in the empty pet
  folder). Never combine the two in the same chat. Do not enable a tool that writes, runs commands
  or opens addresses (`Bash`, `Edit`, `Write`, `WebFetch`, etc.) without asking Alperen; such work
  is done through "Open in terminal".
- For Claude Code behavior (hooks, plugins, `claude -p`) read the official documentation first;
  do not write from memory.
- Hooks must never break or slow down Claude Code: on error they exit silently.
- The repository root is the plugin (DECISIONS 36): everything committed is copied to users and
  scanned by the directory. Keep files other than images under 256 KiB, add no binaries other
  than PNG/JPEG/GIF/WebP images, and show images in Markdown only with `![...](...)` syntax.
- **`.cmd` / `.bat` files contain ASCII only** (comments included). `cmd` reads the file with the
  OEM code page; non-ASCII characters break command parsing (DECISIONS 15).
- Do not run UI tests that use the real mouse/keyboard while the user is at the computer without
  asking first.

## Session end (handoff)

Update docs/STATUS.md 'Last session' and 'Next step', tick docs/PLAN.md, add a dated entry to
docs/DECISIONS.md for lasting decisions.

1. `docs/STATUS.md`: update the "Last session" and "Next step" sections.
2. `docs/PLAN.md`: tick finished items.
3. If a lasting decision was made: add a dated entry to `docs/DECISIONS.md`.
