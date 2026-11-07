//! Tests of the small widgets: radio group keys, combo box, chips, stars and the rest.

use super::*;
use super::super::theme::Theme;
use super::super::widget::{self as ww, Length};
use super::super::{badges, chips, combo, duration, find, hint, picker, rating, steps, tiles, when};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Pick(usize),
    Text(String),
    Chips(chips::Edit),
    Stars(u8),
    Find(find::Edit),
    Back,
    Forward,
    Toggle,
    Shade(picker::Shade),
}

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(true, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui
}

fn press(ui: &mut Ui<Msg>, sym: Keysym) -> Vec<Msg> {
    ui.input(&Input::Key { sym, text: None, mods: Mods::default(), repeat: false }, Instant::now()).messages
}

fn typed(ui: &mut Ui<Msg>, words: &str) -> Vec<Msg> {
    ui.input(&Input::Key { sym: Keysym::NoSymbol, text: Some(words.into()), mods: Mods::default(), repeat: false }, Instant::now()).messages
}

fn tabs(ui: &mut Ui<Msg>, n: usize) {
    for _ in 0..n {
        press(ui, Keysym::Tab);
    }
}

fn shot(ui: &mut Ui<Msg>) -> tiny_skia::Pixmap {
    let mut pix = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix
}

fn labels(ui: &mut Ui<Msg>) -> Vec<String> {
    ui.access("Test").nodes.iter().filter_map(|(_, n)| n.label().map(str::to_string)).collect()
}

#[test]
fn arrows_move_through_a_radio_group_and_choose() {
    let rows = vec![("A".to_string(), String::new()), ("B".to_string(), "b".to_string()), ("C".to_string(), String::new())];
    let mut ui = ui_of(tiles::radio_group(rows, 0, Msg::Pick));
    tabs(&mut ui, 1);
    assert_eq!(press(&mut ui, Keysym::Down), vec![Msg::Pick(1)]);
    assert_eq!(press(&mut ui, Keysym::Down), vec![Msg::Pick(2)]);
    assert_eq!(press(&mut ui, Keysym::Right), vec![Msg::Pick(0)]);
    assert_eq!(press(&mut ui, Keysym::End), vec![Msg::Pick(2)]);
    assert_eq!(press(&mut ui, Keysym::Up), vec![Msg::Pick(1)]);
    assert_eq!(press(&mut ui, Keysym::Home), vec![Msg::Pick(0)]);
    assert!(ui.keyboard().arrows);
}

#[test]
fn tab_stops_only_on_the_chosen_radio_answer() {
    let rows = || vec![("A".to_string(), String::new()), ("B".to_string(), "b".to_string()), ("C".to_string(), String::new())];
    let view = |chosen: usize| ww::column(vec![ww::button("list-add-symbolic", "Before").on_press(Msg::Back), tiles::radio_group(rows(), chosen, Msg::Pick),
                                              ww::button("list-add-symbolic", "After").on_press(Msg::Forward)]).padding(10.0);
    let on = |ui: &Ui<Msg>| ui.focus.as_deref().and_then(|id| ui.node(id)).and_then(|n| n.says.as_deref()).and_then(|s| s.radio);
    let mut ui = ui_of(view(1));
    tabs(&mut ui, 1);
    assert_eq!(on(&ui), None, "the button before the group");
    tabs(&mut ui, 1);
    assert_eq!(on(&ui), Some(true), "the chosen answer takes the first Tab");
    tabs(&mut ui, 1);
    assert_eq!(on(&ui), None, "the next Tab goes past the group, not on to the other answers");
    press(&mut ui, Keysym::ISO_Left_Tab);
    assert_eq!(on(&ui), Some(true), "Shift+Tab comes back to the chosen answer");
    assert_eq!(press(&mut ui, Keysym::Down), vec![Msg::Pick(2)]);
    ui.set_view(view(2));
    ui.settle();
    let mut fresh = ui_of(view(2));
    tabs(&mut fresh, 2);
    assert_eq!(fresh.focus.as_deref().and_then(|id| fresh.node(id)).and_then(|n| n.says.as_deref()).and_then(|s| s.radio), Some(true));
    assert!(fresh.focus.as_deref().is_some_and(|id| fresh.node(id).is_some_and(|n| n.says.as_deref().and_then(|s| s.name.as_deref()).is_some_and(|w| w.starts_with('C')))));
    let mut none = ui_of(view(9));
    tabs(&mut none, 2);
    assert!(none.focus.as_deref().is_some_and(|id| none.node(id).is_some_and(|n| n.says.as_deref().is_some_and(|s| s.radio.is_some()))));
}

#[test]
fn a_combo_shows_matches_picks_one_and_takes_a_new_value() {
    let options: Vec<String> = ["Paris", "Parma", "Rome"].iter().map(|s| s.to_string()).collect();
    let tree = |text: &str| ww::column(vec![combo::combo(text, &options, "City", Msg::Text)]).padding(10.0);
    let mut ui = ui_of(tree("par"));
    let told = labels(&mut ui);
    assert!(told.contains(&"Paris".to_string()) && told.contains(&"Parma".to_string()) && !told.contains(&"Rome".to_string()));
    tabs(&mut ui, 2);
    assert_eq!(press(&mut ui, Keysym::space), vec![Msg::Text("Paris".into())]);
    let mut empty = ui_of(tree(""));
    assert!(!labels(&mut empty).contains(&"Paris".to_string()));
    tabs(&mut empty, 1);
    assert_eq!(typed(&mut empty, "Oslo"), vec![Msg::Text("Oslo".into())]);
}

#[test]
fn chips_come_from_enter_go_by_backspace_or_a_press_and_are_named() {
    let state = chips::Chips { chips: vec!["red".into(), "blue".into()], draft: String::new() };
    let tree = |c: &chips::Chips| ww::column(vec![c.view("Colours", Msg::Chips)]).padding(10.0);
    let mut ui = ui_of(tree(&state));
    assert!(ui.access("Test").nodes.iter().any(|(_, n)| n.label() == Some("Remove blue")));
    tabs(&mut ui, 3);
    assert_eq!(press(&mut ui, Keysym::BackSpace), vec![Msg::Chips(chips::Edit::Back)]);
    assert_eq!(typed(&mut ui, "g"), vec![Msg::Chips(chips::Edit::Draft("g".into()))]);
    assert_eq!(press(&mut ui, Keysym::Return), vec![Msg::Chips(chips::Edit::Add)]);
    let mut full = ui_of(tree(&chips::Chips { chips: vec![], draft: "g".into() }));
    tabs(&mut full, 1);
    assert_eq!(press(&mut full, Keysym::BackSpace), vec![Msg::Chips(chips::Edit::Draft(String::new()))]);
    let mut first = ui_of(tree(&state));
    tabs(&mut first, 1);
    assert_eq!(press(&mut first, Keysym::space), vec![Msg::Chips(chips::Edit::Remove(0))]);
}

#[test]
fn stars_are_pressed_stepped_painted_and_named() {
    let tree = |v| ww::column(vec![rating::rating(v, 5, "Quality", Msg::Stars).key("stars")]).padding(10.0);
    let mut ui = ui_of(tree(2));
    let r = ui.spot("stars").unwrap().rect;
    let at = |n: f32| (r.x + 22.0 * n - 11.0, r.y + 11.0);
    let click = |ui: &mut Ui<Msg>, (x, y): (f32, f32)| {
        let now = Instant::now();
        ui.input(&Input::PointerMove { x, y }, now);
        ui.input(&Input::Press { button: Button::Left, x, y }, now).messages
    };
    assert_eq!(click(&mut ui, at(4.0)), vec![Msg::Stars(4)]);
    assert_eq!(click(&mut ui, at(2.0)), vec![Msg::Stars(0)]);
    assert_eq!(press(&mut ui, Keysym::Right), vec![Msg::Stars(3)]);
    assert_eq!(press(&mut ui, Keysym::Left), vec![Msg::Stars(1)]);
    assert_eq!((press(&mut ui, Keysym::Home), press(&mut ui, Keysym::End)), (vec![Msg::Stars(0)], vec![Msg::Stars(5)]));
    assert!(labels(&mut ui).contains(&"Quality, 2 of 5 stars".to_string()));
    let (a, b) = (shot(&mut ui_of(tree(2))), shot(&mut ui_of(tree(3))));
    assert_ne!(a.data(), b.data());
}

#[test]
fn a_duration_view_has_three_named_boxes() {
    let mut ui = ui_of(duration::DurationBox::new(3725).view(|_| Msg::Back));
    assert_eq!(ui.tree.said().iter().filter(|s| s.starts_with("entry: ")).count(), 3);
    assert!(ui.tree.said().contains(&"entry: Seconds".to_string()));
    ui.settle();
}

#[test]
fn a_date_range_fills_the_days_between_and_paints_differently() {
    let day = |d| when::Date::new(2026, 10, d).unwrap();
    let mut one = when::RangeBox::new(day(1));
    one.update(when::Pick::Day(day(5)));
    let mut two = one;
    two.update(when::Pick::Day(day(9)));
    let (a, b) = (shot(&mut ui_of(one.view(|_| Msg::Back))), shot(&mut ui_of(two.view(|_| Msg::Back))));
    assert_ne!(a.data(), b.data());
}

#[test]
fn the_find_bar_goes_next_and_closes_and_says_where_it_is() {
    let bar = |count| {
        let mut f = find::Find { query: "a".into(), ..find::Find::default() };
        f.set_count(count);
        f.update(find::Edit::Next);
        f
    };
    let mut ui = ui_of(bar(5).view(Msg::Find));
    assert!(ui.tree.said().contains(&"2 of 5".to_string()));
    tabs(&mut ui, 1);
    assert_eq!(press(&mut ui, Keysym::Return), vec![Msg::Find(find::Edit::Next)]);
    assert_eq!(press(&mut ui, Keysym::Escape), vec![Msg::Find(find::Edit::Close)]);
    assert!(ui_of(bar(0).view(Msg::Find)).tree.said().contains(&"No matches".to_string()));
}

#[test]
fn the_find_count_is_as_wide_as_its_words() {
    let width_of = |count: usize, words: &str| {
        let mut f = find::Find { query: "a".into(), ..find::Find::default() };
        f.set_count(count);
        f.update(find::Edit::Next);
        let mut ui = ui_of(f.view(Msg::Find));
        ui.size = (900, 300);
        ui.settle();
        let told = ui.access("Test");
        let b = told.nodes.iter().find(|(_, n)| n.label() == Some(words) || n.value() == Some(words)).and_then(|(_, n)| n.bounds()).expect(words);
        b.x1 - b.x0
    };
    let (short, long) = (width_of(5, "2 of 5"), width_of(1_000_000_000, "2 of 1000000000"));
    assert!(long > 88.0 && long > short, "{short} {long}: the long count takes the room it needs");
}

#[test]
fn a_wizard_says_the_step_and_ends_with_finish() {
    let titles: Vec<String> = ["One", "Two", "Three"].iter().map(|s| s.to_string()).collect();
    let page = |at| ww::column(vec![steps::wizard(at, &titles, ww::label("Body"), Msg::Back, Msg::Forward)]).padding(10.0);
    let mut first = ui_of(page(0));
    let said = first.tree.said();
    assert!(said.contains(&"Step 1 of 3".to_string()) && said.contains(&"button: Next".to_string()));
    tabs(&mut first, 1);
    assert_eq!(press(&mut first, Keysym::space), vec![Msg::Forward], "Back is off, so the first stop is Next");
    let mut last = ui_of(page(2));
    assert!(last.tree.said().contains(&"button: Finish".to_string()));
    tabs(&mut last, 1);
    assert_eq!(press(&mut last, Keysym::space), vec![Msg::Back]);
    tabs(&mut last, 1);
    assert_eq!(press(&mut last, Keysym::space), vec![Msg::Forward]);
    let (a, b) = (shot(&mut ui_of(page(0))), shot(&mut ui_of(page(1))));
    assert_ne!(a.data(), b.data());
}

#[test]
fn an_empty_state_has_its_picture_and_a_skeleton_is_a_busy_loading() {
    let ui = ui_of(tiles::empty_picture("folder-symbolic", "Nothing here yet.", Some(ww::word_button("Add one").on_press(Msg::Back))));
    let said = ui.tree.said();
    assert!(said.contains(&"Nothing here yet.".to_string()) && said.contains(&"button: Add one".to_string()));
    let mut sk = ui_of(ww::column(vec![tiles::skeleton(3)]).padding(10.0));
    let told = sk.access("Test");
    let bar = told.nodes.iter().find(|(_, n)| n.label() == Some("Loading")).map(|(_, n)| n).unwrap();
    assert!(bar.is_busy());
    let pix = shot(&mut sk);
    let colours: std::collections::HashSet<[u8; 4]> = pix.pixels().iter().map(|p| [p.red(), p.green(), p.blue(), p.alpha()]).collect();
    assert!(colours.len() >= 2);
    assert_ne!(pix.data(), shot(&mut ui_of(ww::column(vec![tiles::skeleton(1)]).padding(10.0))).data());
}

#[test]
fn badges_cap_at_99_and_a_group_ends_in_a_count_of_the_rest() {
    let width = |n| {
        let ui = ui_of(ww::row(vec![badges::badge_count(n, "unread").key("b")]).padding(10.0));
        ui.spot("b").unwrap().rect.w
    };
    assert!(width(5) < width(50) && width(50) < width(100) && width(100) == width(5000));
    let people = (0..5).map(|i| (None, format!("Person {i}"))).collect();
    let mut ui = ui_of(badges::avatar_group(people, 32.0, 3));
    let told = labels(&mut ui);
    assert!(told.contains(&"2 more people".to_string()) && told.contains(&"Person 2".to_string()) && !told.contains(&"Person 3".to_string()));
}

#[test]
fn the_help_covers_the_card_while_on_and_escape_puts_it_away() {
    let card = |open| hint::help_over(ww::card(ww::label("The card")).width(Length::Fill).height(Length::Fill), "How it works.", open, Msg::Toggle);
    let closed = ui_of(card(false));
    assert!(!closed.tree.said().contains(&"How it works.".to_string()));
    assert!(closed.tree.said().contains(&"button: Show help".to_string()));
    let mut open = ui_of(card(true));
    assert!(open.tree.said().contains(&"How it works.".to_string()) && open.tree.said().contains(&"button: Hide help".to_string()));
    tabs(&mut open, 1);
    assert_eq!(press(&mut open, Keysym::space), vec![Msg::Toggle]);
    assert_eq!(press(&mut open, Keysym::Escape), vec![Msg::Toggle]);
    assert_ne!(shot(&mut ui_of(card(false))).data(), shot(&mut ui_of(card(true))).data());
}

#[test]
fn swatches_are_named_by_their_code_pressed_and_the_chosen_one_is_lit() {
    let colours: Vec<_> = ["#ff0000", "#00ff00", "#0000ff"].iter().map(|c| super::super::colour::Colour::parse(c).unwrap()).collect();
    let grid = |chosen| ww::column(vec![picker::swatch_grid(&colours, chosen, 2, Msg::Pick)]).padding(10.0);
    let mut ui = ui_of(grid(None));
    assert!(ui.tree.said().contains(&"button: #00ff00".to_string()));
    tabs(&mut ui, 2);
    assert_eq!(press(&mut ui, Keysym::space), vec![Msg::Pick(1)]);
    assert_ne!(shot(&mut ui_of(grid(None))).data(), shot(&mut ui_of(grid(Some(2)))).data());
}

#[test]
fn the_gradient_bar_paints_its_stops_and_its_buttons_work() {
    let g = picker::GradientBox::new(vec![(0.0, super::super::colour::Colour::parse("#ff0000").unwrap()), (1.0, super::super::colour::Colour::parse("#0000ff").unwrap())]).unwrap();
    let mut ui = ui_of(ww::column(vec![g.view(Msg::Shade)]).padding(10.0));
    let told = ui.access("Test");
    let bar = told.nodes.iter().find(|(_, n)| n.label() == Some("Gradient with 2 stops")).map(|(_, n)| n).unwrap();
    let b = bar.bounds().unwrap();
    let pix = shot(&mut ui);
    let at = |x: f64| { let p = pix.pixel(x as u32, (b.y0 + (b.y1 - b.y0) / 2.0) as u32).unwrap(); (p.red(), p.blue()) };
    let (left, right) = (at(b.x0 + 8.0), at(b.x1 - 8.0));
    assert!(left.0 > left.1 + 100 && right.1 > right.0 + 100, "{left:?} {right:?}");
    assert!(ui.tree.said().contains(&"button: Add a stop".to_string()));
}

#[test]
fn the_line_of_a_slider_stops_short_of_the_keyboards_ring() {
    let mut ui: Ui<Msg> = Ui::new(ww::slider(0.5, 0.0, 1.0, 0.1, "Level", |_| Msg::Back).width(Length::Fixed(300.0)).key("level"), Theme::new(false, "#fbc711"));
    ui.size = (320, 60);
    ui.settle();
    let r = ui.spot("level").unwrap().rect;
    let sheet = crate::theme::sheet();
    let (mid, left) = ((r.y + (r.h / 2.0).round()) as u32, r.x + 0.5 * (r.w - sheet.knob));
    let shot = |ui: &mut Ui<Msg>| {
        let mut pix = tiny_skia::Pixmap::new(320, 60).unwrap();
        ui.paint(&mut pix.as_mut(), 1.0);
        pix
    };
    let just_outside = (left - sheet.focus_offset - sheet.focus_width - 1.0) as u32;
    let is_page = |pix: &tiny_skia::Pixmap, x: u32| pix.pixel(x, mid) == pix.pixel(1, 1);
    assert!(!is_page(&shot(&mut ui), just_outside), "the line runs up to the knob at rest");
    tabs(&mut ui, 1);
    assert!(is_page(&shot(&mut ui), just_outside), "the ring has clear page round it");
}
