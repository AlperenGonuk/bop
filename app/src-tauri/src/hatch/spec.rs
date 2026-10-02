//! `spec.json`: the character definition. Three levels in the same file:
//! (a) archetype + palette + options, (b) parts (add, or change by name),
//! (c) raw pixel maps (as a part, or as `overrides` for a specific frame).
//! Error messages are in English: Claude reads them and fixes the spec.

use super::color::{Rgba, Tone};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Logical grid: divides the 192×208 cell evenly.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grid {
    pub w: usize,
    pub h: usize,
    /// Size of a logical pixel in the cell (4 or 2).
    pub scale: usize,
}

impl Grid {
    /// Size factor relative to the 48×52 grid (archetype measurements are written for 48×52).
    pub fn k(&self) -> f64 {
        self.w as f64 / 48.0
    }
    /// Position of the root coordinate (ground, horizontal center) on the logical grid.
    pub fn origin(&self) -> (f64, f64) {
        (self.w as f64 / 2.0, self.h as f64 - 4.5 * self.k())
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GroupId {
    Root,
    #[default]
    Body,
    Head,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ShapeKind {
    Ellipse,
    Rect,
    Triangle,
    Polygon,
    Capsule,
    Pixels,
    Limb,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Shading {
    Ball,
    Band,
    Flat,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Body,
    Head,
    Ear,
    Tail,
    Topper,
    Arm,
    Foot,
    Marking,
    #[default]
    Accessory,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Show {
    Always,
    Gesture,
    Never,
}

/// Level (b): a single part. Coordinates are logical pixels, relative to the center of the `on` group
/// (for `root`, relative to the ground center), y positive downward.
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PartSpec {
    pub name: String,
    #[serde(default)]
    pub on: GroupId,
    pub shape: ShapeKind,
    #[serde(default)]
    pub role: Role,
    /// Attachment/pivot point.
    #[serde(default)]
    pub at: [f64; 2],
    /// The shape's center, relative to the attachment point (rotates with the part).
    #[serde(default)]
    pub offset: [f64; 2],
    /// Depth (+ toward the viewer). If not given, `surface` or 0.
    #[serde(default)]
    pub z: Option<f64>,
    /// Stuck to the front surface of the group's main shape (like face markings): z is automatic.
    #[serde(default)]
    pub surface: bool,
    #[serde(default)]
    pub size: Option<[f64; 2]>,
    #[serde(default)]
    pub rot: f64,
    #[serde(default)]
    pub points: Vec<[f64; 2]>,
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub radius: f64,
    #[serde(default)]
    pub rows: Vec<String>,
    #[serde(default)]
    pub key: BTreeMap<String, String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub shading: Option<Shading>,
    #[serde(default)]
    pub outline: Option<bool>,
    #[serde(default)]
    pub mirror: bool,
    #[serde(default)]
    pub sway: Option<f64>,
    #[serde(default)]
    pub droop: Option<f64>,
    #[serde(default)]
    pub clip: bool,
    #[serde(default)]
    pub join: Option<String>,
    #[serde(default)]
    pub limb: Option<[f64; 2]>,
    #[serde(default)]
    pub hand: Option<f64>,
    #[serde(default)]
    pub show: Option<Show>,
    #[serde(default)]
    pub gloss: bool,
    #[serde(default)]
    pub depth: Option<f64>,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EyeStyle {
    Dot,
    Round,
    Bean,
    Sparkle,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MouthStyle {
    Smile,
    Curve,
    Cat,
    None,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FaceSpec {
    pub eyes: Option<EyeStyle>,
    /// Distance between eye centers.
    pub gap: Option<f64>,
    /// Eye height, relative to the face group's center.
    pub y: Option<f64>,
    pub mouth: Option<MouthStyle>,
    /// How far below the eyes the mouth sits.
    pub mouth_drop: Option<f64>,
    pub cheeks: Option<bool>,
    pub brows: Option<bool>,
    pub color: Option<String>,
}

#[derive(Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Options {
    pub size: Option<f64>,
    pub body: Option<String>,
    pub ears: Option<String>,
    pub tail: Option<String>,
    pub topper: Option<String>,
    pub arms: Option<String>,
    pub feet: Option<bool>,
    pub belly: Option<bool>,
    pub accessory: Option<String>,
    pub pattern: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MotionSpec {
    #[serde(default = "one")]
    pub bounce: f64,
    #[serde(default = "one")]
    pub sway: f64,
    /// Forward lean when running (degrees).
    #[serde(default = "lean_default")]
    pub lean: f64,
    /// Upper limit of sideways turn (degrees); when looking, the head turns 0.6 of it, the body 0.25.
    #[serde(default = "turn_default")]
    pub turn: f64,
    /// Upper limit of head tilt when looking up/down (degrees).
    #[serde(default = "pitch_default")]
    pub pitch: f64,
}

fn one() -> f64 {
    1.0
}
fn lean_default() -> f64 {
    10.0
}
fn turn_default() -> f64 {
    50.0
}
fn pitch_default() -> f64 {
    25.0
}

impl Default for MotionSpec {
    fn default() -> Self {
        MotionSpec { bounce: 1.0, sway: 1.0, lean: 10.0, turn: 50.0, pitch: 25.0 }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Props {
    #[serde(default = "yes")]
    pub laptop: bool,
}

fn yes() -> bool {
    true
}

impl Default for Props {
    fn default() -> Self {
        Props { laptop: true }
    }
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OverrideMode {
    #[default]
    Over,
    Replace,
}

/// Level (c): a raw pixel map drawn over (or instead of) a specific frame.
#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Override {
    pub row: String,
    pub frame: usize,
    #[serde(default)]
    pub mode: OverrideMode,
    /// Top-left corner of the map, on the logical grid.
    #[serde(default)]
    pub at: [i32; 2],
    pub rows: Vec<String>,
    pub key: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Spec {
    #[serde(default = "format_default")]
    pub format_version: u32,
    pub id: String,
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "archetype_default")]
    pub archetype: String,
    #[serde(default)]
    pub grid: Option<String>,
    #[serde(default)]
    pub palette: BTreeMap<String, Value>,
    #[serde(default)]
    pub options: Options,
    #[serde(default)]
    pub face: FaceSpec,
    #[serde(default)]
    pub parts: Vec<Value>,
    #[serde(default)]
    pub remove: Vec<String>,
    #[serde(default)]
    pub motion: MotionSpec,
    #[serde(default)]
    pub props: Props,
    #[serde(default)]
    pub overrides: Vec<Override>,
}

fn format_default() -> u32 {
    1
}
fn archetype_default() -> String {
    "blob".into()
}

pub const ARCHETYPES: &[&str] = &["blob", "critter", "floaty", "custom"];

/// Resolved face settings.
#[derive(Debug, Clone)]
pub struct Face {
    pub eyes: EyeStyle,
    pub gap: f64,
    pub y: f64,
    pub mouth: MouthStyle,
    pub mouth_drop: f64,
    pub cheeks: bool,
    pub brows: bool,
    pub eye_color: Rgba,
}

/// A character ready to draw: palette, parts, face, motion settings.
#[derive(Debug, Clone)]
pub struct Character {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub grid: Grid,
    pub palette: BTreeMap<String, Tone>,
    pub parts: Vec<PartSpec>,
    pub face: Face,
    pub motion: MotionSpec,
    pub laptop: bool,
    /// Footless, floating character (floaty).
    pub floats: bool,
    pub overrides: Vec<Override>,
}

impl Character {
    /// `body`, `body.shade`, `body.hi` or `#RRGGBB`.
    pub fn tone(&self, r: &str) -> Result<Tone, String> {
        resolve_tone(&self.palette, r)
    }
    pub fn color(&self, r: &str) -> Result<Rgba, String> {
        resolve_tone(&self.palette, r).map(|t| t.base)
    }

    /// Shrinks the whole character by factor `s` around the ground center (to fit the cell).
    /// Pixel maps and face patterns keep their pixel size; only their positions change.
    pub fn shrunk(&self, s: f64) -> Character {
        let mut c = self.clone();
        let m2 = |v: &mut [f64; 2]| {
            v[0] *= s;
            v[1] *= s;
        };
        for p in &mut c.parts {
            m2(&mut p.at);
            m2(&mut p.offset);
            if let Some(z) = &mut p.z {
                *z *= s;
            }
            if let Some(v) = &mut p.size {
                m2(v);
            }
            for pt in &mut p.points {
                m2(pt);
            }
            for v in [&mut p.width, &mut p.hand, &mut p.depth].into_iter().flatten() {
                *v *= s;
            }
            if let Some(v) = &mut p.limb {
                m2(v);
            }
            p.radius *= s;
        }
        c.face.gap *= s;
        c.face.y *= s;
        c.face.mouth_drop *= s;
        // Frame overrides move with the pet: the map's center moves toward the ground center
        // by factor `s`; the map itself (pixel size) does not change.
        let (ox, oy) = self.grid.origin();
        for o in &mut c.overrides {
            let w = o.rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as f64;
            let h = o.rows.len() as f64;
            let (cx, cy) = (o.at[0] as f64 + w / 2.0, o.at[1] as f64 + h / 2.0);
            let (nx, ny) = (ox + (cx - ox) * s, oy + (cy - oy) * s);
            o.at = [(nx - w / 2.0).round() as i32, (ny - h / 2.0).round() as i32];
        }
        c
    }

    /// Color of a pixel map character; `None` for transparent or unresolvable characters.
    pub fn pixel(&self, c: char, key: &BTreeMap<String, String>) -> Option<Rgba> {
        pixel_color(c, key, &self.palette).ok().flatten()
    }
}

/// A character in a pixel map: `.` and space are transparent (`None`), the others take a palette
/// color via `key`. Spec checking, part drawing and frame overrides all use the same rule.
pub fn pixel_color(c: char, key: &BTreeMap<String, String>, palette: &BTreeMap<String, Tone>) -> Result<Option<Rgba>, String> {
    if c == '.' || c == ' ' {
        return Ok(None);
    }
    let r = key.get(&c.to_string()).ok_or_else(|| format!("character '{c}' is not in 'key'"))?;
    resolve_tone(palette, r).map(|t| Some(t.base))
}

pub fn resolve_tone(palette: &BTreeMap<String, Tone>, r: &str) -> Result<Tone, String> {
    if r.starts_with('#') {
        return Rgba::parse_hex(r).map(Tone::auto).ok_or_else(|| format!("bad color '{r}' (use #RRGGBB)"));
    }
    let (name, part) = match r.split_once('.') {
        Some((n, p)) => (n, Some(p)),
        None => (r, None),
    };
    let t = palette.get(name).ok_or_else(|| {
        let names: Vec<&str> = palette.keys().map(String::as_str).collect();
        format!("unknown palette color '{name}' (known: {})", names.join(", "))
    })?;
    match part {
        None => Ok(*t),
        Some("shade") => Ok(Tone::flat(t.shade)),
        Some("hi") => Ok(Tone::flat(t.hi)),
        Some("base") => Ok(Tone::flat(t.base)),
        Some(p) => Err(format!("bad color '{r}': '.{p}' must be .shade, .hi or .base")),
    }
}

/// Parses spec text and builds the character.
pub fn parse(text: &str) -> Result<Character, String> {
    let spec: Spec = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|e| format!("spec.json: {e}"))?;
    build(spec)
}

fn parse_grid(g: Option<&str>) -> Result<Grid, String> {
    match g.unwrap_or("48x52") {
        "48x52" => Ok(Grid { w: 48, h: 52, scale: 4 }),
        "96x104" => Ok(Grid { w: 96, h: 104, scale: 2 }),
        other => Err(format!("grid '{other}' is not supported (use \"48x52\" or \"96x104\")")),
    }
}

fn palette_entry(name: &str, v: &Value) -> Result<Tone, String> {
    let hex = |v: &Value, what: &str| -> Result<Rgba, String> {
        v.as_str()
            .and_then(Rgba::parse_hex)
            .ok_or_else(|| format!("palette.{name}{what}: expected \"#RRGGBB\""))
    };
    match v {
        Value::String(_) => Ok(Tone::auto(hex(v, "")?)),
        Value::Object(o) => {
            for k in o.keys() {
                if !["base", "shade", "hi"].contains(&k.as_str()) {
                    return Err(format!("palette.{name}: unknown field '{k}' (base, shade, hi)"));
                }
            }
            let base = hex(o.get("base").ok_or(format!("palette.{name}: 'base' is missing"))?, ".base")?;
            let mut t = Tone::auto(base);
            if let Some(s) = o.get("shade").filter(|s| s.as_str() != Some("auto")) {
                t.shade = hex(s, ".shade")?;
            }
            if let Some(h) = o.get("hi").filter(|s| s.as_str() != Some("auto")) {
                t.hi = hex(h, ".hi")?;
            }
            Ok(t)
        }
        _ => Err(format!("palette.{name}: expected \"#RRGGBB\" or {{\"base\": ...}}")),
    }
}

fn default_body(archetype: &str) -> Rgba {
    match archetype {
        "critter" => Rgba::rgb(0xE9, 0xA1, 0x5B),
        "floaty" => Rgba::rgb(0xEE, 0xF0, 0xFF),
        _ => Rgba::rgb(0xFF, 0xB0, 0x66),
    }
}

fn build_palette(spec: &Spec) -> Result<BTreeMap<String, Tone>, String> {
    let mut user = BTreeMap::new();
    for (name, v) in &spec.palette {
        if name.is_empty() || name.contains('.') || name.starts_with('#') {
            return Err(format!("palette name '{name}' is not valid"));
        }
        user.insert(name.clone(), palette_entry(name, v)?);
    }
    let rgb = Rgba::rgb;
    let body = user.get("body").copied().unwrap_or(Tone::auto(default_body(&spec.archetype)));
    let mut p: BTreeMap<String, Tone> = BTreeMap::new();
    p.insert("body".into(), body);
    p.insert("outline".into(), Tone::flat(rgb(0x2B, 0x21, 0x40)));
    p.insert("eye".into(), user.get("outline").copied().unwrap_or(Tone::flat(rgb(0x2B, 0x21, 0x40))));
    p.insert("white".into(), Tone::flat(rgb(0xFF, 0xFF, 0xFF)));
    p.insert("accent".into(), Tone::auto(rgb(0x8B, 0xD1, 0x4A)));
    p.insert("foot".into(), Tone::auto(body.shade.auto_shade()));
    p.insert("cheek".into(), Tone::flat(rgb(0xFF, 0x6F, 0x85)));
    p.insert("tongue".into(), Tone::flat(rgb(0xFF, 0x7A, 0x8E)));
    p.insert("sweat".into(), Tone::flat(rgb(0x8F, 0xD8, 0xFF)));
    p.insert("belly".into(), Tone::auto(body.hi));
    p.insert("inner".into(), Tone::auto(rgb(0xF7, 0xA1, 0xB0)));
    p.insert("pattern".into(), Tone::auto(body.shade));
    p.insert("laptop".into(), Tone { base: rgb(0xA9, 0xB1, 0xC4), shade: rgb(0x76, 0x7E, 0x93), hi: rgb(0xD5, 0xDA, 0xE6) });
    for (k, v) in user {
        p.insert(k, v);
    }
    Ok(p)
}

/// Merges parts by name: if the name exists its fields are overwritten, otherwise the part is added.
fn merge_parts(base: Vec<Value>, extra: &[Value], remove: &[String]) -> Result<Vec<Value>, String> {
    let mut out = base;
    for (i, p) in extra.iter().enumerate() {
        let Some(obj) = p.as_object() else {
            return Err(format!("parts[{i}]: expected an object"));
        };
        let Some(name) = obj.get("name").and_then(Value::as_str) else {
            return Err(format!("parts[{i}]: 'name' is missing"));
        };
        if let Some(existing) = out.iter_mut().find(|e| e.get("name").and_then(Value::as_str) == Some(name)) {
            let target = existing.as_object_mut().expect("part is an object");
            for (k, v) in obj {
                if v.is_null() {
                    target.remove(k);
                } else {
                    target.insert(k.clone(), v.clone());
                }
            }
        } else {
            out.push(p.clone());
        }
    }
    for r in remove {
        let before = out.len();
        out.retain(|e| e.get("name").and_then(Value::as_str) != Some(r.as_str()));
        if out.len() == before {
            return Err(format!("remove: no part named '{r}'"));
        }
    }
    Ok(out)
}

/// The twin of a mirrored part (reflected across the x axis).
fn mirrored(p: &PartSpec, all: &[PartSpec]) -> PartSpec {
    let mut m = p.clone();
    m.name = format!("{}~mirror", p.name);
    m.mirror = false;
    m.at[0] = -m.at[0];
    m.offset[0] = -m.offset[0];
    m.rot = -m.rot;
    for pt in &mut m.points {
        pt[0] = -pt[0];
    }
    m.rows = m.rows.iter().map(|r| r.chars().rev().collect()).collect();
    if let Some(j) = &p.join {
        if all.iter().any(|o| &o.name == j && o.mirror) {
            m.join = Some(format!("{j}~mirror"));
        }
    }
    m
}

fn check_part(p: &PartSpec, palette: &BTreeMap<String, Tone>) -> Result<(), String> {
    let n = &p.name;
    let need_size = || p.size.filter(|s| s[0] > 0.0 && s[1] > 0.0).ok_or(format!("part '{n}': 'size' [w, h] (> 0) is required"));
    match p.shape {
        ShapeKind::Ellipse | ShapeKind::Rect => {
            need_size()?;
        }
        ShapeKind::Triangle if p.points.len() != 3 => {
            return Err(format!("part '{n}': a triangle needs exactly 3 'points'"))
        }
        ShapeKind::Polygon if p.points.len() < 3 => {
            return Err(format!("part '{n}': a polygon needs at least 3 'points'"))
        }
        ShapeKind::Capsule if p.points.len() < 2 || p.width.is_none() => {
            return Err(format!("part '{n}': a capsule needs 'points' (2 or more) and 'width'"))
        }
        ShapeKind::Pixels => {
            if p.rows.is_empty() {
                return Err(format!("part '{n}': 'rows' is empty"));
            }
            check_pixel_map(&p.rows, &p.key, palette).map_err(|e| format!("part '{n}': {e}"))?;
        }
        ShapeKind::Limb if p.limb.is_none() || p.width.is_none() => {
            return Err(format!("part '{n}': a limb needs 'limb' [upper, lower] and 'width'"))
        }
        _ => {}
    }
    // Zero or negative width/depth causes division by zero and empty shapes when drawing.
    let positive = |v: Option<f64>| v.is_none_or(|x| x.is_finite() && x > 0.0);
    if !positive(p.depth) {
        return Err(format!("part '{n}': 'depth' must be a number > 0"));
    }
    if matches!(p.shape, ShapeKind::Capsule | ShapeKind::Limb) && !positive(p.width) {
        return Err(format!("part '{n}': 'width' must be a number > 0"));
    }
    if p.role == Role::Arm && p.shape != ShapeKind::Limb {
        return Err(format!("part '{n}': role 'arm' needs shape 'limb'"));
    }
    if matches!(p.role, Role::Body | Role::Head) && p.shape != ShapeKind::Ellipse {
        return Err(format!("part '{n}': the main body/head part must be an ellipse"));
    }
    if p.role == Role::Body && p.on != GroupId::Root {
        return Err(format!("part '{n}': the body part must be \"on\": \"root\""));
    }
    if p.role == Role::Head && p.on != GroupId::Body {
        return Err(format!("part '{n}': the head part must be \"on\": \"body\""));
    }
    if let Some(c) = &p.color {
        resolve_tone(palette, c).map_err(|e| format!("part '{n}': {e}"))?;
    }
    Ok(())
}

pub fn check_pixel_map(rows: &[String], key: &BTreeMap<String, String>, palette: &BTreeMap<String, Tone>) -> Result<(), String> {
    for (k, c) in key {
        if k.chars().count() != 1 {
            return Err(format!("key '{k}': keys must be single characters"));
        }
        if k == "." || k == " " {
            return Err("key: '.' and ' ' are reserved for transparent".into());
        }
        resolve_tone(palette, c)?;
    }
    for (y, r) in rows.iter().enumerate() {
        for ch in r.chars() {
            pixel_color(ch, key, palette).map_err(|e| format!("rows[{y}]: {e}"))?;
        }
    }
    Ok(())
}

fn build(spec: Spec) -> Result<Character, String> {
    if spec.format_version != 1 {
        return Err(format!("formatVersion {} is not supported (use 1)", spec.format_version));
    }
    if !crate::pets::valid_id(&spec.id) {
        return Err(format!("id '{}' is not valid: use 1-64 lowercase letters, digits, '-' or '_'", spec.id));
    }
    if spec.display_name.trim().is_empty() {
        return Err("displayName is empty".into());
    }
    if !ARCHETYPES.contains(&spec.archetype.as_str()) {
        return Err(format!("archetype '{}' is unknown (use {})", spec.archetype, ARCHETYPES.join(", ")));
    }
    let grid = parse_grid(spec.grid.as_deref())?;
    let palette = build_palette(&spec)?;
    let (base_parts, face_default) = archetype_parts(&spec, grid)?;
    let merged = merge_parts(base_parts, &spec.parts, &spec.remove)?;

    let mut parts: Vec<PartSpec> = Vec::new();
    for v in merged {
        let name = v.get("name").and_then(Value::as_str).unwrap_or("?").to_string();
        let p: PartSpec = serde_json::from_value(v).map_err(|e| format!("part '{name}': {e}"))?;
        if parts.iter().any(|o| o.name == p.name) {
            return Err(format!("part '{}' is defined twice", p.name));
        }
        check_part(&p, &palette)?;
        parts.push(p);
    }
    let mirrors: Vec<PartSpec> = parts.iter().filter(|p| p.mirror).map(|p| mirrored(p, &parts)).collect();
    parts.extend(mirrors);
    for p in &parts {
        if let Some(j) = &p.join {
            if !parts.iter().any(|o| &o.name == j && o.join.is_none()) {
                return Err(format!("part '{}': join target '{j}' does not exist (or is itself joined)", p.name));
            }
        }
    }
    let bodies = parts.iter().filter(|p| p.role == Role::Body).count();
    if bodies != 1 {
        return Err(format!("exactly one part needs \"role\": \"body\" (found {bodies})"));
    }
    if parts.iter().filter(|p| p.role == Role::Head).count() > 1 {
        return Err("at most one part can have \"role\": \"head\"".into());
    }

    let f = &spec.face;
    let eye_color = match &f.color {
        Some(c) => resolve_tone(&palette, c)?.base,
        None => palette["eye"].base,
    };
    let face = Face {
        eyes: f.eyes.unwrap_or(face_default.eyes),
        gap: f.gap.unwrap_or(face_default.gap),
        y: f.y.unwrap_or(face_default.y),
        mouth: f.mouth.unwrap_or(face_default.mouth),
        mouth_drop: f.mouth_drop.unwrap_or(face_default.mouth_drop),
        cheeks: f.cheeks.unwrap_or(face_default.cheeks),
        brows: f.brows.unwrap_or(face_default.brows),
        eye_color,
    };

    let row_names = super::motion::ROWS.iter().map(|r| r.name).collect::<Vec<_>>();
    for (i, o) in spec.overrides.iter().enumerate() {
        let Some(row) = super::motion::ROWS.iter().find(|r| r.name == o.row) else {
            return Err(format!("overrides[{i}]: row '{}' is unknown (rows: {})", o.row, row_names.join(", ")));
        };
        if o.frame >= row.frames {
            return Err(format!("overrides[{i}]: row '{}' has frames 0-{}", o.row, row.frames - 1));
        }
        check_pixel_map(&o.rows, &o.key, &palette).map_err(|e| format!("overrides[{i}]: {e}"))?;
    }
    for (what, v, lo, hi) in [
        ("motion.bounce", spec.motion.bounce, 0.0, 2.0),
        ("motion.sway", spec.motion.sway, 0.0, 2.0),
        ("motion.lean", spec.motion.lean, 0.0, 25.0),
        ("motion.turn", spec.motion.turn, 0.0, 70.0),
        ("motion.pitch", spec.motion.pitch, 0.0, 40.0),
    ] {
        if !(lo..=hi).contains(&v) {
            return Err(format!("{what} must be between {lo} and {hi}"));
        }
    }

    Ok(Character {
        id: spec.id,
        display_name: spec.display_name,
        description: spec.description,
        grid,
        palette,
        parts,
        face,
        motion: spec.motion,
        laptop: spec.props.laptop,
        floats: spec.archetype == "floaty",
        overrides: spec.overrides,
    })
}

// --- Archetypes ---------------------------------------------------------------------------

/// The face suggested by the archetype.
struct FaceDefault {
    eyes: EyeStyle,
    gap: f64,
    y: f64,
    mouth: MouthStyle,
    mouth_drop: f64,
    cheeks: bool,
    brows: bool,
}

/// Multiplies the numbers in a JSON object (position and size fields) by k.
fn scaled(mut v: Value, k: f64) -> Value {
    fn mul(v: &mut Value, k: f64) {
        match v {
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    *v = json!(f * k);
                }
            }
            Value::Array(a) => a.iter_mut().for_each(|x| mul(x, k)),
            _ => {}
        }
    }
    if let Some(o) = v.as_object_mut() {
        for key in ["at", "offset", "z", "size", "points", "width", "radius", "limb", "hand", "depth"] {
            if let Some(x) = o.get_mut(key) {
                mul(x, k);
            }
        }
    }
    v
}

fn opt<'a>(o: &'a Option<String>, default: &'a str) -> &'a str {
    o.as_deref().unwrap_or(default)
}

fn check_choice(what: &str, v: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&v) {
        Ok(())
    } else {
        Err(format!("options.{what} '{v}' is unknown (use {})", allowed.join(", ")))
    }
}

/// Part list from the archetype (written at 48×52 scale, scaled up to the grid).
fn archetype_parts(spec: &Spec, grid: Grid) -> Result<(Vec<Value>, FaceDefault), String> {
    let o = &spec.options;
    let size = o.size.unwrap_or(1.0);
    if !(0.6..=1.15).contains(&size) {
        return Err("options.size must be between 0.6 and 1.15".into());
    }
    let k = grid.k();
    let mut parts: Vec<Value> = Vec::new();
    let face;
    match spec.archetype.as_str() {
        "custom" => {
            face = FaceDefault { eyes: EyeStyle::Dot, gap: 10.0, y: -1.0, mouth: MouthStyle::Smile, mouth_drop: 4.0, cheeks: true, brows: false };
            return Ok((parts, scale_face(face, k)));
        }
        "blob" => {
            let shape = opt(&o.body, "round");
            check_choice("body", shape, &["round", "tall", "wide"])?;
            let (w, h) = match shape {
                "tall" => (26.0, 30.0),
                "wide" => (34.0, 23.0),
                _ => (30.0, 26.0),
            };
            let (w, h) = (w * size, h * size);
            let cy = -3.5 - h / 2.0;
            parts.push(json!({"name": "body", "role": "body", "on": "root", "shape": "ellipse",
                "at": [0, cy], "size": [w, h], "color": "body", "gloss": true}));
            if o.feet.unwrap_or(true) {
                parts.push(json!({"name": "foot", "role": "foot", "on": "root", "shape": "ellipse",
                    "at": [-6.0 * size, -3], "size": [8, 5.2], "color": "foot", "z": -1, "mirror": true, "shading": "band"}));
            }
            let arms = opt(&o.arms, "gesture");
            add_arms(&mut parts, arms, [w / 2.0 - 2.0, 1.0], [3.0, 2.5], 3.6, 4.0)?;
            add_topper(&mut parts, opt(&o.topper, "none"), "body", -h / 2.0 + 1.5)?;
            add_ears(&mut parts, opt(&o.ears, "none"), "body", w / 2.0, h / 2.0)?;
            add_tail(&mut parts, opt(&o.tail, "none"), [w / 2.0 - 3.0, h / 2.0 - 6.0])?;
            add_pattern(&mut parts, opt(&o.pattern, "none"), "body", w / 2.0, h / 2.0)?;
            if o.belly.unwrap_or(false) {
                parts.push(json!({"name": "belly", "role": "marking", "on": "body", "shape": "ellipse",
                    "at": [0, h * 0.22], "size": [w * 0.5, h * 0.4], "color": "belly", "clip": true, "surface": true, "shading": "flat"}));
            }
            add_accessory(&mut parts, opt(&o.accessory, "none"), "body", w / 2.0, h / 2.0, false)?;
            face = FaceDefault { eyes: EyeStyle::Dot, gap: 10.0, y: -1.0, mouth: MouthStyle::Smile, mouth_drop: 4.0, cheeks: true, brows: false };
        }
        "critter" => {
            let shape = opt(&o.body, "round");
            check_choice("body", shape, &["round", "tall", "wide"])?;
            let (hw, hh, bw, bh) = match shape {
                "tall" => (26.0, 22.0, 20.0, 19.0),
                "wide" => (31.0, 22.0, 25.0, 15.0),
                _ => (28.0, 22.0, 22.0, 16.0),
            };
            let (hw, hh, bw, bh) = (hw * size, hh * size, bw * size, bh * size);
            let feet = o.feet.unwrap_or(true);
            let lift = if feet { 2.0 } else { 0.5 };
            let by = -lift - bh / 2.0;
            parts.push(json!({"name": "body", "role": "body", "on": "root", "shape": "ellipse",
                "at": [0, by], "size": [bw, bh], "color": "body"}));
            parts.push(json!({"name": "head", "role": "head", "on": "body", "shape": "ellipse",
                "at": [0, -bh / 2.0 - hh / 2.0 + 4.0], "z": 1, "size": [hw, hh], "color": "body", "gloss": true}));
            if feet {
                parts.push(json!({"name": "foot", "role": "foot", "on": "root", "shape": "ellipse",
                    "at": [-bw * 0.24, -2.5], "size": [7.5, 5], "color": "foot", "z": -1, "mirror": true, "shading": "band"}));
            }
            add_arms(&mut parts, opt(&o.arms, "always"), [bw / 2.0 - 1.5, -bh * 0.12], [3.5, 3.0], 3.4, 3.6)?;
            add_ears(&mut parts, opt(&o.ears, "cat"), "head", hw / 2.0, hh / 2.0)?;
            add_topper(&mut parts, opt(&o.topper, "none"), "head", -hh / 2.0 + 1.0)?;
            add_tail(&mut parts, opt(&o.tail, "long"), [bw / 2.0 - 3.0, bh / 2.0 - 4.0])?;
            add_pattern(&mut parts, opt(&o.pattern, "none"), "head", hw / 2.0, hh / 2.0)?;
            if o.belly.unwrap_or(true) {
                parts.push(json!({"name": "belly", "role": "marking", "on": "body", "shape": "ellipse",
                    "at": [0, bh * 0.15], "size": [bw * 0.55, bh * 0.6], "color": "belly", "clip": true, "surface": true, "shading": "flat"}));
            }
            add_accessory(&mut parts, opt(&o.accessory, "none"), "body", bw / 2.0, bh / 2.0, true)?;
            face = FaceDefault { eyes: EyeStyle::Dot, gap: 11.0, y: 0.5, mouth: MouthStyle::Cat, mouth_drop: 3.5, cheeks: true, brows: false };
        }
        _ => {
            // floaty: footless, wavy hem, floating.
            let (w, h) = (28.0 * size, 28.0 * size);
            parts.push(json!({"name": "body", "role": "body", "on": "root", "shape": "ellipse",
                "at": [0, -9.0 - h / 2.0 - 3.0], "size": [w, h], "color": "body", "gloss": true}));
            parts.push(json!({"name": "skirt", "on": "body", "shape": "rect", "join": "body",
                "at": [0, h * 0.2], "size": [w, h * 0.45], "color": "body"}));
            for (i, x) in [-1.0, 0.0, 1.0].iter().enumerate() {
                parts.push(json!({"name": format!("hem{i}"), "on": "body", "shape": "ellipse", "join": "body",
                    "at": [x * w / 3.0, h * 0.42], "size": [w / 3.0 + 0.5, 6], "color": "body"}));
            }
            add_arms(&mut parts, opt(&o.arms, "always"), [w / 2.0 - 0.5, 2.0], [2.5, 2.0], 3.4, 3.6)?;
            add_topper(&mut parts, opt(&o.topper, "none"), "body", -h / 2.0 + 1.5)?;
            add_ears(&mut parts, opt(&o.ears, "none"), "body", w / 2.0, h / 2.0)?;
            add_tail(&mut parts, opt(&o.tail, "none"), [w / 2.0 - 3.0, h / 2.0 - 6.0])?;
            add_pattern(&mut parts, opt(&o.pattern, "none"), "body", w / 2.0, h / 2.0)?;
            add_accessory(&mut parts, opt(&o.accessory, "none"), "body", w / 2.0, h / 2.0, false)?;
            face = FaceDefault { eyes: EyeStyle::Dot, gap: 10.0, y: -2.0, mouth: MouthStyle::Curve, mouth_drop: 4.0, cheeks: true, brows: false };
        }
    }
    let parts = parts.into_iter().map(|p| scaled(p, k)).collect();
    Ok((parts, scale_face(face, k)))
}

fn scale_face(mut f: FaceDefault, k: f64) -> FaceDefault {
    f.gap *= k;
    f.y *= k;
    f.mouth_drop *= k;
    f
}

fn add_arms(parts: &mut Vec<Value>, mode: &str, at: [f64; 2], limb: [f64; 2], width: f64, hand: f64) -> Result<(), String> {
    check_choice("arms", mode, &["gesture", "always", "none"])?;
    if mode == "none" {
        return Ok(());
    }
    parts.push(json!({"name": "arm", "role": "arm", "on": "body", "shape": "limb", "at": at, "z": 2,
        "limb": limb, "width": width, "hand": hand, "color": "body", "mirror": true, "show": mode}));
    Ok(())
}

fn add_topper(parts: &mut Vec<Value>, kind: &str, on: &str, top: f64) -> Result<(), String> {
    check_choice("topper", kind, &["none", "sprout", "leaf", "antenna", "tuft", "horn", "bow"])?;
    let at = json!([0, top]);
    match kind {
        "sprout" => {
            parts.push(json!({"name": "stem", "role": "topper", "on": on, "shape": "capsule", "at": at, "z": 2,
                "points": [[0, 0.5], [0, -4.5]], "width": 1.6, "color": "outline", "outline": false}));
            parts.push(json!({"name": "leaf", "role": "topper", "on": on, "shape": "ellipse", "at": at, "z": 2,
                "offset": [-3, -5.5], "size": [6.4, 3.2], "rot": -25, "color": "accent"}));
            parts.push(json!({"name": "leaf2", "role": "topper", "on": on, "shape": "ellipse", "at": at, "z": 2,
                "offset": [3, -6.5], "size": [7.2, 3.4], "rot": 25, "color": "accent"}));
        }
        "leaf" => {
            parts.push(json!({"name": "stem", "role": "topper", "on": on, "shape": "capsule", "at": at, "z": 2,
                "points": [[0, 0.5], [0, -3.5]], "width": 1.6, "color": "outline", "outline": false}));
            parts.push(json!({"name": "leaf", "role": "topper", "on": on, "shape": "ellipse", "at": at, "z": 2,
                "offset": [3, -5], "size": [8, 3.6], "rot": -20, "color": "accent"}));
        }
        "antenna" => {
            parts.push(json!({"name": "stem", "role": "topper", "on": on, "shape": "capsule", "at": at, "z": 2,
                "points": [[0, 0.5], [0, -6]], "width": 1.2, "color": "outline", "outline": false}));
            parts.push(json!({"name": "bulb", "role": "topper", "on": on, "shape": "ellipse", "at": at, "z": 2,
                "offset": [0, -7.5], "size": [4.4, 4.4], "color": "accent"}));
        }
        "tuft" => {
            for (i, (x, r)) in [(-2.5, -25.0), (0.0, 0.0), (2.5, 25.0)].iter().enumerate() {
                parts.push(json!({"name": format!("tuft{i}"), "role": "topper", "on": on, "shape": "triangle", "at": at, "z": 2,
                    "points": [[x - 1.6, 1.5], [x + 1.6, 1.5], [x + r / 25.0, -4.5]], "color": "body", "join": if on == "head" { "head" } else { "body" }}));
            }
        }
        "horn" => {
            parts.push(json!({"name": "horn", "role": "topper", "on": on, "shape": "triangle", "at": at, "z": 2,
                "points": [[-2.5, 1.5], [2.5, 1.5], [0, -7]], "color": "accent"}));
        }
        "bow" => {
            parts.push(json!({"name": "bow", "role": "topper", "on": on, "shape": "triangle", "at": [5, top + 1.5], "z": 3,
                "points": [[0, 0], [5, -3.5], [5, 3]], "color": "accent"}));
            parts.push(json!({"name": "bow2", "role": "topper", "on": on, "shape": "triangle", "at": [5, top + 1.5], "z": 3,
                "points": [[0, 0], [-5, -3.5], [-5, 3]], "color": "accent"}));
            parts.push(json!({"name": "knot", "role": "topper", "on": on, "shape": "ellipse", "at": [5, top + 1.5], "z": 3.5,
                "size": [3, 3], "color": "accent.shade"}));
        }
        _ => {}
    }
    Ok(())
}

fn add_ears(parts: &mut Vec<Value>, kind: &str, on: &str, rx: f64, ry: f64) -> Result<(), String> {
    check_choice("ears", kind, &["none", "cat", "fox", "bear", "mouse", "bunny", "dog"])?;
    let join = if on == "head" { "head" } else { "body" };
    match kind {
        "cat" | "fox" => {
            let (b, t) = if kind == "fox" { (4.5, 10.0) } else { (4.0, 8.0) };
            let at = [-rx * 0.55, -ry * 0.7];
            parts.push(json!({"name": "ear", "role": "ear", "on": on, "shape": "triangle", "at": at, "mirror": true,
                "points": [[-b, 2], [b, 2], [-b * 0.4, -t]], "color": "body", "join": join}));
            parts.push(json!({"name": "earInner", "role": "ear", "on": on, "shape": "triangle", "at": at, "mirror": true, "z": 0.5,
                "points": [[-b * 0.45, 0.5], [b * 0.45, 0.5], [-b * 0.4 * 0.7, -t * 0.62]], "color": "inner", "outline": false, "shading": "flat"}));
        }
        "bear" | "mouse" => {
            let d = if kind == "mouse" { 11.0 } else { 8.0 };
            let at = [-rx * 0.7, -ry * 0.72];
            parts.push(json!({"name": "ear", "role": "ear", "on": on, "shape": "ellipse", "at": at, "mirror": true, "z": -1,
                "offset": [-d * 0.1, -d * 0.2], "size": [d, d], "color": "body", "join": join}));
            parts.push(json!({"name": "earInner", "role": "ear", "on": on, "shape": "ellipse", "at": at, "mirror": true, "z": 0.5,
                "offset": [-d * 0.1, -d * 0.25], "size": [d * 0.5, d * 0.5], "color": "inner", "outline": false, "shading": "flat"}));
        }
        "bunny" => {
            let at = [-rx * 0.35, -ry * 0.6];
            parts.push(json!({"name": "ear", "role": "ear", "on": on, "shape": "ellipse", "at": at, "mirror": true, "z": -1,
                "offset": [0, -4.5], "size": [5.5, 12.5], "rot": -8, "color": "body"}));
            parts.push(json!({"name": "earInner", "role": "ear", "on": on, "shape": "ellipse", "at": at, "mirror": true, "z": -0.5,
                "offset": [0, -4.5], "size": [2.4, 8], "rot": -8, "color": "inner", "outline": false, "shading": "flat"}));
        }
        "dog" => {
            parts.push(json!({"name": "ear", "role": "ear", "on": on, "shape": "ellipse", "at": [-rx * 0.82, -ry * 0.55], "mirror": true, "z": 2,
                "offset": [-0.5, 5], "size": [6, 12], "rot": 14, "color": "body.shade", "shading": "flat"}));
        }
        _ => {}
    }
    Ok(())
}

fn add_tail(parts: &mut Vec<Value>, kind: &str, at: [f64; 2]) -> Result<(), String> {
    check_choice("tail", kind, &["none", "short", "long", "curl", "fluffy"])?;
    match kind {
        "short" => parts.push(json!({"name": "tail", "role": "tail", "on": "body", "shape": "ellipse", "at": at, "z": -6,
            "offset": [2.5, 0], "size": [6, 6], "color": "body"})),
        "long" => parts.push(json!({"name": "tail", "role": "tail", "on": "body", "shape": "capsule", "at": at, "z": -6,
            "points": [[0, 0], [5, -1], [8, -6], [7.5, -11]], "width": 3.4, "color": "body"})),
        "curl" => parts.push(json!({"name": "tail", "role": "tail", "on": "body", "shape": "capsule", "at": at, "z": -6,
            "points": [[0, 0], [5, -2], [7, -6], [5, -9], [3, -7]], "width": 3, "color": "body"})),
        "fluffy" => parts.push(json!({"name": "tail", "role": "tail", "on": "body", "shape": "ellipse", "at": at, "z": -6,
            "offset": [5, -5], "size": [8, 14], "rot": 30, "color": "body"})),
        _ => {}
    }
    Ok(())
}

fn add_pattern(parts: &mut Vec<Value>, kind: &str, on: &str, rx: f64, ry: f64) -> Result<(), String> {
    check_choice("pattern", kind, &["none", "spots", "stripes"])?;
    match kind {
        "spots" => {
            for (i, (x, y, d)) in [(-0.55, -0.45, 4.0), (0.6, -0.25, 3.0), (0.45, 0.45, 3.5)].iter().enumerate() {
                parts.push(json!({"name": format!("spot{i}"), "role": "marking", "on": on, "shape": "ellipse", "surface": true, "clip": true,
                    "at": [x * rx, y * ry], "size": [*d, d * 0.8], "color": "pattern", "shading": "flat"}));
            }
        }
        "stripes" => {
            for (i, x) in [-0.35, 0.0, 0.35].iter().enumerate() {
                parts.push(json!({"name": format!("stripe{i}"), "role": "marking", "on": on, "shape": "rect", "surface": true, "clip": true,
                    "at": [x * rx, -ry * 0.85], "size": [2, ry * 0.5], "rot": x * 25.0, "color": "pattern", "shading": "flat"}));
            }
        }
        _ => {}
    }
    Ok(())
}

fn add_accessory(parts: &mut Vec<Value>, kind: &str, on: &str, rx: f64, ry: f64, has_neck: bool) -> Result<(), String> {
    check_choice("accessory", kind, &["none", "scarf", "bowtie"])?;
    let neck = if has_neck { -ry * 0.75 } else { ry * 0.52 };
    match kind {
        "scarf" => {
            parts.push(json!({"name": "scarf", "role": "accessory", "on": on, "shape": "rect", "z": 1,
                "at": [0, neck], "size": [rx * 2.0 + 1.0, 3.2], "radius": 1.5, "color": "accent"}));
            parts.push(json!({"name": "scarfTail", "role": "accessory", "on": on, "shape": "rect", "z": 1.5,
                "at": [rx * 0.35, neck + 1.0], "offset": [0, 2.5], "size": [3, 6], "rot": -12, "color": "accent"}));
        }
        "bowtie" => {
            let at = [0.0, neck];
            parts.push(json!({"name": "bowtie", "role": "accessory", "on": on, "shape": "triangle", "z": 1, "at": at,
                "points": [[0, 0], [4, -2.5], [4, 2.5]], "color": "accent", "mirror": true}));
            parts.push(json!({"name": "bowtieKnot", "role": "accessory", "on": on, "shape": "ellipse", "z": 1.5, "at": at,
                "size": [2.4, 2.4], "color": "accent.shade"}));
        }
        _ => {}
    }
    Ok(())
}

/// Level (a) example specs for `--example`.
pub fn example(name: &str) -> Option<&'static str> {
    match name {
        "pitir" => Some(include_str!("../../../../skills/hatch/examples/pitir.json")),
        "critter" => Some(include_str!("../../../../skills/hatch/examples/blue-cat.json")),
        "floaty" => Some(include_str!("../../../../skills/hatch/examples/ghost.json")),
        "custom" => Some(include_str!("../../../../skills/hatch/examples/custom-parts.json")),
        _ => None,
    }
}

pub const EXAMPLES: &[&str] = &["pitir", "critter", "floaty", "custom"];

