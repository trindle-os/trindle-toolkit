//! The side panel that slides in beside the content.

use super::*;
use super::super::panel::{side_panel, Side};
use super::super::theme::Theme;
use super::super::widget as ww;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Pressed,
}

fn ui_at(shown: f32, side: Side) -> Ui<Msg> {
    let content = ww::card(ww::label("Content")).key("content");
    let panel = ww::card(ww::column(vec![ww::word_button("Close").on_press(Msg::Pressed).key("close")])).key("panel");
    let mut ui = Ui::new(side_panel(content, panel, 200.0, shown, side), Theme::new(true, "#fbc711"));
    ui.size = (600, 400);
    ui.settle();
    ui
}

#[test]
fn a_panel_fully_in_sits_beside_the_content_at_full_height_and_apart() {
    let ui = ui_at(1.0, Side::End);
    let (content, panel) = (ui.spot("content").unwrap().rect, ui.spot("panel").unwrap().rect);
    assert_eq!(panel.w, 200.0);
    assert_eq!(panel.h, 400.0, "it fills the height");
    assert_eq!(panel.x + panel.w, 600.0);
    assert!(content.x + content.w + 16.0 <= panel.x, "the content gives up the room and never touches it");
}

#[test]
fn a_panel_half_in_shows_its_leading_half_and_the_rest_is_cut_off() {
    let ui = ui_at(0.5, Side::End);
    let panel = ui.spot("panel").unwrap();
    assert_eq!(panel.rect.w, 200.0, "it keeps its width, so its words do not wrap again as it slides");
    assert_eq!(panel.rect.x, 500.0);
    assert_eq!(panel.clip.map(|c| c.x + c.w), Some(600.0));
    let start = ui_at(0.5, Side::Start);
    let panel = start.spot("panel").unwrap();
    assert_eq!(panel.rect.x, -100.0, "from the start side its far half shows first");
    assert!(start.spot("content").unwrap().rect.x >= 100.0);
}

#[test]
fn a_panel_slid_out_is_not_in_the_window_at_all() {
    let mut ui = ui_at(0.0, Side::End);
    assert!(ui.spot("panel").is_none());
    assert_eq!(ui.spot("content").unwrap().rect.w, 600.0);
    let told = ui.access("Test");
    assert!(!told.nodes.iter().any(|(_, n)| n.label() == Some("Close")), "the screen reader cannot reach it");
}

#[test]
fn what_is_in_the_panel_works_and_is_told() {
    let mut ui = ui_at(1.0, Side::End);
    let r = ui.spot("close").unwrap().rect;
    let now = std::time::Instant::now();
    ui.input(&Input::Press { button: Button::Left, x: r.x + r.w / 2.0, y: r.y + r.h / 2.0 }, now);
    let reply = ui.input(&Input::Release { button: Button::Left, x: r.x + r.w / 2.0, y: r.y + r.h / 2.0 }, now);
    assert_eq!(reply.messages, vec![Msg::Pressed]);
    let told = ui.access("Test");
    assert!(told.nodes.iter().any(|(_, n)| n.label() == Some("Close")));
}

#[test]
fn a_panel_is_painted_on_its_side() {
    let shot = |shown| {
        let mut ui = ui_at(shown, Side::End);
        let mut pix = tiny_skia::Pixmap::new(600, 400).unwrap();
        ui.paint(&mut pix.as_mut(), 1.0);
        pix
    };
    let (out, inn) = (shot(0.0), shot(1.0));
    let (a, b) = (out.data(), inn.data());
    let row = 600 * 4;
    let right = |d: &[u8]| (0..400).map(|y| d[y * row + 401 * 4..y * row + 600 * 4].to_vec()).collect::<Vec<_>>();
    assert_ne!(right(a), right(b), "the panel's card is painted at the right");
}
