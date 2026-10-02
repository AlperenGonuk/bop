//! Pet chat: a separate `claude -p` session (DECISIONS.md, decision 7).
//!
//! - The first message runs `claude -p --output-format json`, later ones `--resume <id>`.
//! - Read-only tools, files OR web (DECISIONS.md, decisions 18-20):
//!   - in a trusted folder `Read,Glob,Grep` and no web, so a file that was read can't be
//!     carried out in a search query;
//!   - in an untrusted chat only `WebSearch` (permitted via `--allowedTools`), and the process
//!     runs in the empty `~/.bop/chat` folder: the untrusted folder's `.claude/settings.json`
//!     hooks and CLAUDE.md are never loaded.
//!   Every other permission prompt is denied with `--permission-prompts none` (including reads
//!   outside the working folder). Trust is asked in the bubble before a new chat's first
//!   message; if the project folder is unknown, the chat starts directly in the empty folder as
//!   untrusted. `WebFetch` is never available. MCP tools are disabled with
//!   `--disallowedTools "mcp__*"`.
//! - `--append-system-prompt`: lets Claude know it is in a bubble and what it can't do;
//!   otherwise it writes tool calls as plain text. The prompt is stored with the chat's first
//!   request; older chats continued with `--resume` keep their old one (CLI docs).
//! - The message is passed via stdin (avoids quoting issues in npm's `claude.cmd` wrapper).
//! - No console window opens on Windows (CREATE_NO_WINDOW).
//! - The session folder is fixed at start; after "Open in terminal" the pet no longer writes
//!   to that session.
//! CLI flags verified against the Claude Code CLI docs (2026-09-30, 2026-10-01).

use crate::state;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Maximum time allowed for one message.
const CHAT_TIMEOUT: Duration = Duration::from_secs(300);

/// Tools available in the pet: only file reads in a trusted folder, only web search in an
/// untrusted one; the two are never combined in one session (DECISIONS.md, decision 20).
const TRUSTED_TOOLS: &str = "Read,Glob,Grep";
const UNTRUSTED_TOOLS: &str = "WebSearch";

// Note appended to the system prompt. Since it passes through `claude.cmd`, it must not
// contain double quotes, `%`, `!`, `^`, `&`, `<`, `>` or `|` (checked by a test).
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

/// When the pet is launched from inside a Claude Code session (e.g. `/bop`), it inherits that
/// session's variables. Seeing them, the `claude` started by the pet thinks it is a subsession:
/// transcript saving is turned off (breaking `--resume`) and colors are disabled. These are
/// session-specific; the user's persistent settings (e.g. `CLAUDE_CODE_USE_BEDROCK`) are not
/// in the list.
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

/// Removes inherited Claude session variables from the process environment.
/// `NO_COLOR` is removed only when it evidently came from a Claude session
/// (Claude's shell tool sets it; if it is the user's own setting, it is left alone).
/// Must be called once at startup, before other threads start.
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
    /// Claude session ID; filled in after the first reply.
    pub session_id: Option<String>,
    /// The session's fixed folder.
    pub cwd: Option<String>,
    /// Whether it was handed off to a terminal (if so, the pet no longer writes to this session).
    #[serde(default)]
    pub handed_off: bool,
    /// Whether the folder is trusted: `None` not asked yet (the bubble asks before the first
    /// message), `Some(true)` file reads enabled, `Some(false)` web search only.
    #[serde(default)]
    pub trusted: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatReply {
    pub text: String,
    pub is_error: bool,
    /// The saved session wasn't found; the reply came from a new session (without context).
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

/// Empty folder the chat runs in when the project folder is unknown (`~/.bop/chat`).
/// There is nothing to read in it; the home folder never becomes the working folder by default.
fn neutral_cwd() -> String {
    let dir = state::home_dir().map(|h| h.join("chat")).unwrap_or_else(|| PathBuf::from("."));
    let _ = std::fs::create_dir_all(&dir);
    dir.to_string_lossy().into_owned()
}

/// Folder for a new chat: the user's last Claude session folder, else the empty pet folder.
fn default_cwd() -> String {
    state::state_path()
        .and_then(|p| state::read_state(&p))
        .map(|s| s.last_user_cwd)
        .filter(|c| !c.is_empty() && Path::new(c).is_dir())
        .unwrap_or_else(neutral_cwd)
}

/// The user's home folder (for the "~" shorthand in the bubble header and the home folder warning).
pub fn user_home() -> Option<String> {
    state::user_home().map(|p| p.to_string_lossy().into_owned())
}

// --- Trusted folders (~/.bop/trusted.json) -------------------------------------------

#[derive(Default, Serialize, Deserialize)]
struct TrustList {
    #[serde(default)]
    folders: Vec<String>,
}

fn trust_path() -> Option<PathBuf> {
    state::home_dir().map(|h| h.join("trusted.json"))
}

/// Windows paths: case, separator and trailing separator differences don't matter.
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

/// Trust state of a new session: not asked in the empty pet folder (no files to read, no file
/// tools enabled) or in a previously trusted folder; otherwise the bubble asks.
fn initial_trust(cwd: &str) -> Option<bool> {
    if norm_folder(cwd) == norm_folder(&neutral_cwd()) {
        Some(false)
    } else if is_trusted_folder(cwd) {
        Some(true)
    } else {
        None
    }
}

/// The user's answer: if trusted, the folder is remembered permanently; if not, the chat moves
/// to the empty pet folder (so the untrusted folder's settings, hooks and CLAUDE.md don't load).
/// The question is answered once per session; later answers (e.g. a double click) are ignored.
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

/// Active session; if it was handed off to a terminal, a new one starts (its folder is fixed now).
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

/// The `claude` executable: `BOP_CLAUDE`, else `claude.exe` (native install) or
/// `claude.cmd` (npm) on PATH.
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
    // WebSearch, the only tool that needs permission, is enabled only in an untrusted
    // (file-less) chat.
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

/// Reply text, error flag and session ID from `claude -p` JSON output.
pub fn parse_output(stdout: &str) -> Option<(String, bool, Option<String>)> {
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).ok()?;
    let text = v.get("result").and_then(|r| r.as_str()).unwrap_or("").to_string();
    let is_error = v.get("is_error").and_then(|e| e.as_bool()).unwrap_or(false);
    let sid = v.get("session_id").and_then(|s| s.as_str()).map(String::from);
    Some((text, is_error, sid))
}

/// A session ID may contain only UUID characters (it goes on the command line).
pub fn valid_session_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// Whether the `--resume`d session is missing on disk (deleted, or created while transcript
/// saving was off). Only failed runs are checked: a successful reply's text may contain the
/// phrase.
fn is_unknown_session(ok: bool, stdout: &str, stderr: &str) -> bool {
    let failed = !ok || parse_output(stdout).map_or(true, |(_, is_error, _)| is_error);
    failed && (stdout.contains("No conversation found") || stderr.contains("No conversation found"))
}

pub fn send(message: &str) -> Result<ChatReply, String> {
    let mut session = active_session();
    // Don't start, with or without file tools, before the trust question is answered: the
    // prompt is stored with the chat's first request and can't change later.
    let Some(trusted) = session.trusted else {
        return Err("First answer the folder question in the bubble".into());
    };
    // An untrusted chat never runs in any other folder (including old saved sessions).
    let cwd = if trusted {
        session.cwd.clone().unwrap_or_else(default_cwd)
    } else {
        neutral_cwd()
    };
    let program = claude_program()?;
    // Total time limit, including the retry.
    let deadline = Instant::now() + CHAT_TIMEOUT;

    let (mut ok, mut code, mut stdout, mut stderr) =
        run_claude(&program, session.session_id.as_deref(), trusted, &cwd, message, deadline)?;
    // If the saved session wasn't found, retry once with a new session (same folder).
    // The dead ID is cleared right away, so if the retry also fails, the next message doesn't
    // run twice again.
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

/// Runs `claude -p`: (success, exit code, stdout, stderr).
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
    // Read the outputs on separate threads so a full pipe doesn't block.
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

/// Only http(s) URLs, with no whitespace or control characters (it goes on the command line).
/// Opening links is Windows-only for now; on other platforms only the tests use this.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn valid_url(url: &str) -> bool {
    (url.starts_with("https://") || url.starts_with("http://"))
        && url.len() <= 2048
        && !url.chars().any(|c| c.is_whitespace() || c.is_control() || c == '"')
}

/// Opens a source link in the default browser.
#[cfg(windows)]
pub fn open_url(url: &str) -> Result<(), String> {
    if !valid_url(url) {
        return Err("invalid URL".into());
    }
    // url.dll's handler takes the rest of the command line as is: unlike explorer.exe it doesn't
    // split on commas, and with no shell involved, characters like `&` aren't treated as
    // commands. The URL is passed raw (valid_url already rejects whitespace and quotes).
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

/// Opens the session in a new terminal with `claude --resume <id>` and hands it off.
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

/// First message sent by "Make a new pet": it shows up as the user's message, and the `hatch`
/// skill recognizes the request from its description (a plain sentence, so no command name is
/// shown).
const HATCH_PROMPT: &str = "Let's make a new Bop pet!";

/// Pet creation: `claude "<HATCH_PROMPT>"` in a new terminal, in the empty `~/.bop/hatch`
/// folder (DECISIONS.md, decision 28). Work that writes files happens in the terminal, in
/// plain view of the user.
pub fn open_hatch_terminal() -> Result<(), String> {
    let dir = state::home_dir().ok_or("home folder not found")?.join("hatch");
    std::fs::create_dir_all(&dir).map_err(|e| format!("couldn't create {}: {e}", dir.display()))?;
    spawn_claude_terminal(&dir.to_string_lossy(), &[HATCH_PROMPT])
}

/// Starts `claude <args>` in a new terminal.
fn spawn_claude_terminal(cwd: &str, args: &[&str]) -> Result<(), String> {
    let program = std::env::var("BOP_CLAUDE").unwrap_or_else(|_| "claude".into());
    spawn_terminal(cwd, &program, args)
}

#[cfg(windows)]
fn spawn_terminal(cwd: &str, program: &str, args: &[&str]) -> Result<(), String> {
    // The folder isn't put on the command line (its name may contain cmd characters like `&`);
    // the process starts directly in that folder and the new terminal inherits it.
    // Uses Windows Terminal if available, else a new console window.
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
    fn trusted_folder_allows_only_file_reads_no_web() {
        let a = chat_args(None, true);
        assert_eq!(after(&a, "--tools").as_deref(), Some("Read,Glob,Grep"));
        // No tool runs without permission in a file-reading session: content that was read
        // can't leak out through a search.
        assert!(!a.contains(&"--allowedTools".to_string()));
        assert_eq!(after(&a, "--permission-prompts").as_deref(), Some("none"));
        for tool in ["Bash", "Edit", "Write", "WebFetch", "WebSearch"] {
            assert!(!TRUSTED_TOOLS.split(',').any(|t| t == tool), "{tool} must not be enabled");
        }
        assert!(!a.contains(&"--resume".to_string()));
        let b = chat_args(Some("abc-123"), true);
        assert!(b.ends_with(&["--resume".to_string(), "abc-123".to_string()]));
    }

    #[test]
    fn untrusted_chat_allows_only_web_search() {
        let a = chat_args(None, false);
        assert_eq!(after(&a, "--tools").as_deref(), Some("WebSearch"));
        assert_eq!(after(&a, "--allowedTools").as_deref(), Some("WebSearch"));
        let prompt = after(&a, "--append-system-prompt").unwrap();
        assert!(prompt.contains("cannot read files"));
        assert!(!prompt.contains("Read, Glob"));
        // Button names in the prompt match the ones in the bubble.
        assert!(prompt.contains("Open in terminal"));
        assert!(pet_prompt(true).contains("Don't trust"));
    }

    #[test]
    fn prompt_has_no_cmd_special_chars() {
        // It passes through claude.cmd, so it must not contain cmd special characters.
        for trusted in [true, false] {
            assert!(!pet_prompt(trusted).contains(['"', '%', '!', '^', '&', '<', '>', '|']));
        }
    }

    #[test]
    fn folder_paths_are_compared() {
        assert_eq!(norm_folder("D:/Claude-Pet/"), norm_folder("d:\\claude-pet"));
        assert_ne!(norm_folder("D:\\claude-pet"), norm_folder("D:\\claude-pet2"));
    }

    #[test]
    fn json_output_is_parsed() {
        let (t, e, s) = parse_output(
            r#"{"type":"result","is_error":false,"result":"Merhaba","session_id":"5f0c-aa"}"#,
        )
        .unwrap();
        assert_eq!((t.as_str(), e, s.as_deref()), ("Merhaba", false, Some("5f0c-aa")));
        // Plain non-JSON text (Turkish test data).
        assert!(parse_output("düz metin").is_none());
    }

    #[test]
    fn missing_session_is_checked_only_on_failed_runs() {
        // A successful reply that merely mentions the phrase (Turkish test data).
        let ok_reply = r#"{"is_error":false,"result":"'No conversation found' şu demek...","session_id":"ab"}"#;
        assert!(!is_unknown_session(true, ok_reply, ""));
        assert!(is_unknown_session(false, "", "No conversation found with session ID: ab"));
        let err_reply = r#"{"is_error":true,"result":"No conversation found with session ID: ab"}"#;
        assert!(is_unknown_session(true, err_reply, ""));
        assert!(!is_unknown_session(false, "", "network error"));
    }

    #[test]
    fn only_http_urls_are_opened() {
        assert!(valid_url("https://www.cnnturk.com/hava-durumu-istanbul/?a=1&b=2"));
        assert!(!valid_url("file:///C:/Windows/System32/calc.exe"));
        assert!(!valid_url("calc.exe"));
        assert!(!valid_url("https://x.com/a b"));
        assert!(!valid_url("https://x.com/\"&calc"));
    }

    #[test]
    fn session_id_is_validated() {
        assert!(valid_session_id("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!valid_session_id("x & calc"));
        assert!(!valid_session_id(""));
    }
}
