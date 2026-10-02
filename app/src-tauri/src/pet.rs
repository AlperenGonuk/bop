//! Pet klasörü yükleyici (Codex formatı).
//!
//! `pet.json`'dan yalnız 5 alan okunur (KARARLAR.md, 3. karar); diğer alanlar yok sayılır.
//! v1/v2 ayrımı ve kare sayımı görsel üzerinden arayüz tarafında yapılır.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PetInfo {
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub spritesheet_path: String,
    #[serde(default)]
    pub sprite_version_number: Option<u32>,
}

/// Yüklenmiş pet: bilgiler, klasör, spritesheet dosyasının tam yolu ve isteğe bağlı ayarlar.
#[derive(Debug, Clone)]
pub struct Pet {
    pub info: PetInfo,
    pub dir: PathBuf,
    pub spritesheet: PathBuf,
    pub config: PetConfig,
}

/// Ana spritesheet'in `bop.json`'daki ayrılmış adı.
pub const MAIN_SHEET: &str = "main";
pub const CONFIG_FORMAT: u64 = 1;
const FRAME_MS_RANGE: std::ops::RangeInclusive<u64> = 30..=2000;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AnimationDef {
    pub sheet: String,
    pub row: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_ms: Option<u64>,
}

/// `bop.json` (isteğe bağlı; KARARLAR.md 3. ve 8b. karar).
/// Hoşgörülü okunur: hatalı giriş atlanır, `warnings`'e yazılır, pet bozulmaz.
/// Izgara ve satır sınırı denetimi görsel gerektirdiği için arayüzde yapılır.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PetConfig {
    /// Dosya var mıydı.
    pub present: bool,
    pub format_version: u64,
    /// Tüm animasyonlar için varsayılan kare süresi.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_ms: Option<u64>,
    /// Ek sheet adı → tam yol.
    pub sheets: BTreeMap<String, PathBuf>,
    pub animations: BTreeMap<String, AnimationDef>,
    /// Durum → animasyon adı.
    pub states: BTreeMap<String, String>,
    /// Boşta arada oynayan kısa hareketler; `None` ise varsayılan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_extras: Option<Vec<String>>,
    pub warnings: Vec<String>,
}

fn frame_ms_of(v: Option<&Value>, what: &str, warnings: &mut Vec<String>) -> Option<u64> {
    let v = v?;
    match v.as_u64().filter(|n| FRAME_MS_RANGE.contains(n)) {
        Some(n) => Some(n),
        None => {
            warnings.push(format!("{what}: frameMs must be a number from 30 to 2000, ignored"));
            None
        }
    }
}

/// `bop.json` metnini ayrıştırır. `dir` ek sheet yollarını çözmek için.
pub fn parse_config(text: &str, dir: &Path, known_states: &[&str]) -> PetConfig {
    let mut c = PetConfig { present: true, format_version: CONFIG_FORMAT, ..Default::default() };
    let w = &mut c.warnings;
    let root: Value = match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
        Ok(v @ Value::Object(_)) => v,
        Ok(_) => {
            w.push("bop.json is not an object, ignored".into());
            return c;
        }
        Err(e) => {
            w.push(format!("could not read bop.json, ignored: {e}"));
            return c;
        }
    };

    match root.get("formatVersion").map(|v| v.as_u64()) {
        None => {}
        Some(Some(n)) => {
            c.format_version = n;
            if n > CONFIG_FORMAT {
                w.push(format!("formatVersion {n} daha yeni; bilinen alanlar okunuyor"));
            }
        }
        Some(None) => w.push("formatVersion must be a number, ignored".into()),
    }
    c.frame_ms = frame_ms_of(root.get("frameMs"), "genel", w);

    if let Some(sheets) = root.get("sheets").and_then(Value::as_object) {
        for (name, v) in sheets {
            if name == MAIN_SHEET {
                w.push(format!("sheets.{MAIN_SHEET} is reserved for the main sheet, skipped"));
                continue;
            }
            let Some(rel) = v.as_str() else {
                w.push(format!("sheets.{name}: file path must be a string, skipped"));
                continue;
            };
            match safe_relative(rel) {
                Ok(p) if dir.join(p).is_file() => {
                    c.sheets.insert(name.clone(), dir.join(p));
                }
                Ok(_) => w.push(format!("sheets.{name}: {rel} not found, skipped")),
                Err(e) => w.push(format!("sheets.{name}: {e}, skipped")),
            }
        }
    }

    if let Some(anims) = root.get("animations").and_then(Value::as_object) {
        for (name, v) in anims {
            let sheet = v.get("sheet").and_then(Value::as_str).unwrap_or(MAIN_SHEET);
            if sheet != MAIN_SHEET && !c.sheets.contains_key(sheet) {
                w.push(format!("animations.{name}: no sheet named '{sheet}', skipped"));
                continue;
            }
            let Some(row) = v.get("row").and_then(Value::as_u64).and_then(|r| u32::try_from(r).ok())
            else {
                w.push(format!("animations.{name}: row must be 0 or a positive number, skipped"));
                continue;
            };
            let frame_ms = frame_ms_of(v.get("frameMs"), &format!("animations.{name}"), w);
            c.animations.insert(name.clone(), AnimationDef { sheet: sheet.into(), row, frame_ms });
        }
    }

    if let Some(states) = root.get("states").and_then(Value::as_object) {
        for (state, v) in states {
            if !known_states.contains(&state.as_str()) {
                w.push(format!("states.{state}: unknown state, skipped"));
                continue;
            }
            match v.as_str() {
                Some(anim) => {
                    c.states.insert(state.clone(), anim.into());
                }
                None => w.push(format!("states.{state}: animation name must be a string, skipped")),
            }
        }
    }

    match root.get("idleExtras") {
        None => {}
        Some(Value::Array(items)) => {
            let names: Vec<String> =
                items.iter().filter_map(|i| i.as_str().map(String::from)).collect();
            if names.len() != items.len() {
                w.push("idleExtras: non-string entries skipped".into());
            }
            c.idle_extras = Some(names);
        }
        Some(_) => w.push("idleExtras must be a list, ignored".into()),
    }
    c
}

fn load_config(dir: &Path, known_states: &[&str]) -> PetConfig {
    match fs::read_to_string(dir.join("bop.json")) {
        Ok(text) => parse_config(&text, dir, known_states),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => PetConfig::default(),
        Err(e) => PetConfig {
            present: true,
            format_version: CONFIG_FORMAT,
            warnings: vec![format!("could not read bop.json, ignored: {e}")],
            ..Default::default()
        },
    }
}

pub fn parse_pet_json(text: &str) -> Result<PetInfo, String> {
    // Codex dosyaları BOM ile kaydedilmiş olabilir.
    let text = text.trim_start_matches('\u{feff}');
    serde_json::from_str(text).map_err(|e| format!("could not read pet.json: {e}"))
}

/// Pet dosya yolları pet klasörünün dışına çıkamaz (mutlak yol ya da `..` yok).
fn safe_relative(path: &str) -> Result<&Path, String> {
    let p = Path::new(path);
    let ok = !path.is_empty()
        && p.components().all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    if ok {
        Ok(p)
    } else {
        Err(format!("path must stay inside the pet folder: {path}"))
    }
}

/// Mutlak yol; Windows'taki `\\?\C:\...` önekini (UNC olmayanlarda) atar.
pub fn canonical(path: &Path) -> Result<PathBuf, String> {
    let p = fs::canonicalize(path).map_err(|e| format!("{} not found: {e}", path.display()))?;
    let s = p.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(rest) if !rest.starts_with("UNC\\") => Ok(PathBuf::from(rest)),
        _ => Ok(p),
    }
}

pub fn load_pet(dir: &Path, known_states: &[&str]) -> Result<Pet, String> {
    // Asset kapsamı ve yol karşılaştırmaları için mutlak yol.
    let dir = &canonical(dir)?;
    let json_path = dir.join("pet.json");
    let text = fs::read_to_string(&json_path)
        .map_err(|e| format!("could not read {}: {e}", json_path.display()))?;
    let info = parse_pet_json(&text)?;
    let spritesheet = dir.join(safe_relative(&info.spritesheet_path)?);
    if !spritesheet.is_file() {
        return Err(format!("spritesheet not found: {}", spritesheet.display()));
    }
    let config = load_config(dir, known_states);
    Ok(Pet { info, dir: dir.clone(), spritesheet, config })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bes_alan_okunur_fazlasi_yok_sayilir() {
        let info = parse_pet_json(
            r#"{"displayName":"Johnny","spriteVersionNumber":2,"id":"johnny",
                "spritesheetPath":"spritesheet.webp","description":"x","kind":"bilinmeyen"}"#,
        )
        .unwrap();
        assert_eq!(info.id, "johnny");
        assert_eq!(info.sprite_version_number, Some(2));
    }

    #[test]
    fn surum_numarasi_ve_aciklama_istege_bagli() {
        let info = parse_pet_json(
            "\u{feff}{\"id\":\"a\",\"displayName\":\"A\",\"spritesheetPath\":\"s.webp\"}",
        )
        .unwrap();
        assert_eq!(info.sprite_version_number, None);
        assert_eq!(info.description, "");
    }

    const STATES: &[&str] = &["idle", "thinking", "writing-code"];

    fn temp_pet_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("bop-cfg-{tag}-{}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("extra.webp"), b"x").unwrap();
        d
    }

    #[test]
    fn gecerli_ayar_okunur() {
        let d = temp_pet_dir("ok");
        let c = parse_config(
            r#"{"formatVersion":1,"frameMs":120,
                "sheets":{"extra":"extra.webp"},
                "animations":{"yawn":{"sheet":"extra","row":0,"frameMs":140},"slow-idle":{"row":0}},
                "states":{"thinking":"yawn"},
                "idleExtras":["yawn","look"]}"#,
            &d,
            STATES,
        );
        assert!(c.warnings.is_empty(), "{:?}", c.warnings);
        assert_eq!(c.frame_ms, Some(120));
        assert_eq!(c.sheets["extra"], d.join("extra.webp"));
        assert_eq!(c.animations["yawn"], AnimationDef { sheet: "extra".into(), row: 0, frame_ms: Some(140) });
        // sheet verilmezse ana sheet.
        assert_eq!(c.animations["slow-idle"].sheet, MAIN_SHEET);
        assert_eq!(c.states["thinking"], "yawn");
        assert_eq!(c.idle_extras.as_deref(), Some(&["yawn".to_string(), "look".to_string()][..]));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn hatali_girisler_atlanir_pet_bozulmaz() {
        let d = temp_pet_dir("bad");
        let c = parse_config(
            r#"{"formatVersion":"x","frameMs":5,
                "sheets":{"main":"extra.webp","yok":"yok.webp","kacak":"../extra.webp","sayi":3},
                "animations":{"a":{"sheet":"yok","row":0},"b":{"row":-1},"c":{"row":2,"frameMs":99999}},
                "states":{"uyuyor":"a","thinking":5},
                "idleExtras":"yawn"}"#,
            &d,
            STATES,
        );
        assert!(c.sheets.is_empty());
        // Yalnız "c" geçerli; hatalı frameMs'i yok sayılır.
        assert_eq!(c.animations.keys().collect::<Vec<_>>(), vec!["c"]);
        assert_eq!(c.animations["c"].frame_ms, None);
        assert!(c.states.is_empty());
        assert_eq!(c.idle_extras, None);
        assert_eq!(c.frame_ms, None);
        // formatVersion, frameMs, 4 sheet, 3 animasyon, 2 durum, idleExtras.
        assert_eq!(c.warnings.len(), 12, "{:#?}", c.warnings);
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn bozuk_ya_da_eksik_ayar_dosyasi() {
        let d = temp_pet_dir("broken");
        let c = parse_config("{ bozuk", &d, STATES);
        assert!(c.present && c.warnings.len() == 1 && c.animations.is_empty());
        // Dosya yoksa uyarısız varsayılan.
        let none = load_config(&d, STATES);
        assert!(!none.present && none.warnings.is_empty());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn klasor_disina_cikan_yol_reddedilir() {
        assert!(safe_relative("../x.webp").is_err());
        assert!(safe_relative("C:\\x.webp").is_err());
        assert!(safe_relative("").is_err());
        assert!(safe_relative("alt/s.webp").is_ok());
    }
}
