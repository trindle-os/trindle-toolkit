//! The gallery of pictures in a real window: keys, buttons, thumbnails, and what the screen reader is told.

use std::rc::Rc;

use super::*;
use super::super::photos::gallery;
use super::super::theme::Theme;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Show(usize),
}

fn slides(n: usize) -> Vec<(Rc<image::DynamicImage>, String)> {
    (0..n).map(|i| (Rc::new(image::DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(20, 10, image::Rgba([i as u8 * 40, 90, 200, 255])))), format!("Photo {}", i + 1))).collect()
}

fn ui_at(current: usize, n: usize) -> Ui<Msg> {
    let mut ui = Ui::new(gallery("trip", &slides(n), current, (300.0, 150.0), Msg::Show), Theme::new(true, "#fbc711"));
    ui.size = (400, 400);
    ui.settle();
    ui
}

fn key(sym: Keysym) -> Input {
    Input::Key { sym, text: None, mods: Mods::default(), repeat: false }
}

fn focused(current: usize, n: usize) -> Ui<Msg> {
    let mut ui = ui_at(current, n);
    let now = Instant::now();
    ui.input(&Input::PointerMove { x: 200.0, y: 60.0 }, now);
    ui.input(&Input::Press { button: Button::Left, x: 200.0, y: 60.0 }, now);
    ui.input(&Input::Release { button: Button::Left, x: 200.0, y: 60.0 }, now);
    ui
}

fn pressed(ui: &mut Ui<Msg>, sym: Keysym) -> Vec<Msg> {
    ui.input(&key(sym), Instant::now()).messages
}

fn labelled(ui: &mut Ui<Msg>) -> Vec<(String, Option<accesskit::Toggled>)> {
    ui.access("").nodes.iter().filter_map(|(_, n)| n.label().map(|l| (l.to_string(), n.toggled()))).collect()
}

#[test]
fn the_arrow_keys_home_and_end_move_through_the_pictures() {
    let mut ui = focused(2, 5);
    assert_eq!(pressed(&mut ui, Keysym::Right), vec![Msg::Show(3)]);
    assert_eq!(pressed(&mut ui, Keysym::Left), vec![Msg::Show(1)]);
    assert_eq!(pressed(&mut ui, Keysym::Home), vec![Msg::Show(0)]);
    assert_eq!(pressed(&mut ui, Keysym::End), vec![Msg::Show(4)]);
    assert_eq!(pressed(&mut ui, Keysym::Page_Down), vec![Msg::Show(3)]);
    assert_eq!(pressed(&mut ui, Keysym::Up), vec![Msg::Show(1)]);
}

#[test]
fn a_key_that_goes_nowhere_sends_nothing() {
    let mut first = focused(0, 3);
    assert!(pressed(&mut first, Keysym::Left).is_empty() && pressed(&mut first, Keysym::Home).is_empty());
    let mut last = focused(2, 3);
    assert!(pressed(&mut last, Keysym::Right).is_empty() && pressed(&mut last, Keysym::End).is_empty());
    assert!(pressed(&mut last, Keysym::from(0x61u32)).is_empty());
    let mut alone = focused(0, 1);
    assert!(pressed(&mut alone, Keysym::Right).is_empty());
}

#[test]
fn the_screen_reader_hears_the_name_and_the_place_of_the_large_picture_and_each_thumbnail() {
    let mut ui = ui_at(1, 3);
    let said = labelled(&mut ui);
    assert!(said.iter().any(|(l, _)| l == "Photo 2, 2 of 3"));
    let thumbs: Vec<_> = said.iter().filter(|(l, t)| l.starts_with("Photo") && t.is_some()).collect();
    assert_eq!(thumbs.len(), 3);
    assert_eq!(thumbs.iter().map(|(l, t)| (l.as_str(), *t)).collect::<Vec<_>>(),
               [("Photo 1, 1 of 3", Some(accesskit::Toggled::False)), ("Photo 2, 2 of 3", Some(accesskit::Toggled::True)),
                ("Photo 3, 3 of 3", Some(accesskit::Toggled::False))]);
    let count = ui.access("").nodes.iter().any(|(_, n)| n.value() == Some("2 of 3"));
    assert!(count, "the count is shown in words too");
}

#[test]
fn the_arrow_buttons_and_the_thumbnails_choose_a_picture() {
    let mut ui = ui_at(1, 4);
    let ids: Vec<_> = ui.access("").nodes.iter().filter_map(|(id, n)| n.label().map(|l| (*id, l.to_string()))).collect();
    let id_of = |name: &str| ids.iter().find(|(_, l)| l == name).unwrap().0;
    for (name, want) in [("Next picture", 2), ("Previous picture", 0), ("Photo 4, 4 of 4", 3)] {
        let asked = accesskit::ActionRequest { action: accesskit::Action::Click, target_tree: accesskit::TreeId::ROOT, target_node: id_of(name), data: None };
        assert_eq!(ui.act(&asked).messages, vec![Msg::Show(want)], "{name}");
    }
}

#[test]
fn the_end_buttons_cannot_be_pressed_and_an_empty_gallery_is_nothing() {
    for (current, dead) in [(0, "Previous picture"), (2, "Next picture")] {
        let mut ui = ui_at(current, 3);
        let t = ui.access("");
        let node = t.nodes.iter().find(|(_, n)| n.label() == Some(dead)).unwrap().1.clone();
        assert!(node.is_disabled(), "{dead}");
    }
    assert!(labelled(&mut ui_at(0, 0)).iter().all(|(l, _)| !l.starts_with("Photo") && !l.contains("picture")));
    assert_eq!(labelled(&mut ui_at(9, 3)).iter().filter(|(l, _)| l == "Photo 3, 3 of 3").count(), 2, "a place past the end shows the last");
}

#[test]
fn the_large_picture_follows_the_choice() {
    let mut a = ui_at(0, 3);
    let mut b = ui_at(2, 3);
    let (mut pa, mut pb) = (tiny_skia::Pixmap::new(400, 400).unwrap(), tiny_skia::Pixmap::new(400, 400).unwrap());
    a.paint(&mut pa.as_mut(), 1.0);
    b.paint(&mut pb.as_mut(), 1.0);
    assert_ne!(pa.data(), pb.data());
}
