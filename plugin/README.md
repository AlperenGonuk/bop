# Bop plugin

Bop is a desktop pet for Claude Code. A small pixel pet sits on your desktop, shows what Claude
Code is doing (thinking, editing, running commands, waiting for permission, done, failed) and
lets you chat with Claude in a speech bubble. This folder is the Claude Code plugin: two skills,
the hooks that tell the pet what is happening, and the scripts that install the Bop app.

## Contents

- `skills/bop`: the `/bop` command (`setup`, open or close the pet, `list`, `use`, `install`, `stop`).
- `skills/hatch`: creates new pets; Claude writes a JSON spec and the Bop app draws the frames.
- `hooks/hooks.json`: on Claude Code events, runs the Bop app if it is installed, which writes
  `~/.bop/state.json`. The hooks never download anything and never block Claude Code.
- `scripts/install.ps1`, `scripts/install.sh`: used by `/bop setup`. They download the Bop app
  for this plugin version from the project's GitHub release, verify it against `SHA256SUMS` and
  put it in the plugin data folder.

## Setup

After installing the plugin, run `/bop setup` once, then `/bop` to open the pet.

Pet chat messages run through your own `claude -p` and count toward your plan's usage limits.
Network access happens only during `/bop setup` and when Claude uses web search in the pet chat
because you asked.

Full documentation, downloaded files and checksums, platform notes and privacy details:
see the project README and PRIVACY.md at <https://github.com/AlperenGonuk/bop>.

Support: <https://github.com/AlperenGonuk/bop/issues>. License: MIT. Not affiliated with Anthropic.
