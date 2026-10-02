// Sohbet balonu: aç/kapa (pencere yukarı büyür), mesaj gönder, yeni sohbet, terminale devret.

const { invoke } = window.__TAURI__.core;
const { PhysicalSize, PhysicalPosition } = window.__TAURI__.dpi;
const { currentMonitor } = window.__TAURI__.window;

const PET_W = 192;
const PET_H = 208;
const PANEL_W = 320;
const PANEL_H = 250;
// Balon kuyruğunun ucu ile karakter arasında kalacak boşluk (px).
const GAP = 2;
// Kapanma kuralı tek: balon açıkken pencere odakta değilse kapanır. Olaya (blur) güvenilmez,
// çünkü odak bir kez gidince ikinci blur gelmez; odak düzenli aralıkla da yoklanır.
// İstisnalar yalnız süreli: cevap beklenirken ve keepOpen süresi dolana kadar kapanmaz.
// Mesajlar kaybolmaz; pete tıklayınca yeniden görünür.
const FOCUS_POLL_MS = 500;
// Açılıştan hemen sonraki odak kaybı kapatmasın (pencere boyutlanırken olabiliyor).
const BLUR_GRACE_MS = 400;
// Sağ tık menüsü açıkken balon açık kalsın (menü odağı alabilir).
const MENU_HOLD_MS = 5000;
// "Open in terminal" sonrası devir notu okunabilsin.
const HANDOFF_HOLD_MS = 5000;
// Cevap arka planda (başka pencere odaktayken) geldiyse okunabilsin.
const UNFOCUSED_REPLY_MS = 20000;

const $ = (id) => document.getElementById(id);

export class Chat {
  /**
   * hooks: { onBusy(), onReply(isError), log(line) } — pet animasyonu için.
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
    // Balonun dışındaki saydam alana tıklayınca kapan (pete tıklama kendi açar/kapar).
    document.addEventListener("mousedown", (e) => {
      if (!this.open || e.button !== 0) return;
      if (e.target.closest("#bubble") || e.target.closest("#pet")) return;
      this.toggle(false);
    });
    // Pencere dışına (masaüstü, başka pencere) tıklanınca kapan; ayrıntı FOCUS_POLL_MS'te.
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

  /** Balon `ms` boyunca odak dışındayken de açık kalsın; sonra normal kural geçerli. */
  keepOpen(ms = MENU_HOLD_MS) {
    this.holdUntil = Math.max(this.holdUntil, Date.now() + ms);
  }

  closeIfUnfocused() {
    if (!this.open || this.busy || document.hasFocus()) return;
    if (Date.now() < this.holdUntil || Date.now() - this.openedAt < BLUR_GRACE_MS) return;
    this.toggle(false);
  }

  /** Karakterin hücre içindeki görünen sınırları; balon buna yanaşır. */
  setBounds(bounds) {
    if (bounds) this.bounds = bounds;
  }

  /**
   * Pencereyi pet ekranda yerinde kalacak şekilde büyütür/küçültür.
   * Balon yukarıda yer varsa üstte, yoksa altta açılır; pencere çalışma alanında kalır.
   * Hesap fiziksel pikselle yapılır (ölçek yuvarlaması kaydırmasın).
   */
  async resize(open) {
    const scale = await this.win.scaleFactor();
    const pos = await this.win.outerPosition();
    const px = (v) => Math.round(v * scale);
    const pet = $("pet");

    const panel = $("panel");

    if (!open) {
      // Petin şu anki ekran konumu (balon açıkken sürüklenmiş olabilir).
      const x = pos.x + px(this.petOffset.x);
      const y = pos.y + px(this.petOffset.y);
      await this.win.setSize(new PhysicalSize(px(PET_W), px(PET_H)));
      await this.win.setPosition(new PhysicalPosition(x, y));
      pet.style.left = pet.style.top = "";
      document.body.classList.remove("chat-open", "below");
      return;
    }

    // Hücrenin saydam kenarı kadar balon pete biner; kuyruk karaktere yanaşır.
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
    // Pet pencere içinde, ekrandaki eski yerine denk gelecek noktada.
    this.petOffset = { x: (pos.x - x) / scale, y: (pos.y - y) / scale };
    pet.style.left = `${this.petOffset.x}px`;
    pet.style.top = `${this.petOffset.y}px`;
    panel.style.top = `${below ? this.petOffset.y + PET_H - over : this.petOffset.y - (PANEL_H - over)}px`;
    panel.style.height = `${PANEL_H}px`;
    // Balon kuyruğu petin ortasını göstersin (balonun sol kenarı 6 px içeride).
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
      this.hooks.log(`balon açılamadı: ${e}`);
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

  /** Yeni sohbetin klasörü için güven sorusu henüz cevaplanmadı mı. */
  needsTrust() {
    return !!this.session && this.session.trusted == null;
  }

  /** Claude Code'daki "bu klasöre güveniyor musun" sorusu; ilk mesajdan önce bir kez. */
  renderTrust() {
    const existing = $("trust");
    if (!this.needsTrust()) {
      existing?.remove();
      return;
    }
    const cwd = this.session.cwd ?? "";
    // Klasör ya da ev dizini bilgisi değiştiyse (ör. ev dizini geç geldi) kutu yeniden çizilir.
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
    // Ev dizininin kendisi ya da onu içine alan bir klasör (ör. C:\): anahtarlar da okunabilir.
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
    // Tek cevap: ikinci tık (ya da öbür düğme) cevap gelene kadar engelli.
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

  /** Pet cevabı: metin, kaynaklar varsa altında katlanmış "Sources" listesi. */
  addReply(text) {
    // Güvenilen (dosya okuyan) sohbette web araması yok, dolayısıyla kaynak da yok: cevaptaki
    // bağlantılar tıklanabilir yapılmaz (okunan bir içerik bağlantıya gömülüp dışarı gitmesin).
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
      // href yok: her açılış (tık, klavye) open_url'deki adres denetiminden geçer; orta tık ya da
      // sürükleme WebView'in kendi gezinmesine düşmez.
      const a = document.createElement("a");
      a.setAttribute("role", "link");
      a.tabIndex = 0;
      // Gerçek alan adı her zaman görünür: başlık bir şey, adres başka bir yer olmasın.
      const title = document.createElement("span");
      title.textContent = s.title;
      const host = document.createElement("span");
      host.className = "host";
      host.textContent = hostOf(s.url);
      a.append(title, host);
      a.title = s.url;
      const open = () =>
        invoke("open_url", { url: s.url }).catch((err) => this.hooks.log(`bağlantı açılamadı: ${err}`));
      a.addEventListener("click", open);
      a.addEventListener("keydown", (e) => e.key === "Enter" && open());
      details.append(a);
    }
    // Açılınca liste görünür kalsın.
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
      // Cevap arka planda geldiyse bir süre okunabilsin, sonra normal kural (odak yoksa kapan).
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
    // Açılan terminal odağı alır; devir notu görünsün diye balon bir süre açık kalır. Süre not
    // yazıldıktan sonra yeniden başlar (terminal yavaş açılsa da not okunabilsin).
    this.keepOpen(HANDOFF_HOLD_MS);
    try {
      await invoke("chat_open_terminal");
      // Devredilen oturuma pet artık yazmaz; sıradaki mesaj yeni oturum açar.
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
 * `p`, `base` klasörü ya da onun içindeyse `base`'den sonraki klasör adları ([] = kendisi),
 * değilse null. Windows yolları: büyük/küçük harf ve ayraç farkı önemsiz. Klasör adı klasör adıyla
 * karşılaştırılır (karakter konumuyla değil: küçük harfe çevirmek "İ" gibi harflerde uzunluğu değiştirir).
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

/** CLI gibi kısa yol: kullanıcının ev dizini "~", uzunsa son iki klasör. */
export function shortPath(p, home) {
  if (!p) return "?";
  const rest = pathRest(p, home);
  if (rest) {
    return rest.length <= 2 ? ["~", ...rest].join("\\") : `~\\…\\${rest.slice(-2).join("\\")}`;
  }
  const parts = p.split(/[\\/]/).filter(Boolean);
  return parts.length <= 2 ? p : `…\\${parts.slice(-2).join("\\")}`;
}

// Adres: boşluksuz; içinde tek düzey parantez olabilir (ör. Wikipedia'da "Mercury_(planet)").
// `<` ve `>` adresin parçası sayılmaz: "<https://…>" biçimindeki adreste köşeli parantez dışarıda kalır.
const URL_RE = String.raw`https?:\/\/(?:[^\s()<>]|\([^\s()<>]*\))+`;
const LINK_LINE = new RegExp(String.raw`^\s*(?:[-*•]\s*)?\[([^\]]+)\]\((${URL_RE})\)\s*$`);
const URL_LINE = new RegExp(String.raw`^\s*(?:[-*•]\s*)?<?(${URL_RE})>?\s*$`);
const INLINE_LINK = new RegExp(String.raw`\[([^\]]+)\]\((${URL_RE})\)`, "g");
// Yalnız bilinen kaynak başlıkları ("Kaynaklar:", "**Sources:**"); ":" ile biten her cümle değil.
const SOURCES_HEADING = /^\s*\**\s*(kaynaklar|kaynak|sources?|references|referanslar)\s*:?\s*\**\s*$/i;

/**
 * Cevabın sonundaki bağlantı satırlarını (ve hemen üstlerindeki kısa başlığı) kaynak olarak ayırır.
 * Metin içindeki [başlık](adres) bağlantıları yalnız başlık olarak kalır, adres kaynaklara eklenir.
 * Döner: { body, sources: [{ title, url }] }
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
  // Cevabın kendisi bağlantıysa (ör. "repo adresi ne?") katlanmaz, metin olarak kalır.
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

/** Mesaj satırı yazdıkça en çok 3 satıra uzar. */
function autoGrow(el) {
  el.style.height = "20px";
  el.style.height = `${Math.min(el.scrollHeight, 60)}px`;
}
