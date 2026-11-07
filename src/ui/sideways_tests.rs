//! Tests of sideways scrolling by finger and keys, and Tab bringing a widget into sight along.

use super::*;
use super::super::widget::{button, row, scroll_x, Length};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Go,
}

fn wide() -> Ui<Msg> {
    let cols = (0..12).map(|i| button("x", format!("Column {i}")).on_press(Msg::Go).key(format!("c{i}")).width(Length::Fixed(120.0))).collect();
    let mut ui = Ui::new(scroll_x("wide", row(cols)), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui
}

fn key(sym: Keysym, shift: bool) -> Input {
    Input::Key { sym, text: None, mods: Mods { shift, ..Mods::default() }, repeat: false }
}

fn off(ui: &Ui<Msg>) -> f32 {
    ui.offset(ui.spot("wide").unwrap())
}

#[test]
fn a_finger_swipes_a_sideways_area_without_pressing() {
    let mut ui = wide();
    let now = Instant::now();
    let b = ui.spot("c1").unwrap().rect;
    let (x, y) = (b.x + b.w / 2.0, b.y + b.h / 2.0);
    ui.input(&Input::TouchDown { id: 1, x, y }, now);
    assert_eq!(ui.pressed.as_deref(), Some("c1"));
    ui.input(&Input::TouchMove { id: 1, x: x - 80.0, y: y + 5.0 }, now);
    assert_eq!(off(&ui), 80.0);
    assert!(ui.pressed.is_none());
    assert!(ui.input(&Input::TouchUp { id: 1 }, now).messages.is_empty());
    assert!(ui.swipe_x.is_none());
    let b = ui.spot("c2").unwrap().rect;
    let (x, y) = (b.x + b.w / 2.0, b.y + b.h / 2.0);
    ui.input(&Input::TouchDown { id: 2, x, y }, now);
    ui.input(&Input::TouchMove { id: 2, x: x + 3.0, y }, now);
    assert_eq!(ui.input(&Input::TouchUp { id: 2 }, now).messages, vec![Msg::Go]);
    assert_eq!(off(&ui), 80.0);
}

#[test]
fn a_finger_going_down_more_than_across_leaves_a_sideways_area_still() {
    let mut ui = wide();
    let now = Instant::now();
    let b = ui.spot("c1").unwrap().rect;
    let (x, y) = (b.x + b.w / 2.0, b.y + 4.0);
    ui.input(&Input::TouchDown { id: 1, x, y }, now);
    ui.input(&Input::TouchMove { id: 1, x: x - 12.0, y: y + 30.0 }, now);
    assert_eq!(off(&ui), 0.0);
    ui.input(&Input::TouchCancel, now);
    assert!(ui.swipe_x.is_none());
}

#[test]
fn arrows_and_shifted_page_keys_scroll_the_area_round_the_keyboard() {
    let mut ui = wide();
    let now = Instant::now();
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.focus.as_deref(), Some("c0"));
    assert!(ui.input(&key(Keysym::Right, false), now).redraw);
    let a = off(&ui);
    assert!(a > 0.0);
    ui.input(&key(Keysym::Left, false), now);
    assert_eq!(off(&ui), 0.0);
    ui.input(&key(Keysym::Page_Down, true), now);
    let s = ui.spot("wide").unwrap().clone();
    assert_eq!(off(&ui), ui.page(s.rect.w).round());
    ui.input(&key(Keysym::End, true), now);
    assert_eq!(off(&ui), (s.full - s.rect.w).round());
    ui.input(&key(Keysym::Home, true), now);
    assert_eq!(off(&ui), 0.0);
    ui.focus = None;
    ui.input(&Input::PointerMove { x: 100.0, y: 100.0 }, now);
    ui.input(&key(Keysym::Right, false), now);
    assert_eq!(off(&ui), a);
}

#[test]
fn tab_brings_a_widget_into_sight_along_and_back() {
    let mut ui = wide();
    let now = Instant::now();
    for _ in 0..12 {
        ui.input(&key(Keysym::Tab, false), now);
    }
    assert_eq!(ui.focus.as_deref(), Some("c11"));
    let (a, b) = (ui.spot("wide").unwrap().rect, ui.spot("c11").unwrap().rect);
    assert!(b.x >= a.x && b.x + b.w <= a.x + a.w);
    for _ in 0..11 {
        ui.input(&key(Keysym::ISO_Left_Tab, true), now);
    }
    assert_eq!(ui.focus.as_deref(), Some("c0"));
    let (a, b) = (ui.spot("wide").unwrap().rect, ui.spot("c0").unwrap().rect);
    assert!(b.x >= a.x && b.x + b.w <= a.x + a.w);
}
