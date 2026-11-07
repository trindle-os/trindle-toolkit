//! The select-and-copy behaviour: what a right click offers on every kind of words, text box, list row, tab and button, and how the selection moves.

use super::*;
use super::super::theme::Theme;
use super::super::widget as ww;
use super::super::popup;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Go,
    Name(String),
    Menu,
    Pick(usize),
}

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(false, "#fbc711"));
    ui.size = (500, 400);
    ui.settle();
    ui
}

fn at(ui: &Ui<Msg>, id: &str) -> (f32, f32) {
    let r = ui.spot(id).unwrap().rect;
    (r.x + 4.0, r.y + r.h / 2.0)
}

fn key(sym: Keysym, mods: Mods) -> Input {
    Input::Key { sym, text: None, mods, repeat: false }
}

fn ctrl() -> Mods {
    Mods { ctrl: true, ..Mods::default() }
}

fn shift() -> Mods {
    Mods { shift: true, ..Mods::default() }
}

fn right_click(ui: &mut Ui<Msg>, x: f32, y: f32, now: Instant) -> Reply<Msg> {
    ui.input(&Input::PointerIn { x, y }, now);
    ui.input(&Input::Press { button: Button::Right, x, y }, now)
}

fn words_of(ui: &Ui<Msg>) -> Vec<String> {
    ui.copy_menu.as_ref().unwrap().popup.rows.iter().map(|r| r.words.clone()).collect()
}

fn choose(ui: &mut Ui<Msg>, words: &str, now: Instant) -> Reply<Msg> {
    let m = ui.copy_menu.as_ref().expect("a menu is open");
    let i = m.popup.rows.iter().position(|r| r.words == words).unwrap_or_else(|| panic!("no {words} row in {:?}", m.popup.rows));
    let rules: Vec<bool> = m.popup.rows.iter().map(|r| r.rule).collect();
    let r = ui.menu_rect().unwrap();
    let (x, y) = (r.x + 20.0, r.y + (popup::row_top(&rules, i) + 2.0) * ui.scale);
    ui.input(&Input::PointerMove { x, y }, now);
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    ui.input(&Input::Release { button: Button::Left, x, y }, now)
}

#[test]
fn every_kind_of_words_is_copied_by_a_right_click() {
    let now = Instant::now();
    let kinds: Vec<(&str, Node<Msg>, &str)> = vec![
        ("label", ww::label("a plain label"), "a plain label"),
        ("title", ww::title("A figure"), "A figure"),
        ("page title", ww::page_title("A page"), "A page"),
        ("lead", ww::lead("How things stand"), "How things stand"),
        ("heading", ww::heading("A card"), "A card"),
        ("headline", ww::headline("The headline sentence"), "The headline sentence"),
        ("section", ww::section("A section"), "A section"),
        ("form label", ww::form_label("Name"), "Name"),
        ("column heading", ww::column_heading("Size"), "Size"),
        ("side heading", ww::side_heading("Places"), "Places"),
        ("note", ww::note("A quiet hint"), "A quiet hint"),
    ];
    for (name, node, text) in kinds {
        let mut ui = ui_of(ww::column(vec![node.key("w")]).padding(10.0));
        let (x, y) = at(&ui, "w");
        right_click(&mut ui, x, y, now);
        assert!(ui.menu_open(), "{name}: the menu opens");
        let reply = choose(&mut ui, "Copy", now);
        assert_eq!(reply.copy.as_deref(), Some(text), "{name}: copies all of the words when none are picked");
        assert!(!ui.menu_open(), "{name}: the menu closes once a row is chosen");
    }
}

#[test]
fn words_select_by_dragging_and_the_menu_copies_the_selection() {
    let now = Instant::now();
    let mut ui = ui_of(ww::column(vec![ww::label("hello world again").key("w")]).padding(10.0));
    let r = ui.spot("w").unwrap().rect;
    let y = r.y + r.h / 2.0;
    ui.input(&Input::PointerIn { x: r.x + 2.0, y }, now);
    ui.input(&Input::Press { button: Button::Left, x: r.x + 1.0, y }, now);
    ui.input(&Input::PointerMove { x: r.x + 60.0, y }, now);
    ui.input(&Input::Release { button: Button::Left, x: r.x + 60.0, y }, now);
    let picked = ui.picked.as_ref().map(|(_, e)| e.selected().to_string()).unwrap();
    assert!(!picked.is_empty() && "hello world again".starts_with(&picked));
    assert_eq!(ui.input(&key(Keysym::c, ctrl()), now).copy.as_deref(), Some(picked.as_str()));
    right_click(&mut ui, r.x + 10.0, y, now);
    assert_eq!(choose(&mut ui, "Copy", now).copy.as_deref(), Some(picked.as_str()));
    right_click(&mut ui, r.x + 10.0, y, now);
    choose(&mut ui, "Select all", now);
    assert_eq!(ui.input(&key(Keysym::c, ctrl()), now).copy.as_deref(), Some("hello world again"));
    ui.input(&key(Keysym::Home, Mods::default()), now);
    ui.input(&key(Keysym::Right, shift()), now);
    ui.input(&key(Keysym::Right, shift()), now);
    assert_eq!(ui.input(&key(Keysym::c, ctrl()), now).copy.as_deref(), Some("he"));
}

#[test]
fn words_in_something_pressed_are_pressed_not_selected_but_still_copy() {
    let now = Instant::now();
    let mut ui = ui_of(ww::column(vec![ww::card(ww::label("press these words").key("w")).on_press(Msg::Go)]).padding(10.0));
    let (x, y) = at(&ui, "w");
    ui.input(&Input::PointerIn { x, y }, now);
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    ui.input(&Input::PointerMove { x: x + 40.0, y }, now);
    let reply = ui.input(&Input::Release { button: Button::Left, x: x + 40.0, y }, now);
    assert!(ui.picked.is_none(), "a drag does not select inside a pressed card");
    assert_eq!(reply.messages, vec![Msg::Go]);
    right_click(&mut ui, x, y, now);
    assert_eq!(choose(&mut ui, "Copy", now).copy.as_deref(), Some("press these words"));
}

#[test]
fn a_text_box_menu_cuts_copies_pastes_and_never_copies_a_password() {
    let now = Instant::now();
    let tree = |name: &str, pass: &str| ww::column(vec![
        ww::entry(name.to_string(), "Name", Msg::Name).key("name"),
        ww::entry(pass.to_string(), "Password", Msg::Name).secret().key("pass"),
    ]).padding(10.0);
    let mut ui = ui_of(tree("john smith", "hunter2"));
    let (x, y) = at(&ui, "name");
    ui.input(&Input::PointerIn { x, y }, now);
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    ui.input(&Input::Release { button: Button::Left, x, y }, now);
    ui.input(&key(Keysym::a, ctrl()), now);
    right_click(&mut ui, x, y, now);
    assert_eq!(words_of(&ui), ["Cut", "Copy", "Paste", "Select all"]);
    assert_eq!(choose(&mut ui, "Copy", now).copy.as_deref(), Some("john smith"));
    right_click(&mut ui, x, y, now);
    let cut = choose(&mut ui, "Cut", now);
    assert_eq!(cut.copy.as_deref(), Some("john smith"));
    assert_eq!(cut.messages, vec![Msg::Name(String::new())]);
    ui.set_view(tree("", "hunter2"));
    ui.settle();
    right_click(&mut ui, x, y, now);
    assert!(choose(&mut ui, "Paste", now).paste, "Paste asks for the clipboard");
    let (px, py) = at(&ui, "pass");
    ui.input(&Input::Press { button: Button::Left, x: px, y: py }, now);
    ui.input(&Input::Release { button: Button::Left, x: px, y: py }, now);
    ui.input(&key(Keysym::a, ctrl()), now);
    right_click(&mut ui, px, py, now);
    assert_eq!(words_of(&ui), ["Paste", "Select all"]);
    ui.input(&key(Keysym::Escape, Mods::default()), now);
    assert!(!ui.menu_open());
    assert_eq!(ui.input(&key(Keysym::c, ctrl()), now).copy, None, "Ctrl+C in a password box copies nothing");
}

#[test]
fn a_list_row_a_tab_and_a_button_offer_copy_and_a_programs_own_menu_wins() {
    let now = Instant::now();
    let rows = vec![ww::item("first.txt").detail("12 KB"), ww::item("second.txt").under("a text file")];
    let mut ui = ui_of(ww::column(vec![
        ww::list(rows.clone(), None, Msg::Pick).key("rows"),
        ww::choice(vec!["Icons".into(), "List".into()], 0, "View", Msg::Pick).key("view"),
        ww::main_action("document-open-symbolic", "Save the lot").on_press(Msg::Go).key("save"),
    ]).padding(10.0));
    let r = ui.spot("rows").unwrap().rect;
    right_click(&mut ui, r.x + 10.0, r.y + 5.0, now);
    assert_eq!(choose(&mut ui, "Copy", now).copy.as_deref(), Some("first.txt\t12 KB"));
    let row = ui.px(match &ui.node("rows").unwrap().what { What::List { row, .. } => *row, _ => 0.0 });
    right_click(&mut ui, r.x + 10.0, r.y + row + 5.0, now);
    assert_eq!(choose(&mut ui, "Copy", now).copy.as_deref(), Some("second.txt\na text file"));
    let (x, y) = at(&ui, "view");
    right_click(&mut ui, x + 8.0, y, now);
    assert_eq!(choose(&mut ui, "Copy", now).copy.as_deref(), Some("Icons"));
    let (x, y) = at(&ui, "save");
    right_click(&mut ui, x + 30.0, y, now);
    assert_eq!(choose(&mut ui, "Copy", now).copy.as_deref(), Some("Save the lot"));
    let mut ui = ui_of(ww::column(vec![ww::list(rows, None, Msg::Pick).on_menu(|_, _, _| Msg::Menu).key("rows")]).padding(10.0));
    let r = ui.spot("rows").unwrap().rect;
    let reply = right_click(&mut ui, r.x + 10.0, r.y + 5.0, now);
    assert!(!ui.menu_open());
    assert!(reply.messages.contains(&Msg::Menu));
}

#[test]
fn a_drawing_offers_the_rows_it_names_and_is_still_told_of_the_click() {
    let now = Instant::now();
    let seen = std::rc::Rc::new(std::cell::Cell::new(0));
    let s = seen.clone();
    let tree = ww::column(vec![ww::drawing((200.0, 50.0), "Words", |_, _| {})
        .on_pointer(move |p| {
            if matches!(p, ww::Pointer::Menu(..)) {
                s.set(s.get() + 1);
            }
            Msg::Go
        })
        .menu(|_, _| vec![ww::MenuRow::copy("drawn words"), ww::MenuRow::send("edit-cut-symbolic", "Do it", Msg::Menu)])
        .key("art")]).padding(10.0);
    let mut ui = ui_of(tree);
    let (x, y) = at(&ui, "art");
    right_click(&mut ui, x + 5.0, y, now);
    assert_eq!(seen.get(), 1);
    let reply = choose(&mut ui, "Do it", now);
    assert_eq!(reply.messages, vec![Msg::Menu]);
    right_click(&mut ui, x + 5.0, y, now);
    assert_eq!(choose(&mut ui, "Copy", now).copy.as_deref(), Some("drawn words"));
}

#[test]
fn the_menu_is_closed_by_a_click_elsewhere_or_escape_and_is_painted_over_the_window() {
    let now = Instant::now();
    let mut ui = ui_of(ww::column(vec![ww::label("some words").key("w")]).padding(10.0));
    let (x, y) = at(&ui, "w");
    let mut plain = tiny_skia::Pixmap::new(500, 400).unwrap();
    ui.paint(&mut plain.as_mut(), 1.0);
    right_click(&mut ui, x, y, now);
    let mut shown = tiny_skia::Pixmap::new(500, 400).unwrap();
    ui.paint(&mut shown.as_mut(), 1.0);
    assert_ne!(plain.data(), shown.data(), "the menu is drawn");
    ui.input(&Input::Press { button: Button::Left, x: 450.0, y: 380.0 }, now);
    assert!(!ui.menu_open());
    right_click(&mut ui, x, y, now);
    assert!(ui.menu_open());
    ui.input(&key(Keysym::Escape, Mods::default()), now);
    assert!(!ui.menu_open());
}

#[test]
fn the_menu_key_opens_the_menu_for_the_box_with_the_keyboard_and_its_first_row_is_lit() {
    let now = Instant::now();
    let mut ui = ui_of(ww::column(vec![ww::entry("john smith", "Name", Msg::Name).key("name")]).padding(10.0));
    ui.focus = Some("name".into());
    ui.input(&key(Keysym::Menu, Mods::default()), now);
    assert!(ui.menu_open());
    assert_eq!(words_of(&ui), ["Paste", "Select all"]);
    assert_eq!(ui.copy_menu.as_ref().unwrap().popup.hover, Some(0));
    ui.input(&key(Keysym::Down, Mods::default()), now);
    assert_eq!(ui.copy_menu.as_ref().unwrap().popup.hover, Some(1));
    ui.input(&key(Keysym::Up, Mods::default()), now);
    assert!(ui.input(&key(Keysym::Return, Mods::default()), now).paste, "Enter chooses the lit row");
    assert!(!ui.menu_open());
    ui.input(&key(Keysym::F10, shift()), now);
    assert!(ui.menu_open());
}

#[test]
fn a_touch_on_a_menu_row_chooses_it_and_a_touch_elsewhere_closes_the_menu() {
    let now = Instant::now();
    let mut ui = ui_of(ww::column(vec![ww::label("some words").key("w")]).padding(10.0));
    let (x, y) = at(&ui, "w");
    right_click(&mut ui, x, y, now);
    let m = ui.copy_menu.as_ref().unwrap();
    let rules: Vec<bool> = m.popup.rows.iter().map(|r| r.rule).collect();
    let r = ui.menu_rect().unwrap();
    let (tx, ty) = (r.x + 20.0, r.y + (popup::row_top(&rules, 0) + 2.0) * ui.scale);
    let reply = ui.input(&Input::TouchDown { id: 1, x: tx, y: ty }, now);
    assert_eq!(reply.copy.as_deref(), Some("some words"));
    assert!(!ui.menu_open());
    right_click(&mut ui, x, y, now);
    ui.input(&Input::TouchDown { id: 1, x: 450.0, y: 380.0 }, now);
    assert!(!ui.menu_open());
}
