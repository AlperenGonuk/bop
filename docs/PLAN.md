# Plan

Kararlar: [KARARLAR.md](KARARLAR.md). Mimari: [../PROJE.md](../PROJE.md).

## Aşama 0: Hazırlık

- [x] Kapsam ve mimari kararları (2026-09-30)
- [x] Johnny'yi repodan çıkar → `dev-pets/johnny/` (gitignore), ilk commit'i yeniden oluştur
- [x] Rust (`rustup`, kullanıcı düzeyi); VS 2022 C++ araçları zaten kuruluydu
- [x] Geliştirme peti: clawd-fan → `dev-pets/clawd-fan/` (v2). Varsayılan pet kararı bekliyor, bkz. DURUM.md

## Aşama 1: Maskot ekranda (Tauri)

- [x] Tauri iskeleti: saydam, kenarlıksız, her zaman üstte pencere
- [x] Pet yükleyici: `pet.json` (5 alan), v1/v2 görsel boyutundan, boş hücre atlama
- [x] `idle` döngüsü (clawd-fan ve Johnny ile denendi)
- [x] v1 pet ile deneme: bakış sessizce kapalı, hata yok (geçici 1536×1872 test peti)
- [x] Sürükle-bırak + sürüklerken `running-left/right`
- [x] Sağ tık menüsü: kapat

## Aşama 2: Durum köprüsü (hook'lar)

- [x] Hook belgesini oku: olaylar, girdi alanları (`session_id`, `cwd`, `tool_name`), Windows shell, `timeout`
- [x] Durum sözlüğü ve hook → durum eşlemesi (`state.rs`, testli)
- [x] `claude-pet hook <olay>`: stdin → `state.json` (atomik yazma, çıkış kodu her zaman 0, ~25 ms)
- [x] Exe yoksa sessiz çıkan sarmalayıcı (`hooks/claude-pet-hook.sh` + `.ps1`)
- [x] Maskot `state.json`'u izleyip animasyonu değiştirir
- [x] Repo içi geliştirme hook'ları: `.claude/settings.json` (async, yalnız bu repo)

## Aşama 3: Sohbet

- [x] Tıklayınca mesaj kutusu (pencere yukarı büyür, yer yoksa balon altta; pet ekranda yerinde kalır)
- [x] `claude -p --output-format json` ve `--resume <id>`, araç yok (`--tools ""`, `mcp__*` kapalı, `--permission-prompts none`)
- [x] Windows'ta gizli süreç (CREATE_NO_WINDOW, beklerken konsol penceresi yok — denendi)
- [x] Oturum klasörü: kullanıcının son Claude `cwd`'si (pet oturumu ezmez), oturum açılınca sabitlenir, balonda klasör etiketi
- [x] Balon: cevap, "Yeni sohbet" düğmesi (session id sıfırlanır)
- [x] "Terminalde aç": yeni terminalde `claude --resume <id>`, devredilen oturuma pet yazmaz
- [x] Beklerken ve hata anında animasyonlar
- [x] Gerçek claude ile denendi (2 çağrı): cevap 5-6 sn, `--resume` bağlamı koruyor

## Aşama 4: Pet sistemi

- [x] `claude-pet.json` okuyucu: `formatVersion`, `frameMs`, `sheets`, `animations`, `states`, `idleExtras` (Rust, testli)
- [x] Ek sheet doğrulama (192×208, en fazla 8 sütun), hatalı giriş atlanır ve uyarı olur
- [x] Durum geri düşme zinciri (`writing-code` → `running` → `idle`)
- [x] Boşta küçük hareketler (15-30 sn arayla), fareye bakma (yalnız v2, ön yarım daire; v1'de sessizce kapalı)
- [x] Konumu hatırla (`~/.claude-pet/window.json`, ekran dışındaysa yok sayılır)

## Aşama 5: Plugin "bop" (Windows + macOS + Linux)

Kararlar: 21 (İngilizce), 23 (üç platform), 24 (Release'ten exe), 26 (ad `bop`), 27 (Rust çizim
motoru). Araştırma: [arastirma/](arastirma/) (1 dizin ve marka, 2 teknik, 4 çapraz platform).

- [x] Kararlar: exe `${CLAUDE_PLUGIN_DATA}/bin` (14), ilk sürüm üç platform (23, 16'yı değiştirir)
- [x] Başvuru şartları ve teknik şartlar araştırması (2026-10-02)
- [x] Commit'lenmemiş 2026-10-01 işlerini elle dene ve commit'le (2026-10-02)
- [x] Yeniden adlandırma: `claude-pet` → `bop` (exe, `~/.claude-pet` → `~/.bop`, kod ve belgeler)
- [x] Hook'lar: shell form, sh ve PowerShell'in ikisinin de okuyabildiği tek komut; exe yoksa stdin
      boşaltılıp 0 ile çıkılır (`arastirma/4-capraz-platform.md`). `.cmd` sarmalayıcı kalkar.
- [x] `plugin/.claude-plugin/plugin.json` (`name: bop`, `displayName`, MIT) + `plugin/hooks/hooks.json` (29)
- [x] `/bop` skill'i (aç/kapa, `list`, `use <id>`, `install <yerel-klasör>`) + exe alt komutları
- [x] Aktif pet seçimi kararı (28)
- [x] `~/.bop/pets/` + `config.json`, gömülü Pıtır geri düşmesi, canlı pet değişimi, exe alt komutları
      (`list`, `use`, `install`, `toggle`, `stop`), tek örnek
- [x] Sağ tık: "Pet değiştir ▸"
- [x] Sağ tık: "Yeni pet yap" (terminalde ilk mesaj "Let's make a new Bop pet!"; skill Aşama 6)
- [x] `claude --plugin-dir` ile deneme, `claude plugin validate --strict` (`/bop`, `hatch` akışı `-p` ile)
- [x] Exe indirme: GitHub Releases + sha256, README'de açıkça yazılır (`/bop setup`, 33; gerçek release'le denenmedi)
- [ ] Platform farkları: "Terminalde aç" (macOS/Linux), Tauri 2.12.1+ (macOS saydamlık),
      Wayland sınırları (her zaman üstte, konum, fare konumu) için zarif geri düşme
- [x] GitHub Actions: Windows, macOS, Linux derlemeleri → Releases (yazıldı, hiç çalıştırılmadı)
- [x] Uygulamanın Türkçe metinlerini İngilizceye çevir (21, 31)

## Aşama 6: Pet oluşturma skill'i (`hatch`)

Kararlar: 22 (API yok, kodla çizim), 27. Tasarım: [arastirma/3-hatch-tasarim.md](arastirma/3-hatch-tasarim.md).
Deneme (C#) Rust'a çevrildi: `app/src-tauri/src/hatch/`; deneme kodu yayın öncesi çıkarıldı.

- [x] `SPRITE.md`'yi Codex'in güncel sözleşmesiyle eşitle ((0,6) nötr hücre, `spriteVersionNumber`, 180° = aşağı, kare süreleri) (2026-10-02)
- [x] Rust çizim motoru: `<exe> hatch <spec.json> <çıktı>` (`png` crate); üç seviyeli spec
      (arketip+palet → parçalar → piksel haritası), sözde-3B gövde dönüşü (2026-10-02)
- [x] Deterministik doğrulama (atlas, hücre, yön semantiği) + contact sheet, otomatik sığdırma
- [x] İngilizce `skills/hatch/SKILL.md` (< 500 satır) + `references/`; çiz-bak-düzelt döngüsü
- [x] Pıtır'ı motorla yeniden üret → varsayılan pet (25)
- [ ] Gerçek oturumda dene: "Yeni pet yap" → skill → `hatch` → `install` → `use` (`-p` ile geçti; etkileşimli: Alperen)
- [ ] Uygulamada yeni Pıtır'ı elle gör (idle artık 6 kare, (0,6) nötr ayrı)

## Aşama 7: Açık kaynak yayını ve başvuru

- [x] `LICENSE` (MIT), README (kurulum, indirilenler, limit uyarısı, "resmi değildir" notu)
- [x] Gizlilik politikası (`PRIVACY.md`) ve iletişim (GitHub Issues); linkler repo açılınca `plugin.json`'a
- [x] Kişisel yol ve bilgi kalmadığını kontrol et (denetim + temiz tek commit, 34)
- [ ] Ad kontrolü (`bop` portalda alınmış/genel mi)
- [ ] Public GitHub reposu (Alperen'in açık onayıyla)
- [ ] claude.ai/directory/manage üzerinden başvuru (ücretli plan + public repo)
- [ ] Sonraki: `install <github-url>`
