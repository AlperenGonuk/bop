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

/// Pet folder for development: the `--pet <folder>` argument, else `BOP_DIR`. When given, it
/// overrides the selection and `config.json` is not watched (DECISIONS.md, decision 28).
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

/// Pet to open: the development folder, else the selected installed pet, else the bundled Pitir.
fn load_active(dev: &Option<PathBuf>) -> Result<Pet, String> {
    let known = known_states();
    if let Some(dir) = dev {
        return pet::load_pet(dir, &known);
    }
    pets::ensure_default_pet();
    let dir = pets::active_dir().ok_or("home folder not found")?;
    pet::load_pet(&dir, &known).or_else(|e| {
        eprintln!("bop: {e}; falling back to the default pet");
        let fallback = pets::pets_dir().ok_or("home folder not found")?.join(pets::DEFAULT_PET_ID);
        pet::load_pet(&fallback, &known)
    })
}

/// Command line: `bop --version | list | use <id> | install <folder> | toggle | stop | hatch ...`.
/// Does the work without opening a window and returns the exit code; `None` for any other
/// argument (the window opens). Output is in English: skills and users read it
/// (DECISIONS.md, decision 21).
pub fn cli_main(args: &[String]) -> Option<i32> {
    if args.first().map(String::as_str) == Some("hatch") {
        return Some(hatch::cli(&args[1..]));
    }
    let result: Result<String, String> = match args.first()?.as_str() {
        // Install scripts (scripts/install.*) compare the installed version with the plugin version.
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
    /// If `--pet` / `BOP_DIR` was given, the selection is not watched.
    dev: Option<PathBuf>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PetView {
    #[serde(flatten)]
    info: PetInfo,
    /// Full path of the spritesheet; the UI loads it via the asset protocol.
    spritesheet_file: PathBuf,
    /// `bop.json` (`present: false` if missing).
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

/// Writes a UI log line to stderr (for diagnosis while the webview console is closed).
#[tauri::command]
fn log(line: String) {
    eprintln!("bop: {line}");
}

/// State vocabulary: [[state, parent state | null], ...]
#[tauri::command]
fn state_vocabulary() -> Vec<(&'static str, Option<&'static str>)> {
    state::STATES.to_vec()
}

/// Last known state (the UI requests it on startup).
#[tauri::command]
fn current_state() -> Option<state::PetState> {
    state::state_path().and_then(|p| state::read_state(&p))
}

/// If the selected pet changed, loads it, grants asset access and sends `pet-changed` to the UI.
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
        Err(e) => eprintln!("bop: couldn't load the selected pet: {e}"),
    }
}

fn quit(app: &tauri::AppHandle) {
    pets::clear_heartbeat();
    app.exit(0);
}

/// Polls files: a `state.json` change emits `pet-state`, a `config.json` change reloads the pet,
/// `control` requests quitting; the single-instance heartbeat is written every second.
/// Polls modification times instead of using a file watcher, to avoid a new dependency.
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
                // No partial reads (atomic writes), but if it is corrupt anyway, retry on the next round.
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

/// Active pet chat (the folder is fixed here).
#[tauri::command]
fn chat_status() -> chat::ChatSession {
    chat::active_session()
}

/// Sends a message; runs in the background so the UI doesn't freeze.
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

/// Answer to the folder trust question.
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

/// Screen position of the cursor and the window (physical pixels); for looking at the mouse.
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

/// Saves the pet's screen position (physical, top-left).
#[tauri::command]
fn save_position(x: i32, y: i32) {
    if let (Some(p), Ok(bytes)) = (position_path(), serde_json::to_vec(&SavedPosition { x, y })) {
        let _ = state::write_atomic(&p, &bytes);
    }
}

/// Moves the window to the saved position if it lies within a connected monitor.
fn restore_position(window: &tauri::WebviewWindow) {
    let Some(saved) = position_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<SavedPosition>(&t).ok())
    else {
        return;
    };
    let on_screen = window.available_monitors().unwrap_or_default().iter().any(|m| {
        let (p, s) = (m.position(), m.size());
        // A bit inside the pet's top-left corner must be visible.
        let (x, y) = (saved.x + 40, saved.y + 40);
        x >= p.x && y >= p.y && x < p.x + s.width as i32 && y < p.y + s.height as i32
    });
    if on_screen {
        let _ = window.set_position(tauri::PhysicalPosition::new(saved.x, saved.y));
    }
}

/// Right-click menu (native popup). The "Switch pet" list is rescanned each time it opens.
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
    // So claude processes started by the pet aren't mistaken for subsessions of the Claude session that launched it.
    chat::clean_inherited_env();
    // Single instance: if the pet is already open, a second one doesn't start.
    if pets::other_instance_running() {
        eprintln!("bop: pet is already running");
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
            // Asset scope covers only pet folders; the active pet is added at runtime (decision 13).
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
                // The selection is written to a file; the watcher loads the pet (same path as `bop use` from a terminal).
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
        .expect("failed to start bop");
}
