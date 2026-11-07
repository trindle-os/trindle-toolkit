//! A short thing after long words in a row keeps its whole width.

use super::*;
use super::super::theme::Theme;
use super::super::widget::{self as ww, Length, Tone};

const LONG: &str = "That name cannot be used here because a file with that name is already in this folder, and Text will not write over it";

fn status(said: &str, pill: &str) -> Node<()> {
    ww::row(vec![
        ww::label("Line 12, column 40"),
        ww::note("1024 words, 5800 letters"),
        ww::space(),
        ww::label(said).tone(Tone::Problem),
        ww::card(ww::label(pill)).chip().on_press(()).width(Length::Shrink),
    ]).gap(12.0)
}

fn pill(ui: &Ui<()>) -> (f32, f32) {
    let r = ui.spots.iter().map(|s| s.rect).fold(None::<Rect>, |a, r| match a {
        Some(a) if a.x >= r.x => Some(a),
        _ => Some(r),
    }).expect("the pill is laid out");
    (r.w, r.h)
}

#[test]
fn a_pill_after_long_words_stays_on_one_line() {
    for pill_words in ["UTF-8", "windows-1252", "ISO-8859-15"] {
        let mut wide = Ui::new(status("", pill_words), Theme::new(true, "#fbc711"));
        wide.lay_at(3000, 200);
        let (alone_w, alone_h) = pill(&wide);
        for width in [900u32, 700, 560, 480] {
            let mut ui = Ui::new(status(LONG, pill_words), Theme::new(true, "#fbc711"));
            ui.lay_at(width, 400);
            let (w, h) = pill(&ui);
            assert!(w + 0.5 >= alone_w, "{pill_words} at {width}: its room is {w}, it needs {alone_w}");
            assert!(h <= alone_h + 0.5, "{pill_words} at {width}: it broke onto more lines ({h} tall, one line is {alone_h})");
        }
    }
}
