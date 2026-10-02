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

// Sürükleme eşiği (piksel): altında kalan hareket tıklama sayılır.
const DRAG_THRESHOLD = 4;
// Pencere bu kadar süre kıpırdamazsa sürükleme bitti sayılır.
const DRAG_SETTLE_MS = 220;

let pet = null;
let idle = null;

function log(line) {
  console.info(`[bop] ${line}`);
  invoke("log", { line }).catch(() => {});
}

function showError(message) {
  log(`hata: ${message}`);
  const box = document.getElementById("error");
  box.textContent = message;
  box.hidden = false;
}

function play(name, opts) {
  return player.play(name, pet?.animations[name], opts);
}

// --- Claude durumu -------------------------------------------------------

let parents = new Map();
let clawdState = "idle";

/** Mevcut Claude durumunun animasyonunu oynatır (sürükleme ya da kısa hareket sırasında değil). */
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
  log(`durum: ${record?.event ?? "-"} → ${next}`);
  setState(next);
}

async function setupStates() {
  parents = new Map(await invoke("state_vocabulary"));
  await listen("pet-state", ({ payload }) => applyRecord(payload));
  const initial = await invoke("current_state");
  // Açılışta bir kerelik durumları (done/failed) tekrar oynatma.
  if (initial && !ONE_SHOT.has(initial.state)) applyRecord(initial);
  // Uzun süre güncellenmeyen "çalışıyor" durumu idle'a döner.
  setInterval(async () => {
    if (clawdState === "idle") return;
    const rec = await invoke("current_state");
    if (isStale(rec)) applyRecord(rec);
  }, 30 * 1000);
}

// --- Sürükle-bırak -------------------------------------------------------

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
    // İşletim sisteminin taşıma döngüsü; bırakılana kadar fare olayları gelmez.
    win.startDragging().catch((err) => log(`sürükleme başlamadı: ${err}`));
  });

  // Eşiği geçmeyen sol tık: sohbet balonunu aç/kapa.
  canvas.addEventListener("mouseup", (e) => {
    if (e.button === 0 && down && !dragging) chat.toggle();
    down = null;
  });
  window.addEventListener("mouseup", () => {
    down = null;
  });

  // Yön ve bitiş pencerenin konum olaylarından çıkarılır.
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
  // Petin ekrandaki sol üst köşesini hatırla (balon açıkken de pet konumu).
  try {
    const pos = await win.outerPosition();
    const scale = await win.scaleFactor();
    const r = canvas.getBoundingClientRect();
    await invoke("save_position", {
      x: Math.round(pos.x + r.left * scale),
      y: Math.round(pos.y + r.top * scale),
    });
  } catch (e) {
    log(`konum kaydedilemedi: ${e}`);
  }
}

// --- Sohbet --------------------------------------------------------------

const chat = new Chat(win, {
  onBusy: () => setState("thinking"),
  onReply: (isError) => setState(isError ? "failed" : "idle"),
  log,
});

// --- Sağ tık menüsü ------------------------------------------------------

function setupMenu() {
  window.addEventListener("contextmenu", (e) => {
    e.preventDefault();
    // Yerel menü odağı alabilir; açık sohbet balonu bu yüzden kapanmasın.
    chat.keepOpen();
    invoke("show_menu").catch((err) => log(`menü açılmadı: ${err}`));
  });
}

// --- Başlangıç -----------------------------------------------------------

async function start() {
  setupMenu();
  // Pet değişti (sağ tık menüsü ya da `bop use`): balonu kapatıp sayfayı baştan kur.
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
    log(`${pet.name} v${pet.version} bakış=${pet.hasLook} kareler=${JSON.stringify(counts)}`);
    log(`boşta hareketler=${JSON.stringify(pet.idleExtras)} ayar=${info.config?.present ? "var" : "yok"}`);
    if (info.spriteVersionNumber && info.spriteVersionNumber !== pet.version) {
      pet.warnings.push(
        `pet.json spriteVersionNumber=${info.spriteVersionNumber}, görsel v${pet.version}; görsele göre devam ediliyor`
      );
    }
    for (const w of pet.warnings) log(`uyarı: ${w}`);

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
