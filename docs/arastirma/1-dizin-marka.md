# Araştırma 1: Resmi dizin/marketplace, marka kuralları, plugin'e özel riskler

Tarih: 2026-10-02. Yalnız resmi kaynaklar (code.claude.com, claude.com/docs, support.claude.com, anthropic.com, github.com/anthropics). Resmi olmayan kaynaktan gelen bilgi açıkça "RESMİ DEĞİL" diye işaretlendi. Alıntılar sayfalardan aynen alındı (İngilizce).

---

## 0. Özet (karar verecek kişi için)

1. **"claude-pet" adı engel.** Claude Code'un kendi doğrulayıcısı `claude-` ile başlayan plugin adını **hata** sayar ("reserved: it passes as one of Anthropic's own"); `claude plugin init` / `claude plugin tag` bu adı reddeder. Dizin portalı da adın "kendi özgün ürün adın etrafında" kurulmasını ve resmi görünmemesini şart koşar. Ayrıca GitHub'da en az 8 tane "claude-pet" adlı proje var (RESMİ DEĞİL kaynak) → portalın "Name is taken / may be confused" kontrollerine de takılabilir. **Ad değişmeli** (ör. `<özgün-ad>`; `displayName` içinde "for Claude Code" gibi açıklayıcı kullanım ayrı bir konu, bkz. §2).
2. **Başvuru yolu tek: claude.ai developer portal** (`claude.ai/directory/manage`). Ücretli plan (Pro/Max/Team/Enterprise) gerekir. `anthropics/claude-plugins-official` bireysel başvuru almıyor (yalnız partner iletişimi); `anthropics/claude-plugins-community` reposuna PR açılırsa otomatik kapatılıyor.
3. **İlk çalıştırmada GitHub Releases'tan exe indirmek açıkça yasaklanmamış** ama: plugin klasörüne konan derlenmiş exe "reviewer hold" olur; hook'un çalıştırdığı script içinde indirme/paket kurulumu ve takip edilemeyen script'ler "hold" olur; güvenlik taraması "gizli kod çalıştırma / açıklanmamış hedefe veri gönderme" arar. README'de indirilen her şeyin açıkça yazılması zorunlu pratik. Sonuç: büyük olasılıkla insan incelemesine düşer; reddedilmesi kesin değil, ama garanti de yok.
4. **Yalnız Windows** için resmi bir yasak bulunamadı. Fakat dizin listesi chat/Cowork/Claude Code'a birlikte gider; hook'lar Cowork'te de yüklenir. Dosya adları Windows+macOS'ta geçerli olmalı.
5. **README (≥40 kelime) ve LICENSE (veya `license` alanı) zorunlu**, yoksa başvuru bloklanır. MIT kabul edilir (SPDX).
6. **Logo/Clawd**: Anthropic adları/logoları ürün adında, kendi logonda, ya da onay/ortaklık ima eden şekilde kullanılamaz; logoda değişiklik yasak; diğer kullanımlar yazılı izin ister. Clawd için ayrı bir resmi kural **bulunamadı** — Anthropic markası/sanatı kapsamında değerlendirilmeli.

---

## 1. Resmi dizin / marketplace ve başvuru süreci

### 1.1 Anthropic'in üç marketplace'i + ayrı bir "dizin"

Kaynak: https://code.claude.com/docs/en/plugins/anthropic-marketplaces

| | Official | Community | Demo |
|---|---|---|---|
| Repo | `anthropics/claude-plugins-official` | `anthropics/claude-plugins-community` | `anthropics/claude-code` |
| Marketplace adı | `claude-plugins-official` | `claude-community` | `claude-code-plugins` |
| İçerik | "Plugins Anthropic maintains, plus plugins from partners and other authors" | "Third-party plugins that their authors submitted to Anthropic" | örnek plugin'ler |
| Nasıl eklenir | Claude Code ilk interaktif oturumda kendisi ekler | `/plugin marketplace add anthropics/claude-plugins-community` | `/plugin marketplace add anthropics/claude-code` |

- "Anthropic's directory is separate from these marketplaces. The directory is the catalog on claude.ai." (aynı sayfa)
- Web'de katalog: https://claude.com/marketplace/plugins ("shows install counts and marks some plugins **Anthropic verified**").
- Üçüncü taraf marketplace'ler için: "Anthropic doesn't review third-party marketplaces".

### 1.2 Official marketplace (`claude-plugins-official`) başvuru almıyor

Kaynak: https://code.claude.com/docs/en/plugins/publish
> "Anthropic's official marketplace, `claude-plugins-official`, doesn't take submissions through the directory portal. If you work with an Anthropic partner contact, ask them about an official-marketplace listing."

Repo README'si (https://github.com/anthropics/claude-plugins-official): "Third-party partners can submit plugins for inclusion in the marketplace. External plugins must meet quality and security standards for approval." Form linki `https://clau.de/plugin-directory-submission`, bu da 302 ile `https://claude.com/docs/directory/publish` sayfasına (dizin portalına) yönleniyor. README ayrıca: "The `name` field in a marketplace entry is an immutable slug."

→ Bireysel geliştirici için gerçekçi hedef **dizin (directory)**; official marketplace partner kanalı.

### 1.3 Community marketplace (`claude-community`)

Kaynak: https://github.com/anthropics/claude-plugins-community
- "read-only mirror ... synced nightly from Anthropic's internal review pipeline."
- "Every plugin listed has been submitted via claude.ai, passed automated security scanning, and been approved for distribution."
- Başvuru: `clau.de/plugin-directory-submission` (→ dizin portalı).
- "Pull requests opened directly against this repository are closed automatically."
- Güvenlik sayfasına göre: "Where the `claude-community` catalog pins a plugin to a commit SHA, which it does for nearly every entry, Claude Code refuses to install a different commit." (https://code.claude.com/docs/en/plugins/security)
- Auto-update community marketplace'te **varsayılan kapalı** (https://code.claude.com/docs/en/discover-plugins).

Çıkarım (resmi metinde doğrudan yazmıyor): dizin portalından onaylanan plugin'ler Claude Code tarafında `claude-community` kataloğuna da yansıyor gibi görünüyor; kesin eşleme belgede **bulunamadı**.

### 1.4 Dizine başvuru: kim, nereden, nasıl

Kaynaklar: https://claude.com/docs/directory/publish , https://claude.com/docs/plugins/submit

- Portal: https://claude.ai/directory/manage → **Submit new** → **Plugin bundle**.
- "Anyone on a paid Claude plan can submit, there's no partner program to apply to first, and Anthropic checks each submission before it's listed."
- Plan: "Pro, Max, Team, or Enterprise. Free accounts can't submit". Team/Enterprise'da Owner (ya da Directory izni olan rol).
- Kaynak GitHub reposu olmalı; "The repository can stay private while you validate and submit, and must be public before the listing goes live." GitHub hesabının claude.ai'a bağlanması ve repoya push yetkisi gerekir.
- Adımlar: Source (repo, plugin path, branch/tag) → **Validate** → Listing details (plugin.json + README'den okunur) → **Data handling** soruları ("whether the plugin reads or stores personal data, whether it sends data to services other than its declared connectors, how long it keeps data, and whether it's intended for people under 18") → **Compliance** (iletişim e-postası + dört onay kutusu) → Submit for review.
- Limit: kuruluş başına 24 saatte 10 başvuru; aynı repo+klasör için tek başvuru; "the first organization to submit a given repository folder holds that listing".
- Güncelleme: izlenen branch/tag'e merge → dizin yeni commit'i tarar → yayın ayarına göre yayınlanır. Varsayılan: "An Anthropic reviewer publishes each version".
- İnceleme süresi: "Review time isn't fixed." (https://claude.com/docs/directory/submission-status)
- İletişim: portalda **Get help / Contact Anthropic**, reddedilene **Appeal this decision**; e-posta `directory@anthropic.com`.
- "Verified" etiketi için ayrı başvuru yok; Anthropic inceleme sırasında karar verir.

### 1.5 Kabul kriterleri ve inceleme kuralları

Tüm listelemeler şunlara tabidir:
- Anthropic Software Directory Terms: https://support.claude.com/en/articles/13145338-anthropic-software-directory-terms
- Anthropic Software Directory Policy: https://support.claude.com/en/articles/13145358-anthropic-software-directory-policy

İnceleme mekanizması (https://claude.com/docs/directory/publish):
> "Plugin bundles: every version gets automated validation and a security scan, and a person reviews a new listing before it goes live."

Bulgu türleri (https://claude.com/docs/plugins/pre-submission-checklist): **Blocks** (başvuru yapılamaz), **Policy hold / Held for a reviewer** (başvurulur, insan okur; "A hold isn't a rejection"), **Warning**, **Note**.

Güvenlik taraması:
> "The security scan looks for behavior that a plugin doesn't disclose, such as sending data elsewhere, running hidden code, or changing Claude's permission settings."
> "A first submission that fails the security scan is rejected"
> "Describe in the README everything the plugin runs, sends, or fetches. A complete README doesn't make a behavior allowed."
> "Commit readable source instead of compiled, packed, or minified code. Code that the security scan can't read is held for a reviewer"

Bu plugin için önemli checklist maddeleri (hepsi aynı sayfadan):

| Kural | Sonuç |
|---|---|
| Plugin klasöründe `.claude-plugin/plugin.json` | yoksa Blocks |
| Hook/script'in kullandığı her dosya plugin klasörünün içinde | dışarı işaret eden yol Blocks |
| `.DS_Store`, `Thumbs.db`, `desktop.ini`, `__MACOSX` olmasın | Blocks |
| Dosya adları Windows **ve** macOS'ta geçerli; büyük/küçük harf farkıyla çakışan ad yok | Validation stops |
| Repo GitHub arşivi olarak < 50 MiB, açılmış < 256 MiB, < 10.000 dosya; plugin klasöründe her dosya < 5 MiB | Validation stops |
| Görsel/font dışındaki her dosya < 256 KiB; plugin ≤ 512 dosya | Held |
| Yalnız metin, SVG, PNG/JPEG/GIF/WebP, font. "Any other binary file, such as an `.ico`, `.pdf`, or `.zip` file or a compiled executable, is held." | Held |
| "Don't refer to bundled images or fonts from commands, hooks, or scripts, or write their paths in backticks or a code block." | Held |
| Hook komutlarında yol `${CLAUDE_PLUGIN_ROOT}` ile tam yazılmalı; başka değişken/komut ikamesi/joker/inline program yok | Plugin repo alt klasöründeyse Blocks |
| "Keep launchers and package installs out of each script that a hook or an MCP server runs." | Held ("Scripts the validator couldn't follow") |
| Plugin repo alt klasöründeyse: hook'un shell olmayan bir dosya çalıştırması ya da başka dosyayı çağıran shell script | Held |
| `hooks/hooks.json` geçerli JSON, yalnız belgelenmiş olaylar/tipler | Blocks |
| Gerçek kimlik bilgisi dosyada olmasın | Blocks |
| README ≥ 40 kelime (kod bloğu sayılmaz) | Blocks |
| `LICENSE` dosyası veya `license` alanı | Blocks |
| `description`, `author`, `version` | Warning |

Policy (Directory Policy) içinden ilgili cümleler:
- "Developers must provide a clear, accessible privacy policy link explaining data collection, usage, and retention"
- "Developers must provide verified contact information and support channels for users with product or security concerns"
- "Developers must document how their Software works, its intended purpose"
- "Software must only collect data from the user's context that is necessary to perform their function." (1.D)
- "Instructional Software must not intentionally call or coerce Claude into calling other external software, tools, databases, or resources unless requested and intended by a user." (2.D)
- Yasak kullanım: para/kripto işlemi, bağımsız AI görsel/video/ses üretimi ("design tools permitted"), reklam/sponsorlu içerik.

Terms içinden: geliştirici "implement mechanisms for receiving and investigating security vulnerability reports"; Anthropic "may remove or refuse to display any Software ... at any time for any reason".

`plugin.json`'da dizinin okuduğu alanlar: `icon`, `documentationUrl`, `supportUrl`, `privacyPolicyUrl`, `termsOfServiceUrl` (https://code.claude.com/docs/en/plugins/manifest-reference#directory-listing-fields). Privacy policy linki Policy gereği sağlanmalı.

Yerel ön kontrol: `claude plugin validate --strict ./plugin` (https://code.claude.com/docs/en/plugins/publish). Not: "The portal applies additional directory rules that the CLI doesn't check".

---

## 2. Marka yönergeleri

### 2.1 Plugin adında "claude"

**Claude Code doğrulayıcısı** (https://code.claude.com/docs/en/plugins/manifest-reference#name):

| Ad | Sonuç |
|---|---|
| `claude-`, `anthropic-`, `anthropics-`, `cc-plugin-` ile başlıyor | **Error** |
| Tam olarak `claude`, `anthropic`, `anthropics`, `claude-code`, `claude-mods` | Error |
| `official` + `claude`/`anthropic` yan yana | Error |
| Başka bir yerde tam kelime olarak `claude`/`anthropic` (ör. `mcp-for-claude`) | Warning |

> "The error reads `Plugin name "<name>" is reserved: it passes as one of Anthropic's own` ... `claude plugin init` and `claude plugin tag` refuse a name that draws the error. Only these commands check the name. Claude Code still installs and loads a plugin whose name they refuse."

→ **`claude-pet` = Error.** Kendi marketplace'inde teknik olarak yüklenir, ama `--strict` doğrulama başarısız olur ve dizin için ciddi engel.

**Dizin portalı** (https://claude.com/docs/plugins/pre-submission-checklist, "Manifest and plugin name"):
> "Build the name around your own distinctive product or project name: not a reserved word such as `claude`, `anthropic`, `official`, `plugin`, `mcp`, or `test` as the whole name, not a marketplace name reserved for Anthropic, and nothing that presents the plugin as official" → **Blocks** ("Name is taken"); yalnız generic kelimelerden oluşan ad Held.
> "Choose a name that no other organization's plugin uses. A name that differs only in capitalization or punctuation counts as the same name." → aynı ad Blocks, benzeri Held.
> "Choose a name, `displayName`, and `author.name` that can't be mistaken for an existing plugin, publisher, connector, or well-known brand that isn't yours" → Held ("Name matches a known brand").

Portal metni `claude-pet`'i açıkça "Blocks" diye listelemiyor (yasak örnek "tam ad olarak `claude`"), ama "well-known brand that isn't yours" ve "presents the plugin as official" maddeleri ile CLI'daki `claude-` öneki hatası birlikte düşünülünce ad büyük olasılıkla en az **Held**, muhtemelen reddedilir. `displayName` de marka kontrolüne giriyor; "Claude Pet" gibi bir displayName de aynı riski taşır.

Ad kalıcıdır: "Never change a published plugin's `name`" (https://code.claude.com/docs/en/plugins/publish). → Ad **ilk yayından önce** seçilmeli.

Ad çakışması (RESMİ DEĞİL kaynak, yalnız bilgi): GitHub'da "claude-pet" adlı birçok proje var, örn. https://github.com/daghlny/claude-pet , https://github.com/IMMINJU/claude-pet (Tauri 2), https://github.com/Youl-AI/claude-pet (Windows, "desktop pet plugin for Claude Code"), https://github.com/xtrimsystems/claude-pet , https://github.com/HaneulOscarLee/claude-pet , https://github.com/scm1400/claude-pet . Bunlardan biri dizine başvurmuşsa portal "Name is taken" der.

### 2.2 Marka/logo genel kuralları

Claude Code Legal and compliance (https://code.claude.com/docs/en/legal-and-compliance):
> "You can accurately say, in plain text, that your product has Claude Code preinstalled or that it runs Claude Code. But you can't use the Claude Code or Anthropic names or logos as part of your own product, feature, or company name, in your own logo, or in a way that suggests Anthropic built, endorses, or is partnered with your product. Any other use of Anthropic's names or logos is governed by our Trademark Guidelines and requires our written permission."

(Bu paragraf "Claude Code'u ürününde sunmak" bölümünde geçiyor; ama genel bir marka ilkesi olarak okunabilir.) → README'de düz metinle "a desktop pet for Claude Code" demek uygun; ürün adının parçası yapmak uygun değil.

Anthropic Trademark Guidelines (https://www.anthropic.com/legal/trademark-guidelines):
- "You may only use our trademarks as specifically permitted by us and only in materials we approve beforehand."
- "You may not use our trademarks in a manner that implies Anthropic's sponsorship or endorsement, or a relationship or affiliation with Anthropic, except as we expressly authorize."
- "No alterations of our trademarks (changes to color, font, proportion, or otherwise) are permitted."
- "You may not use our trademarks in any manner that could diminish or damage the reputation of Anthropic or our goods or services."
- İzin: "please email marketing@anthropic.com" (mevcut iş ilişkisi olanlar için).

Directory Terms (https://support.claude.com/en/articles/13145338-anthropic-software-directory-terms):
> "You will not make any statement regarding the Anthropic Services which suggests partnership with, sponsorship by, or endorsement by Anthropic without Anthropic's prior written approval" ; Trademark Guidelines'a uyum şart. "Inclusion of your Software in one or more Directories does not create any partnership..."

### 2.3 Clawd / maskot

- Clawd için ayrı bir resmi kullanım kuralı **bulunamadı**. Resmi repo'da yalnız bir doküman isteği issue'su var: https://github.com/anthropics/claude-code/issues/8536 (kural içermiyor).
- Çıkarım: Clawd Anthropic'in maskot sanatı olduğundan, Trademark Guidelines'daki "yalnız izin verilen şekilde" ve "değişiklik yok" kuralları kapsamında değerlendirilmeli. Clawd'u (ya da Anthropic logosunu, turuncu "spark" işaretini) pet sprite'ı olarak dağıtmak yazılı izin olmadan riskli. GitHub'da "Unofficial fan project" ibaresiyle Clawd kullanan birçok proje var (RESMİ DEĞİL, örn. https://github.com/t1anhe/clawde), bunlar izin olduğunu göstermez.
- Ek dizin riski: portal kuralı "Don't refer to bundled images ... from commands, hooks, or scripts" (Held). Sprite PNG'leri hook/skill'den çağrılıyorsa hold.

### 2.4 "Resmi değildir" ibaresi şartı

- Resmi kaynaklarda **zorunlu bir "unofficial / not affiliated" ibaresi şartı bulunamadı.** Kural, onay/ortaklık **ima etmemek** üzerine kurulu (yukarıdaki alıntılar). README'ye "Not affiliated with or endorsed by Anthropic" cümlesi koymak bu kurala uyumu destekleyen iyi pratik (çıkarım, şart değil).

---

## 3. Plugin'e özel riskler

### (a) İlk çalıştırmada GitHub Releases'tan exe indirmek

- Directory Policy'de runtime'da kod/binary indirme, auto-update, arka plan süreci hakkında **açık hüküm bulunamadı.**
- Plugin klasörüne exe koymak: "a compiled executable, is held" (checklist). Ayrıca dosya < 5 MiB sınırı (aşarsa Validation stops) ve < 256 KiB (aşarsa Held). Tauri exe bunları aşar → exe'yi repoya koymak pratik değil.
- Hook'un çalıştırdığı script'te indirme: checklist yalnız "launchers and package installs" (npx, uvx, pip vb.) için açık kural koyuyor (Held). `curl`/`Invoke-WebRequest` ile exe indirme özel olarak adlandırılmamış; ama güvenlik taraması "running hidden code" ve "sending data elsewhere" arıyor ve okunamayan kodu hold'a alıyor. `.cmd` sarmalayıcı + exe gibi shell olmayan dosya çalıştırma, plugin repo **alt klasöründeyse** "Scripts the validator couldn't follow" hold'una girer; plugin repo kökündeyse bu madde uygulanmıyor gibi görünüyor ("To avoid the hold, keep the plugin at the root of its own repository").
- Rapor başlıklarından biri "Files **or downloads** the validator couldn't inspect" → validator indirmeleri de incelemeye çalışıyor; içeriği göremediği indirmeler hold.
- Marketplace kaynak tipleri arasında `archive` (sha256 pin'li zip) var; `npm` kaynağı "A tarball link on github.com ... refused even when the link is a GitHub release download" (https://code.claude.com/docs/en/plugins/marketplace-reference). Bu, plugin dosyalarının kendisi için; runtime indirmesi için değil, ama Anthropic'in GitHub release tarball'larına temkinli yaklaştığını gösteriyor.
- Kullanıcı tarafı güvenlik: "A Claude Code plugin you install can execute arbitrary code on your machine with your user privileges." ve "Claude Code runs hooks, MCP servers ... outside the sandbox." (https://code.claude.com/docs/en/plugins/security)
- Pratik sonuç: README'de indirme adresi, sürüm, sha256 doğrulaması, nereye yazıldığı açıkça belgelenmeli; indirme betiği okunur kaynak olmalı; büyük olasılıkla insan incelemesi (hold). Kesin kabul/ret kuralı **bulunamadı**.

### (b) 10 olaya bağlanan async hook'lar

Kaynak: https://code.claude.com/docs/en/hooks
- `async: true`: "Claude Code doesn't wait for completion or read the hook's output"; "Exit code and stdout are ignored"; "The `timeout` field is **not enforced** on async hooks".
- Varsayılan timeout 600 sn (komut hook'u); `SessionEnd` hook'ları toplam 1.5 sn bütçe paylaşır.
- "Plugin hooks: always run when plugin is enabled".
- Windows: `shell` alanı varsayılan `bash` (Git Bash), yoksa `powershell`. Exec form (`args`) Windows'ta gerçek `.exe` ister; "`.cmd` and `.bat` shims cannot run in exec form without wrapping them with `node`" → `.cmd` sarmalayıcı shell form ile çağrılmalı.
- Dizin: `hooks/hooks.json` yalnız belgelenmiş olay ve tipleri içermeli (Blocks); yol `${CLAUDE_PLUGIN_ROOT}` ile.
- Olay sayısı için bir sınır **bulunamadı.** Async hook'lar timeout'suz olduğu için takılan süreç birikebilir (çıkarım) → hook'un hızla çıkması gerekir (zaten CLAUDE.md kuralı).
- Hook'lar Cowork'te de yüklenir (https://claude.com/docs/plugins/platform-support) → Cowork kullanıcısında hook çalışır ama pet olmayabilir; sessiz çıkış önemli.

### (c) Arka planda `claude -p` çalıştırmak

- Directory Policy'de "invoking Claude itself" ya da token tüketimi için plugin'e özgü hüküm **bulunamadı** (token tutumluluğu kuralı MCP sunucuları için: "MCP servers must be frugal with their use of tokens").
- Policy 2.D: "must not intentionally call or coerce Claude into calling other external software ... unless requested and intended by a user." → sohbet yalnız kullanıcı isteğiyle başlamalı.
- Kimlik doğrulama (https://code.claude.com/docs/en/legal-and-compliance): OAuth "designed to support ordinary use of Claude Code and other native Anthropic applications"; "Anthropic does not permit third-party developers to offer Claude.ai login into their own applications, or to route requests through Free, Pro, or Max plan credentials on behalf of their users. Moreover, developers may not collect, store, or intermediate Claude.ai credentials or session tokens". Öte yandan: "Nor does it prevent an end user from signing in to the unmodified Claude Code binary with their own Claude subscription".
  → Pet'in kullanıcının kendi kurulu, değiştirilmemiş `claude` CLI'ını kullanıcının kendi oturumuyla çağırması bu metne göre izinli görünüyor (çıkarım); pet kimlik bilgisi okumamalı/saklamamalı/aktarmamalı.
- "Advertised usage limits for Pro and Max plans assume ordinary, individual usage of Claude Code and the Agent SDK." → arka plan çağrıları kullanıcının kotasını harcar; README'de ve Data handling sorularında açıkça belirtilmeli.
- Checklist: "Don't read a credential that is already set in the user's environment ... and send it to a server" → Held/Blocks. Pet ortam değişkenlerinden token okumamalı.

### (d) Yalnız Windows desteklemek

- Resmi bir "çoklu platform zorunluluğu" **bulunamadı.**
- Kısıtlar: dosya adları Windows **ve** macOS'ta geçerli olmalı (Validation stops). Dizin listesi chat/Cowork/Claude Code'a birlikte çıkar; desteklenen yüzeyleri portal bileşenlerden türetir ("the portal derives the surfaces it supports from these same rules") — işletim sistemi seçimi alanı **bulunamadı**. Bu yüzden README'de "Windows only" açıkça yazılmalı ve macOS/Linux'ta hook'lar sessizce çıkmalı (çıkarım).
- Üst düzey `bin/` klasörü chat/Cowork kurulumunu tamamen engeller ("A top-level `bin/` directory stops claude.ai and Cowork from installing the plugin at all") → kullanılmamalı.

### (e) Lisans (MIT) ve README zorunlulukları

- Lisans: "Add a `LICENSE` file to the plugin folder, or set `license` in `plugin.json`" yoksa Blocks. `license` alanı "SPDX identifier such as `MIT` or `Apache-2.0`" (manifest reference). MIT'e karşı bir kural yok; resmi örnek manifest `"license": "MIT"` kullanıyor (https://claude.com/docs/plugins/build).
- README: plugin klasöründe, ≥ 40 kelime, kod blokları sayılmaz; "say what the plugin does, how to use it, and what data it sends" (https://claude.com/docs/plugins/build). Dizin listelemede açıklama olarak gösterilir.
- Ayrıca Policy: privacy policy linki, doğrulanmış iletişim/destek kanalı, işlevin belgelenmesi; Terms: güvenlik açığı bildirim mekanizması (ör. SECURITY.md / GitHub security advisories — çıkarım).
- `plugin.json`: `description`, `author`, `version` (yoksa Warning); `homepage` geçerli URL olmalı (yoksa plugin yüklenmez).

---

## 4. Topluluk marketplace'leri

Resmi (Anthropic):
- `anthropics/claude-plugins-community` (`claude-community`): başvuru yalnız dizin portalı, PR kabul edilmez. https://github.com/anthropics/claude-plugins-community
- Konu bazlı Anthropic marketplace'leri: `anthropics/skills`, `anthropics/knowledge-work-plugins` (https://code.claude.com/docs/en/plugins/anthropic-marketplaces). Bunlara dış başvuru yolu **bulunamadı**.

Kendi marketplace'in (resmi belge, form yok): repoya `.claude-plugin/marketplace.json` eklemek yayın demektir. "Once the file is in the repository, the plugin is published, with no submission form." Kullanıcı: `claude plugin marketplace add <owner>/<repo>` + `claude plugin install <ad>@<marketplace>`. Ayrılmış marketplace adları (ör. `claude-plugins-official`, `claude-community`, `official-claude-plugins` gibi taklitler) kullanılamaz. https://code.claude.com/docs/en/plugins/publish , https://code.claude.com/docs/en/plugins/marketplace-reference#reserved-names

Üçüncü taraf topluluk marketplace'leri (RESMİ DEĞİL kaynaklar; Anthropic incelemez — "Anthropic doesn't review third-party marketplaces"):
- composio-community/awesome-claude-plugins — https://github.com/composio-community/awesome-claude-plugins (küratörlü liste + marketplace.json; başvuru biçimi büyük olasılıkla PR, doğrulanmadı)
- hekmon8/awesome-claude-code-plugins — https://github.com/hekmon8/awesome-claude-code-plugins
- claudemarketplaces.com — https://claudemarketplaces.com/marketplaces (marketplace dizini)
- Başvuru biçimleri resmi kaynakta **bulunamadı**; her reponun CONTRIBUTING dosyasına bakılmalı.

---

## 5. Bulunamayanlar

- Directory Policy'de runtime binary indirme, auto-update, arka plan süreçleri, Claude'u çağırma, işletim sistemi desteği hakkında açık hüküm.
- Zorunlu "unofficial / not affiliated" ibaresi şartı.
- Clawd maskotu için özel kullanım kuralı.
- İnceleme süresi (yalnız "isn't fixed").
- Dizin onayının `claude-community` kataloğuna tam olarak nasıl yansıdığı.
- Üçüncü taraf marketplace'lerin resmi başvuru kuralları.
