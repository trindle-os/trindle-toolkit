//! The plain pictures a program with no icon of its own is drawn as (`trindle-generic-01` to `-20`).

use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

use super::theme::{self, Palette};
use super::ui::rounded;
use super::widget::Rect;

const TILES: [Palette; 10] = [
    Palette::Pink, Palette::Purple, Palette::Indigo, Palette::Blue, Palette::Cyan,
    Palette::Teal, Palette::DeepPurple, Palette::Orange, Palette::Brown, Palette::BlueGrey,
];

const MARGIN: f32 = 0.06;
const ROUND: f32 = 0.22;

pub const NAMES: [&str; 20] = [
    "trindle-generic-01", "trindle-generic-02", "trindle-generic-03", "trindle-generic-04", "trindle-generic-05",
    "trindle-generic-06", "trindle-generic-07", "trindle-generic-08", "trindle-generic-09", "trindle-generic-10",
    "trindle-generic-11", "trindle-generic-12", "trindle-generic-13", "trindle-generic-14", "trindle-generic-15",
    "trindle-generic-16", "trindle-generic-17", "trindle-generic-18", "trindle-generic-19", "trindle-generic-20",
];

pub fn name_for(key: &str) -> &'static str {
    let key = key.strip_suffix(".desktop").unwrap_or(key).to_lowercase();
    let mut h: u32 = 0x811c_9dc5;
    for b in key.bytes() {
        h = (h ^ b as u32).wrapping_mul(0x0100_0193);
    }
    NAMES[(h % NAMES.len() as u32) as usize]
}

pub fn name_with_who(id: &str, name: &str) -> String {
    let clean = |s: &str| s.chars().map(|c| if c.is_control() { ' ' } else { c }).collect::<String>();
    format!("{}?{}\t{}", name_for(id), clean(id), clean(name))
}

pub fn number(name: &str) -> Option<u32> {
    let (name, _) = split(name);
    let digits = name.strip_prefix("trindle-generic-")?;
    if digits.len() != 2 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u32 = digits.parse().ok()?;
    (1..=20).contains(&n).then_some(n)
}

pub fn split(name: &str) -> (&str, Option<(&str, &str)>) {
    match name.split_once('?') {
        Some((n, who)) if n.starts_with("trindle-generic-") => (n, who.split_once('\t').or(Some((who, who)))),
        _ => (name, None),
    }
}

pub fn parts(n: u32) -> (usize, Palette) {
    let i = n.saturating_sub(1) as usize;
    (i % 10, TILES[(i * 3 + i / 10) % 10])
}

pub fn paint(n: u32, px: u32) -> Option<Pixmap> {
    let mut out = Pixmap::new(px.max(1), px.max(1))?;
    let (shape, tile) = parts(n);
    let side = px.max(1) as f32;
    let tile = tile.swatch();
    let ink = theme::on_as_written(tile);
    let (m, w) = (side * MARGIN, side * (1.0 - 2.0 * MARGIN));
    let mut paint = Paint { anti_alias: true, ..Paint::default() };
    paint.set_color(tile.to_skia());
    if let Some(p) = rounded(Rect { x: m, y: m, w, h: w }, w * ROUND) {
        out.fill_path(&p, &paint, FillRule::Winding, Transform::identity(), None);
    }
    paint.set_color(ink.to_skia());
    glyph(&mut out, &paint, shape, side);
    Some(out)
}

fn circle(b: &mut PathBuilder, cx: f32, cy: f32, r: f32) {
    if let Some(p) = PathBuilder::from_circle(cx, cy, r) {
        b.push_path(&p);
    }
}

fn square(b: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32) {
    if let Some(r) = tiny_skia::Rect::from_xywh(x, y, w, h) {
        b.push_rect(r);
    }
}

fn glyph(out: &mut Pixmap, paint: &Paint, shape: usize, side: f32) {
    let c = side / 2.0;
    let u = side * 0.5;
    let mut b = PathBuilder::new();
    let ring = shape_path(&mut b, shape, c, u);
    let Some(path) = b.finish() else { return };
    if ring {
        let stroke = Stroke { width: u * 0.16, ..Stroke::default() };
        out.stroke_path(&path, paint, &stroke, Transform::identity(), None);
    } else {
        out.fill_path(&path, paint, FillRule::Winding, Transform::identity(), None);
    }
}

fn shape_path(b: &mut PathBuilder, shape: usize, c: f32, u: f32) -> bool {
    let mut ring = false;
    match shape {
        0 => circle(b, c, c, u * 0.5),
        1 => {
            square(b, c - u * 0.42, c - u * 0.42, u * 0.84, u * 0.84);
        }
        2 => {
            b.move_to(c, c - u * 0.5);
            b.line_to(c + u * 0.5, c + u * 0.42);
            b.line_to(c - u * 0.5, c + u * 0.42);
            b.close();
        }
        3 => {
            b.move_to(c, c - u * 0.55);
            b.line_to(c + u * 0.55, c);
            b.line_to(c, c + u * 0.55);
            b.line_to(c - u * 0.55, c);
            b.close();
        }
        4 => {
            circle(b, c, c, u * 0.42);
            ring = true;
        }
        5 => {
            let (t, l) = (u * 0.2, u * 0.55);
            for (x, y, w, h) in [(c - t, c - l, t * 2.0, l * 2.0), (c - l, c - t, l * 2.0, t * 2.0)] {
                square(b, x, y, w, h);
            }
        }
        6 => {
            for k in [-1.0f32, 0.0, 1.0] {
                square(b, c - u * 0.5, c + k * u * 0.36 - u * 0.1, u, u * 0.2);
            }
        }
        7 => {
            b.move_to(c - u * 0.5, c + u * 0.25);
            b.cubic_to(c - u * 0.5, c - u * 0.5, c + u * 0.5, c - u * 0.5, c + u * 0.5, c + u * 0.25);
            b.close();
        }
        8 => {
            for k in 0..6 {
                let a = std::f32::consts::FRAC_PI_3 * k as f32 + std::f32::consts::FRAC_PI_6;
                let (x, y) = (c + a.cos() * u * 0.55, c + a.sin() * u * 0.55);
                if k == 0 { b.move_to(x, y) } else { b.line_to(x, y) }
            }
            b.close();
        }
        _ => {
            for (dx, dy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                circle(b, c + dx * u * 0.27, c + dy * u * 0.27, u * 0.2);
            }
        }
    }
    ring
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_twenty_differ_and_a_name_with_a_program_after_it_is_read() {
        assert_eq!(number("trindle-generic-07"), Some(7));
        assert_eq!(number("trindle-generic-07?org.x.Y\tY"), Some(7));
        assert_eq!(number("trindle-generic-21"), None);
        assert_eq!(number("trindle-generic-00"), None);
        assert_eq!(number("firefox"), None);
        assert_eq!((number("trindle-generic-+7"), number("trindle-generic-7"), number("trindle-generic-007")), (None, None, None));
        assert_eq!(split("trindle-generic-07?org.x.Y\tWhy"), ("trindle-generic-07", Some(("org.x.Y", "Why"))));
        assert_eq!(split("a?b"), ("a?b", None));
        let pairs: std::collections::HashSet<_> = (1..=20).map(|n| { let (s, c) = parts(n); (s, c as usize) }).collect();
        assert_eq!(pairs.len(), 20, "shape and colour together tell the twenty apart");
        let pictures: std::collections::HashSet<_> = (1..=20).map(|n| paint(n, 48).unwrap().data().to_vec()).collect();
        assert_eq!(pictures.len(), 20, "twenty different drawings");
        for tile in TILES {
            let c = tile.swatch();
            let (hi, lo) = ([c.r, c.g, c.b].into_iter().fold(0.0f32, f32::max), [c.r, c.g, c.b].into_iter().fold(1.0f32, f32::min));
            if hi > lo && c.g == hi {
                let hue = 60.0 * (2.0 + (c.b - c.r) / (hi - lo));
                assert!(!(75.0..=165.0).contains(&hue), "{tile:?} reads green at hue {hue}");
            }
        }
        for n in 1..=20 {
            assert!(paint(n, 48).unwrap().data().chunks_exact(4).any(|p| p[3] == 255), "generic {n} draws something solid");
        }
    }
}
