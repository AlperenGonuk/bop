# Bop — Claude Code için masaüstü maskotu

> Codex'teki pet özelliğinin Claude Code CLI sürümü. Açık kaynak bir Claude Code plugin'i.
> Masaüstündeki pet, Claude'un ne yaptığını animasyonla gösterir ve üzerinden mesaj atılabilir.

## Amaç

- `/bop` komutuyla açılıp kapanan, her zaman üstte duran ve sürüklenebilen bir maskot.
- **Tek pete bağlı değil:** hazır Codex petleri olduğu gibi çalışır, kullanıcı kendi petini de yapabilir.
- Claude Code çalışırken pet **durumu yansıtır:** düşünüyor, kod yazıyor, izin bekliyor, bitti, hata.
  Çalışırken üstünde balon çıkar.
- Pete tıklayıp **mesaj atmak:** cevap balonda görünür, istenirse sohbet terminale devredilir.
- Geliştirmede varsayılan pet clawd-fan, deneme peti Johnny (ikisi de yalnız yerelde, `dev-pets/`).
  Yayındaki varsayılan pet Aşama 6'da seçilecek.

## Kapsam dışı (şimdilik)

- Ses, sesli konuşma
- Kendi kendine ekranda gezinme (pet yerinde durur)
- GitHub URL ile pet kurma (sonraki aşama)
- Aynı anda birden fazla Claude oturumunu ayrı ayrı gösterme (`session_id` şimdiden tutulur)

## Mimari

```
PLUGIN (Claude Code)
  /bop skill ──► bop.exe (aç/kapa, list, use, install)
  hooks.json (sh/PowerShell tek komut) ──► bop hook <olay> ──► ~/.bop/state.json
                 (exe yoksa sessiz çıkış)       (atomik yazma)            │
                                                                          ▼
UYGULAMA (Tauri, GitHub Actions ile derlenir, Releases'ten indirilir)
  saydam pencere ◄── state.json'u izler ──► durum → animasyon (pete özel)
       │
       └─ mesaj ──► claude -p --output-format json [--resume <id>] ──► balon
                    (gizli süreç, araç izni yok, sabit klasör)
                    balonda: "Yeni sohbet" · "Terminalde aç" (claude --resume <id>)
```

## Pet formatı

Pet klasörü: `~/.bop/pets/<id>/`

| Dosya | Zorunlu | İçerik |
|---|---|---|
| `pet.json` | evet | Codex formatı. Yalnız `id`, `displayName`, `description`, `spritesheetPath`, `spriteVersionNumber` okunur |
| `spritesheet.webp` | evet | v1: 1536×1872 (8×9), v2: 1536×2288 (8×11). Sürüm görsel boyutundan anlaşılır |
| `bop.json` | hayır | Bizim uzantımız: ek sheet'ler, isimli animasyonlar, durum eşlemesi, boşta hareketler |

- v1 petlerde bakış satırları yoktur, fareye bakma sessizce kapanır.
- Ek sheet'ler aynı ızgarayı kullanır (192×208, en fazla 8 sütun). Hatalı giriş atlanır.
- Ayrıntı: [docs/SPRITE.md](docs/SPRITE.md), kararlar: [docs/KARARLAR.md](docs/KARARLAR.md).

## Teknoloji

- Tauri (Rust + HTML/CSS/JS): pencere, saydamlık, animasyon, balon
- GitHub Actions: platform derlemeleri → GitHub Releases
- Claude Code: plugin sistemi, hook'lar, `claude -p`

## Dikkat

- `claude -p` kullanıcının kendi plan limitinden yer. Bu README'de açıkça yazmalı.
- Arayüzsüz modda izin sorulamaz. Pet yalnız sohbet eder, izin gereken işler "Terminalde aç" ile yapılır.
- Geliştirme petleri (`dev-pets/`) yalnız yerelde durur, repoya girmez.
- "Claude" adı Anthropic'in markası; README'de "resmi değildir" notu yer alır.
- Lisans: kod MIT. Varsayılan pet Pıtır kodla çizilir (`plugin/skills/hatch/examples/pitir.json`), aynı lisans.

## Belge sistemi

| Dosya | Ne için |
|---|---|
| `PROJE.md` | Bu dosya: amaç, kapsam, mimari. Nadiren değişir. |
| `CLAUDE.md` | Kodlama oturumunun başlangıç talimatı |
| `docs/DURUM.md` | Şu an neredeyiz, sıradaki adım. **Her oturum sonunda güncellenir.** |
| `docs/PLAN.md` | Aşamalar ve yapılacaklar listesi |
| `docs/KARARLAR.md` | Tarihli karar günlüğü |
| `docs/SPRITE.md` | Spritesheet teknik bilgisi |
