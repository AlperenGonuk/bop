# Bop privacy policy

Last updated: 2026-10-02

Bop is a desktop pet for Claude Code. It has no server, no account, no analytics and no
telemetry. The author receives no data from you.

## What Bop stores, and where

Everything stays on your computer:

- `~/.bop/` holds the pet's own files: `state.json` (what Claude Code is doing right now: the
  hook event, tool name, session id and working folder), `chat.json` (the id of the pet's chat
  session and its folder), `trusted.json` (folders you allowed the pet chat to read),
  `window.json` (window position), `config.json` (the active pet), `running.json` and `control`
  (used to open and close the single pet window), installed pets in `pets/`, and empty working
  folders `chat/` and `hatch/`.
- The Bop app itself lives in the plugin's data folder (`~/.claude/plugins/data/<plugin-id>/bin/`).
  Claude Code deletes that folder when you uninstall the plugin. Delete `~/.bop/` yourself to
  remove the rest.

Bop keeps these files until you delete them. It does not store your prompts or Claude's
replies; the chat history of the pet's chat is kept by Claude Code itself, like any other
Claude Code session.

## What leaves your computer

- **Setup download.** `/bop setup` downloads the Bop app and its `SHA256SUMS` file from the
  plugin's GitHub release (github.com, which may redirect to GitHub's file host
  `*.githubusercontent.com`). It sends nothing except the normal download request.
- **Pet chat.** Messages you type into the pet are sent by your own Claude Code (`claude -p`)
  under your own Claude account, exactly as if you had typed them in Claude Code. They count
  toward your plan's usage limits. Anthropic's privacy policy applies to that traffic.
- **Web search.** In a chat that is not allowed to read files, Claude may use Claude Code's
  built-in WebSearch tool when you ask for something from the web.

The Claude Code hooks Bop installs only run the local Bop app, which writes `~/.bop/state.json`.
They make no network requests.

## Children

Bop is a developer tool and is not directed at children under 18.

## Contact

Questions, problems and security reports: open an issue at
<https://github.com/AlperenGonuk/bop/issues>. For a security problem you do not want to
describe in public, open an issue asking for a private contact.
