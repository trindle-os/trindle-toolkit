//! Tests of the split pane: dragging, least sizes, keys, kept places, the screen reader and saving.

use std::collections::BTreeMap;

use super::*;
use super::super::widget::{button, column, scroll, split, split_down};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Save,
}

fn side(key: &str) -> Node<Msg> {
    scroll(key, column(vec![button("x", "Save").on_press(Msg::Save)]))
}

fn split_ui(down: bool, size: (u32, u32)) -> Ui<Msg> {
    let tree = if down { split_down("files", side("left"), side("right")) } else { split("files", side("left"), side("right")) };
    let mut ui = Ui::new(tree, Theme::new(false, "#fbc711"));
    ui.size = size;
    ui.settle();
    ui
}

fn sizes(ui: &mut Ui<Msg>) -> (f32, f32) {
    ui.settle();
    let (a, b) = (ui.spot("left").unwrap().rect, ui.spot("right").unwrap().rect);
    if a.x == b.x { (a.h, b.h) } else { (a.w, b.w) }
}

fn key(ui: &mut Ui<Msg>, sym: Keysym) -> Reply<Msg> {
    ui.input(&Input::Key { sym, text: None, mods: Mods::default(), repeat: false }, Instant::now())
}

fn drag(ui: &mut Ui<Msg>, from: (f32, f32), to: (f32, f32)) {
    let now = ui.last_click.as_ref().map_or_else(Instant::now, |c| c.0 + Duration::from_secs(5));
    ui.input(&Input::PointerMove { x: from.0, y: from.1 }, now);
    ui.input(&Input::Press { button: Button::Left, x: from.0, y: from.1 }, now);
    ui.input(&Input::PointerMove { x: to.0, y: to.1 }, now);
    ui.input(&Input::Release { button: Button::Left, x: to.0, y: to.1 }, now);
}

#[test]
fn dragging_the_divider_changes_the_sides_and_shows_the_pull_cursor() {
    let mut ui = split_ui(false, (600, 300));
    assert_eq!(sizes(&mut ui), (300.0, 299.0));
    let d = ui.spot("files/2").unwrap().rect;
    assert_eq!((d.w, d.x + d.w / 2.0), (24.0, 300.0), "24 points to grab, over the line");
    ui.input(&Input::PointerMove { x: 295.0, y: 100.0 }, Instant::now());
    assert_eq!(ui.cursor(), Cursor::ColResize);
    drag(&mut ui, (295.0, 100.0), (395.0, 100.0));
    assert_eq!(sizes(&mut ui), (400.0, 199.0));
    assert_eq!(ui.spot("right").unwrap().rect.x, 401.0);
    let mut down = split_ui(true, (300, 600));
    down.input(&Input::PointerMove { x: 100.0, y: 300.0 }, Instant::now());
    assert_eq!(down.cursor(), Cursor::RowResize);
    drag(&mut down, (100.0, 300.0), (100.0, 250.0));
    assert_eq!(sizes(&mut down), (250.0, 349.0));
}

#[test]
fn each_side_keeps_its_least_and_the_second_gives_way_first() {
    let mut ui = split_ui(false, (600, 300));
    drag(&mut ui, (300.0, 100.0), (5.0, 100.0));
    assert_eq!(sizes(&mut ui), (120.0, 479.0));
    drag(&mut ui, (120.0, 100.0), (595.0, 100.0));
    assert_eq!(sizes(&mut ui), (479.0, 120.0));
    ui.size = (200, 300);
    assert_eq!(sizes(&mut ui), (120.0, 79.0));
    ui.size = (100, 300);
    assert_eq!(sizes(&mut ui), (99.0, 0.0));
}

#[test]
fn the_keys_move_a_divider_with_the_keyboard_on_it() {
    let mut ui = split_ui(false, (600, 300));
    ui.focus = Some("files/2".into());
    assert!(key(&mut ui, Keysym::Right).redraw);
    assert_eq!(sizes(&mut ui).0, 316.0);
    key(&mut ui, Keysym::Left);
    key(&mut ui, Keysym::Left);
    assert_eq!(sizes(&mut ui).0, 284.0);
    key(&mut ui, Keysym::Up);
    assert_eq!(sizes(&mut ui).0, 284.0);
    key(&mut ui, Keysym::Home);
    assert_eq!(sizes(&mut ui).0, 120.0);
    key(&mut ui, Keysym::End);
    assert_eq!(sizes(&mut ui), (479.0, 120.0));
    assert!(key(&mut ui, Keysym::Return).messages.is_empty());
    assert_eq!(sizes(&mut ui).0, 479.0);
    let mut down = split_ui(true, (300, 600));
    down.focus = Some("files/2".into());
    key(&mut down, Keysym::Down);
    assert_eq!(sizes(&mut down).0, 316.0);
    key(&mut down, Keysym::Right);
    assert_eq!(sizes(&mut down).0, 316.0);
}

#[test]
fn the_place_is_kept_across_views_and_a_double_click_puts_it_back() {
    let mut ui = split_ui(false, (600, 300));
    drag(&mut ui, (300.0, 100.0), (400.0, 100.0));
    ui.set_view(split("files", side("left"), side("right")));
    assert_eq!(sizes(&mut ui).0, 400.0);
    ui.size = (800, 300);
    assert_eq!(sizes(&mut ui), (400.0, 399.0));
    ui.size = (600, 300);
    let now = Instant::now();
    for _ in 0..2 {
        ui.input(&Input::Press { button: Button::Left, x: 400.0, y: 100.0 }, now);
        ui.input(&Input::Release { button: Button::Left, x: 400.0, y: 100.0 }, now);
    }
    assert_eq!(sizes(&mut ui).0, 300.0, "back at half of the room");
    assert!(ui.split_places().is_empty());
}

#[test]
fn the_screen_reader_hears_a_splitter_with_its_share() {
    let mut ui = split_ui(false, (600, 300));
    drag(&mut ui, (300.0, 100.0), (400.0, 100.0));
    ui.settle();
    let told = ui.access("Test");
    let part = told.nodes.iter().find(|(_, n)| n.role() == accesskit::Role::Splitter).map(|(_, n)| n).unwrap();
    assert_eq!(part.label(), Some(&*crate::t!("kit-divider")));
    assert_eq!((part.numeric_value(), part.min_numeric_value(), part.max_numeric_value()), (Some(67.0), Some(20.0), Some(80.0)));
    assert_eq!(part.orientation(), Some(accesskit::Orientation::Vertical));
}

#[test]
fn the_divider_is_drawn_thin_and_lit_in_the_highlight_under_the_pointer() {
    let mut ui = split_ui(false, (600, 300));
    let shot = |ui: &mut Ui<Msg>| {
        let mut pix = tiny_skia::Pixmap::new(600, 300).unwrap();
        ui.paint(&mut pix.as_mut(), 1.0);
        let p = pix.pixel(301, 150).unwrap();
        [p.red(), p.green(), p.blue()]
    };
    assert_ne!(shot(&mut ui), [251, 199, 17]);
    ui.input(&Input::PointerMove { x: 300.0, y: 150.0 }, Instant::now());
    assert_eq!(shot(&mut ui), [251, 199, 17]);
}

#[test]
fn any_key_comes_back_from_the_settings_file() {
    let keys = ["a;b", ";", "50%", "%3B", " lead", "trail ", "two words", "tab\there", "x=7", "", "é;ü", "a\u{a0}b", "\u{85}", ";;"];
    let places: BTreeMap<String, f32> = keys.iter().enumerate().map(|(i, k)| (k.to_string(), i as f32 * 10.0)).collect();
    let mut ini = crate::common::plumbing::Ini::default();
    assert!(save_splits(&mut ini, "Window", &places));
    let back = load_splits(&crate::common::plumbing::Ini::parse(&ini.text()), "Window");
    assert_eq!(back, places);
}

#[test]
fn a_splits_line_from_before_the_escape_still_loads() {
    let ini = crate::common::plumbing::Ini::parse("[Window]\nSplits=files=300;50%=20;bad%zz=5\n");
    let back = load_splits(&ini, "Window");
    assert_eq!(back, BTreeMap::from([("files".to_string(), 300.0), ("50%".to_string(), 20.0), ("bad%zz".to_string(), 5.0)]));
}

#[test]
fn split_places_are_given_out_saved_and_put_back() {
    let mut ui = split_ui(false, (600, 300));
    drag(&mut ui, (300.0, 100.0), (400.0, 100.0));
    let places = ui.split_places();
    assert_eq!(places, BTreeMap::from([("files".to_string(), 400.0)]));
    let mut ini = crate::common::plumbing::Ini::default();
    assert!(save_splits(&mut ini, "Window", &places));
    let back = load_splits(&crate::common::plumbing::Ini::parse(&ini.text()), "Window");
    assert_eq!(back, places);
    let mut other = split_ui(false, (600, 300));
    other.set_split_places(back);
    assert_eq!(sizes(&mut other).0, 400.0);
}
