# Teknik araştırma: Claude Code plugin + skill yapısı (claude-pet için)

Tarih: 2026-10-02. Kaynaklar 2 Ekim 2026'da indirilen resmi belgeler (`code.claude.com/docs/en/*.md`,
`platform.claude.com`, `agentskills.io`) ve `github.com/anthropics/*` repoları. Ham kopyalar:
`scratchpad/arastirma/raw/`.

> Not: Belge yapısı değişmiş. Eski `plugins-reference` / `plugin-marketplaces` adresleri artık
> `plugins/manifest-reference`, `plugins/marketplace-reference`, `plugins/create-marketplace`,
> `plugins/host-marketplace`, `plugins/publish`, `plugins/loading`, `plugins/cli-reference`,
> `plugins/components` sayfalarına bölünmüş. Tam dizin: https://code.claude.com/docs/llms.txt

---

## 0. Önce bilinmesi gereken en kritik bulgular

1. **`claude-pet` adı plugin adı olarak "reserved" (ayrılmış).** `claude plugin validate`,
   `claude-`, `anthropic-`, `anthropics-` ya da `cc-plugin-` ile başlayan plugin adlarını **hata**
   olarak işaretliyor: `Plugin name "<name>" is reserved: it passes as one of Anthropic's own`.
   `claude plugin init` ve `claude plugin tag` böyle bir adı reddediyor. Claude Code yine de kurup
   yüklüyor, ama `validate --strict` CI'de ve Anthropic dizinine başvuruda sorun çıkar.
   Kaynak: https://code.claude.com/docs/en/plugins/manifest-reference (bölüm `name`)
   > "Starts with `claude-`, `anthropic-`, `anthropics-`, or `cc-plugin-` | Error"
   > "Has `claude`, `anthropic`, or `anthropics` as a whole word anywhere else, such as `mcp-for-claude` | Warning"

   Ayrıca Agent Skills standardı ve Anthropic'in skill yazım rehberi, **skill `name` alanında
   "claude" ve "anthropic" kelimelerini yasaklıyor** (claude.ai yüklemesi ve Skills API için).
   Kaynak: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices
   > "Cannot contain reserved words: "anthropic", "claude""

   **Sonuç:** Plugin adı (`name`) ve skill adı `claude` içermemeli. Örnek: plugin `desk-pet`
   (ya da `pixel-pet`, `code-pet`), görünen ad `displayName` ile ayrıca verilir. Ad kalıcıdır:
   sonradan değiştirmek kurulu kullanıcıları koparır (bkz. 1.6) ve `${CLAUDE_PLUGIN_DATA}`
   klasörünün yolunu da değiştirir. **Bu karar ilk yayından önce verilmeli.**

2. **`bin/` klasörü özeldir.** Plugin kökündeki `bin/` içindekiler Bash aracının `PATH`'ine eklenir;
   claude.ai ve Cowork, kökünde `bin/` olan plugin'i **kurmaz**. Planlanan yer
   `${CLAUDE_PLUGIN_DATA}/bin` olduğu için sorun yok, ama plugin kökünde `bin/` açmayın; betikleri
   `scripts/` gibi bir klasöre koyun.
   Kaynak: https://code.claude.com/docs/en/plugins/components (bölüm Executables),
   https://code.claude.com/docs/en/plugins/host-marketplace (bölüm Distribute through organization settings)

3. **Exe dağıtımı için resmi bir "ikili dosya indirme" mekanizması yok.** Belgede anlatılan yöntemler:
   ikiliyi plugin'e koymak (her sürümde cache'e kopyalanır), `archive` kaynağı (sha256 ile
   sabitlenen zip, en fazla 256 MiB) ya da kurulumu bir hook/skill ile
   `${CLAUDE_PLUGIN_DATA}` içine yapmak (belgede `node_modules` için örneği var). Ayrıntı: bölüm 5.

4. **Windows'ta `${CLAUDE_PLUGIN_ROOT}` içinden çalışan bir exe, güncellemeyi bozabilir.** Windows'ta
   başka bir program kurulu kopyayı tutuyorsa güncelleme "could not be replaced" ile başarısız oluyor.
   Exe'yi `${CLAUDE_PLUGIN_DATA}` içinde tutma kararını destekliyor (ama orada da exe çalışırken
   üzerine yazılamaz; değiştirmeden önce pet kapatılmalı).
   Kaynak: https://code.claude.com/docs/en/plugins/troubleshooting
   > "On Windows, when another program holds the installed copy itself, the message instead says that copy `could not be replaced`..."

5. **Hook'larda `.cmd` exec form ile çalıştırılamaz.** Exec form (`args` verilen) Windows'ta gerçek
   bir `.exe` ister; `.cmd`/`.bat` için ya shell form ya da `cmd.exe` exe'sini `args` ile çağırmak
   gerekir. Bkz. bölüm 7.

---

## 1. `.claude-plugin/plugin.json`

Kaynak (aksi yazmadıkça): https://code.claude.com/docs/en/plugins/manifest-reference

### 1.1 Zorunlu / önerilen alanlar

- Manifest **isteğe bağlı**. Yoksa bileşenler standart yerleşimden bulunur, ad marketplace
  girdisinden ya da klasör adından gelir.
- Manifest varsa **tek zorunlu alan `name`**.
  > "`name` is the only required key."
- `validate` şu eksiklerde **uyarı** verir: kebab-case olmayan ad, eksik `version`, `description`
  veya `author`. `--strict` bunları hataya çevirir.
- Yayın kontrol listesi ayrıca şunları öneriyor: `description`, `author`, `homepage`, `repository`,
  plugin kökünde `README.md`. `homepage` URL olarak çözülemezse plugin **yüklenmez**.
  Kaynak: https://code.claude.com/docs/en/plugins/publish (Prepare your plugin for release)

Tüm üst düzey alanlar (özet):

| Alan | Not |
| :- | :- |
| `$schema` | Editör için; yüklemede yok sayılır |
| `name` | Zorunlu, kebab-case; boşluk, `@`, `:`, yol ayırıcı yok. Tüm bileşenler `name:` ad alanına girer |
| `displayName` | UI'da gösterilen ad; boşluk ve büyük harf olabilir, ad alanında kullanılmaz |
| `version` | Semver denetlenmez. Verilirse kullanıcıyı o sürümde tutar (1.4) |
| `description`, `author` (`name` zorunlu; `email`, `url`), `homepage`, `repository`, `license` (SPDX), `keywords` | Meta veri |
| `metadata` | Serbest nesne, Claude Code okumaz |
| `icon`, `documentationUrl`, `supportUrl`, `privacyPolicyUrl`, `termsOfServiceUrl` | Yalnız Anthropic dizini listelemesi için |
| `defaultEnabled` | Varsayılan `true` |
| `dependencies` | Başka plugin'ler |
| `settings` | Yalnız `agent` ve `subagentStatusLine` etkili |
| `userConfig` | Kullanıcıya sorulan değerler (1.5) |
| `skills`, `commands`, `agents`, `hooks`, `mcpServers`, `lspServers`, `outputStyles`, `workflows`, `experimental.{themes,monitors,evals}` | Bileşen yolları |

- Tanınmayan üst düzey alan **sessizce atılır** (validate uyarır). `userConfig` seçenekleri,
  `channels`, `lspServers`, `monitors` içindeki bilinmeyen anahtar ise **hata**dır, plugin yüklenmez.

### 1.2 Yol kuralları

- Tüm bileşen yolları plugin köküne göredir ve **`./` ile başlamalı** (`commands/foo.md` geçersiz).
- Yol plugin kökü dışına çıkamaz (`..` hata), var olmalı.
- `skills` varsayılan `skills/` taramasına **ekler**; `commands`, `agents`, `outputStyles` vb.
  varsayılanın **yerine geçer**; `hooks`, `mcpServers`, `lspServers` **birleşir**.
- macOS/Linux'ta ters bölü içeren yol reddedilir; her zaman `/` kullanın.
  Kaynak: https://code.claude.com/docs/en/plugins/loading (Paths that escape the plugin directory)

### 1.3 Standart klasör yerleşimi

| Bileşen | Varsayılan yer |
| :- | :- |
| Manifest | `.claude-plugin/plugin.json` (yalnız bu dosya `.claude-plugin/` içinde durur) |
| Skill'ler | `skills/<ad>/SKILL.md` |
| Komutlar (eski) | `commands/` ("Prefer `skills/` for new plugins") |
| Ajanlar | `agents/` |
| Hook'lar | `hooks/hooks.json` (üstte `"hooks"` sarmalayıcısı şart) |
| MCP / LSP | `.mcp.json` / `.lsp.json` |
| Çalıştırılabilirler | `bin/` (Bash `PATH`'ine girer; claude.ai/Cowork reddeder) |
| Ayarlar | `settings.json` |

- Plugin kökündeki `CLAUDE.md` **yüklenmez**, validate uyarır; talimatlar skill olarak yazılmalı.
- `skills/` klasörünü `.claude-plugin/` içine koymak yaygın hata: taranmaz.
  Kaynak: https://code.claude.com/docs/en/plugins/troubleshooting (Plugin loads but its skills are missing)

claude-pet için önerilen iskelet (adlar örnek):

```text
desk-pet/                      # repo kökü = marketplace kökü = plugin kökü
├── .claude-plugin/
│   ├── plugin.json
│   └── marketplace.json       # tek girdi, "source": "./"
├── skills/
│   ├── pet/                   # /desk-pet:pet  (aç/kapa, list, use, install)
│   │   ├── SKILL.md
│   │   └── scripts/...
│   └── hatch/                 # /desk-pet:hatch (sprite çizer)
│       ├── SKILL.md
│       ├── references/...
│       └── scripts/hatch.ps1
├── hooks/
│   ├── hooks.json
│   └── pet-hook.cmd           # ASCII
├── scripts/                   # paylaşılan betikler (bin/ DEĞİL)
├── README.md
└── LICENSE
```

### 1.4 Sürümleme

Kaynak: https://code.claude.com/docs/en/plugins/loading (Versions and updates),
https://code.claude.com/docs/en/plugins/host-marketplace (Release a new version)

- Claude Code her plugin için bir sürüm hesaplar; güncelleme yalnız bu sürüm **değişince** iner.
  Sıra: `plugin.json` `version` → marketplace girdisi `version` → kaynak türü (GitHub/git için
  commit SHA'nın 12 karakteri).
- `"version": "1.0.0"` verip sürümü artırmadan commit atarsanız kullanıcılar eski kopyada kalır
  (`is already at the latest version`).
- İki seçenek: her yayında `version` artırın **ya da** `version`'ı hem `plugin.json`'dan hem
  girdiden çıkarın (commit SHA izlenir). İkisine birden yazmayın (validate uyarır, `plugin.json` kazanır).
- Üçüncü taraf marketplace'lerde **otomatik güncelleme varsayılan olarak kapalı**; kullanıcı
  `/plugin` > Marketplaces > Enable auto-update ile açar ya da `claude plugin update` çalıştırır.
- Sürüm, cache klasörünün adıdır: `~/.claude/plugins/cache/<marketplace>/<plugin>/<version>/`.
  Eski sürüm klasörü 14 gün sonra temizlenir.
- `claude plugin tag` `{name}--v{version}` etiketi atar (yalnız başka plugin'ler size sürüm aralığıyla
  bağımlıysa gerekli). Kaynak: https://code.claude.com/docs/en/plugins/publish

### 1.5 `userConfig` (isteğe bağlı, işe yarayabilir)

- Tür: `string`, `number`, `boolean`, `directory`, `file`; zorunlu alanlar `type`, `title`,
  `description`. `sensitive: true` güvenli depoya yazar.
- Hook süreçlerine `CLAUDE_PLUGIN_OPTION_<KEY>` olarak gelir; skill metninde `${user_config.KEY}`
  (hassas olmayanlar) yerine konur. Shell-form hook komutunda `${user_config.*}` **hata** verir.
- `claude plugin install` (kabuk) soru sormaz; `/plugin` arayüzü sorar.
  Kaynak: https://code.claude.com/docs/en/plugins/components (Ask the user for configuration values)

### 1.6 Ad değişikliği

- "Never change a published plugin's `name`." Gerekirse marketplace `renames` haritası kullanılır.
  Kaynak: https://code.claude.com/docs/en/plugins/publish (Rename or remove a plugin)

### 1.7 `claude plugin validate` neyi denetler

Kaynaklar: manifest-reference (Validate the manifest), https://code.claude.com/docs/en/plugins/cli-reference (plugin validate),
https://code.claude.com/docs/en/plugins/create-marketplace (Problems that validation reports)

- Hedef klasörde önce `.claude-plugin/marketplace.json`, yoksa `plugin.json`, yoksa
  `skills/`/`agents/`/`commands/` dosyaları doğrulanır.
- **Hata:** tür uyuşmazlığı; eksik ya da kök dışına çıkan yol (`..`); strict nesnelerde bilinmeyen
  anahtar; ayrılmış plugin adı; geçersiz marketplace/plugin adı; MCP girdisi sorunları.
- **Uyarı:** bilinmeyen üst düzey alan, kebab-case olmayan ad, eksik `version`/`description`/`author`,
  plugin kökünde `CLAUDE.md`, shell-form hook'ta tırnaksız `${CLAUDE_PLUGIN_ROOT}`, adında
  `claude` kelimesi geçmesi, hem girdide hem `plugin.json`'da `version`.
- **Okumadıkları:** plugin kökündeki tek `SKILL.md`; marketplace modunda plugin'lerin skill/hook
  dosyaları (her plugin klasörünü ayrıca doğrulayın). Skill frontmatter'ı bozuk mu diye
  `claude plugin validate ./skills` çalışır (v2.1.233+).
- Çıkış kodları: 0 geçti, 1 başarısız (`--strict` uyarıları da sayar), 2 doğrulayıcı hatası; `--json` var.
- Doğrulama yetmez: yerel marketplace ekleyip kurarak test edin (`Source path does not exist`
  gibi hatalar yalnız kurulumda çıkar).

---

## 2. Marketplace (`marketplace.json`) — kendi GitHub repomuz

Kaynaklar: https://code.claude.com/docs/en/plugins/create-marketplace,
https://code.claude.com/docs/en/plugins/marketplace-reference, https://code.claude.com/docs/en/plugins/publish

### 2.1 Gereken

- Dosya yeri: `.claude-plugin/marketplace.json` (başka yerde olursa `marketplace add` bulamaz).
- **Zorunlu:** `name`, `owner` (`name` zorunlu; `email`, `url` isteğe bağlı), `plugins` dizisi.
  `description` yoksa validate uyarır.
- Her girdide **zorunlu:** `name`, `source`. İsteğe bağlı: `description`, `version`, `category`,
  `tags`, `strict`, `displayName`, `defaultEnabled`... (plugin.json alanlarının çoğu).
- **Tek repoda plugin + marketplace** (bizim durum): `marketplace.json`'ı `plugin.json`'ın yanına
  koyun, tek girdi `"source": "./"`:

```json
{
  "name": "your-marketplace",
  "owner": { "name": "Your Name" },
  "plugins": [
    { "name": "deploy-helper", "source": "./" }
  ]
}
```
  (Kaynak: publish, "Add the marketplace file to your repository")

- Girdi `name` ile `plugin.json` `name` **aynı olmalı**; farklıysa kurulum `Plugin "<x>" not found
  in marketplace` hatası verebilir.
- Göreli yollar marketplace köküne (`.claude-plugin/`'in üstü) göredir, `..` yasak.
- Git LFS kullanmayın: klon LFS içeriğini indirmez, dosyalar pointer olarak gelir (exe'yi LFS'e
  koymak işe yaramaz). Kaynak: host-marketplace (Keep plugin files out of Git LFS)

### 2.2 Ad kuralları

- Marketplace `name`: harf, rakam, `.`, `_`, `-`; harf/rakamla başlar. Kullanıcı kurulumda
  `plugin@marketplace` yazar.
- Ayrılmış adlar: `claude-code-marketplace`, `claude-plugins-official`, `anthropic-plugins` vb.;
  resmi adı taklit eden adlar (`official-claude-plugins`, `claude-plugins-v2`), ASCII dışı karakter,
  `npm`/`github` vb., `claudeai-` ile başlayanlar. `claude-pet` listede yok, ama içinde `claude`
  geçen bir ad taklit denetimine takılma riski taşır; `alperen-pets` gibi nötr bir ad daha güvenli.
  Kaynak: marketplace-reference (Reserved names)

### 2.3 Kullanıcı kurulumu

```text
claude plugin marketplace add <github-user>/<repo>
claude plugin install <plugin>@<marketplace>
# ya da oturum içinde (v2.1.275+):
/plugin install <plugin> --marketplace <github-user>/<repo>
```
- Güncelleme: `claude plugin update <plugin>@<marketplace>` ya da otomatik güncellemeyi açmak.
- Herkese açık katalog için Anthropic dizinine başvuru ayrı süreç (claude.ai/directory/manage,
  ücretli plan). `claude-plugins-official` başvuru almıyor.

### 2.4 Diğer kaynak türleri (bilgi için)

`github` (`repo`, `ref`, `sha`), `url`, `git-subdir`, `npm`, `archive` (HTTPS zip + `sha256`,
v2.1.224+), `command` (kullanıcı makinesinde komut; Windows'ta `link` modu yasak).

---

## 3. Skill'ler

Kaynak (aksi yazmadıkça): https://code.claude.com/docs/en/skills

### 3.1 Frontmatter alanları (Claude Code)

- Hepsi isteğe bağlı; yalnız `description` önerilir. Bilinmeyen alan **sessizce yok sayılır**.
- Frontmatter yalnız dosyanın ilk satırı `---` ise okunur. YAML bozuksa skill alanlarsız yüklenir
  (`/ad` çalışır ama Claude otomatik seçemez).

| Alan | Özet |
| :- | :- |
| `name` | Komut adı; varsayılan klasör adı. Plugin'de son segmenti değiştirir, önek kalır |
| `description` | Ne yapar + ne zaman kullanılır. `description` + `when_to_use` listede **1.536 karakterde kesilir**; ana kullanım başa |
| `when_to_use` | Tetik ifadeleri; açıklamaya eklenir, sınıra dahil |
| `argument-hint` | Otomatik tamamlamada ipucu, ör. `[on\|off\|list\|use <pet>\|install]` |
| `arguments` | Adlı konumsal argümanlar (`$name`) |
| `disable-model-invocation` | `true`: yalnız kullanıcı `/ad` ile çağırır; açıklama bağlama yüklenmez |
| `user-invocable` | `false`: `/` menüsünden gizlenir, yalnız Claude çağırır |
| `allowed-tools` | Skill'in çağrıldığı turda izin sormadan kullanılabilecek araçlar (araç kısıtlamaz) |
| `disallowed-tools` | Skill etkinken araç havuzundan çıkarılır |
| `model`, `effort` | Tur için model/çaba |
| `context: fork`, `agent`, `background` | Alt ajanda çalıştırma |
| `hooks` | Skill çağrılınca kaydolan hook'lar (`once` destekli) |
| `paths` | Glob ile otomatik etkinleşmeyi sınırlar |
| `shell` | `` !`komut` `` blokları için `bash` (varsayılan) ya da `powershell` |
| `metadata`, `license`, `compatibility` (≤500 karakter) | Agent Skills spec alanları, Claude Code işlem yapmaz |

- **Taşınabilirlik:** claude.ai yüklemesi / Skills API yalnız `name`, `description`, `license`,
  `compatibility`, `metadata`, `allowed-tools` kabul eder; başka alan sert hata verir. Plugin
  içindeki skill'ler Claude Code'da her alanı kullanabilir.
- **Agent Skills spec sınırları** (https://agentskills.io/specification): `name` 1–64 karakter,
  küçük harf/rakam/tire, tireyle başlayıp bitemez, `--` olamaz, **üst klasör adıyla aynı olmalı**;
  `description` 1–1024 karakter. Anthropic rehberi ek olarak XML etiketi ve "claude"/"anthropic"
  kelimesini yasaklar (best-practices). Claude Code'da 1.536 karakter sınırı var ama 1024'ün
  altında kalmak her yerde geçerli olur.

### 3.2 Komut adı ve plugin ad alanı

- Plugin skill'i: `/<plugin>:<klasör>`; frontmatter `name` son segmenti değiştirir.
  > "`my-plugin/skills/review/SKILL.md` → `/my-plugin:review`, or `/my-plugin:fancy` with `name: fancy`"
- **Önek olmadan da çağrılır:** "The bare `/fancy` also invokes the skill unless another command
  already uses that name." Yani plugin `desk-pet` + skill `pet` → hem `/desk-pet:pet` hem `/pet`
  (çakışma yoksa).
- `name` alanına öneki zaten yazarsanız v2.1.246+ tekrar eklemez (v2.1.216–245 arası çiftliyordu;
  yazmamak en güvenlisi).
- Plugin kökünde tek `SKILL.md` de olur ama validate onu denetlemez; `skills/<ad>/` tercih edin.
- Kaynak ayrıca: https://code.claude.com/docs/en/plugins/components (Skills)

### 3.3 Argümanlar

- `$ARGUMENTS` (tümü), `$ARGUMENTS[N]` / `$N` (0 tabanlı, kabuk tarzı tırnak), `$name`.
- Hiç yer tutucu yoksa sona `ARGUMENTS: <girdi>` eklenir. `/pet use johnny` → `$0=use`, `$1=johnny`.
- Gerçek `$1.00` yazmak için `\$1.00`.

### 3.4 Değişkenler: skill metninde ve betiklerde

| Değişken | Nerede |
| :- | :- |
| `${CLAUDE_SKILL_DIR}` | SKILL.md'nin klasörü (plugin'de skill alt klasörü, plugin kökü değil) |
| `${CLAUDE_PLUGIN_ROOT}` | Plugin'in kurulu sürüm klasörü (sürümle değişir; buraya durum yazmayın) |
| `${CLAUDE_PLUGIN_DATA}` | `~/.claude/plugins/data/<id>/`, güncellemelerde korunur |
| `${CLAUDE_PROJECT_DIR}` | Proje kökü (v2.1.196+) |
| `${CLAUDE_SESSION_ID}`, `${CLAUDE_EFFORT}` | Oturum kimliği, çaba düzeyi |

- Bu değişkenler **skill metnine ve `allowed-tools` içindeki Bash kurallarına** metin olarak yerine
  konur. Aynı değişkeni ikisinde kullanmak, betiğin izin sorulmadan çalışmasını sağlar:
  ```yaml
  allowed-tools: Bash(${CLAUDE_SKILL_DIR}/scripts/render.sh *)
  ---
  Run `${CLAUDE_SKILL_DIR}/scripts/render.sh <csv-file>` to render the chart.
  ```
- **Önemli:** Değişkenler Bash aracının ortamında **yoktur**:
  > "The variables aren't present in the environment of commands Claude runs through the Bash tool...
  > write the `${...}` reference in the Markdown body instead"
  (manifest-reference, Where each variable resolves). Yani betik kendi içinde
  `$env:CLAUDE_PLUGIN_DATA` okuyamaz; yolu argüman olarak geçirin (ör. `-DataDir "${CLAUDE_PLUGIN_DATA}"`).
  Hook süreçleri ise `CLAUDE_PLUGIN_ROOT`, `CLAUDE_PLUGIN_DATA`, `CLAUDE_PROJECT_DIR` alır.
- Windows'ta yerine konan yollar `/` ile gelir (`C:/Users/...`). PowerShell ve yerel exe'ler bunu kabul eder.
- Belgede `allowed-tools` substitution'ı yalnız "Bash rules" için yazıyor; `PowerShell(...)` kuralında
  değişken yerine konuyor mu **belgelenmemiş**, denenmeli.

### 3.5 Dinamik bağlam (`` !`komut` ``)

- Skill gönderilmeden önce komut çalışır, çıktısı metne girer. Çalışma klasörü oturumun cwd'si;
  yollar için `${CLAUDE_SKILL_DIR}` kullanın. 2 dk zaman aşımı. Sıfır dışı çıkış **tüm skill
  çağrısını iptal eder**. İzin denetimi: auto mode dışında, izinli olmayan komut çağrıyı iptal eder;
  `allowed-tools` ile önceden izin verin.
- `shell: bash` ve Git Bash yoksa çağrı başarısız olur; `shell: powershell` PowerShell aracı
  açıkken çalışır (Git Bash'siz Windows'ta varsayılan açık).
- `/pet list` gibi durum gösteren işlerde (ör. `` !`powershell -NoProfile -File "${CLAUDE_SKILL_DIR}/scripts/list.ps1"` ``)
  kullanışlı; ama hata iptal ettiği için betik her zaman 0 ile çıkmalı.

### 3.6 Yardımcı dosyalar

- Skill klasörü birden fazla dosya içerebilir; SKILL.md'den bağlantı verin ki Claude ne zaman
  okuyacağını bilsin. "Keep `SKILL.md` under 500 lines."
- Spec önerisi: `scripts/` (çalıştırılır), `references/` (gerektiğinde okunur), `assets/`
  (şablon, görsel, veri). Kaynak: https://agentskills.io/specification
- Skill'ler arası paylaşılan dosyalar için `${CLAUDE_PLUGIN_ROOT}/scripts/...`.

### 3.7 Yaşam döngüsü (tasarımı etkiler)

- Çağrılan skill metni tek mesaj olarak bağlama girer ve **turlar boyunca kalır**; dosya tekrar okunmaz.
- `allowed-tools` izni **bir sonraki kullanıcı mesajında sona erer**.
- Sıkıştırmada her skill'in ilk 5.000 token'ı korunur (toplam 25.000); önemli talimatları başa koyun.
- Kaynak: skills (Skill content lifecycle)

### 3.8 Kim çağırır

- `disable-model-invocation: true`: yan etkili iş akışları için (deploy, commit). Pet'i açıp kapamak,
  exe kurmak yan etkili → `/pet` için uygun. Bonus: açıklaması bağlama yüklenmez, token harcamaz.
- `hatch` için Claude'un "bana bir pet çiz" isteğinde kendiliğinden seçmesi istenirse varsayılan
  bırakılır; sadece elle istenirse `disable-model-invocation: true`.

---

## 4. Anthropic'in skill yazım önerileri

Kaynak: https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices
(+ https://code.claude.com/docs/en/skills ve https://agentskills.io/specification)

- **Kısa olun.** "The context window is a public good." "Default assumption: Claude is already very
  smart." Yalnız Claude'un bilmediğini yazın. Claude Code belgesi: "State what to do rather than
  narrating how or why."
- **Uzunluk:** SKILL.md gövdesi 500 satırın altında; spec "< 5000 tokens recommended" diyor.
  Metadata ~100 token.
- **Progressive disclosure:** SKILL.md içindekiler tablosu gibi; ayrıntı ayrı dosyalarda.
  Başvurular **bir seviye derin** (SKILL.md → dosya; dosya → başka dosya değil). 100 satırdan uzun
  başvuru dosyalarına başta içindekiler tablosu.
- **Açıklama:** her zaman **üçüncü şahıs** ("Processes Excel files...", "I can help" değil);
  ne yaptığını **ve** ne zaman kullanılacağını, kullanıcının doğal olarak söyleyeceği anahtar
  kelimelerle yazın. Belirsiz açıklamalardan kaçının ("Helps with documents").
- **Adlandırma:** gerund önerilir (`processing-pdfs`), isim öbeği (`pdf-processing`) kabul;
  `helper`, `utils`, `tools` gibi belirsiz adlar ve "claude"/"anthropic" yasak.
- **Serbestlik derecesi:** kırılgan işlerde (dosya biçimi, piksel atlas) dar ve kesin talimat /
  hazır betik; yaratıcı işlerde geniş serbestlik.
- **Betikler:** "Solve, don't defer" (hataları betik içinde ele alın); "voodoo constant" yok
  (her sabit gerekçeli); talimatta betiğin **çalıştırılacağını mı okunacağını mı** açıkça söyleyin;
  deterministik işler için hazır betik tercih edin; gerekli bağımlılıkları açıkça yazın.
- **Doğrulanabilir ara çıktı / geri besleme döngüsü:** "validator çalıştır → hataları düzelt → tekrarla".
  Hatch için: sprite üret → doğrulayıcı betik (boyut, kare sayısı, saydamlık) → Claude PNG'yi
  görerek kontrol etsin ("Use visual analysis").
- **Yollar:** her zaman `/` ("Avoid Windows-style paths").
- **Zamana bağlı bilgi yok**; tutarlı terim; somut örnekler; şablon deseni.
- **Test:** en az üç değerlendirme, Haiku/Sonnet/Opus ile deneme. Claude Code'da
  `claude plugin eval` ile skill tetiklenme oranı ölçülebilir (`tool_used: Skill` grader).
  Kaynak: skills (Skill not triggering), https://code.claude.com/docs/en/plugin-evals

---

## 5. Plugin'in exe taşıması/indirmesi ve `CLAUDE_PLUGIN_DATA`

### 5.1 Belgede ne var

- **İkili indirme için özel bir alan/mekanizma yok.** Belgelenen yollar:
  1. **Exe'yi plugin dosyalarına koymak.** Kurulumda plugin
     `cache/<marketplace>/<plugin>/<version>/` içine kopyalanır, `${CLAUDE_PLUGIN_ROOT}` oraya işaret
     eder. Her sürümde yeni klasör; eski 14 gün sonra silinir. Git LFS olmaz. Windows'ta çalışan exe
     cache klasörünü kilitlerse güncelleme başarısız olur (bölüm 0.4).
     Kaynak: https://code.claude.com/docs/en/plugins/loading (Find plugins on disk, Cleanup of previous versions)
  2. **`archive` kaynağı:** marketplace girdisi HTTPS zip'e işaret eder (ör. GitHub Release asset'i),
     `sha256` ile sabitlenir. Sınırlar: zip ≤256 MiB, 120 sn, ≤5 yönlendirme; açılmış ≤1 GiB.
     Sürüm = sha256'nın ilk 12 karakteri (ya da `version`). Kullanıcının git'e ihtiyacı olmaz.
     Kaynak: marketplace-reference (archive plugin source), host-marketplace (Stay within the download limits)
  3. **Kurulumu hook/skill ile `${CLAUDE_PLUGIN_DATA}` içine yapmak.** Belgedeki örnek, `SessionStart`
     hook'unun `package.json` farkını kontrol edip `node_modules`'u veri klasörüne kurması:
     > "When the automatic install can't provide a dependency, install it from a hook into the persistent data directory."
     Kaynak: https://code.claude.com/docs/en/plugins/components (Install dependencies into the data directory),
     https://code.claude.com/docs/en/plugins/loading (When the dependency install fails or is skipped)
- Otomatik bağımlılık kurulumu yalnız npm/Bun paketleri içindir ve yaşam döngüsü betiklerini
  çalıştırmaz; exe için geçerli değil.
- `bin/` (plugin kökü) Bash `PATH`'ine girer, kullanıcının kendi `PATH`'inden sonra gelir; claude.ai/Cowork
  böyle plugin'i reddeder. Pet exe'sinin Claude tarafından çıplak adla çağrılmasına gerek yoksa
  `bin/` kullanmayın.

### 5.2 `CLAUDE_PLUGIN_DATA` ömrü

Kaynaklar: manifest-reference (Environment variables), loading (Find plugins on disk),
cli-reference (What an uninstall deletes and keeps)

- Yol: `~/.claude/plugins/data/<id>/`; `<id>` = `plugin@marketplace`, harf/rakam/`_`/`-` dışındaki
  karakterler `-` olur (`my-plugin@my-marketplace` → `my-plugin-my-marketplace`). Kök
  `CLAUDE_CODE_PLUGIN_CACHE_DIR` ile değişebilir.
  → **Plugin ya da marketplace adı değişirse veri klasörü de değişir.**
- **İlk başvuruda oluşturulur** ("created on first reference"), güncellemelerde **korunur**.
- **Silinir:** plugin son kapsamdan kaldırılınca (seçenekler ve gizli değerlerle birlikte), ya da
  marketplace kaldırılınca. İstisna: `claude plugin uninstall --keep-data`.
- `--plugin-dir` ile yüklenen geliştirme kopyası `inline` marketplace adını alır → veri klasörü
  muhtemelen `<plugin>-inline` olur (belgede açıkça yazmıyor, `id` kuralından çıkarım; denenmeli).
- Hook süreçleri ortamda `CLAUDE_PLUGIN_DATA` alır; Bash aracı almaz (skill metnine yazın).

### 5.3 Güncelleme davranışı

- Kopyalanan plugin oturum ortasında güncellenirse hook'lar eski yolu kullanmaya devam eder;
  `/reload-plugins` ile yeni yola geçer. Otomatik güncelleme ilk mesajdan sonra 10 dk'ya kadar
  rastgele gecikmeyle çalışır, `Plugin updated: <name> · Run /reload-plugins to apply` gösterir.
  Kaynak: loading (When auto-update runs)
- Yerel klasörden eklenmiş marketplace'in göreli plugin'i **yerinde** yüklenir (kopyalanmaz):
  düzenlemeler sonraki oturumda/`/reload-plugins` ile etkili, sürüm artırmaya gerek yok.
  Geliştirme için uygun. Kaynak: loading (In-place and copied plugins)

### 5.4 claude-pet için öneri (çıkarım, belge değil)

- Exe `${CLAUDE_PLUGIN_DATA}/bin/` içinde (karar zaten bu yönde). Gerekçe: güncellemede korunur,
  cache kilidi sorunu yok, sürümden bağımsız sabit yol.
- Exe'yi oraya getirmenin iki makul yolu:
  - (a) Exe GitHub Release'de; `/pet install` skill'i (yan etkili, `disable-model-invocation: true`)
    bir PowerShell betiğiyle indirir, sha256 doğrular, `${CLAUDE_PLUGIN_DATA}/bin/`'e koyar.
    Repo küçük kalır, ama skill bir adres açtığı için **CLAUDE.md'deki araç kuralıyla** birlikte
    değerlendirilmeli (bu, pet sohbeti değil kullanıcının kendi oturumu; yine de Alperen'e sorulmalı).
  - (b) Exe repoya/plugin'e konur; `SessionStart` hook'u ya da `/pet install`, `${CLAUDE_PLUGIN_ROOT}`
    içindeki exe'yi sürüm farkı varsa `${CLAUDE_PLUGIN_DATA}/bin/`'e kopyalar (belgedeki `diff`
    deseninin aynısı). Ağ gerekmez, ama repo her sürümde exe ile büyür.
- Her iki yolda da: exe çalışıyorsa değiştirmeden önce kapatılmalı; hook asla bloklamamalı
  (kopyalama/indirme `SessionStart`'ta yapılacaksa hızlı olmalı ya da `async: true`).
  Anthropic'in `security-guidance` plugin'i benzer bir "SessionStart bootstrap" yapıyor: kalıcı
  klasöre kurulum, eşzamanlı kurulumlara karşı sentinel dosya, hata sonrası bekleme süresi
  (https://github.com/anthropics/claude-plugins-official/blob/main/plugins/security-guidance/hooks/ensure_agent_sdk.py).
  Not: o plugin veriyi `CLAUDE_PLUGIN_DATA` yerine `~/.claude/security/` içine yazıyor.

---

## 6. Örnek: Anthropic repolarında dosya üreten / betik çalıştıran skill'ler

### 6.1 `anthropics/skills` → `slack-gif-creator` (görsel üretir, en yakın örnek)

https://github.com/anthropics/skills/tree/main/skills/slack-gif-creator

```text
slack-gif-creator/
├── SKILL.md            (~7,8 KB)
├── LICENSE.txt
├── requirements.txt    (pillow, imageio, imageio-ffmpeg, numpy)
└── core/
    ├── gif_builder.py
    ├── frame_composer.py
    ├── easing.py
    └── validators.py
```
- Frontmatter yalnız `name`, `description`, `license`. Açıklama: ne yaptığı + "Use when users request
  animated GIFs for Slack like "make me a GIF of X doing Y for Slack.""
- Gövde: önce kısıtlar (boyut 128x128, FPS, renk sayısı), sonra "Core Workflow" kod örneği
  (yardımcı kütüphane `GIFBuilder` + PIL ile kare kare çizim), çizim ilkeleri ("Use thicker lines",
  "Don't use: Emoji fonts"), yardımcıların API özeti, **doğrulayıcı** (`validators.py`).
- Ders: Claude kodu kendisi yazar ama hazır yardımcı kütüphaneyi çağırır; kalite kuralları ve
  doğrulama skill'de. Hatch skill'i için birebir uyarlanabilir: `scripts/` altında System.Drawing
  yardımcıları (atlas oluştur, kare yerleştir, PNG kaydet, doğrula), SKILL.md'de atlas kuralları
  (`docs/SPRITE.md` özeti `references/` altında).

### 6.2 `anthropics/skills` → `algorithmic-art`, `canvas-design`

- `algorithmic-art`: `SKILL.md` + `templates/generator_template.js` + `templates/viewer.html` (şablon deseni).
- `canvas-design`: `SKILL.md` + `canvas-fonts/` (çok sayıda .ttf; büyük varlıkları skill içinde taşıma örneği).

### 6.3 `anthropics/claude-plugins-official` → `session-report` (betik + şablon, plugin içinde)

https://github.com/anthropics/claude-plugins-official/tree/main/plugins/session-report

```text
session-report/
├── LICENSE
└── skills/session-report/
    ├── SKILL.md
    ├── analyze-sessions.mjs
    └── template.html
```
- Dikkat: **`.claude-plugin/plugin.json` yok** (manifest isteğe bağlı; ad marketplace girdisinden).
- SKILL.md numaralı adımlar: betiği çalıştır → JSON'u oku → şablonu kopyala → Edit ile doldur → yolu bildir.
  Betik yolu için `<skill-dir>` diyor (daha yeni yol: `${CLAUDE_SKILL_DIR}`).

### 6.4 `anthropics/claude-plugins-official` → `security-guidance` (hook'lu plugin)

- `hooks/hooks.json` + Python betikleri; komutlar shell form:
  `bash "${CLAUDE_PLUGIN_ROOT}/hooks/sg-python.sh" "${CLAUDE_PLUGIN_ROOT}/hooks/..."`, `SessionStart`
  için `"timeout": 180`, arka plan işler için `asyncRewake`.

### 6.5 `anthropics/skills` marketplace'i

- `.claude-plugin/marketplace.json`, birden çok plugin girdisi, hepsi `"source": "./"`,
  `"strict": false` ve `skills` listesiyle aynı repodaki skill alt kümelerini seçiyor.
  (Plugin'de `plugin.json` olmadığı için girdi manifest görevi görüyor.)

---

## 7. Hook'lar (Windows'a özgü notlar)

Kaynak: https://code.claude.com/docs/en/hooks

- **Exec form** (`args` var): kabuk yok, `command` gerçek bir exe olmalı.
  > "On Windows, exec form requires `command` to resolve to a real executable such as a `.exe`.
  > The `.cmd` and `.bat` shims ... can't be spawned without a shell."
  → `.cmd` sarmalayıcı için seçenekler: `"command": "cmd.exe", "args": ["/d", "/c", "${CLAUDE_PLUGIN_ROOT}/hooks/pet-hook.cmd", "<olay>"]`
  (cmd.exe gerçek exe) ya da shell form. Yolun `/` ile gelmesinin `cmd /c` ile sorunsuz olduğu **denenmeli**.
- **Shell form** (`args` yok): Windows'ta Git Bash, yoksa PowerShell; `"shell": "powershell"` ile seçilir.
  Yol değişkenini çift tırnağa alın; validate tırnaksız kullanıma uyarır.
- Yol biçimi: manifest-reference "substituted paths use forward slashes" diyor; troubleshooting ise
  exec form ve `shell: powershell`'in "native paths" koruduğunu söylüyor. Belgeler arasında küçük
  çelişki var → betik her iki biçimi de kabul etmeli.
- Zaman aşımı varsayılanları: command 600 sn; `UserPromptSubmit` 30 sn; `SessionEnd` toplam 1,5 sn.
  `async: true` bloklamaz (çıktı sonraki turda gelir); `-p` oturumunda kapanışta öldürülür, iş
  oturumu aşacaksa tamamen ayrık süreç başlatılmalı.
- Plugin hook'ları skill kullanılmasını beklemez; oturum yükleyince hep çalışır (`matcher` ile daraltın).
- Hook ortamı: `CLAUDE_PLUGIN_ROOT`, `CLAUDE_PLUGIN_DATA`, `CLAUDE_PROJECT_DIR`, `CLAUDE_PLUGIN_OPTION_*`.

---

## 8. Hatch skill'i için PowerShell notları

Kaynak: https://code.claude.com/docs/en/tools-reference (PowerShell tool), skills

- PowerShell aracı: Git Bash'siz Windows'ta otomatik açık; Git Bash varken claude.ai/Console
  hesaplarında varsayılan açık. Claude Code PowerShell'i süreç kapsamında
  `-ExecutionPolicy Bypass` ile başlatır (Group Policy hariç).
- Kullanıcıda Bash aracı da PowerShell aracı da olabilir. SKILL.md'de betiği tek bir açık komutla
  verin, ör. `powershell.exe -NoProfile -ExecutionPolicy Bypass -File "${CLAUDE_SKILL_DIR}/scripts/hatch.ps1" -Out ...`
  (her iki araçtan da çalışır). `allowed-tools` için hem `Bash(...)` hem `PowerShell(...)` kuralı
  yazılabilir; PowerShell kuralında `${CLAUDE_SKILL_DIR}` yerine konuyor mu belgelenmemiş (3.4).
- System.Drawing Windows PowerShell 5.1'de hazır (.NET Framework). PowerShell 7'de
  (`pwsh`, .NET Core) `System.Drawing.Common` yalnız Windows'ta destekleniyor; Claude Code
  `pwsh.exe`'yi tercih ettiği için betiği iki sürümde de test edin (bu madde genel .NET bilgisi,
  bu araştırmada belge ile doğrulanmadı).

---

## 9. Kısa karar listesi (öneri)

1. Plugin adını `claude` içermeyecek şekilde seç (ör. `desk-pet`), `displayName` ile görünen adı ver.
   Marketplace adı da nötr olsun.
2. Repo kökü = plugin kökü = marketplace kökü; `marketplace.json` tek girdi `"source": "./"`.
3. Skill'ler: `skills/pet/` (`disable-model-invocation: true`, `argument-hint`, `$0/$1` ile alt komut),
   `skills/hatch/` (`scripts/` + `references/`, doğrulayıcı betik + görsel kontrol döngüsü).
4. Kökte `bin/` yok; exe `${CLAUDE_PLUGIN_DATA}/bin/`; getirme yolu (Release indirme mi, repodan
   kopya mı) ayrıca karar.
5. Sürüm: `plugin.json` `version`'ı her yayında artır (ya da hiç yazma); yalnız bir yerde tut.
6. CI: `claude plugin validate --strict .` ve `claude plugin validate --strict ./skills`.
7. Hook: `cmd.exe` + `args` exec form ya da shell form; ikisini de Windows'ta dene.
