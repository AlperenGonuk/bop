//! Draws a single frame: parts, pseudo-3D rotation (yaw/pitch on an ellipsoid), depth order,
//! outline, shading, face and props. Output is a color + tag buffer on the logical grid.
//!
//! Coordinates: root = ground center, y positive downward, z positive toward the viewer.
//! Groups: `root` (ground, does not turn), `body` (body center), `head` (head center; without a
//! head, the "face" group on the body: the same ellipse as the body, but it turns with the head's angles).

use super::color::{Rgba, Tone};
use super::spec::{Character, EyeStyle, GroupId, MouthStyle, PartSpec, Role, Shading, ShapeKind, Show};
use std::f64::consts::PI;

pub const TAG_NONE: u8 = 0;
pub const TAG_BODY: u8 = 1;
pub const TAG_HEAD: u8 = 2;
pub const TAG_EYE: u8 = 3;
pub const TAG_PART: u8 = 4;
pub const TAG_OUTLINE: u8 = 5;
pub const TAG_FACE: u8 = 6;
pub const TAG_PROP: u8 = 7;
pub const TAG_OVERRIDE: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Eyes {
    Open,
    Closed,
    Happy,
    Squeeze,
    Focus,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mouth {
    Rest,
    Open,
    Flat,
    Frown,
    O,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Brows {
    Rest,
    Worried,
    Raised,
}

/// Per-frame pose. Lengths are in 48×52 grid units (scaled by k when drawing),
/// angles in degrees. `eye_x`/`eye_y`: −1…1 (right/up positive).
#[derive(Clone, Debug)]
pub struct Pose {
    pub dx: f64,
    pub dy: f64,
    pub squash: f64,
    pub lean: f64,
    pub body_yaw: f64,
    pub head_yaw: f64,
    pub head_pitch: f64,
    pub head_roll: f64,
    pub eye_x: f64,
    pub eye_y: f64,
    pub eyes: Eyes,
    pub mouth: Mouth,
    pub brows: Brows,
    pub sway: f64,
    pub droop: f64,
    /// (shoulder, elbow) angle; shoulder 0 = down, 90 = horizontal outward, 180 = up; negative = inward.
    pub arm_l: Option<(f64, f64)>,
    pub arm_r: Option<(f64, f64)>,
    pub foot_l: (f64, f64),
    pub foot_r: (f64, f64),
    pub feet_dy: f64,
    pub laptop: Option<u32>,
    pub sweat: bool,
}

impl Default for Pose {
    fn default() -> Self {
        Pose {
            dx: 0.0,
            dy: 0.0,
            squash: 0.0,
            lean: 0.0,
            body_yaw: 0.0,
            head_yaw: 0.0,
            head_pitch: 0.0,
            head_roll: 0.0,
            eye_x: 0.0,
            eye_y: 0.0,
            eyes: Eyes::Open,
            mouth: Mouth::Rest,
            brows: Brows::Rest,
            sway: 0.0,
            droop: 0.0,
            arm_l: None,
            arm_r: None,
            foot_l: (0.0, 0.0),
            foot_r: (0.0, 0.0),
            feet_dy: 0.0,
            laptop: None,
            sweat: false,
        }
    }
}

/// Logical frame: color and tag (which kind of part drew the pixel).
#[derive(Clone)]
pub struct Frame {
    pub w: usize,
    pub h: usize,
    pub px: Vec<Rgba>,
    pub tag: Vec<u8>,
}

impl Frame {
    pub fn new(w: usize, h: usize) -> Frame {
        Frame { w, h, px: vec![Rgba::CLEAR; w * h], tag: vec![TAG_NONE; w * h] }
    }
    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        (x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h).then(|| y as usize * self.w + x as usize)
    }
    pub fn set(&mut self, x: i32, y: i32, c: Rgba, tag: u8) {
        if let Some(i) = self.idx(x, y) {
            if c.is_clear() {
                self.px[i] = Rgba::CLEAR;
                self.tag[i] = TAG_NONE;
            } else {
                self.px[i] = c;
                self.tag[i] = tag;
            }
        }
    }
    pub fn tag_at(&self, x: i32, y: i32) -> u8 {
        self.idx(x, y).map(|i| self.tag[i]).unwrap_or(TAG_NONE)
    }
}

// --- Geometry -----------------------------------------------------------------------------

type P2 = (f64, f64);

fn rot2(p: P2, deg: f64) -> P2 {
    let a = deg * PI / 180.0;
    (p.0 * a.cos() - p.1 * a.sin(), p.0 * a.sin() + p.1 * a.cos())
}

/// Rotation on an ellipsoid: point (x, y, z), radii (rx, ry, rz).
/// Returns the new point and the horizontal angle after rotation (surface normal's angle to the viewer).
fn rotate3(p: [f64; 3], r: [f64; 3], yaw: f64, pitch: f64, roll: f64) -> ([f64; 3], f64) {
    let (u, w) = (p[0] / r[0], p[2] / r[2]);
    let rho = (u * u + w * w).sqrt();
    let phi = u.atan2(w) + yaw * PI / 180.0;
    let (x1, z1) = (r[0] * rho * phi.sin(), r[2] * rho * phi.cos());
    let (v, w2) = (p[1] / r[1], z1 / r[2]);
    let rho2 = (v * v + w2 * w2).sqrt();
    let th = v.atan2(w2) - pitch * PI / 180.0;
    let (y1, z2) = (r[1] * rho2 * th.sin(), r[2] * rho2 * th.cos());
    let (x2, y2) = rot2((x1, y1), roll);
    ([x2, y2, z2], phi)
}

enum Geo {
    Ellipse { c: P2, rx: f64, ry: f64, rot: f64 },
    Rect { c: P2, hw: f64, hh: f64, r: f64, rot: f64 },
    Poly { pts: Vec<P2> },
    Capsule { pts: Vec<P2>, hw: f64 },
    Pixels { c: P2, rows: Vec<Vec<Option<Rgba>>>, rot: f64, fx: f64 },
}

fn seg_dist(p: P2, a: P2, b: P2) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l2 = dx * dx + dy * dy;
    let t = if l2 < 1e-9 { 0.0 } else { (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / l2).clamp(0.0, 1.0) };
    let (qx, qy) = (a.0 + t * dx - p.0, a.1 + t * dy - p.1);
    (qx * qx + qy * qy).sqrt()
}

impl Geo {
    fn bbox(&self) -> (f64, f64, f64, f64) {
        match self {
            Geo::Ellipse { c, rx, ry, .. } => {
                let m = rx.max(*ry);
                (c.0 - m, c.1 - m, c.0 + m, c.1 + m)
            }
            Geo::Rect { c, hw, hh, .. } => {
                let m = (hw * hw + hh * hh).sqrt();
                (c.0 - m, c.1 - m, c.0 + m, c.1 + m)
            }
            Geo::Poly { pts } => bbox_of(pts, 0.0),
            Geo::Capsule { pts, hw } => bbox_of(pts, *hw),
            Geo::Pixels { c, rows, .. } => {
                let w = rows.iter().map(Vec::len).max().unwrap_or(0) as f64;
                let m = (w * w + (rows.len() as f64).powi(2)).sqrt() / 2.0 + 1.0;
                (c.0 - m, c.1 - m, c.0 + m, c.1 + m)
            }
        }
    }

    /// Whether the pixel center (x, y) is inside the shape; if so, the normal (nx, ny) for shading
    /// and (for pixel maps) its own color.
    fn hit(&self, x: f64, y: f64) -> Option<(P2, Option<Rgba>)> {
        match self {
            Geo::Ellipse { c, rx, ry, rot } => {
                let q = rot2((x - c.0, y - c.1), -rot);
                let (u, v) = (q.0 / rx, q.1 / ry);
                (u * u + v * v <= 1.0).then(|| (rot2((u, v), *rot), None))
            }
            Geo::Rect { c, hw, hh, r, rot } => {
                let q = rot2((x - c.0, y - c.1), -rot);
                let (ax, ay) = (q.0.abs(), q.1.abs());
                if ax > *hw || ay > *hh {
                    return None;
                }
                let r = r.min(*hw).min(*hh);
                if ax > hw - r && ay > hh - r {
                    let (ex, ey) = (ax - (hw - r), ay - (hh - r));
                    if ex * ex + ey * ey > r * r {
                        return None;
                    }
                }
                Some((rot2((q.0 / hw, q.1 / hh), *rot), None))
            }
            Geo::Poly { pts } => {
                let mut inside = false;
                let n = pts.len();
                for i in 0..n {
                    let (a, b) = (pts[i], pts[(i + 1) % n]);
                    if (a.1 > y) != (b.1 > y) && x < (b.0 - a.0) * (y - a.1) / (b.1 - a.1) + a.0 {
                        inside = !inside;
                    }
                }
                inside.then(|| (bbox_normal(pts, x, y), None))
            }
            Geo::Capsule { pts, hw } => {
                let hit = pts.windows(2).any(|s| seg_dist((x, y), s[0], s[1]) <= *hw);
                hit.then(|| (bbox_normal(pts, x, y), None))
            }
            Geo::Pixels { c, rows, rot, fx } => {
                let q = rot2((x - c.0, y - c.1), -rot);
                let w = rows.iter().map(Vec::len).max().unwrap_or(0) as f64;
                let h = rows.len() as f64;
                let i = (q.0 / fx + w / 2.0).floor();
                let j = (q.1 + h / 2.0).floor();
                if i < 0.0 || j < 0.0 || j >= h {
                    return None;
                }
                let row = &rows[j as usize];
                let col = row.get(i as usize).copied().flatten()?;
                Some(((0.0, 0.0), Some(col)))
            }
        }
    }
}

fn bbox_of(pts: &[P2], pad: f64) -> (f64, f64, f64, f64) {
    let mut b = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for p in pts {
        b = (b.0.min(p.0), b.1.min(p.1), b.2.max(p.0), b.3.max(p.1));
    }
    (b.0 - pad, b.1 - pad, b.2 + pad, b.3 + pad)
}

fn bbox_normal(pts: &[P2], x: f64, y: f64) -> P2 {
    let b = bbox_of(pts, 0.0);
    let (cx, cy) = ((b.0 + b.2) / 2.0, (b.1 + b.3) / 2.0);
    let (hw, hh) = (((b.2 - b.0) / 2.0).max(0.5), ((b.3 - b.1) / 2.0).max(0.5));
    ((x - cx) / hw, (y - cy) / hh)
}

/// A sub-shape in a combined draw item.
struct Sub {
    geo: Geo,
    tone: Tone,
    shading: Shading,
}

/// An item drawn in painter's order.
enum Item {
    Shape {
        subs: Vec<Sub>,
        outline: bool,
        /// Drawn only on top of this tag (surface markings).
        clip: Option<u8>,
        tag: u8,
        gloss: bool,
    },
    Face,
    Laptop(u32),
    Sweat,
}

/// Group transforms (for one frame).
struct Rig {
    k: f64,
    origin: P2,
    body_at: [f64; 3],
    body_r: [f64; 3],
    head: Option<([f64; 3], [f64; 3])>,
    sx: f64,
    sy: f64,
    pose: Pose,
}

impl Rig {
    fn hip(&self) -> f64 {
        self.body_r[1]
    }

    /// Moves a point in body space (already rotated) to root space: squash, lean, translation.
    fn body_post(&self, p: [f64; 3]) -> [f64; 3] {
        let hip = self.hip();
        let x = p[0] * self.sx;
        let y = hip + (p[1] - hip) * self.sy;
        let (lx, ly) = rot2((x, y - hip), self.pose.lean);
        [
            self.body_at[0] + lx + self.pose.dx * self.k,
            self.body_at[1] + hip + ly + self.pose.dy * self.k,
            self.body_at[2] + p[2],
        ]
    }

    /// A group-space point's position in root space and its horizontal angle after rotation.
    fn place(&self, g: GroupId, p: [f64; 3]) -> ([f64; 3], f64) {
        match g {
            GroupId::Root => (p, 0.0),
            GroupId::Body => {
                let (q, phi) = rotate3(p, self.body_r, self.pose.body_yaw, 0.0, 0.0);
                (self.body_post(q), phi)
            }
            GroupId::Head => match &self.head {
                Some((anchor, r)) => {
                    let (a, _) = self.place(GroupId::Body, *anchor);
                    let (q, phi) = rotate3(p, *r, self.pose.head_yaw, self.pose.head_pitch, self.pose.head_roll);
                    let (lx, ly) = rot2((q[0], q[1]), self.pose.lean);
                    ([a[0] + lx, a[1] + ly, a[2] + q[2]], phi)
                }
                None => {
                    let (q, phi) = rotate3(p, self.body_r, self.pose.head_yaw, self.pose.head_pitch, self.pose.head_roll);
                    (self.body_post(q), phi)
                }
            },
        }
    }

    fn group_r(&self, g: GroupId) -> [f64; 3] {
        match (g, &self.head) {
            (GroupId::Head, Some((_, r))) => *r,
            _ => self.body_r,
        }
    }

    fn group_rot(&self, g: GroupId) -> f64 {
        match g {
            GroupId::Root => 0.0,
            GroupId::Body => self.pose.lean,
            GroupId::Head => self.pose.lean + self.pose.head_roll,
        }
    }

    fn main_tag(&self, g: GroupId) -> u8 {
        if g == GroupId::Head && self.head.is_some() {
            TAG_HEAD
        } else {
            TAG_BODY
        }
    }

    /// Root-space point → logical canvas.
    fn canvas(&self, p: [f64; 3]) -> P2 {
        (self.origin.0 + p[0], self.origin.1 + p[1])
    }
}

/// Depth of a point on the surface (relative to the group's main ellipse).
fn surface_z(r: [f64; 3], x: f64, y: f64) -> f64 {
    let t = 1.0 - (x / r[0]).powi(2) - (y / r[1]).powi(2);
    r[2] * t.max(0.0).sqrt()
}

fn default_sway(role: Role) -> f64 {
    match role {
        Role::Topper => 1.0,
        Role::Tail => 1.2,
        Role::Ear => 0.35,
        _ => 0.0,
    }
}

fn default_droop(role: Role) -> f64 {
    match role {
        Role::Topper => 22.0,
        Role::Ear => 30.0,
        Role::Tail => 25.0,
        _ => 0.0,
    }
}

/// The part's geometry in this frame; `None` if it is not visible.
fn part_geo(ch: &Character, rig: &Rig, p: &PartSpec, squash: bool) -> Option<(Geo, f64)> {
    let pose = &rig.pose;
    // The main body/head is the center of its own group (`at` only places the group).
    let (group, at) = match p.role {
        Role::Body => (GroupId::Body, [0.0, 0.0]),
        Role::Head => (GroupId::Head, [0.0, 0.0]),
        _ => (p.on, p.at),
    };
    let r = rig.group_r(group);
    let center = [at[0] + p.offset[0], at[1] + p.offset[1]];
    let mut fx = 1.0;
    let (anchor, z) = if p.surface && group != GroupId::Root {
        let zs = surface_z(r, center[0], center[1]);
        let (_, phi) = rig.place(group, [center[0], center[1], zs]);
        let facing = phi.cos();
        if facing < 0.2 {
            return None;
        }
        fx = facing.clamp(0.3, 1.0);
        let (a, _) = rig.place(group, [at[0], at[1], zs]);
        (a, a[2])
    } else {
        let z0 = if matches!(p.role, Role::Body | Role::Head) { 0.0 } else { p.z.unwrap_or(0.0) };
        let (a, _) = rig.place(group, [at[0], at[1], z0]);
        (a, a[2])
    };
    let side = if center[0] < -0.01 { -1.0 } else if center[0] > 0.01 { 1.0 } else { 0.0 };
    let sway = p.sway.unwrap_or(default_sway(p.role)) * pose.sway;
    let droop = p.droop.unwrap_or(default_droop(p.role)) * pose.droop * side;
    let rot = p.rot + sway + droop + rig.group_rot(group);
    let base = rig.canvas(anchor);
    let (ssx, ssy) = if squash { (rig.sx, rig.sy) } else { (1.0, 1.0) };
    let off = rot2((p.offset[0] * fx * ssx, p.offset[1] * ssy), rot - p.rot);
    let c = (base.0 + off.0, base.1 + off.1);
    let tf = |pt: &[f64; 2]| -> P2 {
        let q = rot2((pt[0] * fx, pt[1]), rot - p.rot);
        (base.0 + q.0, base.1 + q.1)
    };
    let geo = match p.shape {
        ShapeKind::Ellipse => {
            let s = p.size?;
            Geo::Ellipse { c, rx: s[0] / 2.0 * fx * ssx, ry: s[1] / 2.0 * ssy, rot }
        }
        ShapeKind::Rect => {
            let s = p.size?;
            Geo::Rect { c, hw: s[0] / 2.0 * fx * ssx, hh: s[1] / 2.0 * ssy, r: p.radius, rot }
        }
        ShapeKind::Triangle | ShapeKind::Polygon => Geo::Poly { pts: p.points.iter().map(tf).collect() },
        ShapeKind::Capsule => Geo::Capsule { pts: p.points.iter().map(tf).collect(), hw: p.width? / 2.0 },
        ShapeKind::Pixels => {
            let rows = p.rows.iter().map(|row| row.chars().map(|c| ch.pixel(c, &p.key)).collect()).collect();
            Geo::Pixels { c, rows, rot, fx }
        }
        ShapeKind::Limb => return None,
    };
    Some((geo, z))
}

/// Arm: shoulder + two segments + hand, as one combined shape.
fn limb_subs(ch: &Character, rig: &Rig, p: &PartSpec, angles: (f64, f64)) -> Option<(Vec<Sub>, f64)> {
    let (a, _) = rig.place(p.on, [p.at[0], p.at[1], p.z.unwrap_or(0.0)]);
    let s = if p.at[0] < 0.0 { -1.0 } else { 1.0 };
    let lim = p.limb?;
    let w = p.width?;
    let dir = |deg: f64| -> P2 {
        let r = (deg + rig.pose.lean * s) * PI / 180.0;
        (s * r.sin(), r.cos())
    };
    let sh = rig.canvas(a);
    let d1 = dir(angles.0);
    let el = (sh.0 + d1.0 * lim[0], sh.1 + d1.1 * lim[0]);
    let d2 = dir(angles.0 + angles.1);
    let hand = (el.0 + d2.0 * lim[1], el.1 + d2.1 * lim[1]);
    let tone = ch.tone(p.color.as_deref().unwrap_or("body")).ok()?;
    let hd = p.hand.unwrap_or(w * 1.1) / 2.0;
    let subs = vec![
        Sub { geo: Geo::Capsule { pts: vec![sh, el, hand], hw: w / 2.0 }, tone, shading: Shading::Flat },
        Sub { geo: Geo::Ellipse { c: hand, rx: hd, ry: hd, rot: 0.0 }, tone, shading: Shading::Ball },
    ];
    Some((subs, a[2]))
}

/// Draws one frame of the character.
pub fn render(ch: &Character, pose: &Pose) -> Frame {
    let g = ch.grid;
    let k = g.k();
    let body = ch.parts.iter().find(|p| p.role == Role::Body).expect("body part was validated");
    let bs = body.size.unwrap_or([10.0, 10.0]);
    let body_r = [bs[0] / 2.0, bs[1] / 2.0, body.depth.map(|d| d / 2.0).unwrap_or(bs[0] / 2.0)];
    let head = ch.parts.iter().find(|p| p.role == Role::Head).map(|h| {
        let s = h.size.unwrap_or([10.0, 10.0]);
        ([h.at[0], h.at[1], h.z.unwrap_or(1.0)], [s[0] / 2.0, s[1] / 2.0, h.depth.map(|d| d / 2.0).unwrap_or(s[0] / 2.0)])
    });
    let sq = pose.squash * k;
    let rig = Rig {
        k,
        origin: g.origin(),
        body_at: [body.at[0], body.at[1], 0.0],
        body_r,
        head,
        sx: (body_r[0] + sq) / body_r[0],
        sy: (body_r[1] - 0.8 * sq) / body_r[1],
        pose: pose.clone(),
    };

    // Collect the items.
    let mut items: Vec<(f64, usize, Item)> = Vec::new();
    let mut seq = 0usize;
    let mut push = |items: &mut Vec<(f64, usize, Item)>, z: f64, it: Item| {
        items.push((z, seq, it));
        seq += 1;
    };
    for p in ch.parts.iter().filter(|p| p.join.is_none()) {
        if p.role == Role::Foot && ch.floats {
            continue;
        }
        if p.role == Role::Arm {
            if pose.laptop.is_some() {
                continue;
            }
            let side_r = p.at[0] >= 0.0;
            let show = p.show.unwrap_or(Show::Always);
            let angles = match (if side_r { pose.arm_r } else { pose.arm_l }, show) {
                (_, Show::Never) => None,
                (Some(a), _) => Some(a),
                (None, Show::Always) => Some((14.0, 8.0)),
                (None, Show::Gesture) => None,
            };
            if let Some(a) = angles {
                if let Some((subs, z)) = limb_subs(ch, &rig, p, a) {
                    push(&mut items, z, Item::Shape { subs, outline: true, clip: None, tag: TAG_PART, gloss: false });
                }
            }
            continue;
        }
        let is_main = matches!(p.role, Role::Body | Role::Head);
        let squash = p.role == Role::Body;
        let Some((mut geo, z)) = part_geo(ch, &rig, p, squash) else { continue };
        if p.role == Role::Foot {
            let off = if p.at[0] < 0.0 { pose.foot_l } else { pose.foot_r };
            geo = shift(geo, (off.0 * k, (off.1 + pose.feet_dy) * k));
        }
        let tone = ch.tone(p.color.as_deref().unwrap_or("body")).unwrap_or(Tone::flat(Rgba::rgb(255, 0, 255)));
        let shading = p.shading.unwrap_or(if p.shape == ShapeKind::Pixels { Shading::Flat } else { Shading::Ball });
        let mut subs = vec![Sub { geo, tone, shading }];
        // Parts joined to this one (same outline, same depth).
        for j in ch.parts.iter().filter(|j| j.join.as_deref() == Some(p.name.as_str())) {
            if let Some((geo, _)) = part_geo(ch, &rig, j, squash) {
                let tone = ch.tone(j.color.as_deref().unwrap_or("body")).unwrap_or(tone);
                subs.push(Sub { geo, tone, shading: j.shading.unwrap_or(Shading::Flat) });
            }
        }
        let tag = match p.role {
            Role::Body => TAG_BODY,
            Role::Head => TAG_HEAD,
            _ => TAG_PART,
        };
        let outline = p.outline.unwrap_or(!(p.clip || p.shape == ShapeKind::Pixels));
        let clip = p.clip.then(|| rig.main_tag(p.on));
        push(&mut items, z, Item::Shape { subs, outline, clip, tag, gloss: p.gloss && is_main });
    }
    // Face: the front surface of the face group.
    let fr = rig.group_r(GroupId::Head);
    let (fc, _) = rig.place(GroupId::Head, [0.0, 0.0, 0.0]);
    push(&mut items, fc[2] + fr[2] + 0.5, Item::Face);
    if pose.sweat {
        push(&mut items, fc[2] + fr[2] + 0.6, Item::Sweat);
    }
    if let Some(t) = pose.laptop {
        if ch.laptop {
            push(&mut items, 1000.0, Item::Laptop(t));
        }
    }
    items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal).then(a.1.cmp(&b.1)));

    let mut f = Frame::new(g.w, g.h);
    let outline = ch.color("outline").unwrap_or(Rgba::rgb(0x2B, 0x21, 0x40));
    for (_, _, it) in &items {
        match it {
            Item::Shape { subs, outline: ol, clip, tag, gloss } => {
                paint_shape(&mut f, subs, *ol, *clip, *tag, outline);
                if *gloss {
                    if let Geo::Ellipse { c, rx, ry, .. } = &subs[0].geo {
                        let x = (c.0 - rx * 0.5 + 1.5 * k * 0.75).floor() as i32;
                        let y = (c.1 - ry * 0.6).floor() as i32;
                        if f.tag_at(x, y) == *tag {
                            for (dx, dy) in block(k) {
                                f.set(x + dx, y + dy, ch.color("white").unwrap_or(Rgba::rgb(255, 255, 255)), *tag);
                            }
                        }
                    }
                }
            }
            Item::Face => draw_face(&mut f, ch, &rig),
            Item::Sweat => draw_sweat(&mut f, ch, &rig),
            Item::Laptop(t) => draw_laptop(&mut f, ch, &rig, *t),
        }
    }
    f
}

fn shift(g: Geo, d: P2) -> Geo {
    let m = |p: P2| (p.0 + d.0, p.1 + d.1);
    match g {
        Geo::Ellipse { c, rx, ry, rot } => Geo::Ellipse { c: m(c), rx, ry, rot },
        Geo::Rect { c, hw, hh, r, rot } => Geo::Rect { c: m(c), hw, hh, r, rot },
        Geo::Poly { pts } => Geo::Poly { pts: pts.into_iter().map(m).collect() },
        Geo::Capsule { pts, hw } => Geo::Capsule { pts: pts.into_iter().map(m).collect(), hw },
        Geo::Pixels { c, rows, rot, fx } => Geo::Pixels { c: m(c), rows, rot, fx },
    }
}

/// k×k block (on the 96×104 grid, face patterns are drawn as 2×2 pixels).
fn block(k: f64) -> Vec<(i32, i32)> {
    let n = k.round().max(1.0) as i32;
    (0..n).flat_map(|y| (0..n).map(move |x| (x, y))).collect()
}

fn paint_shape(f: &mut Frame, subs: &[Sub], outline: bool, clip: Option<u8>, tag: u8, ol: Rgba) {
    let mut b = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for s in subs {
        let q = s.geo.bbox();
        b = (b.0.min(q.0), b.1.min(q.1), b.2.max(q.2), b.3.max(q.3));
    }
    let x0 = (b.0.floor() as i32 - 2).max(-1);
    let y0 = (b.1.floor() as i32 - 2).max(-1);
    let x1 = (b.2.ceil() as i32 + 2).min(f.w as i32);
    let y1 = (b.3.ceil() as i32 + 2).min(f.h as i32);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let (bw, bh) = ((x1 - x0 + 1) as usize, (y1 - y0 + 1) as usize);
    let mut fill: Vec<Option<Rgba>> = vec![None; bw * bh];
    for yy in 0..bh {
        for xx in 0..bw {
            let (x, y) = (x0 + xx as i32, y0 + yy as i32);
            let (cx, cy) = (x as f64 + 0.5, y as f64 + 0.5);
            let mut col = None;
            // The first (main) sub-shape sets the lighting; joined shapes are shaded with its normal.
            let main_n = subs.first().and_then(|s| s.geo.hit(cx, cy)).map(|h| h.0);
            for s in subs {
                if let Some((n, own)) = s.geo.hit(cx, cy) {
                    col = Some(match own {
                        Some(c) => c,
                        None => shade(s.tone, s.shading, main_n.unwrap_or(n)),
                    });
                }
            }
            fill[yy * bw + xx] = col;
        }
    }
    if let Some(ct) = clip {
        for yy in 0..bh {
            for xx in 0..bw {
                if fill[yy * bw + xx].is_some() && f.tag_at(x0 + xx as i32, y0 + yy as i32) != ct {
                    fill[yy * bw + xx] = None;
                }
            }
        }
    }
    if outline {
        for yy in 0..bh {
            for xx in 0..bw {
                if fill[yy * bw + xx].is_some() {
                    continue;
                }
                let near = [(0i32, -1i32), (0, 1), (-1, 0), (1, 0)].iter().any(|(dx, dy)| {
                    let (nx, ny) = (xx as i32 + dx, yy as i32 + dy);
                    nx >= 0 && ny >= 0 && (nx as usize) < bw && (ny as usize) < bh && fill[ny as usize * bw + nx as usize].is_some()
                });
                if near {
                    f.set(x0 + xx as i32, y0 + yy as i32, ol, TAG_OUTLINE);
                }
            }
        }
    }
    for yy in 0..bh {
        for xx in 0..bw {
            if let Some(c) = fill[yy * bw + xx] {
                f.set(x0 + xx as i32, y0 + yy as i32, c, tag);
            }
        }
    }
}

/// Light comes from the top left and is fixed (it does not move in the mirrored run either).
fn shade(t: Tone, s: Shading, n: P2) -> Rgba {
    match s {
        Shading::Flat => t.base,
        Shading::Band => {
            if n.1 > 0.45 {
                t.shade
            } else {
                t.base
            }
        }
        Shading::Ball => {
            if n.0 * 0.35 + n.1 * 0.9 > 0.62 {
                return t.shade;
            }
            let (hx, hy) = (n.0 + 0.5, n.1 + 0.55);
            if hx * hx * 1.4 + hy * hy < 0.045 {
                t.hi
            } else {
                t.base
            }
        }
    }
}

// --- Face ----------------------------------------------------------------------------------

/// Pattern: rows, origin column/row. Letters: E eye/outline, W white, T tongue, C cheek.
struct Pat(&'static [&'static str], i32, i32);

const EYE_OPEN: Pat = Pat(&[".EE.", "EEEE", "EEEE", "EEEE", ".EE."], 2, 2);
const EYE_NARROW: Pat = Pat(&[".E.", "EEE", "EEE", "EEE", ".E."], 1, 2);
const EYE_BEAN: Pat = Pat(&["EE", "EE", "EE"], 1, 1);
const EYE_ROUND: Pat = Pat(&[".EEE.", "EWWWE", "EWWWE", "EWWWE", "EWWWE", ".EEE."], 2, 3);
const EYE_CLOSED: Pat = Pat(&["E..E", ".EE."], 2, 0);
const EYE_HAPPY: Pat = Pat(&[".EE.", "E..E"], 2, 0);
const EYE_SQUEEZE_L: Pat = Pat(&["E..", ".E.", "..E", ".E.", "E.."], 2, 2);
const EYE_SQUEEZE_R: Pat = Pat(&["..E", ".E.", "E..", ".E.", "..E"], 1, 2);
const EYE_FOCUS: Pat = Pat(&["EEEE", "EEEE"], 2, 0);
const MOUTH_SMILE: Pat = Pat(&["E.E.E", ".E.E."], 2, 0);
const MOUTH_CURVE: Pat = Pat(&["E...E", ".EEE."], 2, 0);
const MOUTH_CAT: Pat = Pat(&[".TT.", "E.E.E", ".E.E."], 2, 1);
const MOUTH_OPEN: Pat = Pat(&["EEEEE", "ETTTE", ".EEE."], 2, 0);
const MOUTH_FLAT: Pat = Pat(&["EEE"], 1, 0);
const MOUTH_FROWN: Pat = Pat(&[".EEE.", "E...E"], 2, 0);
const MOUTH_O: Pat = Pat(&[".EE.", "ETTE", ".EE."], 1, 0);
const BROW_REST_L: Pat = Pat(&["EEE"], 1, 0);
const BROW_REST_R: Pat = Pat(&["EEE"], 1, 0);
const BROW_WORRIED_L: Pat = Pat(&["..E", "EE."], 1, 0);
const BROW_WORRIED_R: Pat = Pat(&["E..", ".EE"], 1, 0);
const SWEAT: Pat = Pat(&[".E.", "EWE", "ESE", ".E."], 1, 0);

fn stamp(f: &mut Frame, ch: &Character, pat: &Pat, at: (i32, i32), k: f64, tag: u8) {
    let n = k.round().max(1.0) as i32;
    let eye = ch.face.eye_color;
    let white = ch.color("white").unwrap_or(Rgba::rgb(255, 255, 255));
    let tongue = ch.color("tongue").unwrap_or(eye);
    let sweat = ch.color("sweat").unwrap_or(white);
    let outline = ch.color("outline").unwrap_or(eye);
    for (j, row) in pat.0.iter().enumerate() {
        for (i, c) in row.chars().enumerate() {
            let col = match c {
                'E' => {
                    if tag == TAG_PROP {
                        outline
                    } else {
                        eye
                    }
                }
                'W' => white,
                'T' => tongue,
                'S' => sweat,
                _ => continue,
            };
            let (px, py) = (at.0 + (i as i32 - pat.1) * n, at.1 + (j as i32 - pat.2) * n);
            for (dx, dy) in block(k) {
                // Facial features do not spill outside the silhouette.
                if (tag == TAG_EYE || tag == TAG_FACE) && matches!(f.tag_at(px + dx, py + dy), TAG_NONE | TAG_OUTLINE) {
                    continue;
                }
                f.set(px + dx, py + dy, if tag == TAG_PROP && c == 'W' { sweat } else { col }, tag);
            }
        }
    }
}

fn draw_face(f: &mut Frame, ch: &Character, rig: &Rig) {
    let k = rig.k;
    let n = k.round().max(1.0) as i32;
    let pose = &rig.pose;
    let face = &ch.face;
    let r = rig.group_r(GroupId::Head);
    let main_tag = rig.main_tag(GroupId::Head);
    // Maps a surface point to the canvas; None if it is on the back side. Returns (x, y, cos of facing).
    let put = |x: f64, y: f64| -> Option<(f64, f64, f64)> {
        let z = surface_z(r, x, y);
        let (p, phi) = rig.place(GroupId::Head, [x, y, z]);
        let c = rig.canvas(p);
        let facing = phi.cos();
        (facing > 0.15).then_some((c.0, c.1, facing))
    };
    let ex = face.gap / 2.0;
    // Cheeks first (only on top of the face).
    if face.cheeks {
        let cheek = ch.color("cheek").unwrap_or(Rgba::rgb(255, 111, 133));
        for s in [-1.0, 1.0] {
            if let Some((x, y, facing)) = put(s * (ex + 2.5 * k), face.y + 3.0 * k) {
                if facing < 0.35 {
                    continue;
                }
                let (x, y) = ((x - k).round() as i32, y.floor() as i32);
                for i in 0..2 {
                    for (dx, dy) in block(k) {
                        let (px, py) = (x + i * n + dx, y + dy);
                        if f.tag_at(px, py) == main_tag {
                            f.set(px, py, cheek, TAG_FACE);
                        }
                    }
                }
            }
        }
    }
    // Eyes.
    let ox = (pose.eye_x * k).round() as i32;
    let oy = (-pose.eye_y * 0.6 * k).round() as i32;
    for s in [-1.0, 1.0] {
        let Some((x, y, facing)) = put(s * ex, face.y) else { continue };
        let at = ((x).floor() as i32 + ox, (y).floor() as i32 + oy);
        let narrow = facing < 0.5;
        let pat = match pose.eyes {
            Eyes::Open => match face.eyes {
                EyeStyle::Round if !narrow => &EYE_ROUND,
                EyeStyle::Bean => &EYE_BEAN,
                _ if narrow => &EYE_NARROW,
                _ => &EYE_OPEN,
            },
            Eyes::Closed => &EYE_CLOSED,
            Eyes::Happy => &EYE_HAPPY,
            Eyes::Squeeze => {
                if s < 0.0 {
                    &EYE_SQUEEZE_L
                } else {
                    &EYE_SQUEEZE_R
                }
            }
            Eyes::Focus => &EYE_FOCUS,
        };
        stamp(f, ch, pat, at, k, TAG_EYE);
        let white = ch.color("white").unwrap_or(Rgba::rgb(255, 255, 255));
        let hl = |f: &mut Frame, hx: i32, hy: i32| {
            for (dx, dy) in block(k) {
                f.set(hx + dx, hy + dy, white, TAG_EYE);
            }
        };
        match (pose.eyes, face.eyes) {
            (Eyes::Open, EyeStyle::Round) if !narrow => {
                // The 2×2 pupil shifts inside the white toward the look direction.
                let px = (pose.eye_x * 1.2).round().clamp(-1.0, 1.0) as i32;
                let py = (-pose.eye_y * 1.2).round().clamp(-1.0, 1.0) as i32;
                for j in 0..2 {
                    for i in 0..2 {
                        for (dx, dy) in block(k) {
                            f.set(at.0 + (i - 1 + px) * n + dx, at.1 + (j - 1 + py) * n + dy, face.eye_color, TAG_EYE);
                        }
                    }
                }
                hl(f, at.0 + (px - 1) * n, at.1 + (py - 1) * n);
            }
            (Eyes::Open, EyeStyle::Bean) => {}
            (Eyes::Open, style) => {
                let w0 = if narrow { 0 } else { -1 };
                hl(f, at.0 + w0 * n, at.1 - n);
                if style == EyeStyle::Sparkle && !narrow {
                    hl(f, at.0, at.1 + n);
                }
            }
            (Eyes::Focus, _) => {
                let hx = (1.0 + pose.eye_x * 1.5).round().clamp(0.0, 3.0) as i32;
                hl(f, at.0 + (hx - 2) * n, at.1);
            }
            _ => {}
        }
        if face.brows {
            let pat = match (pose.brows, s < 0.0) {
                (Brows::Worried, true) => &BROW_WORRIED_L,
                (Brows::Worried, false) => &BROW_WORRIED_R,
                (_, true) => &BROW_REST_L,
                (_, false) => &BROW_REST_R,
            };
            let lift = if pose.brows == Brows::Raised && s > 0.0 { 5 } else { 4 };
            stamp(f, ch, pat, (at.0, at.1 - lift * n), k, TAG_FACE);
        }
    }
    // Mouth.
    if face.mouth != MouthStyle::None {
        if let Some((x, y, _)) = put(0.0, face.y + face.mouth_drop) {
            let pat = match pose.mouth {
                Mouth::Rest => match face.mouth {
                    MouthStyle::Curve => &MOUTH_CURVE,
                    MouthStyle::Cat => &MOUTH_CAT,
                    _ => &MOUTH_SMILE,
                },
                Mouth::Open => &MOUTH_OPEN,
                Mouth::Flat => &MOUTH_FLAT,
                Mouth::Frown => &MOUTH_FROWN,
                Mouth::O => &MOUTH_O,
            };
            stamp(f, ch, pat, (x.floor() as i32 + (pose.eye_x * 0.5 * k).round() as i32, y.floor() as i32), k, TAG_FACE);
        }
    }
}

/// Sweat drop: beside the face, touching the body.
fn draw_sweat(f: &mut Frame, ch: &Character, rig: &Rig) {
    let k = rig.k;
    let r = rig.group_r(GroupId::Head);
    let (x, y) = (ch.face.gap / 2.0 + 4.5 * k, ch.face.y - 4.5 * k);
    let (x, y) = (x.min(r[0] * 0.8), y.max(-r[1] * 0.8));
    let z = surface_z(r, x, y);
    let (p, _) = rig.place(GroupId::Head, [x, y, z]);
    let c = rig.canvas(p);
    stamp(f, ch, &SWEAT, (c.0.floor() as i32, c.1.floor() as i32), k, TAG_PROP);
}

/// Laptop (work row): in front of the body, its base below the body; hands on the keyboard.
fn draw_laptop(f: &mut Frame, ch: &Character, rig: &Rig, phase: u32) {
    let k = rig.k;
    let (c, _) = rig.place(GroupId::Body, [0.0, 0.0, 0.0]);
    let c = rig.canvas(c);
    let bottom = c.1 + rig.body_r[1] * rig.sy;
    let ground = rig.origin.1 - 1.0 * k;
    let base = bottom.min(ground).floor() as i32;
    let top = base - (7.0 * k).round() as i32;
    let x0 = (c.0 - 8.0 * k).round() as i32;
    let x1 = (c.0 + 7.0 * k).round() as i32 - 1;
    let ol = ch.color("outline").unwrap_or(Rgba::rgb(0, 0, 0));
    let lt = ch.tone("laptop").unwrap_or(Tone::flat(Rgba::rgb(0xA9, 0xB1, 0xC4)));
    let white = ch.color("white").unwrap_or(Rgba::rgb(255, 255, 255));
    let n = k.round().max(1.0) as i32;
    for y in (top - n)..=base {
        for x in (x0 - n)..=(x1 + n) {
            f.set(x, y, ol, TAG_PROP);
        }
    }
    for y in top..=(base - 2 * n) {
        for x in x0..=x1 {
            f.set(x, y, lt.base, TAG_PROP);
        }
    }
    for y in (base - 2 * n + 1)..=(base - n) {
        for x in x0..=x1 {
            f.set(x, y, lt.shade, TAG_PROP);
        }
    }
    let mx = ((x0 + x1) as f64 / 2.0).floor() as i32;
    for y in 0..(2 * n) {
        for x in 0..(2 * n) {
            f.set(mx - n + 1 + x, top + 2 * n + y, white, TAG_PROP);
        }
    }
    // Hands: press keys in turn. An armless pet (`arms: none` or `show: never`) gets no hands.
    let Some(arm) = ch.parts.iter().find(|p| p.role == Role::Arm && p.show != Some(Show::Never)) else { return };
    let tone = arm.color.as_deref().and_then(|c| ch.tone(c).ok()).unwrap_or_else(|| ch.tone("body").unwrap());
    let (l, r) = if phase % 2 == 0 { (-1.0, 0.0) } else { (0.0, -1.0) };
    for (hx, dy) in [(x0 as f64 + 2.0 * k, l), (x1 as f64 - 1.0 * k, r)] {
        let subs = vec![Sub {
            geo: Geo::Ellipse { c: (hx + 0.5, top as f64 - 0.0 * k + dy * k), rx: 2.2 * k, ry: 1.6 * k, rot: 0.0 },
            tone,
            shading: Shading::Ball,
        }];
        paint_shape(f, &subs, true, None, TAG_PROP, ol);
    }
}
