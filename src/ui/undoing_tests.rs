//! Tests of the window's undo and redo keys after a text box's or text area's own run out.

use super::*;
use super::shared_tests::typed;
use super::super::editing::area::{Event, TextArea};
use super::super::editing::editor::Look;
use super::super::widget::{button, column, entry};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Name(String),
    Undo,
    Redo,
    Go,
    Area,
}

fn chord(sym: Keysym, shift: bool) -> Input {
    Input::Key { sym, text: None, mods: Mods { ctrl: true, shift, ..Mods::default() }, repeat: false }
}

fn form(name: &str) -> Ui<Msg> {
    let mut ui = Ui::new(column(vec![entry(name, "Name", Msg::Name).key("name"), button("x", "Go").on_press(Msg::Go).key("go")]), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.on_undo(Msg::Undo, Msg::Redo);
    ui.settle();
    ui
}

#[test]
fn undo_keys_go_to_the_window_away_from_a_text_box() {
    let mut ui = form("");
    let now = Instant::now();
    ui.keyboard_to("go");
    assert_eq!(ui.input(&chord(Keysym::z, false), now).messages, vec![Msg::Undo]);
    assert_eq!(ui.input(&chord(Keysym::Z, true), now).messages, vec![Msg::Redo]);
    assert_eq!(ui.input(&chord(Keysym::y, false), now).messages, vec![Msg::Redo]);
    assert!(ui.input(&chord(Keysym::x, false), now).messages.is_empty());
    let alt = Input::Key { sym: Keysym::z, text: None, mods: Mods { ctrl: true, alt: true, ..Mods::default() }, repeat: false };
    assert!(ui.input(&alt, now).messages.is_empty());
}

#[test]
fn a_text_box_undoes_its_own_first_then_the_window_does() {
    let mut ui = form("");
    let now = Instant::now();
    ui.keyboard_to("name");
    let sent = ui.input(&typed("J"), now).messages;
    assert_eq!(sent, vec![Msg::Name("J".into())]);
    ui.set_view(column(vec![entry("J", "Name", Msg::Name).key("name"), button("x", "Go").on_press(Msg::Go).key("go")]));
    assert_eq!(ui.input(&chord(Keysym::z, false), now).messages, vec![Msg::Name(String::new())]);
    ui.set_view(column(vec![entry("", "Name", Msg::Name).key("name"), button("x", "Go").on_press(Msg::Go).key("go")]));
    assert_eq!(ui.input(&chord(Keysym::z, false), now).messages, vec![Msg::Undo]);
    assert_eq!(ui.input(&chord(Keysym::y, false), now).messages, vec![Msg::Name("J".into())]);
}

#[test]
fn a_window_with_no_undo_leaves_the_keys_alone() {
    let mut ui = Ui::new(button("x", "Go").on_press(Msg::Go).key("go"), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.keyboard_to("go");
    assert!(ui.input(&chord(Keysym::z, false), Instant::now()).messages.is_empty());
}

#[test]
fn a_text_area_passes_undo_on_once_its_own_are_spent() {
    let area = TextArea::new("", Look { font: None, size: 14.0, wrap: true, numbers: false });
    let mut ui = Ui::new(area.node("Notes", 100.0, |_| Msg::Area).key("notes"), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.on_undo(Msg::Undo, Msg::Redo);
    ui.keyboard_to("notes");
    let now = Instant::now();
    assert_eq!(ui.input(&chord(Keysym::z, false), now).messages, vec![Msg::Undo]);
    area.apply(Event::Key(typed("a")));
    assert_eq!(ui.input(&chord(Keysym::z, false), now).messages, vec![Msg::Area]);
    assert!(area.pane().borrow().passes(&chord(Keysym::y, false)), "nothing to redo");
}
