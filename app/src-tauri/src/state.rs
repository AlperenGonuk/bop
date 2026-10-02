//! State bridge: hook event → `~/.bop/state.json` (DECISIONS.md, decisions 5 and 6).
//!
//! Hook fields verified against the Claude Code hooks docs (2026-09-30):
//! common `session_id`, `cwd`, `hook_event_name`; `tool_name` on tool events;
//! `notification_type` on Notification (`permission_prompt`, `idle_prompt` ...).

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const STATE_FORMAT: u32 = 1;

/// Environment variable marking the pet's own `claude -p` sessions.
/// The hook process inherits Claude's environment.
pub const PET_CHAT_ENV: &str = "BOP_CHAT";

/// State vocabulary (fixed list). A state the pet does not define falls back along `parent`.
pub const STATES: &[(&str, Option<&str>)] = &[
    ("idle", None),
    ("running", Some("idle")),
    ("thinking", Some("running")),
    ("reading", Some("running")),
    ("writing-code", Some("running")),
    ("running-command", Some("running")),
    ("waiting-permission", Some("idle")),
    ("done", Some("idle")),
    ("failed", Some("idle")),
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PetState {
    pub format: u32,
    pub state: String,
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub cwd: String,
    /// Whether the event came from the pet's own chat session.
    #[serde(default)]
    pub from_pet: bool,
    /// Folder of the user's last (non-pet) Claude session; a pet session does not overwrite it.
    /// A new pet chat opens in this folder (DECISIONS.md, decision 7).
    #[serde(default)]
    pub last_user_cwd: String,
    /// Unix time, milliseconds.
    pub updated_at: u64,
}

/// `~/.bop` (can be overridden with `BOP_HOME` in tests).
pub fn home_dir() -> Option<PathBuf> {
    if let Some(h) = std::env::var_os("BOP_HOME") {
        return Some(PathBuf::from(h));
    }
    Some(user_home()?.join(".bop"))
}

/// The user's home directory (`USERPROFILE`, else `HOME`).
pub fn user_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from)
}

pub fn state_path() -> Option<PathBuf> {
    home_dir().map(|h| h.join("state.json"))
}

fn tool_state(tool: &str) -> &'static str {
    match tool {
        "Edit" | "Write" | "MultiEdit" | "NotebookEdit" => "writing-code",
        "Read" | "Grep" | "Glob" | "WebFetch" | "WebSearch" | "LS" => "reading",
        "Bash" | "PowerShell" => "running-command",
        _ => "running",
    }
}

/// Derives the state from hook input. `None` for events the pet does not care about.
pub fn state_from_event(input: &Value, event_arg: Option<&str>) -> Option<(String, &'static str)> {
    let event = input
        .get("hook_event_name")
        .and_then(Value::as_str)
        .or(event_arg)?
        .to_string();
    let tool = input.get("tool_name").and_then(Value::as_str).unwrap_or("");
    let state = match event.as_str() {
        "SessionStart" | "SessionEnd" => "idle",
        "UserPromptSubmit" | "PostToolUse" | "PostToolBatch" => "thinking",
        "PreToolUse" => tool_state(tool),
        "PermissionRequest" => "waiting-permission",
        "Notification" => match input.get("notification_type").and_then(Value::as_str) {
            Some("permission_prompt") | Some("elicitation_dialog") => "waiting-permission",
            Some("idle_prompt") => "idle",
            _ => return None,
        },
        "PostToolUseFailure" | "StopFailure" => "failed",
        "Stop" => "done",
        _ => return None,
    };
    Some((event, state))
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Atomic write: temp file in the same folder, then rename over the target.
/// On Windows it retries a few times if a reader has the file open at that moment.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(dir)?;
    let tmp = dir.join(format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("state"),
        std::process::id()
    ));
    fs::write(&tmp, bytes)?;
    let mut last = None;
    for _ in 0..5 {
        match fs::rename(&tmp, path) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last = Some(e);
                std::thread::sleep(Duration::from_millis(15));
            }
        }
    }
    let _ = fs::remove_file(&tmp);
    Err(last.unwrap())
}

pub fn read_state(path: &Path) -> Option<PetState> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// `bop hook [event]`: reads the hook input from stdin and writes the state file.
/// Always returns silently; the exit code is always 0 (never breaks Claude Code).
pub fn hook_main(event_arg: Option<&str>) {
    let _ = run_hook(event_arg, &mut std::io::stdin().lock());
}

pub fn run_hook(event_arg: Option<&str>, stdin: &mut dyn Read) -> Option<PetState> {
    let mut text = String::new();
    // The input is small; the cap avoids hanging on a broken stream.
    stdin.take(4 * 1024 * 1024).read_to_string(&mut text).ok()?;
    let input: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()?;
    let (event, state) = state_from_event(&input, event_arg)?;
    let get = |k: &str| input.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let path = state_path()?;
    let tool = get("tool_name");
    let cwd = get("cwd");
    let from_pet = std::env::var_os(PET_CHAT_ENV).is_some();
    let last_user_cwd = if from_pet || cwd.is_empty() {
        read_state(&path).map(|p| p.last_user_cwd).unwrap_or_default()
    } else {
        cwd.clone()
    };
    let record = PetState {
        format: STATE_FORMAT,
        state: state.to_string(),
        event,
        tool: (!tool.is_empty()).then_some(tool),
        session_id: get("session_id"),
        cwd,
        from_pet,
        last_user_cwd,
        updated_at: now_ms(),
    };
    let bytes = serde_json::to_vec_pretty(&record).ok()?;
    write_atomic(&path, &bytes).ok()?;
    Some(record)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn st(v: Value) -> Option<&'static str> {
        state_from_event(&v, None).map(|(_, s)| s)
    }

    #[test]
    fn events_map_to_states() {
        assert_eq!(st(json!({"hook_event_name":"PreToolUse","tool_name":"Edit"})), Some("writing-code"));
        assert_eq!(st(json!({"hook_event_name":"PreToolUse","tool_name":"Grep"})), Some("reading"));
        assert_eq!(st(json!({"hook_event_name":"PreToolUse","tool_name":"Bash"})), Some("running-command"));
        assert_eq!(st(json!({"hook_event_name":"PreToolUse","tool_name":"mcp__x__y"})), Some("running"));
        assert_eq!(st(json!({"hook_event_name":"UserPromptSubmit"})), Some("thinking"));
        assert_eq!(st(json!({"hook_event_name":"Stop"})), Some("done"));
        assert_eq!(st(json!({"hook_event_name":"PostToolUseFailure","tool_name":"Bash"})), Some("failed"));
        assert_eq!(
            st(json!({"hook_event_name":"Notification","notification_type":"permission_prompt"})),
            Some("waiting-permission")
        );
        assert_eq!(st(json!({"hook_event_name":"Notification","notification_type":"auth_success"})), None);
        assert_eq!(st(json!({"hook_event_name":"PreCompact"})), None);
    }

    #[test]
    fn event_name_falls_back_to_argument() {
        assert_eq!(state_from_event(&json!({}), Some("Stop")).map(|(_, s)| s), Some("done"));
        assert_eq!(state_from_event(&json!({}), None), None);
    }

    #[test]
    fn every_state_chain_ends_at_idle() {
        for (name, _) in STATES {
            let mut cur = *name;
            let mut steps = 0;
            while let Some(parent) = STATES.iter().find(|(n, _)| *n == cur).and_then(|(_, p)| *p) {
                cur = parent;
                steps += 1;
                assert!(steps < 10, "{name} chain has a cycle");
            }
            assert_eq!(cur, "idle", "{name}");
        }
    }

    #[test]
    fn atomic_write_and_read() {
        let dir = std::env::temp_dir().join(format!("bop-test-{}", std::process::id()));
        let path = dir.join("state.json");
        let rec = PetState {
            format: STATE_FORMAT,
            state: "reading".into(),
            event: "PreToolUse".into(),
            tool: Some("Read".into()),
            session_id: "s1".into(),
            cwd: "D:\\x".into(),
            from_pet: false,
            last_user_cwd: "D:\\x".into(),
            updated_at: 1,
        };
        write_atomic(&path, &serde_json::to_vec(&rec).unwrap()).unwrap();
        // Must be able to overwrite an existing file.
        let rec2 = PetState { state: "done".into(), event: "Stop".into(), ..rec };
        write_atomic(&path, &serde_json::to_vec(&rec2).unwrap()).unwrap();
        assert_eq!(read_state(&path), Some(rec2));
        // No temp file should be left behind.
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn broken_input_is_silently_ignored() {
        assert!(run_hook(Some("Stop"), &mut "this is not json".as_bytes()).is_none());
    }
}
