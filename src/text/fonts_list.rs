//! What a font chooser lists, and which font draws each part of some words once the chosen one lacks a letter.

use std::ops::Range;

use skrifa::FontRef;

use super::{emoji, Fonts, Style};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFamily {
    pub name: String,
    pub faces: Vec<(u16, bool)>,
    pub monospace: bool,
    pub colour: bool,
}

impl Fonts {
    pub fn chooser(&self) -> Vec<FontFamily> {
        let db = self.db.borrow();
        let mut out: Vec<FontFamily> = Vec::new();
        for f in db.faces() {
            let Some((name, _)) = f.families.first() else { continue };
            let at = match out.iter().position(|o| o.name.eq_ignore_ascii_case(name)) {
                Some(at) => at,
                None => {
                    let colour = db.with_face_data(f.id, |data, index| FontRef::from_index(data, index).is_ok_and(|font| emoji::has_pictures(&font)));
                    out.push(FontFamily { name: name.clone(), faces: Vec::new(), monospace: true, colour: colour.unwrap_or(false) });
                    out.len() - 1
                }
            };
            let fam = &mut out[at];
            fam.monospace &= f.monospaced;
            let face = (f.weight.0, f.style != fontdb::Style::Normal);
            if !fam.faces.contains(&face) {
                fam.faces.push(face);
            }
        }
        for fam in &mut out {
            fam.faces.sort_unstable();
        }
        out.sort_by_key(|f| f.name.to_lowercase());
        out
    }

    pub fn fallbacks(&self, text: &str, style: &Style) -> Vec<(Range<usize>, Option<String>)> {
        let chosen = self.chosen(style);
        let name = |id| self.db.borrow().face(id).and_then(|f| f.families.first().map(|(n, _)| n.clone()));
        let has = |id, c: char| self.covers(&self.db.borrow(), id).contains(&u32::from(c));
        let mut out: Vec<(Range<usize>, Option<String>)> = Vec::new();
        for (at, c) in text.char_indices() {
            let end = at + c.len_utf8();
            if let Some(last) = out.last_mut().filter(|_| c.is_whitespace() || c.is_control() || c == '\u{fe0f}' || c == '\u{200d}') {
                last.0.end = end;
                continue;
            }
            let id = chosen.filter(|&id| has(id, c)).or_else(|| self.stand_in(c, style));
            let family = id.and_then(name);
            match out.last_mut() {
                Some(last) if last.1 == family => last.0.end = end,
                _ => out.push((at..end, family)),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chooser_lists_each_family_once_by_name_with_its_faces() {
        let fonts = Fonts::new();
        let list = fonts.chooser();
        assert!(!list.is_empty());
        assert!(list.windows(2).all(|w| w[0].name.to_lowercase() < w[1].name.to_lowercase()));
        assert!(list.iter().all(|f| !f.faces.is_empty() && f.faces.windows(2).all(|w| w[0] < w[1])));
        let mono = fonts.monospace().to_string();
        if let Some(f) = list.iter().find(|f| f.name == mono) {
            assert!(f.monospace, "{mono} is fixed width");
        }
        assert_eq!(list.iter().map(|f| f.name.clone()).collect::<Vec<_>>(), fonts.families());
    }

    #[test]
    fn plain_words_are_one_run_and_a_letter_nobody_has_is_said_to_be_missing() {
        let fonts = Fonts::new();
        let style = Style::new(14.0);
        let runs = fonts.fallbacks("Hello there", &style);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].0, 0..11);
        assert!(runs[0].1.is_some());
        let runs = fonts.fallbacks("a\u{10fffd}", &style);
        assert_eq!(runs.last().map(|r| (r.0.clone(), r.1.clone())), Some((1..5, None)));
    }

    #[test]
    fn an_emoji_comes_from_a_colour_font_when_one_is_installed() {
        let fonts = Fonts::new();
        let colour: Vec<String> = fonts.chooser().into_iter().filter(|f| f.colour).map(|f| f.name).collect();
        let runs = fonts.fallbacks("Hi \u{1f600}", &Style::new(14.0));
        if !colour.is_empty() {
            let last = runs.last().and_then(|r| r.1.clone()).unwrap_or_default();
            assert!(colour.contains(&last), "{last} is not one of {colour:?}");
        }
    }
}
