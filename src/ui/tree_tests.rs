//! Tests of the tree view: keys, clicks on rows and arrows, scrolling and painting.

use super::*;
use super::super::widget::{column, item, scroll, tree, tree_row, Length, TreeRow};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Pick(usize),
    Toggle(usize),
    Open(usize),
}

fn rows() -> Vec<TreeRow> {
    let mut rows = vec![
        tree_row(item("Documents"), 0).branch(true),
        tree_row(item("Letters"), 1).branch(false),
        tree_row(item("Notes"), 1),
        tree_row(item("Pictures"), 0).branch(false),
    ];
    rows.extend((0..30).map(|i| tree_row(item(format!("File {i}")), 0)));
    rows
}

fn view(chosen: Option<usize>) -> Node<Msg> {
    column(vec![scroll("area", tree(rows(), chosen, Msg::Pick, Msg::Toggle, Msg::Open).key("tree")).height(Length::Fixed(200.0))])
}

fn tree_ui(chosen: Option<usize>) -> Ui<Msg> {
    let mut ui = Ui::new(view(chosen), Theme::new(false, "#fbc711"));
    ui.size = (400, 300);
    ui.focus = Some("tree".into());
    ui.settle();
    ui
}

fn key(sym: Keysym) -> Input {
    Input::Key { sym, text: None, mods: Mods::default(), repeat: false }
}

fn press(ui: &mut Ui<Msg>, sym: Keysym) -> Vec<Msg> {
    ui.input(&key(sym), Instant::now()).messages
}

#[test]
fn the_arrows_open_close_and_move_through_a_tree() {
    assert_eq!(press(&mut tree_ui(Some(0)), Keysym::Right), vec![Msg::Pick(1)]);
    assert_eq!(press(&mut tree_ui(Some(0)), Keysym::Left), vec![Msg::Toggle(0)]);
    assert_eq!(press(&mut tree_ui(Some(1)), Keysym::Right), vec![Msg::Toggle(1)]);
    assert_eq!(press(&mut tree_ui(Some(1)), Keysym::Left), vec![Msg::Pick(0)]);
    assert!(press(&mut tree_ui(Some(2)), Keysym::Right).is_empty());
    assert_eq!(press(&mut tree_ui(Some(2)), Keysym::Left), vec![Msg::Pick(0)]);
    assert!(press(&mut tree_ui(Some(3)), Keysym::Left).is_empty());
    assert_eq!(press(&mut tree_ui(Some(1)), Keysym::Down), vec![Msg::Pick(2)]);
    assert_eq!(press(&mut tree_ui(Some(1)), Keysym::Up), vec![Msg::Pick(0)]);
    assert_eq!(press(&mut tree_ui(Some(2)), Keysym::Home), vec![Msg::Pick(0)]);
    assert_eq!(press(&mut tree_ui(None), Keysym::Down), vec![Msg::Pick(0)]);
    for sym in [Keysym::Return, Keysym::space] {
        assert_eq!(press(&mut tree_ui(Some(0)), sym), vec![Msg::Toggle(0)]);
        assert_eq!(press(&mut tree_ui(Some(2)), sym), vec![Msg::Open(2)]);
    }
}

#[test]
fn end_scrolls_the_chosen_row_into_sight_and_so_does_the_program() {
    let mut ui = tree_ui(Some(0));
    let last = rows().len() - 1;
    assert_eq!(press(&mut ui, Keysym::End), vec![Msg::Pick(last)]);
    let s = ui.spot("area").unwrap();
    assert_eq!(ui.offset(s), (s.full - s.rect.h).round());
    let mut ui = tree_ui(Some(0));
    ui.set_view(view(Some(last)));
    let (s, t) = (ui.spot("area").unwrap(), ui.spot("tree").unwrap());
    let row = ui.px(theme::sheet().row);
    let top = t.rect.y + last as f32 * row;
    assert!(top >= s.rect.y && top + row <= s.rect.y + s.rect.h, "{top}");
}

#[test]
fn a_click_on_the_arrow_opens_and_one_on_the_words_chooses() {
    let now = Instant::now();
    let click = |ui: &mut Ui<Msg>, x: f32, y: f32, at: Instant| {
        ui.input(&Input::Press { button: Button::Left, x, y }, at);
        ui.input(&Input::Release { button: Button::Left, x, y }, at).messages
    };
    let mut ui = tree_ui(None);
    let r = ui.spot("tree").unwrap().rect;
    let row = ui.px(theme::sheet().row);
    let arrow = r.x + ui.tree_step(1) + ui.px(theme::sheet().entry_padding) + ui.px(w::CHEVRON) / 2.0;
    let label_x = r.x + 200.0;
    let (soon, later) = (now + Duration::from_millis(200), now + Duration::from_secs(1));
    assert_eq!(click(&mut ui, arrow, r.y + 1.5 * row, now), vec![Msg::Toggle(1)]);
    assert_eq!(click(&mut ui, arrow, r.y + 0.5 * row, later), vec![Msg::Pick(0)]);
    let mut ui = tree_ui(None);
    assert_eq!(click(&mut ui, label_x, r.y + 1.5 * row, now), vec![Msg::Pick(1)]);
    assert_eq!(click(&mut ui, label_x, r.y + 1.5 * row, soon), vec![Msg::Pick(1), Msg::Toggle(1)]);
    let mut ui = tree_ui(None);
    click(&mut ui, label_x, r.y + 2.5 * row, now);
    assert_eq!(click(&mut ui, label_x, r.y + 2.5 * row, soon), vec![Msg::Pick(2), Msg::Open(2)]);
}

#[test]
fn a_deeper_row_starts_its_words_further_in() {
    let rows = vec![tree_row(item("Wide words"), 0), tree_row(item("Wide words"), 2)];
    let mut ui = Ui::new(column(vec![tree(rows, None, Msg::Pick, Msg::Toggle, Msg::Open).key("tree")]), Theme::new(false, "#fbc711"));
    ui.size = (400, 200);
    ui.settle();
    let mut pm = tiny_skia::Pixmap::new(400, 200).unwrap();
    ui.paint(&mut pm.as_mut(), 1.0);
    let r = ui.spot("tree").unwrap().rect;
    let row = ui.px(theme::sheet().row);
    let start = |i: f32| {
        let (top, bottom) = ((r.y + i * row) as u32 + 2, (r.y + (i + 1.0) * row) as u32 - 2);
        let back = pm.pixel(r.x as u32 + 3, (top + bottom) / 2).unwrap();
        ((r.x as u32 + 3)..(r.x + r.w) as u32).find(|&x| (top..bottom).any(|y| pm.pixel(x, y).unwrap() != back)).unwrap() as f32
    };
    let (top, deep) = (start(0.0), start(1.0));
    assert!((deep - top - ui.tree_step(2)).abs() <= 1.0, "{top} {deep}");
}
