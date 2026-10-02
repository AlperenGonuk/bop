# Bop — a desktop pet for Claude Code

> The Claude Code CLI version of the pet feature in Codex. An open-source Claude Code plugin.
> The pet on the desktop shows what Claude is doing through animation, and you can message it.

## Goals

- A pet that is toggled with the `/bop` command, always stays on top and can be dragged.
- **Not tied to a single pet:** existing Codex pets work as they are, and users can make their own.
- While Claude Code is working, the pet **reflects its state:** thinking, writing code, waiting for
  permission, done, error. A bubble appears above it while it works.
- **Message the pet** by clicking it: the reply appears in a bubble, and the chat can be handed
  off to the terminal if wanted.
- In development the default pet is clawd-fan and the test pet is Johnny (both local only,
  `dev-pets/`). The published default pet is Pitir (DECISIONS 25).

## Out of scope (for now)

- Sound, voice conversation
- Wandering around the screen on its own (the pet stays in place)
- Installing a pet from a GitHub URL (next phase)
- Showing several Claude sessions separately at the same time (`session_id` is already recorded)

## Architecture

```
PLUGIN (Claude Code)
  /bop skill ──► bop.exe (toggle, list, use, install)
  hooks.json ──► hooks/bop-hook.sh|.ps1 ──► bop hook <event> ──► ~/.bop/state.json
                 (silent exit if no exe)           (atomic write)        │
                                                                             ▼
APP (Tauri, built by GitHub Actions, downloaded from Releases)
  transparent window ◄── watches state.json ──► state → animation (per pet)
       │
       └─ message ──► claude -p --output-format json [--resume <id>] ──► bubble
                      (hidden process, read-only tools, fixed folder)
                      in the bubble: "New chat" · "Open in terminal" (claude --resume <id>)
```

## Pet format

Pet folder: `~/.bop/pets/<id>/`

| File | Required | Content |
|---|---|---|
| `pet.json` | yes | Codex format. Only `id`, `displayName`, `description`, `spritesheetPath`, `spriteVersionNumber` are read |
| `spritesheet.webp` | yes | v1: 1536×1872 (8×9), v2: 1536×2288 (8×11). Version is inferred from the image size |
| `bop.json` | no | Our extension: extra sheets, named animations, state mapping, idle moves |

- v1 pets have no look rows; following the mouse is silently disabled.
- Extra sheets use the same grid (192×208, at most 8 columns). Invalid entries are skipped.
- Details: [docs/SPRITE.md](docs/SPRITE.md), decisions: [docs/DECISIONS.md](docs/DECISIONS.md).

## Technology

- Tauri (Rust + HTML/CSS/JS): window, transparency, animation, bubble
- GitHub Actions: platform builds → GitHub Releases
- Claude Code: plugin system, hooks, `claude -p`

## Caveats

- `claude -p` uses the user's own plan limits. The README must say so clearly.
- Headless mode cannot ask for permission. The pet only chats; work that needs permission is done
  through "Open in terminal".
- Development pets (`dev-pets/`) stay local and never enter the repository.
- "Claude" is a trademark of Anthropic; the README carries a "not official" note.
- License: code MIT. The default pet Pitir is drawn by code
  (`skills/hatch/examples/pitir.json`), same license.

## Documentation system

| File | Purpose |
|---|---|
| `PROJECT.md` | This file: goals, scope, architecture. Rarely changes. |
| `.claude/CLAUDE.md` | Start-of-session instructions for coding sessions |
| `docs/STATUS.md` | Where we are now, next step. **Updated at the end of every session.** |
| `docs/PLAN.md` | Phases and to-do list |
| `docs/DECISIONS.md` | Dated decision log |
| `docs/SPRITE.md` | Spritesheet technical reference |
| `docs/research/` | Research notes (directory and brand, technical, hatch design, cross-platform) |
