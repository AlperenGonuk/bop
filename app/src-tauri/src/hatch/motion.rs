//! Satırlar ve kare pozları. Satır sırası ve kare sayıları Codex v2 sözleşmesi (docs/SPRITE.md).
//! Hareketler koddan türer; spec yalnız ölçekleri (`motion`) verir.

use super::rig::{Brows, Eyes, Mouth, Pose};
use super::spec::Character;
use std::f64::consts::PI;

pub struct Row {
    pub name: &'static str,
    pub frames: usize,
    /// Codex kare süreleri (ms), yalnız bilgi ve önizleme için.
    pub durations: &'static [u32],
}

pub const ROWS: &[Row] = &[
    Row { name: "idle", frames: 6, durations: &[280, 110, 110, 140, 140, 320] },
    Row { name: "running-right", frames: 8, durations: &[120, 120, 120, 120, 120, 120, 120, 220] },
    Row { name: "running-left", frames: 8, durations: &[120, 120, 120, 120, 120, 120, 120, 220] },
    Row { name: "waving", frames: 4, durations: &[140, 140, 140, 280] },
    Row { name: "jumping", frames: 5, durations: &[140, 140, 140, 140, 280] },
    Row { name: "failed", frames: 8, durations: &[140, 140, 140, 140, 140, 140, 140, 240] },
    Row { name: "waiting", frames: 6, durations: &[150, 150, 150, 150, 150, 260] },
    Row { name: "running", frames: 6, durations: &[120, 120, 120, 120, 120, 220] },
    Row { name: "review", frames: 6, durations: &[150, 150, 150, 150, 150, 280] },
    Row { name: "look-a", frames: 8, durations: &[] },
    Row { name: "look-b", frames: 8, durations: &[] },
];

/// v2'de idle satırının 7. hücresi (0,6): nötr ön poz (Codex `neutralLookFrame`).
pub const NEUTRAL_CELL: (usize, usize) = (0, 6);
pub const LOOK_ROW: usize = 9;

/// Bakış açısı (derece) → 16 yönden biri; 0 = yukarı, saat yönünde.
pub fn look_angle(i: usize) -> f64 {
    i as f64 * 22.5
}

/// Bakış pozu: göz 1.0, kafa 0.6, gövde 0.25 oranında yönü izler; ayaklar sabit.
pub fn look(ch: &Character, deg: f64) -> Pose {
    let m = &ch.motion;
    let a = deg * PI / 180.0;
    let (h, v) = (a.sin(), a.cos());
    Pose {
        eye_x: h,
        eye_y: v,
        head_yaw: 0.6 * h * m.turn,
        body_yaw: 0.25 * h * m.turn,
        head_pitch: 0.6 * v * m.pitch,
        // Tepedeki parça yönün tersine hafifçe geride kalır.
        sway: -h * 6.0 * m.sway,
        lean: h * 1.5,
        ..Pose::default()
    }
}

/// Satırın pozları. `jump`: zıplama yüksekliği (48×52 biriminde, boşluğa göre).
pub fn row_poses(ch: &Character, row: usize, jump: f64) -> Vec<Pose> {
    let m = &ch.motion;
    let b = m.bounce;
    let sw = m.sway;
    let n = ROWS[row].frames;
    let float = ch.floats;
    let mut out = Vec::with_capacity(n);
    match ROWS[row].name {
        "idle" => {
            let sq = [0.0, 0.4, 0.8, 0.4, 0.0, 0.0];
            let sway = [0.0, 3.0, 6.0, 3.0, 0.0, -3.0];
            let fl = [0.0, -0.6, -1.2, -1.2, -0.6, 0.0];
            for i in 0..n {
                out.push(Pose {
                    squash: if float { 0.0 } else { sq[i] * b },
                    dy: if float { fl[i] * 1.5 * b } else { 0.0 },
                    sway: sway[i] * sw,
                    eyes: if i == 4 { Eyes::Closed } else { Eyes::Open },
                    ..Pose::default()
                });
            }
        }
        "running-right" | "running-left" => {
            let s = if ROWS[row].name == "running-left" { -1.0 } else { 1.0 };
            for i in 0..n {
                let t = i as f64 / n as f64 * 2.0 * PI;
                let arm = 35.0 * t.sin();
                let bob = -(t.sin().abs() * 2.0) * b;
                out.push(Pose {
                    dy: bob - if float { 2.0 } else { 0.0 },
                    feet_dy: bob * 0.8,
                    squash: 0.6 * (2.0 * t).cos() * b,
                    lean: s * m.lean * (0.85 + 0.15 * (2.0 * t).sin()),
                    body_yaw: s * 0.45 * m.turn,
                    head_yaw: s * 0.5 * m.turn,
                    eye_x: s * 0.8,
                    sway: s * (-18.0 + 6.0 * (2.0 * t).sin()) * sw,
                    mouth: Mouth::Open,
                    foot_l: (s * 4.0 * t.sin(), -(t.cos().max(0.0)) * 2.5),
                    foot_r: (-s * 4.0 * t.sin(), -((-t.cos()).max(0.0)) * 2.5),
                    arm_l: Some((25.0 - arm * s, 20.0)),
                    arm_r: Some((25.0 + arm * s, 20.0)),
                    ..Pose::default()
                });
            }
            // Gesture kolları koşuda görünmesin: kol açıları yalnız "always" kollar için.
            if !arms_always(ch) {
                for p in &mut out {
                    p.arm_l = None;
                    p.arm_r = None;
                }
            }
        }
        "waving" => {
            // Başlangıç (kol kalkar), tepe (dışa), tepe (içe), dönüş.
            let arm = [(80.0, 30.0), (150.0, -25.0), (150.0, 40.0), (100.0, 20.0)];
            let lean = [2.0, 4.0, 3.0, 1.5];
            let sway = [2.0, 6.0, -4.0, 0.0];
            let dy = [0.0, -1.0, -1.0, 0.0];
            for i in 0..n {
                out.push(Pose {
                    arm_r: Some(arm[i]),
                    lean: lean[i],
                    sway: sway[i] * sw,
                    dy: dy[i] * b,
                    squash: if i == 1 || i == 2 { -0.3 * b } else { 0.0 },
                    eyes: Eyes::Happy,
                    mouth: Mouth::Open,
                    head_roll: [0.0, 4.0, 5.0, 2.0][i],
                    ..Pose::default()
                });
            }
        }
        "jumping" => {
            let j = jump;
            let dy = [0.0, -2.0, -j, -0.65 * j, 0.0];
            let feet = [0.0, -1.0, -j + 2.0, -0.65 * j + 1.5, 0.0];
            let sq = [2.0, -1.2, -0.6, 0.0, 1.5];
            let sway = [0.0, -6.0, 12.0, -8.0, 0.0];
            let arms = [(20.0, 0.0), (60.0, 0.0), (155.0, -10.0), (120.0, 0.0), (30.0, 0.0)];
            for i in 0..n {
                let up = i == 2 || i == 3;
                out.push(Pose {
                    dy: dy[i],
                    feet_dy: feet[i],
                    squash: sq[i] * b,
                    sway: sway[i] * sw,
                    eyes: if up { Eyes::Happy } else { Eyes::Open },
                    mouth: if up { Mouth::Open } else { Mouth::Rest },
                    arm_l: (up || arms_always(ch)).then_some(arms[i]),
                    arm_r: (up || arms_always(ch)).then_some(arms[i]),
                    ..Pose::default()
                });
            }
        }
        "failed" => {
            for i in 0..n {
                let s = (i.min(3)) as f64;
                let sad = i >= 2;
                out.push(Pose {
                    squash: s * 0.5 * b,
                    dy: if i == 0 { -1.0 * b } else { s * 0.5 * b },
                    dx: if i >= 4 { if i % 2 == 0 { 0.5 } else { -0.5 } } else { 0.0 },
                    eyes: if sad { Eyes::Squeeze } else { Eyes::Open },
                    mouth: if sad { Mouth::Frown } else { Mouth::O },
                    brows: if sad { Brows::Worried } else { Brows::Rest },
                    eye_y: if sad { -0.4 } else { 0.2 },
                    head_pitch: if sad { -0.3 * m.pitch } else { 0.0 },
                    droop: (i.min(4)) as f64 / 4.0,
                    sway: ((if i % 2 == 0 { 3.0 } else { -3.0 }) - (i.min(4)) as f64 * 2.0) * sw,
                    sweat: sad,
                    arm_l: arms_always(ch).then_some((8.0, 0.0)),
                    arm_r: arms_always(ch).then_some((8.0, 0.0)),
                    ..Pose::default()
                });
            }
        }
        "waiting" => {
            let dy = [0.0, -1.0, -1.5, -1.0, 0.0, 0.0];
            let sway = [4.0, 0.0, -2.0, 0.0, 4.0, 2.0];
            let arm = [(150.0, -10.0), (162.0, -18.0), (168.0, -22.0), (162.0, -18.0), (152.0, -10.0), (145.0, 0.0)];
            for i in 0..n {
                out.push(Pose {
                    dy: dy[i] * b,
                    squash: -0.3 * b,
                    sway: sway[i] * sw,
                    eye_y: 0.7,
                    head_pitch: 0.35 * m.pitch,
                    eyes: if i == 4 { Eyes::Closed } else { Eyes::Open },
                    mouth: Mouth::O,
                    arm_r: Some(arm[i]),
                    ..Pose::default()
                });
            }
        }
        "running" => {
            let sway = [3.0, -3.0, 2.0, -2.0, 3.0, -1.0];
            let ex = [-0.5, -0.2, 0.1, 0.4, 0.1, -0.2];
            // Dizüstü yoksa kıpırdanan kollar yalnız "always" kollarda (gesture kollar yalnız
            // waving/jumping/waiting/review'da görünür, spec-format.md).
            let fidget = !ch.laptop && arms_always(ch);
            for i in 0..n {
                out.push(Pose {
                    eyes: Eyes::Focus,
                    eye_x: ex[i],
                    eye_y: -0.6,
                    head_pitch: -0.3 * m.pitch,
                    mouth: Mouth::Flat,
                    sway: sway[i] * sw,
                    dy: if i % 3 == 2 { -0.5 * b } else { 0.0 },
                    laptop: if ch.laptop { Some(i as u32) } else { None },
                    arm_l: fidget.then_some((-40.0 + (i % 2) as f64 * 15.0, 40.0)),
                    arm_r: fidget.then_some((-40.0 + ((i + 1) % 2) as f64 * 15.0, 40.0)),
                    ..Pose::default()
                });
            }
        }
        "review" => {
            let ex = [-0.7, -0.3, 0.2, 0.7, 0.3, -0.2];
            for i in 0..n {
                out.push(Pose {
                    eyes: Eyes::Focus,
                    eye_x: ex[i],
                    eye_y: 0.1,
                    head_yaw: 0.2 * m.turn * ex[i],
                    head_roll: 7.0,
                    body_yaw: 0.05 * m.turn * ex[i],
                    brows: Brows::Raised,
                    mouth: Mouth::Flat,
                    lean: 3.0,
                    squash: 0.2 * b,
                    sway: (5.0 + if i % 2 == 0 { 1.0 } else { -1.0 }) * sw,
                    arm_r: Some((-35.0, -60.0)),
                    ..Pose::default()
                });
            }
        }
        _ => {
            let base = if ROWS[row].name == "look-a" { 0 } else { 8 };
            for i in 0..n {
                out.push(look(ch, look_angle(base + i)));
            }
        }
    }
    out
}

fn arms_always(ch: &Character) -> bool {
    ch.parts
        .iter()
        .any(|p| p.role == super::spec::Role::Arm && p.show.unwrap_or(super::spec::Show::Always) == super::spec::Show::Always)
}
