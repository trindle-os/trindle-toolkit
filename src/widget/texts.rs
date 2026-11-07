//! Words laid out and kept for the next layout, and how a main action's pill fits its words.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::text::{Fonts, Layout, Style};
use crate::theme;

use super::Words;

type TextKey = (String, u32, u16, u32, u32, bool, Option<String>);

struct Laid {
    layout: Layout,
    turn: u64,
    frame: u64,
}

#[derive(Default)]
pub(crate) struct Texts {
    kept: RefCell<HashMap<TextKey, Laid>>,
    turns: std::cell::Cell<u64>,
    frames: std::cell::Cell<u64>,
    seen: std::cell::Cell<u32>,
    #[cfg(test)]
    laid: std::cell::Cell<u64>,
}

impl Texts {
    pub(crate) fn get(&self, fonts: &Fonts, text: &str, style: &Style, width: Option<f32>) -> Layout {
        if self.seen.replace(fonts.generation()) != fonts.generation() {
            self.kept.borrow_mut().clear();
        }
        let key = (text.to_string(), style.size.to_bits(), style.weight, width.map_or(u32::MAX, |w| w.round() as u32), style.spacing.to_bits(),
                   style.italic, style.family.clone());
        let (turn, frame) = (self.turns.get(), self.frames.get());
        let mut kept = self.kept.borrow_mut();
        if let Some(k) = kept.get_mut(&key) {
            (k.turn, k.frame) = (turn, frame);
            return k.layout.clone();
        }
        #[cfg(test)]
        self.laid.set(self.laid.get() + 1);
        let layout = fonts.layout(text, style, width);
        kept.insert(key, Laid { layout: layout.clone(), turn, frame });
        layout
    }

    pub(crate) fn turn(&self) {
        let (turn, frame) = (self.turns.get() + 1, self.frames.get());
        self.turns.set(turn);
        self.kept.borrow_mut().retain(|_, k| turn - k.turn < 2 || frame - k.frame < 2);
    }

    pub(crate) fn frame(&self) {
        self.frames.set(self.frames.get() + 1);
    }

    #[cfg(test)]
    pub(crate) fn laid(&self) -> u64 {
        self.laid.get()
    }
}

impl Words {
    pub(crate) fn style(self, scale: f32) -> Style {
        let s = theme::sheet();
        let ((size, weight), tighten) = match self {
            Words::Plain => (s.text, 0.0),
            Words::Title => (s.figure, 0.0),
            Words::Note => (s.note, 0.0),
            Words::Page => (s.page_title, s.tighten),
            Words::Lead => (s.lead, 0.0),
            Words::Heading => (s.heading, 0.0),
            Words::Headline => (s.headline, 0.0),
            Words::Section => (s.section, s.tighten / 2.0),
            Words::Label => (s.form_label, 0.0),
            Words::Column => (s.column_heading, 0.0),
            Words::Side => (s.side_heading, 0.0),
            Words::Figure => (s.big_figure, s.tighten),
        };
        let size = size * scale;
        Style { weight, spacing: -tighten * size, ..Style::new(size) }
    }
}

fn action_style_in(s: &theme::Sheet, scale: f32) -> Style {
    Style { weight: s.action_weight, ..Style::new(s.action_text * scale) }
}

pub(crate) fn action_fixed(scale: f32) -> f32 {
    action_fixed_in(theme::sheet(), scale)
}

pub(crate) fn action_fixed_in(s: &theme::Sheet, scale: f32) -> f32 {
    2.0 * (s.action_padding * scale).round() + (s.action_gap * scale).round() + (s.icon * scale).round()
}

pub(crate) fn action_room(scale: f32) -> f32 {
    (theme::sheet().action_padding * scale / 2.0).round()
}

pub(crate) fn action_words(fonts: &Fonts, texts: &Texts, words: &str, across: f32, scale: f32) -> Layout {
    action_words_in(theme::sheet(), fonts, texts, words, across, scale)
}

pub(crate) fn action_words_in(s: &theme::Sheet, fonts: &Fonts, texts: &Texts, words: &str, across: f32, scale: f32) -> Layout {
    let style = action_style_in(s, scale);
    let whole = texts.get(fonts, words, &style, None);
    let room = across - action_fixed_in(s, scale);
    if across <= 0.0 || whole.width.ceil() <= room {
        return whole;
    }
    let room = room.max(1.0);
    let dots = texts.get(fonts, "\u{2026}", &style, None);
    let two = texts.get(fonts, words, &style, Some(room)).cut_after(2, room, &dots);
    if two.lines.iter().all(|l| l.width <= room + 0.5) {
        return two;
    }
    whole.cut(room, &dots)
}

pub(crate) fn word_button_style(scale: f32) -> Style {
    Style::new(theme::sheet().word_button_text * scale)
}
