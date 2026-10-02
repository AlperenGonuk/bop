// State → animation resolution. The vocabulary and parent-state chain come from Rust (state.rs).

// Default mapping onto Codex rows when there is no bop.json (DECISIONS.md, decision 8).
export const CODEX_DEFAULT_STATES = {
  idle: "idle",
  running: "running",
  thinking: "review",
  "waiting-permission": "waiting",
  done: "jumping",
  failed: "failed",
};

// States that play once and then return to idle.
export const ONE_SHOT = new Set(["done", "failed"]);

// A "working" state this old (e.g. from a crashed session) counts as idle.
export const STALE_MS = 10 * 60 * 1000;

/**
 * Turns a state into the animation to play.
 * mapping: { state: animationName }, animations: { animationName: {...} }, parents: Map(state → parent).
 * If the pet does not define the state or lacks its animation, falls back to the parent; finally "idle".
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
