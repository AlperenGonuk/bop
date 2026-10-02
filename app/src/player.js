// Animation player: one canvas, one animation at a time.
import { CELL_W, CELL_H } from "./sprites.js";

export const DEFAULT_FRAME_MS = 150;

export class Player {
  constructor(canvas) {
    this.canvas = canvas;
    this.ctx = canvas.getContext("2d");
    this.timer = null;
    this.current = null;
    const dpr = window.devicePixelRatio || 1;
    canvas.width = Math.round(CELL_W * dpr);
    canvas.height = Math.round(CELL_H * dpr);
    this.ctx.imageSmoothingQuality = "high";
  }

  /**
   * anim: { image, frames: [{x, y}], frameMs }
   * With loop=false it stops on the last frame and calls onDone.
   */
  play(name, anim, { loop = true, onDone } = {}) {
    if (!anim?.frames?.length) return false;
    if (loop && this.current === name) return true;
    this.stop();
    this.current = name;
    let i = 0;
    const tick = () => {
      this.draw(anim.image, anim.frames[i]);
      i++;
      if (i >= anim.frames.length) {
        if (!loop) {
          this.stop();
          onDone?.();
          return;
        }
        i = 0;
      }
      this.timer = setTimeout(tick, anim.frameMs ?? DEFAULT_FRAME_MS);
    };
    tick();
    return true;
  }

  /** Shows a single frame (e.g. looking at the cursor); stops the loop. */
  still(key, image, frame) {
    if (this.current === key) return;
    this.stop();
    this.current = key;
    this.draw(image, frame);
  }

  draw(image, f) {
    const { ctx, canvas } = this;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.drawImage(image, f.x, f.y, CELL_W, CELL_H, 0, 0, canvas.width, canvas.height);
  }

  stop() {
    clearTimeout(this.timer);
    this.timer = null;
    this.current = null;
  }
}
