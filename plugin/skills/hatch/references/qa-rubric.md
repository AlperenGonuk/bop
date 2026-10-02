# QA rubric

## Deterministic checks (printed by `hatch`, also in `report.json`)

| Code | Level | Meaning | Usual fix |
| --- | --- | --- | --- |
| `clipped` | error | a frame touches the cell edge, part of the pet is cut off | smaller `options.size` or part sizes; lower toppers/ears; less `motion.lean`/`turn` |
| `frame-empty` | error | a frame has almost no pixels | a part is far outside the grid, or an override replaced the frame with little content |
| `neutral-empty` | error | cell (0,6) is empty | only happens with `replace` overrides on idle frame 0 |
| `unused-not-empty`, `rgb-residue`, `opaque-cell`, `size` | error | atlas format problems | should not happen with the engine; report them |
| `static-row` | error | all frames of a row are identical | `motion.bounce`/`sway` at 0 together with no moving parts; raise them |
| `look-same-as-neutral` | error | a look frame equals the neutral pose | `motion.turn`/`pitch` at 0, or eyes hidden |
| `look-direction` | error (cardinals) / warning | eyes did not move toward the look angle | eyes covered by a part (move the part or lower its `z`), eyes too close to the edge (`face.gap` smaller), `turn` too low |
| `run-direction` | error | running rows do not face their direction | same as above |
| `eyes-unmeasured` | warning | no eye pixels found | eyes hidden by an override or a part |
| `detached` | warning | a frame has separate pieces | a part does not touch the body (move it closer); a foot lifting off during a run is acceptable |
| `look-size-jump`, `look-jump`, `look-baseline` | warning | neighbouring look directions differ too much | reduce `turn`, avoid very wide parts that swing in and out |
| `look-hierarchy` | warning | the body moved more than the eyes | reduce parts on `body` with big offsets, or raise `turn` |
| `auto-fit` | warning | the pet did not fit and was drawn smaller (down to 80%) | fine if it looks good; otherwise reduce `options.size` or the biggest parts yourself |

All errors must be fixed before `install`.

## Visual review (look at contact-sheet.png)

Check each item; fix only what fails:

1. **Identity**: same silhouette, colors, face and accessories in every row; matches the request.
2. **Readable**: the pet reads clearly at cell size; eyes visible; no muddy colors (outline
   contrasts with body; avoid body colors close to the outline color).
3. **Nothing extra**: no text, logos or objects the user did not ask for.
4. **Motion**: every row visibly changes from frame to frame; running leans and faces its
   direction; waving shows a raised arm; jumping leaves the ground; failed looks sad; waiting,
   working (laptop) and review are distinct from each other.
5. **Looking**: the 16 look cells turn smoothly around the circle; 90 clearly right, 270 clearly
   left, 0 up, 180 down; feet stay in place.
6. **Clean edges**: no stray pixels, no parts floating apart, accessories sit on the body.
7. **Neutral cell** (labeled `N`) matches idle frame 0.

## Fix policy

- Change the smallest thing (one part, one size, one color) and hatch again; the whole atlas is
  re-validated each time.
- At most 3 fix rounds. If the same problem returns twice, change strategy: simplify or remove the
  part, choose another archetype option, or replace a fiddly part with a `pixels` map.
- If a fix moves the problem to another row, revert it and try a different fix.
