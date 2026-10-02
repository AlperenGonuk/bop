# Spritesheet reference

Codex pet format. There is no official documentation page; the definition is the contract in
OpenAI's `hatch-pet` skill (`codex-pet-contract.md`, `animation-rows.md`, `validate_atlas.py`).
Decisions: [DECISIONS.md](DECISIONS.md), decision 3. Comparison:
[research/3-hatch-design.md](research/3-hatch-design.md) 2.5.

## `pet.json`

```json
{
  "id": "pitir",
  "displayName": "Pitir",
  "description": "A code-drawn mandarin sprout, the default Bop pet.",
  "spriteVersionNumber": 2,
  "spritesheetPath": "spritesheet.png"
}
```

- The app reads only these 5 fields; it determines the version from the image size.
- `spriteVersionNumber: 2` is **required** in Codex: without it Codex treats the pet as v1 and
  rejects the 2288-pixel-tall sheet. `bop hatch` always writes it.
- Image is PNG or WebP, transparent RGBA.

## Versions (inferred from the image size)

| Version | Size | Grid | Look rows |
|---|---|---|---|
| v1 | 1536 × 1872 | 8 × 9 | none, following the mouse silently off |
| v2 | 1536 × 2288 | 8 × 11 | rows 9–10 + (0,6) neutral cell |

- Cell: 192 × 208 pixels, transparent RGBA
- Frame position: `x = column * 192`, `y = row * 208`
- Unused cells are fully transparent; transparent pixels must also have RGB 0 (the Codex
  validator reports a "transparent RGB residue" error).
- The app finds the frame count by skipping empty cells (alpha channel entirely 0).

| Row | State | Frames | Codex frame durations (ms) | Meaning |
|---|---|---|---|---|
| 0 | idle | 6 (+ (0,6) neutral in v2) | 280, 110, 110, 140, 140, 320 | breathing, blinking; must not be static |
| 1 | running-right | 8 | 120 ×7, last 220 | running right |
| 2 | running-left | 8 | 120 ×7, last 220 | running left |
| 3 | waving | 4 | 140 ×3, last 280 | start, raised hand, return |
| 4 | jumping | 5 | 140 ×4, last 280 | crouch, takeoff, peak, landing, settle |
| 5 | failed | 8 | 140 ×7, last 240 | sad/deflated |
| 6 | waiting | 6 | 150 ×5, last 260 | eager pose waiting for approval/input |
| 7 | running | 6 | 120 ×5, last 220 | work/processing (not running on feet; e.g. laptop) |
| 8 | review | 6 | 150 ×5, last 280 | focused review |
| 9 | look 0°–157.5° (v2 only) | 8 | – | look directions |
| 10 | look 180°–337.5° (v2 only) | 8 | – | look directions |

- Frame durations are informational; the app uses a single `frameMs` (`bop.json`, below).
- Johnny and clawd-fan frame counts were verified in the app (2026-09-30): idle has 7 filled
  cells (6 + neutral). Johnny's files: `dev-pets/johnny/` (local only, not in the repository).

### (0,6) neutral cell (v2)

In v2, row 0, column 6 **must be filled**: the front/neutral pose (Codex `neutralLookFrame`, at
least 50 pixels). It is the pose shown while the cursor is in the dead zone; it is not an idle
frame. Suggested content: the same as the first idle frame. The app (sprites.js) removes this cell
from the idle loop in v2 (`neutralFrame`); this pose is shown while the cursor is in the dead zone
and at the start/end of the idle "look around" move (idle 0 in v1 or when the cell is missing).
Look around: neutral → right → left → short up → neutral, front half circle only.

### Mirroring and row rules

- `running-left` may be a mirror of `running-right`, but only if identity and handedness (which
  hand holds the prop) are not broken; **frame order is preserved** (each frame is mirrored in
  place). `bop hatch` does not mirror; it redraws facing left (the light stays top-left).
- Effects must touch the pet (no detached stars, dots, question marks, thought bubbles, speed
  lines or shadows); `waiting`, `running`, `review` and `failed` must be distinguishable from each
  other.
- Do not fake looking by rotating the whole sprite: the eyes lead, the head follows, the upper body
  follows slightly, the feet stay fixed; baseline and scale are the same as neutral.

## Look rows (v2)

16 directions, 22.5° steps, clockwise; row 9 = 0°–157.5°, row 10 = 180°–337.5°.
**0° up, 90° right, 180° down, 270° left** (Codex contract: 180° is looking down, not turning
around). Older pets may draw 180° differently (Johnny shows a back view). This is why following the
mouse in the app uses only the front half circle (−90°…+90°); when the cursor is below, the pet
looks sideways or forward.

`bop hatch` derives looking from ratios: pupil 1.0, head ~0.6, body ~0.25 (pseudo-3D rotation),
feet fixed. Direction is verified by measuring coordinates.

## `bop.json` (optional)

```json
{
  "formatVersion": 1,
  "frameMs": 140,
  "sheets": { "extra": "extra.webp" },
  "animations": {
    "yawn":    { "sheet": "extra", "row": 0, "frameMs": 120 },
    "monocle": { "sheet": "extra", "row": 1 },
    "slow-idle": { "row": 0, "frameMs": 250 }
  },
  "states": { "thinking": "monocle", "writing-code": "yawn" },
  "idleExtras": ["yawn", "look"]
}
```

- `frameMs`: default frame duration for all animations (30–2000 ms; 150 if missing).
- `sheets`: extra sheets. Same grid: cell 192 × 208, at most 8 columns, any number of rows.
  `main` is a reserved name (the main sheet). The path must be inside the pet folder.
- `animations`: named rows. Without `sheet`, the main sheet is used. Can override Codex names
  (`idle`, `running` …).
- `states`: state → animation. States: `idle`, `running`, `thinking`, `reading`, `writing-code`,
  `running-command`, `waiting-permission`, `done`, `failed`. An undefined state, or one without an
  animation, falls back to its parent (`reading`/`writing-code`/`running-command`/`thinking` →
  `running` → `idle`). Default (Codex): `thinking`→`review`, `running`→`running`,
  `waiting-permission`→`waiting`, `done`→`jumping`, `failed`→`failed`.
- `idleExtras`: short moves played every 15–30 s while idle. `look` = look around (v2 only).
  Default when missing: `["look"]` in v2, none in v1.
- Invalid entries (missing file, grid mismatch, missing row, unknown state/name) are skipped with a
  warning; the pet does not break. In a v1 pet `look` is silently dropped.
