# spec.json format

Contents:
1. Minimal spec
2. Top-level fields
3. Coordinates
4. Palette and colors
5. Archetypes and options
6. Face
7. Parts (level b)
8. Pixel maps and overrides (level c)
9. Motion and props
10. Examples

## 1. Minimal spec

```json
{
  "formatVersion": 1,
  "id": "blue-cat",
  "displayName": "Blue Cat",
  "description": "A small blue cat.",
  "archetype": "critter",
  "palette": { "body": "#7FA8E8" },
  "options": { "ears": "cat", "tail": "long" }
}
```

Unknown fields are errors, so typos are caught. Error messages name the field to fix.

## 2. Top-level fields

| Field | Default | Notes |
| --- | --- | --- |
| `formatVersion` | `1` | must be 1 |
| `id` | required | 1-64 chars: lowercase letters, digits, `-`, `_`; becomes the folder name |
| `displayName` | required | shown in menus |
| `description` | `""` | one short sentence |
| `archetype` | `"blob"` | `blob`, `critter`, `floaty`, `custom` (no parts; you add all of them) |
| `grid` | `"48x52"` | logical pixel grid; `"96x104"` gives finer pixels (all numbers then double) |
| `palette` | | named colors, section 4 |
| `options` | | archetype options, section 5 |
| `face` | | section 6 |
| `parts` | `[]` | add or change parts, section 7 |
| `remove` | `[]` | part names to delete from the archetype |
| `motion` | | section 9 |
| `props` | | section 9 |
| `overrides` | `[]` | section 8 |

## 3. Coordinates

- Units are logical pixels of the grid (48 x 52 by default; one logical pixel = 4 x 4 screen
  pixels in the 192 x 208 cell). Fractions are fine.
- `x` grows to the right, `y` grows **down**, `z` grows toward the viewer.
- Every part belongs to a group (`on`):
  - `root`: origin is the middle of the ground line (y = 0 is where feet stand; the cell top is
    at y = -47.5 and the cell edges at x = -24 and +24). Root parts never turn.
  - `body`: origin is the body center. Turns a little when the pet looks around, leans when it runs.
  - `head`: origin is the head center. Turns most. Without a `head` part, `head` means the face
    area of the body: same center as the body, but it turns like a head (good for hats on blobs).
- `at` is the attachment point (also the rotation pivot for sway and droop); `offset` moves the
  shape's center away from that pivot.
- Keep the whole pet inside x -21…21 and y -44…0 so jumps and arm raises are not cut off. The
  engine lowers the jump height automatically, but a pet that is too big still gets `clipped`.

Reference sizes (48 x 52): blob body 30 x 26 centered at root (0, -16.5); critter head 28 x 22,
body 22 x 16; floaty body 28 x 28.

## 4. Palette and colors

Values are `"#RRGGBB"` or `{ "base": "#RRGGBB", "shade": "#RRGGBB", "hi": "#RRGGBB" }`; missing
`shade`/`hi` are derived automatically.

Built-in names (override any of them):

| Name | Default | Used for |
| --- | --- | --- |
| `body` | archetype color | body, head, arms |
| `outline` | `#2B2140` | outlines, sprout stem |
| `eye` | outline color | eyes, mouth |
| `white` | `#FFFFFF` | eye highlights, gloss |
| `accent` | `#8BD14A` | toppers, accessories |
| `foot` | darker body | feet |
| `cheek` | `#FF6F85` | blush |
| `tongue` | `#FF7A8E` | open mouth, cat nose |
| `sweat` | `#8FD8FF` | sweat drop in `failed` |
| `belly` | lighter body | belly patch |
| `inner` | `#F7A1B0` | inner ears |
| `pattern` | body shade | spots, stripes |
| `laptop` | grey | laptop in the `running` row |

Add your own names freely (`"hat": "#3B3355"`). Wherever a color is expected you can write
`name`, `name.shade`, `name.hi` or a literal `"#RRGGBB"`.

## 5. Archetypes and options

| Option | Values | Default |
| --- | --- | --- |
| `size` | 0.6 to 1.15 | 1 |
| `body` | `round`, `tall`, `wide` | `round` |
| `ears` | `none`, `cat`, `fox`, `bear`, `mouse`, `bunny`, `dog` | critter: `cat`, others: `none` |
| `tail` | `none`, `short`, `long`, `curl`, `fluffy` | critter: `long`, others: `none` |
| `topper` | `none`, `sprout`, `leaf`, `antenna`, `tuft`, `horn`, `bow` | `none` |
| `arms` | `gesture` (only when waving, jumping, waiting, reviewing, and as hands on the laptop), `always`, `none` (no hands, not even on the laptop) | blob: `gesture`, others: `always` |
| `feet` | `true`, `false` | `true` (floaty never has feet) |
| `belly` | `true`, `false` | critter: `true`, others: `false` |
| `accessory` | `none`, `scarf`, `bowtie` | `none` |
| `pattern` | `none`, `spots`, `stripes` | `none` |

Part names created by archetypes (use them in `parts` to change, or in `remove`):

- all: `body`, `foot`, `arm`, `belly`
- critter: `head`; floaty: `skirt`, `hem0`, `hem1`, `hem2`
- ears: `ear`, `earInner`; tail: `tail`
- toppers: `stem`, `leaf`, `leaf2` (sprout), `stem`, `leaf` (leaf), `stem`, `bulb` (antenna),
  `tuft0`-`tuft2`, `horn`, `bow`, `bow2`, `knot`
- patterns: `spot0`-`spot2`, `stripe0`-`stripe2`; accessories: `scarf`, `scarfTail`, `bowtie`, `bowtieKnot`
- Mirrored parts get a twin named `<name>~mirror`; change the original and both follow.

## 6. Face

```json
"face": { "eyes": "dot", "gap": 10, "y": -1, "mouth": "smile", "mouthDrop": 4, "cheeks": true, "brows": false, "color": "eye" }
```

| Field | Values |
| --- | --- |
| `eyes` | `dot` (with highlight), `round` (white with pupil), `bean` (small, no highlight), `sparkle` |
| `gap` | distance between eye centers |
| `y` | eye height relative to the face group center (negative = higher) |
| `mouth` | `smile` (small w), `curve` (u), `cat` (nose + w), `none` |
| `mouthDrop` | how far below the eyes the mouth sits |
| `cheeks` | blush marks |
| `brows` | eyebrows (worried in `failed`, one raised in `review`) |
| `color` | eye and mouth color |

The face sits on the head (or on the body if there is no head), moves with turning and changes
expression per row automatically (blink, happy, focused, sad, surprised).

## 7. Parts (level b)

A `parts` entry whose `name` already exists **changes** that part (only the given fields; `null`
deletes a field). A new name **adds** a part.

| Field | Meaning |
| --- | --- |
| `name` | unique name |
| `on` | `root`, `body` (default), `head` |
| `shape` | `ellipse`, `rect`, `triangle`, `polygon`, `capsule`, `pixels`, `limb` |
| `role` | `accessory` (default), `ear`, `tail`, `topper` (these three sway and droop), `foot` (moves in runs and jumps; put on `root`), `arm` (needs `limb`), `marking`, `body`, `head` |
| `at` | `[x, y]` attachment/pivot in the group |
| `offset` | `[x, y]` shape center relative to `at` |
| `z` | depth; negative draws behind the body (tails, back ears), positive in front |
| `surface` | `true`: glued to the front of the body/head surface (moves when turning, hides when turned away) |
| `clip` | `true`: only painted on top of the body/head, no outline (spots, belly, face masks) |
| `join` | name of another part: one shared silhouette and outline (ears joined to the head) |
| `size` | `[w, h]` for `ellipse` and `rect` |
| `radius` | rounded corners for `rect` |
| `rot` | rotation in degrees, clockwise |
| `points` | `[[x, y], ...]` relative to `at`: 3 for `triangle`, 3+ for `polygon`, 2+ (a path) for `capsule` |
| `width` | thickness of `capsule` and `limb` |
| `color` | color reference (section 4); default `body` |
| `shading` | `ball` (default: shade bottom right, light top left), `band` (bottom band), `flat` |
| `outline` | default `true` (`false` for `clip` and `pixels`) |
| `mirror` | `true`: add a copy mirrored to the other side |
| `gloss` | white sparkle pixel (main body/head only) |
| `sway`, `droop` | override how strongly the part swings (factor) and droops in `failed` (degrees) |
| `limb`, `hand`, `show` | arms only: `[upper, lower]` lengths, hand diameter, `always`/`gesture`/`never` |
| `depth` | body/head only: front-to-back thickness (default = width) |

Exactly one part has `role: body` (an ellipse on `root`); at most one has `role: head` (an
ellipse on `body`). With `archetype: custom` you must add both yourself.

## 8. Pixel maps and overrides (level c)

`pixels` part: each string is a row, each character one logical pixel. `.` and space are
transparent; other characters map to colors through `key`. The map's center is `at` + `offset`.

```json
{ "name": "badge", "on": "body", "shape": "pixels", "at": [6, 2], "z": 14,
  "rows": [".yy.", "yooy", ".yy."], "key": { "y": "#F2C14E", "o": "outline" } }
```

`overrides` paint over one finished frame (after all parts are drawn); `at` is the top-left
corner in the logical grid (0..47, 0..51 for 48 x 52), `mode` is `over` (default) or `replace`
(clears the frame first; you then draw the whole frame).

```json
"overrides": [
  { "row": "waving", "frame": 1, "at": [36, 10], "rows": ["y.y", ".y.", "y.y"], "key": { "y": "#FFE05A" } }
]
```

Row names: `idle`, `running-right`, `running-left`, `waving`, `jumping`, `failed`, `waiting`,
`running`, `review`, `look-a` (0-157.5 degrees), `look-b` (180-337.5 degrees). Overridden pixels
are not measured by the direction checks, and effects must still touch the pet.

## 9. Motion and props

```json
"motion": { "bounce": 1, "sway": 1, "lean": 10, "turn": 50, "pitch": 25 },
"props": { "laptop": true }
```

| Field | Range | Meaning |
| --- | --- | --- |
| `bounce` | 0-2 | squash, stretch and bob strength |
| `sway` | 0-2 | swing of toppers, tails, ears |
| `lean` | 0-25 | forward lean while running (degrees) |
| `turn` | 0-70 | how far the pet turns sideways; looking turns the head 0.6 and the body 0.25 of this |
| `pitch` | 0-40 | head tilt when looking up or down |
| `laptop` | bool | `running` (working) row shows a small laptop with the pet's hands on it; `false` = focused fidget without a laptop (arms move only when `arms` is `always`) |

Lower `turn` for flat or symmetric objects; raise `bounce` for jelly-like pets.

## 10. Examples

Robot-ish blob with antenna and a screen face patch:

```json
{
  "formatVersion": 1, "id": "bolt", "displayName": "Bolt", "description": "A tiny friendly robot.",
  "archetype": "blob",
  "palette": { "body": "#B8C4D6", "accent": "#FF7A59", "screen": "#2E3A55" },
  "options": { "body": "wide", "topper": "antenna", "arms": "always" },
  "face": { "eyes": "bean", "mouth": "curve", "cheeks": false, "color": "#7CF5C8" },
  "parts": [
    { "name": "screen", "role": "marking", "on": "head", "shape": "rect", "surface": true, "clip": true,
      "at": [0, 0], "size": [20, 11], "radius": 2, "color": "screen", "shading": "flat" }
  ]
}
```

Bunny with a bow on one ear side and no tail:

```json
{
  "formatVersion": 1, "id": "mochi", "displayName": "Mochi", "description": "A soft white bunny.",
  "archetype": "critter",
  "palette": { "body": "#F4F1EC", "accent": "#F28CB1" },
  "options": { "ears": "bunny", "tail": "short", "topper": "bow" }
}
```

More: `hatch --example pitir|critter|floaty|custom` (custom shows a hat built from parts and a
pixel-map band).
