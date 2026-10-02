// Pet assembly: main sheet + bop.json (extra sheets, named animations,
// state mapping, idle moves). Bad entries are skipped with a warning; the pet never breaks.
import { loadSheet, loadExtraSheet, contentBounds, CELL_H } from "./sprites.js";
import { CODEX_DEFAULT_STATES } from "./states.js";
import { DEFAULT_FRAME_MS } from "./player.js";

const { convertFileSrc } = window.__TAURI__.core;

// Look around: neutral → right → neutral → left → neutral → quick glance up → neutral.
// Look index i = i × 22.5°, 0° up, 90° right, 270° left (docs/SPRITE.md). "N" is the neutral pose
// (v2 cell (0,6)). Front half-circle only (−90°…+90°): 180° is a back view on older pets.
const LOOK_AROUND = [
  "N", "N", 3, 4, 4, 4, 4, 3, "N",
  13, 12, 12, 12, 12, 13, "N",
  2, 1, 0, 0, 15, 14, "N", "N",
];
const LOOK_FRAME_MS = 160;

/**
 * info: output of the pet_info command (including config).
 * Returns: { name, version, hasLook, lookFrames, neutralFrame, image, animations, stateMapping, idleExtras, warnings }
 */
export async function buildPet(info) {
  const config = info.config ?? {};
  const warnings = [...(config.warnings ?? [])];
  const sheet = await loadSheet(convertFileSrc(info.spritesheetFile));
  const frameMs = config.frameMs ?? DEFAULT_FRAME_MS;

  const animations = {};
  for (const [name, anim] of Object.entries(sheet.animations)) {
    if (name.startsWith("look-")) continue; // look rows are handled separately: lookFrames
    animations[name] = { ...anim, frameMs };
  }

  // Extra sheets.
  const extra = {};
  for (const [name, path] of Object.entries(config.sheets ?? {})) {
    try {
      extra[name] = await loadExtraSheet(convertFileSrc(path));
    } catch (e) {
      warnings.push(`sheets.${name}: ${e.message ?? e}, skipped`);
    }
  }

  // Named animations (may also override Codex rows).
  for (const [name, def] of Object.entries(config.animations ?? {})) {
    const src =
      def.sheet === "main"
        ? { image: sheet.image, rows: mainRows(sheet) }
        : extra[def.sheet];
    if (!src) {
      warnings.push(`animations.${name}: '${def.sheet}' failed to load, skipped`);
      continue;
    }
    const frames = src.rows[def.row];
    if (!frames) {
      warnings.push(`animations.${name}: row ${def.row} does not exist (${src.rows.length} rows), skipped`);
      continue;
    }
    if (!frames.length) {
      warnings.push(`animations.${name}: row ${def.row} is empty, skipped`);
      continue;
    }
    animations[name] = { image: src.image, frames, frameMs: def.frameMs ?? frameMs };
  }

  // Neutral front pose: cell (0,6) in v2; otherwise (v1 or empty cell) the first idle frame.
  const neutralFrame = sheet.neutralFrame ?? sheet.animations.idle?.frames[0] ?? null;

  // Look around: v2 only (silently absent in v1). Starts and ends with the neutral pose.
  if (sheet.hasLook && neutralFrame) {
    animations.look = {
      image: sheet.image,
      frames: LOOK_AROUND.map((i) => (i === "N" ? neutralFrame : sheet.lookFrames[i])),
      frameMs: LOOK_FRAME_MS,
    };
  }

  const stateMapping = { ...CODEX_DEFAULT_STATES, ...(config.states ?? {}) };
  for (const [state, anim] of Object.entries(config.states ?? {})) {
    if (!animations[anim]) warnings.push(`states.${state}: no '${anim}' animation, falling back to the parent state`);
  }

  const wanted = config.idleExtras ?? (sheet.hasLook ? ["look"] : []);
  const idleExtras = wanted.filter((name) => {
    if (animations[name]) return true;
    // In v1 "look" is dropped silently (DECISIONS.md, decision 3).
    if (name !== "look" || sheet.hasLook) warnings.push(`idleExtras: no '${name}' animation, skipped`);
    return false;
  });

  return {
    name: info.displayName,
    version: sheet.version,
    hasLook: sheet.hasLook,
    lookFrames: sheet.lookFrames,
    neutralFrame,
    image: sheet.image,
    animations,
    stateMapping,
    idleExtras,
    // Visible bounds of the character within its cell; the bubble snaps to them. All animations
    // (each with its own sheet) are included, so the open bubble never covers any frame.
    bounds: animationBounds(animations),
    warnings,
  };
}

// Union of the visible areas of all animations: { top, bottom } (cell pixels).
// The same cell can appear in several animations (e.g. "look", "sheet": "main"); each cell is scanned once.
function animationBounds(animations) {
  const cells = new Map(); // image → { "x,y": frame }
  for (const anim of Object.values(animations)) {
    if (!cells.has(anim.image)) cells.set(anim.image, new Map());
    const byPos = cells.get(anim.image);
    for (const f of anim.frames) byPos.set(`${f.x},${f.y}`, f);
  }
  let top = CELL_H;
  let bottom = 0;
  for (const [image, byPos] of cells) {
    if (!byPos.size) continue;
    const b = contentBounds(image, [...byPos.values()]);
    if (!b) continue; // only faint frames: should not affect the bubble's position
    top = Math.min(top, b.top);
    bottom = Math.max(bottom, b.bottom);
  }
  return top < bottom ? { top, bottom } : { top: 0, bottom: CELL_H };
}

// Rows of the main sheet (for bop.json "sheet": "main"), empty frames skipped.
function mainRows(sheet) {
  return Object.values(sheet.animations).map((a) => a.frames);
}

/**
 * Look frame toward the cursor: 0° up/front, clockwise.
 * Front half-circle only (-90°…+90°): in the Codex contract 180° looks down, but older pets
 * (e.g. Johnny) draw a back view there; the front half-circle looks right on both.
 */
export function lookIndex(dx, dy) {
  let deg = (Math.atan2(dx, -dy) * 180) / Math.PI; // -180…180
  deg = Math.max(-90, Math.min(90, deg));
  return (Math.round(deg / 22.5) + 16) % 16;
}
