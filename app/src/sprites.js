// Codex spritesheet okuyucu. Ayrıntı: docs/SPRITE.md

export const CELL_W = 192;
export const CELL_H = 208;
export const COLS = 8;
// v2 nötr ön poz hücresi: satır 0, sütun 6.
export const NEUTRAL_COL = 6;

// Sürüm görsel boyutundan anlaşılır (KARARLAR.md, 3. karar).
const VERSIONS = [
  { version: 1, width: 1536, height: 1872, rows: 9 },
  { version: 2, width: 1536, height: 2288, rows: 11 },
];

// Codex satır adları. look satırları yalnız v2'de var.
export const ROW_NAMES = [
  "idle", "running-right", "running-left", "waving", "jumping",
  "failed", "waiting", "running", "review", "look-a", "look-b",
];

export function detectVersion(width, height) {
  return VERSIONS.find((v) => v.width === width && v.height === height) ?? null;
}

/** Görseli URL'den yükler (tarayıcı nesnesi olarak). */
export async function loadImage(url) {
  const img = new Image();
  // Asset protokolü pencere origin'ine CORS izni verir; canvas okunabilir kalır.
  img.crossOrigin = "anonymous";
  img.src = url;
  await img.decode();
  return img;
}

/** Görseli yükler, sürümü bulur ve her satırın dolu karelerini çıkarır. */
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
  // v2'de (0,6) hücresi idle karesi değil, nötr ön pozdur (Codex `neutralLookFrame`,
  // docs/SPRITE.md). idle döngüsünden çıkarılır.
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
    // v1'de bakış satırı yok: fareye bakma sessizce kapalı.
    hasLook,
    neutralFrame,
    // 16 yön, 0° yukarı, saat yönünde 22.5° adım (satır 9: 0-157.5°, satır 10: 180-337.5°; 180° = aşağı).
    lookFrames: hasLook
      ? Array.from({ length: 16 }, (_, i) => ({ x: (i % COLS) * CELL_W, y: (9 + Math.floor(i / COLS)) * CELL_H }))
      : [],
    animations,
  };
}

/**
 * Ek sheet (bop.json): aynı ızgara, hücre 192×208, en fazla 8 sütun, satır sayısı serbest.
 * Uymazsa hata fırlatır; çağıran uyarıya çevirip atlar.
 */
export async function loadExtraSheet(url) {
  const img = await loadImage(url);
  const { naturalWidth: w, naturalHeight: h } = img;
  if (w % CELL_W || h % CELL_H || w / CELL_W > COLS || !w || !h) {
    throw new Error(`grid does not fit (${w}×${h}; cell ${CELL_W}×${CELL_H}, at most ${COLS} columns)`);
  }
  return { image: img, rows: rowFrames(img, h / CELL_H) };
}

/** Her satırın dolu karelerini döner: [[{x, y}, ...], ...] */
export function rowFrames(img, rowCount) {
  const canvas = document.createElement("canvas");
  canvas.width = img.naturalWidth;
  canvas.height = img.naturalHeight;
  const ctx = canvas.getContext("2d", { willReadFrequently: true });
  ctx.drawImage(img, 0, 0);
  const alpha = ctx.getImageData(0, 0, canvas.width, canvas.height).data;

  // Ek sheet'ler 8 sütundan dar olabilir; görsel dışındaki hücre taranmaz.
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
 * Karelerdeki görünen alanın hücre içindeki dikey sınırları: { top, bottom } (hücre pikseli).
 * Balonu karakterin hemen üstüne/altına yanaştırmak için (hücrenin saydam kenarı boşluk yapmasın).
 * Hiç belirgin piksel yoksa (ör. çok soluk kareler) null: çağıran bu görseli atlar.
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

// Alfa kanalı tamamen 0 olan hücre boş sayılır.
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
