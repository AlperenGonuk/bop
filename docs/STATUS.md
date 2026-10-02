# Status

**Updated:** 2026-10-02

## Now

Phases 1–4 are done; the app is a standalone pet: it can be dragged, shows Claude's state through
hooks, you can chat through it (terminal-style bubble), and a per-pet `bop.json` is read.
2026-10-02: submission and technical requirements were researched (`docs/research/`), decisions
21–27: plugin name `bop`, everything in English, first release on Windows + macOS + Linux, exe from
Releases, a `hatch` skill that draws pets with code (Rust), default pet Pitir. The plugin is under
the repository root (DECISIONS 36): manifest, hooks, `/bop` and `hatch` skills. Phase 6 (drawing pets with code) is done
with code and tests, not tried in a real session.

Running by hand (without the plugin):

```
cd app\src-tauri
cargo build --release
target\release\bop.exe --pet D:\claude-pet\dev-pets\clawd-fan
```

Claude sessions opened in this repo write to `~/.bop/state.json` through the `.claude/settings.json`
hooks (the hook looks for the repo's **debug** exe, so `cargo build` is needed first).

## Next step: first release (with Alperen's approval)

Local preparation is done (DECISIONS 33–34). Order:

1. ~~`gh repo create AlperenGonuk/bop --public` and push the clean `main` (needs approval).~~
   Done 2026-10-02.
2. ~~Add `homepage`, `repository`, privacy and support links to `plugin.json`.~~ Done 2026-10-02
   (plus `icon`, `documentationUrl`; `validate --strict` passes).
3. ~~See CI pass; tag `v0.1.0` → Actions draft release~~ (done 2026-10-02, 4 executables +
   SHA256SUMS in the draft) → published by Alperen 2026-10-02.
4. ~~Real installation on Windows~~ Done 2026-10-02: release `v0.1.0` published,
   `claude plugin marketplace add AlperenGonuk/bop`, `claude plugin install bop@bop`, install
   script downloaded and verified the real exe, `/bop` opens the pet (after restarting Claude
   Code; a running session does not see a newly installed plugin). Not tried: macOS, Linux.
   Data folder `~/.claude/plugins/data/bop-bop/`.
5. Directory submission (Alperen, claude.ai/directory/manage, paid plan): Submit new → Plugin
   bundle → repository `AlperenGonuk/bop`, plugin path empty (root) → **Validate**. Expected
   holds for a reviewer: `icon.ico`, the downloaded executable, maybe the short name `bop`.
   Fix anything marked Blocking, then submit.

Linux testing (WSL Ubuntu 24.04, WSLg): a clean clone in `~/bop` (updated with `git pull`), Rust at
user level (rustup), Tauri packages installed from the CI list. CI steps passed by hand:
`cargo test --locked` 42/42, `cargo build --release --locked`, `--version` smoke test,
`install.sh` with the real exe + a local fake release (127.0.0.1) in 6 cases. The pet opens in
WSLg. The first attempt surfaced two Linux issues, both fixed (`\`/`:` in the pet path, `valid_url`
warning). Note: passing paths from Git Bash to `wsl.exe` needs `MSYS_NO_PATHCONV=1`.

Known limits: on macOS/Linux "Open in terminal", "Make a new pet" and source links are still
"Windows only" (the README says so). On Wayland there is no always-on-top/position/mouse position.

## Remaining limits

- **Following the mouse** uses only the front half circle (−90°…+90°): 180° is a back view in
  Johnny and a downward look in clawd-fan. If wanted, a look-mode setting in `bop.json` (needs a
  format decision).
- **Hooks are async:** consecutive events can rarely be written in reverse order; the visual
  effect is brief.
- **Click area:** the pet's transparent edges also catch clicks.
- **"Bubble above the pet while Claude works":** in PROJECT.md, not a PLAN item; animation only
  for now.
- **"Open in terminal"** not tried by hand in a real terminal (verified with a fake `claude`).
- **The hooks' PowerShell path** (Windows without Git) and macOS `/bin/sh` not tried in a real
  Claude session.

## Open questions

- Format of the exe version compatibility check (Phase 5, together with Actions).
- `hatch` defaults (DECISIONS 30) await Alperen's approval: no detached effects, 48×52 grid, no
  intermediate approval point (no questions for a simple request; "shall I switch?" after install).
- Do pet images need a separate license (Pitir is ours; DECISIONS 9, 10, 25).

## Last session

- 2026-10-02 (13): Portal **Validate** passed with no Blocking findings. Policy holds: broad
  `allowed-tools` (fixed: each skill now pre-approves only the app subcommands and install
  commands it uses; tested with `--permission-mode default`: `/bop list` and `/bop setup` run, an
  unlisted command is denied), images "the code could run" (12; embedded pet data, not run),
  `icon.ico`, "uses a credential" (the app only *removes* inherited `CLAUDE_CODE_*` session
  variables; `GH_TOKEN` is in the release workflow), name close to the connector "bocp".
  Warnings: unrecognized `plugin.json` fields (3), hook output uninspected (2),
  download-and-run (2, the install scripts).
- 2026-10-02 (12): Checked the repository against the directory's pre-submission checklist:
  hook commands in a plugin subfolder were **Blocking**. Plugin moved to the repository root,
  hook logic moved to `hooks/bop-hook.sh` / `.ps1` (DECISIONS 36). README: Markdown image, "How
  the hooks work", Claude Code only note. Tested: both scripts in bash and PowerShell with and
  without the app; `validate --strict` (plugin + marketplace); tests 42/42; install from a clean
  clone as a marketplace and a real `claude -p` session wrote `SessionEnd` to the state file.
  Version stays 0.1.0 (the app did not change; the v0.1.0 release still matches).
- 2026-10-02 (11): English translation committed (tests 42/42), `plugin.json` got `homepage`,
  `repository`, `author.url`, `icon` (now `assets/icon.png`), `documentationUrl`, `supportUrl`,
  `privacyPolicyUrl` (`validate --strict` passes, links resolve). CI green, tag `v0.1.0` pushed,
  release workflow built all 4 platforms and made the draft release. WSL Ubuntu was removed from
  this PC, so the WSL notes above are history.
- 2026-10-02 (10): Repo published (`AlperenGonuk/bop`, clean single-commit history; the old
  history stays in the local `backup/pre-public` branch). WSL Linux CI steps passed by hand (see
  above). Repository translated to English: docs renamed (`PROJECT.md`, `docs/STATUS.md`,
  `docs/DECISIONS.md`, `docs/research/`), code comments and docs in English (DECISIONS 35).
- 2026-10-02 (9): Release preparation (subagent): `/bop setup` + `install.ps1/.sh` (tested locally
  with a fake release on Windows, Git Bash, WSL), `bop --version`, Actions (draft release + ci),
  marketplace.json, LICENSE, English README, PRIVACY.md, Tauri 2.12.1. Audit (subagent): author
  e-mail in all commits, personal paths in two reports; cleaned up, icon from Pitir, history
  restarted with a single commit (DECISIONS 34). Old history: local `backup/pre-public` branch.
- 2026-10-02 (8): `/code-review high` on top of Phase 6 (`1ccc346`) (10 findings): all fixed
  (overrides carry over when shrinking, no shrinking when an override overflows, look-around with
  the neutral pose and in the front half circle, Pitir test with 0.5% tolerance, `depth`/`width`
  ≤ 0 is an error, laptop/arm rule, pixel color through a single helper, hatch got faster; exe
  console subsystem, DECISIONS 32). Sprite rule clarified (DECISIONS 31, CLAUDE.md). UI in English
  (bubble, menu, instructions, pet loading errors). Tests 42/42. For Alperen to see: English bubble
  and menu, look-around.
- 2026-10-02 (8): Phase 6, `hatch` (subagent, Alperen "don't ask until it's done"). Rust engine
  `app/src-tauri/src/hatch/` (spec; rig: pseudo-3D yaw/pitch on an ellipse, depth order, outline;
  motion: 11 rows; validate; atlas + contact sheet), `png = "0.18"` (DECISIONS 27).
  `bop hatch <spec> <folder>`, `--example`, `--check <png>`. Validation: atlas format, (0,6),
  clipping, static row, detached part, look continuity; direction semantics measured from the eye
  pixels; an overflowing pet is shrunk down to 80%. Pitir was regenerated from
  `plugin/skills/hatch/examples/pitir.json` (`default-pet/`), without the detached effects of the
  old prototype. Skill `plugin/skills/hatch/` (SKILL.md + 3 reference files + 4 examples).
  SPRITE.md written to the Codex contract; sprites.js removes (0,6) from idle in v2. `cargo test`
  39 tests, validate --strict passed. App not tried by hand, real `claude` calls: 0. No commit
  (DECISIONS 30). **When the spec or engine changes**, refresh `default-pet/` with
  `bop hatch plugin/skills/hatch/examples/pitir.json <tmp>`, otherwise the
  `gomulu_pitir_motorun_ciktisi` test fails.
- 2026-10-02 (7): No code, a research and decision session. With subagents: a code-drawn pet
  prototype ("Pitir", PowerShell + C#, full v2 atlas; Alperen liked it, prototype code removed
  before publishing), directory/brand, technical, hatch design and cross-platform research
  (`docs/research/1-4`). Decisions 21–27. PLAN Phases 5–7 rewritten. Then the 2026-10-01 work was
  tested by hand (9 items, all fine; 6: the bubble stays open after the right-click menu, not a
  problem for Alperen, clicking empty space closes it) and committed. Alperen wanted to enable web
  search in the trusted chat; the auto mode classifier blocked it, Alperen said "leave it as is"
  (DECISIONS 20 stands). Active pet selection decided (DECISIONS 28: right-click "Switch pet" and
  "Make a new pet").
  Rename `claude-pet` → `bop` done: exe `bop.exe`, `~/.bop` (old `~/.claude-pet` not deleted, data
  copied), `bop.json`, `BOP_*` environment variables, `hooks/bop-hook.cmd`, Tauri identifier
  `io.github.alperengonuk.bop`. The repo folder is still `D:\claude-pet`.
  Hooks moved to a single polyglot command, `.cmd` removed; plugin under `plugin/` (manifest +
  hooks), validate --strict passed (DECISIONS 29).
  Exe: `pets.rs` (installed pets, `config.json`, embedded Pitir `app/src-tauri/default-pet/` →
  `~/.bop/pets/pitir/`, single instance via a `running.json` heartbeat + closing via a `control`
  file), subcommands `list/use/install/toggle/stop`, right-click "Switch pet ▸", live switching
  (`pet-changed` → page reloads). Alperen tried it: switching and Pitir animations fine.
  clawd-fan and Johnny installed locally in `~/.bop/pets/` (not in the repository).
  `/bop` skill (`plugin/skills/bop/SKILL.md`, English, `disable-model-invocation`, `allowed-tools`
  with the exe path). `claude -p "/bop:bop list" --plugin-dir plugin` worked, no permission denial.
  `--plugin-dir` data folder: `~/.claude/plugins/data/bop-inline/` (debug exe copied there by
  hand). Open: in `-p`, unprefixed `/bop list` did not reach the skill (Claude ran the repo exe
  from the project context); `/bop` should be tried in an interactive session.
  Unprefixed `/bop` works (Claude Code shows it as `/bop:bop`; my `-p` attempt was broken by Git
  Bash turning the `/bop` argument into a path, `MSYS_NO_PATHCONV=1` is needed).
  Right-click "Make a new pet": `claude "Let's make a new Bop pet!"` in a new terminal in
  `~/.bop/hatch` (Alperen tried it; with no skill yet Claude made up its own questions: character,
  name, make it active?).
  Real `claude` calls: ~11 (Alperen ~6, skill tests 5).

- 2026-10-01 (6): Code review (`806464d..HEAD`, 10 findings); 1–8 fixed, the small ones (9
  `.gitattributes` `*.sh`, 10 `autoGrow` constants) pending. Chat: lost-session check only on a
  failed run, dead id removed immediately, 5 min total including the retry, a note in the bubble
  when switching to a new session. Bubble: does not close while a reply is pending / when the
  terminal or the right-click menu takes focus; bounds from all animations; `~` only for the real
  home directory. Hook: DECISIONS 17. Alperen tried the bubble fixes (terminal, right-click, focus
  while waiting) and the lost-session note, no problems. Then read-only tools were enabled for the
  pet chat (DECISIONS 18), short-answer instruction, collapsible "Sources". Second review (10
  findings) all fixed: folder trust question (DECISIONS 19), links with parentheses/commas
  (`rundll32`), timed `keepOpen`, closing 20 s after a reply arrives in the background, source
  splitting only on known headings, path comparison by folder name, sprite cells scanned once,
  home directory in one place (`state::user_home`), CLAUDE.md tool rule updated. Trust question not
  tried by hand. Third review (10 findings) all fixed: DECISIONS 20 (files or web; untrusted chat
  in an empty folder), `<...>` addresses, bubble closing tied to a single rule (close when not
  focused, 500 ms polling; timed exception for menu/handoff/background reply), home directory
  warning on parent folders too, trust box redrawn when the home directory arrives late, double
  replies prevented, no href on source links, faint sheet does not shift the bubble. None tried by
  hand. Real `claude` calls: ~8 (Alperen's tests). No commit.
- 2026-10-01 (5): Alperen tried the pet. Fix: the pet inherited the environment variables of the
  Claude session that launched it (transcript saving and colors were off) → now cleared, new
  session on a lost session (`1e8bad0`). Bubble: terminal style, docking to the pet, closing on
  outside click (`25e75d2`, Alperen approved). Decisions 14–16 and the `.cmd` wrapper (`b943daa`).
  Real `claude` calls: 0.
- 2026-09-30 (4): Asset protocol; Phases 1–4 done. Real `claude` calls: 2.
- 2026-09-30 (3): Rust installed, Tauri skeleton, pet loader and idle.
- 2026-09-30 (2): Scope session, 12 decisions. Johnny removed from the repo.
- 2026-09-30 (1): The idea took shape. Folder, docs and assets created.
