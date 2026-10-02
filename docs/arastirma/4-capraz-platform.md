# 4. Çapraz platform: hook komutu, pencere, "Terminalde aç"

Tarih: 2026-10-02. Yerel Claude Code: 2.1.287 (CHANGELOG'un en üstü, mod desteğinin geldiği sürüm).
Belgeler ham markdown olarak indirildi (`https://code.claude.com/docs/en/<sayfa>.md`), kopyalar
scratchpad `docs/` klasöründe. Denemeler: scratchpad `hook-deneme/` (gerçek Claude oturumu açılmadı,
Claude Code ayarlarına dokunulmadı).

Önceki bulgular tekrar edilmedi: `docs/arastirma/2-teknik.md` §7 ve KARARLAR 14–17, 23–24.

---

## Özet ve öneri

**Önerilen: tek hooks.json, shell form (`args` yok), sh + PowerShell "polyglot" komut.**
Aynı metin `sh -c` (macOS, Linux), Git Bash (Git'li Windows) ve PowerShell (Git'siz Windows)
altında çalışıyor; exe varsa `<exe> hook <Olay>` çağırıyor (stdin aynen geçiyor), yoksa stdin'i
boşaltıp **0 ile çıkıyor**. `cmd.exe` sarmalayıcıya ve `.cmd` dosyasına gerek kalmıyor.
Windows (Git Bash, Git sh, PowerShell 5.1) ve WSL Ubuntu (`dash`, `bash`) altında denendi,
`claude plugin validate --strict` geçti. Gerçek Claude oturumunda henüz denenmedi (aşağıda
"Doğrulanamayanlar").

Komut (olay başına yalnız `SessionStart` kelimesi değişir; JSON içinde `\n` ile tek dize):

```text
echo `# <#` >/dev/null
d="$CLAUDE_PLUGIN_DATA/bin"; for e in "$d/bop" "$d/bop.exe"; do if [ -n "$CLAUDE_PLUGIN_DATA" ] && [ -f "$e" ] && [ -x "$e" ]; then "$e" hook SessionStart >/dev/null 2>&1; exit 0; fi; done; cat >/dev/null 2>&1; exit 0
#> > $null
$e = "$env:CLAUDE_PLUGIN_DATA/bin/bop.exe"; if ($env:CLAUDE_PLUGIN_DATA -and (Test-Path -LiteralPath $e -PathType Leaf)) { & $e hook SessionStart *> $null } else { $null = [Console]::In.ReadToEnd() }; exit 0
```

hooks.json girdisi:

```json
{ "type": "command", "async": true, "timeout": 5,
  "command": "echo `# <#` >/dev/null\nd=\"$CLAUDE_PLUGIN_DATA/bin\"; for e in ... exit 0\n#> > $null\n$e = ... exit 0" }
```

Nasıl çalışıyor:
- **sh/bash:** 1. satırda `` `# <#` `` boş bir komut ikamesi (içi yorum), `echo` boş satırı
  `/dev/null`'a yazar. 2. satır exe'yi arar, çalıştırır ve `exit 0`. sh betiği satır satır
  ayrıştırdığı için 3–4. satırlardaki PowerShell sözdizimine hiç ulaşmaz.
- **PowerShell:** `` `# `` kaçışlı bir `#` (echo'nun argümanı), ardından `<#` blok yorumu başlar ve
  `#>`'e kadar sh kısmını yutar; `> $null` echo çıktısını atar. 4. satır exe'yi çalıştırır.
- Yol hiçbir yerde metin olarak komuta yazılmıyor; ortam değişkeninden okunuyor
  (`$CLAUDE_PLUGIN_DATA` / `$env:CLAUDE_PLUGIN_DATA`). `${...}` yer tutucusu **bilerek
  kullanılmadı**: Claude Code onu PowerShell'de yeniden yazıyor, shell form'da metin olarak
  yerine koyabiliyor (KARARLAR 17'deki sorunun aynısı). Ortam değişkenleri her formda hook
  sürecine aktarılıyor (kaynak aşağıda, 1.3).

Deneme sonuçları (`hook-deneme/run.py`, `wsltest.sh`):

| Kabuk | Klasör | exe var | exe yok |
| :- | :- | :- | :- |
| Git Bash `bash -c` | `plain`, `R&D (x) %PATH% a!b ç'q$y`z` | exe `hook SessionStart` ile çağrıldı, stdin (Türkçe) bayt bayt aynı, rc 0 | rc 0, stderr boş |
| Git `sh -c` | aynı | aynı | aynı |
| `powershell.exe -NoProfile -NonInteractive -Command` (5.1) | aynı | aynı (stdin çocuk sürece miras kalıyor, yeniden kodlanmıyor) | rc 0, stderr boş |
| WSL Ubuntu `dash` (`/bin/sh`) ve `bash` | `R&D (x) a b $y`z` | aynı | rc 0; değişken yoksa da rc 0 |

Süre (exe yok, 5 ölçüm ort.): Git Bash ~60 ms, PowerShell 5.1 ~223 ms. Hook async olduğu için
ikisi de Claude'u bekletmez (1.4).

Neden bu, kısaca: belgede platform alanı yok (2c), `node` garanti değil (2d), exec form her
platformda var olan tek bir exe ister ve böyle bir exe yok (2b), kurulumda ayar yazmak kaldırmada
geride hook bırakır (2e). Polyglot, belgenin söylediği kabuk seçimi kuralına (1.1) dayanıyor ve
bugünkü sürümlerin hepsinde çalışır. Daha temiz ama çok yeni bir yol da var: **mod** (2f); ilk
sürüm için değil, izlenecek seçenek.

---

## 1. Hook `command`: shell form ve exec form

### 1.1 Biçimler ve kabuk seçimi

Kaynak: https://code.claude.com/docs/en/hooks#exec-form-and-shell-form

- "A command hook runs as exec form when `args` is set, and shell form when `args` is omitted."
- **Exec form:** "Claude Code resolves `command` as an executable on `PATH` and spawns it directly
  with `args` as the argument vector. There is no shell ... path placeholders like
  `${CLAUDE_PLUGIN_ROOT}` are substituted into `command` and into each `args` element as plain
  strings. ... No shell tokenization happens on any platform."
- **Shell form:** "The `command` string is passed to a shell: `sh -c` on macOS and Linux, Git Bash
  on Windows, or PowerShell when Git Bash isn't installed. Set the `shell` field to choose
  explicitly."
- Windows notu: "exec form requires `command` to resolve to a real executable such as a `.exe`.
  The `.cmd` and `.bat` shims ... can't be spawned without a shell."

### 1.2 `shell` alanı

Kaynak: https://code.claude.com/docs/en/hooks#command-hook-fields

> `shell` | no | Shell to use for this hook. Accepts `"bash"` or `"powershell"`. Defaults to
> `"bash"`, or to `"powershell"` on Windows when Git Bash isn't installed. Setting `"powershell"`
> runs the command via PowerShell on Windows. ... Ignored when `args` is set

- Windows'ta `"shell": "powershell"`: önce `pwsh.exe`, yoksa `powershell.exe`
  (https://code.claude.com/docs/en/hooks#windows-powershell-tool).
- **macOS/Linux'ta `"shell": "powershell"` ne yapar belgelenmemiş.** Belge yalnız "on Windows"
  diyor. PowerShell aracı macOS/Linux'ta `pwsh` ister
  (https://code.claude.com/docs/en/tools-reference#powershell-tool); hook için `pwsh` yoksa
  başlatılamama hatası beklenir. Bu yüzden `shell` alanı **kullanılmamalı**; polyglot varsayılan
  seçimle çalışıyor.
- Git Bash'in yeri bulunamazsa ayarla verilebilir (`CLAUDE_CODE_GIT_BASH_PATH`, setup belgesi:
  https://code.claude.com/docs/en/setup). Git for Windows isteğe bağlı: "Installing Git for Windows
  is optional" (aynı sayfa).

### 1.3 Değişkenlerin genişletilmesi

- Her iki form da yer tutucuları destekler ve ortam değişkeni olarak aktarır: "Both forms support
  the same path placeholders, and both export them as the environment variables
  `CLAUDE_PROJECT_DIR`, `CLAUDE_PLUGIN_ROOT`, and `CLAUDE_PLUGIN_DATA` on the spawned process"
  (https://code.claude.com/docs/en/hooks#exec-form-and-shell-form).
- Plugin hook ortamı: "every hook process receives `CLAUDE_PLUGIN_ROOT` and `CLAUDE_PLUGIN_DATA`
  in its environment" (https://code.claude.com/docs/en/plugins/components, "Environment, quoting,
  and matching MCP tools").
- Shell form'da yer tutucu çift tırnağa alınmalı; `claude plugin validate` tırnaksız
  `${CLAUDE_PLUGIN_ROOT}` için uyarır, `shell: "powershell"` ise uyarmaz
  (https://code.claude.com/docs/en/plugins-reference, "Quoting and path separators").
- Windows'ta yerine konan yollar `/` ile gelir: "On Windows, the substituted paths use forward
  slashes so a shell doesn't read backslashes as escapes." (aynı bölüm).
- PowerShell shell form: "As of v2.1.198, Claude Code rewrites the `${CLAUDE_PROJECT_DIR}`,
  `${CLAUDE_PLUGIN_ROOT}`, and `${CLAUDE_PLUGIN_DATA}` placeholders in a PowerShell shell-form
  command to PowerShell's `${env:NAME}` form" ve "Don't write the bare `$CLAUDE_PROJECT_DIR`
  spelling in a PowerShell hook" (https://code.claude.com/docs/en/hooks#windows-powershell-tool).
  Polyglot'ta PowerShell kısmı bu yüzden `$env:CLAUDE_PLUGIN_DATA` kullanıyor; sh kısmındaki
  çıplak `$CLAUDE_PLUGIN_DATA` blok yorumun içinde, PowerShell onu çalıştırmıyor.
- `${user_config.*}` yalnız exec form'da yerine konur; shell form'da hata verir (hooks belgesi,
  aynı bölüm). Bizi etkilemiyor.

### 1.4 async ve timeout

Kaynak: https://code.claude.com/docs/en/hooks#run-hooks-in-the-background

- "Once an async hook is running in the background, Claude Code doesn't enforce `timeout` on it."
  Yani `"timeout": 5` async hook'ta **işlemiyor** (common fields tablosu da aynısını söylüyor:
  https://code.claude.com/docs/en/hooks#common-fields). Exe'nin kendisi hızlı çıkmalı.
- `-p` oturumunda kapanışta çalışan async hook öldürülür ("finalizes it with outcome `cancelled`").
- "Async hook completion notifications are suppressed by default." (`--verbose`/Ctrl+O ile görünür.)

---

## 2. Tek hooks.json ile üç platformda sessiz çalışma: adaylar

### (a) Düz `sh` betiği (Windows'ta Git Bash zorunlu sayarak)

- Artı: macOS/Linux'ta doğal, tek dosya.
- Eksi: Git Windows'ta isteğe bağlı (setup belgesi). Git'siz Windows'ta shell form PowerShell'e
  düşer; `sh script.sh` ya da `"$CLAUDE_PLUGIN_ROOT/hooks/x.sh"` PowerShell'de çalışmaz, her olayda
  hata bildirimi çıkar. **Elendi** (KARARLAR 15'teki gerekçeyle aynı).

### (b) Exec form ile platforma göre seçim

- Exec form `command`'ı `PATH`'te arar (1.1). Üç platformda da var olan tek bir exe yok:
  `cmd.exe` macOS/Linux'ta yok; `sh` Git'siz Windows'ta yok (Git kurulu olsa bile `sh.exe`
  genelde `PATH`'te değil; bu kısım belgeden değil, Git for Windows'un varsayılan `PATH`
  ayarından, doğrulanmadı).
- Exe'nin kendisini çağırmak (`command: "${CLAUDE_PLUGIN_DATA}/bin/bop"`) exe yokken başlatılamaz,
  hata bildirimi üretir (3. bölüm).
- Plugin içine küçük bir başlatıcı exe koymak mümkün ama ikili dosya plugin incelemesinde sorun
  (KARARLAR 24) ve Windows'ta uzantısız yolun `.exe`'ye çözülüp çözülmediği belgelenmemiş.
  **Elendi.**

### (c) Platform/OS koşulu

- Hook'un ortak alanları: `type`, `if`, `timeout`, `statusMessage`, `once`
  (https://code.claude.com/docs/en/hooks#common-fields); command hook ek alanları: `command`,
  `args`, `async`, `asyncRewake`, `shell` (1.2). **Platform/OS alanı yok.** `if` yalnız araç
  olaylarında ve izin kuralı sözdizimiyle çalışıyor; "On other events, a hook with `if` set never
  runs." Plugin manifestinde de platform koşulu yok (plugins-reference'ta arandı, bulunmadı).
  **Yok.**

### (d) Her yerde var olan bir yorumlayıcı (node?)

- Native kurulumda Node yok: "The installed `claude` binary does not itself invoke Node." ve npm
  paketi de "the same native binary" kuruyor (https://code.claude.com/docs/en/setup, npm bölümü).
  Belgedeki `node` örneği (hooks belgesi Windows notu) kullanıcıda node olduğunu varsayıyor.
  **Elendi.**
- `sh` macOS/Linux'ta, PowerShell her Windows'ta (5.1) var. Polyglot tam olarak bu iki
  yorumlayıcıya dayanıyor; Git Bash varken Windows'ta da sh kısmı çalışıyor ve exe'yi `.exe`
  uzantısıyla buluyor.

### (e) Kurulumda platforma göre hook yazmak (`/bop setup`)

- Artı: her platforma en hızlı komut yazılabilir.
- Eksi: plugin hook'ları `hooks/hooks.json`'dan otomatik yüklenir ve plugin'le birlikte
  kaldırılır ("When a plugin is enabled, its hooks merge with your user and project hooks",
  https://code.claude.com/docs/en/hooks#reference-scripts-by-path). Kullanıcının
  `settings.json`'ına yazılan hook ise plugin kaldırılınca **geride kalır** ve exe gittiği için
  (`${CLAUDE_PLUGIN_DATA}` kaldırmada silinir, KARARLAR 14) her olayda hata üretir. Kullanıcı
  ayarını değiştirmek dizin incelemesinde de kötü görünür (bu son kısım tahmin, belge yok).
  **Önerilmez.**

### (f) Mod (yeni, 2.1.287): hook'u Claude Code'un içinde JS fonksiyonu olarak çalıştırmak

Kaynaklar: https://code.claude.com/docs/en/plugins/mods/overview,
https://code.claude.com/docs/en/plugins/mods/reference,
https://code.claude.com/docs/en/plugins/mods/api,
https://github.com/anthropics/claude-code/blob/main/mods/types/claude-code.d.ts,
CHANGELOG 2.1.287 ("Added Claude Mods").

- `hooks/hooks.json` içinde `"modules": ["./register.js"]`; modül klasik olaylara
  `classic.<Olay>` adıyla abone olabiliyor ve "`e` is the hook's stdin JSON" (reference,
  "Settings hook events").
- `$.fs.exists(path)` ve `$.process.run(argv, { stdin, timeoutMs })` var; `run` "uses no shell"
  (api, "Reach files, processes, and the network"; `stdin` alanı d.ts'te `ProcessRunInit`).
  Yani exe yoksa hiç süreç başlatmadan çıkılabilir: **kabuk yok, platform farkı yok, bildirim yok.**
- Scratchpad'te `vmod/` örneği `claude plugin validate --strict` geçti (olay adları string literal
  olmak zorunda; döngüyle `on('classic.' + ev)` doğrulayıcıdan geçmedi).
- Neden şimdilik değil:
  - **Çok yeni:** mod desteği 2.1.287 ile geldi (bugünkü sürüm). Daha eski Claude Code'un
    `modules` anahtarını yok sayıp saymadığı belgelenmemiş; eski sürümde pet sessizce çalışmaz.
  - Kurumsal ortam: Team/Enterprise'da yerleşik bir koruma modu yükleniyor; yönetici
    `allowManagedModsOnly` ile kullanıcı modlarını kapatabilir ya da `process.run` çağıran modu
    reddedebilir (https://code.claude.com/docs/en/plugins/mods/admin). `disableAllHooks` de
    modları durdurur.
  - d.ts'te `process` için "CLI only" yazıyor; Desktop'ın Code sekmesinde ne olduğu net değil.
  - `CLAUDE_PLUGIN_DATA` mod sürecinin (`$.env.get`) ortamında var mı belgelenmemiş; yoksa veri
    klasörü yolu başka yerden bulunmalı (`$.plugin.root` var, veri klasörü için bir yöntem
    görülmedi).
  - Dizin incelemesinde `calls: $.process.run` satırı görünür ("Starts programs as the user",
    admin belgesi); ek soru doğurabilir (tahmin).
- Sonuç: Aşama 7 sonrası için izlenecek seçenek. İstenirse hooks.json'da hem polyglot hook hem
  mod olur, ama ikisi birden olay başına exe'yi iki kez çağırır; birini seçmek gerekir.

### (g) Önerilen: polyglot shell form

Belgeyle doğrulanan dayanaklar:
- Shell form kabuk seçimi: sh (macOS/Linux), Git Bash ya da PowerShell (Windows) (1.1).
- Ortam değişkenleri her formda hook sürecinde (1.3).
- Çıkış 0 ve boş stdout = başarı, transcript'te bir şey görünmez: "Exit 0 means success" ve
  "Stderr from a hook that exits 0 goes to the debug log only, never the transcript"
  (https://code.claude.com/docs/en/hooks#exit-code-0).
- `SessionStart` ve `UserPromptSubmit`'te düz stdout Claude'a bağlam olarak eklenir (aynı bölüm).
  Polyglot ve exe'nin çıktısı `/dev/null` / `*> $null`'a gidiyor; boş satır yazan `echo` da
  yönlendirilmiş. Exe'nin stdout'a hiçbir şey yazmaması bu yüzden önemli.

Artı: tek dosya, ek bağımlılık yok, `.cmd`/ASCII kuralı ve `cmd` gecikmeli genişletme hilesi
(KARARLAR 15, 17) gereksizleşir; yol özel karakterleri sorun değil (denendi).
Eksi: okunması zor; Git'siz Windows'ta PowerShell açılışı ~220 ms (async, bloklamaz);
hooks.json'da 10 kopya (olay adı dışında aynı). Exe olay adını stdin'deki `hook_event_name`'den
okursa komut olaylardan bağımsız olur, ama mevcut `<exe> hook <Olay>` arayüzü korunabilir.

---

## 3. Exe yokken hata bildirimi

Kaynak: https://code.claude.com/docs/en/hooks#exit-code-output

- Çıkış 0: başarı. stderr yalnız debug log'a gider.
- Çıkış 2: engelleme (olaya göre); `SessionStart`'ta stderr transcript'te "hook error" olarak
  gösterilir.
- Diğer kodlar (1, 127...): stdout boş ya da düz metinse "non-blocking error ... the transcript
  shows a `<hook name> hook error` notice followed by the first line of stderr, prefixed with
  `Failed with non-blocking status code:`".
- Başlatılamayan hook: "A hook that can't start lands in the same non-blocking bucket. When the
  script path doesn't exist or isn't executable, the shell exits with a code like 127 and you see
  the same notice" (aynı bölüm). Yani `cmd.exe` olmayan macOS/Linux'ta mevcut hook her olayda bu
  bildirimi üretir (KARARLAR 15 ile uyumlu).
- JSON gibi görünen (`{` ile başlayan) ama bozuk stdout her çıkış kodunda hata bildirimi üretir
  ("when Claude Code tries to parse your stdout as JSON and can't, it reports a non-blocking
  error"). Exe stdout'a yazmamalı.

**Async hook'ta durum belgede açık değil.** Belge async hook'un sonuçlarını (`systemMessage`,
`additionalContext`) bir sonraki turda Claude'a verdiğini ve "completion notifications are
suppressed by default" olduğunu söylüyor; başarısız bir async hook'un "hook error" bildirimini
kullanıcıya gösterip göstermediğini söylemiyor. Dolaylı işaretler:
- Issue #79847: Windows'ta aynı betik senkron hook'larda her seferinde `hook_non_blocking_error`
  üretirken "0 occurrences of this error for hooks marked `"async": true`"
  (https://github.com/anthropics/claude-code/issues/79847). Bu bir hata türüne özgü, genel kural
  değil.
- CHANGELOG 2.1.287: `asyncRewake` hook'un betik dosyası yoksa "found issues" bildirimi
  tekrarlanıyordu, artık "reported once". `asyncRewake` farklı bir mekanizma (çıkış 2'de Claude'u
  uyandırır), düz `async` için bilgi vermiyor.
Sonuç: async'e güvenmek yerine komut **her durumda 0 ile çıkmalı**; polyglot bunu yapıyor.
Gerçek oturumda doğrulanmalı (aşağıda).

---

## 4. macOS/Linux'ta Tauri saydam, kenarlıksız, her zaman üstte pencere

Projede kilitli sürüm: `tauri 2.12.0`, `tao 0.37.1`, `wry 0.57.0` (`app/src-tauri/Cargo.lock`).

- **macOS saydamlık:** Tauri 2.12.1 CHANGELOG: "The `macos-private-api` feature flag /
  `macOSPrivateAPI` tauri.conf.json value is no longer required to use transparency or fullscreen
  on macOS." (https://github.com/tauri-apps/tauri/blob/dev/crates/tauri/CHANGELOG.md, 2.12.1).
  Şema da "No-op in Tauri 2.12.1+" diyor (https://schema.tauri.app/config/2,
  `AppConfig.macOSPrivateApi`). **Proje 2.12.0'da**: ya 2.12.1+'a yükseltilmeli ya da
  `"macOSPrivateApi": true` + `macos-private-api` özelliği açılmalı (2.12.0 öncesi kural, eski
  sürümde App Store'a giremeyen özel API). Yükseltmek daha temiz.
- **Gölge:** `shadow` "Linux: Unsupported"; Windows'ta kenarlıksız pencerede `true` 1 px kenar
  verir (şema, `WindowConfig.shadow`).
- **Linux, Wayland** (tao `Window` belgesi, https://docs.rs/tao/0.37.1/tao/window/struct.Window.html):
  - `set_always_on_top`: "Linux(x11): Result depends on the system's window manager. Consider this
    setting a suggestion. Linux(Wayland) / iOS / Android: Unsupported."
  - `set_outer_position`: "Android / Linux(Wayland): Unsupported."; `outer_position`:
    "Linux(Wayland): Has no effect, since Wayland doesn't support a global coordinate system".
    Pencere konumunu kaydetme/geri yükleme (`window.json`) ve pete yanaşan balon Wayland'da
    çalışmaz.
  - `cursor_position`: "iOS / Android / Linux(Wayland): Unsupported, returns 0,0". Fareye bakma
    Wayland'da çalışmaz.
  - `set_ignore_cursor_events`: Linux için kısıt yazmıyor (yalnız iOS/Android).
  - `drag_window`: "macOS: May prevent the button release event to be triggered."
  - `skipTaskbar`: "hides the window icon from the taskbar on Windows and Linux" (şema); macOS'ta
    Dock simgesi için ayrı ayar gerekir (activation policy; belge araştırılmadı).
  - Yaygın geçici çözüm: uygulamayı XWayland üzerinden açmak (`GDK_BACKEND=x11`). Bu Tauri
    belgesinde değil, genel GTK bilgisi; doğrulanmadı.
- **Linux, X11 saydamlık:** gerçek piksel saydamlığı bir compositor ister (compositing olmayan
  pencere yöneticisinde arka plan siyah görünür). Bu genel X11 bilgisi, Tauri belgesinde açık bir
  cümle bulunmadı. GNOME/KDE varsayılan olarak compositor çalıştırır.
- **Linux bağımlılığı:** WebKitGTK 4.1 (`libwebkit2gtk-4.1`) çalışma anında gerekir
  (https://v2.tauri.app/start/prerequisites/). AppImage ya da .deb ile dağıtım seçimi Aşama 7 işi.
- **macOS imza:** indirilen imzasız ikili Gatekeeper'a takılabilir; karantina özniteliğini indiren
  uygulama koyar (curl koymaz). Bu kısım belge ile doğrulanmadı, indirme tasarımında ele alınmalı.

---

## 5. "Terminalde aç" (yeni terminalde `claude --resume <id>`)

Ortak: komutu doğrudan değil, yalnız `cd <klasör>` ve `claude --resume <uuid>` içeren geçici bir
betik dosyasıyla vermek tırnak sorunlarını önler (id yalnız UUID karakteri olmalı, doğrulanmalı).

- **macOS:** Terminal.app her macOS'ta var.
  - `open -a Terminal /tmp/bop-resume.command` (dosya `#!/bin/sh`, `chmod +x`). `open` sistem aracı;
    ek izin istemez.
  - Alternatif: `osascript -e 'tell application "Terminal" to do script "..."'`. Otomasyon izni
    (TCC "X, Terminal'i denetlemek istiyor") sorusu çıkarabilir. Bu iki madde genel macOS bilgisi,
    resmi belgeyle doğrulanmadı.
- **Linux:** standart yok, sırayla dene:
  1. `$TERMINAL` (gelenek, standart değil).
  2. `xdg-terminal-exec <komut> [arg...]`: freedesktop'a önerilmiş, henüz taslak spesifikasyon
     ("while this spec is in proposed state, backwards compatibility ... is not guaranteed",
     https://github.com/Vladimir-csp/xdg-terminal-exec).
  3. `x-terminal-emulator -e <komut> [arg...]` (Debian/Ubuntu ailesi): "`-e command` ... creates a
     new terminal window and runs the specified command ... as though the arguments were passed
     directly to `execvp`, bypassing the shell" (Debian Policy 11.8.3,
     https://www.debian.org/doc/debian-policy/ch-customized-programs.html).
  4. Bilinenler: `gnome-terminal -- <komut>`, `konsole -e`, `xfce4-terminal -x`, `kitty`,
     `alacritty -e`, `wezterm start --`, `xterm -e` (her birinin argüman biçimi farklı; doğrulanmadı).
  Hiçbiri yoksa balonda "komutu kopyala" (`claude --resume <id>`) göstermek en güvenli yedek.

---

## Doğrulanamayanlar (gerçek oturumda denenmeli)

1. Claude Code'un Windows'ta Git Bash'i ve PowerShell'i **hangi argümanlarla** başlattığı
   (`bash -c`/`-lc`? `powershell -Command`/`-File` geçici dosya?). Denemeler `bash -c` ve
   `powershell -NoProfile -NonInteractive -Command` ile yapıldı. `-File` ile geçici `.ps1`
   kullanılıyorsa yürütme ilkesi devreye girebilir (PowerShell **aracı** için `-ExecutionPolicy
   Bypass` belgeli, hook için belgede yok).
2. Async hook başarısız olunca kullanıcıya bildirim gösterilip gösterilmediği (3. bölüm).
3. Çok satırlı `command` dizesinin üç platformda Claude Code tarafından olduğu gibi kabuğa
   verildiği (validate geçti; çalıştırma görülmedi).
4. macOS'ta `/bin/sh` (bash 3.2 sh kipi) ile polyglot (Git sh ve dash ile denendi, macOS'ta değil;
   GitHub Actions macOS koşucusunda denenebilir).
5. macOS/Linux'ta `"shell": "powershell"` davranışı (kullanmıyoruz).
6. Mod yolu için: eski sürümlerde `modules` anahtarı, mod ortamında `CLAUDE_PLUGIN_DATA`, Desktop'ta
   `$.process.run`.

Önerilen deneme sırası: (1) Windows, Git kurulu: plugin'i `--plugin-dir` ile yükle, `--debug` ile
hook satırlarını oku, exe varken/yokken; (2) Git'siz Windows (PATH'ten Git'i çıkararak ya da
`CLAUDE_CODE_GIT_BASH_PATH`'i geçersiz bir yere vererek); (3) WSL'de Linux Claude Code;
(4) macOS: Actions'ta `claude plugin validate` + polyglot'un `/bin/sh -c` ile birim testi.

## Deneme dosyaları

geçici deneme klasörü `hook-deneme/`:
`poly.txt` (komut), `run.py` (Windows kabukları), `wsltest.sh` (dash/bash), `fake.cs`/`fake.exe`
(argüman ve stdin döken sahte exe), `vplug/` (polyglot hooks.json, validate geçti), `vmod/` (mod
örneği, validate geçti).
