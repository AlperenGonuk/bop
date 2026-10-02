//! `bop hatch`: draws a pet in code, without an API (DECISIONS.md, decisions 22, 27).
//!
//! `bop hatch <spec.json> <output-folder>` → `pet.json`, `spritesheet.png` (v2 atlas),
//! `contact-sheet.png` (labeled, for QA), `report.json`. Opens no window.
//! Exit code: 0 clean, 2 validation errors (files are still written), 1 spec/input error.

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

/// The skill's references are embedded in the exe: Claude reads them with the one command the
/// skill allows (`bop hatch --docs`), without asking to read the plugin folder. Source: the plugin's files.
const DOCS: &[(&str, &str)] = &[
    ("spec-format", include_str!("../../../../plugin/skills/hatch/references/spec-format.md")),
    ("qa-rubric", include_str!("../../../../plugin/skills/hatch/references/qa-rubric.md")),
    ("animation-rows", include_str!("../../../../plugin/skills/hatch/references/animation-rows.md")),
];

/// Drawing result: atlas, logical frames and issues (tests read the frames).
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

/// Draws every frame of the character, builds the atlas and validates it. If the pet (body or
/// parts) overflows the cell, the character is shrunk step by step around the ground center (down
/// to 80% at most), reported as a warning. If only an override overflows, nothing is shrunk: the
/// error says to move the override.
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

/// A single attempt without shrinking (tests use this to see overflow).
#[cfg(test)]
fn hatch_once(ch: &Character) -> Hatched {
    finish(ch, &base_frames(ch))
}

/// Whether the frame touches the cell edge (the logical grid covers the cell exactly: an edge pixel =
/// the cell's 2 edge pixels, same as the "clipped" check in `validate::check_atlas`).
fn touches_edge(f: &Frame) -> bool {
    (0..f.h).any(|y| (0..f.w).any(|x| (x == 0 || y == 0 || x + 1 == f.w || y + 1 == f.h) && !f.px[y * f.w + x].is_clear()))
}

fn any_clipped(frames: &[Vec<Frame>]) -> bool {
    frames.iter().flatten().any(touches_edge)
}

/// The rows' frames without overrides (jump height is chosen to fit the cell).
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

/// Jump height: of the candidates descending from 9 in 0.5 steps, the highest whose frames all stay
/// at least k pixels below the top of the cell (1 if none fits). The top rises as height grows, so a
/// binary search is enough. Also returns the frames for the chosen height (so they aren't redrawn).
fn jump_frames(ch: &Character) -> (f64, Vec<Frame>) {
    let k = ch.grid.k();
    let top_of = |f: &Frame| (0..f.h).find(|&y| (0..f.w).any(|x| !f.px[y * f.w + x].is_clear())).unwrap_or(0);
    let draw = |j: f64| -> Vec<Frame> { motion::row_poses(ch, JUMP_ROW, j).iter().map(|p| rig::render(ch, p)).collect() };
    let fits = |fs: &[Frame]| fs.iter().all(|f| top_of(f) as f64 >= k);
    // Candidates: 9, 8.5, ..., 1.5; the last one (1) is accepted unchecked.
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
    // `best` always holds the frames of the last attempt that fit (`hi`, which is `lo` after the loop).
    let frames = best.unwrap_or_else(|| draw(jump));
    (jump, frames)
}

/// Applies overrides, builds the atlas and validates it. `base` is the frames without overrides.
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
    // If only an override touches the edge, shrinking won't help: the message points at the override.
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

/// The `bop hatch ...` command line; returns the exit code.
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
