// Codex spritesheet reader. Details: docs/SPRITE.md

export const CELL_W = 192;
export const CELL_H = 208;
export const COLS = 8;
// v2 neutral front pose cell: row 0, column 6.
export const NEUTRAL_COL = 6;

// The version is detected from the image size (DECISIONS.md, decision 3).
const VERSIONS = [
  { version: 1, width: 1536, height: 1872, rows: 9 },
  { version: 2, width: 1536, height: 2288, rows: 11 },
];

// Codex row names. The look rows exist only in v2.
export const ROW_NAMES = [
  "idle", "running-right", "running-left", "waving", "jumping",
  "failed", "waiting", "running", "review", "look-a", "look-b",
];

export function detectVersion(width, height) {
  return VERSIONS.find((v) => v.width === width && v.height === height) ?? null;
}

/** Loads an image from a URL (as a browser Image object). */
export async function loadImage(url) {
  const img = new Image();
  // The asset protocol grants CORS to the window origin, so the canvas stays readable.
  img.crossOrigin = "anonymous";
  img.src = url;
  await img.decode();
  return img;
}

/** Loads the image, detects its version and extracts the non-empty frames of each row. */
export async function loadSheet(url) {
  const img = await loadImage(url);

  const ver = detectVersion(img.naturalWidth, img.naturalHeight);
  if (!ver) {
    throw new Error(
      `Unsupported spritesheet size: ${img.naturalWidth}×${img.naturalHeight} ` +
      "(v1: 1536×1872, v2: 1536×2288)"
    );
  }

  const rows = rowFrames(img, ver.rows);
  // In v2, cell (0,6) is not an idle frame but the neutral front pose (Codex `neutralLookFrame`,
  // docs/SPRITE.md). It is removed from the idle loop.
  let neutralFrame = null;
  if (ver.version >= 2) {
    const i = rows[0].findIndex((f) => f.x === NEUTRAL_COL * CELL_W);
    if (i > 0) neutralFrame = rows[0].splice(i, 1)[0];
  }
  const animations = {};
  rows.forEach((frames, row) => {
    animations[ROW_NAMES[row]] = { image: img, frames };
  });

  const hasLook = ver.version >= 2;
  return {
    image: img,
    version: ver.version,
    // v1 has no look rows: looking at the cursor is silently off.
    hasLook,
    neutralFrame,
    // 16 directions, 0° up, clockwise in 22.5° steps (row 9: 0-157.5°, row 10: 180-337.5°; 180° = down).
    lookFrames: hasLook
      ? Array.from({ length: 16 }, (_, i) => ({ x: (i % COLS) * CELL_W, y: (9 + Math.floor(i / COLS)) * CELL_H }))
      : [],
    animations,
  };
}

/**
 * Extra sheet (bop.json): same grid, 192×208 cells, at most 8 columns, any number of rows.
 * Throws if it does not fit; the caller turns that into a warning and skips it.
 */
export async function loadExtraSheet(url) {
  const img = await loadImage(url);
  const { naturalWidth: w, naturalHeight: h } = img;
  if (w % CELL_W || h % CELL_H || w / CELL_W > COLS || !w || !h) {
    throw new Error(`grid does not fit (${w}×${h}; cell ${CELL_W}×${CELL_H}, at most ${COLS} columns)`);
  }
  return { image: img, rows: rowFrames(img, h / CELL_H) };
}

/** Returns the non-empty frames of each row: [[{x, y}, ...], ...] */
export function rowFrames(img, rowCount) {
  const canvas = document.createElement("canvas");
  canvas.width = img.naturalWidth;
  canvas.height = img.naturalHeight;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  ctx.drawImage(img, 0, 0);
  const alpha = ctx.getImageData(0, 0, canvas.width, canvas.height).data;

  // Extra sheets may be narrower than 8 columns; cells outside the image are not scanned.
  const cols = Math.min(COLS, Math.floor(canvas.width / CELL_W));
  const rows = [];
  for (let row = 0; row < rowCount; row++) {
    const frames = [];
    for (let col = 0; col < cols; col++) {
      if (!cellIsEmpty(alpha, canvas.width, col, row)) {
        frames.push({ x: col * CELL_W, y: row * CELL_H });
      }
    }
    rows.push(frames);
  }
  return rows;
}

/**
 * Vertical bounds of the visible area of the frames within the cell: { top, bottom } (cell pixels).
 * Used to snap the bubble right above/below the character (so the cell's transparent margin adds no gap).
 * Returns null if there are no clearly visible pixels (e.g. very faint frames): the caller skips this image.
 */
export function contentBounds(img, frames) {
  const canvas = document.createElement("canvas");
  canvas.width = CELL_W;
  canvas.height = CELL_H;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  let top = CELL_H;
  let bottom = 0;
  for (const f of frames) {
    ctx.clearRect(0, 0, CELL_W, CELL_H);
    ctx.drawImage(img, f.x, f.y, CELL_W, CELL_H, 0, 0, CELL_W, CELL_H);
    const a = ctx.getImageData(0, 0, CELL_W, CELL_H).data;
    for (let y = 0; y < CELL_H; y++) {
      for (let x = 0; x < CELL_W; x++) {
        if (a[(y * CELL_W + x) * 4 + 3] > 16) {
          top = Math.min(top, y);
          bottom = Math.max(bottom, y + 1);
          break;
        }
      }
    }
  }
  return top < bottom ? { top, bottom } : null;
}

// A cell whose alpha channel is entirely 0 counts as empty.
function cellIsEmpty(data, sheetWidth, col, row) {
  const x0 = col * CELL_W;
  const y0 = row * CELL_H;
  for (let y = y0; y < y0 + CELL_H; y++) {
    let i = (y * sheetWidth + x0) * 4 + 3;
    for (let x = 0; x < CELL_W; x++, i += 4) {
      if (data[i] !== 0) return false;
    }
  }
  return true;
}
