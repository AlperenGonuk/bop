//! Pet sohbeti: ayrı bir `claude -p` oturumu (KARARLAR.md, 7. karar).
//!
//! - İlk mesaj `claude -p --output-format json`, sonrakiler `--resume <id>`.
//! - Yalnız okuyan araçlar, dosya YA DA web (KARARLAR.md, 18–20. karar):
//!   - güvenilen klasörde `Read,Glob,Grep`, web yok: okunan dosya bir arama sorgusuyla dışarı
//!     taşınamasın;
//!   - güvenilmeyen sohbette yalnız `WebSearch` (`--allowedTools` ile izinsiz) ve süreç boş
//!     `~/.bop/chat` klasöründe çalışır: güvenilmeyen klasörün `.claude/settings.json`
//!     hook'ları ve CLAUDE.md'si hiç yüklenmez.
//!   Başka her izin sorusu `--permission-prompts none` ile reddedilir (çalışma klasörü dışını
//!   okumak dahil). Güven yeni sohbetin ilk mesajından önce balonda sorulur; proje klasörü
//!   bilinmiyorsa sohbet doğrudan boş klasörde, güvenilmeyen olarak başlar. `WebFetch` hiç yok.
//!   MCP araçları `--disallowedTools "mcp__*"` ile kapalı.
//! - `--append-system-prompt`: Claude balonda olduğunu ve neyi yapamadığını bilsin; yoksa araç
//!   çağrısını düz metin olarak yazıyor. Talimat sohbetin ilk isteğinde kaydedilir, `--resume`
//!   ile devam eden eski sohbetler eskisini kullanır (CLI belgesi).
//! - Mesaj stdin'den verilir (npm'in `claude.cmd` sarmalayıcısında tırnak sorunu olmasın).
//! - Windows'ta konsol penceresi açılmaz (CREATE_NO_WINDOW).
//! - Oturum klasörü açılışta sabitlenir; "Open in terminal" sonrası pet o oturuma yazmaz.
//! CLI bayrakları Claude Code CLI belgesinden doğrulandı (2026-09-30, 2026-10-01).

use crate::state;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Bir mesaja verilen en uzun süre.
const CHAT_TIMEOUT: Duration = Duration::from_secs(300);

/// Pette kullanılabilen araçlar: güvenilen klasörde yalnız dosya okuma, güvenilmeyende yalnız
/// web araması; ikisi aynı oturumda hiç birlikte olmaz (KARARLAR.md, 20. karar).
const TRUSTED_TOOLS: &str = "Read,Glob,Grep";
const UNTRUSTED_TOOLS: &str = "WebSearch";

// Sistem talimatına eklenen not. `claude.cmd` üzerinden geçtiği için çift tırnak, `%`, `!`,
// `^`, `&`, `<`, `>`, `|` içermez (testte denetlenir).
const PROMPT_HEAD: &str = "You are answering inside the small chat bubble of a desktop pet, \
not in a terminal. The bubble shows only a few lines, so answer like a friend in a chat: \
one to three short sentences of plain text. No markdown, no headings, no bullet lists, \
no tables, no emoji. Answer only what was asked, without unrequested advice or tips. \
Example for a weather question: Today Istanbul is 14 to 20 degrees and rainy. \
Give more detail only when the user asks for it. \
Never mention sources in the text. If you used WebSearch, put the sources only at the \
very end, one per line, each as a markdown link in the form [title](url), with no heading; \
the bubble folds them away.";
const PROMPT_TRUSTED_TOOLS: &str = "Your only tools are Read, Glob and Grep, for the files \
in this folder. Web search is off in this chat so that file contents cannot leave the \
computer; if the user asks for something from the web, say so in one sentence and suggest \
starting a new chat with the plus button in the bubble header and answering Don't trust to \
the folder question. You cannot run commands, edit files or open URLs";
const PROMPT_UNTRUSTED_TOOLS: &str = "Your only tool is WebSearch. You cannot read files, \
run commands, edit files or open URLs";
const PROMPT_TAIL: &str = "and nobody can approve a permission prompt here. Never write \
tool calls as text. If a request needs more than these tools, say so in one sentence and \
suggest the Open in terminal button in the bubble header, which continues this chat in a \
terminal with full tools. Reply in the language of the user.";

fn pet_tools(trusted: bool) -> &'static str {
    if trusted { TRUSTED_TOOLS } else { UNTRUSTED_TOOLS }
}

fn pet_prompt(trusted: bool) -> String {
    let tools = if trusted { PROMPT_TRUSTED_TOOLS } else { PROMPT_UNTRUSTED_TOOLS };
    format!("{PROMPT_HEAD} {tools}, {PROMPT_TAIL}")
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Pet bir Claude Code oturumunun içinden başlatılınca (ör. `/bop`) o oturuma ait
/// değişkenleri devralır. Pet'in açtığı `claude` bunları görünce kendini alt oturum sanar:
/// transkript kaydı kapanır (`--resume` bozulur), renkler kapanır. Bunlar oturuma özeldir;
/// kullanıcının kalıcı ayarları (ör. `CLAUDE_CODE_USE_BEDROCK`) listede yoktur.
const INHERITED_SESSION_VARS: &[&str] = &[
    "CLAUDECODE",
    "CLAUDE_CODE_CHILD_SESSION",
    "CLAUDE_CODE_ENTRYPOINT",
    "CLAUDE_CODE_MESSAGING_SOCKET",
    "CLAUDE_CODE_MESSAGING_TOKEN",
    "CLAUDE_CODE_SESSION_ATTENDED",
    "CLAUDE_CODE_SESSION_ID",
    "CLAUDE_PID",
];

/// Devralınan Claude oturum değişkenlerini süreç ortamından siler.
/// `NO_COLOR` yalnız bir Claude oturumundan devralındığı anlaşılıyorsa silinir
/// (Claude'un shell aracı koyar; kullanıcının kendi ayarıysa dokunulmaz).
/// Diğer iş parçacıkları başlamadan, açılışta bir kez çağrılmalı.
pub fn clean_inherited_env() {
    let from_claude = std::env::var_os("CLAUDECODE").is_some()
        || std::env::var_os("CLAUDE_CODE_CHILD_SESSION").is_some();
    for v in INHERITED_SESSION_VARS {
        std::env::remove_var(v);
    }
    if from_claude {
        std::env::remove_var("NO_COLOR");
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatSession {
    /// Claude oturum kimliği; ilk cevaptan sonra dolar.
    pub session_id: Option<String>,
    /// Oturumun sabit klasörü.
    pub cwd: Option<String>,
    /// Terminale devredildi mi (evetse pet bu oturuma yazmaz).
    #[serde(default)]
    pub handed_off: bool,
    /// Klasöre güveniliyor mu: `None` henüz sorulmadı (ilk mesajdan önce balon sorar),
    /// `Some(true)` dosya okuma açık, `Some(false)` yalnız web araması.
    #[serde(default)]
    pub trusted: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatReply {
    pub text: String,
    pub is_error: bool,
    /// Kayıtlı oturum bulunamadı, cevap yeni (bağlamsız) bir oturumdan geldi.
    pub new_session: bool,
    pub session: ChatSession,
}

fn session_path() -> Option<PathBuf> {
    state::home_dir().map(|h| h.join("chat.json"))
}

pub fn load_session() -> ChatSession {
    session_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn save_session(s: &ChatSession) {
    if let (Some(p), Ok(bytes)) = (session_path(), serde_json::to_vec_pretty(s)) {
        let _ = state::write_atomic(&p, &bytes);
    }
}

pub fn reset_session() -> ChatSession {
    let s = ChatSession::default();
    save_session(&s);
    s
}

/// Proje klasörü bilinmiyorsa sohbetin çalıştığı boş klasör (`~/.bop/chat`).
/// İçinde okunacak bir şey yok; ev dizini hiçbir zaman kendiliğinden çalışma klasörü olmaz.
fn neutral_cwd() -> String {
    let dir = state::home_dir().map(|h| h.join("chat")).unwrap_or_else(|| PathBuf::from("."));
    let _ = std::fs::create_dir_all(&dir);
    dir.to_string_lossy().into_owned()
}

/// Yeni sohbetin klasörü: kullanıcının son Claude oturumu, yoksa boş pet klasörü.
fn default_cwd() -> String {
    state::state_path()
        .and_then(|p| state::read_state(&p))
        .map(|s| s.last_user_cwd)
        .filter(|c| !c.is_empty() && Path::new(c).is_dir())
        .unwrap_or_else(neutral_cwd)
}

/// Kullanıcının ev dizini (balon başlığında "~" kısaltması ve ev klasörü uyarısı için).
pub fn user_home() -> Option<String> {
    state::user_home().map(|p| p.to_string_lossy().into_owned())
}

// --- Güvenilen klasörler (~/.bop/trusted.json) ---------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct TrustList {
    #[serde(default)]
    folders: Vec<String>,
}

fn trust_path() -> Option<PathBuf> {
    state::home_dir().map(|h| h.join("trusted.json"))
}

/// Windows yolları: büyük/küçük harf, ayraç ve sondaki ayraç farkı önemsiz.
fn norm_folder(p: &str) -> String {
    p.replace('/', "\\").trim_end_matches('\\').to_lowercase()
}

fn load_trust() -> TrustList {
    trust_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn is_trusted_folder(cwd: &str) -> bool {
    let n = norm_folder(cwd);
    load_trust().folders.iter().any(|f| norm_folder(f) == n)
}

fn remember_trusted(cwd: &str) {
    let mut list = load_trust();
    if list.folders.iter().any(|f| norm_folder(f) == norm_folder(cwd)) {
        return;
    }
    list.folders.push(cwd.to_string());
    if let (Some(p), Ok(bytes)) = (trust_path(), serde_json::to_vec_pretty(&list)) {
        let _ = state::write_atomic(&p, &bytes);
    }
}

/// Yeni oturumun güven durumu: boş pet klasöründe sorulmaz (okunacak dosya yok, araç da
/// açılmaz), daha önce güvenilen klasörde sorulmaz, diğerlerinde balon sorar.
fn initial_trust(cwd: &str) -> Option<bool> {
    if norm_folder(cwd) == norm_folder(&neutral_cwd()) {
        Some(false)
    } else if is_trusted_folder(cwd) {
        Some(true)
    } else {
        None
    }
}

/// Kullanıcının cevabı: güvenirse klasör kalıcı olarak hatırlanır; güvenmezse sohbet boş pet
/// klasörüne taşınır (güvenilmeyen klasörün ayarları, hook'ları ve CLAUDE.md'si yüklenmesin).
/// Soru bir oturumda bir kez cevaplanır; sonraki cevaplar (ör. çift tık) yok sayılır.
pub fn set_trust(trust: bool) -> ChatSession {
    let mut s = active_session();
    if s.trusted.is_some() {
        return s;
    }
    if trust {
        if let Some(cwd) = &s.cwd {
            remember_trusted(cwd);
        }
    } else {
        s.cwd = Some(neutral_cwd());
    }
    s.trusted = Some(trust);
    save_session(&s);
    s
}

/// Aktif oturum; terminale devredildiyse yenisi başlar (klasör şimdi sabitlenir).
pub fn active_session() -> ChatSession {
    let s = load_session();
    if s.handed_off || s.cwd.is_none() {
        let cwd = default_cwd();
        let fresh = ChatSession {
            session_id: None,
            trusted: initial_trust(&cwd),
            cwd: Some(cwd),
            handed_off: false,
        };
        save_session(&fresh);
        fresh
    } else {
        s
    }
}

/// `claude` yürütülebilir dosyası: `BOP_CLAUDE`, yoksa PATH'te
/// `claude.exe` (yerel kurulum) ya da `claude.cmd` (npm).
fn claude_program() -> Result<PathBuf, String> {
    if let Some(p) = std::env::var_os("BOP_CLAUDE") {
        return Ok(PathBuf::from(p));
    }
    let names: &[&str] = if cfg!(windows) { &["claude.exe", "claude.cmd"] } else { &["claude"] };
    let path = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path) {
        for n in names {
            let c = dir.join(n);
            if c.is_file() {
                return Ok(c);
            }
        }
    }
    Err("claude not found (no claude.exe or claude.cmd on PATH)".into())
}

fn hide_console(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = cmd;
}

pub fn chat_args(session_id: Option<&str>, trusted: bool) -> Vec<String> {
    let mut a: Vec<String> = ["-p", "--output-format", "json", "--tools", pet_tools(trusted)]
        .iter()
        .map(|s| s.to_string())
        .collect();
    // İzin isteyen tek araç WebSearch, yalnız güvenilmeyen (dosyasız) sohbette açılır.
    if !trusted {
        a.extend(["--allowedTools".into(), "WebSearch".into()]);
    }
    a.extend(
        ["--disallowedTools", "mcp__*", "--permission-prompts", "none", "--append-system-prompt"]
            .iter()
            .map(|s| s.to_string()),
    );
    a.push(pet_prompt(trusted));
    if let Some(id) = session_id {
        a.push("--resume".into());
        a.push(id.into());
    }
    a
}

/// `claude -p` JSON çıktısından cevap metni, hata bayrağı ve oturum kimliği.
pub fn parse_output(stdout: &str) -> Option<(String, bool, Option<String>)> {
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).ok()?;
    let text = v.get("result").and_then(|r| r.as_str()).unwrap_or("").to_string();
    let is_error = v.get("is_error").and_then(|e| e.as_bool()).unwrap_or(false);
    let sid = v.get("session_id").and_then(|s| s.as_str()).map(String::from);
    Some((text, is_error, sid))
}

/// Oturum kimliği yalnız UUID karakterleri içerebilir (komut satırına gider).
pub fn valid_session_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// `--resume` edilen oturum diskte yoksa (silinmiş, ya da transkript kaydı kapalıyken açılmış).
/// Yalnız başarısız çalışmaya bakılır: başarılı bir cevabın metni bu ifadeyi içerebilir.
fn is_unknown_session(ok: bool, stdout: &str, stderr: &str) -> bool {
    let failed = !ok || parse_output(stdout).map_or(true, |(_, is_error, _)| is_error);
    failed && (stdout.contains("No conversation found") || stderr.contains("No conversation found"))
}

pub fn send(message: &str) -> Result<ChatReply, String> {
    let mut session = active_session();
    // Güven sorusu cevaplanmadan dosya araçlarıyla ya da onlarsız başlamak yok: talimat
    // sohbetin ilk isteğinde kaydedilir, sonradan değişmez.
    let Some(trusted) = session.trusted else {
        return Err("First answer the folder question in the bubble".into());
    };
    // Güvenilmeyen sohbet hiçbir zaman başka bir klasörde çalışmaz (eski kayıtlar dahil).
    let cwd = if trusted {
        session.cwd.clone().unwrap_or_else(default_cwd)
    } else {
        neutral_cwd()
    };
    let program = claude_program()?;
    // Yeniden deneme dahil toplam süre sınırı.
    let deadline = Instant::now() + CHAT_TIMEOUT;

    let (mut ok, mut code, mut stdout, mut stderr) =
        run_claude(&program, session.session_id.as_deref(), trusted, &cwd, message, deadline)?;
    // Kayıtlı oturum bulunamadıysa bir kez yeni oturumla dene (klasör aynı kalır).
    // Ölü kimlik hemen silinir: deneme de başarısız olursa sonraki mesaj yine iki kez çalışmasın.
    let mut new_session = false;
    if session.session_id.is_some() && is_unknown_session(ok, &stdout, &stderr) {
        session.session_id = None;
        save_session(&session);
        new_session = true;
        (ok, code, stdout, stderr) = run_claude(&program, None, trusted, &cwd, message, deadline)?;
    }

    match parse_output(&stdout) {
        Some((text, is_error, sid)) => {
            if let Some(id) = sid.filter(|i| valid_session_id(i)) {
                session.session_id = Some(id);
            }
            session.cwd = Some(cwd);
            save_session(&session);
            Ok(ChatReply { text, is_error: is_error || !ok, new_session, session })
        }
        None => {
            let first = stderr.lines().chain(stdout.lines()).find(|l| !l.trim().is_empty());
            Err(format!("claude failed (exit {code}): {}", first.unwrap_or("no output")))
        }
    }
}

/// `claude -p` çalıştırır: (başarılı mı, çıkış kodu, stdout, stderr).
fn run_claude(
    program: &Path,
    session_id: Option<&str>,
    trusted: bool,
    cwd: &str,
    message: &str,
    deadline: Instant,
) -> Result<(bool, i32, String, String), String> {
    let mut cmd = Command::new(program);
    cmd.args(chat_args(session_id, trusted))
        .current_dir(cwd)
        .env(state::PET_CHAT_ENV, "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_console(&mut cmd);

    let mut child = cmd.spawn().map_err(|e| format!("couldn't start claude: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(message.as_bytes());
    }
    // Çıktıları ayrı iş parçacıklarında oku; boru dolup takılmasın.
    let mut out = child.stdout.take().unwrap();
    let mut err = child.stderr.take().unwrap();
    let out_t = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        s
    });
    let err_t = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        s
    });

    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break st,
            Ok(None) if Instant::now() > deadline => {
                let _ = child.kill();
                return Err("claude didn't answer within 5 minutes".into());
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => return Err(format!("error while waiting for claude: {e}")),
        }
    };
    let stdout = out_t.join().unwrap_or_default();
    let stderr = err_t.join().unwrap_or_default();
    Ok((status.success(), status.code().unwrap_or(-1), stdout, stderr))
}

/// Yalnız http(s) adresleri; boşluk ve kontrol karakteri yok (komut satırına gider).
/// Bağlantı açma şimdilik yalnız Windows'ta; öbür platformlarda yalnız testler kullanır.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn valid_url(url: &str) -> bool {
    (url.starts_with("https://") || url.starts_with("http://"))
        && url.len() <= 2048
        && !url.chars().any(|c| c.is_whitespace() || c.is_control() || c == '"')
}

/// Kaynak bağlantısını varsayılan tarayıcıda açar.
#[cfg(windows)]
pub fn open_url(url: &str) -> Result<(), String> {
    if !valid_url(url) {
        return Err("invalid URL".into());
    }
    // url.dll'in işleyicisi komut satırının kalanını olduğu gibi alır: explorer.exe gibi virgülden
    // bölmez, kabuk da olmadığı için `&` gibi karakterler komut sayılmaz. Adres ham verilir
    // (valid_url boşluk ve tırnağı zaten reddeder).
    use std::os::windows::process::CommandExt;
    Command::new("rundll32.exe")
        .arg("url.dll,FileProtocolHandler")
        .raw_arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("couldn't open the browser: {e}"))
}

#[cfg(not(windows))]
pub fn open_url(_url: &str) -> Result<(), String> {
    Err("opening links only works on Windows for now".into())
}

/// Oturumu yeni bir terminalde `claude --resume <id>` ile açar ve devreder.
pub fn open_in_terminal() -> Result<ChatSession, String> {
    let mut session = load_session();
    let id = session.session_id.clone().filter(|i| valid_session_id(i));
    let (Some(id), Some(cwd)) = (id, session.cwd.clone()) else {
        return Err("No chat to hand off yet".into());
    };
    spawn_claude_terminal(&cwd, &["--resume", &id])?;
    session.handed_off = true;
    save_session(&session);
    Ok(session)
}

/// "Yeni pet yap" ile gönderilen ilk mesaj: kullanıcının mesajı olarak görünür, `hatch` skill'i
/// bu isteği açıklamasından tanır (komut adı göstermemek için düz cümle).
const HATCH_PROMPT: &str = "Let's make a new Bop pet!";

/// Pet oluşturma: yeni terminalde, boş `~/.bop/hatch` klasöründe `claude "<HATCH_PROMPT>"`
/// (KARARLAR.md, 28. karar). Yazan iş terminalde, kullanıcının gözü önünde yapılır.
pub fn open_hatch_terminal() -> Result<(), String> {
    let dir = state::home_dir().ok_or("home folder not found")?.join("hatch");
    std::fs::create_dir_all(&dir).map_err(|e| format!("couldn't create {}: {e}", dir.display()))?;
    spawn_claude_terminal(&dir.to_string_lossy(), &[HATCH_PROMPT])
}

/// Yeni bir terminalde `claude <args>` başlatır.
fn spawn_claude_terminal(cwd: &str, args: &[&str]) -> Result<(), String> {
    let program = std::env::var("BOP_CLAUDE").unwrap_or_else(|_| "claude".into());
    spawn_terminal(cwd, &program, args)
}

#[cfg(windows)]
fn spawn_terminal(cwd: &str, program: &str, args: &[&str]) -> Result<(), String> {
    // Klasör komut satırına yazılmaz (adında `&` gibi cmd karakterleri olabilir);
    // süreç doğrudan o klasörde başlatılır, yeni terminal klasörü devralır.
    // Windows Terminal varsa onu, yoksa yeni bir konsol penceresi.
    let wt = Command::new("wt.exe")
        .current_dir(cwd)
        .args(["-d", ".", "cmd", "/k", program])
        .args(args)
        .spawn();
    if wt.is_ok() {
        return Ok(());
    }
    let mut cmd = Command::new("cmd");
    cmd.current_dir(cwd).args(["/c", "start", "", "cmd", "/k", program]).args(args);
    hide_console(&mut cmd);
    cmd.spawn().map(|_| ()).map_err(|e| format!("couldn't open a terminal: {e}"))
}

#[cfg(not(windows))]
fn spawn_terminal(_cwd: &str, _program: &str, _args: &[&str]) -> Result<(), String> {
    Err("opening a terminal only works on Windows for now".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn after(a: &[String], flag: &str) -> Option<String> {
        a.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone())
    }

    #[test]
    fn guvenilen_klasorde_yalniz_dosya_okuma_web_yok() {
        let a = chat_args(None, true);
        assert_eq!(after(&a, "--tools").as_deref(), Some("Read,Glob,Grep"));
        // Dosya okuyan oturumda izinsiz çalışan araç yok: okunan içerik aramayla dışarı çıkamaz.
        assert!(!a.contains(&"--allowedTools".to_string()));
        assert_eq!(after(&a, "--permission-prompts").as_deref(), Some("none"));
        for tool in ["Bash", "Edit", "Write", "WebFetch", "WebSearch"] {
            assert!(!TRUSTED_TOOLS.split(',').any(|t| t == tool), "{tool} açık olmamalı");
        }
        assert!(!a.contains(&"--resume".to_string()));
        let b = chat_args(Some("abc-123"), true);
        assert!(b.ends_with(&["--resume".to_string(), "abc-123".to_string()]));
    }

    #[test]
    fn guvenilmeyen_sohbette_yalniz_web_aramasi() {
        let a = chat_args(None, false);
        assert_eq!(after(&a, "--tools").as_deref(), Some("WebSearch"));
        assert_eq!(after(&a, "--allowedTools").as_deref(), Some("WebSearch"));
        let prompt = after(&a, "--append-system-prompt").unwrap();
        assert!(prompt.contains("cannot read files"));
        assert!(!prompt.contains("Read, Glob"));
        // Talimattaki düğme adları balondakilerle aynı.
        assert!(prompt.contains("Open in terminal"));
        assert!(pet_prompt(true).contains("Don't trust"));
    }

    #[test]
    fn talimatta_cmd_ozel_karakteri_yok() {
        // claude.cmd üzerinden geçer: cmd'nin özel karakterleri olmamalı.
        for trusted in [true, false] {
            assert!(!pet_prompt(trusted).contains(['"', '%', '!', '^', '&', '<', '>', '|']));
        }
    }

    #[test]
    fn klasor_yollari_karsilastirilir() {
        assert_eq!(norm_folder("D:/Claude-Pet/"), norm_folder("d:\\claude-pet"));
        assert_ne!(norm_folder("D:\\claude-pet"), norm_folder("D:\\claude-pet2"));
    }

    #[test]
    fn json_cikti_ayristirilir() {
        let (t, e, s) = parse_output(
            r#"{"type":"result","is_error":false,"result":"Merhaba","session_id":"5f0c-aa"}"#,
        )
        .unwrap();
        assert_eq!((t.as_str(), e, s.as_deref()), ("Merhaba", false, Some("5f0c-aa")));
        assert!(parse_output("düz metin").is_none());
    }

    #[test]
    fn kayip_oturum_yalniz_basarisiz_calismada_aranir() {
        let ok_reply = r#"{"is_error":false,"result":"'No conversation found' şu demek...","session_id":"ab"}"#;
        assert!(!is_unknown_session(true, ok_reply, ""));
        assert!(is_unknown_session(false, "", "No conversation found with session ID: ab"));
        let err_reply = r#"{"is_error":true,"result":"No conversation found with session ID: ab"}"#;
        assert!(is_unknown_session(true, err_reply, ""));
        assert!(!is_unknown_session(false, "", "network error"));
    }

    #[test]
    fn yalniz_http_adresleri_acilir() {
        assert!(valid_url("https://www.cnnturk.com/hava-durumu-istanbul/?a=1&b=2"));
        assert!(!valid_url("file:///C:/Windows/System32/calc.exe"));
        assert!(!valid_url("calc.exe"));
        assert!(!valid_url("https://x.com/a b"));
        assert!(!valid_url("https://x.com/\"&calc"));
    }

    #[test]
    fn oturum_kimligi_dogrulanir() {
        assert!(valid_session_id("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!valid_session_id("x & calc"));
        assert!(!valid_session_id(""));
    }
}
