# 3. Kodla çizilen pet skill'i: lisans, Codex'ten alınacaklar, tasarım

Tarih: 2026-10-02. Yalnız okuma ve tasarım; kod yazılmadı, repoya dokunulmadı.

İncelenen:
- `~\.codex\skills\hatch-pet\` (SKILL.md 924 satır, references/ 3 dosya, scripts/ 18 betik, tests/ 4, LICENSE.txt)
- Deneme: `scratchpad\hatch-deneme\Pitir.cs` (387 satır), `build.ps1`, `contact-sheet.png`, `pet\pet.json`
- `D:\claude-pet\docs\SPRITE.md`, `docs\KARARLAR.md` (3. ve 10. kararlar), `app\src-tauri\src\pet.rs`, `app\src\main.js`

---

## 1. Lisans

### Bulgular

| Ne | Lisans | Kaynak |
|---|---|---|
| Yereldeki hatch-pet skill'i | Apache-2.0 (`LICENSE.txt`, tam metin, telif satırı doldurulmamış şablon) | `~\.codex\skills\hatch-pet\LICENSE.txt` |
| Aynı skill, resmi depo | Apache-2.0 (`skills/.curated/hatch-pet/LICENSE.txt`) | https://github.com/openai/skills/tree/main/skills/.curated/hatch-pet |
| openai/skills deposu geneli | Depo düzeyinde lisans yok. README: "The license of an individual skill can be found directly inside the skill's directory inside the `LICENSE.txt` file." | https://github.com/openai/skills (README, "License" bölümü) |
| openai/codex (uygulama) | Apache-2.0 | https://github.com/openai/codex (GitHub API: `license.spdx_id = Apache-2.0`) |
| Pet biçimi (atlas boyutu, ızgara, satır düzeni, pet.json alanları) | Biçimin kendisi bir yazılım arayüzü / olgu; telifle korunan bir "eser" değil. Resmi bir belge sayfası yok, tanım skill'in `references/` dosyalarında. | https://github.com/openai/skills/blob/main/skills/.curated/hatch-pet/SKILL.md |

- Skill klasöründe `NOTICE` dosyası yok, betiklerde telif başlığı yok (`grep copyright` boş).
- Yereldeki kopya ile depodaki sürüm farklı olabilir (depodaki `openai.yaml` açıklaması "any pet-safe style" diyor, yereldeki "v2 pet with ... 16 look directions"). Lisans ikisinde de aynı.

### Sonuç

- **Uyarlayabiliriz.** Apache-2.0 metni ve betikleri kopyalamaya, değiştirmeye, MIT projede dağıtmaya izin verir. Koşullar (Madde 4):
  1. Uyarlanan dosyayla birlikte Apache-2.0 lisans metnini vermek,
  2. Değiştirilen dosyalara "değiştirildi" ibaresi koymak (4b),
  3. Varsa telif/NOTICE bildirimlerini korumak (burada yok),
  4. Madde 6: OpenAI/Codex markasını onay ima edecek biçimde kullanmamak ("Codex uyumlu" demek serbest, adı "Codex Hatch" yapmak değil).
- **Önerim: sıfırdan yaz, yalnız fikir ve olgu al.** Gerekçe:
  - Boru hattı temelden farklı. Onlarınki "görsel üret, chroma anahtarla ayır, bileşen çıkar, despill"; bizimki "parametreden çiz". Betiklerin %70'i (extract_strip_frames, despill_chroma_edges, assemble_extended_atlas kayıt/ölçek, blind A/B, imagegen iş listesi) bizde gereksiz.
  - Onların kuralı bizim yaptığımızı açıkça yasaklıyor: "Never substitute locally drawn, tiled, transformed, or code-generated row strips" (SKILL.md 880). Metni uyarlamak yerine kendi metnimizi yazmak daha temiz.
  - Betikler Python + Pillow. Biz Python istemiyoruz (KARARLAR 1: kullanıcıda Python/Node gerekmesin).
  - Saf MIT kalır, lisans dosyası taşımaya gerek kalmaz.
- **Atıf:** Zorunlu değil (kod kopyalanmazsa). Nezaket ve şeffaflık için README'de bir satır önerilir: "Atlas düzeni ve QA fikirleri OpenAI'ın Apache-2.0 lisanslı hatch-pet skill'inden esinlenmiştir." Eğer bir doğrulama fonksiyonu satır satır port edilirse (ör. `measure_direction_continuity.py` → C#) o dosyanın başına "Portions adapted from openai/skills hatch-pet, Apache-2.0, modified" yazılır ve `THIRD_PARTY_NOTICES` ya da `licenses/Apache-2.0.txt` eklenir. Eşik sayıları (2 px kenar payı, 1.15 alan oranı vb.) olgu sayılır, kopyalanması atıf gerektirmez.

---

## 2. Codex skill'inden API'den bağımsız değerli parçalar

### 2.1 Atlas sözleşmesi (`codex-pet-contract.md`, `animation-rows.md`)

- 1536×2288, 8×11, hücre 192×208, saydam, PNG ya da WebP, `spriteVersionNumber: 2`.
- Satır kare sayıları ve **süreleri** (bizde süre yok, SPRITE.md'ye eklenebilir):

| Satır | Durum | Kare | Süreler (ms) |
|---|---|---|---|
| 0 | idle | 6 | 280, 110, 110, 140, 140, 320 |
| 1 | running-right | 8 | 120 ×7, son 220 |
| 2 | running-left | 8 | 120 ×7, son 220 |
| 3 | waving | 4 | 140 ×3, son 280 |
| 4 | jumping | 5 | 140 ×4, son 280 |
| 5 | failed | 8 | 140 ×7, son 240 |
| 6 | waiting | 6 | 150 ×5, son 260 |
| 7 | running | 6 | 120 ×5, son 220 |
| 8 | review | 6 | 150 ×5, son 280 |
| 9 | look 000–157.5 | 8 | – |
| 10 | look 180–337.5 | 8 | – |

- Kullanılmayan hücreler tamamen saydam. Saydam piksellerde RGB = 0 olmalı ("transparent RGB residue" hatası).
- **v2'de (0,6) hücresi "nötr/ön" karedir ve dolu olmalıdır.** `validate_atlas.py`: `EXTENDED_NEUTRAL_LOOK_FRAME = (0, 6)`, v2'de bu hücre "used" sayılır, 50 pikselden azsa hata. `assemble_extended_atlas.py` bunu `neutralLookFrame: {rowIndex:0, columnIndex:6}` olarak yazar. SPRITE.md'deki "idle 7 kare (6 + nötr)" gözlemi bunun sonucu.
- 000° = yukarı, saat yönünde; nötr/ön = imleç ölü bölgesi, idle'a düşer.
- Satır anlamları: `running` = iş/işlem (ayakla koşu değil), `review` = odaklı inceleme, `waiting` = onay/girdi bekleyen istekli poz, `failed` = üzgün/sönük. `waiting`, `running`, `review`, `failed` birbirinden ayırt edilebilir olmalı.
- `running-left`: ayna ancak kimlik ve el tercihi (prop hangi elde) bozulmuyorsa; **kare sırası korunur** (şerit bütün olarak aynalanmaz, her kare yerinde aynalanır).

### 2.2 QA rubriği (`qa-rubric.md` + SKILL.md kuralları), bize uyan maddeler

- Kimlik: siluet, oran, yüz, palet, malzeme, işaretler, proplar 11 satırda aynı.
- 192×208'de okunur olmalı, detaylar pet boyutunda seçilmeli.
- İstenmemiş karakter/nesne/logo/yazı/sahne yok.
- Döngüler patlamamalı (size pop), kadansı ters dönmemeli, yanlış yöne bakmamalı, **durağan olmamalı** (idle'da 6 aynı kare kabul edilmez).
- İlk idle karesi "reduced motion" durağan görüntü olarak çalışmalı.
- Bakış: 16 yön sabit sırada, hepsi nötrden görsel olarak ayrışmalı; ana yönler (000/090/180/270) kesin okunmalı; çaprazlar doğru çeyrekte. **Bütün sprite'ı döndürerek/eğerek bakış taklidi yasak** (nesne zaten dönen bir şey değilse). Yeni "googly" gözler, göz yerine boncuk eklemek yasak. Ardışık yönlerde gövde kesintisiz ilerlemeli; taban/ayak sabit çapa, ölçek ve taban çizgisi nötrle aynı.
- **Göz/kafa/gövde hiyerarşisi:** gözler bakışı önden götürür, kafa izler, üst gövde hafifçe izler, alt gövde/ayak sabit. Yalnız göz bebeği kaydırmak "istisna"dır (uyarı).
- **Efekt kuralları:** yalnız duruma uygun, pete **değen/bağlı**, opak, küçük efekt. Kopuk yıldız, nokta, soru işareti, düşünce balonu, hız çizgisi, gölge, parıltı yasak. (Onlarda gerekçe chroma ayrıştırma; bizde teknik engel yok ama "pet boyutunda temizlik" gerekçesi geçerli.)
- Durum rehberi: idle yalnız nefes/göz kırpma; waving yalnız uzuv pozu (dalga çizgisi yok); jumping yalnız gövde konumu (gölge/toz yok); review yalnız eğilme/göz/kafa/el (büyüteç, kağıt yok, temel kimlikte yoksa).
- Onarım politikası: en küçük kapsamı onar (bir satır), her onarımdan sonra tüm deterministik doğrulamayı yeniden çalıştır.
- Yakınsama kuralı: aynı kök hata iki kez tekrarlarsa parametre oynamayı bırak, stratejiyi değiştir; onarım hatayı başka hücreye taşıyorsa bu bir döngüdür.

### 2.3 Deterministik doğrulama betikleri: ne denetliyor

| Betik | Denetim | Bizde |
|---|---|---|
| `validate_atlas.py` | boyut (v2 zorunlu), PNG/WebP, alfa kanalı var mı; satır başına used/unused hücre; used ≥ 50 piksel; unused = 0 piksel; (0,6) nötr dolu; hücre %95'ten fazla opak ise "arka plan kalmış"; saydam piksel RGB artığı; chroma sızıntı/saçak | Hepsi chroma hariç. C#'a taşınır. |
| `inspect_frames.py` | kare sayısı; kare boyutu 192×208; ≥ 400 piksel; kenarda 2 px şeritte > 24 piksel → kırpılma uyarısı; satır medyan alanının < %35 ya da > %275'i → aykırı uyarısı; çıkarım yöntemi | Kenar ve alan aykırılığı değerli. |
| `measure_direction_continuity.py` | 16 bakış hücresinde ardışık çiftler (337.5→000 dahil): fark piksel sayısı yerel aykırı (> komşu ort. ×1.45), bbox merkez kayması > 8 px, alan oranı > 1.15; gövde içinde yatay saydam "delik satırı" | Aynen değerli. |
| `make_contact_sheet.py` | dama arka planlı atlas, kullanılan hücre etiketli | Görsel QA için. |
| `make_direction_qa_sheet.py` | nötr + 16 yön, derece etiketli, ayrıca kafa/üst gövde yakın plan | Görsel QA için. |
| `render_animation_previews.py` | satır başına GIF (gerçek sürelerle) | Bizde GIF zor; HTML önizleme önerilir. |
| `make_direction_blind_qa_sheet.py` + `combine_*` + `validate_*` | rastgele A/B çiftleri, 3 bağımsız işçi, çoğunluk; ana yön çiftleri sert kapı | Bizde büyük kısmı gereksiz (aşağıda: yön denetimi koddan ölçülür). İsteğe bağlı tek bir kör alt ajan. |
| `derive_running_left_from_running_right.py` | kare kare yerinde ayna, sıra korunur | Bizde ayna yerine `facing=-1` ile yeniden çizim. |
| `extract_*`, `despill_*`, `assemble_extended_atlas.py`, `prepare_pet_run.py` | görsel üretim çıktısını ayıklama/kayıt | Gereksiz. Tek fikir: **ortak ölçek + nötrden alınan taban çizgisi ve alt gövde çapası.** |

### 2.4 İş akışı / ilerleme planı

Codex'in görünür kontrol listesi (4 adım) iyi bir kullanıcı deneyimi:
1. `<Pet>` hazırlanıyor (ad, açıklama, stil, klasör)
2. `<Pet>`'in ana görünüşü (kanonik temel kare)
3. `<Pet>`'in pozları (satır 0–8, bakış mekaniği planı, satır 9–10)
4. `<Pet>` yumurtadan çıkıyor (atlas, QA, paketleme)

Ayrıca: "bir adımı ancak gerçek dosya/karar varken tamamla"; zaman bütçesi (~30 dk); bakış satırlarından önce `look-mechanics.md` yazdırmak ("bu karakter etrafa bakarken en doğal hareket ne? ne sabit, ne önden gider, ne izler, ne bükülür?") ve önce 4 ana yönü onaylatıp sonra araları doldurmak. Bu son ikisi bizim için de çok değerli.

### 2.5 SPRITE.md ile farklar

1. **(0,6) nötr hücre:** SPRITE.md bunu yalnız "Johnny'de idle 7 kare (6 + nötr)" diye gözlem olarak yazıyor. Codex sözleşmesinde v2'de zorunlu, adı `neutralLookFrame`. Bizim uygulama boş olmayan hücreleri saydığı için nötrü idle döngüsünün 7. karesi olarak oynatıyor olabilir (doğrulanmalı). Skill her zaman (0,6)'yı doldurmalı (idle 0 ile aynı ön poz önerilir, böylece döngüde fark edilmez).
2. **`spriteVersionNumber`:** KARARLAR 3 "yalnız toplulukta geçiyor" diyor. Güncel Codex skill'i bunu **zorunlu** sayıyor: "Omitting it defaults the pet to v1 and causes the 2288-pixel-tall spritesheet to be rejected." Bizim uygulama görsel boyutuna bakıyor (doğru), ama ürettiğimiz pet.json Codex'te de çalışsın diye alanı **yazmalıyız**. Pitir'in `pet.json`'unda bu alan yok. KARARLAR 3'teki cümle güncellenmeli.
3. **Kare süreleri:** SPRITE.md'de yok (biz `frameMs` ile tek süre kullanıyoruz). Codex satır başına süre listesi tanımlıyor. Bilgi olarak eklenebilir; skill bunları `claude-pet.json`'a yazmaz (biçim tek `frameMs` destekliyor), yalnız önizlemede kullanır.
4. **180°:** SPRITE.md "petten pete farklı (Johnny arkadan, clawd-fan aşağı)" diyor. Codex sözleşmesi açık: 180 = **aşağı bakış**, arkasını dönme değil. Bizim skill aşağı bakış çizmeli.
5. **Ayna kuralı:** Codex "kare sırasını koruyarak, ancak kimlik bozulmuyorsa". SPRITE.md'de yok; skill belgesine girer.
6. Kopuk efekt yasağı, "running ≠ ayakla koşu" gibi satır anlamları SPRITE.md'de yok; skill'in `references/` belgesine girer.

---

## 3. Bizim kodla-çizim skill'i: tasarım önerisi

Hedef: **basit istek ("turuncu bir kedi") → arketip + palet + ön ayarla iyi sonuç; ayrıntılı istek → parça parça tanım ya da serbest kodla istenene yakın sonuç.** Boru hattı (atlas, hareket türetme, QA, paketleme) her durumda aynı kalır; yalnız "karakter çizimi" katmanı değişir.

### 3.0 Katmanlar

```
spec.json  ──►  Rig (parçalar, pivotlar, derinlik)  ──►  Pose (kare başına)  ──►  Raster (ızgara, palet, kontur)  ──►  Atlas + meta.json
   ▲                      ▲                                   ▲
 arketip şablonu     custom/*.cs (kaçış yolu)          Motion kuralları (satır üreticileri)
```

### 3.1 (a) Karakter tanım biçimi: `spec.json`

Üç ayrıntı seviyesi, aynı dosya:

**Seviye 1 (basit istek):** yalnız arketip + palet + birkaç seçenek.
```json
{
  "formatVersion": 1,
  "id": "pitir", "displayName": "Pitir", "description": "Tiny mandarin sprout.",
  "archetype": "blob",
  "style": "pixel",
  "palette": { "body": "#FFB066", "accent": "#8BD14A", "outline": "#2B2140" },
  "features": { "eyes": "dot", "mouth": "smile", "cheeks": true, "topper": "sprout", "feet": true, "arms": "nub" }
}
```

**Seviye 2 (ayrıntılı istek):** arketipin parçalarını ezme/ekleme.
```json
{
  "archetype": "biped",
  "grid": { "w": 48, "h": 52, "scale": 4 },
  "light": "top-left",
  "palette": {
    "fur":   { "base": "#E9A15B", "shade": "auto", "hi": "auto" },
    "belly": "#FBE3C4", "outline": "#2B2140", "eye": "#1C1626"
  },
  "parts": {
    "head":  { "shape": "ellipse", "size": [22, 18], "color": "fur", "parent": "body", "pivot": [0, -14] },
    "earL":  { "shape": "triangle", "size": [6, 8], "color": "fur", "parent": "head", "at": [-7, -8], "depth": 0.6 },
    "earR":  { "mirrorOf": "earL" },
    "tail":  { "shape": "curve", "length": 10, "width": 3, "color": "fur", "parent": "body", "at": [9, 6], "depth": -0.8, "follow": 0.4 },
    "scarf": { "shape": "band", "color": "#D9443A", "parent": "body", "at": [0, -9], "worn": true }
  },
  "face": { "eyes": { "type": "globe", "size": [3, 4], "gap": 9 }, "mouth": "cat", "brows": true },
  "props": { "laptop": { "use": "running" } },
  "asymmetric": ["scarf"]
}
```

Alan kuralları:
- `grid`: ön ayarlar `48×52 @4` (iri piksel, varsayılan "pixel"), `96×104 @2` (ince piksel), `192×208 @1` (pürüzsüz/antialias, "flat"/"sticker"). Hepsi 192×208'e tam bölünür.
- `palette`: adlandırılmış tonlar; `"auto"` gölge/ışık otomatik türetilir (HSL'de L ±, renk tonu hafif kaydırma). Piksel stilinde **yalnız palet renkleri** kullanılır (deterministik kimlik denetimine izin verir).
- `parts`: ilkel şekiller (`ellipse`, `roundrect`, `triangle`, `polygon`, `curve`/kapsül, `band`, `pixelmap`), `parent` ile hiyerarşi, `pivot`/`at` ebeveyne göre, `depth` (−1 arka … +1 ön; gövde dönüşünde kullanılır), `follow` (ikincil hareket gecikmesi 0–1), `mirrorOf`.
- `pixelmap`: ASCII ızgara + palet harfleri (ör. özel bir yüz/kask/logo-dışı desen). Claude'un küçük ayrıntıyı elle "boyaması" için en güvenilir yol.
- `style` ön ayarları: `pixel` (seçici kontur, 1 mantıksal px), `pixel-fine`, `flat` (GDI+ antialias, kalın kontur), `sticker` (dış beyaz kenar + kontur), `soft` (yumuşak gölge geçişi). Ön ayar; kontur kalınlığı, gölge bandı sayısı, ışık yönü, antialias'ı belirler.
- `archetype` şablonları (scripts/templates/): `blob` (Pitir tipi), `biped` (kafa+gövde+kol+bacak), `quadruped`, `object` (kupa, robot, ekran yüzlü; gözler yüzeyde basılı), `floaty` (hayalet/bulut, ayaksız, süzülen). Her şablon: parça listesi + varsayılan oranlar + **bakış mekaniği** + satır ayarları.

### 3.2 (b) Serbest kod kaçış yolu

Basitten serbeste üç kademe:
1. **`pixelmap` parçası** (kod yok, JSON içinde ASCII).
2. **Özel parça:** `"shape": "custom", "class": "Visor"` → `custom/Visor.cs` içinde `IPart` arayüzü:
   `void Draw(Canvas c, PartPose p)` — `Canvas` mantıksal ızgarada `Px`, `Ell`, `Poly`, `Line`, `Fill`, renk adıyla `Pal("fur.shade")` verir; `PartPose` konum, ölçek, yaw/pitch, ifade alır. Parça hâlâ rig'in içindedir; hareket, derinlik sırası, kontur ve QA otomatik.
3. **Tam özel karakter:** `"renderer": "custom/MyPet.cs"`, `ICharacter.Render(Canvas c, Pose p)`. Claude her şeyi çizer, ama Pose'u boru hattı verir (satır kuralları, bakış yönleri, efekt kuralları aynı kalır) ve atlas/QA/paket aynı. Pitir bugün bu kademede.

Kural: kaçış yolu yalnız **çizimi** serbest bırakır; atlas geometrisi, kare sayıları, (0,6) nötr, paketleme ve doğrulayıcıya dokunamaz. Derleme: PowerShell 5.1 `Add-Type` = **C# 5** (string interpolation, `?.`, tuple yok; deneme dosyası buna zaten uyuyor). Skill belgesi bunu açıkça söylemeli.

### 3.3 (c) Animasyon ve bakış türetme kuralları

**Pose modeli (kare başına):**
`rootDy, squash, lean (derece, kalça pivotu), yaw (−1…1), pitch (−1…1), headYaw, headPitch, eyeX, eyeY, blink, expression {eyes, mouth, brows}, limbs {ad: açı}, prop {ad: durum}, effects [bağlı]`.

**Sözde-3B gövde dönüşü (Pitir'in en büyük eksiği):**
- Her parçanın `depth` değeri ve ebeveyn merkezine göre yatay konumu bir silindir/elipsoid üzerinde açıya çevrilir: `φ = asin(x / r)`. Dönüşte `x' = r·sin(φ + yaw·Ymax)`, görünür genişlik `w' = w·cos(φ + yaw·Ymax)` (kısalma), `z' = cos(φ + yaw·Ymax)` → çizim sırası z'ye göre.
- Sonuç: yüz özellikleri (göz, ağız, yanak) dönüş yönüne kayar ve uzak göz daralır/gizlenir; kulak, kol, kuyruk ön/arka sırası yer değiştirir; uzak kol gövdenin arkasına geçer; yanak uzak tarafta kaybolur.
- Siluet de değişir: elips gövdede ışık bandı ve gölge kenarı yaw ile kayar (gölge dönüşün ters tarafında genişler). `blob` için hafif asimetrik şişme (yaw yönünde %3–5) "döndü" hissini verir.
- `Ymax` arketipten: blob 50°, biped 60°, object 0 (yüzey sabit, yalnız eğilme/menteşe).
- Pitch: özellikler `ry·sin(pitch·Pmax)` kadar dikey kayar; yukarıda üst parçalar sıkışır, kafanın altı (çene) görünür, aşağıda tersi.

**Bakış (16 yön), θ: 0 = yukarı, saat yönü:**
- Hedef vektör: `h = sin θ` (sağ +), `v = cos θ` (yukarı +).
- Hiyerarşi (Codex'in "gözler önden, kafa izler, gövde az" kuralı): göz `1.0`, kafa `0.6`, üst gövde `0.25`, ayak/taban `0`. Yani `eyeX = h`, `headYaw = 0.6h`, `bodyYaw = 0.25h`; dikey için `eyeY = v`, `headPitch = 0.5v`, gövde pitch yok, aşağı bakışta küçük bir öne eğilme.
- Gözler: `globe` tipte bütün göz küresi yeniden çizilir (beyaz + iris + parıltı birlikte kayar); `dot` tipte göz noktası kafa yüzeyinde kayar, parıltı sabit ışık yönünde kalır; `screen` tipte (object) yalnız çizili özellik.
- Taban çizgisi ve alt gövde çapası nötrle aynı (deterministik; ölçek yok, kayma yok).
- 180° = aşağı bakış (Codex sözleşmesi), arka görünüş değil.
- Eşit adım: her 22.5° adımda aynı parçalar yaklaşık aynı miktarda hareket eder (sin/cos zaten sağlar; tamsayıya yuvarlamadan önce alt-piksel konum hesapla, sonra yuvarla, aynı adımda iki parçanın birden zıplamasını önlemek için yuvarlamayı tek yerde yap).
- Bütün sprite'ı döndürmek yasak; elle `rotate` yalnız parça düzeyinde.

**Satır kuralları (Motion kütüphanesi, arketipten bağımsız, parametreli):**
- `idle` (6): nefes squash 0→0.8→0, topper/kuyruk 1 kare gecikmeli salınım, 5. karede göz kırpma; durağan olamaz.
- `running-right` (8): **yaw = +0.6 (3/4 görünüş), lean = +8…12° (kalça pivotu, yalnız üst parçalar)**, iki fazlı bob (temas/geçiş), bacaklar zıt fazda, kollar bacaklara ters salınım, saç/kuyruk/topper `follow` gecikmesiyle geriye. Ağız açık, gözler yöne bakar.
- `running-left`: **piksel aynası değil, `yaw = −0.6, lean = −10°` ile yeniden çizim** (ışık yönü ve asimetrik prop doğru kalır). Spec `asymmetric` boşsa ve stil izin veriyorsa ayna seçeneği; kare sırası korunur.
- `waving` (4): başlangıç (kol kalkar, omuz +), tepe sol, tepe sağ, dönüş. Kol 2 segment (omuz + dirsek/bilek), dirsekte salınım; gövde sallanan kola doğru 3–5° eğilir, gözler mutlu. Codex: "clear start, raised gesture, and return".
- `jumping` (5): hazırlık (squash +), kalkış (stretch), tepe, iniş (stretch azalır), oturma (squash). Topper tepe karesinde gecikmeli.
- `failed` (8): şaşkınlık (2 kare) → sönme (squash artar, topper/kulak düşer, gözler sıkılı) → hafif titreme. Ter damlası **gövdeye değerek**.
- `waiting` (6): istekli poz: hafif öne eğilme, yukarı bakış, bir kol/topper hafif kalkık, gözler kırpışır.
- `running` (6): iş: laptop/klavye propu (spec'te varsa) ya da "odaklanmış kıpırtı"; ayak koşusu yok.
- `review` (6): eğilme + kafa yana yatık + kısık gözler, bir el çenede; yeni prop yok.
- `look` (16): yukarıdaki bakış kuralı; (0,6) = nötr ön poz (idle 0 ile aynı).

**Efektler:** varsayılan `effects: "attached"` (ter, gövdeye değen yıldız/duman). Kopuk efekt (Pitir'deki "...", "?", düşünce balonu, kıvılcımlar) varsayılan kapalı; açık soru 4'te.

### 3.4 (d) Çiz-bak-düzelt QA döngüsü

**Deterministik doğrulama (`Qa.cs`, Python'suz):**
1. Atlas: 1536×2288, alfa, used/unused hücreler (yukarıdaki tablo), (0,6) dolu, used ≥ 50 px, unused = 0 px, saydam pikselde RGB = 0, hücre %95'ten fazla opak değil.
2. Kare: kenarda 2 px şeritte piksel (kırpılma), satır medyan alanına göre aykırı kare (< 0.35 / > 2.75).
3. Durağanlık: satırdaki tüm kareler birbirinin aynısıysa hata (idle dahil).
4. Taban çizgisi: idle, look ve (0,6) karelerinde en alt opak satır ±1 mantıksal piksel.
5. Bakış sürekliliği: ardışık çift fark/merkez/alan oranı (Codex eşikleri), gövde içi yatay saydam delik satırı.
6. **Yön semantiği (Codex'in kör 3-işçi testinin yerine):** renderer her kare için `meta.json`'a özellik konumlarını yazar (göz merkezleri, kafa merkezi, gövde merkezi). Denetim: `sign(gözlerin kafa merkezine göre x farkı) == sign(sin θ)` ve dikeyde `cos θ` ile aynı; 000/090/180/270'te büyüklük eşik üstünde (sert kapı), aralarda doğru çeyrek (uyarı). Bu bizim en büyük avantajımız: yönü tahmin etmeye gerek yok, ölçebiliyoruz.
7. Palet (piksel stili): kullanılan her renk palette mi; satırlar arası renk histogramı benzer mi (kimlik kayması yakalama).
8. Bağlı bileşen sayısı: kare başına opak bileşen > 1 ise uyarı (kopuk efekt/parça), `effects: "detached"` açık değilse.
9. Ayna denetimi: running-left, running-right'ın tersi yöne bakıyor mu (göz x işareti).
Çıktı: `qa/review.json` (`ok`, `errors`, `warnings`, hücre ayrıntısı).

**Görsel QA (Claude `Read` ile PNG'lere bakar):**
- `qa/contact-sheet.png` (dama arka plan + açık arka plan iki sürüm, Pitir'in açık arka planlı sayfası iyi).
- `qa/rows/<state>.png`: satır 2× büyütülmüş, kareler yan yana + "onion" (ardışık kare farkı) şeridi.
- `qa/look-directions.png`: nötr + 16 yön, derece etiketli, ayrıca kafa yakın planı (Codex `make_direction_qa_sheet` fikri).
- `qa/preview.html`: atlas + Codex süreleriyle CSS `steps()` animasyonu; Alperen tarayıcıda açar. (System.Drawing ile animasyonlu GIF zahmetli; HTML sıfır bağımlılık.) İsteğe bağlı: uygulamada geçici pet olarak açma.

**Döngü:**
1. Temel kare (idle 0, 4× büyük) çiz → Claude bakar → Alperen'e gösterilir (tek onay noktası: "görünüş tamam mı?").
2. 4 ana yön (000/090/180/270) + running-right 1 kare çiz → bak (Codex'in "önce kardinaller" fikri; gövde dönüşü burada oturur).
3. Tam atlas → deterministik QA → hata varsa düzelt (spec ya da custom kod), en fazla N=3 tur.
4. Görsel QA rubriği (kısa liste, references/qa-rubric.md) maddeleri tek tek "geçti/uyarı/kaldı".
5. Aynı kök hata 2 kez → stratejiyi değiştir (ör. parçayı sadeleştir, arketip değiştir, custom koda geç).
6. İsteğe bağlı: bağımsız bir alt ajana yalnız etiketsiz yön sayfasını verip kardinalleri sınıflandırtmak (Codex'in kör testinin tek-işçilik hafif hâli). Deterministik yön denetimi varken varsayılan kapalı olabilir.

### 3.5 (e) Çıktı klasörü ve paketleme

Çalışma klasörü (silinebilir):
```
~/.claude-pet/hatch/<id>-<yyyyMMdd-HHmm>/
  spec.json
  custom/*.cs            (varsa)
  build/frames/<state>/NN.png, meta.json
  final/spritesheet.png
  qa/review.json, contact-sheet.png, look-directions.png, rows/*.png, preview.html
```
Kurulu pet:
```
~/.claude-pet/pets/<id>/
  pet.json               {"id","displayName","description","spriteVersionNumber":2,"spritesheetPath":"spritesheet.png"}
  spritesheet.png        (Codex PNG'yi de kabul ediyor; System.Drawing WebP yazamaz)
  claude-pet.json        (isteğe bağlı: frameMs, states, idleExtras; ek sheet gerekirse)
  source/spec.json       (+ custom/*.cs) — sonradan "Pitir'e şapka ekle" gibi düzenleme için
```
- Paketleme yalnız `qa/review.json` `ok: true` iken.
- İsteğe bağlı Codex kopyası `~/.codex/pets/<id>/` (sorulur).
- `source/` klasörünü uygulama okumaz (pet.rs yalnız 5 alan ve bilinen dosyaları okuyor), zararsız.
- Aynı id varsa üzerine yazmadan önce sor ya da `<id>-2`.

### 3.6 (f) Skill dosya yerleşimi

Plugin içinde (dosyalar İngilizce, `.ps1` de ASCII tutulmalı; PS 5.1 BOM'suz dosyayı ANSI okur):
```
skills/hatch/                     (ad önerisi; "hatch-pet" Codex'le çakışır, bkz. açık soru 1)
  SKILL.md                        (~150-200 satır: ne zaman, 4 adımlı kontrol listesi, komutlar, döngü, sert kurallar)
  references/
    atlas-contract.md             (satırlar, kare sayıları, süreler, (0,6), 180=aşağı, pet.json)
    spec-format.md                (spec.json alanları, örnekler 3 seviye)
    archetypes.md                 (blob/biped/quadruped/object/floaty + bakış mekaniği)
    motion-rules.md               (satır kuralları, gövde dönüşü, bakış hiyerarşisi, efekt kuralları)
    style-presets.md              (pixel, pixel-fine, flat, sticker, soft; okunurluk ipuçları)
    custom-code.md                (IPart/ICharacter, Canvas API, C# 5 kısıtları, örnek)
    qa-rubric.md                  (deterministik + görsel liste, onarım/yakınsama kuralı)
  scripts/
    hatch.ps1                     (tek giriş: build | preview | validate | package; Add-Type, derlenmiş DLL'i önbelleğe al)
    PetKit/Canvas.cs              (ızgara, palet, ilkel şekiller, seçici kontur, ölçekli blit, PNG kaydet)
    PetKit/Rig.cs                 (parça ağacı, pivot, derinlik, sözde-3B yaw/pitch, z-sırası)
    PetKit/Motion.cs              (satır üreticileri, easing, follow gecikmesi, look 16)
    PetKit/Spec.cs                (JSON okuma: PS 5.1'de ConvertFrom-Json → C#'a aktarım ya da JavaScriptSerializer)
    PetKit/Qa.cs                  (doğrulayıcı, sayfalar, preview.html, meta.json)
    templates/*.json              (arketip şablonları)
  examples/pitir/spec.json        (deneme petin spec karşılığı, regresyon örneği)
```
Not: JSON okumak için .NET Framework'te `System.Web.Script.Serialization.JavaScriptSerializer` (System.Web.Extensions) var; ek paket gerekmez. Alternatif: PowerShell'de `ConvertFrom-Json` ile okuyup C# nesnesine elle aktarmak.

### 3.7 Deneme kodundan yeniden kullanılabilir parçalar (`Pitir.cs`)

Doğrudan alınacak:
- `Px`/`Get` + `mirror` bayrağı, sınır kırpma (satır 51–60).
- `In`/`Ell`/`OEll` döndürülebilir elips (61–75), `Line` (76–82), `Rot` (85–88).
- Işık yönünü aynada koruyan gölgelendirme (`nx` işaret çevirme, 121) ve tek piksel parıltı (129).
- `Blit` tamsayı ölçekli kopyalama (256–264), `Save` LockBits + `Marshal.Copy` ile hızlı PNG (266–278), arka planlı kopya sayfası.
- `FrameSpec` fikri (kare başına poz verisi) ve `Rows()` içindeki sin/cos tabanlı döngü üretimi (running bob, ayak fazları).
- Ağız/göz ifade enum'ları (`Eyes`, `Mouth`) → `expression` kütüphanesinin çekirdeği.

Değiştirilecek:
- Kontur: `OEll` her parçanın dışına ayrı kontur çiziyor; parçalar birleşince iç konturlar kalıyor ya da kayboluyor. Öneri: önce dolgu + parça kimliği maskesi, sonra **kontur geçişi** (boş komşusu olan dolu piksel → kontur; farklı parça sınırında isteğe bağlı koyu ton = "seçici kontur").
- Statik global `buf` → `Canvas` nesnesi.
- Sabit kodlu renkler/oranlar → palet + spec.
- Her elips için tüm ızgarayı tarama: 48×52'de sorun değil; 192×208 @1'de bbox ile sınırla.

Denemede görülen eksikler (kontak sayfasına bakıldı):
- Bakış satırları yalnız yüzü kaydırıyor (Fx ±4, Fy ±2), gövde/siluet dönmüyor, 180 ve 000 civarı nötre çok yakın.
- Koşuda eğilme ve 3/4 görünüş yok; yalnız ayak ve topper.
- El sallama: 25°/65° iki açı, başlangıç/dönüş yok, tek segment kol.
- (0,6) nötr hücre boş → Codex `validate_atlas.py --require-v2` hata verir.
- `pet.json`'da `spriteVersionNumber: 2` yok → Codex 2288 yüksekliği reddeder.
- Kopuk efektler: waiting'de "...", review'da düşünce balonu, jumping'de kıvılcım → Codex kuralına aykırı.
- running-left piksel aynası (Pitir simetrik olduğu için sorun yok, genel çözüm değil).

---

## 4. Açık sorular (Alperen'in kararı)

1. **Skill adı ve yeri:** `hatch` mi, `/claude-pet hatch` alt komutu mu, ayrı skill mi? ("hatch-pet" Codex'inkiyle aynı ad, kafa karıştırır.)
2. **Teknoloji:** PowerShell 5.1 + C# 5 + System.Drawing (sıfır kurulum, yalnız Windows; ilk sürüm yalnız Windows kararıyla uyumlu) mı, yoksa çizim motorunu Rust exe'ye `claude-pet hatch` alt komutu olarak mı koyalım (ileride çok platform, ama Claude'un serbest kod kaçış yolu derleme gerektirir, zorlaşır)? Önerim: ilk sürüm PowerShell/C#.
3. **Lisans/atıf:** sıfırdan yazıp README'ye nezaket atfı mı (önerim), yoksa doğrulama betiklerini port edip Apache-2.0 bildirimi taşımak mı?
4. **Kopuk efektler:** Codex kuralına uyalım mı (varsayılan yalnız bağlı efekt), yoksa bizim uygulamada sorun olmadığı için "...", düşünce balonu gibi efektlere izin mi? (Pitir'in waiting/review satırları bunlara dayanıyor.) Önerim: varsayılan kapalı, spec'te `effects: "detached"` ile açılır.
5. **Codex uyumu ne kadar sıkı:** ürettiğimiz pet Codex'te de çalışmalı mı (`spriteVersionNumber`, (0,6) nötr, kopuk efekt yasağı, `~/.codex/pets` kopyası)? Uyumun maliyeti düşük, önerim evet (efekt hariç, bkz. 4).
6. **Varsayılan stil ve ızgara:** `pixel 48×52 @4` mü (deneme), yoksa daha ayrıntılı `96×104 @2` mi? Pürüzsüz (`flat`) stil ilk sürüme girsin mi?
7. **Onay noktaları:** kaç kez Alperen'e sorulsun? Önerim: (1) temel görünüş, (2) 4 ana yön + koşu karesi, (3) son önizleme ve kurulum.
8. **Fazladan satırlar:** skill `claude-pet.json` ile bizim ek durumlarımız için (`thinking`, `writing-code` vb.) ek sheet de üretsin mi, yoksa yalnız Codex 11 satırı mı?
9. **KARARLAR 3 güncellemesi:** `spriteVersionNumber` artık resmi skill'de zorunlu; karar metnindeki "yalnız toplulukta geçiyor" ifadesi düzeltilsin mi? Ayrıca (0,6) nötr hücrenin uygulamada idle döngüsünde oynatılıp oynatılmadığı kontrol edilmeli.
10. **Kör yön incelemesi:** deterministik yön ölçümü yeterli mi, yoksa ek olarak bir alt ajana etiketsiz kardinal testi de yaptıralım mı (maliyet: bir alt ajan çağrısı)?

## Kaynaklar

- https://github.com/openai/skills (README "License": lisans her skill klasöründe)
- https://github.com/openai/skills/tree/main/skills/.curated/hatch-pet (LICENSE.txt: Apache-2.0)
- https://github.com/openai/skills/blob/main/skills/.curated/hatch-pet/SKILL.md
- https://github.com/openai/codex (Apache-2.0)
- https://www.apache.org/licenses/LICENSE-2.0 (Madde 4 dağıtım koşulları, Madde 6 marka)
- https://github.com/openai/codex/issues/20863 (özel pet animasyon dizileri isteği, bağlam)
