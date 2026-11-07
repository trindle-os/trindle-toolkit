//! What a program that paints its own grid needs of the toolkit: keyboard on open, copy shortcuts, scroll-back, selecting.

use super::*;
use super::super::theme::Theme;
use super::super::widget::{self as ww, Length, Pointer};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Key(String),
    Lines(f32),
    Point(Pointer),
    Other,
}

fn grid() -> Node<Msg> {
    ww::drawing((300.0, 200.0), "Terminal", |_, _| {})
        .on_keys(|i| match i {
            Input::Key { text: Some(t), .. } => Some(Msg::Key(t.clone())),
            _ => None,
        })
        .on_scroll(Msg::Lines)
        .on_pointer(Msg::Point)
        .first()
        .width(Length::Fill)
        .height(Length::Fill)
}

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(true, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui
}

fn typed(t: &str) -> Input {
    Input::Key { sym: Keysym::from(0x61u32), text: Some(t.into()), mods: Mods::default(), repeat: false }
}

#[test]
fn a_drawing_marked_first_takes_the_keyboard_without_a_click() {
    let mut ui = ui_of(ww::column(vec![ww::button("go-next-symbolic", "Other").on_press(Msg::Other), grid()]));
    let reply = ui.input(&typed("a"), Instant::now());
    assert_eq!(reply.messages, vec![Msg::Key("a".into())]);
}

#[test]
fn without_first_a_drawing_waits_for_a_click() {
    let tree = ww::drawing((300.0, 200.0), "Terminal", |_, _| {}).on_keys(|_| Some(Msg::Other)).width(Length::Fill).height(Length::Fill);
    let mut ui = ui_of(tree);
    assert!(ui.input(&typed("a"), Instant::now()).messages.is_empty());
}

#[test]
fn the_wheel_over_a_drawing_scrolls_it_in_lines() {
    let mut ui = ui_of(grid());
    let now = Instant::now();
    ui.input(&Input::PointerMove { x: 100.0, y: 100.0 }, now);
    let r = ui.input(&Input::Scroll { x: 0.0, y: 1.0, steps: true }, now);
    assert_eq!(r.messages, vec![Msg::Lines(3.0)]);
    let r = ui.input(&Input::Scroll { x: 0.0, y: -12.0, steps: false }, now);
    assert_eq!(r.messages, vec![Msg::Lines(-0.5)]);
}

#[test]
fn the_pointer_drags_a_selection_across_a_drawing() {
    let mut ui = ui_of(grid());
    let now = Instant::now();
    ui.input(&Input::PointerMove { x: 20.0, y: 20.0 }, now);
    let down = ui.input(&Input::Press { button: Button::Left, x: 20.0, y: 20.0 }, now);
    let moved = ui.input(&Input::PointerMove { x: 80.0, y: 40.0 }, now);
    let up = ui.input(&Input::Release { button: Button::Left, x: 80.0, y: 40.0 }, now);
    assert!(down.messages.iter().any(|m| matches!(m, Msg::Point(Pointer::Down(..)))), "{:?}", down.messages);
    assert!(moved.messages.iter().any(|m| matches!(m, Msg::Point(Pointer::Move(..)))), "{:?}", moved.messages);
    assert!(up.messages.iter().any(|m| matches!(m, Msg::Point(Pointer::Up(..)))), "{:?}", up.messages);
}

struct Shell {
    copy: Option<String>,
    title: Option<String>,
}

impl App for Shell {
    type Msg = Msg;
    fn update(&mut self, _: Msg) {}
    fn view(&self) -> Node<Msg> {
        grid()
    }
    fn to_clipboard(&mut self) -> Option<String> {
        self.copy.take()
    }
    fn window_title(&self) -> Option<String> {
        self.title.clone()
    }
}

#[test]
fn words_for_the_clipboard_are_handed_over_once() {
    let mut app = Shell { copy: Some("ls -l".into()), title: Some("vim".into()) };
    assert_eq!(outgoing(&mut app), (Some("ls -l".into()), Some("vim".into())));
    assert_eq!(outgoing(&mut app), (None, Some("vim".into())));
}
