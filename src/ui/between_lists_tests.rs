//! Tests of rows dragged from one list into another.

use super::*;
use super::super::widget::{item, list, row};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Pick(usize),
    Moved(String, usize, usize),
    Reordered(usize, usize),
}

fn two() -> Ui<Msg> {
    let todo = list(vec![item("Wash"), item("Shop"), item("Cook")], None, Msg::Pick)
        .on_reorder(Msg::Reordered).on_move_in(Msg::Moved).key("todo");
    let done = list(vec![item("Post"), item("Pay")], None, Msg::Pick).on_move_in(Msg::Moved).key("done");
    let mut ui = Ui::new(row(vec![todo, done]), Theme::new(false, "#fbc711"));
    ui.size = (600, 400);
    ui.settle();
    ui
}

fn left(down: bool, x: f32, y: f32) -> Input {
    if down { Input::Press { button: Button::Left, x, y } } else { Input::Release { button: Button::Left, x, y } }
}

#[test]
fn a_row_dragged_into_another_list_says_where_from_and_to() {
    let mut ui = two();
    let now = Instant::now();
    let (a, b) = (ui.spot("todo").unwrap().rect, ui.spot("done").unwrap().rect);
    let h = ui.px(theme::sheet().row);
    ui.input(&left(true, a.x + 20.0, a.y + h * 1.5), now);
    ui.input(&Input::PointerMove { x: b.x + 20.0, y: b.y + h * 1.1 }, now);
    assert_eq!(ui.gap, Some(("done".to_string(), 1)), "the line shows in the other list");
    let sent = ui.input(&left(false, b.x + 20.0, b.y + h * 1.1), now).messages;
    assert_eq!(sent, vec![Msg::Moved("todo".to_string(), 1, 1)]);
    assert!(ui.gap.is_none() && ui.row_drag.is_none());
    ui.input(&left(true, a.x + 20.0, a.y + h * 2.5), now);
    ui.input(&Input::PointerMove { x: a.x + 20.0, y: a.y + h * 0.1 }, now);
    assert_eq!(ui.input(&left(false, a.x + 20.0, a.y + h * 0.1), now).messages, vec![Msg::Reordered(2, 0)]);
}

#[test]
fn a_list_that_only_takes_rows_in_never_reorders_its_own() {
    let mut ui = two();
    let now = Instant::now();
    let (a, b) = (ui.spot("todo").unwrap().rect, ui.spot("done").unwrap().rect);
    let h = ui.px(theme::sheet().row);
    ui.input(&left(true, b.x + 20.0, b.y + h * 0.5), now);
    ui.input(&Input::PointerMove { x: b.x + 20.0, y: b.y + h * 2.0 }, now);
    assert!(ui.gap.is_none());
    assert!(ui.input(&left(false, b.x + 20.0, b.y + h * 2.0), now).messages.is_empty());
    ui.input(&left(true, b.x + 20.0, b.y + h * 0.5), now);
    ui.input(&Input::PointerMove { x: a.x + 20.0, y: a.y + h * 2.9 }, now);
    assert_eq!(ui.input(&left(false, a.x + 20.0, a.y + h * 2.9), now).messages, vec![Msg::Moved("done".to_string(), 0, 3)]);
    ui.input(&left(true, b.x + 20.0, b.y + h * 1.5), now);
    assert_eq!(ui.input(&left(false, b.x + 20.0, b.y + h * 1.5), now).messages, vec![Msg::Pick(1)]);
}
