//! The dropdown: its open list floats, answers the keyboard, narrows as letters are typed, and closes on a press elsewhere.

use super::*;
use super::shared_tests::{click, key, middle, typed};
use super::dropdown::{narrowed, opens_up};
use super::super::theme::Theme;
use super::super::widget::{self as ww, Length};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Pick(usize),
    Go,
}

const W: u32 = 600;
const H: u32 = 500;

fn names(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("Choice {i}")).collect()
}

fn page(n: usize, chosen: Option<usize>, above: f32) -> Node<Msg> {
    ww::column(vec![
        ww::space().height(Length::Fixed(above)),
        ww::row(vec![ww::dropdown(names(n), chosen, "Time zone", Msg::Pick).key("zone"), ww::label("Time zone")]).gap(12.0),
        ww::label("The words under it"),
        ww::button("go-next-symbolic", "Go").on_press(Msg::Go).key("go"),
    ]).gap(8.0).padding(10.0)
}

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(true, "#fbc711"));
    ui.size = (W, H);
    ui.settle();
    ui
}

fn open(ui: &mut Ui<Msg>) {
    let (x, y) = middle(ui.spot("zone").unwrap().rect);
    assert!(click(ui, x, y).is_empty());
    assert!(ui.dropdown_open(), "a click opens the list");
}

fn lit(ui: &Ui<Msg>) -> Option<usize> {
    ui.drop.as_ref().unwrap().lit
}

#[test]
fn the_closed_box_is_as_tall_as_the_sheet_says_and_as_wide_as_its_answer() {
    let ui = ui_of(page(5, Some(2), 0.0));
    let r = ui.spot("zone").unwrap().rect;
    assert_eq!(r.h, theme::sheet().dropdown_height);
    let wide = ui_of(ww::column(vec![ww::dropdown(vec!["A much longer answer than the others".into(), "B".into()], Some(0), "Pick", Msg::Pick).key("zone")]));
    assert!(wide.spot("zone").unwrap().rect.w > r.w);
    let none = ui_of(page(5, None, 0.0));
    assert!(none.spot("zone").unwrap().rect.w > 60.0);
}

#[test]
fn opening_the_list_moves_and_resizes_nothing() {
    for n in [3, 5, 40] {
        let mut ui = ui_of(page(n, Some(1), 40.0));
        let before = ui.spots.clone();
        open(&mut ui);
        ui.settle();
        assert_eq!(ui.spots, before, "{n} choices: every widget is where it was");
        ui.input(&typed("7"), Instant::now());
        ui.settle();
        assert_eq!(ui.spots, before);
        ui.input(&key(Keysym::Escape), Instant::now());
        assert!(!ui.dropdown_open());
        assert_eq!(ui.spots, before);
    }
}

#[test]
fn the_list_opens_under_its_box_or_over_it_when_there_is_no_room_below() {
    let mut ui = ui_of(page(40, Some(0), 0.0));
    open(&mut ui);
    let (b, l) = (ui.spot("zone").unwrap().rect, ui.drop_laid().unwrap());
    assert!(!ui.drop.as_ref().unwrap().up);
    assert_eq!(l.panel.y, b.y + b.h + 4.0);
    assert_eq!(l.panel.x, b.x, "its left edge is the box's");
    assert!(l.panel.w >= b.w);
    let mut ui = ui_of(page(40, Some(0), 380.0));
    open(&mut ui);
    let (b, l) = (ui.spot("zone").unwrap().rect, ui.drop_laid().unwrap());
    assert!(ui.drop.as_ref().unwrap().up);
    assert_eq!(l.panel.y + l.panel.h, b.y - 4.0);
    ui.input(&typed("39"), Instant::now());
    let l = ui.drop_laid().unwrap();
    assert_eq!(l.shown, vec![39]);
    assert_eq!(l.panel.y + l.panel.h, b.y - 4.0);
    for above in [0.0, 100.0, 200.0, 300.0, 380.0, 420.0] {
        for n in [2, 6, 9, 40] {
            let mut ui = ui_of(page(n, None, above));
            open(&mut ui);
            let p = ui.drop_laid().unwrap().panel;
            assert!(p.x >= 0.0 && p.y >= 0.0 && p.x + p.w <= W as f32 && p.y + p.h <= H as f32, "{n} choices at {above}: {p:?}");
            let b = ui.spot("zone").unwrap().rect;
            assert!(p.y >= b.y + b.h || p.y + p.h <= b.y, "the list never covers its own box");
        }
    }
}

#[test]
fn the_side_is_chosen_by_the_room() {
    let b = |y: f32| Rect { x: 10.0, y, w: 100.0, h: 34.0 };
    assert!(!opens_up(b(10.0), 300.0, 600.0, 4.0, 4.0), "room below");
    assert!(opens_up(b(500.0), 300.0, 600.0, 4.0, 4.0), "no room below, room above");
    assert!(!opens_up(b(200.0), 500.0, 600.0, 4.0, 4.0));
    assert!(opens_up(b(400.0), 500.0, 600.0, 4.0, 4.0));
    assert!(!opens_up(b(400.0), 100.0, 600.0, 4.0, 4.0));
}

#[test]
fn a_list_at_the_right_edge_is_kept_inside_the_window() {
    let tree = ww::row(vec![ww::space(), ww::dropdown(vec!["No".into(), "A very long answer that is wider than its closed box by far".into()], Some(0), "Pick", Msg::Pick).key("zone")]).padding(10.0);
    let mut ui = ui_of(tree);
    open(&mut ui);
    let p = ui.drop_laid().unwrap().panel;
    assert!(p.x + p.w <= W as f32 - 4.0 + 0.5, "{p:?}");
    assert!(p.w > ui.spot("zone").unwrap().rect.w);
}

#[test]
fn the_keyboard_opens_moves_chooses_and_closes() {
    let now = Instant::now();
    let mut ui = ui_of(page(6, Some(2), 0.0));
    ui.focus = Some("zone".to_string());
    for sym in [Keysym::space, Keysym::Return, Keysym::Down, Keysym::Up] {
        assert!(ui.input(&key(sym), now).messages.is_empty());
        assert!(ui.dropdown_open(), "{sym:?} opens it");
        assert_eq!(lit(&ui), Some(2));
        assert!(ui.input(&key(Keysym::Escape), now).messages.is_empty());
        assert!(!ui.dropdown_open(), "Escape closes it");
        assert_eq!(ui.focus.as_deref(), Some("zone"), "and the keyboard stays on its box");
    }
    ui.input(&key(Keysym::space), now);
    ui.input(&key(Keysym::Down), now);
    assert_eq!(lit(&ui), Some(3));
    ui.input(&key(Keysym::End), now);
    assert_eq!(lit(&ui), Some(5));
    ui.input(&key(Keysym::Down), now);
    assert_eq!(lit(&ui), Some(5), "it stops at the end");
    ui.input(&key(Keysym::Home), now);
    assert_eq!(lit(&ui), Some(0));
    ui.input(&key(Keysym::Up), now);
    assert_eq!(lit(&ui), Some(0));
    ui.input(&key(Keysym::Down), now);
    assert_eq!(ui.input(&key(Keysym::Return), now).messages, vec![Msg::Pick(1)]);
    assert!(!ui.dropdown_open(), "Enter chooses and closes");
    ui.input(&key(Keysym::space), now);
    assert!(ui.input(&key(Keysym::Return), now).messages.is_empty());
    assert!(!ui.dropdown_open());
    ui.input(&key(Keysym::space), now);
    ui.input(&key(Keysym::Tab), now);
    assert!(!ui.dropdown_open());
    assert_eq!(ui.focus.as_deref(), Some("go"));
    ui.focus = Some("zone".to_string());
    ui.input(&key(Keysym::space), now);
    assert!(ui.menu_open());
}

#[test]
fn the_page_keys_move_a_page_and_keep_the_lit_row_in_sight() {
    let now = Instant::now();
    let mut ui = ui_of(page(40, Some(0), 0.0));
    ui.focus = Some("zone".to_string());
    ui.input(&key(Keysym::Down), now);
    let rows = theme::sheet().dropdown_rows as usize;
    ui.input(&key(Keysym::Page_Down), now);
    assert_eq!(lit(&ui), Some(rows));
    ui.input(&key(Keysym::End), now);
    assert_eq!(lit(&ui), Some(39));
    let (l, top) = (ui.drop_laid().unwrap(), ui.drop.as_ref().unwrap().top);
    assert_eq!(top, 40.0 * l.row - l.rows.h, "the last row shows");
    ui.input(&key(Keysym::Page_Up), now);
    assert_eq!(lit(&ui), Some(39 - rows));
    ui.input(&key(Keysym::Home), now);
    assert_eq!(ui.drop.as_ref().unwrap().top, 0.0);
    let mut ui = ui_of(page(40, Some(20), 0.0));
    open(&mut ui);
    let (l, top) = (ui.drop_laid().unwrap(), ui.drop.as_ref().unwrap().top);
    assert!(20.0 * l.row >= top && 21.0 * l.row <= top + l.rows.h);
    assert!(top > 0.0);
}

#[test]
fn a_long_list_has_a_search_box_with_a_count_and_typing_narrows_it() {
    let now = Instant::now();
    let rows = theme::sheet().dropdown_rows as usize;
    let mut short = ui_of(page(rows, Some(0), 0.0));
    open(&mut short);
    assert!(short.drop_laid().unwrap().search.is_none());
    assert_eq!(short.drop_laid().unwrap().shown.len(), rows);
    let mut ui = ui_of(page(rows + 1, Some(0), 0.0));
    open(&mut ui);
    assert!(ui.drop_laid().unwrap().search.is_some());
    let mut ui = ui_of(page(40, Some(0), 0.0));
    open(&mut ui);
    let l = ui.drop_laid().unwrap();
    assert!(l.search.is_some());
    assert_eq!(l.shown.len(), 40);
    assert_eq!(l.rows.h, rows as f32 * l.row, "eight rows show, the rest scroll");
    ui.input(&typed("3"), now);
    assert_eq!(ui.drop_laid().unwrap().shown.len(), 13);
    ui.input(&typed("7"), now);
    let l = ui.drop_laid().unwrap();
    assert_eq!(l.shown, vec![37]);
    assert_eq!(l.rows.h, l.row, "the list is as short as what is left");
    assert_eq!(lit(&ui), Some(37), "the first left is lit, ready for Enter");
    assert_eq!(ui.news, "Choice 37, 1 of 40", "and the screen reader hears the count");
    ui.input(&typed("x"), now);
    assert!(ui.drop_laid().unwrap().shown.is_empty());
    assert_eq!(ui.news, "Nothing matches");
    assert!(ui.input(&key(Keysym::Return), now).messages.is_empty(), "nothing to choose");
    open(&mut ui);
    ui.input(&typed("CHOICE 1"), now);
    assert_eq!(ui.drop_laid().unwrap().shown.len(), 11, "capitals do not matter");
    ui.input(&key(Keysym::BackSpace), now);
    assert_eq!(ui.drop_laid().unwrap().shown.len(), 40);
    ui.input(&typed("12"), now);
    assert_eq!(ui.input(&key(Keysym::Return), now).messages, vec![Msg::Pick(12)]);
    open(&mut ui);
    ui.input(&typed("choice"), now);
    let space = Input::Key { sym: Keysym::space, text: Some(" ".to_string()), mods: Mods::default(), repeat: false };
    assert!(ui.input(&space, now).messages.is_empty());
    assert!(ui.dropdown_open());
    ui.input(&typed("5"), now);
    assert_eq!(ui.drop_laid().unwrap().shown, vec![5]);
    assert_eq!(narrowed(&super::dropdown::folded(&names(40)), "  "), (0..40).collect::<Vec<_>>());
}

#[test]
fn typing_over_a_short_list_lights_the_answer_that_starts_with_it() {
    let now = Instant::now();
    let options = vec!["Never".to_string(), "After 5 minutes".into(), "After an hour".into(), "Tomorrow".into(), "When idle".into()];
    let mut ui = ui_of(ww::column(vec![ww::dropdown(options, Some(0), "Lock", Msg::Pick).key("zone")]).padding(10.0));
    open(&mut ui);
    assert!(ui.drop_laid().unwrap().search.is_none());
    ui.input(&typed("t"), now);
    assert_eq!(lit(&ui), Some(3));
    assert_eq!(ui.drop_laid().unwrap().shown.len(), 5, "every answer still shows");
    let later = now + Duration::from_secs(3);
    ui.input(&typed("a"), later);
    ui.input(&typed("f"), later);
    ui.input(&typed("ter an"), later);
    assert_eq!(lit(&ui), Some(2));
    assert_eq!(ui.input(&key(Keysym::Return), later).messages, vec![Msg::Pick(2)]);
}

#[test]
fn a_click_on_a_row_chooses_it_and_a_click_anywhere_else_only_closes() {
    let mut ui = ui_of(page(5, Some(0), 0.0));
    open(&mut ui);
    let l = ui.drop_laid().unwrap();
    let (x, y) = (l.rows.x + 20.0, l.rows.y + 3.5 * l.row);
    assert_eq!(click(&mut ui, x, y), vec![Msg::Pick(3)]);
    assert!(!ui.dropdown_open());
    open(&mut ui);
    let go = middle(ui.spot("go").unwrap().rect);
    let l = ui.drop_laid().unwrap();
    assert!(l.panel.contains(go.0, go.1), "the list floats over the button under it");
    let at = ((go.1 - l.rows.y) / l.row).floor() as usize;
    assert_eq!(click(&mut ui, go.0, go.1), vec![Msg::Pick(at)], "a click on the list is the list's, and presses nothing under it");
    let mut ui = ui_of(ww::row(vec![
        ww::dropdown(names(2), Some(0), "Time zone", Msg::Pick).key("zone"), ww::space(),
        ww::button("go-next-symbolic", "Go").on_press(Msg::Go).key("go"),
    ]).padding(10.0));
    open(&mut ui);
    let go = middle(ui.spot("go").unwrap().rect);
    assert!(!ui.drop_laid().unwrap().panel.contains(go.0, go.1));
    assert!(click(&mut ui, go.0, go.1).is_empty());
    assert!(!ui.dropdown_open());
    assert_eq!(click(&mut ui, go.0, go.1), vec![Msg::Go]);
    open(&mut ui);
    let (x, y) = middle(ui.spot("zone").unwrap().rect);
    assert!(click(&mut ui, x, y).is_empty());
    assert!(!ui.dropdown_open());
    open(&mut ui);
    ui.input(&Input::Focus(false), Instant::now());
    assert!(!ui.dropdown_open());
}

#[test]
fn the_wheel_scrolls_the_list_and_scrolling_the_page_closes_it() {
    let now = Instant::now();
    let tree = ww::scroll("area", ww::column(vec![
        ww::dropdown(names(40), Some(0), "Zone", Msg::Pick).key("zone"),
        ww::space().height(Length::Fixed(2000.0)),
    ]).padding(10.0));
    let mut ui = ui_of(tree);
    let before = ui.spots.clone();
    open(&mut ui);
    let l = ui.drop_laid().unwrap();
    let (x, y) = middle(l.rows);
    ui.input(&Input::PointerMove { x, y }, now);
    ui.input(&Input::Scroll { x: 0.0, y: 1.0, steps: true }, now);
    assert_eq!(ui.drop.as_ref().unwrap().top, 2.0 * l.row);
    ui.settle();
    assert_eq!(ui.spots, before, "the page under it stayed still");
    ui.input(&Input::PointerMove { x: 500.0, y: 450.0 }, now);
    ui.input(&Input::Scroll { x: 0.0, y: 1.0, steps: true }, now);
    assert!(!ui.dropdown_open());
    ui.settle();
    assert!(ui.spot("zone").unwrap().rect.y < before.iter().find(|s| s.id == "zone").unwrap().rect.y);
}

#[test]
fn the_bar_drags_the_rows() {
    let now = Instant::now();
    let mut ui = ui_of(page(40, Some(0), 0.0));
    open(&mut ui);
    let l = ui.drop_laid().unwrap();
    let x = l.rows.x + l.rows.w - 4.0;
    ui.input(&Input::PointerMove { x, y: l.rows.y + 6.0 }, now);
    ui.input(&Input::Press { button: Button::Left, x, y: l.rows.y + 6.0 }, now);
    ui.input(&Input::PointerMove { x, y: l.rows.y + l.rows.h }, now);
    assert!(ui.input(&Input::Release { button: Button::Left, x, y: l.rows.y + l.rows.h }, now).messages.is_empty());
    assert!(ui.dropdown_open(), "letting go of the bar chooses nothing");
    assert_eq!(ui.drop.as_ref().unwrap().top, 40.0 * l.row - l.rows.h);
}

#[test]
fn the_list_goes_when_its_dropdown_does() {
    let mut ui = ui_of(page(5, Some(0), 0.0));
    open(&mut ui);
    ui.set_view(page(5, Some(1), 0.0));
    ui.settle();
    assert!(ui.dropdown_open(), "a new view of the same page keeps it");
    ui.set_view(ww::column(vec![ww::label("Another page")]));
    ui.settle();
    assert!(!ui.dropdown_open());
}

#[test]
fn the_open_list_is_painted_over_the_page_in_the_box_colour_with_its_shadow() {
    let mut ui = ui_of(page(5, Some(1), 0.0));
    let mut closed = tiny_skia::Pixmap::new(W, H).unwrap();
    ui.paint(&mut closed.as_mut(), 1.0);
    open(&mut ui);
    ui.input(&Input::PointerOut, Instant::now());
    let mut pix = tiny_skia::Pixmap::new(W, H).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    let l = ui.drop_laid().unwrap();
    let at = |p: &tiny_skia::Pixmap, x: f32, y: f32| {
        let c = p.pixel(x as u32, y as u32).unwrap();
        (c.red(), c.green(), c.blue())
    };
    let hex = |c: Colour| ((c.r * 255.0).round() as u8, (c.g * 255.0).round() as u8, (c.b * 255.0).round() as u8);
    let near = |a: (u8, u8, u8), b: (u8, u8, u8)| a.0.abs_diff(b.0) <= 3 && a.1.abs_diff(b.1) <= 3 && a.2.abs_diff(b.2) <= 3;
    let plain = at(&pix, l.rows.x + l.rows.w - 8.0, l.rows.y + 3.5 * l.row);
    assert!(near(plain, hex(ui.theme.raised)), "{plain:?}");
    let kept = at(&pix, l.rows.x + l.rows.w - 8.0, l.rows.y + 1.5 * l.row);
    assert!(near(kept, hex(ui.theme.picked)), "{kept:?}");
    let (sx, sy) = (l.panel.x + l.panel.w / 2.0, l.panel.y + l.panel.h + 4.0);
    assert_ne!(at(&pix, sx, sy), at(&closed, sx, sy), "a shadow under the list");
    for (x, y) in [(W as f32 - 10.0, 20.0), (W as f32 - 10.0, H as f32 - 10.0), (4.0, 4.0)] {
        assert_eq!(at(&pix, x, y), at(&closed, x, y));
    }
}

#[test]
fn a_few_short_answers_are_a_strip_and_the_rest_a_dropdown() {
    let kind = |options: Vec<&str>, chosen: Option<usize>| {
        let n = ww::one_of(options.into_iter().map(String::from).collect(), chosen, "Pick", Msg::Pick);
        matches!(n.what, What::Dropdown { .. })
    };
    assert!(!kind(vec!["Dark", "Light"], Some(0)));
    assert!(!kind(vec!["One", "Two", "Three", "Four"], Some(3)));
    assert!(kind(vec!["One", "Two", "Three", "Four", "Five"], Some(0)), "more than four");
    assert!(!kind(vec!["A long answer that needs its room", "Another long answer too"], Some(0)), "two are never a dropdown");
    assert!(kind(vec!["A long answer that needs its room", "Another long answer too", "And a third"], Some(0)), "long answers");
    assert!(kind(vec!["Dark", "Light"], None), "nothing kept: the box says Choose one");
    assert!(kind(vec!["Dark", "Light"], Some(7)), "a place that is not there counts as nothing kept");
}

#[test]
fn alt_and_up_closes_the_list_and_the_screen_reader_hears_the_row_lit() {
    let now = Instant::now();
    let mut ui = ui_of(page(6, Some(2), 0.0));
    ui.focus = Some("zone".to_string());
    ui.input(&key(Keysym::space), now);
    ui.input(&key(Keysym::Down), now);
    assert_eq!(ui.news, "Choice 3");
    let alt = Input::Key { sym: Keysym::Up, text: None, mods: Mods { alt: true, ..Mods::default() }, repeat: false };
    assert!(ui.input(&alt, now).messages.is_empty());
    assert!(!ui.dropdown_open());
}

#[test]
fn a_composed_word_and_a_backspace_work_by_whole_letters_and_the_search_has_an_end() {
    let now = Instant::now();
    let mut ui = ui_of(page(40, Some(0), 0.0));
    open(&mut ui);
    ui.input(&Input::Composed { delete: (0, 0), preedit: String::new(), commit: "Cho".into(), caret: None }, now);
    assert_eq!(ui.drop.as_ref().unwrap().search, "Cho");
    ui.input(&typed("e\u{301}"), now);
    ui.input(&key(Keysym::BackSpace), now);
    assert_eq!(ui.drop.as_ref().unwrap().search, "Cho");
    let long = "x".repeat(500);
    ui.input(&typed(&long), now);
    ui.input(&typed(&long), now);
    assert!(ui.drop.as_ref().unwrap().search.chars().count() <= 103);
    let options = vec!["Never".to_string(), "Tomorrow".into()];
    let mut ui = ui_of(ww::column(vec![ww::dropdown(options, Some(0), "Lock", Msg::Pick).key("zone")]).padding(10.0));
    open(&mut ui);
    ui.input(&Input::Composed { delete: (0, 0), preedit: String::new(), commit: "To".into(), caret: None }, now);
    assert_eq!(lit(&ui), Some(1));
}
