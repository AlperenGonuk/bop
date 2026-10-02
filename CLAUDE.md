# Bop — oturum talimatı

Claude Code için masaüstü maskotu (Codex pet benzeri), açık kaynak plugin. Çoklu pet destekli.
Kullanıcı: Alperen. Dil: Türkçe. Kısa, somut, tek adım tek adım.

## Oturum başında oku (sırayla)

1. `docs/DURUM.md`: neredeyiz, sıradaki adım
2. `docs/PLAN.md`: aktif aşama
3. Gerekirse `PROJE.md` (amaç, mimari) ve `docs/SPRITE.md` (atlas)

## Kurallar

- Kod yazmadan önce ilgili dosyayı oku, API'leri ve imzaları hafızadan uydurma.
- Küçük adımlarla ilerle: her adım çalıştırılıp görülebilir olsun.
- Önce Windows 11. Teknoloji Tauri (Rust + HTML/CSS/JS), ek bağımlılık için önce sor.
- `dev-pets/` ve dışarıdan gelen pet sprite dosyalarını değiştirme. Johnny yalnız yerelde
  `dev-pets/johnny/` içinde durur, repoya girmez. `app/src-tauri/default-pet/` motorun çıktısıdır:
  `plugin/skills/hatch/examples/pitir.json`'dan yeniden üretilir, elle düzenlenmez (KARARLAR 31).
- Pet sohbetinin (`claude -p`) araçları yalnız KARARLAR 18–20'dekiler: güvenilen klasörde yalnız
  dosya okuma (`Read`, `Glob`, `Grep`), güvenilmeyen sohbette yalnız `WebSearch` (boş pet
  klasöründe). İkisini aynı sohbette birleştirme. Yazan, komut çalıştıran ya da adres açan bir
  aracı (`Bash`, `Edit`, `Write`, `WebFetch` vb.) Alperen'e sormadan açma; böyle işler
  "Terminalde aç" ile yapılır.
- Claude Code davranışı (hook'lar, plugin, `claude -p`) için önce resmi belgeyi oku, hafızadan yazma.
- Hook'lar asla Claude Code'u bozmamalı ya da yavaşlatmamalı: hata durumunda sessizce çıkılır.
- **`.cmd` / `.bat` dosyaları yalnız ASCII içerir** (yorumlar dahil, İngilizce yaz). `cmd` dosyayı OEM
  kod sayfasıyla okur; Türkçe karakter komut ayrıştırmasını bozar (KARARLAR.md, 15. karar).
- İlk sürüm yalnız Windows (KARARLAR.md, 16. karar).
- Kullanıcı bilgisayar başındayken gerçek fare/klavye kullanan UI testlerini sormadan çalıştırma.

## Oturum sonunda (devir kuralı)

1. `docs/DURUM.md`: "Son oturum" ve "Sıradaki adım" bölümlerini güncelle.
2. `docs/PLAN.md`: biten maddeleri işaretle.
3. Kalıcı bir karar alındıysa `docs/KARARLAR.md` dosyasına tarihli satır ekle.
