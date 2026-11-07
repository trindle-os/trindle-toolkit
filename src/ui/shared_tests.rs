//! Input the tests of several files send the same way.

use super::*;

pub(super) fn key(sym: Keysym) -> Input {
    Input::Key { sym, text: None, mods: Mods::default(), repeat: false }
}

pub(super) fn typed(t: &str) -> Input {
    Input::Key { sym: Keysym::NoSymbol, text: Some(t.into()), mods: Mods::default(), repeat: false }
}

pub(super) fn middle(r: Rect) -> (f32, f32) {
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

pub(super) fn click<M: Clone>(ui: &mut Ui<M>, x: f32, y: f32) -> Vec<M> {
    let now = Instant::now();
    ui.input(&Input::PointerMove { x, y }, now);
    let mut all = ui.input(&Input::Press { button: Button::Left, x, y }, now).messages;
    all.extend(ui.input(&Input::Release { button: Button::Left, x, y }, now).messages);
    all
}
