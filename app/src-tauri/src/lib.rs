mod chat;
mod hatch;
mod pet;
mod pets;
mod state;

pub use state::hook_main;

use pet::{Pet, PetInfo};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::{Emitter, Manager, State};

/// Geliştirme için pet klasörü: `--pet <klasör>` argümanı, yoksa `BOP_DIR`. Verilirse seçimi
/// geçersiz kılar ve `config.json` izlenmez (KARARLAR.md, 28. karar).
fn pet_dir_override() -> Option<PathBuf> {
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--pet" {
            return args.next().map(PathBuf::from);
        }
    }
    std::env::var_os("BOP_DIR").map(PathBuf::from)
}

fn known_states() -> Vec<&'static str> {
    state::STATES.iter().map(|(n, _)| *n).collect()
}

/// Açılacak pet: geliştirme klasörü, yoksa seçili kurulu pet, o da yoksa gömülü Pıtır.
fn load_active(dev: &Option<PathBuf>) -> Result<Pet, String> {
    let known = known_states();
    if let Some(dir) = dev {
        return pet::load_pet(dir, &known);
    }
    pets::ensure_default_pet();
    let dir = pets::active_dir().ok_or("home folder not found")?;
    pet::load_pet(&dir, &known).or_else(|e| {
        eprintln!("bop: {e}; varsayılan pete dönülüyor");
        let fallback = pets::pets_dir().ok_or("home folder not found")?.join(pets::DEFAULT_PET_ID);
        pet::load_pet(&fallback, &known)
    })
}

/// Komut satırı: `bop --version | list | use <id> | install <klasör> | toggle | stop | hatch ...`.
/// Pencere açmadan iş görüp çıkış kodunu döner; başka bir argümanda `None` (pencere açılır).
/// Çıktı İngilizce: skill'ler ve kullanıcılar okur (KARARLAR.md, 21. karar).
pub fn cli_main(args: &[String]) -> Option<i32> {
    if args.first().map(String::as_str) == Some("hatch") {
        return Some(hatch::cli(&args[1..]));
    }
    let result: Result<String, String> = match args.first()?.as_str() {
        // Kurulum betikleri (plugin/scripts/install.*) kurulu sürümü plugin sürümüyle karşılaştırır.
        "--version" | "version" => Ok(format!("bop {}", env!("CARGO_PKG_VERSION"))),
        "list" => {
            pets::ensure_default_pet();
            let active = pets::active_dir();
            Ok(pets::list_installed()
                .iter()
                .map(|p| {
                    let mark = if Some(&p.dir) == active.as_ref() { "*" } else { " " };
                    format!("{mark} {}\t{}", p.id, p.display_name)
                })
                .collect::<Vec<_>>()
                .join("\n"))
        }
        "use" => match args.get(1) {
            Some(id) => pets::use_pet(id).map(|()| format!("active pet: {id}")),
            None => Err("usage: bop use <id>".into()),
        },
        "install" => match args.get(1) {
            Some(dir) => pets::install(std::path::Path::new(dir), &known_states())
                .map(|id| format!("installed: {id}")),
            None => Err("usage: bop install <folder>".into()),
        },
        "toggle" if pets::other_instance_running() => {
            pets::request_quit().map(|()| "pet closed".into())
        }
        "toggle" => pets::spawn_detached().map(|()| "pet opened".into()),
        "stop" if pets::other_instance_running() => {
            pets::request_quit().map(|()| "pet closed".into())
        }
        "stop" => Ok("pet is not running".into()),
        _ => return None,
    };
    Some(match result {
        Ok(out) => {
            if !out.is_empty() {
                println!("{out}");
            }
            0
        }
        Err(e) => {
            eprintln!("bop: {e}");
            1
        }
    })
}

struct ActivePet {
    pet: Mutex<Result<Pet, String>>,
    /// `--pet` / `BOP_DIR` verildiyse seçim izlenmez.
    dev: Option<PathBuf>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PetView {
    #[serde(flatten)]
    info: PetInfo,
    /// Spritesheet'in tam yolu; arayüz asset protokolüyle yükler.
    spritesheet_file: PathBuf,
    /// `bop.json` (yoksa `present: false`).
    config: pet::PetConfig,
}

#[tauri::command]
fn pet_info(active: State<ActivePet>) -> Result<PetView, String> {
    let guard = active.pet.lock().map_err(|e| e.to_string())?;
    let pet = guard.as_ref().map_err(Clone::clone)?;
    Ok(PetView {
        info: pet.info.clone(),
        spritesheet_file: pet.spritesheet.clone(),
        config: pet.config.clone(),
    })
}

/// Arayüz günlüğünü stderr'e yazar (webview konsolu kapalıyken tanı için).
#[tauri::command]
fn log(line: String) {
    eprintln!("bop: {line}");
}

/// Durum sözlüğü: [[durum, üst durum | null], ...]
#[tauri::command]
fn state_vocabulary() -> Vec<(&'static str, Option<&'static str>)> {
    state::STATES.to_vec()
}

/// Son bilinen durum (arayüz açılışta ister).
#[tauri::command]
fn current_state() -> Option<state::PetState> {
    state::state_path().and_then(|p| state::read_state(&p))
}

/// Seçili pet değiştiyse yükler, asset iznini açar ve arayüze `pet-changed` gönderir.
fn reload_pet(app: &tauri::AppHandle) {
    let active = app.state::<ActivePet>();
    if active.dev.is_some() {
        return;
    }
    let Some(dir) = pets::active_dir().and_then(|d| pet::canonical(&d).ok()) else { return };
    let same = active.pet.lock().map(|p| p.as_ref().is_ok_and(|p| p.dir == dir)).unwrap_or(true);
    if same {
        return;
    }
    match pet::load_pet(&dir, &known_states()) {
        Ok(new) => {
            if let Err(e) = app.asset_protocol_scope().allow_directory(&new.dir, true) {
                eprintln!("bop: {e}");
                return;
            }
            if let Ok(mut p) = active.pet.lock() {
                *p = Ok(new);
            }
            let _ = app.emit("pet-changed", ());
        }
        Err(e) => eprintln!("bop: seçilen pet yüklenemedi: {e}"),
    }
}

fn quit(app: &tauri::AppHandle) {
    pets::clear_heartbeat();
    app.exit(0);
}

/// Dosyaları yoklar: `state.json` değişince `pet-state`, `config.json` değişince pet yeniden
/// yüklenir, `control` kapatma ister; her saniye tek örnek kalp atışı yazılır.
/// Yeni bağımlılık olmasın diye dosya izleyici yerine değişiklik zamanı yoklaması.
fn watch_files(app: tauri::AppHandle) {
    let Some(path) = state::state_path() else { return };
    let config = pets::config_path();
    let modified = |p: &Option<PathBuf>| {
        p.as_ref().and_then(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok())
    };
    std::thread::spawn(move || {
        let mut last = None;
        let mut last_config = modified(&config);
        let mut tick: u32 = 0;
        loop {
            if tick % 5 == 0 {
                pets::beat();
                if pets::take_quit_request() {
                    quit(&app);
                    return;
                }
            }
            tick = tick.wrapping_add(1);
            let now_config = modified(&config);
            if now_config != last_config {
                last_config = now_config;
                reload_pet(&app);
            }
            let modified = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
            if modified.is_some() && modified != last {
                last = modified;
                // Yarım okuma olmaz (atomik yazma), yine de bozuksa sonraki turda tekrar dener.
                match state::read_state(&path) {
                    Some(s) => {
                        let _ = app.emit("pet-state", s);
                    }
                    None => last = None,
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    });
}

/// Aktif pet sohbeti (klasör burada sabitlenir).
#[tauri::command]
fn chat_status() -> chat::ChatSession {
    chat::active_session()
}

/// Mesaj gönderir; arka planda çalışır, arayüz donmaz.
#[tauri::command]
async fn chat_send(message: String) -> Result<chat::ChatReply, String> {
    tauri::async_runtime::spawn_blocking(move || chat::send(&message))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn chat_new() -> chat::ChatSession {
    chat::reset_session();
    chat::active_session()
}

#[tauri::command]
fn chat_open_terminal() -> Result<chat::ChatSession, String> {
    chat::open_in_terminal()
}

/// Klasör güven sorusunun cevabı.
#[tauri::command]
fn chat_trust(trust: bool) -> chat::ChatSession {
    chat::set_trust(trust)
}

#[tauri::command]
fn user_home() -> Option<String> {
    chat::user_home()
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    chat::open_url(&url)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CursorInfo {
    cursor_x: f64,
    cursor_y: f64,
    window_x: i32,
    window_y: i32,
}

/// İmlecin ve pencerenin ekran konumu (fiziksel piksel); fareye bakma için.
#[tauri::command]
fn cursor_info(window: tauri::Window) -> Result<CursorInfo, String> {
    let c = window.cursor_position().map_err(|e| e.to_string())?;
    let w = window.outer_position().map_err(|e| e.to_string())?;
    Ok(CursorInfo { cursor_x: c.x, cursor_y: c.y, window_x: w.x, window_y: w.y })
}

#[derive(Serialize, serde::Deserialize)]
struct SavedPosition {
    x: i32,
    y: i32,
}

fn position_path() -> Option<PathBuf> {
    state::home_dir().map(|h| h.join("window.json"))
}

/// Petin ekran konumunu (fiziksel, sol üst) kaydeder.
#[tauri::command]
fn save_position(x: i32, y: i32) {
    if let (Some(p), Ok(bytes)) = (position_path(), serde_json::to_vec(&SavedPosition { x, y })) {
        let _ = state::write_atomic(&p, &bytes);
    }
}

/// Kayıtlı konum bağlı bir monitörün içindeyse pencereyi oraya taşır.
fn restore_position(window: &tauri::WebviewWindow) {
    let Some(saved) = position_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<SavedPosition>(&t).ok())
    else {
        return;
    };
    let on_screen = window.available_monitors().unwrap_or_default().iter().any(|m| {
        let (p, s) = (m.position(), m.size());
        // Petin sol üst köşesinin biraz içi görünür olmalı.
        let (x, y) = (saved.x + 40, saved.y + 40);
        x >= p.x && y >= p.y && x < p.x + s.width as i32 && y < p.y + s.height as i32
    });
    if on_screen {
        let _ = window.set_position(tauri::PhysicalPosition::new(saved.x, saved.y));
    }
}

/// Sağ tık menüsü (yerel popup). "Switch pet" listesi her açılışta taranır.
#[tauri::command]
fn show_menu(window: tauri::Window, active: State<ActivePet>) -> Result<(), String> {
    let err = |e: tauri::Error| e.to_string();
    let current = active.pet.lock().ok().and_then(|p| p.as_ref().ok().map(|p| p.dir.clone()));
    let switch = Submenu::new(&window, "Switch pet", active.dev.is_none()).map_err(err)?;
    for p in pets::list_installed() {
        let checked = pet::canonical(&p.dir).ok() == current;
        let item = CheckMenuItem::with_id(
            &window,
            format!("pet:{}", p.id),
            &p.display_name,
            true,
            checked,
            None::<&str>,
        )
        .map_err(err)?;
        switch.append(&item).map_err(err)?;
    }
    let hatch = MenuItem::with_id(&window, "hatch", "Make a new pet", true, None::<&str>).map_err(err)?;
    let sep = PredefinedMenuItem::separator(&window).map_err(err)?;
    let quit = MenuItem::with_id(&window, "quit", "Quit", true, None::<&str>).map_err(err)?;
    let menu = Menu::with_items(&window, &[&switch, &hatch, &sep, &quit]).map_err(err)?;
    window.popup_menu(&menu).map_err(err)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Pet'in açacağı claude süreçleri, peti başlatan Claude oturumunun alt oturumu sanılmasın.
    chat::clean_inherited_env();
    // Tek örnek: pet zaten açıksa ikincisi açılmaz.
    if pets::other_instance_running() {
        eprintln!("bop: pet zaten açık");
        return;
    }
    pets::beat();
    let dev = pet_dir_override();
    let loaded = load_active(&dev);
    if let Err(e) = &loaded {
        eprintln!("bop: {e}");
    }
    let active = ActivePet { pet: Mutex::new(loaded), dev };
    tauri::Builder::default()
        .setup(|app| {
            // Asset kapsamı yalnız pet klasörleri; aktif pet çalışma anında eklenir (13. karar).
            let dir = app.state::<ActivePet>().pet.lock().ok().and_then(|p| p.as_ref().ok().map(|p| p.dir.clone()));
            if let Some(dir) = dir {
                app.asset_protocol_scope().allow_directory(&dir, true)?;
            }
            if let Some(w) = app.get_webview_window("main") {
                restore_position(&w);
            }
            watch_files(app.handle().clone());
            Ok(())
        })
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id == "quit" {
                quit(app);
            } else if id == "hatch" {
                if let Err(e) = chat::open_hatch_terminal() {
                    eprintln!("bop: {e}");
                }
            } else if let Some(pet_id) = id.strip_prefix("pet:") {
                // Seçim dosyaya yazılır; izleyici peti yükler (terminalden `bop use` ile aynı yol).
                if let Err(e) = pets::use_pet(pet_id) {
                    eprintln!("bop: {e}");
                }
            }
        })
        .manage(active)
        .invoke_handler(tauri::generate_handler![
            pet_info,
            log,
            show_menu,
            state_vocabulary,
            current_state,
            chat_status,
            chat_send,
            chat_new,
            chat_open_terminal,
            chat_trust,
            user_home,
            open_url,
            cursor_info,
            save_position
        ])
        .run(tauri::generate_context!())
        .expect("bop başlatılamadı");
}
