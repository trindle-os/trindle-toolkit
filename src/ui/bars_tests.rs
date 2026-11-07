//! Menus, number boxes, dialog pages and bars, worked as a person would.

use super::*;
use super::super::bars::{path_bar, progress_bar, status_bar, tool, ToolMsg, Toolbar};
use super::super::dialog::dialog;
use super::super::menubar::{BarDone, BarMsg, MenuBar};
use super::super::popup;
use super::super::range::{Range, RangeMsg};
use super::super::spin::{Spin, SpinMsg};
use super::super::theme::Theme;
use super::super::widget::{self as ww, MenuRow};
use accesskit::{Role, Toggled};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Wrap(bool),
    Sort(usize),
    Bar(BarMsg),
    Spin(SpinMsg),
    Cancel,
    Go,
    Range(RangeMsg),
    Tool(ToolMsg),
    Path(usize),
}

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(true, "#fbc711"));
    ui.size = (500, 400);
    ui.settle();
    ui
}

fn shot(ui: &mut Ui<Msg>) -> tiny_skia::Pixmap {
    let mut pix = tiny_skia::Pixmap::new(500, 400).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix
}

fn key(ui: &mut Ui<Msg>, sym: Keysym) -> Vec<Msg> {
    ui.input(&Input::Key { sym, text: None, mods: Mods::default(), repeat: false }, Instant::now()).messages
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

fn sheet_with_menu(wrap: bool) -> Node<Msg> {
    let d = ww::drawing((200.0, 100.0), "Sheet", |_, _| {})
        .on_keys(|_| None)
        .menu(move |_, _| vec![
            MenuRow::send("", "Wrap lines", Msg::Wrap(!wrap)).checked(wrap),
            MenuRow::send("", "Sort by", Msg::Sort(9)).submenu(vec![MenuRow::send("", "Name", Msg::Sort(0)), MenuRow::send("", "Size", Msg::Sort(1))]).rule(),
        ])
        .key("sheet");
    ww::column(vec![d]).padding(10.0)
}

fn row_spot(ui: &Ui<Msg>, i: usize, sub: bool) -> (f32, f32) {
    let m = ui.copy_menu.as_ref().unwrap();
    let rows = if sub { m.popup.sub_rows() } else { &m.popup.rows };
    let rules: Vec<bool> = rows.iter().map(|r| r.rule).collect();
    let (menu, under) = ui.menu_rects().unwrap();
    let r = if sub { under.unwrap() } else { menu };
    (r.x + 30.0, r.y + popup::row_top(&rules, i) + 4.0)
}

#[test]
fn a_context_menu_shows_a_tick_and_a_sub_menu_beside_it() {
    let mut ui = ui_of(sheet_with_menu(true));
    let at = middle(&ui, "sheet");
    click(&mut ui, at, Button::Right);
    let m = ui.copy_menu.as_ref().expect("a right click opens the menu");
    assert_eq!((m.popup.rows[0].check, m.popup.rows[1].sub.len()), (Some(true), 2));
    let (x, y) = row_spot(&ui, 1, false);
    ui.input(&Input::PointerMove { x, y }, Instant::now());
    assert_eq!(ui.copy_menu.as_ref().unwrap().popup.beside.open, Some(1));
    let size = row_spot(&ui, 1, true);
    assert_eq!(click(&mut ui, size, Button::Left), vec![Msg::Sort(1)]);
    assert!(ui.copy_menu.is_none());
    click(&mut ui, at, Button::Right);
    let wrap = row_spot(&ui, 0, false);
    assert_eq!(click(&mut ui, wrap, Button::Left), vec![Msg::Wrap(false)]);
}

#[test]
fn the_menu_key_opens_a_drawings_menu_and_the_arrows_work_its_sub_menu() {
    let mut ui = ui_of(sheet_with_menu(false));
    key(&mut ui, Keysym::Tab);
    assert_eq!(ui.focus.as_deref(), Some("sheet"));
    key(&mut ui, Keysym::Menu);
    assert_eq!(ui.copy_menu.as_ref().expect("the Menu key opens it").popup.hover, Some(0));
    key(&mut ui, Keysym::Down);
    key(&mut ui, Keysym::Right);
    let p = &ui.copy_menu.as_ref().unwrap().popup;
    assert_eq!((p.beside.open, p.beside.hover), (Some(1), Some(0)));
    let told = ui.access("Test");
    let focus = told.nodes.iter().find(|(id, _)| *id == told.focus).unwrap().1.clone();
    assert_eq!((focus.role(), focus.label()), (Role::MenuItem, Some("Name")));
    key(&mut ui, Keysym::Left);
    assert_eq!(ui.copy_menu.as_ref().unwrap().popup.beside.open, None);
    key(&mut ui, Keysym::Right);
    key(&mut ui, Keysym::Down);
    assert_eq!(key(&mut ui, Keysym::Return), vec![Msg::Sort(1)]);
}

#[test]
fn a_ticked_row_is_a_check_item_to_the_reader_and_is_drawn_with_its_tick() {
    let mut on = ui_of(sheet_with_menu(true));
    let at = middle(&on, "sheet");
    click(&mut on, at, Button::Right);
    let told = on.access("Test");
    let item = told.nodes.iter().find(|(_, p)| p.role() == Role::MenuItemCheckBox).expect("a check item").1.clone();
    assert_eq!((item.label(), item.toggled()), (Some("Wrap lines"), Some(Toggled::True)));
    let mut off = ui_of(sheet_with_menu(false));
    click(&mut off, at, Button::Right);
    assert_ne!(shot(&mut on).data(), shot(&mut off).data());
}

#[test]
fn a_radio_row_is_a_radio_item_to_the_reader_and_a_letter_lights_a_row_of_the_open_menu() {
    let rows = || vec![
        MenuRow::send("", "Small", Msg::Sort(0)).radio(false),
        MenuRow::send("", "Medium", Msg::Sort(1)).radio(true),
        MenuRow::send("", "Large", Msg::Sort(2)).radio(false),
    ];
    let d = ww::drawing((200.0, 100.0), "Sheet", |_, _| {}).on_keys(|_| None).menu(move |_, _| rows()).key("sheet");
    let mut ui = ui_of(ww::column(vec![d]).padding(10.0));
    let at = middle(&ui, "sheet");
    click(&mut ui, at, Button::Right);
    let told = ui.access("Test");
    let items: Vec<_> = told.nodes.iter().filter(|(_, p)| p.role() == Role::MenuItemRadio).map(|(_, p)| (p.label().map(str::to_string), p.toggled())).collect();
    assert_eq!(items.len(), 3);
    assert!(items.contains(&(Some("Medium".into()), Some(Toggled::True))));
    assert!(items.contains(&(Some("Large".into()), Some(Toggled::False))));
    let typed = |ui: &mut Ui<Msg>, c: &str| ui.input(&Input::Key { sym: Keysym::l, text: Some(c.into()), mods: Mods::default(), repeat: false }, Instant::now()).messages;
    typed(&mut ui, "l");
    assert_eq!(ui.copy_menu.as_ref().unwrap().popup.hover, Some(2));
    assert_eq!(key(&mut ui, Keysym::Return), vec![Msg::Sort(2)]);
}

#[test]
fn typing_in_an_open_menu_of_the_bar_lights_a_row_and_alt_still_opens_menus() {
    let mut b = MenuBar::new(vec![
        ("File".into(), vec![popup::row("", "New"), popup::row("", "Open"), popup::row("", "Quit")]),
        ("Edit".into(), vec![popup::row("", "Copy")]),
    ]);
    b.f10();
    let mut ui = bar_ui(&b);
    let typed = |ui: &mut Ui<Msg>, text: &str, mods: Mods| ui.input(&Input::Key { sym: Keysym::q, text: Some(text.into()), mods, repeat: false }, Instant::now()).messages;
    let msgs = typed(&mut ui, "q", Mods::default());
    assert_eq!(msgs, vec![Msg::Bar(BarMsg::Letter('q'))]);
    feed(&mut b, &mut ui, msgs);
    assert_eq!(b.open.as_ref().unwrap().hover, Some(2));
    assert!(typed(&mut ui, "q", Mods { ctrl: true, ..Mods::default() }).is_empty());
    let mut shut = bar();
    let mut quiet = bar_ui(&shut);
    shut.lit = Some(0);
    assert!(typed(&mut quiet, "n", Mods::default()).is_empty(), "with no menu open, typing is not taken");
}

fn bar() -> MenuBar {
    MenuBar::new(vec![
        ("File".into(), vec![popup::row("", "New"), popup::row("", "Open")]),
        ("Edit".into(), vec![popup::row("", "Copy"), popup::row("", "Paste")]),
    ])
}

fn bar_ui(bar: &MenuBar) -> Ui<Msg> {
    let mut ui = ui_of(bar.window(ww::label("Page"), Msg::Bar));
    shot(&mut ui);
    ui.set_view(bar.window(ww::label("Page"), Msg::Bar));
    ui.settle();
    ui
}

fn feed(bar: &mut MenuBar, ui: &mut Ui<Msg>, msgs: Vec<Msg>) -> BarDone {
    let mut done = BarDone::Open;
    for m in msgs {
        if let Msg::Bar(b) = m {
            done = bar.update(b);
        }
    }
    ui.set_view(bar.window(ww::label("Page"), Msg::Bar));
    ui.settle();
    shot(ui);
    done
}

#[test]
fn a_menu_bar_opens_a_menu_under_a_title_and_the_keys_go_along_and_choose() {
    let mut b = bar();
    let mut ui = bar_ui(&b);
    let r = ui.spots.iter().find(|s| ui.tree.at(&s.path).and_then(|n| n.says.as_deref()).is_some_and(|k| matches!(k.role, Some(crate::role::Role::MenuBar { .. })))).unwrap().rect;
    let msgs = click(&mut ui, (r.x + r.w - 1.0, r.y + r.h / 2.0), Button::Left);
    let msgs = msgs.into_iter().filter(|m| matches!(m, Msg::Bar(BarMsg::Press(_)))).collect();
    feed(&mut b, &mut ui, msgs);
    assert_eq!(b.lit, None, "a press off the titles opens nothing");
    let told = ui.access("Test");
    let edit = told.nodes.iter().find(|(_, p)| p.role() == Role::MenuItem && p.label() == Some("Edit")).and_then(|(_, p)| p.bounds()).unwrap();
    let x = ((edit.x0 + edit.x1) / 2.0) as f32;
    let msgs = click(&mut ui, (x, r.y + r.h / 2.0), Button::Left).into_iter().filter(|m| matches!(m, Msg::Bar(BarMsg::Press(_)))).collect();
    feed(&mut b, &mut ui, msgs);
    assert_eq!(b.lit, Some(1));
    let menu = b.open.as_ref().unwrap().spots((500.0, 400.0)).0;
    assert!(menu.y >= r.y + r.h - 1.0 && menu.x > r.x + 8.0, "under the Edit title: {menu:?}");
    assert_eq!(b.update(BarMsg::Key(Keysym::Left)), BarDone::Open);
    assert_eq!(b.lit, Some(0));
    b.update(BarMsg::Key(Keysym::Down));
    assert_eq!(b.update(BarMsg::Key(Keysym::Return)), BarDone::Picked(0, 1));
    assert!(b.open.is_none());
    b.f10();
    let mut ui = bar_ui(&b);
    let msgs = vec![Msg::Bar(BarMsg::Key(Keysym::Escape))];
    assert!(ui.focus.as_deref().is_some_and(|f| ui.node(f).is_some_and(|n| n.says.as_deref().is_some_and(|k| matches!(k.role, Some(crate::role::Role::MenuBar { .. }))))));
    assert_eq!(key(&mut ui, Keysym::Escape), msgs);
    assert_eq!(feed(&mut b, &mut ui, msgs), BarDone::Closed);
}

#[test]
fn a_menu_bar_is_told_as_a_menu_bar_of_titles() {
    let mut b = bar();
    b.update(BarMsg::Key(Keysym::Right));
    b.update(BarMsg::Key(Keysym::Right));
    let mut ui = bar_ui(&b);
    let told = ui.access("Test");
    let bar = told.nodes.iter().find(|(_, p)| p.role() == Role::MenuBar).expect("a menu bar").1.clone();
    assert_eq!(bar.size_of_set(), Some(2));
    let titles: Vec<(Option<&str>, Option<bool>)> = told.nodes.iter().filter(|(id, _)| bar.children().contains(id)).map(|(_, p)| (p.label(), p.is_selected())).collect();
    assert!(titles.contains(&(Some("File"), Some(false))) && titles.contains(&(Some("Edit"), Some(true))), "{titles:?}");
}

fn type_in(ui: &mut Ui<Msg>, text: &str) -> Vec<Msg> {
    ui.input(&Input::Key { sym: Keysym::NoSymbol, text: Some(text.into()), mods: Mods::default(), repeat: false }, Instant::now()).messages
}

fn feed_spin(spin: &mut Spin, msgs: Vec<Msg>) {
    for m in msgs {
        if let Msg::Spin(m) = m {
            spin.update(m);
        }
    }
}

#[test]
fn a_number_box_steps_with_the_arrows_wheel_and_buttons_and_keeps_in_its_limits() {
    let mut spin = Spin::new(4.0, 0.0, 10.0, 0.5).unit("px");
    assert_eq!(spin.text(), "4.0");
    let view = |s: &Spin| ww::column(vec![s.node("Width", Msg::Spin)]).padding(10.0);
    let mut ui = ui_of(view(&spin));
    key(&mut ui, Keysym::Tab);
    for (sym, want) in [(Keysym::Up, 4.5), (Keysym::Page_Up, 9.5), (Keysym::Up, 10.0), (Keysym::Up, 10.0), (Keysym::Down, 9.5), (Keysym::Page_Down, 4.5)] {
        feed_spin(&mut spin, key(&mut ui, sym));
        assert_eq!(spin.value, want, "after {sym:?}");
    }
    let at = middle(&ui, ui.focus.clone().unwrap().as_str());
    ui.input(&Input::PointerMove { x: at.0, y: at.1 }, Instant::now());
    let m = ui.input(&Input::Scroll { x: 0.0, y: -1.0, steps: true }, Instant::now()).messages;
    assert_eq!(m, vec![Msg::Spin(SpinMsg::Step(1))]);
    assert!(view(&spin).said().contains(&"px".to_string()));
    let mut ui = ui_of(view(&spin));
    let told = ui.access("Test");
    let part = told.nodes.iter().find(|(_, p)| p.role() == Role::SpinButton).expect("a spin button").1.clone();
    assert_eq!((part.numeric_value(), part.min_numeric_value(), part.max_numeric_value()), (Some(4.5), Some(0.0), Some(10.0)));
    let plus = ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Button { icon, .. }) if icon == "list-add-symbolic")).unwrap().rect;
    assert_eq!(click(&mut ui, (plus.x + plus.w / 2.0, plus.y + plus.h / 2.0), Button::Left), vec![Msg::Spin(SpinMsg::Step(1))]);
}

#[test]
fn a_number_box_takes_what_is_typed_and_says_when_it_is_not_a_number() {
    let mut spin = Spin::new(2.0, 1.0, 5.0, 1.0);
    let view = |s: &Spin| ww::column(vec![s.node("Copies", Msg::Spin)]).padding(10.0);
    let mut ui = ui_of(view(&spin));
    key(&mut ui, Keysym::Tab);
    assert_eq!(type_in(&mut ui, "9"), vec![Msg::Spin(SpinMsg::Typed("29".into()))]);
    spin.update(SpinMsg::Typed("29".into()));
    assert!(spin.update(SpinMsg::Done));
    assert_eq!((spin.value, spin.typed.clone()), (5.0, None));
    spin.update(SpinMsg::Typed("3,4".into()));
    spin.update(SpinMsg::Done);
    assert_eq!(spin.value, 3.0);
    spin.update(SpinMsg::Typed("9".into()));
    let mut ui = ui_of(view(&spin));
    assert!(ui.access("Test").nodes.iter().any(|(_, p)| p.role() == Role::Alert));
    spin.update(SpinMsg::Typed("4".into()));
    let mut ui = ui_of(view(&spin));
    assert!(!ui.access("Test").nodes.iter().any(|(_, p)| p.role() == Role::Alert));
    spin.update(SpinMsg::Typed("3".into()));
    spin.update(SpinMsg::Done);
    spin.update(SpinMsg::Typed("lots".into()));
    assert!(!spin.update(SpinMsg::Done));
    let mut ui = ui_of(view(&spin));
    let told = ui.access("Test");
    let alert = told.nodes.iter().find(|(_, p)| p.role() == Role::Alert).expect("red words").1.clone();
    assert!(alert.label().is_some_and(|l| l.contains('1') && l.contains('5')), "{:?}", alert.label());
}

#[test]
fn a_dialog_page_has_its_title_lead_and_answers_and_escape_cancels() {
    let page = |turn| dialog("Delete 3 files?", "They go to the bin, where they can be had back.", Msg::Cancel)
        .holds(ww::label("report.pdf"))
        .answer(ww::main_word("Delete").on_press(Msg::Go).key("go"))
        .turn(turn)
        .node();
    let texts = page(1).said();
    assert!(["Delete 3 files?", "They go to the bin, where they can be had back.", "report.pdf"].iter().all(|w| texts.contains(&w.to_string())), "{texts:?}");
    let mut ui = ui_of(page(1));
    let cancel = ui.focus.clone().expect("the keyboard is on Cancel");
    assert_eq!(key(&mut ui, Keysym::Return), vec![Msg::Cancel]);
    key(&mut ui, Keysym::Tab);
    assert_eq!(ui.focus.as_deref(), Some("go"));
    key(&mut ui, Keysym::Tab);
    assert_eq!(ui.focus.as_ref(), Some(&cancel));
    assert_eq!(key(&mut ui, Keysym::Escape), vec![Msg::Cancel]);
    let (c, g) = (ui.spot(&cancel).unwrap().rect, ui.spot("go").unwrap().rect);
    assert!(c.x < g.x && c.y > 200.0, "{c:?} {g:?}");
    let told = ui.access("Test");
    let d = told.nodes.iter().find(|(_, p)| p.role() == Role::Dialog).expect("a dialog").1.clone();
    assert_eq!(d.label(), Some("Delete 3 files?"));
}

#[test]
fn confirm_choose_and_ask_pages_keep_the_keyboard_on_the_safe_part_and_escape_cancels() {
    use super::super::dialog::{ask, choose, confirm};
    let mut ui = ui_of(confirm("Delete 3 files?", "They go to the bin.", "Delete", Msg::Go, Msg::Cancel, 1));
    let cancel = ui.focus.clone().expect("the keyboard is on Cancel");
    assert_ne!(cancel, "dialog-yes");
    assert_eq!(key(&mut ui, Keysym::Return), vec![Msg::Cancel]);
    key(&mut ui, Keysym::Tab);
    assert_eq!(key(&mut ui, Keysym::Return), vec![Msg::Go]);
    assert_eq!(key(&mut ui, Keysym::Escape), vec![Msg::Cancel]);
    assert!(ui.access("Test").nodes.iter().any(|(_, p)| p.role() == Role::Dialog && p.label() == Some("Delete 3 files?")));

    let options = vec![("Keep both".to_string(), Msg::Go), ("Replace".to_string(), Msg::Sort(1))];
    let mut ui = ui_of(choose("Name taken", "A file with that name is here.", options, Msg::Cancel, 1));
    assert!(ui.focus.as_deref().is_some_and(|f| !f.starts_with("dialog-option")));
    let (a, b) = (ui.spot("dialog-option-0").unwrap().rect, ui.spot("dialog-option-1").unwrap().rect);
    assert!(a.y + a.h <= b.y, "{a:?} {b:?}");
    key(&mut ui, Keysym::Tab);
    key(&mut ui, Keysym::Tab);
    assert_eq!(ui.focus.as_deref(), Some("dialog-option-1"));
    assert_eq!(key(&mut ui, Keysym::Return), vec![Msg::Sort(1)]);
    assert_eq!(key(&mut ui, Keysym::Escape), vec![Msg::Cancel]);

    let mut ui = ui_of(ask("Rename", "Give it a new name.", "", "New name", |s| Msg::Spin(SpinMsg::Typed(s)), "Rename", Msg::Go, Msg::Cancel, 1));
    assert_eq!(ui.focus.as_deref(), Some("dialog-text"));
    assert_eq!(type_in(&mut ui, "a"), vec![Msg::Spin(SpinMsg::Typed("a".into()))]);
    assert_eq!(key(&mut ui, Keysym::Return), vec![Msg::Go]);
    assert_eq!(key(&mut ui, Keysym::Escape), vec![Msg::Cancel]);
}

#[test]
fn a_progress_bar_fills_with_the_soft_highlight_and_is_told_its_share() {
    let bar = |share| ww::column(vec![progress_bar(share, "Copying").key("bar")]).padding(10.0);
    let mut ui = ui_of(bar(Some(0.5)));
    let r = ui.spot("bar").unwrap().rect;
    let pix = shot(&mut ui);
    let byte = |c: f32| (c * 255.0).round() as u8;
    let soft = [byte(ui.theme.soft.r), byte(ui.theme.soft.g), byte(ui.theme.soft.b)];
    let at = |x: f32| {
        let p = pix.pixel(x as u32, (r.y + r.h / 2.0) as u32).unwrap();
        [p.red(), p.green(), p.blue()]
    };
    assert_eq!(at(r.x + r.w * 0.25), soft);
    assert_ne!(at(r.x + r.w * 0.75), soft);
    let told = ui.access("Test");
    let p = told.nodes.iter().find(|(_, p)| p.role() == Role::ProgressIndicator).unwrap().1.clone();
    assert_eq!((p.label(), p.numeric_value()), (Some("Copying"), Some(0.5)));
    assert_eq!(p.value(), Some("50%"));
    let mut ui = ui_of(bar(None));
    assert!(ui.access("Test").nodes.iter().any(|(_, p)| p.role() == Role::ProgressIndicator && p.is_busy()));
    assert!(matches!(ui.node("bar").map(|n| &n.what), Some(What::Drawing { alive: true, .. })));
    crate::motion::reduce(true);
    let still = progress_bar::<Msg>(None, "Copying");
    crate::motion::reduce(false);
    assert!(matches!(still.what, What::Drawing { alive: false, .. }));
}

#[test]
fn a_range_slider_moves_the_nearer_handle_and_never_lets_them_cross() {
    let mut r = Range::new(20.0, 80.0, 0.0, 100.0, 5.0);
    let mut ui = ui_of(ww::column(vec![r.node("Price", Msg::Range).key("range")]).padding(10.0));
    shot(&mut ui);
    let s = ui.spot("range").unwrap().rect;
    for m in click(&mut ui, (s.x + s.w * 0.9, s.y + s.h / 2.0), Button::Left) {
        if let Msg::Range(m) = m {
            r.update(m);
        }
    }
    assert_eq!(r.lit, 1);
    assert!(r.high > 85.0 && r.low == 20.0, "{r:?}");
    r.update(RangeMsg::Drag(0.0));
    assert_eq!((r.low, r.high), (20.0, 20.0));
    r.update(RangeMsg::Light(0));
    r.update(RangeMsg::Step(-2.0));
    assert_eq!(r.low, 10.0);
    r.update(RangeMsg::End(true));
    assert_eq!(r.low, 20.0, "the low handle stops at the high one");
    let mut ui = ui_of(ww::column(vec![r.node("Price", Msg::Range).key("range")]).padding(10.0));
    key(&mut ui, Keysym::Tab);
    assert_eq!(key(&mut ui, Keysym::Tab), vec![Msg::Range(RangeMsg::Light(1))]);
    assert_eq!(key(&mut ui, Keysym::Right), vec![Msg::Range(RangeMsg::Step(1.0))]);
    let told = ui.access("Test");
    let ends: Vec<Option<f64>> = told.nodes.iter().filter(|(_, p)| p.role() == Role::Slider).map(|(_, p)| p.numeric_value()).collect();
    assert_eq!(ends, vec![Some(20.0), Some(20.0)]);
}

fn tools() -> Toolbar {
    Toolbar::new(vec![tool("document-new-symbolic", "New"), tool("document-open-symbolic", "Open"), tool("edit-cut-symbolic", "Cut").rule(),
        tool("edit-copy-symbolic", "Copy"), tool("edit-paste-symbolic", "Paste"), tool("edit-undo-symbolic", "Undo").rule(), tool("edit-redo-symbolic", "Redo")])
}

fn narrow(b: &Toolbar) -> Node<Msg> {
    ww::column(vec![ww::row(vec![b.node(Msg::Tool).key("tools")]).width(ww::Length::Fixed(160.0))]).padding(10.0)
}

#[test]
fn a_toolbar_presses_its_tools_and_puts_those_that_do_not_fit_in_a_menu() {
    let bar = tools();
    let mut ui = ui_of(narrow(&bar));
    shot(&mut ui);
    ui.set_view(narrow(&bar));
    ui.settle();
    let r = ui.spot("tools").unwrap().rect;
    let b = theme::sheet().button;
    assert_eq!(click(&mut ui, (r.x + b / 2.0, r.y + r.h / 2.0), Button::Left), vec![Msg::Tool(ToolMsg::Press(0))]);
    let told = ui.access("Test");
    assert!(told.nodes.iter().any(|(_, p)| p.role() == Role::Toolbar));
    let names: Vec<String> = told.nodes.iter().filter(|(_, p)| p.role() == Role::Button).filter_map(|(_, p)| p.label().map(str::to_string)).collect();
    let more = crate::t!("kit-more-tools").to_string();
    assert!(names.contains(&"New".to_string()) && names.contains(&more) && !names.contains(&"Redo".to_string()), "{names:?}");
    let shown = names.len() - 1;
    let (x, y) = (r.x + r.w - 1.0, r.y + r.h / 2.0);
    let x = (0..).map(|k| x - k as f32).find(|&x| {
        let k = ui.access("Test");
        k.nodes.iter().any(|(_, p)| p.label() == Some(more.as_str()) && p.bounds().is_some_and(|b| (b.x0 as f32) <= x && x < b.x1 as f32))
    }).unwrap();
    let now = Instant::now();
    ui.input(&Input::PointerMove { x, y }, now);
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    let m = ui.copy_menu.as_ref().expect("More opens a menu");
    assert_eq!(m.popup.rows.len(), 7 - shown);
    assert_eq!(m.popup.rows.last().map(|r| r.words.as_str()), Some("Redo"));
    let mut ui = ui_of(narrow(&bar));
    key(&mut ui, Keysym::Tab);
    assert_eq!(key(&mut ui, Keysym::Right), vec![Msg::Tool(ToolMsg::Light(Some(0)))]);
    let mut lit = tools();
    lit.lit = Some(1);
    let mut ui = ui_of(narrow(&lit));
    key(&mut ui, Keysym::Tab);
    assert_eq!(key(&mut ui, Keysym::Return), vec![Msg::Tool(ToolMsg::Press(1))]);
}

#[test]
fn toggle_tools_stay_pressed_show_their_square_and_are_told_pressed() {
    let mut bar = Toolbar::new(vec![tool("format-text-bold-symbolic", "Bold").toggle(true), tool("format-text-italic-symbolic", "Italic").toggle(false), tool("edit-undo-symbolic", "Undo")]);
    let view = |b: &Toolbar| ww::column(vec![b.node(Msg::Tool).key("tools")]).padding(10.0);
    let mut ui = ui_of(view(&bar));
    let told = |ui: &mut Ui<Msg>| -> Vec<(Option<String>, Option<Toggled>)> {
        ui.access("Test").nodes.iter().filter(|(_, p)| p.role() == Role::Button).map(|(_, p)| (p.label().map(str::to_string), p.toggled())).collect()
    };
    assert_eq!(told(&mut ui), vec![(Some("Bold".into()), Some(Toggled::True)), (Some("Italic".into()), Some(Toggled::False)), (Some("Undo".into()), None)]);
    let pix = shot(&mut ui);
    let r = ui.spot("tools").unwrap().rect;
    let (b, gap) = (theme::sheet().button, theme::sheet().gap);
    let at = |x: f32| { let c = pix.pixel((r.x + x) as u32, (r.y + 2.0) as u32).unwrap(); (c.red(), c.green(), c.blue()) };
    assert_ne!(at(2.0), at(b + gap + 2.0));
    key(&mut ui, Keysym::Tab);
    assert_eq!(key(&mut ui, Keysym::Right), vec![Msg::Tool(ToolMsg::Light(Some(0)))]);
    bar.update(ToolMsg::Light(Some(1)));
    let mut ui = ui_of(view(&bar));
    key(&mut ui, Keysym::Tab);
    assert_eq!(key(&mut ui, Keysym::space), vec![Msg::Tool(ToolMsg::Press(1))]);
    assert_eq!(bar.update(ToolMsg::Press(1)), Some(1));
    assert_eq!(bar.update(ToolMsg::Press(2)), Some(2));
    assert_eq!(bar.update(ToolMsg::Press(0)), Some(0));
    assert_eq!((bar.tools[0].on, bar.tools[1].on, bar.tools[2].on), (Some(false), Some(true), None));
    assert_eq!(bar.update(ToolMsg::Press(9)), None);
    let mut ui = ui_of(view(&bar));
    assert_eq!(told(&mut ui)[..2], [(Some("Bold".into()), Some(Toggled::False)), (Some("Italic".into()), Some(Toggled::True))]);
}

#[test]
fn a_status_bar_is_told_as_status_and_a_long_path_folds_its_middle_into_a_list() {
    let mut ui = ui_of(ww::column(vec![ww::space(), status_bar(vec![ww::note("3 files"), progress_bar(Some(0.3), "Copying")])]));
    assert!(ui.access("Test").nodes.iter().any(|(_, p)| p.role() == Role::Status && p.label() == Some(&*crate::t!("kit-status"))));
    let parts: Vec<String> = ["Home", "john", "Documents", "Work", "2026", "Reports"].iter().map(|s| s.to_string()).collect();
    let mut ui = ui_of(ww::column(vec![path_bar(&parts, 2, Msg::Path)]).padding(10.0));
    assert!(ui.access("Test").nodes.iter().any(|(_, p)| p.role() == Role::Navigation));
    let drop = ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Dropdown { .. }))).unwrap();
    let Some(What::Dropdown { options, choose, .. }) = ui.tree.at(&drop.path).map(|n| &n.what) else { unreachable!() };
    assert_eq!(options, &vec!["john".to_string(), "Documents".into(), "Work".into()]);
    assert_eq!(choose(1), Msg::Path(2));
    let worded = |ui: &Ui<Msg>, w: &str| ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Button { words: Some(x), .. }) if x == w)).map(|s| s.rect);
    let home = worded(&ui, "Home").unwrap();
    assert_eq!(click(&mut ui, (home.x + home.w / 2.0, home.y + home.h / 2.0), Button::Left), vec![Msg::Path(0)]);
    assert!(worded(&ui, "2026").is_some() && worded(&ui, "Reports").is_none() && worded(&ui, "Documents").is_none());
    let ui = ui_of(path_bar(&parts[..3], 2, Msg::Path));
    assert!(!ui.spots.iter().any(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Dropdown { .. }))));
}

#[test]
fn alt_and_a_letter_opens_the_menu_with_that_letter_shown_in_the_highlight() {
    let mut b = MenuBar::new(vec![
        ("&File".into(), vec![popup::row("", "New")]),
        ("Ed&it".into(), vec![popup::row("", "Copy")]),
        ("View".into(), vec![popup::row("", "Zoom")]),
    ]);
    assert!(!b.alt('x'));
    assert!(b.open.is_none());
    assert!(b.alt('I'));
    assert_eq!((b.lit, b.open.is_some()), (Some(1), true));
    assert!(b.alt('v'));
    assert_eq!(b.lit, Some(2));
    let mut ui = bar_ui(&b);
    let alt = Mods { alt: true, ..Mods::default() };
    let msgs = ui.input(&Input::Key { sym: Keysym::f, text: None, mods: alt, repeat: false }, Instant::now()).messages;
    assert_eq!(msgs, vec![Msg::Bar(BarMsg::Alt('f'))]);
    feed(&mut b, &mut ui, msgs);
    assert_eq!(b.lit, Some(0));
    let told = ui.access("Test");
    assert!(told.nodes.iter().any(|(_, p)| p.role() == Role::MenuItem && p.label() == Some("File")));
    let first = MenuBar::new(vec![("&File".into(), vec![])]);
    let last = MenuBar::new(vec![("Fil&e".into(), vec![])]);
    let (mut a, mut c) = (bar_ui(&first), bar_ui(&last));
    assert_ne!(shot(&mut a).data(), shot(&mut c).data());
}

fn ask(ui: &mut Ui<Msg>, role: Role, label: &str, action: accesskit::Action, nth: usize) -> Vec<Msg> {
    let told = ui.access("Test");
    let id = told.nodes.iter().filter(|(_, p)| p.role() == role && p.label().is_some_and(|l| l.starts_with(label))).nth(nth).expect("the part").0;
    ui.act(&accesskit::ActionRequest { action, target_tree: accesskit::TreeId::ROOT, target_node: id, data: None }).messages
}

#[test]
fn a_tool_is_named_where_it_is_pointed_at_and_the_label_sits_under_it() {
    let bar = tools();
    let mut ui = ui_of(narrow(&bar));
    shot(&mut ui);
    ui.set_view(narrow(&bar));
    ui.settle();
    let r = ui.spot("tools").unwrap().rect;
    let b = theme::sheet().button;
    let (y, now) = (r.y + r.h / 2.0, Instant::now());
    assert_eq!(ui.label_over(r.x + b / 2.0, y).as_deref(), Some("New"));
    let second = r.x + b * 1.5 + theme::sheet().gap;
    assert_eq!(ui.label_over(second, y).as_deref(), Some("Open"));
    let mut centres = Vec::new();
    for x in [r.x + b / 2.0, second] {
        let mut ui = ui_of(narrow(&bar));
        shot(&mut ui);
        ui.set_view(narrow(&bar));
        ui.settle();
        ui.input(&Input::PointerMove { x, y }, now);
        ui.tick(now + tip_rest());
        let mut pix = tiny_skia::Pixmap::new(500, 400).unwrap();
        let drawn = ui.tip_drawn(&mut pix.as_mut()).expect("a label shows");
        centres.push(drawn.x + drawn.w / 2.0);
        assert!(drawn.y >= r.y + r.h - 1.0, "under the tool: {drawn:?}");
    }
    assert!(centres[1] > centres[0] + b / 2.0, "{centres:?}");
}

#[test]
fn the_reader_clicks_a_tool() {
    let bar = tools();
    let mut ui = ui_of(narrow(&bar));
    shot(&mut ui);
    ui.set_view(narrow(&bar));
    ui.settle();
    assert_eq!(ask(&mut ui, Role::Button, "Open", accesskit::Action::Click, 0), vec![Msg::Tool(ToolMsg::Press(1))]);
    assert_eq!(ask(&mut ui, Role::Button, "New", accesskit::Action::Click, 0), vec![Msg::Tool(ToolMsg::Press(0))]);
}

#[test]
fn the_reader_steps_and_takes_a_range_sliders_handles() {
    let r = Range::new(20.0, 80.0, 0.0, 100.0, 5.0);
    let mut ui = ui_of(ww::column(vec![r.node("Price", Msg::Range).key("range")]).padding(10.0));
    shot(&mut ui);
    assert_eq!(ask(&mut ui, Role::Slider, "Price", accesskit::Action::Decrement, 0), vec![Msg::Range(RangeMsg::Step(-1.0))]);
    assert_eq!(ask(&mut ui, Role::Slider, "Price", accesskit::Action::Increment, 1), vec![Msg::Range(RangeMsg::Light(1)), Msg::Range(RangeMsg::Step(1.0))]);
    assert_eq!(ask(&mut ui, Role::Slider, "Price", accesskit::Action::Focus, 1), vec![Msg::Range(RangeMsg::Light(1))]);
}

#[test]
fn a_range_slider_is_tall_enough_to_touch_and_each_handle_is_told_where_it_is_with_its_value() {
    let r = Range::new(20.0, 80.0, 0.0, 100.0, 5.0);
    let mut ui = ui_of(ww::column(vec![r.node("Price", Msg::Range).key("range")]).padding(10.0));
    shot(&mut ui);
    let s = ui.spot("range").unwrap().rect;
    assert!(s.h >= 24.0, "{s:?}");
    let told = ui.access("Test");
    let handles: Vec<_> = told.nodes.iter().filter(|(_, p)| p.role() == Role::Slider).map(|(_, p)| p.clone()).collect();
    assert_eq!(handles.len(), 2);
    let boxes: Vec<_> = handles.iter().map(|p| p.bounds().expect("bounds")).collect();
    assert!(boxes.iter().all(|b| b.x1 - b.x0 >= 24.0 && b.y1 - b.y0 >= 24.0), "{boxes:?}");
    assert!(boxes[0].x1 <= boxes[1].x0, "{boxes:?}");
    assert_eq!((handles[0].numeric_value(), handles[1].numeric_value()), (Some(20.0), Some(80.0)));
    assert!(boxes[1].x0 > f64::from(s.x + s.w / 2.0) - 12.0 && boxes[0].x1 < f64::from(s.x + s.w / 2.0) + 12.0);
}

#[test]
fn the_screen_reader_hears_the_figures_the_person_sees() {
    struct Chose(&'static str);
    impl crate::host::Host for Chose {
        fn numerals(&self) -> Option<String> {
            Some(self.0.to_string())
        }
    }
    let heard = |tag: &str, choice: &'static str| {
        let catalogue = crate::lang::Catalogue::new(crate::lang::Bundle::from_text(tag, ""), crate::lang::Bundle::from_files("en", crate::lang::KIT_ENGLISH));
        let host: &'static Chose = Box::leak(Box::new(Chose(choice)));
        crate::host::with(host, || crate::lang::with_catalogue(catalogue, || {
            let spin = Spin::new(4.5, 0.0, 10.0, 0.5);
            let mut ui = ui_of(ww::column(vec![spin.node("Width", Msg::Spin)]).padding(10.0));
            let told = ui.access("Test");
            told.nodes.iter().flat_map(|(_, p)| [p.label(), p.value()]).flatten().map(str::to_string).collect::<Vec<_>>()
        }))
    };
    assert!(heard("ar-EG", "auto").iter().any(|w| w.contains("٤.٥")));
    assert!(heard("ar-MA", "auto").iter().any(|w| w.contains("4.5")));
    assert!(heard("ar-EG", "latin").iter().any(|w| w.contains("4.5")));
}

#[test]
fn a_number_box_writes_the_places_it_is_told_and_its_buttons_are_big_enough_to_touch() {
    let spin = Spin::new(2.0, 0.0, 10.0, 1.0).decimals(2);
    assert_eq!(spin.text(), "2.00");
    let mut spin = Spin::new(1.0, 0.0, 10.0, 0.5).decimals(1);
    spin.update(SpinMsg::Step(1));
    assert_eq!((spin.value, spin.text()), (1.5, "1.5".to_string()));
    let mut ui = ui_of(ww::column(vec![spin.node("Width", Msg::Spin)]).padding(10.0));
    ui.access("Test");
    let buttons: Vec<_> = ui.spots.iter().filter(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Button { .. }))).collect();
    assert_eq!(buttons.len(), 2);
    assert!(buttons.iter().all(|b| b.rect.w >= 24.0 && b.rect.h >= 24.0), "{:?}", buttons.iter().map(|b| b.rect).collect::<Vec<_>>());
}

#[test]
fn limits_given_the_wrong_way_round_are_swapped_not_a_panic() {
    let spin = Spin::new(4.0, 10.0, 0.0, 1.0);
    assert_eq!((spin.min, spin.max, spin.value), (0.0, 10.0, 4.0));
    let range = Range::new(20.0, 80.0, 100.0, 0.0, 5.0);
    assert_eq!((range.min, range.max, range.low, range.high), (0.0, 100.0, 20.0, 80.0));
}
