//! Tests of the band dragged across a list to pick rows: where it may begin, what it picks and how it is painted.

use super::*;
use super::super::widget::{item, list, Length};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Pick(usize),
    Many(Vec<usize>),
    Reorder(usize, usize),
}

fn band_ui(picked: Vec<usize>) -> Ui<Msg> {
    let rows = (0..5).map(|i| item(format!("Row {i}"))).collect();
    let tree = list(rows, None, Msg::Pick).many(picked, Msg::Many).on_reorder(Msg::Reorder).height(Length::Fixed(400.0)).key("list");
    let mut ui = Ui::new(tree, Theme::new(false, "#fbc711"));
    ui.size = (300, 420);
    ui.settle();
    ui
}

fn row_middle(ui: &Ui<Msg>, r: usize) -> (f32, f32) {
    let s = ui.spot("list").unwrap().rect;
    let h = ui.row_of("list");
    (s.x + s.w / 2.0, s.y + h * (r as f32 + 0.5))
}

fn drag(ui: &mut Ui<Msg>, from: (f32, f32), to: (f32, f32)) -> (Vec<Msg>, Vec<Msg>) {
    let now = Instant::now();
    ui.input(&Input::PointerMove { x: from.0, y: from.1 }, now);
    let mut moved = ui.input(&Input::Press { button: Button::Left, x: from.0, y: from.1 }, now).messages;
    moved.extend(ui.input(&Input::PointerMove { x: to.0, y: to.1 }, now).messages);
    let let_go = ui.input(&Input::Release { button: Button::Left, x: to.0, y: to.1 }, now).messages;
    (moved, let_go)
}

fn ctrl(ui: &mut Ui<Msg>, held: bool) {
    ui.input(&Input::Mods(Mods { ctrl: held, ..Mods::default() }), Instant::now());
}

#[test]
fn a_band_from_the_room_below_the_rows_picks_every_row_it_crosses() {
    let mut ui = band_ui(vec![0]);
    let (x, _) = row_middle(&ui, 0);
    let below = (x, row_middle(&ui, 4).1 + 100.0);
    let to = row_middle(&ui, 2);
    let (moved, let_go) = drag(&mut ui, below, to);
    assert_eq!(moved, vec![Msg::Many(vec![2, 3, 4])]);
    assert!(let_go.is_empty(), "{let_go:?}");
    assert!(ui.band.is_none());
}

#[test]
fn a_press_on_a_row_without_ctrl_is_left_for_the_row_and_its_dragging() {
    let mut ui = band_ui(vec![]);
    let (from, to) = (row_middle(&ui, 0), row_middle(&ui, 3));
    let (_, let_go) = drag(&mut ui, from, to);
    assert_eq!(let_go, vec![Msg::Reorder(0, 4)]);
    assert!(ui.band.is_none());
}

#[test]
fn with_ctrl_a_band_begins_on_a_row_and_adds_to_what_was_picked() {
    let mut ui = band_ui(vec![4]);
    ctrl(&mut ui, true);
    let (from, to) = (row_middle(&ui, 0), row_middle(&ui, 1));
    let (moved, let_go) = drag(&mut ui, from, to);
    assert_eq!(moved, vec![Msg::Many(vec![0, 1, 4])]);
    assert!(let_go.is_empty(), "{let_go:?}");
    let mut ui = band_ui(vec![4]);
    ctrl(&mut ui, true);
    let at = row_middle(&ui, 4);
    let (_, let_go) = drag(&mut ui, at, at);
    assert!(let_go.contains(&Msg::Many(vec![])), "{let_go:?}");
}

#[test]
fn the_band_is_painted_in_the_highlight_while_it_is_held() {
    let mut ui = band_ui(vec![]);
    let (x, _) = row_middle(&ui, 0);
    let below = (x - 40.0, row_middle(&ui, 4).1 + 100.0);
    let now = Instant::now();
    ui.input(&Input::PointerMove { x: below.0, y: below.1 }, now);
    ui.input(&Input::Press { button: Button::Left, x: below.0, y: below.1 }, now);
    ui.input(&Input::PointerMove { x: x + 40.0, y: below.1 - 60.0 }, now);
    let mut pix = tiny_skia::Pixmap::new(300, 420).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    let a = ui.theme.accent.to_skia().to_color_u8();
    let p = pix.pixel(below.0 as u32, (below.1 - 30.0) as u32).unwrap();
    assert_eq!((p.red(), p.green(), p.blue()), (a.red(), a.green(), a.blue()), "the band's edge is in the highlight");
    let inside = pix.pixel(x as u32, (below.1 - 30.0) as u32).unwrap();
    assert_ne!((inside.red(), inside.green(), inside.blue()), (255, 255, 255), "the band is washed in the highlight");
    ui.input(&Input::Release { button: Button::Left, x: x + 40.0, y: below.1 - 60.0 }, now);
    let mut after = tiny_skia::Pixmap::new(300, 420).unwrap();
    ui.paint(&mut after.as_mut(), 1.0);
    assert_ne!(after.pixel(below.0 as u32, (below.1 - 30.0) as u32), pix.pixel(below.0 as u32, (below.1 - 30.0) as u32));
}
