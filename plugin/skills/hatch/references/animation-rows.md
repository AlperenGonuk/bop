# Animation rows

The atlas is 1536 x 2288 pixels: 8 columns x 11 rows of 192 x 208 cells (Codex v2 pet
format). The engine generates every row; this is what each one should look like when you review
the contact sheet.

| Row | Name | Frames | Bop state | What the engine draws |
| --- | --- | --- | --- | --- |
| 0 | idle | 6 + neutral | idle | breathing squash, topper/tail swing, blink on frame 4 |
| 0, col 6 | neutral | 1 | look dead zone | same as idle frame 0, labeled `N` on the contact sheet |
| 1 | running-right | 8 | dragging right | 3/4 turn to the right, forward lean, bob, alternating feet, open mouth, topper trails back |
| 2 | running-left | 8 | dragging left | the same cycle redrawn facing left (not a pixel mirror; light stays top left) |
| 3 | waving | 4 | greeting | arm rises, waves out, waves in, returns; happy eyes, slight lean and head tilt |
| 4 | jumping | 5 | done | crouch, take off (stretch), apex (arms up, happy), fall, land (squash) |
| 5 | failed | 8 | failed | startled hop and "o" mouth, then sagging, squeezed eyes, frown, drooping topper/ears, attached sweat drop, small shiver |
| 6 | waiting | 6 | waiting for permission | eager: raised hand, looking up, small hops, blink |
| 7 | running | 6 | working | typing on a small laptop with focused eyes (with `props.laptop: false`: a focused fidget, arms move only when `arms` is `always`) |
| 8 | review | 6 | thinking | squint, head tilt, raised brow, hand at the side of the face, eyes scanning left to right |
| 9 | look-a | 8 | follows the cursor | look 0, 22.5, ... 157.5 degrees |
| 10 | look-b | 8 | follows the cursor | look 180, 202.5, ... 337.5 degrees |

Look angles: 0 = up, 90 = right, 180 = down, 270 = left, clockwise. Eyes lead (full amount),
the head follows about 0.6, the body about 0.25, feet stay planted and the baseline does not
move. The direction is measured from the eye pixels, so wrong directions are reported as errors.

Codex frame durations (ms, for reference; Bop plays rows at one speed):

| Row | Durations |
| --- | --- |
| idle | 280, 110, 110, 140, 140, 320 |
| running-right / running-left | 120 x 7, 220 |
| waving | 140 x 3, 280 |
| jumping | 140 x 4, 280 |
| failed | 140 x 7, 240 |
| waiting | 150 x 5, 260 |
| running | 120 x 5, 220 |
| review | 150 x 5, 280 |

Unused cells (row 0 col 7, the ends of short rows) stay fully transparent.
