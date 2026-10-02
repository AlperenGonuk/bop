# Spritesheet bilgisi

Codex pet formatı. Resmi bir belge sayfası yok; tanım OpenAI'ın `hatch-pet` skill'indeki
sözleşme (`codex-pet-contract.md`, `animation-rows.md`, `validate_atlas.py`). Kararlar:
[KARARLAR.md](KARARLAR.md) 3. karar. Karşılaştırma: [arastirma/3-hatch-tasarim.md](arastirma/3-hatch-tasarim.md) 2.5.

## `pet.json`

```json
{
  "id": "pitir",
  "displayName": "Pitir",
  "description": "A code-drawn mandarin sprout, the default Bop pet.",
  "spriteVersionNumber": 2,
  "spritesheetPath": "spritesheet.png"
}
```

- Uygulama yalnız bu 5 alanı okur; sürümü görsel boyutundan bulur.
- `spriteVersionNumber: 2` Codex'te **zorunlu**: yazılmazsa Codex peti v1 sayar ve 2288 piksel
  yüksekliğindeki sheet'i reddeder. `bop hatch` her zaman yazar.
- Görsel PNG ya da WebP, saydam RGBA.

## Sürümler (görsel boyutundan anlaşılır)

| Sürüm | Boyut | Izgara | Bakış satırları |
|---|---|---|---|
| v1 | 1536 × 1872 | 8 × 9 | yok, fareye bakma sessizce kapalı |
| v2 | 1536 × 2288 | 8 × 11 | 9–10. satırlar + (0,6) nötr hücre |

- Hücre: 192 × 208 piksel, saydam RGBA
- Kare konumu: `x = sütun * 192`, `y = satır * 208`
- Kullanılmayan hücreler tamamen saydam; saydam piksellerde RGB de 0 olmalı (Codex
  doğrulayıcısı "transparent RGB residue" hatası verir).
- Kare sayısı uygulamada boş hücreler (alfa kanalı tamamen 0) atlanarak bulunur.

| Satır | Durum | Kare | Codex kare süreleri (ms) | Anlamı |
|---|---|---|---|---|
| 0 | idle | 6 (+ v2'de (0,6) nötr) | 280, 110, 110, 140, 140, 320 | nefes, göz kırpma; durağan olamaz |
| 1 | running-right | 8 | 120 ×7, son 220 | sağa koşu |
| 2 | running-left | 8 | 120 ×7, son 220 | sola koşu |
| 3 | waving | 4 | 140 ×3, son 280 | başlangıç, kalkık el, dönüş |
| 4 | jumping | 5 | 140 ×4, son 280 | hazırlık, kalkış, tepe, iniş, oturma |
| 5 | failed | 8 | 140 ×7, son 240 | üzgün/sönük |
| 6 | waiting | 6 | 150 ×5, son 260 | onay/girdi bekleyen istekli poz |
| 7 | running | 6 | 120 ×5, son 220 | iş/işlem (ayakla koşu değil; ör. dizüstü) |
| 8 | review | 6 | 150 ×5, son 280 | odaklı inceleme |
| 9 | look 0°–157.5° (yalnız v2) | 8 | – | bakış yönleri |
| 10 | look 180°–337.5° (yalnız v2) | 8 | – | bakış yönleri |

- Kare süreleri bilgi içindir; uygulama tek `frameMs` kullanır (`bop.json`, aşağıda).
- Johnny ve clawd-fan kare sayıları uygulamada doğrulandı (2026-09-30): idle 7 dolu hücre
  (6 + nötr). Johnny dosyaları: `dev-pets/johnny/` (yalnız yerelde, repoda değil).

### (0,6) nötr hücre (v2)

v2'de satır 0, sütun 6 **dolu olmalıdır**: ön/nötr poz (Codex `neutralLookFrame`, en az 50 piksel).
İmleç ölü bölgedeyken gösterilen pozdur; idle karesi değildir. Önerilen içerik: idle'ın ilk karesiyle
aynı. Uygulama (sprites.js) v2'de bu hücreyi idle döngüsünden çıkarır (`neutralFrame`); imleç ölü
bölgedeyken ve boşta "etrafa bakınma" hareketinin başında/sonunda bu poz gösterilir (v1'de ya da
hücre yoksa idle 0). Etrafa bakınma: nötr → sağ → sol → kısa yukarı → nötr, yalnız ön yarım daire.

### Ayna ve satır kuralları

- `running-left`, `running-right`'ın aynası olabilir ama yalnız kimlik ve el tercihi (prop hangi
  elde) bozulmuyorsa; **kare sırası korunur** (her kare yerinde aynalanır). `bop hatch` aynalamaz,
  sola dönük yeniden çizer (ışık sol üstte kalır).
- Efektler pete değmeli (kopuk yıldız, nokta, soru işareti, düşünce balonu, hız çizgisi, gölge
  yok); `waiting`, `running`, `review`, `failed` birbirinden ayırt edilebilir olmalı.
- Bütün sprite'ı döndürerek bakış taklidi yapılmaz: gözler önden gider, kafa izler, üst gövde
  hafifçe izler, ayaklar sabit; taban çizgisi ve ölçek nötrle aynı.

## Bakış satırları (v2)

16 yön, 22.5° adım, saat yönünde; satır 9 = 0°–157.5°, satır 10 = 180°–337.5°.
**0° yukarı, 90° sağ, 180° aşağı, 270° sol** (Codex sözleşmesi: 180° aşağı bakıştır, arkasını
dönme değil). Eski petlerde 180° farklı çizilmiş olabilir (Johnny'de arkadan görünüş). Bu yüzden
uygulamada fareye bakma yalnız ön yarım daireyi (−90°…+90°) kullanır; imleç aşağıdaysa pet yana
ya da öne bakar.

`bop hatch` bakışı oranlarla türetir: gözbebeği 1.0, kafa ~0.6, gövde ~0.25 (sözde-3B dönüş),
ayaklar sabit. Yön koordinattan ölçülerek doğrulanır.

## `bop.json` (isteğe bağlı)

```json
{
  "formatVersion": 1,
  "frameMs": 140,
  "sheets": { "extra": "extra.webp" },
  "animations": {
    "yawn":    { "sheet": "extra", "row": 0, "frameMs": 120 },
    "monocle": { "sheet": "extra", "row": 1 },
    "slow-idle": { "row": 0, "frameMs": 250 }
  },
  "states": { "thinking": "monocle", "writing-code": "yawn" },
  "idleExtras": ["yawn", "look"]
}
```

- `frameMs`: tüm animasyonların varsayılan kare süresi (30–2000 ms; yoksa 150).
- `sheets`: ek sheet'ler. Aynı ızgara: hücre 192 × 208, en fazla 8 sütun, satır sayısı serbest.
  `main` ayrılmış addır (ana sheet). Yol pet klasörünün içinde olmalı.
- `animations`: isimli satırlar. `sheet` verilmezse ana sheet. Codex adlarını (`idle`, `running` …)
  ezebilir.
- `states`: durum → animasyon. Durumlar: `idle`, `running`, `thinking`, `reading`, `writing-code`,
  `running-command`, `waiting-permission`, `done`, `failed`. Tanımsız ya da animasyonu olmayan
  durum üst duruma düşer (`reading`/`writing-code`/`running-command`/`thinking` → `running` → `idle`).
  Varsayılan (Codex): `thinking`→`review`, `running`→`running`, `waiting-permission`→`waiting`,
  `done`→`jumping`, `failed`→`failed`.
- `idleExtras`: boşta 15–30 sn arayla oynayan kısa hareketler. `look` = etrafa bakma (yalnız v2).
  Yoksa varsayılan: v2'de `["look"]`, v1'de hiçbiri.
- Hatalı giriş (dosya yok, ızgara uymuyor, satır yok, bilinmeyen durum/ad) atlanır ve uyarı olur;
  pet bozulmaz. v1 pette `look` sessizce düşer.
