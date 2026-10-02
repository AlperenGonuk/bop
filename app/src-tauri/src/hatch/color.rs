//! Renkler ve palet tonları (taban, gölge, ışık).

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

impl Rgba {
    pub const CLEAR: Rgba = Rgba(0, 0, 0, 0);

    pub fn rgb(r: u8, g: u8, b: u8) -> Rgba {
        Rgba(r, g, b, 255)
    }

    pub fn is_clear(self) -> bool {
        self.3 == 0
    }

    /// `#RRGGBB` ya da `#RRGGBBAA`.
    pub fn parse_hex(s: &str) -> Option<Rgba> {
        let h = s.strip_prefix('#')?;
        if !(h.len() == 6 || h.len() == 8) || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).ok();
        let a = if h.len() == 8 { byte(6)? } else { 255 };
        Some(Rgba(byte(0)?, byte(2)?, byte(4)?, a))
    }

    fn to_hsl(self) -> (f64, f64, f64) {
        let r = self.0 as f64 / 255.0;
        let g = self.1 as f64 / 255.0;
        let b = self.2 as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let l = (max + min) / 2.0;
        if (max - min).abs() < 1e-9 {
            return (0.0, 0.0, l);
        }
        let d = max - min;
        let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
        let h = if max == r {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if max == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
        (h * 60.0, s, l)
    }

    fn from_hsl(h: f64, s: f64, l: f64, a: u8) -> Rgba {
        let h = h.rem_euclid(360.0) / 360.0;
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);
        if s == 0.0 {
            let v = (l * 255.0).round() as u8;
            return Rgba(v, v, v, a);
        }
        let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let p = 2.0 * l - q;
        let ch = |t: f64| {
            let t = t.rem_euclid(1.0);
            let v = if t < 1.0 / 6.0 {
                p + (q - p) * 6.0 * t
            } else if t < 0.5 {
                q
            } else if t < 2.0 / 3.0 {
                p + (q - p) * (2.0 / 3.0 - t) * 6.0
            } else {
                p
            };
            (v * 255.0).round() as u8
        };
        Rgba(ch(h + 1.0 / 3.0), ch(h), ch(h - 1.0 / 3.0), a)
    }

    /// Gölge tonu: daha koyu, renk tonu hafif kırmızıya/mora kayar (sıcak gölge).
    pub fn auto_shade(self) -> Rgba {
        let (h, s, l) = self.to_hsl();
        let shift = if (40.0..200.0).contains(&h) { 6.0 } else { -3.0 };
        Rgba::from_hsl(h + shift, (s * 0.85).max(s - 0.2), l - 0.13, self.3)
    }

    /// Işık tonu: daha açık, biraz daha az doygun.
    pub fn auto_hi(self) -> Rgba {
        let (h, s, l) = self.to_hsl();
        Rgba::from_hsl(h - 2.0, s, l + (1.0 - l) * 0.5, self.3)
    }

    /// Algısal parlaklık (0–255).
    #[cfg(test)]
    pub fn luma(self) -> f64 {
        0.299 * self.0 as f64 + 0.587 * self.1 as f64 + 0.114 * self.2 as f64
    }
}

/// Bir palet girdisinin üç tonu.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tone {
    pub base: Rgba,
    pub shade: Rgba,
    pub hi: Rgba,
}

impl Tone {
    pub fn auto(base: Rgba) -> Tone {
        Tone { base, shade: base.auto_shade(), hi: base.auto_hi() }
    }
    /// Tek renkli (gölgesiz) ton.
    pub fn flat(c: Rgba) -> Tone {
        Tone { base: c, shade: c, hi: c }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_okunur() {
        assert_eq!(Rgba::parse_hex("#FFB066"), Some(Rgba(255, 176, 102, 255)));
        assert_eq!(Rgba::parse_hex("#ffb06680"), Some(Rgba(255, 176, 102, 128)));
        assert_eq!(Rgba::parse_hex("FFB066"), None);
        assert_eq!(Rgba::parse_hex("#FFB06"), None);
        assert_eq!(Rgba::parse_hex("#GGGGGG"), None);
    }

    #[test]
    fn golge_koyu_isik_acik() {
        let c = Rgba::rgb(0xFF, 0xB0, 0x66);
        assert!(c.auto_shade().luma() < c.luma());
        assert!(c.auto_hi().luma() > c.luma());
    }
}
