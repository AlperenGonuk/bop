// Durum → animasyon çözümleme. Sözlük ve üst durum zinciri Rust'tan gelir (state.rs).

// bop.json yokken Codex satırlarıyla varsayılan eşleme (KARARLAR.md, 8. karar).
export const CODEX_DEFAULT_STATES = {
  idle: "idle",
  running: "running",
  thinking: "review",
  "waiting-permission": "waiting",
  done: "jumping",
  failed: "failed",
};

// Bir kez oynayıp idle'a dönen durumlar.
export const ONE_SHOT = new Set(["done", "failed"]);

// Bu kadar eski bir "çalışıyor" durumu (ör. çöken oturum) idle sayılır.
export const STALE_MS = 10 * 60 * 1000;

/**
 * Durumu oynatılacak animasyona çevirir.
 * mapping: { durum: animasyonAdı }, animations: { animasyonAdı: {...} }, parents: Map(durum → üst).
 * Pet durumu tanımlamıyorsa ya da animasyonu yoksa üst duruma düşer; en sonda "idle".
 */
export function resolveAnimation(state, mapping, animations, parents) {
  const seen = new Set();
  let cur = state;
  while (cur && !seen.has(cur)) {
    seen.add(cur);
    const anim = mapping[cur];
    if (anim && animations[anim]?.frames?.length) return anim;
    cur = parents.get(cur);
  }
  return animations.idle ? "idle" : null;
}

export function isStale(record, now = Date.now()) {
  return !record || now - record.updated_at > STALE_MS;
}
