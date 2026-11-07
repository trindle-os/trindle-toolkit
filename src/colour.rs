//! Colours as Settings and the style write them: `#rgb`, `#rrggbb`, `#rrggbbaa`, `rgb(r, g, b)` and `rgba(r, g, b, a)`, plus a few plain names.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colour {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Colour {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Colour {
        Colour { r, g, b, a: 1.0 }
    }

    pub fn parse(text: &str) -> Option<Colour> {
        let t = text.trim().to_ascii_lowercase();
        if let Some(hex) = t.strip_prefix('#') {
            return hex_colour(hex);
        }
        if let Some(inner) = t.strip_prefix("rgba(").or_else(|| t.strip_prefix("rgb(")).and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
            let byte = |s: &str| s.parse::<f32>().ok().filter(|v| (0.0..=255.0).contains(v)).map(|v| v / 255.0);
            return match parts.as_slice() {
                [r, g, b] => Some(Colour::rgb(byte(r)?, byte(g)?, byte(b)?)),
                [r, g, b, a] => {
                    let a = a.parse::<f32>().ok().filter(|v| (0.0..=1.0).contains(v))?;
                    Some(Colour { a, ..Colour::rgb(byte(r)?, byte(g)?, byte(b)?) })
                }
                _ => None,
            };
        }
        match t.as_str() {
            "black" => Some(Colour::rgb(0.0, 0.0, 0.0)),
            "white" => Some(Colour::rgb(1.0, 1.0, 1.0)),
            "transparent" => Some(Colour { a: 0.0, ..Colour::rgb(0.0, 0.0, 0.0) }),
            _ => None,
        }
    }

    pub fn css(self) -> String {
        let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        let (r, g, b) = (byte(self.r), byte(self.g), byte(self.b));
        if self.a >= 1.0 {
            format!("#{r:02x}{g:02x}{b:02x}")
        } else {
            format!("rgba({r}, {g}, {b}, {})", (self.a.clamp(0.0, 1.0) * 1000.0).round() / 1000.0)
        }
    }

    #[allow(clippy::manual_clamp)]
    pub fn to_skia(self) -> tiny_skia::Color {
        let unit = |v: f32| v.max(0.0).min(1.0);
        tiny_skia::Color::from_rgba(unit(self.r), unit(self.g), unit(self.b), unit(self.a)).unwrap_or(tiny_skia::Color::BLACK)
    }
}

pub fn paint(c: Colour) -> tiny_skia::Paint<'static> {
    let mut p = tiny_skia::Paint::default();
    p.set_color(c.to_skia());
    p.anti_alias = true;
    p
}

fn hex_colour(hex: &str) -> Option<Colour> {
    if !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let digit = |i: usize| u8::from_str_radix(&hex[i..=i], 16).ok().map(|d| d * 17);
    let pair = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    let (r, g, b, a) = match hex.len() {
        3 => (digit(0)?, digit(1)?, digit(2)?, 255),
        4 => (digit(0)?, digit(1)?, digit(2)?, digit(3)?),
        6 => (pair(0)?, pair(2)?, pair(4)?, 255),
        8 => (pair(0)?, pair(2)?, pair(4)?, pair(6)?),
        _ => return None,
    };
    let f = |v: u8| v as f32 / 255.0;
    Some(Colour { r: f(r), g: f(g), b: f(b), a: f(a) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_part_out_of_range_is_drawn_clamped_not_black() {
        let c = Colour { r: 2.0, g: 0.5, b: -1.0, a: 1.0 }.to_skia();
        assert_eq!((c.red(), c.blue()), (1.0, 0.0));
    }

    #[test]
    fn every_way_settings_writes_a_colour_reads() {
        let c = Colour::parse("#09090b").unwrap();
        assert_eq!((c.r * 255.0).round() as u8, 9);
        assert_eq!((c.b * 255.0).round() as u8, 11);
        assert_eq!(Colour::parse("#fff"), Some(Colour::rgb(1.0, 1.0, 1.0)));
        assert_eq!(Colour::parse(" RGB(255, 0, 0) "), Some(Colour::rgb(1.0, 0.0, 0.0)));
        assert_eq!(Colour::parse("rgba(0,0,0,0.5)").map(|c| c.a), Some(0.5));
        assert_eq!(Colour::parse("#00000080").map(|c| (c.a * 255.0).round() as u8), Some(128));
        assert_eq!(Colour::parse("black"), Some(Colour::rgb(0.0, 0.0, 0.0)));
    }

    #[test]
    fn a_colour_writes_itself_as_css() {
        assert_eq!(Colour::parse("#1c1e26").unwrap().css(), "#1c1e26");
        assert_eq!(Colour::parse("rgba(255, 255, 255, 0.3)").unwrap().css(), "rgba(255, 255, 255, 0.3)");
    }

    #[test]
    fn nonsense_is_not_a_colour() {
        for bad in ["", "#12", "#ggg", "rgb(1,2)", "rgb(300,0,0)", "not a colour", "#ééé"] {
            assert_eq!(Colour::parse(bad), None, "{bad}");
        }
    }
}
