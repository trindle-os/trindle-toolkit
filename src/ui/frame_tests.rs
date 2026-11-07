//! Tests of the title bar a program draws: moving, resizing by each edge, the window buttons, the double press and window menu, the shadow, the screen reader and painting.

use super::*;
use crate::frame::{framed, Frame, Framing};
use crate::window::Edges;
use super::super::widget::{label, Length};

fn frame_ui(maximised: bool) -> Ui<Framing> {
    framed_ui(Frame { maximised, ..Frame::default() })
}

fn framed_ui(frame: Frame) -> Ui<Framing> {
    let body = label::<Framing>("John Smith's notes").key("body").width(Length::Fill);
    let mut ui = Ui::new(framed("Notes", frame, body, |f| f), Theme::new(false, "#fbc711"));
    ui.size = (600, 400);
    ui.settle();
    ui
}

fn press(ui: &mut Ui<Framing>, x: f32, y: f32) -> Vec<Framing> {
    press_at(ui, x, y, Instant::now(), Button::Left)
}

fn press_at(ui: &mut Ui<Framing>, x: f32, y: f32, now: Instant, button: Button) -> Vec<Framing> {
    ui.input(&Input::PointerMove { x, y }, now);
    let mut got = ui.input(&Input::Press { button, x, y }, now).messages;
    got.extend(ui.input(&Input::Release { button, x, y }, now).messages);
    got
}

#[test]
fn the_bar_moves_the_window_and_the_edges_resize_it() {
    let mut ui = frame_ui(false);
    let edge = ui.px(theme::sheet().frame_edge);
    let bar = ui.px(theme::sheet().button + 2.0 * theme::sheet().title_bar_room);
    assert_eq!(press(&mut ui, edge + 20.0, edge + bar / 2.0), [Framing::Move]);
    assert_eq!(press(&mut ui, 300.0, edge + bar / 2.0), [Framing::Move]);
    for (x, y, e) in [(1.0, 200.0, Edges::LEFT), (599.0, 200.0, Edges::RIGHT), (300.0, 1.0, Edges::TOP), (300.0, 399.0, Edges::BOTTOM),
                      (1.0, 1.0, Edges::TOP | Edges::LEFT), (599.0, 399.0, Edges::BOTTOM | Edges::RIGHT)] {
        assert_eq!(press(&mut ui, x, y), [Framing::Resize(e)], "{x} {y}");
    }
    assert!(press(&mut ui, 300.0, 300.0).is_empty());
}

#[test]
fn the_window_buttons_minimise_maximise_and_close() {
    for (maximised, max_tip) in [(false, "Maximise"), (true, "Restore")] {
        let node = framed("Notes", Frame { maximised, ..Frame::default() }, label::<Framing>("body"), |f| f);
        assert_eq!(node.buttons(), [("Minimise".to_string(), true), (max_tip.to_string(), true), ("Close".to_string(), true)]);
    }
    let mut ui = frame_ui(true);
    for (key, did) in [("frame-minimise", Framing::Minimise), ("frame-maximise", Framing::Maximise), ("frame-close", Framing::Close)] {
        let r = ui.spot(key).unwrap().rect;
        assert_eq!(press(&mut ui, r.x + r.w / 2.0, r.y + r.h / 2.0), [did]);
    }
    assert_eq!(press(&mut ui, 1.0, 1.0), [Framing::Move]);
}

#[test]
fn the_bar_is_painted_apart_from_the_page_and_nothing_touches_the_edge() {
    let mut ui = frame_ui(false);
    ui.see_through = true;
    let mut pm = tiny_skia::Pixmap::new(600, 400).unwrap();
    ui.paint(&mut pm.as_mut(), 1.0);
    let edge = ui.px(theme::sheet().frame_edge) as u32;
    let bar = ui.px(theme::sheet().button + 2.0 * theme::sheet().title_bar_room) as u32;
    assert_ne!(pm.pixel(300, edge + bar / 2), pm.pixel(300, edge + bar + 20), "the bar stands apart from the body");
    assert_eq!(pm.pixel(300, edge + bar + 20).unwrap().alpha(), 255);
    let body = ui.spot("body").unwrap().rect;
    assert!(body.y >= (edge + bar) as f32 && body.x >= edge as f32);
}

#[test]
fn a_double_press_on_the_bar_maximises_and_two_slow_or_far_ones_move() {
    let mut ui = frame_ui(false);
    let y = ui.px(theme::sheet().frame_edge) + ui.px(theme::sheet().button) / 2.0;
    let now = Instant::now();
    assert_eq!(press_at(&mut ui, 300.0, y, now, Button::Left), [Framing::Move]);
    assert_eq!(press_at(&mut ui, 302.0, y, now + Duration::from_millis(150), Button::Left), [Framing::Maximise]);
    assert_eq!(press_at(&mut ui, 302.0, y, now + Duration::from_millis(300), Button::Left), [Framing::Move]);
    let later = now + MULTI_CLICK * 4;
    assert_eq!(press_at(&mut ui, 300.0, y, later + MULTI_CLICK * 2, Button::Left), [Framing::Move], "too slow is two presses");
    let at = later + MULTI_CLICK * 4;
    assert_eq!(press_at(&mut ui, 200.0, y, at, Button::Left), [Framing::Move]);
    assert_eq!(press_at(&mut ui, 260.0, y, at + Duration::from_millis(100), Button::Left), [Framing::Move], "too far is two presses");
}

#[test]
fn a_right_click_on_the_bar_asks_for_the_compositors_menu_where_it_has_one() {
    let mut ui = framed_ui(Frame { compositor_menu: true, ..Frame::default() });
    let edge = ui.px(theme::sheet().frame_edge);
    let (x, y) = (300.0, edge + 4.0);
    assert_eq!(press_at(&mut ui, x, y, Instant::now(), Button::Right), [Framing::Menu(x, y)], "the place is the window's, not the bar's");
    assert!(ui.copy_menu.is_none(), "the toolkit shows none of its own");
}

#[test]
fn without_the_compositors_menu_the_toolkit_shows_its_own() {
    for (maximised, words) in [(false, vec!["Maximise", "Minimise", "Move", "Resize", "Close"]), (true, vec!["Restore", "Minimise", "Move", "Close"])] {
        let mut ui = frame_ui(maximised);
        let y = if maximised { 4.0 } else { ui.px(theme::sheet().frame_edge) + 4.0 };
        assert!(press_at(&mut ui, 300.0, y, Instant::now(), Button::Right).is_empty());
        let menu = ui.copy_menu.as_ref().expect("a right click on the bar opens the window menu");
        assert_eq!(menu.popup.rows.iter().map(|r| r.words.as_str()).collect::<Vec<_>>(), words);
        assert!(menu.popup.rows.last().unwrap().rule, "Close stands apart");
    }
    let mut ui = frame_ui(false);
    let y = ui.px(theme::sheet().frame_edge) + 4.0;
    press_at(&mut ui, 300.0, y, Instant::now(), Button::Right);
    let rules: Vec<bool> = ui.copy_menu.as_ref().unwrap().popup.rows.iter().map(|r| r.rule).collect();
    let menu = ui.menu_rects().unwrap().0;
    let close = (menu.x + 30.0, menu.y + crate::popup::row_top(&rules, 4) + 4.0);
    assert_eq!(press_at(&mut ui, close.0, close.1, Instant::now(), Button::Left), [Framing::Close]);
}

#[test]
fn the_shadow_falls_in_its_room_and_the_edges_and_bar_sit_inside_it() {
    let room = 24.0;
    let mut ui = framed_ui(Frame { shadow: room, ..Frame::default() });
    let edge = ui.px(theme::sheet().frame_edge);
    assert!(press(&mut ui, room / 2.0, 200.0).is_empty());
    assert_eq!(press(&mut ui, room + 1.0, 200.0), [Framing::Resize(Edges::LEFT)]);
    assert_eq!(press(&mut ui, 600.0 - room - 1.0, 400.0 - room - 1.0), [Framing::Resize(Edges::BOTTOM | Edges::RIGHT)]);
    assert_eq!(press(&mut ui, 300.0, room + edge + 4.0), [Framing::Move]);
    let body = ui.spot("body").unwrap().rect;
    assert!(body.x >= room + edge && body.x + body.w <= 600.0 - room - edge);
    ui.see_through = true;
    let mut pm = tiny_skia::Pixmap::new(600, 400).unwrap();
    ui.paint(&mut pm.as_mut(), 1.0);
    let alpha = |y: u32| pm.pixel(300, y).unwrap().alpha();
    assert!(alpha(390) > 0 && alpha(390) < 255, "the shadow shows below the window");
    assert!(alpha(2) < alpha(390), "and falls more below than above");
    assert!(pm.pixel(1, 1).unwrap().alpha() < 16, "the far corner is all but clear");
    assert_eq!(alpha(300), 255, "the window is opaque");
}

#[test]
fn the_screen_reader_hears_the_title_once_and_the_buttons() {
    let mut ui = framed_ui(Frame { shadow: 24.0, ..Frame::default() });
    let names: Vec<String> = ui.access("Test").nodes.iter().filter_map(|(_, p)| p.label().map(str::to_string)).collect();
    assert_eq!(names.iter().filter(|n| *n == "Notes").count(), 1, "{names:?}");
    for b in ["Minimise", "Maximise", "Close"] {
        assert!(names.iter().any(|n| n == b), "{b} in {names:?}");
    }
    assert!(!names.iter().any(String::is_empty), "nothing unnamed: {names:?}");
}
