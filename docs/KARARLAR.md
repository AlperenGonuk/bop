# Karar günlüğü

Yeni karar en üste. Biçim: tarih, karar, neden, kaynak.

## 2026-10-02

34. **Yayın öncesi temizlik ve yeni geçmiş.** Kaynak: Alperen (denetim sonrası seçimler).
    - Yayın dalı tek "Initial commit" ile başlar; eski geçmiş yalnız yerelde yedek dalda kalır.
      Commit e-postası GitHub noreply adresi.
    - İç belgeler (DURUM, PLAN, KARARLAR, araştırma) repoda kalır; kişisel yollar ve iç notlar
      temizlendi, eski C# denemesi (`pitir-deneme/`) çıkarıldı, Johnny reposu bağlantısı kaldırıldı.
    - Uygulama ikonu Pıtır'ın nötr karesinden üretildi (eski turuncu ikon Clawd'u çağrıştırıyordu).
    - Repo `AlperenGonuk/bop`; marketplace adı `bop`, plugin `bop@bop`.
33. **Exe kurulumu `/bop setup` ile, hook'la değil** (24. kararı somutlaştırır). Kaynak: Claude
    (Alperen "bitene kadar sorma"); neden: indirme yapan hook'lar dizin incelemesinde bekletilir.
    - `plugin/scripts/install.ps1` / `install.sh`: `plugin.json` `version` → release `v<sürüm>`,
      varlık ve `SHA256SUMS` indirilir, sha256 doğrulanır, `<CLAUDE_PLUGIN_DATA>/bin/` içine konur;
      aynı sürümse atlanır (`bop --version`), `--force` yeniden kurar. Repo tek yerde:
      `plugin.json` `metadata.releaseRepo`. Test için `BOP_RELEASE_BASE_URL`.
    - Actions (`.github/workflows/release.yml`): `v*` etiketinde Windows x64, macOS arm64/x64,
      Linux x64; `SHA256SUMS`; release **taslak** oluşur, Alperen yayınlamadan indirme çalışmaz.
    - Tauri 2.12.0 → 2.12.1 (macOS saydamlık). Pet görselleri kodla aynı lisans (MIT).

31. **Sprite kuralı netleşti:** `dev-pets/` ve dışarıdan gelen sprite'lar değiştirilmez;
    `app/src-tauri/default-pet/` motor çıktısıdır, `pitir.json`'dan yeniden üretilir (CLAUDE.md
    güncellendi). Kaynak: Alperen (kod incelemesi bulgusu 8). Aynı gün: arayüz metinleri İngilizceye
    çevrildi (21. karar, Alperen "ayarlar kısmını falan da İngilizce yap").
    Terimler: Yeni sohbet → New chat, Terminalde aç → Open in terminal, "Bu klasöre güveniyor
    musun?" → "Trust this folder?", Güven/Güvenme → Trust/Don't trust, Kaynaklar → Sources, Pet
    değiştir → Switch pet, Yeni pet yap → Make a new pet, Kapat → Quit. 18–20 ve 28. kararlardaki
    Türkçe adlar artık bu İngilizce karşılıklarıyla okunmalı. Günlük satırları ve yorumlar Türkçe.
32. **Exe konsol alt sistemiyle derlenir** (`windows_subsystem = "windows"` kalktı). Kaynak: Claude
    (kod incelemesi bulgu 5; Alperen "bitene kadar sorma"). Neden: PowerShell 5.1 GUI alt sistemli
    exe'yi beklemiyor, skill'in okuduğu çıktı ve `$LASTEXITCODE` kayboluyordu; `AttachConsole`
    yetmedi. Pet penceresi `toggle` ile CREATE_NO_WINDOW açılır; Gezgin'den açılırsa kendi konsolu
    `FreeConsole` ile bırakılır (kısa bir yanıp sönme olabilir). Release exe ile denendi: çıktı
    yakalanıyor, çıkış kodları 0/1 doğru, `toggle`/`stop` çalışıyor.


30. **`hatch` motoru ve spec biçimi** (Aşama 6; Alperen "bitene kadar sorma" dedi, varsayılanlar
    ajan seçimi, gözden geçirilecek). `bop hatch <spec.json> <klasör>` → `pet.json`
    (`spriteVersionNumber: 2` her zaman; 3. karardaki "yalnız toplulukta" ifadesi eskidi, güncel
    Codex skill'i zorunlu sayıyor), `spritesheet.png`, `contact-sheet.png`, `report.json`.
    - Spec üç seviye: arketip (`blob`, `critter`, `floaty`, `custom`) + palet + seçenekler; ada göre
      birleşen parçalar; `pixels` parçaları ve kare `overrides`. Bilinmeyen alan hatadır.
    - Varsayılan ızgara 48×52 @4 (Pıtır denemesi gibi); 96×104 @2 isteğe bağlı. Yalnız piksel stili.
    - Kopuk efekt yok (Codex kuralı; Pıtır'ın "...", düşünce balonu, kıvılcımları kalktı). Ter
      damlası pete değer. Kopuk parça yalnız uyarı.
    - `running-left` ayna değil, sola dönük yeniden çizim; 180° = aşağı bakış.
    - (0,6) nötr = idle 0; uygulama (sprites.js) v2'de bu hücreyi idle döngüsünden çıkarır.
    - Taşan pet otomatik %80'e kadar küçültülür (uyarı). Çıkış kodu 0/2 (doğrulama)/1 (spec).
    - Örnek spec'ler `plugin/skills/hatch/examples/` (exe'ye gömülü, `--example`); gömülü Pıtır
      `pitir.json`'dan üretilir, test motor çıktısıyla aynı olmasını denetler.
    - Skill model tarafından çağrılabilir, `allowed-tools` yalnız exe; `Write` (spec) kullanıcıya sorulur.
    - (Gözden geçirme, Claude) Referanslar exe'ye gömülü: `bop hatch --docs <ad>`; Claude plugin
      klasörünü okuma izni istemez. İki skill'de PowerShell kuralları komut adıyla (`&` olmadan,
      belge: PowerShell kuralları Bash ile aynı biçim, AST'den eşlenir) ve "Bash aracı varsa onu
      kullan, komutu tek başına çalıştır" talimatı. Gerçek `claude -p` denemesinde ("A small sleepy
      blue ghost") akış izin reddi olmadan bitti: spec, hatch, QA, install.

29. **Hook'lar shell form, tek "polyglot" komut; plugin `plugin/` klasöründe** (15. ve 17. kararı
    değiştirir). Kaynak: Alperen ("devam et"), araştırma `docs/arastirma/4-capraz-platform.md`.
    - Komut `args` olmadan verilir: macOS/Linux'ta `sh -c`, Windows'ta Git Bash, Git yoksa
      PowerShell çalıştırır. Aynı metnin ilk yarısı sh, ikinci yarısı PowerShell (`<# ... #>`
      blok yorumu sh kısmını gizler). Yol ortam değişkeninden okunur (`$CLAUDE_PLUGIN_DATA`,
      PowerShell'de `$env:`), `${...}` yer tutucusu kullanılmaz. Exe yoksa stdin boşaltılır,
      her durumda 0 ile çıkılır. `shell` alanı kullanılmaz (macOS/Linux'ta davranışı belgesiz).
    - Plugin: `plugin/hooks/hooks.json` (`$CLAUDE_PLUGIN_DATA/bin/bop[.exe]`). Repo geliştirme
      hook'ları `.claude/settings.json` aynı komutla `$CLAUDE_PROJECT_DIR/app/src-tauri/target/debug`.
      `hooks/claude-pet-hook.cmd` kaldırıldı.
    - Plugin kökü repo kökü değil `plugin/`: kökteki CLAUDE.md `validate --strict`'i uyarıyla
      düşürüyor, uygulama kaynağı da plugin önbelleğine kopyalanmasın. Marketplace girdisi
      `source: "./plugin"` olacak.
    - Denendi: bu repoda gerçek Claude oturumu (Windows, Git Bash) `~/.bop/state.json`'a yazıyor;
      `claude plugin validate --strict plugin` geçti. PowerShell yolu ve macOS yalnız araştırmadaki
      elle denemelerle doğrulandı.

28. **Aktif pet seçimi ve yeni pet** (bekleyen kararı kapatır). Kaynak: Alperen.
    - Kurulu petler `~/.bop/pets/<id>/`; seçili pet `~/.bop/config.json` (`{"activePet": "<id>"}`).
    - Sağ tık menüsü: **Pet değiştir ▸** (kurulu petler, menü her açılışta taranır, seçili olanda ✓)
      ve **Yeni pet yap** (yeni terminalde `claude` `hatch` skill'iyle başlar, boş çalışma klasörü
      `~/.bop/hatch/`; skill sonunda "bu pete geçeyim mi?" diye sorar). Pet sohbetinin araçları
      değişmez: yazan iş terminalde yapılır.
    - `/bop use <id>` da aynı dosyaya yazar; pet dosyayı izler, yeniden başlatmadan değişir (asset
      izni yeni klasöre eklenir, `claude-pet.json` ve animasyonlar yeniden yüklenir).
    - Pet yoksa ya da seçili pet bulunamazsa exe'ye gömülü Pıtır açılır. `--pet` yalnız geliştirme.
    - (Aynı gün, uygulamada) "Yeni pet yap" ilk mesajı komut değil düz cümle: `claude "Let's make a
      new Bop pet!"` (CLI belgesi: `claude "query"` etkileşimli oturumu ilk mesajla açar; giriş
      kutusunu gönderilmeden doldurmanın yolu yok). Kullanıcının mesajı gibi görünür; `hatch`
      skill'inin açıklaması bu isteği tanıyacak şekilde yazılır, model çağırabilir kalır.
      Kaynak: Alperen.

27. **Pet çizim motoru Rust'ta, exe'nin alt komutu** (`<exe> hatch ...`), `png` crate'i doğrudan
    bağımlılık (Tauri ağacında zaten var). Kaynak: Alperen (onaylı). Neden: üç platformda aynı kod,
    kullanıcıya ek kurulum yok. Pıtır denemesindeki C# (scratchpad) Rust'a çevrilecek.
    Test: Windows gerçek cihaz, Linux WSL, macOS yalnız GitHub Actions derlemesi + otomatik testler.
26. **Plugin adı `bop`.** Tanıtım: "Bop — a desktop pet for Claude Code" (araştırma: "for Claude
    Code" düz metinde uygun, adın parçası değil). Kaynak: Alperen. Not: tek ve yaygın bir kelime
    olduğu için portalda "Name is taken" ya da "generic" bekletmesi riski var; başvurudan önce
    kontrol edilecek. `~/.claude-pet`, exe adı ve repo adı da buna göre değişecek.

25. **Yayındaki varsayılan pet Pıtır** (pet oluşturma skill'iyle kodla çizilen mandalina). Kaynak:
    Alperen. Neden: özgün, lisans sorunu yok; Clawd için izin yok (`docs/arastirma/1-dizin-marka.md`).
    Aşama 6'daki varsayılan pet sorusunu kapatır.
24. **Exe ilk çalıştırmada GitHub Releases'tan indirilir**, sha256 ile doğrulanır, README'de açıkça
    yazılır. Kaynak: Alperen. Neden: plugin içine exe koymak boyut sınırını aşıyor ve incelemede
    bekletiliyor; resmi bir indirme mekanizması yok (`docs/arastirma/2-teknik.md`).
23. **İlk sürüm Windows, macOS ve Linux'u hedefler** (16. kararı değiştirir). Kaynak: Alperen
    ("ilk sürümden macOS ve Linux'u da hedefleyelim"). Neden: resmi ya da topluluk dizinine kabul.
    Sonuç: platforma bağlı parçalar (hook sarmalayıcı, çizim motoru, "Terminalde aç", Actions)
    üç platformda çalışacak biçimde yeniden ele alınır. Plugin adı `claude-` ile başlayamaz
    (`claude plugin validate` hata verir); yeni ad seçilecek.

22. **Pet oluşturma skill'i görsel üretim API'si olmadan, kodla çizerek çalışır.** Karakter bir kez
    veri olarak tanımlanır (ızgara + palet + parçalar), kareler koddan türetilir; Claude çıktıyı
    görüp düzeltir. Basit istekte iyi sonuç, ayrıntılı istekte istenene yakın sonuç hedefi.
    Referans: Codex `hatch-pet` (yalnız fikir; onun `$imagegen` yolu Claude Code'da yok). Kaynak:
    Alperen ("API kullanmak istemiyorum"). Neden: scratchpad denemesi ("Pıtır", tam v2 atlas,
    PowerShell + .NET System.Drawing, ek bağımlılık yok) Alperen'ce beğenildi.
21. **Plugin'e giren her şey İngilizce:** skill dosyaları, manifest, hook ve kullanıcıya görünen
    metinler, README. Proje içi belgeler (`docs/`, CLAUDE.md) Türkçe kalır. Kaynak: Alperen
    ("global'e sunacağız"). Uygulama arayüzündeki Türkçe metinler de Aşama 5–6'da çevrilecek.

## 2026-10-01

20. **Dosya ya da web, ikisi aynı sohbette hiç birlikte değil** (19. kararı değiştirir). Kaynak:
    Alperen (seçenek A). Neden: üçüncü kod incelemesi.
    - Güvenilen klasör: `--tools "Read,Glob,Grep"`, `--allowedTools` yok. Okunan bir dosya izinsiz
      bir web aramasının sorgusuna konup dışarı taşınamaz. Cevaptaki bağlantılar tıklanabilir
      yapılmaz. Web isteyen kullanıcıya talimat yeni sohbette "Güvenme"yi önerir.
    - Güvenilmeyen sohbet: yalnız `WebSearch` ve süreç boş `~/.claude-pet/chat` klasöründe çalışır.
      Önceden yalnız araç listesi daralıyordu; `claude -p` güvenilmeyen klasörde başlıyor, o
      klasörün `.claude/settings.json` hook'ları ve CLAUDE.md'si (`@` içe aktarmaları dahil)
      yükleniyordu.
    - Güven sorusu oturum başına bir kez cevaplanır; ikinci cevap yok sayılır (çift tık).
    - Ev dizini uyarısı, ev dizinini içine alan klasörlerde de (ör. `C:\`) gösterilir.
19. **Dosya okuma yalnız güvenilen klasörde; yeni sohbette balon sorar** (18. kararı daraltır).
    Kaynak: Alperen ("trust this folder gibi"). Neden: kod incelemesi; proje klasörü bilinmezken
    sohbet ev dizininde başlıyordu ve `Read` orada izinsiz (`.ssh`, `.claude/.credentials.json`).
    Bir aramaya gömülü talimat sırrı okutup "kaynak" bağlantısına koyabilirdi; tıklama sızdırırdı.
    - Yeni sohbetin ilk mesajından önce balonda "Bu klasöre güveniyor musun?" (ev dizininde ek
      uyarı). Güven → `Read,Glob,Grep,WebSearch` ve klasör `~/.claude-pet/trusted.json`'a yazılır,
      bir daha sorulmaz. Güvenme → yalnız `WebSearch` (o sohbet boyunca).
    - Proje klasörü bilinmiyorsa sohbet boş `~/.claude-pet/chat`'te başlar, soru sorulmaz (yalnız
      `WebSearch`). Ev dizini hiçbir zaman kendiliğinden çalışma klasörü olmaz.
    - Kaynak bağlantısında başlığın yanında gerçek alan adı her zaman görünür.
    - Sistem talimatı araçlara göre değişir; ilk istekte kaydedildiği için soru cevaplanmadan
      mesaj gönderilmez.
18. **Pet sohbetinde yalnız okuyan araçlar açık** (7. kararın "araç yok" kısmını değiştirir):
    `--tools "Read,Glob,Grep,WebSearch" --allowedTools "WebSearch" --permission-prompts none`, ayrıca
    `--append-system-prompt` ile Claude'a balonda olduğu ve sınırları söylenir. Kaynak: Alperen.
    Neden: araçsız Claude hava durumu sorusunda araç çağrısını düz metin olarak yazdı (transkriptte
    `totalToolDuration: 0`). Belgeye göre (tools-reference, 2026-10-01): `Read`/`Glob`/`Grep` çalışma
    klasöründe izin istemez, dışında ister (soru `none` ile reddedilir); `WebSearch` ve `WebFetch`
    izin ister. `WebFetch` bilerek kapalı: dosya okuyan oturum keyfî adrese istek atıp içerik
    sızdırabilir. Talimat yalnız yeni sohbetlerde geçerli (CLI belgesi: sistem talimatı ilk
    istekte kaydedilir).
    - Balon dar: talimat 1–3 kısa cümle, düz metin ister; istenmeyen öneri/tavsiye vermez
      (Alperen: öneriler beklenmedik sonuç doğurabilir). Kaynaklar metinde
      yazılmaz; sonda bağlantı satırı olarak gelirse balonda katlanmış "Kaynaklar (n)" altında
      gösterilir, tıklanınca `rundll32 url.dll,FileProtocolHandler` ile tarayıcıda açılır (yalnız
      http/https; `explorer.exe` adresi virgülden bölüyordu). Kaynak: Alperen.
17. **Hook yolu metin olarak değil, gecikmeli genişletmeyle verilir** (15. kararın biçimini değiştirir):
    `"args": ["/d", "/v:on", "/c", "!CLAUDE_PLUGIN_ROOT!/hooks/claude-pet-hook.cmd", "<Olay>"]`
    (repo ayarında `!CLAUDE_PROJECT_DIR!`). Sarmalayıcı `setlocal DisableDelayedExpansion` ile başlar.
    Neden: kod incelemesi; `${...}` yer tutucusu yolu `cmd /c` satırına düz metin yazıyordu ve yolda
    boşluk yoksa tırnaksız kalıyordu. `&`, `%`, `^`, `(` içeren yollarda hook'ların hepsi bozuluyordu
    (denendi). `!VAR!` ayrıştırmadan sonra genişler; `plain`, `R&D`, `a b`, `R&D b`, `x%PATH%y`, `c^d`,
    `e!f`, `(g)` klasörlerinin hepsinde exe doğru argümanla çağrıldı, çıkış 0. Ortam değişkenleri
    hook sürecine aktarılır (hooks belgesi: "both export them as the environment variables
    `CLAUDE_PROJECT_DIR`, `CLAUDE_PLUGIN_ROOT`, and `CLAUDE_PLUGIN_DATA`", 2026-10-01'de okundu).
    Gerçek Claude oturumuyla henüz denenmedi.
16. **İlk sürüm yalnız Windows.** README'de açıkça yazar. GitHub Actions ilk sürümde yalnız Windows
    derler. Mac/Linux desteği ayrı bir sonraki aşamada (PLAN.md, Aşama 7); platforma göre hook
    kurulumu orada çözülecek. Kaynak: Alperen.
15. **Hook'lar `cmd.exe` exec biçimiyle `.cmd` sarmalayıcıyı çağırır** (async, kısa timeout):
    `"command": "cmd.exe", "args": ["/d", "/c", "${CLAUDE_PLUGIN_ROOT}/hooks/claude-pet-hook.cmd", "<Olay>"]`.
    Kaynak: Alperen. Neden ve ölçümler (2026-10-01):
    - Git Bash Windows'ta zorunlu değil (setup belgesi: "Installing Git for Windows is optional"),
      bu yüzden bash elendi. `cmd.exe` her Windows'ta var.
    - Süre: `cmd` sarmalayıcı ~31 ms, PowerShell ~271 ms (10 ölçüm ortalaması). Hook async olduğu
      için ikisi de Claude'u bekletmez ("runs in the background without blocking", hooks belgesi),
      ama `cmd` arka planda çok daha az yük bindirir.
    - `cmd` stdin'i yeniden kodlamadan geçirir; Türkçe yollar bozulmadan iletildi.
    - Windows'ta `${CLAUDE_PLUGIN_*}` ileri eğik çizgiyle yerine konur (plugins-reference);
      boşluklu ve karışık ayraçlı yolla denendi, çalışıyor.
    - `.cmd` dosyaları yalnız ASCII olmalı: `cmd` dosyayı OEM kod sayfasıyla okur, Türkçe yorum
      satırı komut ayrıştırmasını bozdu (denendi). Kural CLAUDE.md'de.
    - Mac/Linux'ta `cmd.exe` yok; başlatılamayan hook "Failed with non-blocking status code"
      bildirimi üretir, hook'larda platform alanı da yok. Bu yüzden 16. karar.
14. **Exe'nin yeri: `${CLAUDE_PLUGIN_DATA}/bin/`.** Exe sürümü plugin sürümüyle uyuşmazsa yenisi
    indirilir. Kaynak: Alperen.
    - Davranış kaynağı: Claude Code, *Plugin manifest reference → Environment variables*
      (https://code.claude.com/docs/en/plugins-reference#environment-variables, 2026-10-01'de okundu):
      "`${CLAUDE_PLUGIN_DATA}` | `~/.claude/plugins/data/<id>/`, created on first reference and kept
      across plugin updates." ve "By default, Claude Code deletes the `${CLAUDE_PLUGIN_DATA}` directory
      when you uninstall the plugin from the last place it's installed." (`--keep-data` istisnası:
      plugin uninstall belgesi.)
    - Aynı belgeye göre bu değişkenler hook süreçlerine aktarılır, ama Claude'un Bash aracıyla
      çalıştırdığı komutlara aktarılmaz. Bu yüzden `/claude-pet` skill'i yolu ortamdan değil, skill
      metnine yazılan `${CLAUDE_PLUGIN_DATA}` ile alır (Claude Code yüklerken yerine koyar).
    - Pet verileri (`state.json`, `chat.json`, `window.json`, petler) `~/.claude-pet`'te kalır;
      plugin kaldırılınca yalnız exe gider, kullanıcının petleri silinmez.

## 2026-09-30 (kapsam oturumu)

Proje açık kaynak bir Claude Code plugin'i olacak ve tek bir pete bağlı kalmayacak.
Ayrıntılı gerekçeler: `PROJE.md`.

1. **Dağıtım: Claude Code plugin + GitHub Actions ile derleme.** Plugin; `/claude-pet` skill'ini,
   hook'ları ve başlatıcıyı taşır. Actions her platform için uygulamayı derleyip Releases'e koyar,
   plugin ilk çalıştırmada indirir. Neden: kullanıcıda Python/Node gerekmesin, kurulum tek komut olsun.
   Kaynak: Alperen.
2. **Teknoloji: Tauri (Rust + web arayüzü).** Önceki Python + tkinter kararının yerini alır.
   Neden: gerçek piksel saydamlığı, balon ve animasyon HTML/CSS ile kolay, küçük çıktı, hazır Actions
   desteği. Rust ve VS C++ Build Tools kurulacak. Kaynak: Alperen onayı.
3. **Pet formatı: Codex formatı + isteğe bağlı `claude-pet.json`.** Codex'in resmi bir pet.json şeması
   yok. Resmi kaynak (openai/skills `hatch-pet`) 4 alan tanımlıyor: `id`, `displayName`, `description`,
   `spritesheetPath`. `spriteVersionNumber` yalnız toplulukta geçiyor. Bu 5 alanın dışında alan okunmaz.
   v1/v2 ayrımı görsel boyutundan yapılır (1536×1872 → 8×9, 1536×2288 → 8×11). Kare sayısı boş
   hücrelerin (alfa = 0) atlanmasıyla bulunur. Ek ayarlar ayrı `claude-pet.json`'da durur, böylece
   aynı klasör hem Codex'te hem bizde çalışır. Kaynak: Alperen.
   - **v1 petlerde bakış satırları yoktur:** fareye bakma sessizce kapanır, hata ya da uyarı verilmez.
     Kaynak: Alperen.
4. **Komutlar:** `/claude-pet` (aç/kapa), `list`, `use <id>`, `install <yerel-klasör>`. GitHub URL ile
   kurulum sonraki aşamaya ertelendi. Kaynak: Alperen.
5. **Durum köprüsü: `~/.claude-pet/state.json`.** Yazma atomiktir (önce geçici dosya, sonra yeniden
   adlandırma, Windows'ta kısa tekrar denemesi). İlk sürümde son yazan kazanır, ama her kayıtta
   `session_id` tutulur. Kaynak: Alperen.
6. **Hook'u exe'nin kendi alt komutu çalıştırır:** `claude-pet hook <olay>`. Pencere açmaz, çıkış kodu
   her zaman 0'dır. Exe yoksa hook sessizce çıkar, Claude Code'u asla bozmaz ya da yavaşlatmaz
   (varlık kontrolü için küçük bir sarmalayıcı gerekir). Kaynak: Alperen.
7. **Pet mesajları ayrı bir pet oturumuna gider:** ilk mesaj `claude -p --output-format json` ile gönderilir,
   sonrakiler `--resume <id>` ile devam eder. Araç izni verilmez. Balonda "Terminalde aç" düğmesi
   `claude --resume <id>` çalıştırır. Terminale devredilen oturuma pet bir daha yazmaz, yeni oturum
   açar. Oturumun klasörü son aktif Claude `cwd`'sidir ve oturum açılınca sabitlenir. Balonda klasör
   etiketi görünür. Belgeden doğrulandı: `-p` oturumları `claude --resume <session-id>` ile açılabilir.
   Balonda açık bir **"Yeni sohbet"** düğmesi bulunur ve saklı `session_id`'yi sıfırlar. Windows'ta
   `claude -p` çağrılırken **konsol penceresi açılıp kapanmaz** (gizli süreç olarak başlatılır).
   Kaynak: Alperen.
8. **Hareket: pet yerinde durur,** yalnız sürüklenince yer değiştirir. Boştayken kısa küçük hareketler
   yapar. Durum → animasyon eşlemesi pete özeldir ve `claude-pet.json`'da tanımlanır, yoksa Codex
   varsayılanları kullanılır. Kaynak: Alperen.
   - **Ek spritesheet:** aynı ızgara (192×208, en fazla 8 sütun, satır sayısı serbest), satırlar isimle
     çağrılır. Hatalı giriş atlanır, pet bozulmaz. `formatVersion` alanı bulunur. Kaynak: Alperen.
9. **Varsayılan pet** (2026-09-30 güncellendi): geliştirmede varsayılan pet `clawd-fan`
   (`dev-pets/clawd-fan/`, gitignore). Anthropic tasarımından esinlenen bir hayran işi olduğu için
   repoya girmez. **Yayındaki varsayılan pet Aşama 6'da kararlaştırılacak.** Johnny de repoya gömülmez,
   örnek pet olarak kalır. Kaynak: Alperen.
10. **Lisans: MIT (kod).** (2026-09-30 güncellendi) clawd-fan hayran işi olduğu için ona CC BY
    verilemez. Pet görsellerinin lisansı ve `LICENSE-ASSETS` gerekip gerekmediği, yayındaki varsayılan
    pet seçilince (Aşama 6) belirlenecek. Kaynak: Alperen.
11. **Johnny repodan çıkarılır:** yerel geliştirme için `.gitignore`'daki `dev-pets/johnny/` klasörüne
    taşınır, ilk commit Johnny olmadan yeniden oluşturulur. Kaynak: Alperen.
12. **Dizine başvurudan önce** Anthropic'in plugin dizini başvuru kriterleri ve marka yönergeleri
    okunur. Kaynak: Alperen.
13. **Spritesheet arayüze Tauri asset protokolüyle yüklenir.** IPC'den gelen bayt dizisi (JSON)
    yerine. Kapsam yalnız pet klasörleridir: aktif pet klasörü çalışma anında izne eklenir, statik
    kapsam boştur. Kaynak: Alperen.

## 2026-09-30 (ilk kurulum)

- **Ayrı git reposu (`D:\claude-pet`, dal `main`).** Proje bitince açık kaynak
  yapılıp paylaşılacak. Şimdilik yalnız yerel, remote yok. Kaynak: Alperen.
- **Johnny sprite'ı olduğu gibi kullanılacak.** Neden: Codex pet formatı zaten
  9 durum ve 16 bakış yönü içeriyor. Kaynak: Alperen.
- ~~**Teknoloji: Python + tkinter + Pillow.**~~ Yerini Tauri aldı (yukarıda 2. karar).
- **Önce sohbet, sonra hook köprüsü.** Neden: küçük ve görünür adımlar. Kaynak: planlama.
- **Kodlama ve planlama ayrı pencerede yapılacak**, ayrı bir repo klasöründe. Kaynak: Alperen.
