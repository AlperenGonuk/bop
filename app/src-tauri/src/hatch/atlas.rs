//! Atlas (1536×2288, 8×11 cells), PNG read/write and the labeled contact sheet for QA.

use super::color::Rgba;
use super::motion::{look_angle, LOOK_ROW, NEUTRAL_CELL, ROWS};
use super::rig::Frame;
use super::validate::{Issue, Level};

pub const CELL_W: usize = 192;
pub const CELL_H: usize = 208;
pub const COLS: usize = 8;
pub const ROWS_V2: usize = 11;
pub const SHEET_W: usize = CELL_W * COLS;
pub const SHEET_H: usize = CELL_H * ROWS_V2;

/// RGBA byte buffer.
pub struct Image {
    pub w: usize,
    pub h: usize,
    pub data: Vec<u8>,
}

impl Image {
    pub fn new(w: usize, h: usize) -> Image {
        Image { w, h, data: vec![0; w * h * 4] }
    }
    pub fn filled(w: usize, h: usize, c: Rgba) -> Image {
        let mut img = Image::new(w, h);
        for p in img.data.chunks_exact_mut(4) {
            p.copy_from_slice(&[c.0, c.1, c.2, c.3]);
        }
        img
    }
    pub fn get(&self, x: usize, y: usize) -> Rgba {
        let i = (y * self.w + x) * 4;
        Rgba(self.data[i], self.data[i + 1], self.data[i + 2], self.data[i + 3])
    }
    pub fn put(&mut self, x: i64, y: i64, c: Rgba) {
        if x < 0 || y < 0 || x as usize >= self.w || y as usize >= self.h {
            return;
        }
        let i = (y as usize * self.w + x as usize) * 4;
        self.data[i..i + 4].copy_from_slice(&[c.0, c.1, c.2, c.3]);
    }
    pub fn rect(&mut self, x: i64, y: i64, w: i64, h: i64, c: Rgba) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.put(xx, yy, c);
            }
        }
    }
}

/// Scales a logical frame up into a cell (nearest neighbor).
pub fn blit(sheet: &mut Image, col: usize, row: usize, f: &Frame, scale: usize) {
    let (ox, oy) = (col * CELL_W, row * CELL_H);
    for y in 0..f.h {
        for x in 0..f.w {
            let c = f.px[y * f.w + x];
            if c.is_clear() {
                continue;
            }
            for j in 0..scale {
                for i in 0..scale {
                    sheet.put((ox + x * scale + i) as i64, (oy + y * scale + j) as i64, c);
                }
            }
        }
    }
}

pub fn write_png(path: &std::path::Path, img: &Image) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| format!("could not write {}: {e}", path.display()))?;
    let w = std::io::BufWriter::new(file);
    let mut enc = png::Encoder::new(w, img.w as u32, img.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| format!("png: {e}"))?;
    writer.write_image_data(&img.data).map_err(|e| format!("png: {e}"))?;
    writer.finish().map_err(|e| format!("png: {e}"))
}

/// Reads a PNG as RGBA (8-bit RGBA, RGB or grayscale; others are rejected).
pub fn read_png(path: &std::path::Path) -> Result<Image, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("could not read {}: {e}", path.display()))?;
    let mut dec = png::Decoder::new(std::io::BufReader::new(file));
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = dec.read_info().map_err(|e| format!("png: {e}"))?;
    let size = reader.output_buffer_size().ok_or("png: image too large")?;
    let mut buf = vec![0; size];
    let info = reader.next_frame(&mut buf).map_err(|e| format!("png: {e}"))?;
    let (w, h) = (info.width as usize, info.height as usize);
    if info.bit_depth != png::BitDepth::Eight {
        return Err("png: only 8-bit images are supported".into());
    }
    let data = match info.color_type {
        png::ColorType::Rgba => buf[..w * h * 4].to_vec(),
        png::ColorType::Rgb => buf[..w * h * 3].chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => buf[..w * h * 2].chunks_exact(2).flat_map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        png::ColorType::Grayscale => buf[..w * h].iter().flat_map(|&g| [g, g, g, 255]).collect(),
        other => return Err(format!("png: unsupported color type {other:?}")),
    };
    Ok(Image { w, h, data })
}

// --- Tiny font (3×5) ----------------------------------------------------------------

fn glyph(c: char) -> [u8; 5] {
    let rows: &str = match c.to_ascii_uppercase() {
        '0' => "111101101101111",
        '1' => "010110010010111",
        '2' => "111001111100111",
        '3' => "111001111001111",
        '4' => "101101111001001",
        '5' => "111100111001111",
        '6' => "111100111101111",
        '7' => "111001010010010",
        '8' => "111101111101111",
        '9' => "111101111001111",
        'A' => "010101111101101",
        'B' => "110101110101110",
        'C' => "011100100100011",
        'D' => "110101101101110",
        'E' => "111100110100111",
        'F' => "111100110100100",
        'G' => "011100101101011",
        'H' => "101101111101101",
        'I' => "111010010010111",
        'J' => "001001001101010",
        'K' => "101101110101101",
        'L' => "100100100100111",
        'M' => "101111111101101",
        'N' => "110101101101101",
        'O' => "010101101101010",
        'P' => "110101110100100",
        'Q' => "010101101110011",
        'R' => "110101110101101",
        'S' => "011100010001110",
        'T' => "111010010010010",
        'U' => "101101101101111",
        'V' => "101101101101010",
        'W' => "101101111111101",
        'X' => "101101010101101",
        'Y' => "101101010010010",
        'Z' => "111001010100111",
        '-' => "000000111000000",
        '.' => "000000000000010",
        ':' => "000010000010000",
        '(' => "010100100100010",
        ')' => "010001001001010",
        '/' => "001001010100100",
        '!' => "010010010000010",
        _ => "000000000000000",
    };
    let b = rows.as_bytes();
    let mut g = [0u8; 5];
    for (r, item) in g.iter_mut().enumerate() {
        *item = (b[r * 3] - b'0') << 2 | (b[r * 3 + 1] - b'0') << 1 | (b[r * 3 + 2] - b'0');
    }
    g
}

pub fn text(img: &mut Image, x: i64, y: i64, s: &str, scale: i64, c: Rgba) {
    let mut cx = x;
    for ch in s.chars() {
        let g = glyph(ch);
        for (r, bits) in g.iter().enumerate() {
            for col in 0..3 {
                if bits & (4 >> col) != 0 {
                    img.rect(cx + col * scale, y + r as i64 * scale, scale, scale, c);
                }
            }
        }
        cx += 4 * scale;
    }
}

const LABEL_W: usize = 170;
const HEADER_H: usize = 34;

/// Contact sheet: light background, grid, row names, frame numbers, look angles,
/// ground line; cells with errors get a red frame, cells with warnings an orange one.
pub fn contact_sheet(sheet: &Image, issues: &[Issue], ground_y: Option<usize>) -> Image {
    let bg = Rgba::rgb(0xEE, 0xEC, 0xF4);
    let cell_bg = Rgba::rgb(0xF8, 0xF7, 0xFC);
    let grid = Rgba::rgb(0xC9, 0xC4, 0xD8);
    let ink = Rgba::rgb(0x3A, 0x34, 0x4E);
    let faint = Rgba::rgb(0x8C, 0x86, 0xA0);
    let hatch = Rgba::rgb(0xE2, 0xDE, 0xEC);
    let ground = Rgba::rgb(0xDD, 0xD7, 0xEA);
    let (w, h) = (LABEL_W + SHEET_W, HEADER_H + sheet.h.max(CELL_H));
    let mut out = Image::filled(w, h, bg);
    let rows = sheet.h / CELL_H;
    for c in 0..COLS {
        text(&mut out, (LABEL_W + c * CELL_W + CELL_W / 2 - 6) as i64, 8, &c.to_string(), 4, ink);
    }
    for r in 0..rows {
        let name = ROWS.get(r).map(|x| x.name).unwrap_or("extra");
        let oy = (HEADER_H + r * CELL_H) as i64;
        text(&mut out, 8, oy + 80, &r.to_string(), 4, ink);
        let label = name.to_uppercase();
        text(&mut out, 8, oy + 110, &label[..label.len().min(13)], 3, ink);
        let expected = ROWS.get(r).map(|x| x.frames).unwrap_or(0);
        for c in 0..COLS {
            let ox = (LABEL_W + c * CELL_W) as i64;
            let used = c < expected || (r, c) == NEUTRAL_CELL;
            if used {
                out.rect(ox, oy, CELL_W as i64, CELL_H as i64, cell_bg);
                if let Some(g) = ground_y {
                    out.rect(ox, oy + g as i64, CELL_W as i64, 2, ground);
                }
            } else {
                for i in (-(CELL_H as i64)..CELL_W as i64).step_by(14) {
                    for t in 0..CELL_H as i64 {
                        if (0..CELL_W as i64).contains(&(i + t)) {
                            out.put(ox + i + t, oy + t, hatch);
                        }
                    }
                }
            }
            for y in 0..CELL_H {
                for x in 0..CELL_W {
                    let p = sheet.get(c * CELL_W + x, r * CELL_H + y);
                    if p.3 > 0 {
                        let base = out.get(ox as usize + x, oy as usize + y);
                        out.put(ox + x as i64, oy + y as i64, over(base, p));
                    }
                }
            }
            let tag = if (r, c) == NEUTRAL_CELL {
                Some("N".to_string())
            } else if r >= LOOK_ROW && r < LOOK_ROW + 2 {
                Some(format!("{}", look_angle((r - LOOK_ROW) * 8 + c)))
            } else {
                None
            };
            if let Some(t) = tag {
                text(&mut out, ox + 5, oy + 5, &t, 3, faint);
            }
        }
    }
    // Grid lines.
    for r in 0..=rows {
        out.rect(LABEL_W as i64, (HEADER_H + r * CELL_H) as i64, SHEET_W as i64, 1, grid);
    }
    for c in 0..=COLS {
        out.rect((LABEL_W + c * CELL_W) as i64, HEADER_H as i64, 1, (rows * CELL_H) as i64, grid);
    }
    // Cells with issues.
    for level in [Level::Warning, Level::Error] {
        let color = if level == Level::Error { Rgba::rgb(0xE0, 0x38, 0x3E) } else { Rgba::rgb(0xF0, 0xA0, 0x20) };
        for i in issues.iter().filter(|i| i.level == level) {
            if let (Some(r), Some(c)) = (i.row, i.frame) {
                let (ox, oy) = ((LABEL_W + c * CELL_W) as i64, (HEADER_H + r * CELL_H) as i64);
                for t in 0..3 {
                    out.rect(ox + t, oy + t, CELL_W as i64 - 2 * t, 1, color);
                    out.rect(ox + t, oy + CELL_H as i64 - 1 - t, CELL_W as i64 - 2 * t, 1, color);
                    out.rect(ox + t, oy + t, 1, CELL_H as i64 - 2 * t, color);
                    out.rect(ox + CELL_W as i64 - 1 - t, oy + t, 1, CELL_H as i64 - 2 * t, color);
                }
            }
        }
    }
    out
}

fn over(base: Rgba, top: Rgba) -> Rgba {
    let a = top.3 as u32;
    let mix = |b: u8, t: u8| ((t as u32 * a + b as u32 * (255 - a)) / 255) as u8;
    Rgba(mix(base.0, top.0), mix(base.1, top.1), mix(base.2, top.2), 255)
}
