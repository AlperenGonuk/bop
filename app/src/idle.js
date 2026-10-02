// Boştayken pet: imlece bakma (yalnız v2) ve arada kısa hareketler (idleExtras).
import { lookIndex } from "./pet.js";

const { invoke } = window.__TAURI__.core;

const LOOK_TICK_MS = 120;
// İmleç bu yarıçap (mantıksal px) içindeyse bakar; petin üstündeyse (ölü bölge) nötr ön pozu
// gösterir (docs/SPRITE.md, (0,6) nötr hücre).
const LOOK_RADIUS = 450;
const LOOK_DEADZONE = 60;
// Kısa hareketler arası bekleme (ms).
const EXTRA_MIN_MS = 15_000;
const EXTRA_MAX_MS = 30_000;

export class IdleLife {
  /**
   * opts: { pet, player, canvas, canAct(): bool, play(name, opts), release() }
   * canAct: pet boşta mı (idle durum, sürükleme yok, sohbet beklemiyor).
   * release: bakış/hareket bitince mevcut durumun animasyonuna dön.
   */
  constructor(opts) {
    Object.assign(this, opts);
    this.tracking = false;
    this.extraPlaying = false;
  }

  start() {
    if (this.pet.hasLook) setInterval(() => this.lookTick(), LOOK_TICK_MS);
    if (this.pet.idleExtras.length) this.scheduleExtra();
  }

  busy() {
    return this.extraPlaying;
  }

  async lookTick() {
    if (!this.canAct() || this.extraPlaying) {
      this.tracking = false;
      return;
    }
    let info;
    try {
      info = await invoke("cursor_info");
    } catch {
      return;
    }
    const scale = window.devicePixelRatio || 1;
    const r = this.canvas.getBoundingClientRect();
    const cx = info.windowX + (r.left + r.width / 2) * scale;
    const cy = info.windowY + (r.top + r.height / 2) * scale;
    const dx = (info.cursorX - cx) / scale;
    const dy = (info.cursorY - cy) / scale;
    const dist = Math.hypot(dx, dy);
    if (!this.canAct()) return; // istek sürerken durum değişmiş olabilir
    if (dist > LOOK_DEADZONE && dist < LOOK_RADIUS) {
      const i = lookIndex(dx, dy);
      this.tracking = true;
      this.player.still(`look-${i}`, this.pet.image, this.pet.lookFrames[i]);
    } else if (dist <= LOOK_DEADZONE && this.pet.neutralFrame) {
      this.tracking = true;
      this.player.still("look-neutral", this.pet.image, this.pet.neutralFrame);
    } else if (this.tracking) {
      this.tracking = false;
      this.release();
    }
  }

  scheduleExtra() {
    const wait = EXTRA_MIN_MS + Math.random() * (EXTRA_MAX_MS - EXTRA_MIN_MS);
    setTimeout(() => {
      if (this.canAct() && !this.tracking && !this.extraPlaying) this.playExtra();
      this.scheduleExtra();
    }, wait);
  }

  playExtra(name) {
    const list = this.pet.idleExtras;
    const pick = name ?? list[Math.floor(Math.random() * list.length)];
    this.extraPlaying = true;
    const ok = this.play(pick, {
      loop: false,
      onDone: () => {
        this.extraPlaying = false;
        this.release();
      },
    });
    if (!ok) this.extraPlaying = false;
    return ok;
  }

  /** Durum değişince yarım kalan kısa hareketi bırak. */
  interrupt() {
    this.extraPlaying = false;
    this.tracking = false;
  }
}
