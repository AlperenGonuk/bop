// Pet derleme: ana sheet + bop.json (ek sheet'ler, isimli animasyonlar,
// durum eşlemesi, boşta hareketler). Hatalı giriş atlanır ve uyarı olur; pet bozulmaz.
import { loadSheet, loadExtraSheet, contentBounds, CELL_H } from "./sprites.js";
import { CODEX_DEFAULT_STATES } from "./states.js";
import { DEFAULT_FRAME_MS } from "./player.js";

const { convertFileSrc } = window.__TAURI__.core;

// Etrafa bakma: nötr → sağ → nötr → sol → nötr → kısa yukarı süzme → nötr.
// Bakış indeksi i = i × 22.5°, 0° yukarı, 90° sağ, 270° sol (docs/SPRITE.md). "N" nötr pozdur
// (v2 (0,6) hücresi). Yalnız ön yarım daire (−90°…+90°): 180° eski petlerde arkadan görünüş.
const LOOK_AROUND = [
  "N", "N", 3, 4, 4, 4, 4, 3, "N",
  13, 12, 12, 12, 12, 13, "N",
  2, 1, 0, 0, 15, 14, "N", "N",
];
const LOOK_FRAME_MS = 160;

/**
 * info: pet_info komutunun çıktısı (config dahil).
 * Döner: { name, version, hasLook, lookFrames, neutralFrame, image, animations, stateMapping, idleExtras, warnings }
 */
export async function buildPet(info) {
  const config = info.config ?? {};
  const warnings = [...(config.warnings ?? [])];
  const sheet = await loadSheet(convertFileSrc(info.spritesheetFile));
  const frameMs = config.frameMs ?? DEFAULT_FRAME_MS;

  const animations = {};
  for (const [name, anim] of Object.entries(sheet.animations)) {
    if (name.startsWith("look-")) continue; // bakış satırları ayrıca: lookFrames
    animations[name] = { ...anim, frameMs };
  }

  // Ek sheet'ler.
  const extra = {};
  for (const [name, path] of Object.entries(config.sheets ?? {})) {
    try {
      extra[name] = await loadExtraSheet(convertFileSrc(path));
    } catch (e) {
      warnings.push(`sheets.${name}: ${e.message ?? e}, atlandı`);
    }
  }

  // İsimli animasyonlar (Codex satırlarını da ezebilir).
  for (const [name, def] of Object.entries(config.animations ?? {})) {
    const src =
      def.sheet === "main"
        ? { image: sheet.image, rows: mainRows(sheet) }
        : extra[def.sheet];
    if (!src) {
      warnings.push(`animations.${name}: '${def.sheet}' yüklenemedi, atlandı`);
      continue;
    }
    const frames = src.rows[def.row];
    if (!frames) {
      warnings.push(`animations.${name}: satır ${def.row} yok (${src.rows.length} satır), atlandı`);
      continue;
    }
    if (!frames.length) {
      warnings.push(`animations.${name}: satır ${def.row} boş, atlandı`);
      continue;
    }
    animations[name] = { image: src.image, frames, frameMs: def.frameMs ?? frameMs };
  }

  // Nötr ön poz: v2'de (0,6) hücresi; yoksa (v1 ya da boş hücre) idle'ın ilk karesi.
  const neutralFrame = sheet.neutralFrame ?? sheet.animations.idle?.frames[0] ?? null;

  // Etrafa bakma: yalnız v2 (v1'de sessizce yok). Nötr pozla başlar ve biter.
  if (sheet.hasLook && neutralFrame) {
    animations.look = {
      image: sheet.image,
      frames: LOOK_AROUND.map((i) => (i === "N" ? neutralFrame : sheet.lookFrames[i])),
      frameMs: LOOK_FRAME_MS,
    };
  }

  const stateMapping = { ...CODEX_DEFAULT_STATES, ...(config.states ?? {}) };
  for (const [state, anim] of Object.entries(config.states ?? {})) {
    if (!animations[anim]) warnings.push(`states.${state}: '${anim}' animasyonu yok, üst duruma düşülecek`);
  }

  const wanted = config.idleExtras ?? (sheet.hasLook ? ["look"] : []);
  const idleExtras = wanted.filter((name) => {
    if (animations[name]) return true;
    // v1'de "look" sessizce düşer (KARARLAR.md, 3. karar).
    if (name !== "look" || sheet.hasLook) warnings.push(`idleExtras: '${name}' animasyonu yok, atlandı`);
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
    // Karakterin hücre içindeki görünen sınırları; balon buna yanaşır. Tüm animasyonlar
    // (her biri kendi sheet'iyle) hesaba katılır: balon açıkken oynayan hiçbir kare örtülmesin.
    bounds: animationBounds(animations),
    warnings,
  };
}

// Bütün animasyonların görünen alanlarının birleşimi: { top, bottom } (hücre pikseli).
// Aynı hücre birden çok animasyonda geçebilir (ör. "look", "sheet": "main"); her hücre bir kez taranır.
function animationBounds(animations) {
  const cells = new Map(); // görsel → { "x,y": kare }
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
    if (!b) continue; // yalnız soluk kareler: balonun yerini etkilemesin
    top = Math.min(top, b.top);
    bottom = Math.max(bottom, b.bottom);
  }
  return top < bottom ? { top, bottom } : { top: 0, bottom: CELL_H };
}

// Ana sheet'in satırları (bop.json "sheet": "main" için), boş kareler atlanmış.
function mainRows(sheet) {
  return Object.values(sheet.animations).map((a) => a.frames);
}

/**
 * İmlece göre bakış karesi: 0° yukarı/ön, saat yönünde.
 * Yalnız ön yarım daire (-90°…+90°): Codex sözleşmesinde 180° aşağı bakıştır, ama eski petlerde
 * (ör. Johnny) arkadan görünüş çizilmiş; ön yarım daire ikisinde de doğru görünür.
 */
export function lookIndex(dx, dy) {
  let deg = (Math.atan2(dx, -dy) * 180) / Math.PI; // -180…180
  deg = Math.max(-90, Math.min(90, deg));
  return (Math.round(deg / 22.5) + 16) % 16;
}
