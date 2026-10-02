//! Installed pets, active pet selection and single instance (DECISIONS.md, decision 28).
//!
//! - Pets live under `~/.bop/pets/<id>/`; the selection is in `~/.bop/config.json`
//!   (`{"activePet": "<id>"}`).
//! - The default pet Pitir is embedded in the exe and written to `pets/pitir/` at startup
//!   (rewritten if the content differs). If no pet is selected or it fails to load, Pitir opens.
//! - Single instance: the running pet writes `running.json` every second; if a fresh record
//!   exists, a second instance does not start. A quit request goes through the `control` file
//!   (no extra dependency).

use crate::state;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const DEFAULT_PET_ID: &str = "pitir";
const DEFAULT_PET_JSON: &[u8] = include_bytes!("../default-pet/pet.json");
const DEFAULT_PET_SHEET: &[u8] = include_bytes!("../default-pet/spritesheet.png");

/// If the heartbeat is older than this, the pet is considered not running.
const HEARTBEAT_FRESH: Duration = Duration::from_secs(3);
/// Quit request written to the `control` file.
const QUIT: &str = "quit";

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_pet: Option<String>,
}

pub fn pets_dir() -> Option<PathBuf> {
    state::home_dir().map(|h| h.join("pets"))
}

pub fn config_path() -> Option<PathBuf> {
    state::home_dir().map(|h| h.join("config.json"))
}

pub fn read_config() -> Config {
    config_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok())
        .unwrap_or_default()
}

/// The pet id is used as a folder name: only lowercase letters, digits, `-` and `_`.
pub fn valid_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

/// Writes the embedded Pitir to `pets/pitir/`; leaves identical files untouched.
pub fn ensure_default_pet() -> Option<PathBuf> {
    let dir = pets_dir()?.join(DEFAULT_PET_ID);
    for (name, bytes) in [("pet.json", DEFAULT_PET_JSON), ("spritesheet.png", DEFAULT_PET_SHEET)] {
        let path = dir.join(name);
        if fs::read(&path).ok().as_deref() != Some(bytes) {
            state::write_atomic(&path, bytes).ok()?;
        }
    }
    Some(dir)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPet {
    pub id: String,
    pub display_name: String,
    pub dir: PathBuf,
}

/// Valid pets under `pets/` (readable `pet.json`, id equal to the folder name);
/// Pitir first, the rest by name.
pub fn list_installed() -> Vec<InstalledPet> {
    let Some(root) = pets_dir() else { return Vec::new() };
    let Ok(entries) = fs::read_dir(&root) else { return Vec::new() };
    let mut pets: Vec<InstalledPet> = entries
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| {
            let dir = e.path();
            let text = fs::read_to_string(dir.join("pet.json")).ok()?;
            let info = crate::pet::parse_pet_json(&text).ok()?;
            let folder = e.file_name().to_string_lossy().into_owned();
            (info.id == folder && valid_id(&folder))
                .then(|| InstalledPet { id: folder, display_name: info.display_name, dir })
        })
        .collect();
    pets.sort_by(|a, b| {
        (a.id != DEFAULT_PET_ID, a.display_name.to_lowercase())
            .cmp(&(b.id != DEFAULT_PET_ID, b.display_name.to_lowercase()))
    });
    pets
}

/// Folder of the pet to open: the selected pet if installed, otherwise Pitir.
pub fn active_dir() -> Option<PathBuf> {
    let root = pets_dir()?;
    if let Some(id) = read_config().active_pet.filter(|id| valid_id(id)) {
        let dir = root.join(&id);
        if dir.join("pet.json").is_file() {
            return Some(dir);
        }
    }
    Some(root.join(DEFAULT_PET_ID))
}

/// Writes the selection; the running pet watches `config.json` and switches without a restart.
pub fn use_pet(id: &str) -> Result<(), String> {
    if !list_installed().iter().any(|p| p.id == id) {
        return Err(format!("no installed pet with id '{id}' (see: bop list)"));
    }
    let path = config_path().ok_or("home directory not found")?;
    let mut config = read_config();
    config.active_pet = Some(id.to_string());
    let bytes = serde_json::to_vec_pretty(&config).map_err(|e| e.to_string())?;
    state::write_atomic(&path, &bytes).map_err(|e| format!("could not write config: {e}"))
}

/// Validates a local pet folder and copies it to `pets/<id>/`; returns the id.
/// Replaces an existing pet with the same id. Pitir is never overwritten.
pub fn install(src: &Path, known_states: &[&str]) -> Result<String, String> {
    let pet = crate::pet::load_pet(src, known_states)?;
    let id = pet.info.id.clone();
    if !valid_id(&id) {
        return Err(format!(
            "pet id '{id}' is not valid: use 1-64 lowercase letters, digits, '-' or '_'"
        ));
    }
    if id == DEFAULT_PET_ID {
        return Err(format!("'{DEFAULT_PET_ID}' is the built-in pet; choose another id"));
    }
    let root = pets_dir().ok_or("home directory not found")?;
    let dest = root.join(&id);
    if crate::pet::canonical(&dest).ok().as_deref() == Some(pet.dir.as_path()) {
        return Ok(id); // already the installed folder itself
    }
    // Copy to a temp folder first, then swap it in: no half-finished install is left behind.
    let tmp = root.join(format!(".install-{id}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp);
    copy_dir(&pet.dir, &tmp).map_err(|e| {
        let _ = fs::remove_dir_all(&tmp);
        format!("could not copy pet: {e}")
    })?;
    if dest.exists() {
        fs::remove_dir_all(&dest).map_err(|e| format!("could not replace old pet: {e}"))?;
    }
    fs::rename(&tmp, &dest).map_err(|e| format!("could not install pet: {e}"))?;
    Ok(id)
}

/// Only regular files and folders are copied; links are skipped.
fn copy_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let to = dest.join(entry.file_name());
        if kind.is_dir() {
            copy_dir(&entry.path(), &to)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}

// --- Single instance ----------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
struct Heartbeat {
    pid: u32,
    at: u64,
}

fn heartbeat_path() -> Option<PathBuf> {
    state::home_dir().map(|h| h.join("running.json"))
}

fn control_path() -> Option<PathBuf> {
    state::home_dir().map(|h| h.join("control"))
}

/// Called by the running pet every second.
pub fn beat() {
    let hb = Heartbeat { pid: std::process::id(), at: state::now_ms() };
    if let (Some(p), Ok(bytes)) = (heartbeat_path(), serde_json::to_vec(&hb)) {
        let _ = state::write_atomic(&p, &bytes);
    }
}

/// Whether another instance is running (fresh heartbeat, not our own process).
pub fn other_instance_running() -> bool {
    heartbeat_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<Heartbeat>(&t).ok())
        .is_some_and(|hb| {
            hb.pid != std::process::id()
                && state::now_ms().saturating_sub(hb.at) < HEARTBEAT_FRESH.as_millis() as u64
        })
}

/// Removes the heartbeat when the pet closes.
pub fn clear_heartbeat() {
    if let Some(p) = heartbeat_path() {
        let _ = fs::remove_file(p);
    }
}

/// Tells the running pet to quit.
pub fn request_quit() -> Result<(), String> {
    let p = control_path().ok_or("home directory not found")?;
    state::write_atomic(&p, QUIT.as_bytes()).map_err(|e| e.to_string())
}

/// If a quit request arrived, removes the file and returns `true`.
pub fn take_quit_request() -> bool {
    let Some(p) = control_path() else { return false };
    let asked = fs::read_to_string(&p).is_ok_and(|t| t.trim() == QUIT);
    if asked {
        let _ = fs::remove_file(&p);
    }
    asked
}

/// Starts the pet as a separate process (this process can exit without waiting).
pub fn spawn_detached() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let mut cmd = std::process::Command::new(exe);
    cmd.stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        cmd.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Own process group: the pet must not close when the calling shell closes.
        cmd.process_group(0);
    }
    cmd.spawn().map(|_| ()).map_err(|e| format!("could not start pet: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_allows_only_safe_characters() {
        assert!(valid_id("pitir"));
        assert!(valid_id("my_pet-2"));
        // "ç" is deliberate: a non-ASCII letter must be rejected.
        for bad in ["", "Pitir", "../x", "a/b", "a b", "ç", &"x".repeat(65)] {
            assert!(!valid_id(bad), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn config_ignores_unknown_fields() {
        let c: Config = serde_json::from_str(r#"{"activePet":"johnny","other":1}"#).unwrap();
        assert_eq!(c.active_pet.as_deref(), Some("johnny"));
        assert_eq!(serde_json::to_string(&Config::default()).unwrap(), "{}");
    }

    #[test]
    fn embedded_pet_is_valid() {
        let info = crate::pet::parse_pet_json(std::str::from_utf8(DEFAULT_PET_JSON).unwrap()).unwrap();
        assert_eq!(info.id, DEFAULT_PET_ID);
        assert!(DEFAULT_PET_SHEET.starts_with(b"\x89PNG"));
    }
}
