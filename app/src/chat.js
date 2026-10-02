// Chat bubble: open/close (the window grows upward), send messages, new chat, hand off to the terminal.

const { invoke } = window.__TAURI__.core;
const { PhysicalSize, PhysicalPosition } = window.__TAURI__.dpi;
const { currentMonitor } = window.__TAURI__.window;

const PET_W = 192;
const PET_H = 208;
const PANEL_W = 320;
const PANEL_H = 250;
// Gap between the tip of the bubble's tail and the character (px).
const GAP = 2;
// One closing rule: while the bubble is open, it closes when the window is not focused. The blur
// event alone is not reliable (once focus is gone, no second blur arrives), so focus is also polled.
// The only exceptions are timed: it stays open while waiting for a reply and until keepOpen expires.
// Messages are not lost; clicking the pet shows them again.
const FOCUS_POLL_MS = 500;
// A focus loss right after opening should not close it (can happen while the window resizes).
const BLUR_GRACE_MS = 400;
// Keep the bubble open while the right-click menu is open (the menu may take focus).
const MENU_HOLD_MS = 5000;
// Leave time to read the handoff note after "Open in terminal".
const HANDOFF_HOLD_MS = 5000;
// Leave time to read a reply that arrived in the background (while another window had focus).
const UNFOCUSED_REPLY_MS = 20000;

const $ = (id) => document.getElementById(id);

export class Chat {
  /**
   * hooks: { onBusy(), onReply(isError), log(line) } for the pet animation.
   */
  constructor(win, hooks) {
    this.win = win;
    this.hooks = hooks;
    this.open = false;
    this.busy = false;
    this.resizing = false;
    this.session = null;

    $("form").addEventListener("submit", (e) => {
      e.preventDefault();
      this.send();
    });
    $("input").addEventListener("input", (e) => autoGrow(e.target));
    $("input").addEventListener("keydown", (e) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        this.send();
      }
    });
    $("close").addEventListener("click", () => this.toggle(false));
    $("new-chat").addEventListener("click", () => this.newChat());
    $("to-terminal").addEventListener("click", () => this.toTerminal());
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && this.open) this.toggle(false);
    });
    // Close on a click in the transparent area outside the bubble (a pet click toggles on its own).
    document.addEventListener("mousedown", (e) => {
      if (!this.open || e.button !== 0) return;
      if (e.target.closest("#bubble") || e.target.closest("#pet")) return;
      this.toggle(false);
    });
    // Close on a click outside the window (desktop, another window); see FOCUS_POLL_MS.
    window.addEventListener("blur", () => this.closeIfUnfocused());
    this.focusTimer = null;
    this.holdUntil = 0;
    this.bounds = { top: 0, bottom: PET_H };
    this.home = null;
    invoke("user_home")
      .then((h) => {
        this.home = h;
        if (this.open) this.renderSession();
      })
      .catch(() => {});
  }

  /** Keep the bubble open for `ms` even without focus; after that the normal rule applies. */
  keepOpen(ms = MENU_HOLD_MS) {
    this.holdUntil = Math.max(this.holdUntil, Date.now() + ms);
  }

  closeIfUnfocused() {
    if (!this.open || this.busy || document.hasFocus()) return;
    if (Date.now() < this.holdUntil || Date.now() - this.openedAt < BLUR_GRACE_MS) return;
    this.toggle(false);
  }

  /** Visible bounds of the character within its cell; the bubble snaps to them. */
  setBounds(bounds) {
    if (bounds) this.bounds = bounds;
  }

  /**
   * Grows/shrinks the window so the pet stays in place on screen.
   * The bubble opens above if there is room, otherwise below; the window stays in the work area.
   * Math is done in physical pixels (so scale rounding does not shift anything).
   */
  async resize(open) {
    const scale = await this.win.scaleFactor();
    const pos = await this.win.outerPosition();
    const px = (v) => Math.round(v * scale);
    const pet = $("pet");

    const panel = $("panel");

    if (!open) {
      // The pet's current screen position (it may have been dragged while the bubble was open).
      const x = pos.x + px(this.petOffset.x);
      const y = pos.y + px(this.petOffset.y);
      await this.win.setSize(new PhysicalSize(px(PET_W), px(PET_H)));
      await this.win.setPosition(new PhysicalPosition(x, y));
      pet.style.left = pet.style.top = "";
      document.body.classList.remove("chat-open", "below");
      return;
    }

    // The bubble overlaps the cell's transparent margin; the tail snaps to the character.
    const overAbove = Math.max(0, this.bounds.top - GAP);
    const overBelow = Math.max(0, PET_H - this.bounds.bottom - GAP);
    const area = (await currentMonitor())?.workArea;
    let below = !!area && pos.y - px(PANEL_H - overAbove) < area.position.y;
    const over = below ? overBelow : overAbove;
    const w = px(PANEL_W);
    const h = px(PET_H + PANEL_H - over);
    let x = pos.x - px((PANEL_W - PET_W) / 2);
    let y = below ? pos.y : pos.y - px(PANEL_H - over);
    if (area) {
      x = Math.min(Math.max(x, area.position.x), area.position.x + area.size.width - w);
      y = Math.min(Math.max(y, area.position.y), area.position.y + area.size.height - h);
    }
    // Place the pet inside the window where it lines up with its old screen position.
    this.petOffset = { x: (pos.x - x) / scale, y: (pos.y - y) / scale };
    pet.style.left = `${this.petOffset.x}px`;
    pet.style.top = `${this.petOffset.y}px`;
    panel.style.top = `${below ? this.petOffset.y + PET_H - over : this.petOffset.y - (PANEL_H - over)}px`;
    panel.style.height = `${PANEL_H}px`;
    // Point the bubble's tail at the pet's center (the bubble's left edge is inset 6 px).
    document.body.style.setProperty("--tail-x", `${this.petOffset.x + PET_W / 2 - 6}px`);
    document.body.classList.add("chat-open");
    document.body.classList.toggle("below", below);
    await this.win.setPosition(new PhysicalPosition(x, y));
    await this.win.setSize(new PhysicalSize(w, h));
  }

  async toggle(force) {
    const next = force ?? !this.open;
    if (next === this.open || this.resizing) return;
    this.resizing = true;
    try {
      if (next) {
        this.session = await invoke("chat_status");
        this.renderSession();
        await this.resize(true);
        $("panel").hidden = false;
        this.openedAt = Date.now();
        $("input").focus();
        clearInterval(this.focusTimer);
        this.focusTimer = setInterval(() => this.closeIfUnfocused(), FOCUS_POLL_MS);
      } else {
        clearInterval(this.focusTimer);
        this.focusTimer = null;
        $("panel").hidden = true;
        await this.resize(false);
      }
      this.open = next;
    } catch (e) {
      this.hooks.log(`bubble failed to open: ${e}`);
    } finally {
      this.resizing = false;
    }
  }

  renderSession() {
    const cwd = this.session?.cwd ?? "";
    const folder = $("folder");
    folder.textContent = shortPath(cwd, this.home);
    folder.title = cwd;
    $("to-terminal").disabled = this.busy || !this.session?.sessionId;
    $("send").disabled = this.busy || this.needsTrust();
    this.renderTrust();
  }

  /** Whether the trust question for the new chat's folder is still unanswered. */
  needsTrust() {
    return !!this.session && this.session.trusted == null;
  }

  /** Like Claude Code's "do you trust this folder" question; asked once before the first message. */
  renderTrust() {
    const existing = $("trust");
    if (!this.needsTrust()) {
      existing?.remove();
      return;
    }
    const cwd = this.session.cwd ?? "";
    // Redraw the box if the folder or home directory changed (e.g. the home directory arrived late).
    const key = `${cwd}|${this.home ?? ""}`;
    if (existing?.dataset.key === key) return;
    existing?.remove();
    const box = document.createElement("div");
    box.id = "trust";
    box.dataset.key = key;
    box.className = "msg trust";
    const line = (cls, text) => {
      const d = document.createElement("div");
      if (cls) d.className = cls;
      d.textContent = text;
      box.append(d);
    };
    line("q", "Trust this folder?");
    line("path", cwd);
    line(
      "",
      "Trust: the pet can read files in this folder (but not change them), and web search is off. " +
        "Don't trust: the pet stays out of this folder and can only search the web."
    );
    // The home directory itself or a folder containing it (e.g. C:\): keys could be read too.
    if (pathRest(this.home, cwd) !== null) {
      line("warn", "Careful: this folder contains your home folder. Keys and personal files could be read too.");
    }
    const buttons = document.createElement("div");
    buttons.className = "buttons";
    for (const [label, trust] of [["Trust", true], ["Don't trust", false]]) {
      const b = document.createElement("button");
      b.type = "button";
      b.textContent = label;
      b.addEventListener("click", () => this.answerTrust(trust));
      buttons.append(b);
    }
    box.append(buttons);
    $("messages").append(box);
    box.scrollIntoView({ block: "end" });
  }

  async answerTrust(trust) {
    // One answer only: a second click (or the other button) is blocked until the reply arrives.
    const buttons = [...document.querySelectorAll("#trust button")];
    if (buttons.some((b) => b.disabled)) return;
    buttons.forEach((b) => (b.disabled = true));
    try {
      this.session = await invoke("chat_trust", { trust });
      this.add(
        "note",
        this.session.trusted
          ? "Folder trusted: I can read files here. Web search is off."
          : "Web search only. I won't touch this folder."
      );
      this.renderSession();
      $("input").focus();
    } catch (e) {
      buttons.forEach((b) => (b.disabled = false));
      this.add("pet error", String(e));
    }
  }

  add(kind, text) {
    const el = document.createElement("div");
    el.className = `msg ${kind}`;
    el.textContent = text;
    $("messages").append(el);
    el.scrollIntoView({ block: "end" });
    return el;
  }

  /** Pet reply: the text, plus a collapsed "Sources" list below it if there are sources. */
  addReply(text) {
    // A trusted (file-reading) chat has no web search and therefore no sources: links in the reply
    // are not made clickable (so content read from files cannot be smuggled out via a link).
    if (this.session?.trusted) {
      this.add("pet", text);
      return;
    }
    const { body, sources } = splitSources(text);
    const el = this.add("pet", body || "(empty reply)");
    if (!sources.length) return;
    const details = document.createElement("details");
    details.className = "sources";
    const summary = document.createElement("summary");
    summary.textContent = `Sources (${sources.length})`;
    details.append(summary);
    for (const s of sources) {
      // No href: every open (click, keyboard) goes through the URL check in open_url; a middle click
      // or drag never falls through to the WebView's own navigation.
      const a = document.createElement("a");
      a.setAttribute("role", "link");
      a.tabIndex = 0;
      // Always show the real domain, so a title cannot point somewhere else.
      const title = document.createElement("span");
      title.textContent = s.title;
      const host = document.createElement("span");
      host.className = "host";
      host.textContent = hostOf(s.url);
      a.append(title, host);
      a.title = s.url;
      const open = () =>
        invoke("open_url", { url: s.url }).catch((err) => this.hooks.log(`link failed to open: ${err}`));
      a.addEventListener("click", open);
      a.addEventListener("keydown", (e) => e.key === "Enter" && open());
      details.append(a);
    }
    // Keep the list in view when expanded.
    details.addEventListener("toggle", () => details.open && details.scrollIntoView({ block: "end" }));
    el.append(details);
  }

  setBusy(busy) {
    this.busy = busy;
    $("new-chat").disabled = busy;
    this.renderSession();
  }

  async send() {
    const input = $("input");
    const text = input.value.trim();
    if (!text || this.busy) return;
    if (this.needsTrust()) {
      $("trust")?.scrollIntoView({ block: "end" });
      return;
    }
    input.value = "";
    this.add("user", text);
    autoGrow(input);
    const waiting = this.add("busy", "✻ Thinking…");
    this.setBusy(true);
    this.hooks.onBusy();
    try {
      const reply = await invoke("chat_send", { message: text });
      this.session = reply.session;
      waiting.remove();
      if (reply.newSession) this.add("note", "Couldn't find the previous chat, so a new one started.");
      if (reply.isError) this.add("pet error", reply.text || "(empty reply)");
      else this.addReply(reply.text || "(empty reply)");
      this.hooks.onReply(reply.isError);
    } catch (e) {
      waiting.remove();
      this.add("pet error", String(e));
      this.hooks.onReply(true);
    } finally {
      // If the reply arrived in the background, keep it readable for a while, then the normal rule applies (close if unfocused).
      if (!document.hasFocus()) this.keepOpen(UNFOCUSED_REPLY_MS);
      this.setBusy(false);
      input.focus();
    }
  }

  async newChat() {
    if (this.busy) return;
    this.session = await invoke("chat_new");
    $("messages").replaceChildren();
    this.add("note", "New chat");
    this.renderSession();
  }

  async toTerminal() {
    if (this.busy || !this.session?.sessionId) return;
    // The new terminal takes focus; the bubble stays open for a while so the handoff note is visible.
    // The timer restarts after the note is written (so it is readable even if the terminal opens slowly).
    this.keepOpen(HANDOFF_HOLD_MS);
    try {
      await invoke("chat_open_terminal");
      // The pet no longer writes to the handed-off session; the next message starts a new one.
      this.session = await invoke("chat_status");
      $("messages").replaceChildren();
      this.add("note", "Chat moved to the terminal. Your next message starts a new chat.");
      this.renderSession();
      this.keepOpen(HANDOFF_HOLD_MS);
    } catch (e) {
      this.add("pet error", String(e));
    }
  }
}

/**
 * If `p` is `base` or inside it, returns the folder names after `base` ([] = base itself),
 * otherwise null. Windows paths: case and separator differences are ignored. Folder names are
 * compared one by one (not by character offset: lowercasing changes the length of letters like "İ").
 */
export function pathRest(p, base) {
  if (!p || !base) return null;
  const split = (s) => s.split(/[\\/]+/).filter(Boolean);
  const pp = split(p);
  const bp = split(base);
  if (pp.length < bp.length) return null;
  const same = bp.every((seg, i) => seg.toLowerCase() === pp[i].toLowerCase());
  return same ? pp.slice(bp.length) : null;
}

/** Short path like the CLI: the user's home directory as "~", only the last two folders if long. */
export function shortPath(p, home) {
  if (!p) return "?";
  const rest = pathRest(p, home);
  if (rest) {
    return rest.length <= 2 ? ["~", ...rest].join("\\") : `~\\…\\${rest.slice(-2).join("\\")}`;
  }
  const parts = p.split(/[\\/]/).filter(Boolean);
  return parts.length <= 2 ? p : `…\\${parts.slice(-2).join("\\")}`;
}

// URL: no whitespace; may contain one level of parentheses (e.g. Wikipedia's "Mercury_(planet)").
// `<` and `>` are not part of the URL: in "<https://…>" the angle brackets stay outside.
const URL_RE = String.raw`https?:\/\/(?:[^\s()<>]|\([^\s()<>]*\))+`;
const LINK_LINE = new RegExp(String.raw`^\s*(?:[-*•]\s*)?\[([^\]]+)\]\((${URL_RE})\)\s*$`);
const URL_LINE = new RegExp(String.raw`^\s*(?:[-*•]\s*)?<?(${URL_RE})>?\s*$`);
const INLINE_LINK = new RegExp(String.raw`\[([^\]]+)\]\((${URL_RE})\)`, "g");
// Only known source headings ("Sources:", "**Sources:**"), not every line ending in ":".
// Turkish headings (kaynaklar, kaynak, referanslar) are kept on purpose: the model may answer in Turkish.
const SOURCES_HEADING = /^\s*\**\s*(kaynaklar|kaynak|sources?|references|referanslar)\s*:?\s*\**\s*$/i;

/**
 * Splits the link lines at the end of a reply (and the short heading right above them) into sources.
 * Inline [title](url) links keep only the title in the text; the URL is added to the sources.
 * Returns: { body, sources: [{ title, url }] }
 */
export function splitSources(text) {
  const lines = text.replace(/\r/g, "").split("\n");
  const sources = [];
  let end = lines.length;
  while (end > 0) {
    const line = lines[end - 1];
    let m;
    if (!line.trim()) {
      end--;
    } else if ((m = line.match(LINK_LINE))) {
      sources.unshift({ title: m[1].trim(), url: m[2] });
      end--;
    } else if ((m = line.match(URL_LINE))) {
      sources.unshift({ title: hostOf(m[1]), url: m[1] });
      end--;
    } else {
      break;
    }
  }
  if (sources.length && end > 0 && SOURCES_HEADING.test(lines[end - 1])) end--;
  // If the reply itself is just a link (e.g. "what's the repo URL?"), keep it as text, not collapsed.
  if (!lines.slice(0, end).join("").trim()) {
    return { body: text.replace(/\*\*(.+?)\*\*/g, "$1").trim(), sources: [] };
  }

  const seen = new Set(sources.map((s) => s.url));
  const body = lines
    .slice(0, end)
    .join("\n")
    .replace(INLINE_LINK, (_, title, url) => {
      if (!seen.has(url)) {
        seen.add(url);
        sources.push({ title: title.trim(), url });
      }
      return title;
    })
    .replace(/\*\*(.+?)\*\*/g, "$1")
    .trim();
  return { body, sources };
}

export function hostOf(url) {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return url;
  }
}

/** The message input grows as you type, up to 3 lines. */
function autoGrow(el) {
  el.style.height = "20px";
  el.style.height = `${Math.min(el.scrollHeight, 60)}px`;
}
