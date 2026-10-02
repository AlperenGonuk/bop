//! Deterministic validation. Two layers:
//! - `check_atlas`: any v2 spritesheet (size, filled/empty cells, transparency, clipping,
//!   static rows, the (0,6) neutral, detached pieces, look continuity and baseline).
//! - `check_semantics`: direction checks on the engine's frames (eye tags): look direction and
//!   run direction are measured from coordinates.

use super::atlas::{Image, CELL_H, CELL_W, COLS, SHEET_H, SHEET_W};
use super::motion::{look_angle, LOOK_ROW, NEUTRAL_CELL, ROWS};
use super::rig::{Frame, TAG_BODY, TAG_EYE, TAG_HEAD};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warning,
}

#[derive(Clone, Debug, Serialize)]
pub struct Issue {
    pub level: Level,
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame: Option<usize>,
    pub message: String,
}

fn err(code: &'static str, row: Option<usize>, frame: Option<usize>, message: String) -> Issue {
    Issue { level: Level::Error, code, row, frame, message }
}
fn warn(code: &'static str, row: Option<usize>, frame: Option<usize>, message: String) -> Issue {
    Issue { level: Level::Warning, code, row, frame, message }
}

/// Measurements of one cell.
#[derive(Clone, Debug, Default)]
struct CellStat {
    opaque: usize,
    residue: usize,
    edge: usize,
    /// (x0, y0, x1, y1) inclusive, in cell pixels.
    bbox: Option<(usize, usize, usize, usize)>,
}

fn cell_stat(img: &Image, col: usize, row: usize) -> CellStat {
    let mut s = CellStat::default();
    let (ox, oy) = (col * CELL_W, row * CELL_H);
    for y in 0..CELL_H {
        for x in 0..CELL_W {
            let p = img.get(ox + x, oy + y);
            if p.3 == 0 {
                if p.0 != 0 || p.1 != 0 || p.2 != 0 {
                    s.residue += 1;
                }
                continue;
            }
            s.opaque += 1;
            if x < 2 || y < 2 || x >= CELL_W - 2 || y >= CELL_H - 2 {
                s.edge += 1;
            }
            s.bbox = Some(match s.bbox {
                None => (x, y, x, y),
                Some(b) => (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y)),
            });
        }
    }
    s
}

fn cell_bytes(img: &Image, col: usize, row: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(CELL_W * CELL_H * 4);
    for y in 0..CELL_H {
        let i = ((row * CELL_H + y) * img.w + col * CELL_W) * 4;
        v.extend_from_slice(&img.data[i..i + CELL_W * 4]);
    }
    v
}

/// Number of opaque 8-connected components (small specks included).
fn components(img: &Image, col: usize, row: usize) -> usize {
    let (ox, oy) = (col * CELL_W, row * CELL_H);
    let mut seen = vec![false; CELL_W * CELL_H];
    let mut count = 0;
    let mut stack = Vec::new();
    for start in 0..CELL_W * CELL_H {
        if seen[start] || img.get(ox + start % CELL_W, oy + start / CELL_W).3 == 0 {
            continue;
        }
        count += 1;
        seen[start] = true;
        stack.push(start);
        while let Some(i) = stack.pop() {
            let (x, y) = ((i % CELL_W) as i64, (i / CELL_W) as i64);
            for dy in -1..=1 {
                for dx in -1..=1 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= CELL_W as i64 || ny >= CELL_H as i64 {
                        continue;
                    }
                    let j = ny as usize * CELL_W + nx as usize;
                    if !seen[j] && img.get(ox + nx as usize, oy + ny as usize).3 != 0 {
                        seen[j] = true;
                        stack.push(j);
                    }
                }
            }
        }
    }
    count
}

fn expected_used(row: usize, col: usize) -> bool {
    col < ROWS[row].frames || (row, col) == NEUTRAL_CELL
}

/// Image-level validation (also works on spritesheets not made by the engine).
pub fn check_atlas(img: &Image) -> Vec<Issue> {
    let mut out = Vec::new();
    if (img.w, img.h) != (SHEET_W, SHEET_H) {
        out.push(err(
            "size",
            None,
            None,
            format!("spritesheet is {}x{}; a v2 atlas must be {SHEET_W}x{SHEET_H} (8 x 11 cells of {CELL_W}x{CELL_H})", img.w, img.h),
        ));
        return out;
    }
    let mut stats = vec![vec![CellStat::default(); COLS]; ROWS.len()];
    for (r, row) in ROWS.iter().enumerate() {
        for c in 0..COLS {
            let s = cell_stat(img, c, r);
            let used = expected_used(r, c);
            if s.residue > 0 {
                out.push(err("rgb-residue", Some(r), Some(c), format!("{} transparent pixels have non-zero RGB", s.residue)));
            }
            if used {
                if (r, c) == NEUTRAL_CELL && s.opaque < 50 {
                    out.push(err("neutral-empty", Some(r), Some(c), "cell (0,6) is the neutral front pose and must not be empty".into()));
                } else if s.opaque < 400 {
                    out.push(err("frame-empty", Some(r), Some(c), format!("{} frame {c} has only {} visible pixels", row.name, s.opaque)));
                }
                if s.opaque * 100 > CELL_W * CELL_H * 95 {
                    out.push(err("opaque-cell", Some(r), Some(c), "cell is more than 95% opaque (background left in?)".into()));
                }
                if s.edge > 0 {
                    out.push(err(
                        "clipped",
                        Some(r),
                        Some(c),
                        format!("{} frame {c} touches the cell edge ({} px): part of the pet is cut off; make it smaller or lower the motion", row.name, s.edge),
                    ));
                }
            } else if s.opaque > 0 {
                out.push(err("unused-not-empty", Some(r), Some(c), format!("cell ({r},{c}) must be fully transparent ({} px)", s.opaque)));
            }
            stats[r][c] = s;
        }
        // Static row: if every frame is identical, there is no motion.
        if row.frames > 1 && r < LOOK_ROW {
            let first = cell_bytes(img, 0, r);
            if (1..row.frames).all(|c| cell_bytes(img, c, r) == first) {
                out.push(err("static-row", Some(r), None, format!("all {} frames of '{}' are identical", row.frames, row.name)));
            }
        }
        // Detached pieces (effects should touch the pet).
        for c in 0..row.frames {
            let n = components(img, c, r);
            if n > 1 {
                out.push(warn("detached", Some(r), Some(c), format!("{} frame {c} has {n} separate pieces; effects and parts should touch the pet", row.name)));
            }
        }
    }
    // Look: differs from neutral, baseline fixed, no jumps between consecutive directions.
    let neutral = cell_bytes(img, NEUTRAL_CELL.1, NEUTRAL_CELL.0);
    let nstat = &stats[NEUTRAL_CELL.0][NEUTRAL_CELL.1];
    let look_cell = |i: usize| (LOOK_ROW + i / 8, i % 8);
    for i in 0..16 {
        let (r, c) = look_cell(i);
        let deg = look_angle(i);
        if cell_bytes(img, c, r) == neutral {
            out.push(err("look-same-as-neutral", Some(r), Some(c), format!("look {deg} is identical to the neutral pose")));
        }
        if let (Some(a), Some(b)) = (stats[r][c].bbox, nstat.bbox) {
            if (a.3 as i64 - b.3 as i64).abs() > 4 {
                out.push(warn("look-baseline", Some(r), Some(c), format!("look {deg}: bottom moved {} px from neutral; feet should stay planted", a.3 as i64 - b.3 as i64)));
            }
        }
        let (r2, c2) = look_cell((i + 1) % 16);
        let (s1, s2) = (&stats[r][c], &stats[r2][c2]);
        if s1.opaque > 0 && s2.opaque > 0 {
            let ratio = s1.opaque.max(s2.opaque) as f64 / s1.opaque.min(s2.opaque) as f64;
            if ratio > 1.15 {
                out.push(warn("look-size-jump", Some(r2), Some(c2), format!("look {} -> {}: visible area changes by {:.0}%", deg, look_angle((i + 1) % 16), (ratio - 1.0) * 100.0)));
            }
            if let (Some(a), Some(b)) = (s1.bbox, s2.bbox) {
                let (ax, ay) = ((a.0 + a.2) as f64 / 2.0, (a.1 + a.3) as f64 / 2.0);
                let (bx, by) = ((b.0 + b.2) as f64 / 2.0, (b.1 + b.3) as f64 / 2.0);
                let d = ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt();
                if d > 8.0 {
                    out.push(warn("look-jump", Some(r2), Some(c2), format!("look {} -> {}: pet center jumps {d:.0} px", deg, look_angle((i + 1) % 16))));
                }
            }
        }
    }
    out
}

/// Centroid of the tagged pixels (logical pixels).
fn centroid(f: &Frame, tags: &[u8]) -> Option<(f64, f64)> {
    let (mut sx, mut sy, mut n) = (0.0, 0.0, 0.0);
    for y in 0..f.h {
        for x in 0..f.w {
            if tags.contains(&f.tag[y * f.w + x]) {
                sx += x as f64 + 0.5;
                sy += y as f64 + 0.5;
                n += 1.0;
            }
        }
    }
    (n > 0.0).then(|| (sx / n, sy / n))
}

/// Direction semantics: do the eyes really move that way in the look frames, and does the pet
/// face the right way when running. `frames[row][col]` are the engine's logical frames, `neutral` is the (0,6) frame.
pub fn check_semantics(frames: &[Vec<Frame>], neutral: &Frame) -> Vec<Issue> {
    let mut out = Vec::new();
    let Some(n_eye) = centroid(neutral, &[TAG_EYE]) else {
        out.push(warn("eyes-unmeasured", Some(NEUTRAL_CELL.0), Some(NEUTRAL_CELL.1), "no eyes found in the neutral pose; look directions were not measured".into()));
        return out;
    };
    let n_body = centroid(neutral, &[TAG_BODY, TAG_HEAD]);
    for i in 0..16 {
        let (r, c) = (LOOK_ROW + i / 8, i % 8);
        let deg = look_angle(i);
        let Some(e) = centroid(&frames[r][c], &[TAG_EYE]) else {
            out.push(warn("eyes-unmeasured", Some(r), Some(c), format!("look {deg}: no eyes found; direction not measured")));
            continue;
        };
        let a = deg.to_radians();
        let (ex, ey) = (a.sin(), a.cos());
        let (dx, dy) = (e.0 - n_eye.0, -(e.1 - n_eye.1));
        if i % 4 == 0 {
            // Cardinal directions: at least 1 logical pixel on the main axis, correct sign, dominant.
            let (main, cross) = if ex.abs() > 0.5 { (dx * ex.signum(), dy) } else { (dy * ey.signum(), dx) };
            if main < 1.0 || cross.abs() > main {
                out.push(err(
                    "look-direction",
                    Some(r),
                    Some(c),
                    format!("look {deg}: eyes moved ({dx:+.1}, {dy:+.1}) px (right, up); expected a clear move toward {deg} degrees (0 = up, 90 = right, 180 = down)"),
                ));
            }
        } else {
            let bad_x = ex.abs() > 0.3 && dx * ex <= 0.0;
            let bad_y = ey.abs() > 0.3 && dy * ey <= 0.0;
            if bad_x || bad_y {
                out.push(warn("look-direction", Some(r), Some(c), format!("look {deg}: eyes moved ({dx:+.1}, {dy:+.1}) px (right, up); not in the expected quadrant")));
            }
        }
        // Hierarchy: the eyes should move more than the body.
        if i % 8 == 4 {
            if let (Some(nb), Some(b)) = (n_body, centroid(&frames[r][c], &[TAG_BODY, TAG_HEAD])) {
                let bx = b.0 - nb.0;
                if bx.abs() > dx.abs() {
                    out.push(warn("look-hierarchy", Some(r), Some(c), format!("look {deg}: body moved more than the eyes ({bx:+.1} vs {dx:+.1} px)")));
                }
            }
        }
    }
    // Run direction.
    for (row, sign) in [(1usize, 1.0f64), (2, -1.0)] {
        for (c, f) in frames[row].iter().enumerate() {
            match centroid(f, &[TAG_EYE]) {
                Some(e) if (e.0 - n_eye.0) * sign >= 1.0 => {}
                Some(e) => out.push(err(
                    "run-direction",
                    Some(row),
                    Some(c),
                    format!("{} frame {c}: eyes moved {:+.1} px sideways; the pet should face {}", ROWS[row].name, e.0 - n_eye.0, if sign > 0.0 { "right" } else { "left" }),
                )),
                None => out.push(warn("eyes-unmeasured", Some(row), Some(c), format!("{} frame {c}: no eyes found", ROWS[row].name))),
            }
        }
    }
    out
}

/// Auto-shrink notice.
pub fn fit_warning(scale: f64, fits: bool) -> Issue {
    let pct = (scale * 100.0).round();
    if fits {
        warn("auto-fit", None, None, format!("the pet did not fit its cell and was drawn at {pct}% size; make it smaller in the spec to control this yourself"))
    } else {
        warn("auto-fit", None, None, format!("the pet does not fit its cell even at {pct}% size; make it smaller or reduce motion"))
    }
}
