//! Kurulu petler, aktif pet seçimi ve tek örnek (KARARLAR.md, 28. karar).
//!
//! - Petler `~/.bop/pets/<id>/` altında; seçim `~/.bop/config.json` (`{"activePet": "<id>"}`).
//! - Varsayılan pet Pıtır exe'ye gömülüdür, açılışta `pets/pitir/` klasörüne yazılır (içerik
//!   farklıysa yenilenir). Seçili pet yoksa ya da yüklenemiyorsa Pıtır açılır.
//! - Tek örnek: çalışan pet `running.json`'a her saniye yazar; taze kayıt varsa ikinci örnek
//!   açılmaz. Kapatma isteği `control` dosyasıyla iletilir (ek bağımlılık yok).

use crate::state;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const DEFAULT_PET_ID: &str = "pitir";
const DEFAULT_PET_JSON: &[u8] = include_bytes!("../default-pet/pet.json");
const DEFAULT_PET_SHEET: &[u8] = include_bytes!("../default-pet/spritesheet.png");

/// Kalp atışı bundan eskiyse pet çalışmıyor sayılır.
const HEARTBEAT_FRESH: Duration = Duration::from_secs(3);
/// `control` dosyasına yazılan kapatma isteği.
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

/// Pet kimliği klasör adı olarak kullanılır: yalnız küçük harf, rakam, `-` ve `_`.
pub fn valid_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

/// Gömülü Pıtır'ı `pets/pitir/` klasörüne yazar; dosya aynıysa dokunmaz.
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

/// `pets/` altındaki geçerli petler (okunabilen `pet.json`, kimlik klasör adıyla aynı);
/// Pıtır en başta, diğerleri ada göre.
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

/// Açılacak petin klasörü: seçili pet kuruluysa o, değilse Pıtır.
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

/// Seçimi yazar; çalışan pet `config.json`'u izler ve yeniden başlatmadan değişir.
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

/// Yerel bir pet klasörünü doğrulayıp `pets/<id>/` altına kopyalar; kimliği döndürür.
/// Aynı kimlikte pet varsa yerine geçer. Pıtır'ın üzerine yazılmaz.
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
        return Ok(id); // zaten kurulu klasörün kendisi
    }
    // Önce geçici klasöre kopyala, sonra eskisinin yerine koy: yarım kurulum kalmasın.
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

/// Yalnız normal dosyalar ve klasörler kopyalanır; bağlantılar atlanır.
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

// --- Tek örnek ----------------------------------------------------------------------------

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

/// Çalışan pet her saniye çağırır.
pub fn beat() {
    let hb = Heartbeat { pid: std::process::id(), at: state::now_ms() };
    if let (Some(p), Ok(bytes)) = (heartbeat_path(), serde_json::to_vec(&hb)) {
        let _ = state::write_atomic(&p, &bytes);
    }
}

/// Başka bir örnek çalışıyor mu (taze kalp atışı, kendi sürecimiz değil).
pub fn other_instance_running() -> bool {
    heartbeat_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<Heartbeat>(&t).ok())
        .is_some_and(|hb| {
            hb.pid != std::process::id()
                && state::now_ms().saturating_sub(hb.at) < HEARTBEAT_FRESH.as_millis() as u64
        })
}

/// Pet kapanırken kalp atışını siler.
pub fn clear_heartbeat() {
    if let Some(p) = heartbeat_path() {
        let _ = fs::remove_file(p);
    }
}

/// Çalışan pete kapanmasını söyler.
pub fn request_quit() -> Result<(), String> {
    let p = control_path().ok_or("home directory not found")?;
    state::write_atomic(&p, QUIT.as_bytes()).map_err(|e| e.to_string())
}

/// Kapatma isteği geldiyse dosyayı siler ve `true` döner.
pub fn take_quit_request() -> bool {
    let Some(p) = control_path() else { return false };
    let asked = fs::read_to_string(&p).is_ok_and(|t| t.trim() == QUIT);
    if asked {
        let _ = fs::remove_file(&p);
    }
    asked
}

/// Peti ayrı bir süreç olarak başlatır (bu süreç beklemeden çıkabilir).
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
        // Kendi süreç grubu: çağıran kabuk kapanınca pet de kapanmasın.
        cmd.process_group(0);
    }
    cmd.spawn().map(|_| ()).map_err(|e| format!("could not start pet: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kimlik_yalniz_guvenli_karakterler() {
        assert!(valid_id("pitir"));
        assert!(valid_id("my_pet-2"));
        for bad in ["", "Pitir", "../x", "a/b", "a b", "ç", &"x".repeat(65)] {
            assert!(!valid_id(bad), "{bad:?} geçmemeli");
        }
    }

    #[test]
    fn config_bilinmeyen_alanlari_yok_sayar() {
        let c: Config = serde_json::from_str(r#"{"activePet":"johnny","other":1}"#).unwrap();
        assert_eq!(c.active_pet.as_deref(), Some("johnny"));
        assert_eq!(serde_json::to_string(&Config::default()).unwrap(), "{}");
    }

    #[test]
    fn gomulu_pet_gecerli() {
        let info = crate::pet::parse_pet_json(std::str::from_utf8(DEFAULT_PET_JSON).unwrap()).unwrap();
        assert_eq!(info.id, DEFAULT_PET_ID);
        assert!(DEFAULT_PET_SHEET.starts_with(b"\x89PNG"));
    }
}
