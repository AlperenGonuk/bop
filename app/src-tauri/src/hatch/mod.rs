//! `bop hatch`: API'siz, kodla pet çizimi (KARARLAR.md 22, 27).
//!
//! `bop hatch <spec.json> <çıktı-klasörü>` → `pet.json`, `spritesheet.png` (v2 atlas),
//! `contact-sheet.png` (QA için etiketli), `report.json`. Pencere açmaz.
//! Çıkış kodu: 0 sorunsuz, 2 doğrulama hatası (dosyalar yine yazılır), 1 spec/girdi hatası.

mod atlas;
mod color;
mod motion;
mod rig;
mod spec;
mod validate;

use atlas::Image;
use motion::{NEUTRAL_CELL, ROWS};
use rig::{Frame, TAG_OVERRIDE};
use spec::{Character, OverrideMode};
use std::path::Path;
use validate::{Issue, Level};

const USAGE: &str = "usage:
  bop hatch <spec.json> <output-folder>   draw a pet from a spec
  bop hatch --example [name]              print an example spec (pitir, critter, floaty, custom)
  bop hatch --docs <name>                 print a reference (spec-format, qa-rubric, animation-rows)
  bop hatch --check <spritesheet.png>     validate an existing v2 spritesheet";

/// Skill'in referansları exe'ye gömülü: Claude onları plugin klasörünü okuma izni istemeden,
/// skill'in izin verdiği tek komutla (`bop hatch --docs`) okur. Kaynak plugin'deki dosyalar.
const DOCS: &[(&str, &str)] = &[
    ("spec-format", include_str!("../../../../plugin/skills/hatch/references/spec-format.md")),
    ("qa-rubric", include_str!("../../../../plugin/skills/hatch/references/qa-rubric.md")),
    ("animation-rows", include_str!("../../../../plugin/skills/hatch/references/animation-rows.md")),
];

/// Çizim sonucu: atlas, mantıksal kareler ve sorunlar (kareler testlerde okunur).
#[cfg_attr(not(test), allow(dead_code))]
pub struct Hatched {
    pub sheet: Image,
    pub frames: Vec<Vec<Frame>>,
    pub neutral: Frame,
    pub issues: Vec<Issue>,
}

fn apply_override(f: &mut Frame, ch: &Character, o: &spec::Override) {
    if o.mode == OverrideMode::Replace {
        *f = Frame::new(f.w, f.h);
    }
    for (j, row) in o.rows.iter().enumerate() {
        for (i, c) in row.chars().enumerate() {
            if let Some(col) = ch.pixel(c, &o.key) {
                f.set(o.at[0] + i as i32, o.at[1] + j as i32, col, TAG_OVERRIDE);
            }
        }
    }
}

/// Karakterin bütün karelerini çizer, atlası kurar ve doğrular. Pet (gövde ya da parçaları)
/// hücreden taşarsa karakter zemin ortası etrafında adım adım küçültülür (en çok %80'e), uyarı
/// olarak bildirilir. Yalnız override taşıyorsa küçültülmez: hata override'ı taşımayı söyler.
pub fn hatch(ch: &Character) -> Hatched {
    let mut base = base_frames(ch);
    if !any_clipped(&base) {
        return finish(ch, &base);
    }
    let mut fit = None;
    let mut cur = ch.clone();
    for s in [0.95, 0.9, 0.85, 0.8] {
        cur = ch.shrunk(s);
        base = base_frames(&cur);
        let fits = !any_clipped(&base);
        fit = Some(validate::fit_warning(s, fits));
        if fits {
            break;
        }
    }
    let mut h = finish(&cur, &base);
    h.issues.extend(fit);
    h
}

/// Küçültmeden tek deneme (testler taşmayı bununla görür).
#[cfg(test)]
fn hatch_once(ch: &Character) -> Hatched {
    finish(ch, &base_frames(ch))
}

/// Kare hücre kenarına değiyor mu (mantıksal ızgara hücreyi tam kaplar: kenar pikseli = hücrenin
/// kenardaki 2 pikseli, `validate::check_atlas`'taki "clipped" denetimiyle aynı).
fn touches_edge(f: &Frame) -> bool {
    (0..f.h).any(|y| (0..f.w).any(|x| (x == 0 || y == 0 || x + 1 == f.w || y + 1 == f.h) && !f.px[y * f.w + x].is_clear()))
}

fn any_clipped(frames: &[Vec<Frame>]) -> bool {
    frames.iter().flatten().any(touches_edge)
}

/// Satırların override'sız kareleri (zıplama yüksekliği hücreye göre seçilir).
fn base_frames(ch: &Character) -> Vec<Vec<Frame>> {
    let (jump, jumping) = jump_frames(ch);
    ROWS.iter()
        .enumerate()
        .map(|(r, _)| {
            if r == JUMP_ROW {
                jumping.clone()
            } else {
                motion::row_poses(ch, r, jump).iter().map(|p| rig::render(ch, p)).collect()
            }
        })
        .collect()
}

const JUMP_ROW: usize = 4;

/// Zıplama yüksekliği: 9'dan 0.5 adımla inen adaylardan, bütün kareleri hücrenin üstünden en az
/// k piksel aşağıda kalan en yükseği (hiçbiri sığmazsa 1). Yükseklik arttıkça tepe yükseldiği
/// için ikili arama yeter. Seçilen yüksekliğin kareleri de döner (yeniden çizilmesin).
fn jump_frames(ch: &Character) -> (f64, Vec<Frame>) {
    let k = ch.grid.k();
    let top_of = |f: &Frame| (0..f.h).find(|&y| (0..f.w).any(|x| !f.px[y * f.w + x].is_clear())).unwrap_or(0);
    let draw = |j: f64| -> Vec<Frame> { motion::row_poses(ch, JUMP_ROW, j).iter().map(|p| rig::render(ch, p)).collect() };
    let fits = |fs: &[Frame]| fs.iter().all(|f| top_of(f) as f64 >= k);
    // Adaylar: 9, 8.5, ..., 1.5; sonuncusu (1) denetimsiz kabul.
    const LAST: usize = 16;
    let jump_at = |i: usize| 9.0 - 0.5 * i as f64;
    let (mut lo, mut hi) = (0usize, LAST);
    let mut best: Option<Vec<Frame>> = None;
    while lo < hi {
        let mid = (lo + hi) / 2;
        let fs = draw(jump_at(mid));
        if fits(&fs) {
            hi = mid;
            best = Some(fs);
        } else {
            lo = mid + 1;
        }
    }
    let jump = jump_at(lo);
    // `best` her zaman son sığan denemenin (`hi`, döngü sonunda `lo`) kareleridir.
    let frames = best.unwrap_or_else(|| draw(jump));
    (jump, frames)
}

/// Override'ları uygular, atlası kurar ve doğrular. `base` override'sız karelerdir.
fn finish(ch: &Character, base: &[Vec<Frame>]) -> Hatched {
    let g = ch.grid;
    let mut frames: Vec<Vec<Frame>> = base.to_vec();
    for (r, row) in ROWS.iter().enumerate() {
        for o in ch.overrides.iter().filter(|o| o.row == row.name) {
            apply_override(&mut frames[r][o.frame], ch, o);
        }
    }
    let neutral = frames[0][0].clone();

    let mut sheet = Image::new(atlas::SHEET_W, atlas::SHEET_H);
    for (r, fs) in frames.iter().enumerate() {
        for (c, f) in fs.iter().enumerate() {
            atlas::blit(&mut sheet, c, r, f, g.scale);
        }
    }
    atlas::blit(&mut sheet, NEUTRAL_CELL.1, NEUTRAL_CELL.0, &neutral, g.scale);

    let mut issues = validate::check_atlas(&sheet);
    issues.extend(validate::check_semantics(&frames, &neutral));
    // Kenara yalnız override değiyorsa küçültme işe yaramaz: mesaj override'ı gösterir.
    for i in issues.iter_mut().filter(|i| i.code == "clipped") {
        let (Some(r), Some(c)) = (i.row, i.frame) else { continue };
        let b = if (r, c) == NEUTRAL_CELL { &base[0][0] } else { &base[r][c] };
        if !touches_edge(b) {
            let (orow, oframe) = if (r, c) == NEUTRAL_CELL { ("idle", 0) } else { (ROWS[r].name, c) };
            i.message = format!(
                "the override for row '{orow}' frame {oframe} reaches the cell edge: move its 'at' inward (or trim its rows); the pet is not shrunk for overrides"
            );
        }
    }
    Hatched { sheet, frames, neutral, issues }
}

fn pet_json(ch: &Character) -> String {
    let s = |v: &str| serde_json::to_string(v).unwrap_or_else(|_| "\"\"".into());
    format!(
        "{{\n  \"id\": {},\n  \"displayName\": {},\n  \"description\": {},\n  \"spriteVersionNumber\": 2,\n  \"spritesheetPath\": \"spritesheet.png\"\n}}\n",
        s(&ch.id),
        s(&ch.display_name),
        s(&ch.description)
    )
}

fn report_json(ch: Option<&Character>, issues: &[Issue]) -> String {
    let errors = issues.iter().filter(|i| i.level == Level::Error).count();
    let rows: Vec<serde_json::Value> = ROWS
        .iter()
        .enumerate()
        .map(|(i, r)| serde_json::json!({"row": i, "name": r.name, "frames": r.frames, "durationsMs": r.durations}))
        .collect();
    let v = serde_json::json!({
        "ok": errors == 0,
        "id": ch.map(|c| c.id.clone()),
        "errors": errors,
        "warnings": issues.len() - errors,
        "issues": issues,
        "rows": rows,
        "neutralCell": {"row": NEUTRAL_CELL.0, "column": NEUTRAL_CELL.1},
    });
    serde_json::to_string_pretty(&v).unwrap_or_default() + "\n"
}

fn issue_line(i: &Issue) -> String {
    let lvl = if i.level == Level::Error { "error" } else { "warning" };
    let at = match (i.row, i.frame) {
        (Some(r), Some(c)) => format!(" [row {r} {}, frame {c}]", ROWS.get(r).map(|x| x.name).unwrap_or("?")),
        (Some(r), None) => format!(" [row {r} {}]", ROWS.get(r).map(|x| x.name).unwrap_or("?")),
        _ => String::new(),
    };
    format!("  {lvl} {}{at}: {}", i.code, i.message)
}

fn print_issues(issues: &[Issue]) {
    const MAX: usize = 25;
    let mut sorted: Vec<&Issue> = issues.iter().collect();
    sorted.sort_by_key(|i| i.level != Level::Error);
    for i in sorted.iter().take(MAX) {
        println!("{}", issue_line(i));
    }
    if sorted.len() > MAX {
        println!("  ... {} more in report.json", sorted.len() - MAX);
    }
}

fn run_spec(spec_path: &Path, out: &Path) -> Result<i32, String> {
    let text = std::fs::read_to_string(spec_path).map_err(|e| format!("could not read {}: {e}", spec_path.display()))?;
    let ch = spec::parse(&text)?;
    std::fs::create_dir_all(out).map_err(|e| format!("could not create {}: {e}", out.display()))?;
    let h = hatch(&ch);
    let ground = (ch.grid.origin().1 * ch.grid.scale as f64) as usize;
    let sheet_path = out.join("spritesheet.png");
    let contact_path = out.join("contact-sheet.png");
    let report_path = out.join("report.json");
    atlas::write_png(&sheet_path, &h.sheet)?;
    atlas::write_png(&contact_path, &atlas::contact_sheet(&h.sheet, &h.issues, Some(ground)))?;
    std::fs::write(out.join("pet.json"), pet_json(&ch)).map_err(|e| format!("could not write pet.json: {e}"))?;
    std::fs::write(&report_path, report_json(Some(&ch), &h.issues)).map_err(|e| format!("could not write report.json: {e}"))?;
    let errors = h.issues.iter().filter(|i| i.level == Level::Error).count();
    let warnings = h.issues.len() - errors;
    let verdict = if errors == 0 { "OK" } else { "NEEDS FIXES" };
    println!("hatch {}: '{}' ({}), {errors} errors, {warnings} warnings", verdict, ch.id, ch.display_name);
    print_issues(&h.issues);
    println!("files:");
    println!("  {}", out.join("pet.json").display());
    println!("  {}", sheet_path.display());
    println!("  {}   <- look at this to review every frame", contact_path.display());
    println!("  {}", report_path.display());
    Ok(if errors == 0 { 0 } else { 2 })
}

fn run_check(path: &Path) -> Result<i32, String> {
    let img = atlas::read_png(path)?;
    let issues = validate::check_atlas(&img);
    let errors = issues.iter().filter(|i| i.level == Level::Error).count();
    println!("check {}: {errors} errors, {} warnings", path.display(), issues.len() - errors);
    print_issues(&issues);
    Ok(if errors == 0 { 0 } else { 2 })
}

/// `bop hatch ...` komut satırı; çıkış kodunu döner.
pub fn cli(args: &[String]) -> i32 {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let result = match a.as_slice() {
        ["--example"] => {
            print!("{}", spec::example("pitir").unwrap_or_default());
            Ok(0)
        }
        ["--example", name] => match spec::example(name) {
            Some(t) => {
                print!("{t}");
                Ok(0)
            }
            None => Err(format!("unknown example '{name}' (use {})", spec::EXAMPLES.join(", "))),
        },
        ["--docs", name] => match DOCS.iter().find(|(n, _)| n == name) {
            Some((_, text)) => {
                print!("{text}");
                Ok(0)
            }
            None => {
                let names: Vec<&str> = DOCS.iter().map(|(n, _)| *n).collect();
                Err(format!("unknown reference '{name}' (use {})", names.join(", ")))
            }
        },
        ["--check", path] => run_check(Path::new(path)),
        [spec_path, out] if !spec_path.starts_with("--") => run_spec(Path::new(spec_path), Path::new(out)),
        _ => {
            println!("{USAGE}");
            return if a.is_empty() || a == ["--help"] { 0 } else { 1 };
        }
    };
    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("bop hatch: {e}");
            1
        }
    }
}

#[cfg(test)]
mod tests;
