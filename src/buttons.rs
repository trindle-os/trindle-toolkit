//! Buttons that show what they hold: a colour, a font, a file and a lock. Each press is the program's to answer, such as by showing a `picker::ColourBox` on the page or asking `chooser::open`.

use std::path::Path;

use super::colour::Colour;
use super::theme;
use super::widget::{button, word_button, Node};
use crate::t;

pub fn colour_button<M>(colour: Colour, tip: &str, press: M) -> Node<M> {
    let code = colour.css();
    button("", tip).face(move |canvas, r, _, pen| {
        let corner = theme::corner_px(theme::Part::IconButton, r.w.min(r.h), pen.scale);
        crate::ui::fill(canvas, r, corner, pen.theme.control_edge);
        let inset = pen.scale.round().max(1.0);
        let inner = super::widget::Rect { x: r.x + inset, y: r.y + inset, w: r.w - 2.0 * inset, h: r.h - 2.0 * inset };
        crate::ui::fill(canvas, inner, (corner - inset).max(0.0), colour);
    }).spoken(t!("kit-colour-button", name = tip, code = code)).on_press(press)
}

pub fn font_button<M>(family: &str, size: f32, tip: &str, press: M) -> Node<M> {
    let words = t!("kit-font-button", family = family, size = f64::from(size));
    word_button(words.clone()).spoken(format!("{tip}: {words}")).on_press(press)
}

pub fn file_button<M>(file: Option<&Path>, tip: &str, press: M) -> Node<M> {
    let words = match file.and_then(Path::file_name) {
        Some(name) => name.to_string_lossy().into_owned(),
        None => t!("kit-file-none").to_string(),
    };
    word_button(words.clone()).spoken(format!("{tip}: {words}")).on_press(press)
}

pub fn lock_button<M>(locked: bool, press: M) -> Node<M> {
    let (icon, tip) = if locked {
        ("changes-prevent-symbolic", t!("kit-lock-unlock"))
    } else {
        ("changes-allow-symbolic", t!("kit-lock-lock"))
    };
    button(icon, tip).on(!locked).on_press(press)
}
