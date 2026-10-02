# Durum

**Güncelleme:** 2026-10-02

## Şu an

Aşama 1–4 bitti; uygulama tek başına çalışan bir pet: sürüklenir, hook'larla Claude'un durumunu
gösterir, üzerinden sohbet edilir (terminal havasında balon), pete özel `bop.json` okunur.
2026-10-02: başvuru ve teknik şartlar araştırıldı (`docs/arastirma/`), kararlar 21–27: plugin adı
`bop`, her şey İngilizce, ilk sürüm Windows + macOS + Linux, exe Releases'tan, kodla pet çizen
`hatch` skill'i (Rust), varsayılan pet Pıtır. Plugin `plugin/` altında: manifest, hook'lar, `/bop`
ve `hatch` skill'leri. Aşama 6 (kodla pet çizimi) kodu ve testleriyle bitti, gerçek oturumda
denenmedi.

Elle çalıştırma (plugin olmadan):

```
cd app\src-tauri
cargo build --release
target\release\bop.exe --pet D:\claude-pet\dev-pets\clawd-fan
```

Bu repoda açılan Claude oturumları `.claude/settings.json` hook'larıyla `~/.bop/state.json`'a
yazar (hook repodaki **debug** exe'yi arar, yani önce `cargo build` gerekir).

## Sıradaki adım: repo aç ve ilk sürüm (Alperen'in onayıyla)

Yerel hazırlık bitti (KARARLAR 33–34). Repo `AlperenGonuk/bop` henüz yok. Sıra:

1. `gh repo create AlperenGonuk/bop --public` ve temiz `main`'i push (onay gerekir).
2. `plugin.json`'a `homepage`, `repository`, gizlilik ve destek linkleri (repo açılınca; URL
   çözülmezse plugin yüklenmez).
3. CI'ın geçtiğini gör; `v0.1.0` etiketi → Actions taslak release → Alperen "Publish".
4. Gerçek kurulum: marketplace ekle, `bop@bop` kur, `/bop setup`, `/bop` (Windows, WSL; macOS
   yalnız Actions). Veri klasörü `~/.claude/plugins/data/bop-bop/`.
5. Dizin başvurusu öncesi: `bop` adı portalda alınmış mı; claude.ai/directory/manage.

Bilinen sınırlar: macOS/Linux'ta "Open in terminal", "Make a new pet" ve kaynak bağlantıları
henüz "Windows only" (README yazıyor). Wayland'da her zaman üstte/konum/fare konumu yok.

## Sınırlı kalanlar

- **Fareye bakma** yalnız ön yarım daire (−90°…+90°): 180° Johnny'de arkadan görünüş, clawd-fan'da
  aşağı bakış. İstenirse `bop.json`'a bakış kipi ayarı (biçim kararı gerekir).
- **Hook'lar async:** ardışık olaylar nadiren ters sırayla yazılabilir; görsel etkisi kısa.
- **Tıklama alanı:** petin saydam kenarları da tıklamayı yakalar.
- **"Claude çalışırken üstünde balon":** PROJE.md'de var, PLAN maddesi değil; şimdilik yalnız animasyon.
- **"Terminalde aç"** gerçek terminalde elle denenmedi (sahte `claude` ile doğrulandı).
- **Hook'ların PowerShell yolu** (Git'siz Windows) ve macOS `/bin/sh` gerçek Claude oturumunda denenmedi.

## Açık sorular

- Exe sürüm uyumu kontrolünün biçimi (Aşama 5, Actions ile birlikte).
- `hatch` varsayılanları (KARARLAR 30) Alperen'in onayını bekliyor: kopuk efekt yok, 48×52
  ızgara, ara onay noktası yok (basit istekte soru yok; kurulumdan sonra "geçeyim mi?").
- Pet görselleri için ayrı lisans gerekir mi (Pıtır bizim; KARARLAR 9, 10, 25).

## Son oturum

- 2026-10-02 (9): Yayın hazırlığı (alt ajan): `/bop setup` + `install.ps1/.sh` (yerelde sahte
  release'le Windows, Git Bash, WSL denendi), `bop --version`, Actions (release taslak + ci),
  marketplace.json, LICENSE, İngilizce README, PRIVACY.md, Tauri 2.12.1. Denetim (alt ajan):
  yazar e-postası tüm commit'lerde, iki raporda kişisel yol; temizlendi, ikon Pıtır'dan,
  geçmiş tek commit'le yeniden başladı (KARARLAR 34). Eski geçmiş: yerel `backup/pre-public` dalı.
- 2026-10-02 (8): Aşama 6 (`1ccc346`) üstüne `/code-review high` (10 bulgu): hepsi düzeltildi
  (override'lar küçültmede taşınır, override taşınca küçültme yok, etrafa bakınma nötr pozla ve
  ön yarım dairede, Pıtır testi %0.5 toleranslı, `depth`/`width` ≤ 0 hata, laptop/kol kuralı,
  piksel rengi tek yardımcı, hatch hızlandı; exe konsol alt sistemi, KARARLAR 32). Sprite kuralı
  netleşti (KARARLAR 31, CLAUDE.md). Arayüz İngilizce (balon, menü, talimat, pet yükleme hataları).
  Testler 42/42. Alperen'in görmesi gereken: İngilizce balon ve menü, etrafa bakınma.
- 2026-10-02 (8): Aşama 6, `hatch` (alt ajan, Alperen "bitene kadar sorma"). Rust motoru
  `app/src-tauri/src/hatch/` (spec; rig: elips üzerinde sözde-3B yaw/pitch, derinlik sırası,
  kontur; motion: 11 satır; validate; atlas + kontak sayfası), `png = "0.18"` (KARARLAR 27).
  `bop hatch <spec> <klasör>`, `--example`, `--check <png>`. Doğrulama: atlas biçimi, (0,6),
  kırpılma, durağan satır, kopuk parça, bakış sürekliliği; yön semantiği göz piksellerinden
  ölçülür; taşan pet %80'e kadar küçültülür. Pıtır `plugin/skills/hatch/examples/pitir.json`'dan
  yeniden üretildi (`default-pet/`), eski denemenin kopuk efektleri yok. Skill
  `plugin/skills/hatch/` (SKILL.md + references 3 dosya + 4 örnek). SPRITE.md Codex sözleşmesine
  göre yazıldı; sprites.js v2'de (0,6)'yı idle'dan çıkarıyor. `cargo test` 39 test, validate
  --strict geçti. Uygulama elle denenmedi, gerçek `claude` çağrısı: 0. Commit yok (KARARLAR 30).
  **Spec ya da motor değişince** `bop hatch plugin/skills/hatch/examples/pitir.json <tmp>` ile
  `default-pet/` yenilenmeli, yoksa `gomulu_pitir_motorun_ciktisi` testi düşer.
- 2026-10-02 (7): Kod yok, araştırma ve karar oturumu. Alt ajanlarla: kodla pet çizim denemesi
  ("Pıtır", PowerShell + C#, tam v2 atlas; Alperen beğendi, deneme kodu yayın öncesi çıkarıldı),
  dizin/marka, teknik, hatch tasarımı, çapraz platform araştırmaları (`docs/arastirma/1-4`).
  Kararlar 21–27. PLAN Aşama 5–7 yeniden yazıldı. Sonra 2026-10-01 işi elle denendi (9 madde,
  hepsi tamam; 6: sağ tık menüsünden sonra balon açık kalıyor, Alperen için sorun değil, boşluğa
  tıklayınca kapanıyor) ve commit'lendi. Alperen güvenilen sohbete web aramasını açmak istedi;
  auto mode denetleyicisi engelledi, Alperen "böyle kalsın" dedi (KARARLAR 20 geçerli).
  Aktif pet seçimi karara bağlandı (KARARLAR 28: sağ tık "Pet değiştir" ve "Yeni pet yap").
  Yeniden adlandırma `claude-pet` → `bop` yapıldı: exe `bop.exe`, `~/.bop` (eski `~/.claude-pet`
  silinmedi, veriler kopyalandı), `bop.json`, `BOP_*` ortam değişkenleri, `hooks/bop-hook.cmd`,
  Tauri kimliği `io.github.alperengonuk.bop`. Repo klasörü hâlâ `D:\claude-pet`.
  Hook'lar tek polyglot komuta geçti, `.cmd` kalktı; plugin `plugin/` altında (manifest + hooks),
  validate --strict geçti (KARARLAR 29).
  Exe: `pets.rs` (kurulu petler, `config.json`, gömülü Pıtır `app/src-tauri/default-pet/` →
  `~/.bop/pets/pitir/`, tek örnek `running.json` kalp atışı + `control` dosyasıyla kapatma),
  alt komutlar `list/use/install/toggle/stop`, sağ tık "Pet değiştir ▸", canlı değişim
  (`pet-changed` → sayfa yeniden yüklenir). Alperen denedi: değişim ve Pıtır animasyonları tamam.
  Yerelde `~/.bop/pets/` içine clawd-fan ve Johnny kuruldu (repoya girmez).
  `/bop` skill'i (`plugin/skills/bop/SKILL.md`, İngilizce, `disable-model-invocation`,
  `allowed-tools` exe yoluyla). `claude -p "/bop:bop list" --plugin-dir plugin` çalıştı, izin reddi
  yok. `--plugin-dir` veri klasörü: `~/.claude/plugins/data/bop-inline/` (debug exe oraya elle
  kopyalandı). Açık: `-p`'de önieksiz `/bop list` skill'e gitmedi (Claude proje bağlamıyla repo
  exe'sini çalıştırdı); etkileşimli oturumda `/bop` denenmeli.
  Önieksiz `/bop` çalışıyor (Claude Code ekranda `/bop:bop` diye gösteriyor; benim `-p` denemem
  Git Bash'in `/bop` argümanını yola çevirmesiyle bozulmuştu, `MSYS_NO_PATHCONV=1` gerekir).
  Sağ tık "Yeni pet yap": yeni terminalde `~/.bop/hatch`'te `claude "Let's make a new Bop pet!"`
  (Alperen denedi; skill olmadığı için Claude kendi sorularını uydurdu: karakter, ad, aktif mi).
  Gerçek `claude` çağrısı: ~11 (Alperen ~6, skill denemeleri 5).

- 2026-10-01 (6): Kod incelemesi (`806464d..HEAD`, 10 bulgu); 1–8 düzeltildi, küçük olanlar (9
  `.gitattributes` `*.sh`, 10 `autoGrow` sabitleri) bekliyor. Sohbet: kayıp oturum kontrolü yalnız
  başarısız çalışmada, ölü kimlik hemen silinir, deneme dahil toplam 5 dk, yeni oturuma geçişte
  balonda not. Balon: cevap beklenirken / terminal ve sağ tık menüsü odağı alınca kapanmaz; sınırlar
  tüm animasyonlardan; `~` yalnız gerçek ev dizini. Hook: KARARLAR 17. Alperen balon düzeltmelerini
  (terminal, sağ tık, beklerken odak) ve kayıp oturum notunu denedi, sorun yok. Sonra pet sohbetine
  yalnız okuyan araçlar açıldı (KARARLAR 18), kısa cevap talimatı, katlanır "Kaynaklar". İkinci
  inceleme (10 bulgu) hepsi düzeltildi: klasör güven sorusu (KARARLAR 19), parantezli/virgüllü
  bağlantılar (`rundll32`), süreli `keepOpen`, arka planda gelen cevapta 20 sn sonra kapanma,
  kaynak ayırmada yalnız bilinen başlıklar, yol karşılaştırması klasör adıyla, sprite hücreleri bir
  kez taranır, ev dizini tek yerde (`state::user_home`), CLAUDE.md araç kuralı güncellendi. Güven
  sorusu elle denenmedi. Üçüncü inceleme (10 bulgu) hepsi düzeltildi: KARARLAR 20 (dosya ya da
  web; güvenilmeyen sohbet boş klasörde), `<...>` adresler, balon kapanması tek kurala bağlandı
  (odak yoksa kapan, 500 ms yoklama; menü/devir/arka plan cevabı için süreli istisna), ev dizini
  uyarısı üst klasörlerde de, güven kutusu ev dizini geç gelince yeniden çizilir, çift cevap
  engellenir, kaynak bağlantılarında href yok, soluk sheet balonu kaydırmaz. Hiçbiri elle
  denenmedi. Gerçek `claude` çağrısı: ~8 (Alperen'in denemeleri). Commit yok.
- 2026-10-01 (5): Alperen peti denedi. Düzeltme: pet, başlatıldığı Claude oturumunun değişkenlerini
  devralıyordu (transkript kaydı ve renkler kapanıyordu) → temizleniyor, kayıp oturumda yeni oturum
  (`1e8bad0`). Balon: terminal havası, pete yanaşma, dışarı tıklayınca kapanma (`25e75d2`, Alperen
  onayladı). Kararlar 14–16 ve `.cmd` sarmalayıcı (`b943daa`). Gerçek `claude` çağrısı: 0.
- 2026-09-30 (4): Asset protokolü; Aşama 1–4 tamamlandı. Gerçek `claude` çağrısı: 2.
- 2026-09-30 (3): Rust kuruldu, Tauri iskeleti, pet yükleyici ve idle.
- 2026-09-30 (2): Kapsam oturumu, 12 karar. Johnny repodan çıkarıldı.
- 2026-09-30 (1): Fikir netleşti. Klasör, belgeler ve asset'ler oluşturuldu.
