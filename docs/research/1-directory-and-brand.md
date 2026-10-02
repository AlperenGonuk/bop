# Research 1: Official directory/marketplace, brand rules, plugin-specific risks

Date: 2026-10-02. Official sources only (code.claude.com, claude.com/docs, support.claude.com, anthropic.com, github.com/anthropics). Information from unofficial sources is explicitly marked "UNOFFICIAL". Quotes are taken verbatim from the pages (in English).

---

## 0. Summary (for the decision maker)

1. **The name "claude-pet" (the project's former name) is a blocker.** Claude Code's own validator treats a plugin name starting with `claude-` as an **error** ("reserved: it passes as one of Anthropic's own"); `claude plugin init` / `claude plugin tag` refuse this name. The directory portal also requires the name to be built around your own distinctive product name and not look official. In addition, there are at least 8 projects named "claude-pet" on GitHub (UNOFFICIAL source) → it may also trip the portal's "Name is taken / may be confused" checks. **The name must change** (e.g. `<distinctive-name>`; descriptive use such as "for Claude Code" inside `displayName` is a separate matter, see §2).
2. **There is only one submission path: the claude.ai developer portal** (`claude.ai/directory/manage`). A paid plan (Pro/Max/Team/Enterprise) is required. `anthropics/claude-plugins-official` does not accept individual submissions (partner contacts only); a PR opened against the `anthropics/claude-plugins-community` repo is closed automatically.
3. **Downloading an exe from GitHub Releases on first run is not explicitly forbidden**, but: a compiled exe placed in the plugin folder triggers a "reviewer hold"; downloads/package installs inside a script run by a hook, and scripts the validator can't follow, trigger a "hold"; the security scan looks for "hidden code execution / sending data to an undisclosed destination". Clearly documenting everything downloaded in the README is required practice. Conclusion: it will most likely go to human review; rejection is not certain, but neither is acceptance.
4. **No official ban on Windows-only** support was found. However, a directory listing ships to chat/Cowork/Claude Code together; hooks are also loaded in Cowork. File names must be valid on Windows+macOS.
5. **A README (≥40 words) and a LICENSE (or the `license` field) are mandatory**, otherwise the submission is blocked. MIT is accepted (SPDX).
6. **Logo/Clawd**: Anthropic names/logos may not be used in a product name, in your own logo, or in a way that implies endorsement/partnership; altering the logo is forbidden; other uses require written permission. No separate official rule for Clawd was **found** — it should be treated as falling under Anthropic's brand/artwork.

---

## 1. Official directory / marketplace and submission process

### 1.1 Anthropic's three marketplaces + a separate "directory"

Source: https://code.claude.com/docs/en/plugins/anthropic-marketplaces

| | Official | Community | Demo |
|---|---|---|---|
| Repo | `anthropics/claude-plugins-official` | `anthropics/claude-plugins-community` | `anthropics/claude-code` |
| Marketplace name | `claude-plugins-official` | `claude-community` | `claude-code-plugins` |
| Content | "Plugins Anthropic maintains, plus plugins from partners and other authors" | "Third-party plugins that their authors submitted to Anthropic" | example plugins |
| How it is added | Claude Code adds it itself in the first interactive session | `/plugin marketplace add anthropics/claude-plugins-community` | `/plugin marketplace add anthropics/claude-code` |

- "Anthropic's directory is separate from these marketplaces. The directory is the catalog on claude.ai." (same page)
- Catalog on the web: https://claude.com/marketplace/plugins ("shows install counts and marks some plugins **Anthropic verified**").
- For third-party marketplaces: "Anthropic doesn't review third-party marketplaces".

### 1.2 The official marketplace (`claude-plugins-official`) does not accept submissions

Source: https://code.claude.com/docs/en/plugins/publish
> "Anthropic's official marketplace, `claude-plugins-official`, doesn't take submissions through the directory portal. If you work with an Anthropic partner contact, ask them about an official-marketplace listing."

Repo README (https://github.com/anthropics/claude-plugins-official): "Third-party partners can submit plugins for inclusion in the marketplace. External plugins must meet quality and security standards for approval." The form link is `https://clau.de/plugin-directory-submission`, which redirects with a 302 to `https://claude.com/docs/directory/publish` (the directory portal). The README also says: "The `name` field in a marketplace entry is an immutable slug."

→ For an individual developer the realistic target is the **directory**; the official marketplace is a partner channel.

### 1.3 Community marketplace (`claude-community`)

Source: https://github.com/anthropics/claude-plugins-community
- "read-only mirror ... synced nightly from Anthropic's internal review pipeline."
- "Every plugin listed has been submitted via claude.ai, passed automated security scanning, and been approved for distribution."
- Submission: `clau.de/plugin-directory-submission` (→ directory portal).
- "Pull requests opened directly against this repository are closed automatically."
- According to the security page: "Where the `claude-community` catalog pins a plugin to a commit SHA, which it does for nearly every entry, Claude Code refuses to install a different commit." (https://code.claude.com/docs/en/plugins/security)
- Auto-update is **off by default** for the community marketplace (https://code.claude.com/docs/en/discover-plugins).

Inference (not stated directly in the official text): plugins approved through the directory portal appear to also show up in the `claude-community` catalog on the Claude Code side; the exact mapping was **not found** in the docs.

### 1.4 Submitting to the directory: who, where, how

Sources: https://claude.com/docs/directory/publish , https://claude.com/docs/plugins/submit

- Portal: https://claude.ai/directory/manage → **Submit new** → **Plugin bundle**.
- "Anyone on a paid Claude plan can submit, there's no partner program to apply to first, and Anthropic checks each submission before it's listed."
- Plan: "Pro, Max, Team, or Enterprise. Free accounts can't submit". On Team/Enterprise, the Owner (or a role with the Directory permission).
- The source must be a GitHub repo; "The repository can stay private while you validate and submit, and must be public before the listing goes live." The GitHub account must be connected to claude.ai and have push access to the repo.
- Steps: Source (repo, plugin path, branch/tag) → **Validate** → Listing details (read from plugin.json + README) → **Data handling** questions ("whether the plugin reads or stores personal data, whether it sends data to services other than its declared connectors, how long it keeps data, and whether it's intended for people under 18") → **Compliance** (contact email + four checkboxes) → Submit for review.
- Limits: 10 submissions per organization per 24 hours; one submission per repo+folder; "the first organization to submit a given repository folder holds that listing".
- Updates: merge to the tracked branch/tag → the directory scans the new commit → it is published according to the publishing setting. Default: "An Anthropic reviewer publishes each version".
- Review time: "Review time isn't fixed." (https://claude.com/docs/directory/submission-status)
- Contact: **Get help / Contact Anthropic** in the portal, **Appeal this decision** for a rejection; email `directory@anthropic.com`.
- There is no separate application for the "Verified" badge; Anthropic decides during review.

### 1.5 Acceptance criteria and review rules

All listings are subject to:
- Anthropic Software Directory Terms: https://support.claude.com/en/articles/13145338-anthropic-software-directory-terms
- Anthropic Software Directory Policy: https://support.claude.com/en/articles/13145358-anthropic-software-directory-policy

Review mechanism (https://claude.com/docs/directory/publish):
> "Plugin bundles: every version gets automated validation and a security scan, and a person reviews a new listing before it goes live."

Finding types (https://claude.com/docs/plugins/pre-submission-checklist): **Blocks** (cannot submit), **Policy hold / Held for a reviewer** (can submit, a human reads it; "A hold isn't a rejection"), **Warning**, **Note**.

Security scan:
> "The security scan looks for behavior that a plugin doesn't disclose, such as sending data elsewhere, running hidden code, or changing Claude's permission settings."
> "A first submission that fails the security scan is rejected"
> "Describe in the README everything the plugin runs, sends, or fetches. A complete README doesn't make a behavior allowed."
> "Commit readable source instead of compiled, packed, or minified code. Code that the security scan can't read is held for a reviewer"

Checklist items that matter for this plugin (all from the same page):

| Rule | Result |
|---|---|
| `.claude-plugin/plugin.json` in the plugin folder | Blocks if missing |
| Every file a hook/script uses is inside the plugin folder | A path pointing outside Blocks |
| No `.DS_Store`, `Thumbs.db`, `desktop.ini`, `__MACOSX` | Blocks |
| File names valid on Windows **and** macOS; no names that collide when case is ignored | Validation stops |
| Repo < 50 MiB as a GitHub archive, < 256 MiB unpacked, < 10,000 files; every file in the plugin folder < 5 MiB | Validation stops |
| Every non-image/non-font file < 256 KiB; plugin ≤ 512 files | Held |
| Only text, SVG, PNG/JPEG/GIF/WebP, fonts. "Any other binary file, such as an `.ico`, `.pdf`, or `.zip` file or a compiled executable, is held." | Held |
| "Don't refer to bundled images or fonts from commands, hooks, or scripts, or write their paths in backticks or a code block." | Held |
| Paths in hook commands must be written in full with `${CLAUDE_PLUGIN_ROOT}`; no other variables/command substitution/wildcards/inline programs | Blocks if the plugin is in a repo subfolder |
| "Keep launchers and package installs out of each script that a hook or an MCP server runs." | Held ("Scripts the validator couldn't follow") |
| If the plugin is in a repo subfolder: a hook running a non-shell file, or a shell script calling another file | Held |
| `hooks/hooks.json` is valid JSON, only documented events/types | Blocks |
| No real credentials in files | Blocks |
| README ≥ 40 words (code blocks don't count) | Blocks |
| `LICENSE` file or `license` field | Blocks |
| `description`, `author`, `version` | Warning |

Relevant sentences from the Policy (Directory Policy):
- "Developers must provide a clear, accessible privacy policy link explaining data collection, usage, and retention"
- "Developers must provide verified contact information and support channels for users with product or security concerns"
- "Developers must document how their Software works, its intended purpose"
- "Software must only collect data from the user's context that is necessary to perform their function." (1.D)
- "Instructional Software must not intentionally call or coerce Claude into calling other external software, tools, databases, or resources unless requested and intended by a user." (2.D)
- Prohibited uses: money/crypto transactions, standalone AI image/video/audio generation ("design tools permitted"), advertising/sponsored content.

From the Terms: the developer must "implement mechanisms for receiving and investigating security vulnerability reports"; Anthropic "may remove or refuse to display any Software ... at any time for any reason".

Fields in `plugin.json` that the directory reads: `icon`, `documentationUrl`, `supportUrl`, `privacyPolicyUrl`, `termsOfServiceUrl` (https://code.claude.com/docs/en/plugins/manifest-reference#directory-listing-fields). A privacy policy link must be provided per the Policy.

Local pre-check: `claude plugin validate --strict ./plugin` (https://code.claude.com/docs/en/plugins/publish). Note: "The portal applies additional directory rules that the CLI doesn't check".

---

## 2. Brand guidelines

### 2.1 "claude" in the plugin name

**Claude Code validator** (https://code.claude.com/docs/en/plugins/manifest-reference#name):

| Name | Result |
|---|---|
| Starts with `claude-`, `anthropic-`, `anthropics-`, `cc-plugin-` | **Error** |
| Exactly `claude`, `anthropic`, `anthropics`, `claude-code`, `claude-mods` | Error |
| `official` next to `claude`/`anthropic` | Error |
| `claude`/`anthropic` as a whole word elsewhere (e.g. `mcp-for-claude`) | Warning |

> "The error reads `Plugin name "<name>" is reserved: it passes as one of Anthropic's own` ... `claude plugin init` and `claude plugin tag` refuse a name that draws the error. Only these commands check the name. Claude Code still installs and loads a plugin whose name they refuse."

→ **`claude-pet` = Error.** It technically installs from your own marketplace, but `--strict` validation fails and it is a serious obstacle for the directory.

**Directory portal** (https://claude.com/docs/plugins/pre-submission-checklist, "Manifest and plugin name"):
> "Build the name around your own distinctive product or project name: not a reserved word such as `claude`, `anthropic`, `official`, `plugin`, `mcp`, or `test` as the whole name, not a marketplace name reserved for Anthropic, and nothing that presents the plugin as official" → **Blocks** ("Name is taken"); a name made only of generic words is Held.
> "Choose a name that no other organization's plugin uses. A name that differs only in capitalization or punctuation counts as the same name." → the same name Blocks, a similar one is Held.
> "Choose a name, `displayName`, and `author.name` that can't be mistaken for an existing plugin, publisher, connector, or well-known brand that isn't yours" → Held ("Name matches a known brand").

The portal text does not explicitly list `claude-pet` as "Blocks" (the forbidden example is "`claude` as the whole name"), but taking the "well-known brand that isn't yours" and "presents the plugin as official" items together with the CLI's `claude-` prefix error, the name would most likely be at least **Held** and probably rejected. `displayName` is also subject to the brand check; a displayName like "Claude Pet" carries the same risk.

The name is permanent: "Never change a published plugin's `name`" (https://code.claude.com/docs/en/plugins/publish). → The name must be chosen **before the first release**.

Name collisions (UNOFFICIAL source, for information only): there are many projects named "claude-pet" on GitHub, e.g. https://github.com/daghlny/claude-pet , https://github.com/IMMINJU/claude-pet (Tauri 2), https://github.com/Youl-AI/claude-pet (Windows, "desktop pet plugin for Claude Code"), https://github.com/xtrimsystems/claude-pet , https://github.com/HaneulOscarLee/claude-pet , https://github.com/scm1400/claude-pet . If any of these has submitted to the directory, the portal will say "Name is taken".

### 2.2 General brand/logo rules

Claude Code Legal and compliance (https://code.claude.com/docs/en/legal-and-compliance):
> "You can accurately say, in plain text, that your product has Claude Code preinstalled or that it runs Claude Code. But you can't use the Claude Code or Anthropic names or logos as part of your own product, feature, or company name, in your own logo, or in a way that suggests Anthropic built, endorses, or is partnered with your product. Any other use of Anthropic's names or logos is governed by our Trademark Guidelines and requires our written permission."

(This paragraph appears in the section on "offering Claude Code in your product", but it can be read as a general brand principle.) → Saying "a desktop pet for Claude Code" in plain text in the README is fine; making it part of the product name is not.

Anthropic Trademark Guidelines (https://www.anthropic.com/legal/trademark-guidelines):
- "You may only use our trademarks as specifically permitted by us and only in materials we approve beforehand."
- "You may not use our trademarks in a manner that implies Anthropic's sponsorship or endorsement, or a relationship or affiliation with Anthropic, except as we expressly authorize."
- "No alterations of our trademarks (changes to color, font, proportion, or otherwise) are permitted."
- "You may not use our trademarks in any manner that could diminish or damage the reputation of Anthropic or our goods or services."
- Permission: "please email marketing@anthropic.com" (for those with an existing business relationship).

Directory Terms (https://support.claude.com/en/articles/13145338-anthropic-software-directory-terms):
> "You will not make any statement regarding the Anthropic Services which suggests partnership with, sponsorship by, or endorsement by Anthropic without Anthropic's prior written approval" ; compliance with the Trademark Guidelines is required. "Inclusion of your Software in one or more Directories does not create any partnership..."

### 2.3 Clawd / mascot

- No separate official usage rule for Clawd was **found**. The official repo only has a documentation-request issue: https://github.com/anthropics/claude-code/issues/8536 (contains no rules).
- Inference: since Clawd is Anthropic's mascot artwork, it should be treated as covered by the Trademark Guidelines rules "only as permitted" and "no alterations". Distributing Clawd (or the Anthropic logo, the orange "spark" mark) as a pet sprite without written permission is risky. There are many projects on GitHub using Clawd with an "Unofficial fan project" notice (UNOFFICIAL, e.g. https://github.com/t1anhe/clawde); these do not show that permission exists.
- Additional directory risk: the portal rule "Don't refer to bundled images ... from commands, hooks, or scripts" (Held). If sprite PNGs are referenced from a hook/skill, that is a hold.

### 2.4 Requirement for a "not official" notice

- **No mandatory "unofficial / not affiliated" notice requirement was found** in official sources. The rule is built around **not implying** endorsement/partnership (quotes above). Putting the sentence "Not affiliated with or endorsed by Anthropic" in the README is good practice that supports compliance with this rule (inference, not a requirement).

---

## 3. Plugin-specific risks

### (a) Downloading an exe from GitHub Releases on first run

- **No explicit provision was found** in the Directory Policy about downloading code/binaries at runtime, auto-update, or background processes.
- Putting an exe in the plugin folder: "a compiled executable, is held" (checklist). There is also the per-file < 5 MiB limit (Validation stops if exceeded) and < 256 KiB (Held if exceeded). A Tauri exe exceeds these → putting the exe in the repo is not practical.
- Downloading in a script run by a hook: the checklist only sets an explicit rule for "launchers and package installs" (npx, uvx, pip, etc.) (Held). Downloading an exe with `curl`/`Invoke-WebRequest` is not specifically named; but the security scan looks for "running hidden code" and "sending data elsewhere" and holds code it can't read. Running a non-shell file, such as a `.cmd` wrapper + exe, falls under the "Scripts the validator couldn't follow" hold if the plugin is in a repo **subfolder**; if the plugin is at the repo root this item does not seem to apply ("To avoid the hold, keep the plugin at the root of its own repository").
- One of the report headings is "Files **or downloads** the validator couldn't inspect" → the validator also tries to inspect downloads; downloads whose content it can't see are held.
- Among marketplace source types there is `archive` (zip pinned by sha256); for the `npm` source, "A tarball link on github.com ... refused even when the link is a GitHub release download" (https://code.claude.com/docs/en/plugins/marketplace-reference). This applies to the plugin files themselves, not to runtime downloads, but it shows that Anthropic is cautious about GitHub release tarballs.
- User-side security: "A Claude Code plugin you install can execute arbitrary code on your machine with your user privileges." and "Claude Code runs hooks, MCP servers ... outside the sandbox." (https://code.claude.com/docs/en/plugins/security)
- Practical outcome: the README must clearly document the download URL, version, sha256 verification, and where it is written; the download script must be readable source; most likely human review (hold). No definitive accept/reject rule was **found**.

### (b) Async hooks attached to 10 events

Source: https://code.claude.com/docs/en/hooks
- `async: true`: "Claude Code doesn't wait for completion or read the hook's output"; "Exit code and stdout are ignored"; "The `timeout` field is **not enforced** on async hooks".
- Default timeout is 600 s (command hooks); `SessionEnd` hooks share a total budget of 1.5 s.
- "Plugin hooks: always run when plugin is enabled".
- Windows: the `shell` field defaults to `bash` (Git Bash), otherwise `powershell`. Exec form (`args`) requires a real `.exe` on Windows; "`.cmd` and `.bat` shims cannot run in exec form without wrapping them with `node`" → the `.cmd` wrapper must be called with shell form.
- Directory: `hooks/hooks.json` must contain only documented events and types (Blocks); paths via `${CLAUDE_PLUGIN_ROOT}`.
- No limit on the number of events was **found**. Since async hooks have no timeout, stuck processes can pile up (inference) → the hook must exit quickly (already a CLAUDE.md rule).
- Hooks are also loaded in Cowork (https://claude.com/docs/plugins/platform-support) → for a Cowork user the hook runs but the pet may not exist; a silent exit matters.

### (c) Running `claude -p` in the background

- **No plugin-specific provision was found** in the Directory Policy about "invoking Claude itself" or token consumption (the token frugality rule is for MCP servers: "MCP servers must be frugal with their use of tokens").
- Policy 2.D: "must not intentionally call or coerce Claude into calling other external software ... unless requested and intended by a user." → a chat must only start at the user's request.
- Authentication (https://code.claude.com/docs/en/legal-and-compliance): OAuth is "designed to support ordinary use of Claude Code and other native Anthropic applications"; "Anthropic does not permit third-party developers to offer Claude.ai login into their own applications, or to route requests through Free, Pro, or Max plan credentials on behalf of their users. Moreover, developers may not collect, store, or intermediate Claude.ai credentials or session tokens". On the other hand: "Nor does it prevent an end user from signing in to the unmodified Claude Code binary with their own Claude subscription".
  → According to this text, the pet calling the user's own installed, unmodified `claude` CLI with the user's own session appears to be allowed (inference); the pet must not read/store/forward credentials.
- "Advertised usage limits for Pro and Max plans assume ordinary, individual usage of Claude Code and the Agent SDK." → background calls consume the user's quota; this must be stated clearly in the README and in the Data handling answers.
- Checklist: "Don't read a credential that is already set in the user's environment ... and send it to a server" → Held/Blocks. The pet must not read tokens from environment variables.

### (d) Supporting Windows only

- No official "multi-platform requirement" was **found**.
- Constraints: file names must be valid on Windows **and** macOS (Validation stops). A directory listing ships to chat/Cowork/Claude Code together; the portal derives the supported surfaces from the components ("the portal derives the surfaces it supports from these same rules") — no field for choosing an operating system was **found**. So the README must clearly say "Windows only" and hooks must exit silently on macOS/Linux (inference).
- A top-level `bin/` folder blocks chat/Cowork installation entirely ("A top-level `bin/` directory stops claude.ai and Cowork from installing the plugin at all") → must not be used.

### (e) License (MIT) and README requirements

- License: "Add a `LICENSE` file to the plugin folder, or set `license` in `plugin.json`", otherwise Blocks. The `license` field is a "SPDX identifier such as `MIT` or `Apache-2.0`" (manifest reference). There is no rule against MIT; the official example manifest uses `"license": "MIT"` (https://claude.com/docs/plugins/build).
- README: in the plugin folder, ≥ 40 words, code blocks don't count; "say what the plugin does, how to use it, and what data it sends" (https://claude.com/docs/plugins/build). Shown as the description in the directory listing.
- Also from the Policy: a privacy policy link, a verified contact/support channel, documentation of the functionality; from the Terms: a vulnerability reporting mechanism (e.g. SECURITY.md / GitHub security advisories — inference).
- `plugin.json`: `description`, `author`, `version` (Warning if missing); `homepage` must be a valid URL (otherwise the plugin does not load).

---

## 4. Community marketplaces

Official (Anthropic):
- `anthropics/claude-plugins-community` (`claude-community`): submissions only through the directory portal, PRs are not accepted. https://github.com/anthropics/claude-plugins-community
- Topic-based Anthropic marketplaces: `anthropics/skills`, `anthropics/knowledge-work-plugins` (https://code.claude.com/docs/en/plugins/anthropic-marketplaces). No external submission path to these was **found**.

Your own marketplace (official docs, no form): adding `.claude-plugin/marketplace.json` to the repo means publishing. "Once the file is in the repository, the plugin is published, with no submission form." User: `claude plugin marketplace add <owner>/<repo>` + `claude plugin install <name>@<marketplace>`. Reserved marketplace names (e.g. `claude-plugins-official`, `claude-community`, and imitations like `official-claude-plugins`) cannot be used. https://code.claude.com/docs/en/plugins/publish , https://code.claude.com/docs/en/plugins/marketplace-reference#reserved-names

Third-party community marketplaces (UNOFFICIAL sources; Anthropic does not review them — "Anthropic doesn't review third-party marketplaces"):
- composio-community/awesome-claude-plugins — https://github.com/composio-community/awesome-claude-plugins (curated list + marketplace.json; submission format most likely a PR, not verified)
- hekmon8/awesome-claude-code-plugins — https://github.com/hekmon8/awesome-claude-code-plugins
- claudemarketplaces.com — https://claudemarketplaces.com/marketplaces (a directory of marketplaces)
- Submission formats were **not found** in official sources; check each repo's CONTRIBUTING file.

---

## 5. Not found

- Explicit provisions in the Directory Policy on runtime binary downloads, auto-update, background processes, invoking Claude, or operating system support.
- A mandatory "unofficial / not affiliated" notice requirement.
- A specific usage rule for the Clawd mascot.
- Review time (only "isn't fixed").
- How exactly directory approval is reflected in the `claude-community` catalog.
- Official submission rules for third-party marketplaces.
