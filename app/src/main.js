import { Player } from "./player.js";
import { ONE_SHOT, isStale, resolveAnimation } from "./states.js";
import { buildPet } from "./pet.js";
import { IdleLife } from "./idle.js";
import { Chat } from "./chat.js";

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const { listen } = window.__TAURI__.event;

const win = getCurrentWindow();
const canvas = document.getElementById("pet");
const player = new Player(canvas);

// Drag threshold (px): movement below it counts as a click.
const DRAG_THRESHOLD = 4;
// The drag is considered over once the window stays still this long.
const DRAG_SETTLE_MS = 220;

let pet = null;
let idle = null;

function log(line) {
  console.info(`[bop] ${line}`);
  invoke("log", { line }).catch(() => {});
}

function showError(message) {
  log(`error: ${message}`);
  const box = document.getElementById("error");
  box.textContent = message;
  box.hidden = false;
}

function play(name, opts) {
  return player.play(name, pet?.animations[name], opts);
}

// --- Claude state --------------------------------------------------------

let parents = new Map();
let clawdState = "idle";

/** Plays the animation for the current Claude state (not while dragging or during a short move). */
function showState() {
  if (dragging || !pet || idle?.busy()) return;
  const anim = resolveAnimation(clawdState, pet.stateMapping, pet.animations, parents);
  if (!anim) return;
  if (ONE_SHOT.has(clawdState)) {
    play(anim, {
      loop: false,
      onDone: () => {
        clawdState = "idle";
        showState();
      },
    });
  } else {
    play(anim);
  }
}

function setState(name) {
  clawdState = name;
  idle?.interrupt();
  showState();
}

function applyRecord(record) {
  const next = isStale(record) ? "idle" : record.state;
  if (next === clawdState && !ONE_SHOT.has(next)) return;
  log(`state: ${record?.event ?? "-"} → ${next}`);
  setState(next);
}

async function setupStates() {
  parents = new Map(await invoke("state_vocabulary"));
  await listen("pet-state", ({ payload }) => applyRecord(payload));
  const initial = await invoke("current_state");
  // Do not replay one-shot states (done/failed) on startup.
  if (initial && !ONE_SHOT.has(initial.state)) applyRecord(initial);
  // A "working" state that has not been updated for a long time falls back to idle.
  setInterval(async () => {
    if (clawdState === "idle") return;
    const rec = await invoke("current_state");
    if (isStale(rec)) applyRecord(rec);
  }, 30 * 1000);
}

// --- Drag and drop -------------------------------------------------------

let dragging = false;
let settleTimer = null;
let lastX = null;

function setupDrag() {
  let down = null;

  canvas.addEventListener("mousedown", (e) => {
    if (e.button !== 0) return;
    down = { x: e.screenX, y: e.screenY };
  });

  canvas.addEventListener("mousemove", (e) => {
    if (!down || dragging) return;
    if (Math.hypot(e.screenX - down.x, e.screenY - down.y) < DRAG_THRESHOLD) return;
    down = null;
    dragging = true;
    lastX = null;
    idle?.interrupt();
    // The OS move loop; no mouse events arrive until release.
    win.startDragging().catch((err) => log(`drag failed to start: ${err}`));
  });

  // A left click that stays under the threshold toggles the chat bubble.
  canvas.addEventListener("mouseup", (e) => {
    if (e.button === 0 && down && !dragging) chat.toggle();
    down = null;
  });
  window.addEventListener("mouseup", () => {
    down = null;
  });

  // Direction and end of the drag are derived from the window's move events.
  win.onMoved(({ payload }) => {
    if (!dragging) return;
    if (lastX !== null && payload.x !== lastX) {
      play(payload.x > lastX ? "running-right" : "running-left");
    }
    lastX = payload.x;
    clearTimeout(settleTimer);
    settleTimer = setTimeout(endDrag, DRAG_SETTLE_MS);
  });
}

async function endDrag() {
  dragging = false;
  showState();
  // Remember the pet's top-left corner on screen (the pet's position even while the bubble is open).
  try {
    const pos = await win.outerPosition();
    const scale = await win.scaleFactor();
    const r = canvas.getBoundingClientRect();
    await invoke("save_position", {
      x: Math.round(pos.x + r.left * scale),
      y: Math.round(pos.y + r.top * scale),
    });
  } catch (e) {
    log(`failed to save position: ${e}`);
  }
}

// --- Chat ----------------------------------------------------------------

const chat = new Chat(win, {
  onBusy: () => setState("thinking"),
  onReply: (isError) => setState(isError ? "failed" : "idle"),
  log,
});

// --- Right-click menu ----------------------------------------------------

function setupMenu() {
  window.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    // The native menu may take focus; don't let that close an open chat bubble.
    chat.keepOpen();
    invoke("show_menu").catch((err) => log(`menu failed to open: ${err}`));
  });
}

// --- Startup -------------------------------------------------------------

async function start() {
  setupMenu();
  // Pet changed (right-click menu or `bop use`): close the bubble and rebuild the page.
  await listen("pet-changed", async () => {
    if (chat.open) await chat.toggle(false);
    location.reload();
  });
  try {
    const info = await invoke("pet_info");
    pet = await buildPet(info);

    const counts = Object.fromEntries(
      Object.entries(pet.animations).map(([k, v]) => [k, v.frames.length])
    );
    log(`${pet.name} v${pet.version} look=${pet.hasLook} frames=${JSON.stringify(counts)}`);
    log(`idleExtras=${JSON.stringify(pet.idleExtras)} config=${info.config?.present ? "yes" : "no"}`);
    if (info.spriteVersionNumber && info.spriteVersionNumber !== pet.version) {
      pet.warnings.push(
        `pet.json spriteVersionNumber=${info.spriteVersionNumber}, image is v${pet.version}; going by the image`
      );
    }
    for (const w of pet.warnings) log(`warning: ${w}`);

    chat.setBounds(pet.bounds);
    setupDrag();
    idle = new IdleLife({
      pet,
      player,
      canvas,
      canAct: () => clawdState === "idle" && !dragging && !chat.busy,
      play,
      release: showState,
    });
    showState();
    await setupStates();
    idle.start();
  } catch (e) {
    showError(String(e?.message ?? e));
  }
}

start();
