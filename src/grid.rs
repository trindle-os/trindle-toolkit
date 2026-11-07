//! Text in a grid of cells as a terminal draws it: character widths and a layout snapped to the cells.

use super::text::{Fonts, Layout, Style};

fn joins(c: char) -> bool {
    matches!(c as u32,
        0x0300..=0x036F | 0x0483..=0x0489 | 0x0591..=0x05BD | 0x0610..=0x061A | 0x064B..=0x065F | 0x0670
        | 0x0900..=0x0902 | 0x093A | 0x093C | 0x0941..=0x0948 | 0x094D | 0x0951..=0x0957 | 0x0962..=0x0963
        | 0x0E31 | 0x0E34..=0x0E3A | 0x0E47..=0x0E4E | 0x1160..=0x11FF | 0xD7B0..=0xD7FF
        | 0x200B..=0x200F | 0x2028..=0x202E | 0x2060..=0x2064 | 0x20D0..=0x20F0
        | 0xFE00..=0xFE0F | 0xFE20..=0xFE2F | 0xE0100..=0xE01EF)
}

fn wide(c: char) -> bool {
    matches!(c as u32,
        0x1100..=0x115F | 0x231A..=0x231B | 0x2329..=0x232A | 0x23E9..=0x23EC | 0x23F0 | 0x23F3
        | 0x25FD..=0x25FE | 0x2614..=0x2615 | 0x2648..=0x2653 | 0x267F | 0x2693 | 0x26A1
        | 0x26AA..=0x26AB | 0x26BD..=0x26BE | 0x26C4..=0x26C5 | 0x26CE | 0x26D4 | 0x26EA
        | 0x26F2..=0x26F3 | 0x26F5 | 0x26FA | 0x26FD | 0x2705 | 0x270A..=0x270B | 0x2728 | 0x274C
        | 0x274E | 0x2753..=0x2755 | 0x2757 | 0x2795..=0x2797 | 0x27B0 | 0x27BF | 0x2B1B..=0x2B1C
        | 0x2B50 | 0x2B55 | 0x2E80..=0x303E | 0x3041..=0x33FF | 0x3400..=0x4DBF | 0x4E00..=0xA4CF
        | 0xA960..=0xA97F | 0xAC00..=0xD7A3 | 0xF900..=0xFAFF | 0xFE10..=0xFE19 | 0xFE30..=0xFE6F
        | 0xFF00..=0xFF60 | 0xFFE0..=0xFFE6 | 0x1F004 | 0x1F0CF | 0x1F18E | 0x1F191..=0x1F19A
        | 0x1F200..=0x1F251 | 0x1F300..=0x1F320 | 0x1F32D..=0x1F335 | 0x1F337..=0x1F37C | 0x1F37E..=0x1F393
        | 0x1F3A0..=0x1F3CA | 0x1F3CF..=0x1F3D3 | 0x1F3E0..=0x1F3F0 | 0x1F3F4 | 0x1F3F8..=0x1F43E | 0x1F440
        | 0x1F442..=0x1F4FC | 0x1F4FF..=0x1F53D | 0x1F54B..=0x1F54E | 0x1F550..=0x1F567 | 0x1F57A
        | 0x1F595..=0x1F596 | 0x1F5A4 | 0x1F5FB..=0x1F64F | 0x1F680..=0x1F6C5 | 0x1F6CC | 0x1F6D0..=0x1F6D2
        | 0x1F6D5..=0x1F6D7 | 0x1F6DC..=0x1F6DF | 0x1F6EB..=0x1F6EC | 0x1F6F4..=0x1F6FC | 0x1F7E0..=0x1F7EB
        | 0x1F7F0 | 0x1F90C..=0x1F93A | 0x1F93C..=0x1F945 | 0x1F947..=0x1F9FF | 0x1FA70..=0x1FA7C
        | 0x1FA80..=0x1FA88 | 0x1FA90..=0x1FABD | 0x1FABF..=0x1FAC5 | 0x1FACE..=0x1FADB | 0x1FAE0..=0x1FAE8
        | 0x1FAF0..=0x1FAF8 | 0x20000..=0x3FFFD)
}

pub fn cell_width(c: char) -> usize {
    if c.is_control() || joins(c) {
        0
    } else if wide(c) {
        2
    } else {
        1
    }
}

pub fn cells(text: &str) -> usize {
    widths(text).map(|(_, w)| w).sum()
}

fn widths(text: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    let mut joined = false;
    text.char_indices().map(move |(i, c)| {
        let w = if joined { 0 } else { cell_width(c) };
        joined = c == '\u{200d}';
        (i, w)
    })
}

impl Fonts {
    pub fn cell_size(&self, style: &Style) -> (f32, f32) {
        let l = self.layout("M", style, None);
        (l.width, l.height)
    }
}

impl Layout {
    pub fn snapped(&self, text: &str, cell: f32) -> Layout {
        let mut at = Vec::new();
        let mut col = 0;
        for (i, w) in widths(text) {
            if w > 0 {
                at.push((i, col));
            }
            col += w;
        }
        self.snapped_to(cell, col, |byte| {
            let i = at.partition_point(|&(b, _)| b <= byte).saturating_sub(1);
            at.get(i).map_or(0, |&(_, c)| c)
        })
    }

    pub fn snapped_to(&self, cell: f32, columns: usize, column: impl Fn(usize) -> usize) -> Layout {
        let mut out = self.clone();
        for line in &mut out.lines {
            let mut group: Option<(usize, f32, f32)> = None;
            for g in &mut line.glyphs {
                let col = column(g.at);
                match group {
                    Some((c, old, new)) if c == col => g.x = new + (g.x - old),
                    _ => {
                        let (old, new) = (g.x, col as f32 * cell);
                        group = Some((col, old, new));
                        g.x = new;
                    }
                }
            }
            line.x = 0.0;
            line.width = columns as f32 * cell;
        }
        out.width = columns as f32 * cell;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_characters_take_two_cells_and_marks_none() {
        assert_eq!(cells("abc"), 3);
        assert_eq!(cells("日本語"), 6);
        assert_eq!(cells("한글"), 4);
        assert_eq!(cells("😀"), 2);
        assert_eq!(cells("e\u{301}"), 1);
        assert_eq!(cell_width('\u{200d}'), 0);
        assert_eq!(cell_width('ｱ'), 1);
        assert_eq!(cell_width('Ａ'), 2);
        assert_eq!(cells("👨\u{200d}👩\u{200d}👧"), 2, "a family is one picture");
        assert_eq!((cell_width('\u{1F321}'), cell_width('\u{1F680}'), cell_width('\u{1160}'), cell_width('\u{0E31}')), (1, 2, 0, 0));
    }

    #[test]
    fn columns_after_a_wide_character_line_up() {
        let fonts = Fonts::new();
        let style = Style::monospace(14.0);
        let (cell, tall) = fonts.cell_size(&style);
        assert!(cell > 3.0 && tall > cell);
        let text = "a日b😀c";
        let laid = fonts.layout(text, &style, None).snapped(text, cell);
        let xs: Vec<f32> = laid.lines[0].glyphs.iter().map(|g| g.x).collect();
        let cols = [0.0, 1.0, 3.0, 4.0, 6.0];
        assert_eq!(xs.len(), cols.len());
        for (x, c) in xs.iter().zip(cols) {
            assert!((x - c * cell).abs() < 0.01, "{xs:?} against {cell}");
        }
        assert!((laid.width - 7.0 * cell).abs() < 0.01);
    }

    #[test]
    fn a_mark_stays_with_its_letter() {
        let fonts = Fonts::new();
        let style = Style::monospace(14.0);
        let (cell, _) = fonts.cell_size(&style);
        let text = "xq\u{301}y";
        let laid = fonts.layout(text, &style, None).snapped(text, cell);
        let y = laid.lines[0].glyphs.iter().find(|g| g.at == 4).unwrap();
        assert!((y.x - 2.0 * cell).abs() < 0.01);
        let joined: Vec<_> = laid.lines[0].glyphs.iter().filter(|g| (1..4).contains(&g.at)).collect();
        assert!(!joined.is_empty());
        for g in joined {
            assert!((g.x - cell).abs() < cell, "the mark is on the letter's cell, not the next: {}", g.x);
        }
    }

    #[test]
    fn the_monospace_family_is_a_fixed_width_font() {
        let fonts = Fonts::new();
        let name = fonts.monospace().to_string();
        assert!(!name.is_empty());
        let narrow = fonts.layout("iiiiiiii", &Style::monospace(14.0), None).width;
        let broad = fonts.layout("MMMMMMMM", &Style::monospace(14.0), None).width;
        assert!((narrow - broad).abs() < 0.5, "{name}: {narrow} against {broad}");
        let spelt = Style { family: Some("Monospace".into()), ..Style::new(14.0) };
        assert!((fonts.layout("MMMMMMMM", &spelt, None).width - broad).abs() < 0.01);
    }

    #[test]
    #[ignore = "a timing run, for a machine with nothing else busy"]
    fn a_screen_of_3000_cells_is_laid_out_in_time() {
        let fonts = Fonts::new();
        let style = Style::monospace(14.0);
        let (cell, _) = fonts.cell_size(&style);
        let row: String = "the quick brown fox jumps over the lazy dog 0123456789 .:;".chars().take(60).collect();
        let timed = |name: &str, f: &dyn Fn()| {
            f();
            let t = std::time::Instant::now();
            for _ in 0..20 {
                f();
            }
            let each = t.elapsed() / 20;
            eprintln!("{name}: {each:?} a screen");
            assert!(each < std::time::Duration::from_millis(100), "{name}: {each:?} a screen");
        };
        timed("rows", &|| {
            for _ in 0..50 {
                let _ = fonts.layout(&row, &style, None).snapped(&row, cell);
            }
        });
        timed("cells", &|| {
            for _ in 0..50 {
                for c in row.chars() {
                    let one = c.to_string();
                    let _ = fonts.layout(&one, &style, None).snapped(&one, cell);
                }
            }
        });
        let pix = std::cell::RefCell::new(tiny_skia::Pixmap::new(600, 800).unwrap());
        timed("rows laid out and painted", &|| {
            for r in 0..50 {
                let l = fonts.layout(&row, &style, None).snapped(&row, cell);
                fonts.draw(&mut pix.borrow_mut().as_mut(), &l, 0.0, r as f32 * 16.0, crate::colour::Colour::rgb(1.0, 1.0, 1.0));
            }
        });
    }
}
