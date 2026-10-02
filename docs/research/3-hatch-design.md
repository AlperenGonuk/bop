# 3. Code-drawn pet skill: license, what to take from Codex, design

Date: 2026-10-02. Reading and design only; no code was written, the repo was not touched.

Examined:
- `~\.codex\skills\hatch-pet\` (SKILL.md 924 lines, references/ 3 files, scripts/ 18 scripts, tests/ 4, LICENSE.txt)
- Trial: `scratchpad\hatch-deneme\Pitir.cs` (387 lines), `build.ps1`, `contact-sheet.png`, `pet\pet.json`
- `docs\SPRITE.md`, `docs\DECISIONS.md` (decisions 3 and 10), `app\src-tauri\src\pet.rs`, `app\src\main.js`

---

## 1. License

### Findings

| What | License | Source |
|---|---|---|
| Local hatch-pet skill | Apache-2.0 (`LICENSE.txt`, full text, template with the copyright line left unfilled) | `~\.codex\skills\hatch-pet\LICENSE.txt` |
| Same skill, official repo | Apache-2.0 (`skills/.curated/hatch-pet/LICENSE.txt`) | https://github.com/openai/skills/tree/main/skills/.curated/hatch-pet |
| openai/skills repo as a whole | No repo-level license. README: "The license of an individual skill can be found directly inside the skill's directory inside the `LICENSE.txt` file." | https://github.com/openai/skills (README, "License" section) |
| openai/codex (the app) | Apache-2.0 | https://github.com/openai/codex (GitHub API: `license.spdx_id = Apache-2.0`) |
| Pet format (atlas size, grid, row layout, pet.json fields) | The format itself is a software interface / fact, not a copyrightable "work". There is no official documentation page; the definition lives in the skill's `references/` files. | https://github.com/openai/skills/blob/main/skills/.curated/hatch-pet/SKILL.md |

- The skill folder has no `NOTICE` file, and the scripts have no copyright headers (`grep copyright` is empty).
- The local copy and the repo version may differ (the repo's `openai.yaml` description says "any pet-safe style", the local one "v2 pet with ... 16 look directions"). The license is the same in both.

### Conclusion

- **We may adapt it.** Apache-2.0 allows copying the text and scripts, modifying them, and distributing them in an MIT project. Conditions (Section 4):
  1. Ship the Apache-2.0 license text with the adapted file,
  2. Mark modified files as "modified" (4b),
  3. Preserve any copyright/NOTICE statements (there are none here),
  4. Section 6: do not use the OpenAI/Codex marks in a way that implies endorsement (saying "Codex compatible" is fine, naming it "Codex Hatch" is not).
- **My recommendation: write from scratch, take only ideas and facts.** Reasons:
  - The pipeline is fundamentally different. Theirs is "generate an image, separate it with a chroma key, extract components, despill"; ours is "draw from parameters". About 70% of the scripts (extract_strip_frames, despill_chroma_edges, assemble_extended_atlas registration/scaling, blind A/B, the imagegen job list) are unnecessary for us.
  - Their rule explicitly forbids what we do: "Never substitute locally drawn, tiled, transformed, or code-generated row strips" (SKILL.md 880). Writing our own text is cleaner than adapting theirs.
  - The scripts are Python + Pillow. We do not want Python (DECISIONS 1: the user should not need Python/Node).
  - The project stays pure MIT, with no need to carry a license file.
- **Attribution:** Not required (if no code is copied). For courtesy and transparency, a line in the README is recommended: "The atlas layout and QA ideas are inspired by OpenAI's Apache-2.0 licensed hatch-pet skill." If a validation function is ported line by line (e.g. `measure_direction_continuity.py` → C#), that file gets the header "Portions adapted from openai/skills hatch-pet, Apache-2.0, modified" and a `THIRD_PARTY_NOTICES` or `licenses/Apache-2.0.txt` is added. Threshold numbers (2 px edge margin, 1.15 area ratio, etc.) count as facts; copying them does not require attribution.

---

## 2. API-independent valuable parts of the Codex skill

### 2.1 Atlas contract (`codex-pet-contract.md`, `animation-rows.md`)

- 1536×2288, 8×11, cell 192×208, transparent, PNG or WebP, `spriteVersionNumber: 2`.
- Frame counts and **durations** per row (we have no durations; they could be added to SPRITE.md):

| Row | State | Frames | Durations (ms) |
|---|---|---|---|
| 0 | idle | 6 | 280, 110, 110, 140, 140, 320 |
| 1 | running-right | 8 | 120 ×7, last 220 |
| 2 | running-left | 8 | 120 ×7, last 220 |
| 3 | waving | 4 | 140 ×3, last 280 |
| 4 | jumping | 5 | 140 ×4, last 280 |
| 5 | failed | 8 | 140 ×7, last 240 |
| 6 | waiting | 6 | 150 ×5, last 260 |
| 7 | running | 6 | 120 ×5, last 220 |
| 8 | review | 6 | 150 ×5, last 280 |
| 9 | look 000–157.5 | 8 | – |
| 10 | look 180–337.5 | 8 | – |

- Unused cells are fully transparent. Transparent pixels must have RGB = 0 ("transparent RGB residue" error).
- **In v2, cell (0,6) is the "neutral/front" frame and must be filled.** `validate_atlas.py`: `EXTENDED_NEUTRAL_LOOK_FRAME = (0, 6)`; in v2 this cell counts as "used", and it is an error if it has fewer than 50 pixels. `assemble_extended_atlas.py` writes it as `neutralLookFrame: {rowIndex:0, columnIndex:6}`. The observation "idle 7 frames (6 + neutral)" in SPRITE.md is a consequence of this.
- 000° = up, clockwise; neutral/front = cursor dead zone, falls back to idle.
- Row meanings: `running` = work/processing (not running on foot), `review` = focused inspection, `waiting` = eager pose awaiting approval/input, `failed` = sad/deflated. `waiting`, `running`, `review`, `failed` must be distinguishable from each other.
- `running-left`: mirror only if identity and handedness (which hand holds the prop) are preserved; **frame order is kept** (the strip is not mirrored as a whole, each frame is mirrored in place).

### 2.2 QA rubric (`qa-rubric.md` + SKILL.md rules), items that fit us

- Identity: silhouette, proportions, face, palette, material, markings, props are the same across all 11 rows.
- Must be readable at 192×208; details should be chosen for pet size.
- No unrequested characters/objects/logos/text/scenery.
- Loops must not pop (size pop), must not have a reversed cadence, must not face the wrong way, **must not be static** (6 identical frames in idle are not accepted).
- The first idle frame must work as the "reduced motion" still image.
- Look: 16 directions in fixed order, all visually distinct from neutral; cardinal directions (000/090/180/270) must read unambiguously; diagonals in the correct quadrant. **Faking the look by rotating/skewing the whole sprite is forbidden** (unless the object is something that actually rotates). Adding new "googly" eyes or beads in place of eyes is forbidden. Across consecutive directions the body must progress continuously; base/feet are a fixed anchor, and scale and baseline match neutral.
- **Eye/head/body hierarchy:** the eyes lead the look, the head follows, the upper body follows slightly, the lower body/feet stay fixed. Shifting only the pupils is an "exception" (warning).
- **Effect rules:** only small, opaque effects that fit the state and **touch/are attached to** the pet. Detached stars, dots, question marks, thought bubbles, speed lines, shadows, sparkles are forbidden. (Their reason is chroma separation; we have no technical obstacle, but the "cleanliness at pet size" reason still applies.)
- State guide: idle only breathing/blinking; waving only limb pose (no wave lines); jumping only body position (no shadow/dust); review only leaning/eyes/head/hands (no magnifier or paper, unless in the base identity).
- Repair policy: repair the smallest scope (one row), and rerun the full deterministic validation after every repair.
- Convergence rule: if the same root error repeats twice, stop tweaking parameters and change strategy; if a repair moves the error to another cell, that is a loop.

### 2.3 Deterministic validation scripts: what they check

| Script | Check | For us |
|---|---|---|
| `validate_atlas.py` | size (v2 required), PNG/WebP, whether there is an alpha channel; used/unused cells per row; used ≥ 50 pixels; unused = 0 pixels; (0,6) neutral filled; cell more than 95% opaque → "background left over"; RGB residue in transparent pixels; chroma leak/fringe | All except chroma. Ports to C#. |
| `inspect_frames.py` | frame count; frame size 192×208; ≥ 400 pixels; > 24 pixels in the 2 px edge strip → clipping warning; < 35% or > 275% of the row's median area → outlier warning; extraction method | Edge and area outlier checks are valuable. |
| `measure_direction_continuity.py` | consecutive pairs among the 16 look cells (including 337.5→000): diff pixel count is a local outlier (> neighbor avg ×1.45), bbox center shift > 8 px, area ratio > 1.15; horizontal transparent "hole rows" inside the body | Valuable as is. |
| `make_contact_sheet.py` | atlas on a checkerboard background, used cells labeled | For visual QA. |
| `make_direction_qa_sheet.py` | neutral + 16 directions, labeled with degrees, plus head/upper-body close-up | For visual QA. |
| `render_animation_previews.py` | one GIF per row (with real durations) | GIF is hard for us; an HTML preview is recommended. |
| `make_direction_blind_qa_sheet.py` + `combine_*` + `validate_*` | random A/B pairs, 3 independent workers, majority vote; cardinal pairs are a hard gate | Mostly unnecessary for us (below: direction check is measured from code). Optionally a single blind subagent. |
| `derive_running_left_from_running_right.py` | frame-by-frame in-place mirror, order preserved | We redraw with `facing=-1` instead of mirroring. |
| `extract_*`, `despill_*`, `assemble_extended_atlas.py`, `prepare_pet_run.py` | cleaning/registering image generation output | Unnecessary. One idea to keep: **common scale + baseline and lower-body anchor taken from neutral.** |

### 2.4 Workflow / progress plan

Codex's visible checklist (4 steps) is a good user experience:
1. Preparing `<Pet>` (name, description, style, folder)
2. `<Pet>`'s main look (canonical base frame)
3. `<Pet>`'s poses (rows 0–8, look mechanics plan, rows 9–10)
4. `<Pet>` is hatching (atlas, QA, packaging)

Also: "complete a step only when a real file/decision exists"; a time budget (~30 min); having `look-mechanics.md` written before the look rows ("when this character looks around, what is the most natural movement? what stays fixed, what leads, what follows, what bends?"), and getting the 4 cardinal directions approved first, then filling in between. These last two are very valuable for us too.

### 2.5 Differences from SPRITE.md

1. **(0,6) neutral cell:** SPRITE.md only notes it as an observation, "idle 7 frames in Johnny (6 + neutral)". In the Codex contract it is required in v2 and named `neutralLookFrame`. Since our app counts non-empty cells, it may be playing neutral as the 7th frame of the idle loop (needs verification). The skill must always fill (0,6) (the same front pose as idle 0 is recommended, so it goes unnoticed in the loop).
2. **`spriteVersionNumber`:** DECISIONS 3 says it "only appears in the community". The current Codex skill treats it as **required**: "Omitting it defaults the pet to v1 and causes the 2288-pixel-tall spritesheet to be rejected." Our app looks at the image size (correct), but we **should write** the field so the pet.json we produce also works in Codex. Pitir's `pet.json` lacks this field. The sentence in DECISIONS 3 should be updated.
3. **Frame durations:** Not in SPRITE.md (we use a single duration via `frameMs`). Codex defines a duration list per row. Could be added for information; the skill does not write them to `claude-pet.json` (the format supports a single `frameMs`), it only uses them in the preview.
4. **180°:** SPRITE.md says "differs from pet to pet (Johnny from behind, clawd-fan down)". The Codex contract is clear: 180 = **looking down**, not turning around. Our skill should draw a downward look.
5. **Mirror rule:** Codex: "preserving frame order, and only if identity is not broken". Not in SPRITE.md; goes into the skill docs.
6. The ban on detached effects and row meanings such as "running ≠ running on foot" are not in SPRITE.md; they go into the skill's `references/` docs.

---

## 3. Our code-drawing skill: design proposal

Goal: **simple request ("an orange cat") → good result from archetype + palette + presets; detailed request → a result close to what was asked, via part-by-part definition or free code.** The pipeline (atlas, motion derivation, QA, packaging) stays the same in every case; only the "character drawing" layer changes.

### 3.0 Layers

```
spec.json  ──►  Rig (parts, pivots, depth)  ──►  Pose (per frame)  ──►  Raster (grid, palette, outline)  ──►  Atlas + meta.json
   ▲                      ▲                                   ▲
 archetype template   custom/*.cs (escape hatch)        Motion rules (row generators)
```

### 3.1 (a) Character definition format: `spec.json`

Three levels of detail, same file:

**Level 1 (simple request):** only archetype + palette + a few options.
```json
{
  "formatVersion": 1,
  "id": "pitir", "displayName": "Pitir", "description": "Tiny mandarin sprout.",
  "archetype": "blob",
  "style": "pixel",
  "palette": { "body": "#FFB066", "accent": "#8BD14A", "outline": "#2B2140" },
  "features": { "eyes": "dot", "mouth": "smile", "cheeks": true, "topper": "sprout", "feet": true, "arms": "nub" }
}
```

**Level 2 (detailed request):** override/add the archetype's parts.
```json
{
  "archetype": "biped",
  "grid": { "w": 48, "h": 52, "scale": 4 },
  "light": "top-left",
  "palette": {
    "fur":   { "base": "#E9A15B", "shade": "auto", "hi": "auto" },
    "belly": "#FBE3C4", "outline": "#2B2140", "eye": "#1C1626"
  },
  "parts": {
    "head":  { "shape": "ellipse", "size": [22, 18], "color": "fur", "parent": "body", "pivot": [0, -14] },
    "earL":  { "shape": "triangle", "size": [6, 8], "color": "fur", "parent": "head", "at": [-7, -8], "depth": 0.6 },
    "earR":  { "mirrorOf": "earL" },
    "tail":  { "shape": "curve", "length": 10, "width": 3, "color": "fur", "parent": "body", "at": [9, 6], "depth": -0.8, "follow": 0.4 },
    "scarf": { "shape": "band", "color": "#D9443A", "parent": "body", "at": [0, -9], "worn": true }
  },
  "face": { "eyes": { "type": "globe", "size": [3, 4], "gap": 9 }, "mouth": "cat", "brows": true },
  "props": { "laptop": { "use": "running" } },
  "asymmetric": ["scarf"]
}
```

Field rules:
- `grid`: presets `48×52 @4` (chunky pixels, default for "pixel"), `96×104 @2` (fine pixels), `192×208 @1` (smooth/antialiased, "flat"/"sticker"). All divide 192×208 exactly.
- `palette`: named tones; with `"auto"` the shade/highlight is derived automatically (L ± in HSL, slight hue shift). In the pixel style **only palette colors** are used (this enables a deterministic identity check).
- `parts`: primitive shapes (`ellipse`, `roundrect`, `triangle`, `polygon`, `curve`/capsule, `band`, `pixelmap`), hierarchy via `parent`, `pivot`/`at` relative to the parent, `depth` (−1 back … +1 front; used for body turn), `follow` (secondary motion lag 0–1), `mirrorOf`.
- `pixelmap`: ASCII grid + palette letters (e.g. a custom face/helmet/non-logo pattern). The most reliable way for Claude to "paint" small details by hand.
- `style` presets: `pixel` (selective outline, 1 logical px), `pixel-fine`, `flat` (GDI+ antialiasing, thick outline), `sticker` (outer white border + outline), `soft` (smooth shading transition). A preset sets outline thickness, number of shade bands, light direction, and antialiasing.
- `archetype` templates (scripts/templates/): `blob` (Pitir type), `biped` (head+body+arms+legs), `quadruped`, `object` (mug, robot, screen-faced; eyes printed on the surface), `floaty` (ghost/cloud, footless, hovering). Each template: part list + default proportions + **look mechanics** + row settings.

### 3.2 (b) Free code escape hatch

Three tiers from simple to free:
1. **`pixelmap` part** (no code, ASCII inside JSON).
2. **Custom part:** `"shape": "custom", "class": "Visor"` → `IPart` interface in `custom/Visor.cs`:
   `void Draw(Canvas c, PartPose p)` — `Canvas` provides `Px`, `Ell`, `Poly`, `Line`, `Fill` on the logical grid, plus colors by name via `Pal("fur.shade")`; `PartPose` carries position, scale, yaw/pitch, expression. The part is still inside the rig; motion, depth order, outline and QA are automatic.
3. **Fully custom character:** `"renderer": "custom/MyPet.cs"`, `ICharacter.Render(Canvas c, Pose p)`. Claude draws everything, but the pipeline supplies the Pose (row rules, look directions, effect rules stay the same), and the atlas/QA/package are the same. Pitir is at this tier today.

Rule: the escape hatch frees only the **drawing**; it cannot touch atlas geometry, frame counts, the (0,6) neutral, packaging, or the validator. Compilation: PowerShell 5.1 `Add-Type` = **C# 5** (no string interpolation, `?.`, or tuples; the trial file already complies). The skill docs should state this explicitly.

### 3.3 (c) Animation and look derivation rules

**Pose model (per frame):**
`rootDy, squash, lean (degrees, hip pivot), yaw (−1…1), pitch (−1…1), headYaw, headPitch, eyeX, eyeY, blink, expression {eyes, mouth, brows}, limbs {name: angle}, prop {name: state}, effects [attached]`.

**Pseudo-3D body turn (Pitir's biggest gap):**
- Each part's `depth` value and its horizontal position relative to the parent's center are converted to an angle on a cylinder/ellipsoid: `φ = asin(x / r)`. On turning, `x' = r·sin(φ + yaw·Ymax)`, visible width `w' = w·cos(φ + yaw·Ymax)` (foreshortening), `z' = cos(φ + yaw·Ymax)` → draw order by z.
- Result: facial features (eyes, mouth, cheeks) shift toward the turn direction and the far eye narrows/hides; the front/back order of ears, arms, tail swaps; the far arm goes behind the body; the cheek disappears on the far side.
- The silhouette changes too: on an ellipse body, the highlight band and shadow edge shift with yaw (the shadow widens on the side opposite the turn). For `blob`, a slight asymmetric bulge (3–5% toward yaw) gives the "it turned" feel.
- `Ymax` from the archetype: blob 50°, biped 60°, object 0 (surface fixed, only tilt/hinge).
- Pitch: features shift vertically by `ry·sin(pitch·Pmax)`; looking up, upper parts compress and the underside of the head (chin) shows; looking down, the reverse.

**Look (16 directions), θ: 0 = up, clockwise:**
- Target vector: `h = sin θ` (right +), `v = cos θ` (up +).
- Hierarchy (Codex's "eyes lead, head follows, body a little" rule): eyes `1.0`, head `0.6`, upper body `0.25`, feet/base `0`. So `eyeX = h`, `headYaw = 0.6h`, `bodyYaw = 0.25h`; vertically `eyeY = v`, `headPitch = 0.5v`, no body pitch, a small forward lean when looking down.
- Eyes: for the `globe` type the whole eyeball is redrawn (white + iris + highlight shift together); for the `dot` type the eye dot slides across the head surface and the highlight stays in the fixed light direction; for the `screen` type (object) only the drawn feature moves.
- Baseline and lower-body anchor identical to neutral (deterministic; no scaling, no shifting).
- 180° = looking down (Codex contract), not a back view.
- Even steps: at each 22.5° step the same parts move by roughly the same amount (sin/cos already ensures this; compute sub-pixel positions before rounding to integers, then round, and do the rounding in a single place to avoid two parts jumping in the same step).
- Rotating the whole sprite is forbidden; manual `rotate` only at part level.

**Row rules (Motion library, archetype-independent, parameterized):**
- `idle` (6): breathing squash 0→0.8→0, topper/tail sway lagging by 1 frame, blink on frame 5; cannot be static.
- `running-right` (8): **yaw = +0.6 (3/4 view), lean = +8…12° (hip pivot, upper parts only)**, two-phase bob (contact/passing), legs in opposite phase, arms swinging opposite to the legs, hair/tail/topper trailing back with `follow` lag. Mouth open, eyes look in the direction of travel.
- `running-left`: **not a pixel mirror, redrawn with `yaw = −0.6, lean = −10°`** (light direction and asymmetric props stay correct). If the spec's `asymmetric` is empty and the style allows it, mirroring is an option; frame order is preserved.
- `waving` (4): start (arm raises, shoulder +), peak left, peak right, return. Arm in 2 segments (shoulder + elbow/wrist), swinging at the elbow; the body leans 3–5° toward the waving arm, happy eyes. Codex: "clear start, raised gesture, and return".
- `jumping` (5): anticipation (squash +), takeoff (stretch), apex, landing (stretch decreasing), settle (squash). Topper lags on the apex frame.
- `failed` (8): surprise (2 frames) → deflating (squash increases, topper/ears droop, eyes squeezed) → slight tremble. Sweat drop **touching the body**.
- `waiting` (6): eager pose: slight forward lean, looking up, one arm/topper slightly raised, eyes blinking.
- `running` (6): work: laptop/keyboard prop (if in the spec) or "focused fidgeting"; no running on foot.
- `review` (6): leaning + head tilted + narrowed eyes, one hand on the chin; no new prop.
- `look` (16): the look rule above; (0,6) = neutral front pose (same as idle 0).

**Effects:** default `effects: "attached"` (sweat, stars/smoke touching the body). Detached effects (Pitir's "...", "?", thought bubble, sparks) are off by default; see open question 4.

### 3.4 (d) Draw-look-fix QA loop

**Deterministic validation (`Qa.cs`, no Python):**
1. Atlas: 1536×2288, alpha, used/unused cells (table above), (0,6) filled, used ≥ 50 px, unused = 0 px, RGB = 0 in transparent pixels, no cell more than 95% opaque.
2. Frame: pixels in the 2 px edge strip (clipping), outlier frames relative to the row's median area (< 0.35 / > 2.75).
3. Staticness: error if all frames in a row are identical (idle included).
4. Baseline: in idle, look and (0,6) frames, the lowest opaque row within ±1 logical pixel.
5. Look continuity: consecutive pair diff/center/area ratio (Codex thresholds), horizontal transparent hole rows inside the body.
6. **Direction semantics (replacing Codex's blind 3-worker test):** the renderer writes feature positions to `meta.json` for each frame (eye centers, head center, body center). Check: `sign(x offset of the eyes relative to the head center) == sign(sin θ)` and the same with `cos θ` vertically; at 000/090/180/270 the magnitude must exceed a threshold (hard gate), in between the correct quadrant (warning). This is our biggest advantage: no need to guess the direction, we can measure it.
7. Palette (pixel style): is every color used in the palette; are color histograms similar across rows (catches identity drift).
8. Connected component count: warning if a frame has > 1 opaque component (detached effect/part), unless `effects: "detached"` is enabled.
9. Mirror check: does running-left face the opposite way of running-right (sign of eye x).
Output: `qa/review.json` (`ok`, `errors`, `warnings`, per-cell details).

**Visual QA (Claude looks at the PNGs with `Read`):**
- `qa/contact-sheet.png` (two versions, checkerboard background + light background; Pitir's light-background sheet is good).
- `qa/rows/<state>.png`: the row enlarged 2×, frames side by side + an "onion" strip (consecutive frame difference).
- `qa/look-directions.png`: neutral + 16 directions, labeled with degrees, plus a head close-up (the Codex `make_direction_qa_sheet` idea).
- `qa/preview.html`: atlas + CSS `steps()` animation with the Codex durations; Alperen opens it in a browser. (An animated GIF via System.Drawing is cumbersome; HTML has zero dependencies.) Optional: open it in the app as a temporary pet.

**Loop:**
1. Draw the base frame (idle 0, 4× enlarged) → Claude looks → shown to Alperen (single approval point: "is the look right?").
2. Draw the 4 cardinal directions (000/090/180/270) + 1 running-right frame → look (Codex's "cardinals first" idea; the body turn settles here).
3. Full atlas → deterministic QA → if there are errors, fix (spec or custom code), at most N=3 rounds.
4. Visual QA rubric (short list, references/qa-rubric.md), each item marked "pass/warning/fail".
5. Same root error twice → change strategy (e.g. simplify the part, change the archetype, switch to custom code).
6. Optional: give an independent subagent only the unlabeled direction sheet and have it classify the cardinals (a lightweight single-worker version of Codex's blind test). With the deterministic direction check in place, this can be off by default.

### 3.5 (e) Output folder and packaging

Working folder (deletable):
```
~/.claude-pet/hatch/<id>-<yyyyMMdd-HHmm>/
  spec.json
  custom/*.cs            (if any)
  build/frames/<state>/NN.png, meta.json
  final/spritesheet.png
  qa/review.json, contact-sheet.png, look-directions.png, rows/*.png, preview.html
```
Installed pet:
```
~/.claude-pet/pets/<id>/
  pet.json               {"id","displayName","description","spriteVersionNumber":2,"spritesheetPath":"spritesheet.png"}
  spritesheet.png        (Codex accepts PNG too; System.Drawing cannot write WebP)
  claude-pet.json        (optional: frameMs, states, idleExtras; if an extra sheet is needed)
  source/spec.json       (+ custom/*.cs) — for later edits like "add a hat to Pitir"
```
- Package only when `qa/review.json` has `ok: true`.
- Optional Codex copy to `~/.codex/pets/<id>/` (asked).
- The app does not read the `source/` folder (pet.rs reads only 5 fields and known files), so it is harmless.
- If the same id exists, ask before overwriting, or use `<id>-2`.

### 3.6 (f) Skill file layout

Inside the plugin (files in English, `.ps1` kept ASCII too; PS 5.1 reads BOM-less files as ANSI):
```
skills/hatch/                     (name proposal; "hatch-pet" clashes with Codex, see open question 1)
  SKILL.md                        (~150-200 lines: when to use, 4-step checklist, commands, loop, hard rules)
  references/
    atlas-contract.md             (rows, frame counts, durations, (0,6), 180=down, pet.json)
    spec-format.md                (spec.json fields, examples at 3 levels)
    archetypes.md                 (blob/biped/quadruped/object/floaty + look mechanics)
    motion-rules.md               (row rules, body turn, look hierarchy, effect rules)
    style-presets.md              (pixel, pixel-fine, flat, sticker, soft; readability tips)
    custom-code.md                (IPart/ICharacter, Canvas API, C# 5 constraints, example)
    qa-rubric.md                  (deterministic + visual list, repair/convergence rule)
  scripts/
    hatch.ps1                     (single entry: build | preview | validate | package; Add-Type, cache the compiled DLL)
    PetKit/Canvas.cs              (grid, palette, primitive shapes, selective outline, scaled blit, save PNG)
    PetKit/Rig.cs                 (part tree, pivot, depth, pseudo-3D yaw/pitch, z-order)
    PetKit/Motion.cs              (row generators, easing, follow lag, look 16)
    PetKit/Spec.cs                (JSON reading: ConvertFrom-Json in PS 5.1 → transfer to C#, or JavaScriptSerializer)
    PetKit/Qa.cs                  (validator, sheets, preview.html, meta.json)
    templates/*.json              (archetype templates)
  examples/pitir/spec.json        (spec equivalent of the trial pet, regression example)
```
Note: for reading JSON, .NET Framework has `System.Web.Script.Serialization.JavaScriptSerializer` (System.Web.Extensions); no extra package needed. Alternative: read with `ConvertFrom-Json` in PowerShell and transfer to a C# object by hand.

### 3.7 Reusable parts of the trial code (`Pitir.cs`)

Take directly:
- `Px`/`Get` + `mirror` flag, bounds clipping (lines 51–60).
- `In`/`Ell`/`OEll` rotatable ellipse (61–75), `Line` (76–82), `Rot` (85–88).
- Shading that preserves the light direction under mirroring (`nx` sign flip, 121) and the single-pixel highlight (129).
- `Blit` integer-scaled copy (256–264), `Save` fast PNG via LockBits + `Marshal.Copy` (266–278), sheet copy with a background.
- The `FrameSpec` idea (pose data per frame) and the sin/cos-based loop generation in `Rows()` (running bob, foot phases).
- Mouth/eye expression enums (`Eyes`, `Mouth`) → core of the `expression` library.

Change:
- Outline: `OEll` draws a separate outline around each part; when parts merge, inner outlines remain or vanish. Proposal: first fill + part ID mask, then an **outline pass** (filled pixel with an empty neighbor → outline; optionally a darker tone at boundaries between different parts = "selective outline").
- Static global `buf` → `Canvas` object.
- Hard-coded colors/proportions → palette + spec.
- Scanning the whole grid for every ellipse: not a problem at 48×52; at 192×208 @1, limit it to the bbox.

Gaps seen in the trial (looked at the contact sheet):
- The look rows only shift the face (Fx ±4, Fy ±2); the body/silhouette does not turn, and around 180 and 000 it is too close to neutral.
- Running has no lean and no 3/4 view; only feet and topper.
- Waving: two angles, 25°/65°, no start/return, single-segment arm.
- (0,6) neutral cell empty → Codex `validate_atlas.py --require-v2` reports an error.
- `pet.json` lacks `spriteVersionNumber: 2` → Codex rejects the 2288 height.
- Detached effects: "..." in waiting, thought bubble in review, sparks in jumping → violate the Codex rule.
- running-left is a pixel mirror (fine because Pitir is symmetric, but not a general solution).

---

## 4. Open questions (Alperen's call)

1. **Skill name and location:** `hatch`, a `/claude-pet hatch` subcommand, or a separate skill? ("hatch-pet" is the same name as Codex's, which is confusing.)
2. **Technology:** PowerShell 5.1 + C# 5 + System.Drawing (zero install, Windows only; consistent with the Windows-only first release decision), or should we put the drawing engine in the Rust exe as a `claude-pet hatch` subcommand (cross-platform later, but Claude's free-code escape hatch would need compilation, which gets harder)? My recommendation: PowerShell/C# for the first release.
3. **License/attribution:** write from scratch with a courtesy attribution in the README (my recommendation), or port the validation scripts and carry an Apache-2.0 notice?
4. **Detached effects:** follow the Codex rule (attached effects only by default), or allow effects like "..." and thought bubbles since they are no problem in our app? (Pitir's waiting/review rows rely on them.) My recommendation: off by default, enabled with `effects: "detached"` in the spec.
5. **How strict should Codex compatibility be:** should the pets we produce also work in Codex (`spriteVersionNumber`, (0,6) neutral, ban on detached effects, `~/.codex/pets` copy)? Compatibility is cheap; my recommendation is yes (except effects, see 4).
6. **Default style and grid:** `pixel 48×52 @4` (the trial), or the more detailed `96×104 @2`? Should the smooth (`flat`) style be in the first release?
7. **Approval points:** how many times should Alperen be asked? My recommendation: (1) base look, (2) 4 cardinal directions + running frame, (3) final preview and installation.
8. **Extra rows:** should the skill also produce an extra sheet with `claude-pet.json` for our additional states (`thinking`, `writing-code`, etc.), or only the 11 Codex rows?
9. **DECISIONS 3 update:** `spriteVersionNumber` is now required in the official skill; should the decision text's "only appears in the community" be corrected? Also, check whether the app plays the (0,6) neutral cell in the idle loop.
10. **Blind direction review:** is the deterministic direction measurement enough, or should we additionally have a subagent run an unlabeled cardinal test (cost: one subagent call)?

## Sources

- https://github.com/openai/skills (README "License": the license is in each skill folder)
- https://github.com/openai/skills/tree/main/skills/.curated/hatch-pet (LICENSE.txt: Apache-2.0)
- https://github.com/openai/skills/blob/main/skills/.curated/hatch-pet/SKILL.md
- https://github.com/openai/codex (Apache-2.0)
- https://www.apache.org/licenses/LICENSE-2.0 (Section 4 distribution conditions, Section 6 trademarks)
- https://github.com/openai/codex/issues/20863 (request for custom pet animation sequences, context)
