//! Tests of the window's widget tree: painting, input, focus and refusals.

use super::*;

pub(super) fn ago(d: Duration) -> Instant {
    Instant::now().checked_sub(d).unwrap_or_else(Instant::now)
}

use super::super::input::Mods;
use super::super::widget::{button, Field, card, choice, column, drawing, entry, item, label, list, ring, row, scroll, slider, switch};
use super::super::widget::Tone;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Save,
    Wifi(bool),
    Volume(f32),
    Name(String),
    Pass(String),
    Go,
    Pick(usize),
    Open(usize),
    Many(Vec<usize>),
    Menu(usize, i32, i32),
}

fn view(wifi: bool, volume: f32) -> Node<Msg> {
    column(vec![
        row(vec![button("document-open-symbolic", "Open").on_press(Msg::Save), button("x", "Not ready")]),
        switch(wifi, "Wi-Fi", Msg::Wifi).key("wifi"),
        slider(volume, 0.0, 1.0, 0.1, "Volume", Msg::Volume),
    ]).padding(10.0)
}

fn ui(wifi: bool, volume: f32) -> Ui<Msg> {
    let mut ui = Ui::new(view(wifi, volume), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui
}

fn key(sym: Keysym, shift: bool) -> Input {
    Input::Key { sym, text: None, mods: Mods { shift, ..Mods::default() }, repeat: false }
}

fn typed(t: &str) -> Input {
    Input::Key { sym: Keysym::NoSymbol, text: Some(t.into()), mods: Mods::default(), repeat: false }
}

fn ctrl(sym: Keysym) -> Input {
    Input::Key { sym, text: None, mods: Mods { ctrl: true, ..Mods::default() }, repeat: false }
}

fn wheel(y: f32) -> Input {
    Input::Scroll { x: 0.0, y, steps: true }
}

fn rows(n: usize) -> Vec<Item> {
    (0..n).map(|i| item(format!("Row {i}"))).collect()
}

mod pressing;
mod painting;
mod typing;
mod scrolling;
mod dragging;
mod lists;
