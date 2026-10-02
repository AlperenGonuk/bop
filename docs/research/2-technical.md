# Technical research: Claude Code plugin + skill structure (for claude-pet)

Date: 2026-10-02. Sources are the official docs downloaded on 2 October 2026 (`code.claude.com/docs/en/*.md`,
`platform.claude.com`, `agentskills.io`) and the `github.com/anthropics/*` repos. Raw copies:
`scratchpad/arastirma/raw/`.

> Note: The docs structure has changed. The old `plugins-reference` / `plugin-marketplaces` pages are now
> split into `plugins/manifest-reference`, `plugins/marketplace-reference`, `plugins/create-marketplace`,
> `plugins/host-marketplace`, `plugins/publish`, `plugins/loading`, `plugins/cli-reference`,
> `plugins/components`. Full index: https://code.claude.com/docs/llms.txt

---

## 0. The most critical findings to know first

1. **The name `claude-pet` is "reserved" as a plugin name.** `claude plugin validate` flags plugin
   names starting with `claude-`, `anthropic-`, `anthropics-` or `cc-plugin-` as an **error**:
   `Plugin name "<name>" is reserved: it passes as one of Anthropic's own`.
   `claude plugin init` and `claude plugin tag` reject such a name. Claude Code still installs and
   loads it, but `validate --strict` will cause trouble in CI and when applying to the Anthropic directory.
   Source: https://code.claude.com/docs/en/plugins/manifest-reference (section `name`)
   > "Starts with `claude-`, `anthropic-`, `anthropics-`, or `cc-plugin-` | Error"
   > "Has `claude`, `anthropic`, or `anthropics` as a whole word anywhere else, such as `mcp-for-claude` | Warning"

   In addition, the Agent Skills standard and Anthropic's skill authoring guide **forbid the words
   "claude" and "anthropic" in the skill `name` field** (for claude.ai upload and the Skills API).
   Source: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices
   > "Cannot contain reserved words: "anthropic", "claude""

   **Conclusion:** The plugin name (`name`) and skill names must not contain `claude`. Example: plugin `desk-pet`
   (or `pixel-pet`, `code-pet`), with the display name given separately via `displayName`. The name is permanent:
   changing it later breaks existing installs (see 1.6) and also changes the path of the
   `${CLAUDE_PLUGIN_DATA}` folder. **This decision must be made before the first release.**

2. **The `bin/` folder is special.** Contents of `bin/` at the plugin root are added to the Bash tool's `PATH`;
   claude.ai and Cowork **will not install** a plugin that has `bin/` at its root. Since the planned location is
   `${CLAUDE_PLUGIN_DATA}/bin`, this is fine, but do not create `bin/` at the plugin root; put scripts
   in a folder such as `scripts/`.
   Source: https://code.claude.com/docs/en/plugins/components (section Executables),
   https://code.claude.com/docs/en/plugins/host-marketplace (section Distribute through organization settings)

3. **There is no official "binary download" mechanism for distributing an exe.** The methods described in the docs:
   putting the binary in the plugin (copied into the cache on every version), the `archive` source (a zip
   pinned with sha256, at most 256 MiB), or doing the install via a hook/skill into
   `${CLAUDE_PLUGIN_DATA}` (the docs have an example for `node_modules`). Details: section 5.

4. **On Windows, an exe running from inside `${CLAUDE_PLUGIN_ROOT}` can break updates.** On Windows,
   if another program holds the installed copy, the update fails with "could not be replaced".
   This supports the decision to keep the exe in `${CLAUDE_PLUGIN_DATA}` (but even there the exe cannot be
   overwritten while running; the pet must be closed before replacing it).
   Source: https://code.claude.com/docs/en/plugins/troubleshooting
   > "On Windows, when another program holds the installed copy itself, the message instead says that copy `could not be replaced`..."

5. **In hooks, a `.cmd` cannot be run in exec form.** Exec form (with `args`) requires a real
   `.exe` on Windows; for `.cmd`/`.bat` you need either shell form or to call the `cmd.exe` executable with `args`.
   See section 7.

---

## 1. `.claude-plugin/plugin.json`

Source (unless stated otherwise): https://code.claude.com/docs/en/plugins/manifest-reference

### 1.1 Required / recommended fields

- The manifest is **optional**. Without it, components are found from the standard layout, and the name comes from the
  marketplace entry or the folder name.
- If a manifest exists, **the only required field is `name`**.
  > "`name` is the only required key."
- `validate` gives a **warning** for: a non-kebab-case name, missing `version`, `description`
  or `author`. `--strict` turns these into errors.
- The release checklist also recommends: `description`, `author`, `homepage`, `repository`,
  a `README.md` at the plugin root. If `homepage` cannot be resolved as a URL, the plugin **does not load**.
  Source: https://code.claude.com/docs/en/plugins/publish (Prepare your plugin for release)

All top-level fields (summary):

| Field | Note |
| :- | :- |
| `$schema` | For the editor; ignored at load time |
| `name` | Required, kebab-case; no spaces, `@`, `:`, or path separators. All components go into the `name:` namespace |
| `displayName` | Name shown in the UI; may contain spaces and capitals, not used in the namespace |
| `version` | Semver is not checked. If set, it keeps the user on that version (1.4) |
| `description`, `author` (`name` required; `email`, `url`), `homepage`, `repository`, `license` (SPDX), `keywords` | Metadata |
| `metadata` | Free-form object, not read by Claude Code |
| `icon`, `documentationUrl`, `supportUrl`, `privacyPolicyUrl`, `termsOfServiceUrl` | Only for the Anthropic directory listing |
| `defaultEnabled` | Default `true` |
| `dependencies` | Other plugins |
| `settings` | Only `agent` and `subagentStatusLine` take effect |
| `userConfig` | Values asked of the user (1.5) |
| `skills`, `commands`, `agents`, `hooks`, `mcpServers`, `lspServers`, `outputStyles`, `workflows`, `experimental.{themes,monitors,evals}` | Component paths |

- An unrecognized top-level field is **silently dropped** (validate warns). An unknown key inside `userConfig` options,
  `channels`, `lspServers` or `monitors`, however, is an **error** and the plugin does not load.

### 1.2 Path rules

- All component paths are relative to the plugin root and **must start with `./`** (`commands/foo.md` is invalid).
- A path cannot leave the plugin root (`..` is an error) and must exist.
- `skills` **adds to** the default `skills/` scan; `commands`, `agents`, `outputStyles` etc.
  **replace** the default; `hooks`, `mcpServers`, `lspServers` are **merged**.
- On macOS/Linux, a path containing backslashes is rejected; always use `/`.
  Source: https://code.claude.com/docs/en/plugins/loading (Paths that escape the plugin directory)

### 1.3 Standard folder layout

| Component | Default location |
| :- | :- |
| Manifest | `.claude-plugin/plugin.json` (only this file lives in `.claude-plugin/`) |
| Skills | `skills/<name>/SKILL.md` |
| Commands (legacy) | `commands/` ("Prefer `skills/` for new plugins") |
| Agents | `agents/` |
| Hooks | `hooks/hooks.json` (top-level `"hooks"` wrapper required) |
| MCP / LSP | `.mcp.json` / `.lsp.json` |
| Executables | `bin/` (added to Bash `PATH`; claude.ai/Cowork reject it) |
| Settings | `settings.json` |

- A `CLAUDE.md` at the plugin root is **not loaded**, and validate warns; instructions should be written as skills.
- Putting the `skills/` folder inside `.claude-plugin/` is a common mistake: it is not scanned.
  Source: https://code.claude.com/docs/en/plugins/troubleshooting (Plugin loads but its skills are missing)

Recommended skeleton for claude-pet (names are examples):

```text
desk-pet/                      # repo root = marketplace root = plugin root
├── .claude-plugin/
│   ├── plugin.json
│   └── marketplace.json       # single entry, "source": "./"
├── skills/
│   ├── pet/                   # /desk-pet:pet  (on/off, list, use, install)
│   │   ├── SKILL.md
│   │   └── scripts/...
│   └── hatch/                 # /desk-pet:hatch (draws sprites)
│       ├── SKILL.md
│       ├── references/...
│       └── scripts/hatch.ps1
├── hooks/
│   ├── hooks.json
│   └── pet-hook.cmd           # ASCII
├── scripts/                   # shared scripts (NOT bin/)
├── README.md
└── LICENSE
```

### 1.4 Versioning

Source: https://code.claude.com/docs/en/plugins/loading (Versions and updates),
https://code.claude.com/docs/en/plugins/host-marketplace (Release a new version)

- Claude Code computes a version for each plugin; an update is downloaded only when this version **changes**.
  Order: `plugin.json` `version` → marketplace entry `version` → source type (for GitHub/git, the first
  12 characters of the commit SHA).
- If you set `"version": "1.0.0"` and commit without bumping the version, users stay on the old copy
  (`is already at the latest version`).
- Two options: bump `version` on every release **or** remove `version` from both `plugin.json` and
  the entry (the commit SHA is then tracked). Do not set it in both (validate warns, `plugin.json` wins).
- In third-party marketplaces **auto-update is off by default**; the user turns it on via
  `/plugin` > Marketplaces > Enable auto-update or runs `claude plugin update`.
- The version is the name of the cache folder: `~/.claude/plugins/cache/<marketplace>/<plugin>/<version>/`.
  The old version folder is cleaned up after 14 days.
- `claude plugin tag` creates a `{name}--v{version}` tag (only needed if other plugins depend on you
  with a version range). Source: https://code.claude.com/docs/en/plugins/publish

### 1.5 `userConfig` (optional, could be useful)

- Types: `string`, `number`, `boolean`, `directory`, `file`; required fields `type`, `title`,
  `description`. `sensitive: true` writes to secure storage.
- Arrives in hook processes as `CLAUDE_PLUGIN_OPTION_<KEY>`; in skill text, `${user_config.KEY}`
  is substituted (non-sensitive ones only). In a shell-form hook command, `${user_config.*}` is an **error**.
- `claude plugin install` (shell) does not prompt; the `/plugin` UI does.
  Source: https://code.claude.com/docs/en/plugins/components (Ask the user for configuration values)

### 1.6 Renaming

- "Never change a published plugin's `name`." If needed, the marketplace `renames` map is used.
  Source: https://code.claude.com/docs/en/plugins/publish (Rename or remove a plugin)

### 1.7 What `claude plugin validate` checks

Sources: manifest-reference (Validate the manifest), https://code.claude.com/docs/en/plugins/cli-reference (plugin validate),
https://code.claude.com/docs/en/plugins/create-marketplace (Problems that validation reports)

- In the target folder it validates `.claude-plugin/marketplace.json` first, otherwise `plugin.json`, otherwise
  the `skills/`/`agents/`/`commands/` files.
- **Errors:** type mismatch; missing path or one escaping the root (`..`); unknown key in strict
  objects; reserved plugin name; invalid marketplace/plugin name; MCP entry problems.
- **Warnings:** unknown top-level field, non-kebab-case name, missing `version`/`description`/`author`,
  `CLAUDE.md` at the plugin root, unquoted `${CLAUDE_PLUGIN_ROOT}` in a shell-form hook, the word
  `claude` in the name, `version` in both the entry and `plugin.json`.
- **Not read:** a single `SKILL.md` at the plugin root; in marketplace mode, the plugins' skill/hook
  files (validate each plugin folder separately). To check whether skill frontmatter is broken,
  run `claude plugin validate ./skills` (v2.1.233+).
- Exit codes: 0 passed, 1 failed (`--strict` also counts warnings), 2 validator error; `--json` is available.
- Validation is not enough: test by adding a local marketplace and installing (errors like `Source path does not exist`
  only show up at install time).

---

## 2. Marketplace (`marketplace.json`) — our own GitHub repo

Sources: https://code.claude.com/docs/en/plugins/create-marketplace,
https://code.claude.com/docs/en/plugins/marketplace-reference, https://code.claude.com/docs/en/plugins/publish

### 2.1 Requirements

- File location: `.claude-plugin/marketplace.json` (anywhere else, `marketplace add` cannot find it).
- **Required:** `name`, `owner` (`name` required; `email`, `url` optional), the `plugins` array.
  If `description` is missing, validate warns.
- **Required** in each entry: `name`, `source`. Optional: `description`, `version`, `category`,
  `tags`, `strict`, `displayName`, `defaultEnabled`... (most of the plugin.json fields).
- **Plugin + marketplace in one repo** (our case): put `marketplace.json` next to `plugin.json`,
  single entry `"source": "./"`:

```json
{
  "name": "your-marketplace",
  "owner": { "name": "Your Name" },
  "plugins": [
    { "name": "deploy-helper", "source": "./" }
  ]
}
```
  (Source: publish, "Add the marketplace file to your repository")

- The entry `name` and the `plugin.json` `name` **must match**; if they differ, install may fail with `Plugin "<x>" not found
  in marketplace`.
- Relative paths are relative to the marketplace root (the parent of `.claude-plugin/`); `..` is forbidden.
- Do not use Git LFS: the clone does not download LFS content, files arrive as pointers (putting the exe in LFS
  does not work). Source: host-marketplace (Keep plugin files out of Git LFS)

### 2.2 Naming rules

- Marketplace `name`: letters, digits, `.`, `_`, `-`; starts with a letter/digit. At install time the user types
  `plugin@marketplace`.
- Reserved names: `claude-code-marketplace`, `claude-plugins-official`, `anthropic-plugins` etc.;
  names imitating the official name (`official-claude-plugins`, `claude-plugins-v2`), non-ASCII characters,
  `npm`/`github` etc., and names starting with `claudeai-`. `claude-pet` is not on the list, but a name containing `claude`
  risks getting caught by the impersonation check; a neutral name like `alperen-pets` is safer.
  Source: marketplace-reference (Reserved names)

### 2.3 User installation

```text
claude plugin marketplace add <github-user>/<repo>
claude plugin install <plugin>@<marketplace>
# or inside a session (v2.1.275+):
/plugin install <plugin> --marketplace <github-user>/<repo>
```
- Updating: `claude plugin update <plugin>@<marketplace>` or turning on auto-update.
- Applying to the Anthropic directory for a public catalog is a separate process (claude.ai/directory/manage,
  paid plan). `claude-plugins-official` does not accept submissions.

### 2.4 Other source types (for reference)

`github` (`repo`, `ref`, `sha`), `url`, `git-subdir`, `npm`, `archive` (HTTPS zip + `sha256`,
v2.1.224+), `command` (a command on the user's machine; `link` mode forbidden on Windows).

---

## 3. Skills

Source (unless stated otherwise): https://code.claude.com/docs/en/skills

### 3.1 Frontmatter fields (Claude Code)

- All optional; only `description` is recommended. Unknown fields are **silently ignored**.
- Frontmatter is only read if the file's first line is `---`. If the YAML is broken, the skill loads without fields
  (`/name` works but Claude cannot select it automatically).

| Field | Summary |
| :- | :- |
| `name` | Command name; defaults to the folder name. In a plugin it replaces the last segment, the prefix stays |
| `description` | What it does + when to use it. `description` + `when_to_use` are **truncated at 1,536 characters** in the listing; put the main use first |
| `when_to_use` | Trigger phrases; appended to the description, counted toward the limit |
| `argument-hint` | Autocomplete hint, e.g. `[on\|off\|list\|use <pet>\|install]` |
| `arguments` | Named positional arguments (`$name`) |
| `disable-model-invocation` | `true`: only the user invokes it via `/name`; description is not loaded into context |
| `user-invocable` | `false`: hidden from the `/` menu, only Claude invokes it |
| `allowed-tools` | Tools usable without a permission prompt in the turn the skill is invoked (does not restrict tools) |
| `disallowed-tools` | Removed from the tool pool while the skill is active |
| `model`, `effort` | Model/effort for the turn |
| `context: fork`, `agent`, `background` | Run in a subagent |
| `hooks` | Hooks registered when the skill is invoked (supports `once`) |
| `paths` | Limits automatic activation by glob |
| `shell` | `bash` (default) or `powershell` for `` !`command` `` blocks |
| `metadata`, `license`, `compatibility` (≤500 characters) | Agent Skills spec fields, Claude Code does nothing with them |

- **Portability:** claude.ai upload / the Skills API accept only `name`, `description`, `license`,
  `compatibility`, `metadata`, `allowed-tools`; any other field is a hard error. Skills inside a
  plugin can use every field in Claude Code.
- **Agent Skills spec limits** (https://agentskills.io/specification): `name` 1–64 characters,
  lowercase letters/digits/hyphens, cannot start or end with a hyphen, no `--`, **must match the parent folder name**;
  `description` 1–1024 characters. The Anthropic guide additionally forbids XML tags and the words "claude"/"anthropic"
  (best-practices). Claude Code has a 1,536-character limit, but staying under 1024
  is valid everywhere.

### 3.2 Command name and plugin namespace

- Plugin skill: `/<plugin>:<folder>`; frontmatter `name` replaces the last segment.
  > "`my-plugin/skills/review/SKILL.md` → `/my-plugin:review`, or `/my-plugin:fancy` with `name: fancy`"
- **Also invocable without the prefix:** "The bare `/fancy` also invokes the skill unless another command
  already uses that name." So plugin `desk-pet` + skill `pet` → both `/desk-pet:pet` and `/pet`
  (if there is no conflict).
- If you already write the prefix in the `name` field, v2.1.246+ does not add it again (v2.1.216–245
  doubled it; not writing it is safest).
- A single `SKILL.md` at the plugin root also works, but validate does not check it; prefer `skills/<name>/`.
- Also see: https://code.claude.com/docs/en/plugins/components (Skills)

### 3.3 Arguments

- `$ARGUMENTS` (all), `$ARGUMENTS[N]` / `$N` (0-based, shell-style quoting), `$name`.
- If there are no placeholders at all, `ARGUMENTS: <input>` is appended. `/pet use johnny` → `$0=use`, `$1=johnny`.
- To write a literal `$1.00`, use `\$1.00`.

### 3.4 Variables: in skill text and in scripts

| Variable | Where |
| :- | :- |
| `${CLAUDE_SKILL_DIR}` | The SKILL.md folder (in a plugin, the skill subfolder, not the plugin root) |
| `${CLAUDE_PLUGIN_ROOT}` | The plugin's installed version folder (changes per version; do not write state here) |
| `${CLAUDE_PLUGIN_DATA}` | `~/.claude/plugins/data/<id>/`, preserved across updates |
| `${CLAUDE_PROJECT_DIR}` | Project root (v2.1.196+) |
| `${CLAUDE_SESSION_ID}`, `${CLAUDE_EFFORT}` | Session ID, effort level |

- These variables are substituted as text **into skill text and into Bash rules inside `allowed-tools`**.
  Using the same variable in both lets the script run without a permission prompt:
  ```yaml
  allowed-tools: Bash(${CLAUDE_SKILL_DIR}/scripts/render.sh *)
  ---
  Run `${CLAUDE_SKILL_DIR}/scripts/render.sh <csv-file>` to render the chart.
  ```
- **Important:** The variables are **not** present in the Bash tool's environment:
  > "The variables aren't present in the environment of commands Claude runs through the Bash tool...
  > write the `${...}` reference in the Markdown body instead"
  (manifest-reference, Where each variable resolves). So a script cannot read
  `$env:CLAUDE_PLUGIN_DATA` itself; pass the path as an argument (e.g. `-DataDir "${CLAUDE_PLUGIN_DATA}"`).
  Hook processes, on the other hand, receive `CLAUDE_PLUGIN_ROOT`, `CLAUDE_PLUGIN_DATA`, `CLAUDE_PROJECT_DIR`.
- On Windows, substituted paths come with `/` (`C:/Users/...`). PowerShell and native exes accept this.
- The docs describe `allowed-tools` substitution only for "Bash rules"; whether variables are substituted in a
  `PowerShell(...)` rule is **undocumented** and should be tested.

### 3.5 Dynamic context (`` !`command` ``)

- The command runs before the skill is sent, and its output goes into the text. The working directory is the session's cwd;
  use `${CLAUDE_SKILL_DIR}` for paths. 2-minute timeout. A non-zero exit **cancels the entire skill
  invocation**. Permission check: outside auto mode, a command that is not permitted cancels the invocation;
  pre-approve it with `allowed-tools`.
- With `shell: bash` and no Git Bash, the invocation fails; `shell: powershell` works when the PowerShell tool
  is enabled (enabled by default on Windows without Git Bash).
- Useful for status-display tasks like `/pet list` (e.g. `` !`powershell -NoProfile -File "${CLAUDE_SKILL_DIR}/scripts/list.ps1"` ``);
  but since an error cancels, the script must always exit with 0.

### 3.6 Supporting files

- A skill folder can contain multiple files; link them from SKILL.md so Claude knows when
  to read them. "Keep `SKILL.md` under 500 lines."
- Spec recommendation: `scripts/` (executed), `references/` (read when needed), `assets/`
  (templates, images, data). Source: https://agentskills.io/specification
- For files shared between skills, use `${CLAUDE_PLUGIN_ROOT}/scripts/...`.

### 3.7 Lifecycle (affects design)

- The invoked skill text enters the context as a single message and **stays across turns**; the file is not re-read.
- The `allowed-tools` permission **expires at the next user message**.
- During compaction the first 5,000 tokens of each skill are kept (25,000 total); put important instructions first.
- Source: skills (Skill content lifecycle)

### 3.8 Who invokes

- `disable-model-invocation: true`: for workflows with side effects (deploy, commit). Turning the pet on/off
  and installing the exe have side effects → suitable for `/pet`. Bonus: its description is not loaded into context, costing no tokens.
- For `hatch`, if Claude should pick it on its own for a "draw me a pet" request, leave the default;
  if it should only be invoked manually, `disable-model-invocation: true`.

---

## 4. Anthropic's skill authoring recommendations

Source: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices
(+ https://code.claude.com/docs/en/skills and https://agentskills.io/specification)

- **Be concise.** "The context window is a public good." "Default assumption: Claude is already very
  smart." Write only what Claude does not know. Claude Code docs: "State what to do rather than
  narrating how or why."
- **Length:** SKILL.md body under 500 lines; the spec says "< 5000 tokens recommended".
  Metadata ~100 tokens.
- **Progressive disclosure:** SKILL.md acts like a table of contents; details live in separate files.
  References **one level deep** (SKILL.md → file; not file → another file). Reference files longer than 100 lines
  get a table of contents at the top.
- **Description:** always in the **third person** ("Processes Excel files...", not "I can help");
  write what it does **and** when to use it, with keywords the user would naturally say.
  Avoid vague descriptions ("Helps with documents").
- **Naming:** gerunds recommended (`processing-pdfs`), noun phrases (`pdf-processing`) acceptable;
  vague names like `helper`, `utils`, `tools` and "claude"/"anthropic" are forbidden.
- **Degrees of freedom:** for fragile tasks (file format, pixel atlas), narrow and precise instructions /
  ready-made scripts; for creative tasks, broad freedom.
- **Scripts:** "Solve, don't defer" (handle errors inside the script); no "voodoo constants"
  (every constant justified); state clearly in the instructions whether the script is **to be run or to be read**;
  prefer ready-made scripts for deterministic tasks; list required dependencies explicitly.
- **Verifiable intermediate output / feedback loop:** "run validator → fix errors → repeat".
  For hatch: generate sprite → validator script (size, frame count, transparency) → Claude checks
  the PNG by looking at it ("Use visual analysis").
- **Paths:** always `/` ("Avoid Windows-style paths").
- **No time-sensitive information**; consistent terminology; concrete examples; template pattern.
- **Testing:** at least three evaluations, try with Haiku/Sonnet/Opus. In Claude Code,
  `claude plugin eval` can measure the skill trigger rate (`tool_used: Skill` grader).
  Source: skills (Skill not triggering), https://code.claude.com/docs/en/plugin-evals

---

## 5. Shipping/downloading an exe in the plugin and `CLAUDE_PLUGIN_DATA`

### 5.1 What the docs say

- **There is no dedicated field/mechanism for downloading binaries.** The documented routes:
  1. **Putting the exe in the plugin files.** On install the plugin is copied into
     `cache/<marketplace>/<plugin>/<version>/`, and `${CLAUDE_PLUGIN_ROOT}` points there.
     A new folder per version; the old one is deleted after 14 days. Git LFS does not work. On Windows, if a running exe
     locks the cache folder, the update fails (section 0.4).
     Source: https://code.claude.com/docs/en/plugins/loading (Find plugins on disk, Cleanup of previous versions)
  2. **The `archive` source:** the marketplace entry points to an HTTPS zip (e.g. a GitHub Release asset),
     pinned with `sha256`. Limits: zip ≤256 MiB, 120 s, ≤5 redirects; unpacked ≤1 GiB.
     Version = first 12 characters of the sha256 (or `version`). The user does not need git.
     Source: marketplace-reference (archive plugin source), host-marketplace (Stay within the download limits)
  3. **Doing the install via a hook/skill into `${CLAUDE_PLUGIN_DATA}`.** The example in the docs is a `SessionStart`
     hook that checks for a `package.json` difference and installs `node_modules` into the data folder:
     > "When the automatic install can't provide a dependency, install it from a hook into the persistent data directory."
     Source: https://code.claude.com/docs/en/plugins/components (Install dependencies into the data directory),
     https://code.claude.com/docs/en/plugins/loading (When the dependency install fails or is skipped)
- Automatic dependency installation is only for npm/Bun packages and does not run lifecycle
  scripts; it does not apply to an exe.
- `bin/` (plugin root) is added to Bash `PATH`, after the user's own `PATH`; claude.ai/Cowork
  reject such a plugin. If Claude does not need to call the pet exe by its bare name,
  do not use `bin/`.

### 5.2 `CLAUDE_PLUGIN_DATA` lifetime

Sources: manifest-reference (Environment variables), loading (Find plugins on disk),
cli-reference (What an uninstall deletes and keeps)

- Path: `~/.claude/plugins/data/<id>/`; `<id>` = `plugin@marketplace`, with characters other than letters/digits/`_`/`-`
  turned into `-` (`my-plugin@my-marketplace` → `my-plugin-my-marketplace`). The root
  can be changed with `CLAUDE_CODE_PLUGIN_CACHE_DIR`.
  → **If the plugin or marketplace name changes, the data folder changes too.**
- **Created on first reference** ("created on first reference"), **preserved** across updates.
- **Deleted:** when the plugin is removed from its last scope (together with options and secret values), or when
  the marketplace is removed. Exception: `claude plugin uninstall --keep-data`.
- A development copy loaded with `--plugin-dir` gets the marketplace name `inline` → the data folder
  is probably `<plugin>-inline` (not stated explicitly in the docs, inferred from the `id` rule; should be tested).
- Hook processes get `CLAUDE_PLUGIN_DATA` in their environment; the Bash tool does not (write it into the skill text).

### 5.3 Update behavior

- If a copied plugin is updated mid-session, hooks keep using the old path;
  `/reload-plugins` switches to the new path. Auto-update runs after the first message with a random delay of up to
  10 minutes and shows `Plugin updated: <name> · Run /reload-plugins to apply`.
  Source: loading (When auto-update runs)
- A relative plugin of a marketplace added from a local folder is loaded **in place** (not copied):
  edits take effect in the next session/with `/reload-plugins`, no version bump needed.
  Good for development. Source: loading (In-place and copied plugins)

### 5.4 Recommendation for claude-pet (inference, not from the docs)

- Exe in `${CLAUDE_PLUGIN_DATA}/bin/` (the decision already leans this way). Rationale: preserved across updates,
  no cache-lock problem, a fixed path independent of version.
- Two reasonable ways to get the exe there:
  - (a) Exe on a GitHub Release; the `/pet install` skill (side effects, `disable-model-invocation: true`)
    downloads it with a PowerShell script, verifies sha256, and puts it in `${CLAUDE_PLUGIN_DATA}/bin/`.
    The repo stays small, but since the skill opens a URL it must be weighed **against the tool rule in CLAUDE.md**
    (this is the user's own session, not the pet chat; Alperen should still be asked).
  - (b) Exe is put in the repo/plugin; a `SessionStart` hook or `/pet install` copies the exe from
    `${CLAUDE_PLUGIN_ROOT}` to `${CLAUDE_PLUGIN_DATA}/bin/` if the version differs (the same as the `diff`
    pattern in the docs). No network needed, but the repo grows with the exe on every release.
- In both cases: if the exe is running it must be closed before replacing it; the hook must never block
  (if copying/downloading happens in `SessionStart`, it must be fast or `async: true`).
  Anthropic's `security-guidance` plugin does a similar "SessionStart bootstrap": install into a persistent
  folder, a sentinel file against concurrent installs, a back-off period after failure
  (https://github.com/anthropics/claude-plugins-official/blob/main/plugins/security-guidance/hooks/ensure_agent_sdk.py).
  Note: that plugin writes its data into `~/.claude/security/` instead of `CLAUDE_PLUGIN_DATA`.

---

## 6. Examples: skills in Anthropic repos that generate files / run scripts

### 6.1 `anthropics/skills` → `slack-gif-creator` (generates images, closest example)

https://github.com/anthropics/skills/tree/main/skills/slack-gif-creator

```text
slack-gif-creator/
├── SKILL.md            (~7.8 KB)
├── LICENSE.txt
├── requirements.txt    (pillow, imageio, imageio-ffmpeg, numpy)
└── core/
    ├── gif_builder.py
    ├── frame_composer.py
    ├── easing.py
    └── validators.py
```
- Frontmatter has only `name`, `description`, `license`. Description: what it does + "Use when users request
  animated GIFs for Slack like "make me a GIF of X doing Y for Slack.""
- Body: constraints first (size 128x128, FPS, color count), then a "Core Workflow" code example
  (helper library `GIFBuilder` + frame-by-frame drawing with PIL), drawing principles ("Use thicker lines",
  "Don't use: Emoji fonts"), an API summary of the helpers, a **validator** (`validators.py`).
- Lesson: Claude writes the code itself but calls a ready-made helper library; quality rules and
  validation live in the skill. Directly adaptable for the hatch skill: System.Drawing helpers under `scripts/`
  (create atlas, place frame, save PNG, validate), atlas rules in SKILL.md
  (a summary of `docs/SPRITE.md` under `references/`).

### 6.2 `anthropics/skills` → `algorithmic-art`, `canvas-design`

- `algorithmic-art`: `SKILL.md` + `templates/generator_template.js` + `templates/viewer.html` (template pattern).
- `canvas-design`: `SKILL.md` + `canvas-fonts/` (many .ttf files; an example of carrying large assets inside a skill).

### 6.3 `anthropics/claude-plugins-official` → `session-report` (script + template, inside a plugin)

https://github.com/anthropics/claude-plugins-official/tree/main/plugins/session-report

```text
session-report/
├── LICENSE
└── skills/session-report/
    ├── SKILL.md
    ├── analyze-sessions.mjs
    └── template.html
```
- Note: **there is no `.claude-plugin/plugin.json`** (the manifest is optional; the name comes from the marketplace entry).
- SKILL.md numbered steps: run the script → read the JSON → copy the template → fill it with Edit → report the path.
  For the script path it says `<skill-dir>` (the newer way: `${CLAUDE_SKILL_DIR}`).

### 6.4 `anthropics/claude-plugins-official` → `security-guidance` (plugin with hooks)

- `hooks/hooks.json` + Python scripts; commands in shell form:
  `bash "${CLAUDE_PLUGIN_ROOT}/hooks/sg-python.sh" "${CLAUDE_PLUGIN_ROOT}/hooks/..."`, `"timeout": 180`
  for `SessionStart`, `asyncRewake` for background jobs.

### 6.5 The `anthropics/skills` marketplace

- `.claude-plugin/marketplace.json`, multiple plugin entries, all `"source": "./"`,
  selecting subsets of skills in the same repo via `"strict": false` and a `skills` list.
  (Since the plugin has no `plugin.json`, the entry acts as the manifest.)

---

## 7. Hooks (Windows-specific notes)

Source: https://code.claude.com/docs/en/hooks

- **Exec form** (with `args`): no shell, `command` must be a real exe.
  > "On Windows, exec form requires `command` to resolve to a real executable such as a `.exe`.
  > The `.cmd` and `.bat` shims ... can't be spawned without a shell."
  → Options for a `.cmd` wrapper: `"command": "cmd.exe", "args": ["/d", "/c", "${CLAUDE_PLUGIN_ROOT}/hooks/pet-hook.cmd", "<event>"]`
  (cmd.exe is a real exe) or shell form. Whether a path with `/` works fine with `cmd /c` **should be tested**.
- **Shell form** (no `args`): on Windows, Git Bash, otherwise PowerShell; selected with `"shell": "powershell"`.
  Put the path variable in double quotes; validate warns about unquoted use.
- Path format: manifest-reference says "substituted paths use forward slashes"; troubleshooting, however, says
  exec form and `shell: powershell` keep "native paths". There is a minor contradiction between the docs
  → the script should accept both formats.
- Timeout defaults: command 600 s; `UserPromptSubmit` 30 s; `SessionEnd` 1.5 s total.
  `async: true` does not block (output arrives in the next turn); in a `-p` session it is killed at shutdown, so if the job
  will outlive the session, a fully detached process should be started.
- Plugin hooks do not wait for the skill to be used; they always run once the session loads them (narrow with `matcher`).
- Hook environment: `CLAUDE_PLUGIN_ROOT`, `CLAUDE_PLUGIN_DATA`, `CLAUDE_PROJECT_DIR`, `CLAUDE_PLUGIN_OPTION_*`.

---

## 8. PowerShell notes for the hatch skill

Source: https://code.claude.com/docs/en/tools-reference (PowerShell tool), skills

- PowerShell tool: enabled automatically on Windows without Git Bash; with Git Bash present, enabled by default
  for claude.ai/Console accounts. Claude Code starts PowerShell with process-scoped
  `-ExecutionPolicy Bypass` (except under Group Policy).
- The user may have both the Bash tool and the PowerShell tool. In SKILL.md, give the script as one explicit
  command, e.g. `powershell.exe -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_SKILL_DIR}/scripts/hatch.ps1" -Out ...`
  (works from either tool). For `allowed-tools`, both a `Bash(...)` and a `PowerShell(...)` rule
  can be written; whether `${CLAUDE_SKILL_DIR}` is substituted in the PowerShell rule is undocumented (3.4).
- System.Drawing is built into Windows PowerShell 5.1 (.NET Framework). In PowerShell 7
  (`pwsh`, .NET Core), `System.Drawing.Common` is only supported on Windows; since Claude Code
  prefers `pwsh.exe`, test the script on both versions (this item is general .NET knowledge,
  not verified against docs in this research).

---

## 9. Short decision list (recommendation)

1. Choose a plugin name that does not contain `claude` (e.g. `desk-pet`), and give the display name via `displayName`.
   The marketplace name should be neutral too.
2. Repo root = plugin root = marketplace root; `marketplace.json` single entry `"source": "./"`.
3. Skills: `skills/pet/` (`disable-model-invocation: true`, `argument-hint`, subcommands via `$0/$1`),
   `skills/hatch/` (`scripts/` + `references/`, validator script + visual check loop).
4. No `bin/` at the root; exe in `${CLAUDE_PLUGIN_DATA}/bin/`; how to get it there (Release download or
   copy from the repo) is a separate decision.
5. Version: bump `plugin.json` `version` on every release (or never set it); keep it in only one place.
6. CI: `claude plugin validate --strict .` and `claude plugin validate --strict ./skills`.
7. Hook: `cmd.exe` + `args` exec form or shell form; test both on Windows.
