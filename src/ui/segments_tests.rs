//! The segmented time box: keys, typing, pointer, painting and the screen reader.

use super::*;
use super::super::segments::{SegMsg, SegmentedTime};
use super::super::theme::Theme;
use super::super::when::Time;
use super::super::widget::{self as ww};
use accesskit::{Action, Role};

fn view(b: &SegmentedTime) -> Node<SegMsg> {
    ww::column(vec![b.node("Start", |m| m).key("time")]).padding(10.0)
}

fn ui_of(b: &SegmentedTime) -> Ui<SegMsg> {
    let mut ui = Ui::new(view(b), Theme::new(true, "#fbc711"));
    ui.size = (300, 120);
    ui.settle();
    shot(&mut ui);
    ui.set_view(view(b));
    ui.settle();
    ui
}

fn shot(ui: &mut Ui<SegMsg>) -> tiny_skia::Pixmap {
    let mut pix = tiny_skia::Pixmap::new(300, 120).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix
}

fn press(ui: &mut Ui<SegMsg>, sym: Keysym, shift: bool) -> Vec<SegMsg> {
    ui.input(&Input::Key { sym, text: None, mods: Mods { shift, ..Mods::default() }, repeat: false }, Instant::now()).messages
}

fn typed(ui: &mut Ui<SegMsg>, figure: &str) -> Vec<SegMsg> {
    ui.input(&Input::Key { sym: Keysym::NoSymbol, text: Some(figure.into()), mods: Mods::default(), repeat: false }, Instant::now()).messages
}

fn at(h: u32, m: u32) -> SegmentedTime {
    SegmentedTime::new(Time::new(h, m).unwrap())
}

#[test]
fn up_and_down_step_the_lit_segment_going_round_and_the_page_keys_step_more() {
    let mut b = at(23, 0);
    let mut ui = ui_of(&b);
    press(&mut ui, Keysym::Tab, false);
    assert_eq!(press(&mut ui, Keysym::Up, false), vec![SegMsg::Step(1)]);
    assert!(b.update(SegMsg::Step(1)));
    assert_eq!(b.time, Time { hour: 0, minute: 0 }, "23 goes round to 0");
    b.update(SegMsg::Along(1));
    b.update(SegMsg::Step(-1));
    assert_eq!(b.time.minute, 59);
    assert_eq!(press(&mut ui, Keysym::Page_Up, false), vec![SegMsg::Step(6)], "hours go by six");
    let minutes = ui_of(&b);
    let mut minutes = minutes;
    press(&mut minutes, Keysym::Tab, false);
    assert_eq!(press(&mut minutes, Keysym::Page_Down, false), vec![SegMsg::Step(-10)]);
    assert_eq!(press(&mut ui, Keysym::Home, false), vec![SegMsg::End(false)]);
    assert_eq!(press(&mut ui, Keysym::End, false), vec![SegMsg::End(true)]);
    b.update(SegMsg::End(true));
    assert_eq!(b.time.minute, 59);
    b.update(SegMsg::End(false));
    assert_eq!(b.time.minute, 0);
}

#[test]
fn left_right_and_tab_take_a_segment_and_tab_leaves_after_the_last() {
    let mut b = at(7, 5);
    let mut ui = ui_of(&b);
    press(&mut ui, Keysym::Tab, false);
    assert_eq!(press(&mut ui, Keysym::Right, false), vec![SegMsg::Along(1)]);
    assert_eq!(press(&mut ui, Keysym::Tab, false), vec![SegMsg::Along(1)], "Tab goes on to the minutes first");
    b.update(SegMsg::Along(1));
    assert_eq!(b.lit, 1);
    let mut ui = ui_of(&b);
    press(&mut ui, Keysym::Tab, false);
    assert_eq!(press(&mut ui, Keysym::Tab, false), Vec::<SegMsg>::new(), "from the minutes Tab leaves the box");
    assert_eq!(press(&mut ui, Keysym::ISO_Left_Tab, true), vec![SegMsg::Along(-1)]);
    b.update(SegMsg::Along(5));
    assert_eq!(b.lit, 1, "held at the last segment");
    b.update(SegMsg::Along(-5));
    assert_eq!(b.lit, 0);
}

#[test]
fn figures_typed_fill_a_segment_and_go_on_to_the_next() {
    let mut b = at(0, 0);
    let mut ui = ui_of(&b);
    press(&mut ui, Keysym::Tab, false);
    assert_eq!(typed(&mut ui, "0"), vec![SegMsg::Digit(0)]);
    assert!(typed(&mut ui, "ab").is_empty(), "only one figure at a time");
    for d in [0, 7] {
        b.update(SegMsg::Digit(d));
    }
    assert_eq!((b.time.hour, b.lit), (7, 1), "07 is hours, and the minutes are taken");
    for d in [4, 5] {
        b.update(SegMsg::Digit(d));
    }
    assert_eq!(b.time, Time { hour: 7, minute: 45 });
    let mut c = at(0, 0);
    c.update(SegMsg::Digit(9));
    assert_eq!((c.time.hour, c.lit), (9, 1));
    c.update(SegMsg::Digit(7));
    assert_eq!((c.time.minute, c.lit), (7, 1), "minutes 7 is too big to wait for a second figure");
    let mut d = at(0, 0);
    d.update(SegMsg::Digit(2));
    d.update(SegMsg::Digit(9));
    assert_eq!(d.time.hour, 23);
}

#[test]
fn a_press_takes_the_segment_under_it() {
    let mut b = at(7, 5);
    let mut ui = ui_of(&b);
    let r = ui.spot("time").unwrap().rect;
    let (y, now) = (r.y + r.h / 2.0, Instant::now());
    let x = r.x + r.w - 6.0;
    ui.input(&Input::PointerMove { x, y }, now);
    let sent = ui.input(&Input::Press { button: Button::Left, x, y }, now).messages;
    assert_eq!(sent.len(), 1);
    let SegMsg::Press(across) = sent[0] else { panic!("{sent:?}") };
    assert!(!b.update(SegMsg::Press(across)));
    assert_eq!(b.lit, 1, "the right end is the minutes");
    let first = ui.input(&Input::Press { button: Button::Left, x: r.x + 14.0, y }, now).messages;
    let SegMsg::Press(left) = first[0] else { panic!("{first:?}") };
    b.update(SegMsg::Press(left));
    assert_eq!(b.lit, 0);
}

#[test]
fn the_lit_segment_and_the_time_change_the_picture() {
    let (mut a, mut b, mut c) = (ui_of(&at(7, 5)), ui_of(&{ let mut x = at(7, 5); x.update(SegMsg::Along(1)); x }), ui_of(&at(8, 5)));
    let (pa, pb, pc) = (shot(&mut a), shot(&mut b), shot(&mut c));
    assert_ne!(pa.data(), pb.data(), "the lit segment moved");
    assert_ne!(pa.data(), pc.data(), "the hours changed");
}

#[test]
fn the_reader_hears_two_spin_buttons_and_can_step_and_take_them() {
    let b = at(7, 5);
    let mut ui = ui_of(&b);
    let told = ui.access("Test");
    let spins: Vec<(Option<&str>, Option<f64>, Option<f64>)> = told.nodes.iter().filter(|(_, p)| p.role() == Role::SpinButton)
        .map(|(_, p)| (p.label(), p.numeric_value(), p.max_numeric_value())).collect();
    assert_eq!(spins, vec![(Some("Start Hours"), Some(7.0), Some(23.0)), (Some("Start Minutes"), Some(5.0), Some(59.0))]);
    assert!(told.nodes.iter().any(|(_, p)| p.role() == Role::Group && p.label() == Some("Start")));
    let ask = |ui: &mut Ui<SegMsg>, label: &str, action: Action| {
        let id = ui.access("Test").nodes.iter().find(|(_, p)| p.label() == Some(label)).unwrap().0;
        ui.act(&accesskit::ActionRequest { action, target_tree: accesskit::TreeId::ROOT, target_node: id, data: None }).messages
    };
    assert_eq!(ask(&mut ui, "Start Hours", Action::Increment), vec![SegMsg::Step(1)]);
    assert_eq!(ask(&mut ui, "Start Minutes", Action::Increment), vec![SegMsg::Along(1), SegMsg::Step(1)]);
    assert_eq!(ask(&mut ui, "Start Minutes", Action::Focus), vec![SegMsg::Along(1)]);
}
