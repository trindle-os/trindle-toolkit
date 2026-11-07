//! Context menus on any widget (`Node::context_menu`), worked as a person, a finger and a screen reader would.

use super::*;
use super::super::popup;
use super::super::widget::{self as ww, MenuRow};
use accesskit::{Action, ActionRequest, Role, TreeId};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Open,
    Pin,
    Remove,
    Inner,
    Name(String),
    Wrapped(Box<Msg>),
}

fn rows() -> Vec<MenuRow<Msg>> {
    vec![
        MenuRow::send("", "Pin", Msg::Pin).checked(false),
        MenuRow::send("", "Remove", Msg::Remove).rule(),
    ]
}

fn page(off: bool) -> Node<Msg> {
    ww::column(vec![
        ww::card(ww::column(vec![
            ww::label("Shopping list").key("words"),
            ww::button("", "Open").on_press(Msg::Open).key("open"),
            ww::entry("", "Note", Msg::Name).key("note"),
            ww::card(ww::label("Inner").key("inner")).context_menu(|| vec![MenuRow::send("", "Inner", Msg::Inner)]).key("inner-card"),
        ])).context_menu(rows).off(off).key("card"),
        ww::button("", "Elsewhere").on_press(Msg::Open).key("elsewhere"),
    ]).padding(10.0)
}

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(true, "#fbc711"));
    ui.size = (500, 400);
    ui.settle();
    ui
}

fn key(ui: &mut Ui<Msg>, sym: Keysym, mods: Mods) -> Vec<Msg> {
    ui.input(&Input::Key { sym, text: None, mods, repeat: false }, Instant::now()).messages
}

fn click(ui: &mut Ui<Msg>, (x, y): (f32, f32), button: Button) -> Vec<Msg> {
    let now = Instant::now();
    ui.input(&Input::PointerMove { x, y }, now);
    let mut m = ui.input(&Input::Press { button, x, y }, now).messages;
    m.extend(ui.input(&Input::Release { button, x, y }, now).messages);
    m
}

fn middle(ui: &Ui<Msg>, id: &str) -> (f32, f32) {
    let r = ui.spot(id).unwrap().rect;
    (r.x + r.w / 2.0, r.y + r.h / 2.0)
}

fn words(ui: &Ui<Msg>) -> Vec<String> {
    ui.copy_menu.as_ref().map(|m| m.popup.rows.iter().map(|r| r.words.clone()).collect()).unwrap_or_default()
}

fn row_spot(ui: &Ui<Msg>, i: usize) -> (f32, f32) {
    let m = ui.copy_menu.as_ref().unwrap();
    let rules: Vec<bool> = m.popup.rows.iter().map(|r| r.rule).collect();
    let r = ui.menu_rect().unwrap();
    (r.x + 30.0, r.y + popup::row_top(&rules, i) + 4.0)
}

#[test]
fn a_right_click_anywhere_inside_opens_the_rows_and_a_row_sends_its_message() {
    let mut ui = ui_of(page(false));
    for id in ["open", "words"] {
        let at = middle(&ui, id);
        assert!(click(&mut ui, at, Button::Right).is_empty(), "the right click on {id} presses nothing");
        assert_eq!(words(&ui), ["Pin", "Remove"], "on {id}");
        assert_eq!(ui.copy_menu.as_ref().unwrap().popup.rows[0].check, Some(false));
        ui.close_menu();
    }
    let at = middle(&ui, "open");
    click(&mut ui, at, Button::Right);
    let remove = row_spot(&ui, 1);
    assert_eq!(click(&mut ui, remove, Button::Left), vec![Msg::Remove]);
    assert!(ui.copy_menu.is_none());
    let at = middle(&ui, "elsewhere");
    click(&mut ui, at, Button::Right);
    assert_ne!(words(&ui), ["Pin", "Remove"]);
}

#[test]
fn the_nearest_one_opens_and_a_text_box_keeps_its_own() {
    let mut ui = ui_of(page(false));
    let at = middle(&ui, "inner");
    click(&mut ui, at, Button::Right);
    assert_eq!(words(&ui), ["Inner"]);
    ui.close_menu();
    let at = middle(&ui, "note");
    click(&mut ui, at, Button::Right);
    assert!(words(&ui).iter().any(|w| w == crate::t!("kit-paste")), "{:?}", words(&ui));
}

#[test]
fn a_greyed_out_node_offers_nothing_of_its_own() {
    let mut ui = ui_of(page(true));
    let at = middle(&ui, "words");
    click(&mut ui, at, Button::Right);
    assert_ne!(words(&ui), ["Pin", "Remove"]);
    assert!(!ui.has_context_at(at.0, at.1));
}

#[test]
fn the_menu_key_and_shift_f10_open_it_lit_under_the_widget_with_the_keyboard() {
    let mut ui = ui_of(page(false));
    ui.focus = Some("open".into());
    key(&mut ui, Keysym::Menu, Mods::default());
    let p = &ui.copy_menu.as_ref().expect("the Menu key opens it").popup;
    assert_eq!((p.hover, words(&ui).len()), (Some(0), 2));
    let r = ui.spot("open").unwrap().rect;
    let menu = ui.menu_rect().unwrap();
    assert!(menu.x >= r.x && menu.y >= r.y && menu.y <= r.y + r.h, "{menu:?} by {r:?}");
    assert!(key(&mut ui, Keysym::Escape, Mods::default()).is_empty());
    assert!(ui.copy_menu.is_none());
    key(&mut ui, Keysym::F10, Mods { shift: true, ..Mods::default() });
    key(&mut ui, Keysym::Down, Mods::default());
    assert_eq!(key(&mut ui, Keysym::Return, Mods::default()), vec![Msg::Remove]);
}

#[test]
fn a_finger_held_still_opens_it_and_lifting_presses_nothing() {
    let now = Instant::now();
    let mut ui = ui_of(page(false));
    let (x, y) = middle(&ui, "open");
    let n = ui.input(&Input::TouchDown { id: 1, x, y }, now).hold.expect("a hold is timed");
    ui.held(n);
    assert_eq!(words(&ui), ["Pin", "Remove"]);
    assert!(ui.input(&Input::TouchUp { id: 1 }, now).messages.is_empty());
}

#[test]
fn a_screen_reader_is_offered_it_and_hears_it_as_a_menu() {
    let mut ui = ui_of(page(false));
    let told = ui.access("Test");
    let (id, _) = told.nodes.iter().find(|(_, p)| p.label() == Some("Open")).expect("the button");
    assert!(told.nodes.iter().find(|(n, _)| n == id).unwrap().1.supports_action(Action::ShowContextMenu));
    let elsewhere = told.nodes.iter().find(|(_, p)| p.label() == Some("Elsewhere")).unwrap();
    assert!(!elsewhere.1.supports_action(Action::ShowContextMenu));
    ui.act(&ActionRequest { action: Action::ShowContextMenu, target_tree: TreeId::ROOT, target_node: *id, data: None });
    let told = ui.access("Test");
    let menu = told.nodes.iter().find(|(_, p)| p.role() == Role::Menu).expect("a menu").1.clone();
    assert_eq!(menu.children().len(), 2);
    let focus = told.nodes.iter().find(|(n, _)| *n == told.focus).unwrap().1.clone();
    assert_eq!((focus.role(), focus.label()), (Role::MenuItemCheckBox, Some("Pin")));
}

#[test]
fn a_page_carried_into_another_program_carries_its_menu() {
    let f: Rc<dyn Fn(Msg) -> Msg> = Rc::new(|m| Msg::Wrapped(Box::new(m)));
    let mut ui = ui_of(page(false).map(f));
    let at = middle(&ui, "open");
    click(&mut ui, at, Button::Right);
    let pin = row_spot(&ui, 0);
    assert_eq!(click(&mut ui, pin, Button::Left), vec![Msg::Wrapped(Box::new(Msg::Pin))]);
}

#[test]
fn an_open_context_menu_looks_as_its_stored_picture_does() {
    let tree = ww::column(vec![ww::card(ww::space()).height(ww::Length::Fixed(60.0)).context_menu(|| vec![
        MenuRow::send("", "", Msg::Pin),
        MenuRow::send("", "", Msg::Remove).rule(),
        MenuRow::send("", "", Msg::Open),
    ]).key("card")]).padding(10.0);
    let mut ui = Ui::new(tree, Theme::new(true, "#fbc711"));
    ui.skip_audit();
    ui.lay_at(300, 200);
    ui.focus = None;
    let r = ui.spot("card").unwrap().rect;
    click(&mut ui, (r.x + 20.0, r.y + 20.0), Button::Right);
    let lit = row_spot(&ui, 2);
    ui.input(&Input::PointerMove { x: lit.0, y: lit.1 }, Instant::now());
    let mut pix = tiny_skia::Pixmap::new(300, 200).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/context-menu.png");
    if std::env::var("TRINDLE_GOLDEN").is_ok_and(|v| v == "refresh") {
        pix.save_png(&file).unwrap();
        return;
    }
    let kept = tiny_skia::Pixmap::load_png(&file).expect("a stored picture; run with TRINDLE_GOLDEN=refresh");
    assert_eq!((kept.width(), kept.height()), (300, 200));
    let wrong = pix.data().iter().zip(kept.data()).filter(|(a, b)| a.abs_diff(**b) > 6).count();
    assert!(wrong <= pix.data().len() / 400, "{wrong} values differ by more than a shade; TRINDLE_GOLDEN=refresh keeps the new picture");
}
