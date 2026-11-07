//! Things dragged between a window and other programs, as the widgets see them: what would take what, where it lands, and what may leave.

use std::path::PathBuf;

use super::*;
use super::super::dnd::{Given, Kind, Offered, Ways, Why};
use super::super::drag_test::{harness, Mouse, Plain, Stage, LEFT_BUTTON};
use super::super::theme::Theme;
use super::super::widget::{self as ww, Held, Part};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Name(String),
    Pass(String),
    Row(usize),
    OnRow(usize, Vec<PathBuf>, Way),
    OnDrawing(i32, i32, Given),
    OnCard(Given),
    Point,
}

fn view(name: &str, pass: &str) -> Node<Msg> {
    let files = |held: &Held| if held.offered.files { held.usual() } else { None };
    ww::column(vec![
        ww::entry(name, "Name", Msg::Name).key("name"),
        ww::entry(pass, "Password", Msg::Pass).secret().key("pass"),
        ww::label("Words to drag away").key("words"),
        ww::list(vec![ww::item("Home"), ww::item("Documents"), ww::item("Trash")], None, Msg::Row)
            .on_drop_from_outside(
                move |held| match held.part {
                    Part::Row(0 | 1) | Part::Gap(3) => files(held),
                    _ => None,
                },
                |held, given, way| match (held.part, given) {
                    (Part::Row(row), Given::Files(f)) => Msg::OnRow(row, f, way),
                    (Part::Gap(gap), Given::Files(f)) => Msg::OnRow(100 + gap, f, way),
                    _ => Msg::Point,
                })
            .key("places"),
        ww::card(ww::column(vec![
            ww::label("In the card"),
            ww::drawing((200.0, 60.0), "", |_, _| {}).on_pointer(|_| Msg::Point).on_hover()
                .on_drop_from_outside(|held| if held.offered.files { Some(Way::Copy) } else { None },
                                      |held, given, _| match held.part {
                                          Part::Point(x, y) => Msg::OnDrawing(x as i32, y as i32, given),
                                          _ => Msg::Point,
                                      })
                .key("drawing"),
        ])).on_drop_from_outside(|_| Some(Way::Copy), |_, given, _| Msg::OnCard(given)).key("card"),
    ])
}

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(false, "#fbc711"));
    ui.size = (500, 500);
    ui.settle();
    ui
}

fn middle(ui: &Ui<Msg>, id: &str) -> (f32, f32) {
    let r = ui.spot(id).unwrap().rect;
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn offered(files: bool, text: bool, ways: Ways) -> Offered {
    Offered { files, text, ways }
}

fn over(ui: &mut Ui<Msg>, at: (f32, f32), what: Offered) -> Reply<Msg> {
    ui.input(&Input::DragIn { x: at.0, y: at.1, offered: what, kinds: Vec::new() }, Instant::now())
}

fn let_go(ui: &mut Ui<Msg>, at: (f32, f32), given: Given, way: Way) -> Reply<Msg> {
    ui.input(&Input::Dropped { x: at.0, y: at.1, given, way, id: 9 }, Instant::now())
}

fn paths(list: &[&str]) -> Vec<PathBuf> {
    list.iter().map(PathBuf::from).collect()
}

#[test]
fn text_let_go_on_a_text_box_goes_in_at_the_pointer() {
    let mut ui = ui_of(view("hello", ""));
    let r = ui.spot("name").unwrap().rect;
    let reply = over(&mut ui, (r.x + r.w - 6.0, r.y + r.h / 2.0), offered(false, true, Ways::COPY));
    assert_eq!(reply.wanted, Some(Some((Way::Copy, Kind::Text))));
    assert_eq!(ui.hover.as_deref(), Some("name"));
    let reply = let_go(&mut ui, (r.x + r.w - 6.0, r.y + r.h / 2.0), Given::Text(" there\nyou".into()), Way::Copy);
    assert_eq!(reply.messages, vec![Msg::Name("hello there you".into())]);
    assert_eq!(reply.dropped, Some((9, Way::Copy, true)));
    assert_eq!(ui.focus.as_deref(), Some("name"), "the keyboard goes where the words went");
    assert!(ui.hover.is_none() && ui.outside.is_none());
    let mut ui = ui_of(view("hello", ""));
    over(&mut ui, (r.x + 1.0, r.y + r.h / 2.0), offered(false, true, Ways::COPY));
    let reply = let_go(&mut ui, (r.x + 1.0, r.y + r.h / 2.0), Given::Text("oh ".into()), Way::Copy);
    assert_eq!(reply.messages, vec![Msg::Name("oh hello".into())]);
}

#[test]
fn a_text_box_takes_text_and_turns_away_what_has_none() {
    let mut ui = ui_of(view("", ""));
    let at = middle(&ui, "name");
    assert_eq!(over(&mut ui, at, offered(true, true, Ways::ALL)).wanted, Some(Some((Way::Copy, Kind::Text))));
    assert_eq!(over(&mut ui, at, offered(true, false, Ways::ALL)).wanted, Some(None));
    assert_eq!(over(&mut ui, at, offered(false, true, Ways::ASK)).wanted, Some(None));
    assert_eq!(over(&mut ui, at, offered(false, true, Ways::MOVE)).wanted, Some(Some((Way::Move, Kind::Text))));
    over(&mut ui, at, offered(false, true, Ways::COPY));
    let reply = let_go(&mut ui, at, Given::Nothing(Why::TooBig), Way::Copy);
    assert!(reply.messages.is_empty());
    assert_eq!(reply.dropped, Some((9, Way::Copy, false)));
    let words = middle(&ui, "words");
    assert_eq!(over(&mut ui, words, offered(true, true, Ways::ALL)).wanted, Some(None));
    let mut ui = ui_of(ww::column(vec![ww::entry("", "Name", Msg::Name).key("name").off(true)]));
    let at = middle(&ui, "name");
    assert_eq!(over(&mut ui, at, offered(false, true, Ways::COPY)).wanted, Some(None));
}

#[test]
fn a_secret_box_takes_words_in_and_lets_none_out() {
    let mut ui = ui_of(view("", "abc"));
    let r = ui.spot("pass").unwrap().rect;
    let at = (r.x + 3.0, r.y + r.h / 2.0);
    assert_eq!(over(&mut ui, at, offered(false, true, Ways::COPY)).wanted, Some(Some((Way::Copy, Kind::Text))));
    let reply = let_go(&mut ui, at, Given::Text("xyz".into()), Way::Copy);
    assert_eq!(reply.messages, vec![Msg::Pass("xyzabc".into())]);
    let mut ui = ui_of(view("", "xyzabc"));
    let now = Instant::now();
    ui.input(&Input::Press { button: Button::Left, x: at.0, y: at.1 }, now);
    ui.input(&Input::Release { button: Button::Left, x: at.0, y: at.1 }, now);
    ui.input(&Input::Key { sym: Keysym::a, text: None, mods: Mods { ctrl: true, ..Mods::default() }, repeat: false }, now);
    assert_eq!(ui.edits["pass"].selection(), 0..6);
    let later = now + MULTI_CLICK * 2;
    ui.input(&Input::Press { button: Button::Left, x: at.0 + 10.0, y: at.1 }, later);
    let reply = ui.input(&Input::PointerMove { x: at.0 + 60.0, y: at.1 + 30.0 }, later);
    assert_eq!(reply.lift, None);
    assert!(!ui.lifting());
}

#[test]
fn selected_words_in_a_text_box_are_dragged_out_and_a_click_on_them_is_still_a_click() {
    let mut ui = ui_of(view("hello there", ""));
    let r = ui.spot("name").unwrap().rect;
    let at = (r.x + 14.0, r.y + r.h / 2.0);
    let now = Instant::now();
    for _ in 0..2 {
        ui.input(&Input::Press { button: Button::Left, x: at.0, y: at.1 }, now);
        ui.input(&Input::Release { button: Button::Left, x: at.0, y: at.1 }, now);
    }
    assert_eq!(ui.edits["name"].selected(), "hello");
    let later = now + MULTI_CLICK * 2;
    ui.input(&Input::Press { button: Button::Left, x: at.0, y: at.1 }, later);
    assert_eq!(ui.edits["name"].selected(), "hello", "the press alone keeps what is selected");
    assert_eq!(ui.input(&Input::PointerMove { x: at.0 + 2.0, y: at.1 }, later).lift, None, "not yet far enough");
    let reply = ui.input(&Input::PointerMove { x: at.0 + 40.0, y: at.1 + 40.0 }, later);
    assert_eq!(reply.lift.as_deref(), Some("hello"));
    assert!(ui.lifting());
    assert_eq!(ui.input(&Input::PointerMove { x: at.0 + 80.0, y: at.1 + 80.0 }, later).lift, None);
    assert_eq!(ui.edits["name"].selected(), "hello");
    ui.input(&Input::DragEnded { took: Some(crate::dnd::Taken::made_up(Way::Copy)), short: None }, later);
    assert!(!ui.lifting() && ui.dragging.is_none() && ui.pressed.is_none());
    assert_eq!(ui.edits["name"].text, "hello there", "a copy leaves the words where they were");
    let last = later + MULTI_CLICK * 2;
    ui.input(&Input::Press { button: Button::Left, x: at.0, y: at.1 }, last);
    ui.input(&Input::Release { button: Button::Left, x: at.0, y: at.1 }, last);
    assert!(ui.edits["name"].selection().is_empty());
    ui.input(&Input::Press { button: Button::Left, x: at.0, y: at.1 }, last + MULTI_CLICK * 2);
    let reply = ui.input(&Input::PointerMove { x: at.0 + 40.0, y: at.1 }, last + MULTI_CLICK * 2);
    assert_eq!(reply.lift, None);
    assert!(!ui.edits["name"].selection().is_empty());
}

#[test]
fn words_dragged_within_their_own_box_are_moved() {
    let lifted = |name: &str| {
        let mut ui = ui_of(view(name, ""));
        let r = ui.spot("name").unwrap().rect;
        let at = (r.x + 14.0, r.y + r.h / 2.0);
        let now = Instant::now();
        for _ in 0..2 {
            ui.input(&Input::Press { button: Button::Left, x: at.0, y: at.1 }, now);
            ui.input(&Input::Release { button: Button::Left, x: at.0, y: at.1 }, now);
        }
        let later = now + MULTI_CLICK * 2;
        ui.input(&Input::Press { button: Button::Left, x: at.0, y: at.1 }, later);
        let reply = ui.input(&Input::PointerMove { x: at.0 + 40.0, y: at.1 + 40.0 }, later);
        assert_eq!(reply.lift.as_deref(), Some("hello"));
        (ui, r, at)
    };
    let (mut ui, r, _) = lifted("hello there");
    let end = (r.x + r.w - 6.0, r.y + r.h / 2.0);
    over(&mut ui, end, offered(false, true, Ways::COPY));
    let reply = let_go(&mut ui, end, Given::Text("hello".into()), Way::Copy);
    assert_eq!(reply.messages, vec![Msg::Name(" therehello".into())]);
    let (mut ui, r, _) = lifted("hello there");
    let end = (r.x + r.w - 6.0, r.y + r.h / 2.0);
    over(&mut ui, end, offered(false, true, Ways::ALL));
    let reply = let_go(&mut ui, end, Given::Text("hello".into()), Way::Copy);
    assert_eq!(reply.messages, vec![Msg::Name("hello therehello".into())]);
    let (mut ui, _, at) = lifted("hello there");
    over(&mut ui, at, offered(false, true, Ways::COPY));
    let reply = let_go(&mut ui, at, Given::Text("hello".into()), Way::Copy);
    assert!(reply.messages.is_empty());
    assert_eq!(ui.edits["name"].text, "hello there");
}

#[test]
fn selected_words_of_a_label_are_dragged_out() {
    let mut ui = ui_of(view("", ""));
    let r = ui.spot("words").unwrap().rect;
    let at = (r.x + 8.0, r.y + r.h / 2.0);
    let now = Instant::now();
    for _ in 0..2 {
        ui.input(&Input::Press { button: Button::Left, x: at.0, y: at.1 }, now);
        ui.input(&Input::Release { button: Button::Left, x: at.0, y: at.1 }, now);
    }
    assert_eq!(ui.picked.as_ref().map(|(_, e)| e.selected()), Some("Words"));
    let later = now + MULTI_CLICK * 2;
    ui.input(&Input::Press { button: Button::Left, x: at.0, y: at.1 }, later);
    let reply = ui.input(&Input::PointerMove { x: at.0 + 30.0, y: at.1 + 30.0 }, later);
    assert_eq!(reply.lift.as_deref(), Some("Words"));
    ui.input(&Input::DragEnded { took: None, short: None }, later);
    assert!(!ui.lifting());
}

#[test]
fn a_list_is_told_the_row_and_lights_it_as_for_anything_held() {
    let mut ui = ui_of(view("", ""));
    let r = ui.spot("places").unwrap().rect;
    let row = ui.row_of("places");
    let at = |i: usize| (r.x + 20.0, r.y + row * (i as f32 + 0.5));
    let reply = over(&mut ui, at(1), offered(true, true, Ways::ALL));
    assert_eq!(reply.wanted, Some(Some((Way::Copy, Kind::Files))));
    assert_eq!(ui.hover_row(), Some(("places".to_string(), 1)), "the row is lit as when the pointer rests there");
    assert!(reply.redraw);
    let again = ui.input(&Input::DragOver { x: at(1).0, y: at(1).1 }, Instant::now());
    assert!(!again.redraw);
    assert_eq!(again.wanted, Some(Some((Way::Copy, Kind::Files))));
    let reply = ui.input(&Input::DragOver { x: at(2).0, y: at(2).1 }, Instant::now());
    assert_eq!(reply.wanted, Some(None));
    assert!(ui.hover.is_none());
    ui.input(&Input::DragOver { x: at(0).0, y: at(0).1 }, Instant::now());
    let reply = let_go(&mut ui, at(0), Given::Files(paths(&["/home/john/a.txt"])), Way::Copy);
    assert_eq!(reply.messages, vec![Msg::OnRow(0, paths(&["/home/john/a.txt"]), Way::Copy)]);
    assert_eq!(reply.dropped, Some((9, Way::Copy, true)));
    assert!(ui.hover.is_none(), "nothing stays lit");
    assert_eq!(over(&mut ui, at(0), offered(true, false, Ways::MOVE)).wanted, Some(Some((Way::Move, Kind::Files))));
    assert_eq!(over(&mut ui, at(0), offered(true, false, Ways::ASK | Ways::COPY)).wanted, Some(Some((Way::Ask, Kind::Files))));
    let edge = |i: usize| (r.x + 20.0, r.y + row * i as f32 + 1.0);
    assert_eq!(over(&mut ui, edge(1), offered(true, false, Ways::COPY)).wanted, Some(Some((Way::Copy, Kind::Files))));
    assert_eq!((ui.gap.clone(), ui.hover_row()), (None, Some(("places".to_string(), 1))), "the gap is not taken, so the row is");
    let foot = (r.x + 20.0, r.y + row * 3.0 - 1.0);
    assert_eq!(over(&mut ui, foot, offered(true, false, Ways::COPY)).wanted, Some(Some((Way::Copy, Kind::Files))));
    assert_eq!((ui.gap.clone(), ui.hover.clone()), (Some(("places".to_string(), 3)), None));
    let reply = let_go(&mut ui, foot, Given::Files(paths(&["/home/john/Photos"])), Way::Copy);
    assert_eq!(reply.messages, vec![Msg::OnRow(103, paths(&["/home/john/Photos"]), Way::Copy)]);
    assert!(ui.gap.is_none(), "the line goes when it is let go");
    assert_eq!(over(&mut ui, at(0), offered(false, true, Ways::ALL)).wanted, Some(None));
    let reply = ui.input(&Input::DragGone, Instant::now());
    assert!(ui.outside.is_none() && ui.hover.is_none() && reply.wanted.is_none());
}

#[test]
fn the_innermost_widget_takes_it_and_what_lies_over_keeps_it_off_what_is_under() {
    let mut ui = ui_of(view("", ""));
    let d = ui.spot("drawing").unwrap().rect;
    let reply = over(&mut ui, (d.x + 30.0, d.y + 20.0), offered(true, false, Ways::COPY));
    assert_eq!(reply.wanted, Some(Some((Way::Copy, Kind::Files))));
    assert_eq!(reply.messages, vec![Msg::Point]);
    let reply = let_go(&mut ui, (d.x + 30.0, d.y + 20.0), Given::Files(paths(&["/home/john/a.txt"])), Way::Copy);
    assert!(reply.messages.contains(&Msg::OnDrawing(30, 20, Given::Files(paths(&["/home/john/a.txt"])))), "{:?}", reply.messages);
    over(&mut ui, (d.x + 30.0, d.y + 20.0), offered(false, true, Ways::COPY));
    let reply = let_go(&mut ui, (d.x + 30.0, d.y + 20.0), Given::Text("hi".into()), Way::Copy);
    assert!(reply.messages.contains(&Msg::OnCard(Given::Text("hi".into()))), "{:?}", reply.messages);
    over(&mut ui, (d.x + 30.0, d.y + 20.0), offered(true, false, Ways::COPY));
    let reply = let_go(&mut ui, (d.x + 30.0, d.y + 20.0), Given::Nothing(Why::NotFiles), Way::Copy);
    assert!(reply.messages.contains(&Msg::OnDrawing(30, 20, Given::Nothing(Why::NotFiles))));
    assert_eq!(reply.dropped, Some((9, Way::Copy, false)));

    let under = ww::list(vec![ww::item("Home"), ww::item("Documents")], None, Msg::Row)
        .on_drop_from_outside(|_| Some(Way::Copy), |_, _, _| Msg::Point).key("under");
    let cover = ww::placed(ww::card(ww::label("A question")).key("cover"), |_, _| (0.0, 0.0));
    let mut ui = ui_of(ww::layers(vec![under, cover]));
    let c = ui.spot("cover").unwrap().rect;
    let l = ui.spot("under").unwrap().rect;
    assert!(l.contains(c.x + 4.0, c.y + 4.0), "the card lies over the list");
    assert_eq!(over(&mut ui, (c.x + 4.0, c.y + 4.0), offered(true, true, Ways::ALL)).wanted, Some(None));
    let reply = let_go(&mut ui, (c.x + 4.0, c.y + 4.0), Given::Files(paths(&["/home/john/a.txt"])), Way::Copy);
    assert!(reply.messages.is_empty());
    assert_eq!(reply.dropped, Some((9, Way::Copy, false)));
    let clear = (l.x + l.w - 4.0, l.y + l.h - 4.0);
    if !c.contains(clear.0, clear.1) {
        assert_eq!(over(&mut ui, clear, offered(true, true, Ways::ALL)).wanted, Some(Some((Way::Copy, Kind::Files))));
    }
}

#[test]
fn a_picture_that_passes_the_pointer_does_not_turn_away_what_is_let_go_under_it() {
    let under = || ww::list(vec![ww::item("Home"), ww::item("Documents")], None, Msg::Row)
        .on_drop_from_outside(|_| Some(Way::Copy), |_, _, _| Msg::Point).key("under");
    let band = |passes: bool| {
        let d = ww::drawing((200.0, 60.0), "", |_, _| {}).key("band");
        ww::placed(if passes { d.passes() } else { d }, |_, _| (0.0, 0.0))
    };
    let mut ui = ui_of(ww::layers(vec![under(), band(true)]));
    let at = middle(&ui, "band");
    assert_eq!(over(&mut ui, at, offered(true, false, Ways::COPY)).wanted, Some(Some((Way::Copy, Kind::Files))));
    let mut ui = ui_of(ww::layers(vec![under(), band(false)]));
    let at = middle(&ui, "band");
    assert_eq!(over(&mut ui, at, offered(true, false, Ways::COPY)).wanted, Some(None));
}

#[test]
fn a_way_the_other_program_does_not_allow_is_never_answered() {
    let tree = ww::list(vec![ww::item("Home")], None, Msg::Row)
        .on_drop_from_outside(|_| Some(Way::Move), |_, _, _| Msg::Point).key("places");
    let mut ui = ui_of(tree);
    let at = middle(&ui, "places");
    assert_eq!(over(&mut ui, at, offered(true, false, Ways::COPY)).wanted, Some(None));
    assert_eq!(over(&mut ui, at, offered(true, false, Ways::MOVE)).wanted, Some(Some((Way::Move, Kind::Files))));
}

#[test]
fn data_of_its_own_kind_is_taken_by_name_and_the_screen_reader_hears_drop_target() {
    let card = "application/x-card";
    let tree = ww::column(vec![ww::drawing((200.0, 60.0), "Board", |_, _| {})
        .on_drop_of(&["image/png", card], Held::usual, |_, given, _| Msg::OnCard(given)).key("board")]);
    let mut ui = ui_of(tree);
    let at = middle(&ui, "board");
    let held = |kinds: &[&str]| Input::DragIn { x: at.0, y: at.1, offered: offered(false, true, Ways::COPY), kinds: kinds.iter().map(|k| k.to_string()).collect() };
    assert_eq!(ui.input(&held(&["text/plain"]), Instant::now()).wanted, Some(None));
    assert!(ui.hover.is_none());
    let reply = ui.input(&held(&["text/plain", card]), Instant::now());
    assert_eq!(reply.wanted, Some(Some((Way::Copy, Kind::Typed(card.into())))));
    assert_eq!(ui.hover.as_deref(), Some("board"));
    let told = |ui: &mut Ui<Msg>| -> Vec<Option<String>> {
        ui.access("T").nodes.iter().filter(|(_, p)| p.description() == Some("Drop target")).map(|(_, p)| p.label().map(str::to_string)).collect()
    };
    assert_eq!(told(&mut ui), vec![Some("Board".to_string())]);
    let reply = let_go(&mut ui, at, Given::Typed(card.into(), vec![0, 255, 7]), Way::Copy);
    assert_eq!(reply.messages, vec![Msg::OnCard(Given::Typed(card.into(), vec![0, 255, 7]))]);
    assert_eq!(reply.dropped, Some((9, Way::Copy, true)));
    assert!(ui.hover.is_none() && told(&mut ui).is_empty());
    let mut ui = ui_of(view("", ""));
    let at = middle(&ui, "card");
    let reply = ui.input(&Input::DragIn { x: at.0, y: at.1, offered: offered(false, false, Ways::COPY), kinds: vec![card.into()] }, Instant::now());
    assert_eq!(reply.wanted, Some(None));
}

#[test]
fn what_is_dragged_has_a_small_picture() {
    let ui = ui_of(view("", ""));
    let (one, scale) = ui.drag_picture(Some("folder-symbolic"), true, "Reports").expect("a picture");
    assert_eq!(scale, 1);
    assert!(one.width() > 20 && one.height() > 8 && one.height() < 80);
    assert!(one.pixels().iter().any(|p| p.alpha() == 255));
    assert_eq!(one.pixel(0, 0).map(|p| p.alpha()), Some(0));
    let (long, _) = ui.drag_picture(None, false, &"a very long name ".repeat(40)).unwrap();
    let (short, _) = ui.drag_picture(None, false, &"a very long name ".repeat(2)).unwrap();
    assert!(long.width() <= short.width() + 24, "{} against {}", long.width(), short.width());
    assert!(ui.drag_picture(None, false, "two\nlines").unwrap().0.height() == short.height());
}

struct Pad {
    text: String,
    at: Rc<std::cell::Cell<(f32, f32)>>,
}

impl App for Pad {
    type Msg = Msg;
    fn update(&mut self, msg: Msg) {
        if let Msg::Name(text) = msg {
            self.text = text;
        }
    }
    fn view(&self) -> Node<Msg> {
        ww::column(vec![ww::entry(self.text.clone(), "Words", Msg::Name).key("box").width(ww::Length::Fixed(220.0))]).padding(20.0)
    }
    fn intercept(&self, input: &Input, _: bool) -> Option<Msg> {
        if let Input::PointerIn { x, y } | Input::PointerMove { x, y } = input {
            self.at.set((*x, *y));
        }
        None
    }
}

fn drag_the_word_across(stage: &Stage<(Shown<Pad>, Shown<Pad>)>, plain: &mut Plain, mouse: &mut Mouse, word: (f32, f32), target: (f32, f32)) {
    mouse.place(word.0, word.1);
    plain.settle(150);
    for _ in 0..2 {
        mouse.button(LEFT_BUTTON, true);
        plain.settle(30);
        mouse.button(LEFT_BUTTON, false);
        plain.settle(30);
    }
    stage.until("the word was picked", |(a, _)| a.held.borrow().ui.edits.get("box").is_some_and(|e| e.selected() == "hello"));
    plain.settle(700);
    mouse.button(LEFT_BUTTON, true);
    plain.settle(120);
    mouse.glide((word.0 + 16.0, word.1 + 14.0), plain);
    stage.until("the word was offered", |(a, _)| a.window.dragging_out() && a.held.borrow().ui.lifting());
    mouse.glide(target, plain);
    plain.settle(200);
    mouse.button(LEFT_BUTTON, false);
    stage.until("the word arrived in the other window's box", |(_, b)| b.with(|p| p.text == "hello"));
    stage.until("the first window let go of it", |(a, _)| !a.window.dragging_out() && !a.held.borrow().ui.lifting());
    assert_eq!(stage.ask(|(a, _)| a.with(|p| p.text.clone())), "hello there", "a copy leaves the words where they were");
}

#[test]
#[ignore = "needs a real compositor on the screen, named by the script that runs it: run by hand"]
fn between_programs_selected_words_go_from_one_window_into_another() {
    let run = harness().expect("TRINDLE_DND_TEST must name the harness's runtime folder: see drag_test.rs");
    let socket = run.join(std::env::var("WAYLAND_DISPLAY").unwrap());
    let mut plain = Plain::open(&socket, "A mouse", (120, 60));
    let stage: Stage<(Shown<Pad>, Shown<Pad>)> = Stage::start(|events, display| {
        let pad = |title: &str, text: &str, size| {
            let kind = super::super::window::Kind::Window { title: title.into(), app_id: "org.trindle.Test".into() };
            Shown::open(events, display, kind, size, Pad { text: text.into(), at: Rc::default() }).expect("a window")
        };
        (pad("From", "hello there", (1240, 300)), pad("Into", "", (600, 600)))
    });
    stage.until("both windows showed", |(a, b)| a.window.seen() && b.window.seen());
    plain.settle(600);
    let mut mouse = plain.mouse();
    let screen = mouse.screen;
    const NOWHERE: (f32, f32) = (-1.0, -1.0);
    let mut find = |first: bool, tries: &[(f32, f32)]| -> (f32, f32) {
        for at in tries {
            for nudge in [3.0, 0.0] {
                stage.ask(move |(a, b)| if first { a.with(|p| p.at.set(NOWHERE)) } else { b.with(|p| p.at.set(NOWHERE)) });
                mouse.place(at.0 + nudge, at.1 + nudge);
                plain.settle(80);
            }
            let p = stage.ask(move |(a, b)| if first { a.with(|p| p.at.get()) } else { b.with(|p| p.at.get()) });
            if p != NOWHERE {
                return (at.0 - p.0, at.1 - p.1);
            }
        }
        panic!("the pointer never found the {} window", if first { "first" } else { "second" });
    };
    let (w, h) = (screen.0 as f32, screen.1 as f32);
    let from = find(true, &[(100.0, h / 2.0), (w - 100.0, h / 2.0), (60.0, h / 2.0 - 60.0), (60.0, h / 2.0 + 60.0)]);
    let into = find(false, &[(w / 2.0, 110.0), (w / 2.0, h - 110.0), (w / 2.0, 70.0), (w / 2.0 - 120.0, 150.0)]);
    let rect = |first: bool| stage.ask(move |(a, b)| {
        let held = if first { a.held.borrow() } else { b.held.borrow() };
        let (r, k) = (held.ui.spot("box").unwrap().rect, held.ui.scale);
        (r.x / k, r.y / k, r.w / k, r.h / k)
    });
    let (a, b) = (rect(true), rect(false));
    let word = (from.0 + a.0 + 22.0, from.1 + a.1 + a.3 / 2.0);
    let target = (into.0 + b.0 + b.2 / 2.0, into.1 + b.1 + b.3 / 2.0);
    println!("on a screen {screen:?} the first window is at {from:?}, the second at {into:?}; from {word:?} to {target:?}");
    drag_the_word_across(&stage, &mut plain, &mut mouse, word, target);
}
