---
name: hatch
description: Creates a new pet for the Bop desktop pet app by describing it as a JSON spec and letting the Bop app draw every animation frame with code (no image generation). Use when the user wants to make, create, design, draw or hatch a new Bop pet or mascot, for example "Let's make a new Bop pet!", "make me a blue cat pet", or "hatch a ghost for Bop".
allowed-tools: Bash(sh "${CLAUDE_PLUGIN_ROOT}/scripts/bop.sh" hatch *), Bash(sh "${CLAUDE_PLUGIN_ROOT}/scripts/bop.sh" install *), Bash(sh "${CLAUDE_PLUGIN_ROOT}/scripts/bop.sh" use *), PowerShell(powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/bop.ps1" hatch *), PowerShell(powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/bop.ps1" install *), PowerShell(powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/bop.ps1" use *)
---

# Hatch a Bop pet

You design the pet as data (`spec.json`); the Bop app draws all 11 animation rows and 16 look
directions from it, validates the atlas and renders a contact sheet you can look at. You never
draw pixels by hand except through the spec's pixel-map escape hatch.

## The app

`<app>` below stands for the plugin's script that runs the Bop app:

- Bash tool (macOS, Linux, Windows with Git Bash): `sh "${CLAUDE_PLUGIN_ROOT}/scripts/bop.sh"`
- PowerShell tool (Windows without Git Bash): `powershell -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_PLUGIN_ROOT}/scripts/bop.ps1"`

Use the Bash tool when it is available. Write the prefix exactly as above, then the arguments.

Run each app command on its own: no `cd`, no existence check, no pipes or `;`, so it matches
the permissions this skill grants. If the script reports that the Bop app is not installed,
show its message to the user as it is, then stop. Do not try to download, build or find the app
any other way.

Commands you use:

| Command | What it does |
| --- | --- |
| `<app> hatch --example [pitir\|critter\|floaty\|custom]` | print an example spec |
| `<app> hatch --docs <spec-format\|qa-rubric\|animation-rows>` | print a reference |
| `<app> hatch "<id>/spec.json" "<id>"` | draw the pet into folder `<id>` |
| `<app> install "<id>"` | install the finished pet into Bop |
| `<app> use <id>` | switch the desktop pet to it (only if the user agrees) |

`hatch` exits with 0 when validation passes, 2 when the atlas has errors (files are still
written so you can look), 1 when the spec is invalid (the message says what to fix).

## Workflow

Progress checklist (keep it short in your replies):

1. Understand the pet
2. Write the spec
3. Hatch and review
4. Install

### 1. Understand the pet

- Simple request ("a blue cat", "a mandarin"): ask nothing; choose sensible defaults.
- Vague request ("Let's make a new Bop pet!"): ask at most one or two short questions in one
  message, for example "What kind of creature or object, and any colors or accessories?" Offer
  two or three quick ideas. If the user says "surprise me", pick one yourself.
- Never ask about grid size, motion values or other technical settings.
- Real people, brand mascots or copyrighted characters: do not imitate them. Draw an original
  pet inspired by the general idea (for example "a yellow electric mouse" becomes your own
  round yellow critter with different ears and markings) and say so in one sentence.

### 2. Write the spec

Pick an `id` (lowercase letters, digits, `-`, `_`; not `pitir`, which is built in) and a
display name. Write `<id>/spec.json` in the current working folder with the Write tool.

Start from the closest level and add detail only when the request needs it:

- **Level a, archetype + palette + options.** Enough for most requests.
  `blob` (round body, optional gesture arms; mandarins, slimes, beans), `critter` (big head on a
  small body, ears, tail, arms; cats, dogs, bunnies, bears, foxes, mice), `floaty` (no feet, wavy
  hem; ghosts, clouds, spirits).
- **Level b, parts.** Change archetype parts by `name` (size, color, position) or add new ones
  (hats, horns, wings, markings) with shapes `ellipse`, `rect`, `triangle`, `polygon`, `capsule`.
- **Level c, pixel maps.** A `pixels` part (rows of palette letters) for small details no shape
  can make, or `overrides` to paint over one specific frame.

Full field reference, coordinates and examples: run `hatch --docs spec-format`. Read it before
writing your first spec.
Run `hatch --example <name>` when you want a working starting point.

Keep the design readable at 192 x 208 pixels: a clear silhouette, 3 to 6 main colors, a
contrasting outline, eyes large enough to read. Accessories should be few and chunky.

### 3. Hatch and review

1. Run `<app> hatch "<id>/spec.json" "<id>"`.
2. If it exits with 1, fix the spec field named in the message and run again.
3. Read `<id>/contact-sheet.png` with the Read tool and review it against
   `hatch --docs qa-rubric`. Red cell borders are errors, orange are
   warnings; the printed report says why. What each row should show:
   `hatch --docs animation-rows`.
4. Fix the smallest thing that solves the problem (one part, one color, one size), then hatch
   again. Stop after at most 3 fix rounds; if the same problem comes back twice, change approach
   (simplify the part, pick another archetype option, or use a pixel map).

Errors must be zero before installing. Warnings are acceptable when you have looked at the cell
and it reads fine (for example a foot briefly lifting off during a run).

Optionally show the user the contact sheet path and a one-line description of the pet before
installing; do not wait for approval unless they asked to approve the look.

### 4. Install

1. Run `<app> install "<id>"`. It copies the folder into Bop (`pet.json`, `spritesheet.png`,
   plus the spec and QA files, which Bop ignores).
2. Ask: "Shall I switch to <name> now?" Run `<app> use <id>` only if the user says yes. A
   running pet changes at once.
3. Tell the user they can switch pets later from the pet's right-click menu or with `/bop use <id>`.

## Rules

- Only run the commands listed above; do not install packages, write scripts or edit other files.
- The spec is the only source of the pet. To change the pet later, edit `<id>/spec.json` and
  hatch again.
- The output also works as a Codex pet: v2 atlas, `spriteVersionNumber: 2`, neutral cell (0,6).
- Answer in the user's language; keep replies short.
