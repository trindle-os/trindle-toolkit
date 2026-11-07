//! A typing box laid over a name selects it as it opens, keeps Tab for the next name, and moves nothing round it.

use super::*;
use super::shared_tests::{key, typed};
use super::super::theme::Theme;
use super::super::widget::{self as ww, InPlace, Length};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Text(String),
    Keep,
    Left,
    Stop,
    Next(bool),
    Go,
}

const W: u32 = 600;
const H: u32 = 400;
const ROW: f32 = 40.0;

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(false, "#fbc711"));
    ui.size = (W, H);
    ui.settle();
    ui
}

fn name_box(text: &str, turn: u32) -> Node<Msg> {
    ww::entry(text, "New name for report.pdf", Msg::Text).key("name").grab(turn).selects(0..6)
        .on_submit(Msg::Keep).on_escape(Msg::Stop).on_leave(Msg::Left).on_tab(Msg::Next)
}

fn list(text: &str, at: Option<usize>, problem: Option<&str>) -> Node<Msg> {
    let rows: Vec<Node<Msg>> = (0..5).map(|i| {
        ww::drawing((100.0, ROW), format!("Row {i}"), |_, _| {}).key(format!("row{i}")).width(Length::Fill).height(Length::Fixed(ROW))
    }).collect();
    let edit = at.map(|i| InPlace {
        place: Rect { x: 40.0, y: i as f32 * ROW + 8.0, w: 200.0, h: 24.0 },
        entry: name_box(text, 1),
        problem: problem.map(str::to_string),
    });
    ww::column(vec![
        ww::scroll("list", ww::typing_over(ww::column(rows).gap(0.0), edit)).height(Length::Fixed(160.0)),
        ww::button("go-next-symbolic", "Go").on_press(Msg::Go).key("go"),
    ]).padding(10.0).gap(10.0)
}

fn places(ui: &Ui<Msg>) -> Vec<(String, Rect)> {
    ui.spots.iter().map(|s| (s.id.clone(), s.rect)).collect()
}

#[test]
fn a_box_selects_part_of_its_text_as_it_first_shows_and_typing_replaces_it() {
    let mut ui = ui_of(list("report.pdf", Some(1), None));
    assert_eq!(ui.focus.as_deref(), Some("name"));
    let t = ui.typing().unwrap();
    assert_eq!((t.text.as_str(), t.anchor, t.caret), ("report.pdf", 0, 6));
    let reply = ui.input(&typed("n"), Instant::now());
    assert_eq!(reply.messages, vec![Msg::Text("n.pdf".into())]);
    ui.set_view(list("n.pdf", Some(1), None));
    let t = ui.typing().unwrap();
    assert_eq!((t.text.as_str(), t.anchor, t.caret), ("n.pdf", 1, 1));
    ui.set_view(list("n.pdf", None, None));
    ui.settle();
    ui.set_view(list("report.pdf", Some(2), None));
    let t = ui.typing();
    assert!(t.is_none(), "the same turn does not take the keyboard twice");
    assert_eq!(ui.edits.get("name").map(|e| e.selection()), Some(0..6));
}

#[test]
fn a_selection_asked_for_past_the_text_or_inside_a_letter_stops_at_a_letters_edge() {
    let short = ww::entry("ab", "Name", Msg::Text).key("name").selects(0..6);
    let ui = ui_of(ww::column(vec![short]));
    assert_eq!(ui.edits.get("name").map(|e| e.selection()), Some(0..2));
    let accent = ww::entry("caf\u{e9}.txt", "Name", Msg::Text).key("name").selects(0..4);
    let ui = ui_of(ww::column(vec![accent]));
    assert_eq!(ui.edits.get("name").map(|e| e.selection()), Some(0..3));
}

#[test]
fn enter_escape_and_tab_in_the_box_go_to_the_program() {
    let mut ui = ui_of(list("report.pdf", Some(1), None));
    assert_eq!(ui.input(&key(Keysym::Return), Instant::now()).messages, vec![Msg::Keep]);
    assert_eq!(ui.input(&key(Keysym::Escape), Instant::now()).messages, vec![Msg::Stop]);
    assert_eq!(ui.input(&key(Keysym::Tab), Instant::now()).messages, vec![Msg::Next(true)]);
    assert_eq!(ui.focus.as_deref(), Some("name"));
    let back = Input::Key { sym: Keysym::ISO_Left_Tab, text: None, mods: Mods { shift: true, ..Mods::default() }, repeat: false };
    assert_eq!(ui.input(&back, Instant::now()).messages, vec![Msg::Next(false)]);
    assert_eq!(ui.focus.as_deref(), Some("name"));
    let plain = ww::column(vec![ww::entry("a", "One", Msg::Text).key("one").first(), ww::button("go-next-symbolic", "Go").on_press(Msg::Go).key("go")]);
    let mut ui = ui_of(plain);
    assert!(ui.input(&key(Keysym::Tab), Instant::now()).messages.is_empty());
    assert_eq!(ui.focus.as_deref(), Some("go"));
}

#[test]
fn the_program_hears_when_the_keyboard_leaves_the_box_after_what_the_click_sent() {
    let mut ui = ui_of(list("report.pdf", Some(1), None));
    let go = ui.spot("go").unwrap().rect;
    let (x, y) = (go.x + go.w / 2.0, go.y + go.h / 2.0);
    let reply = ui.input(&Input::Press { button: Button::Left, x, y }, Instant::now());
    assert_eq!(reply.messages, vec![Msg::Left]);
    assert_eq!(ui.input(&Input::Release { button: Button::Left, x, y }, Instant::now()).messages, vec![Msg::Go]);
    let mut ui = ui_of(list("report.pdf", Some(1), None));
    let row = ui.spot("row3").unwrap().rect;
    let reply = ui.input(&Input::Press { button: Button::Left, x: row.x + 300.0, y: row.y + 10.0 }, Instant::now());
    assert_eq!(reply.messages, vec![Msg::Left]);
    assert_eq!(ui.focus, None);
    let mut ui = ui_of(list("report.pdf", Some(1), None));
    let name = ui.spot("name").unwrap().rect;
    let reply = ui.input(&Input::Press { button: Button::Left, x: name.x + 20.0, y: name.y + 10.0 }, Instant::now());
    assert!(reply.messages.is_empty());
    assert_eq!(ui.focus.as_deref(), Some("name"));
    ui.set_view(list("report.pdf", None, None));
    assert!(ui.input(&Input::PointerMove { x: 5.0, y: 5.0 }, Instant::now()).messages.is_empty());
    assert_eq!(ui.focus, None);
}

#[test]
fn a_box_over_a_row_takes_its_place_and_moves_nothing() {
    let mut plain = ui_of(list("report.pdf", None, None));
    let before = places(&plain);
    assert!(plain.spot("name").is_none());
    for problem in [None, Some("There is already something called notes.txt here.")] {
        let ui = ui_of(list("report.pdf", Some(1), problem));
        let now = places(&ui);
        for (id, was) in &before {
            assert_eq!(now.iter().find(|(n, _)| n == id).map(|(_, r)| *r), Some(*was), "{id} moved, {problem:?}");
        }
        let row = ui.spot("row1").unwrap().rect;
        let name = ui.spot("name").unwrap().rect;
        assert_eq!(name, Rect { x: row.x + 40.0, y: row.y + 8.0, w: 200.0, h: 24.0 });
        let words: Vec<&Spot> = ui.spots.iter().filter(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what),
            Some(What::Label { tone: Tone::Problem, .. }))).collect();
        assert_eq!(words.len(), usize::from(problem.is_some()));
        if let Some(w) = words.first() {
            assert!(w.rect.y >= name.y + name.h && w.rect.y < name.y + name.h + ROW, "{:?} under {name:?}", w.rect);
            assert!(w.rect.y + w.rect.h > row.y + row.h, "they lie over the next row, which has not moved");
        }
    }
    let row = plain.spot("row3").unwrap().rect;
    plain.set_view(list("report.pdf", Some(1), None));
    plain.settle();
    assert_eq!(plain.spot("row3").map(|s| s.rect), Some(row));
}

#[test]
fn a_box_in_a_row_scrolls_with_the_row() {
    let mut ui = ui_of(list("report.pdf", Some(4), None));
    let (row, name) = (ui.spot("row4").unwrap().rect, ui.spot("name").unwrap().rect);
    ui.pointer = Some((50.0, 50.0));
    ui.input(&Input::Scroll { x: 0.0, y: 1.0, steps: true }, Instant::now());
    ui.settle();
    let (row_now, name_now) = (ui.spot("row4").unwrap().rect, ui.spot("name").unwrap().rect);
    assert!(row_now.y < row.y, "the list scrolled");
    assert_eq!(name_now.y - name.y, row_now.y - row.y);
}

#[test]
fn a_snug_box_keeps_little_room_at_its_ends_and_a_centred_one_puts_its_words_in_its_middle() {
    let sheet = theme::sheet();
    let boxes = ww::column(vec![
        ww::entry("report", "Name", Msg::Text).key("usual").width(Length::Fixed(300.0)),
        ww::entry("report", "Name", Msg::Text).key("snug").snug().width(Length::Fixed(300.0)),
        ww::entry("report", "Name", Msg::Text).key("middle").snug().centred().width(Length::Fixed(300.0)),
    ]);
    let mut ui = ui_of(boxes);
    let words = ui.entry_layout("usual", "report", false).width;
    let mut start = |id: &str| {
        ui.focus = Some(id.to_string());
        ui.edit(id).unwrap().move_to(0, false);
        let rect = ui.spot(id).unwrap().rect;
        ui.typing().unwrap().area.0 as f32 - rect.x
    };
    assert_eq!(start("usual"), sheet.entry_padding.round());
    assert_eq!(start("snug"), sheet.snug_padding.round());
    let middle = start("middle");
    assert!((middle - (300.0 - words) / 2.0).abs() <= 1.0, "{middle} is the middle for words {words} wide");
    let rect = ui.spot("middle").unwrap().rect;
    assert_eq!(ui.entry_hit("middle", rect.x + middle + 1.0), 0);
    assert_eq!(ui.entry_hit("middle", rect.x + middle + words + 4.0), "report".len());
    let mut pix = Pixmap::new(W, H).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    assert!(pix.data().iter().any(|&b| b != 0), "nothing was painted");
}

#[test]
fn a_box_can_keep_the_weight_of_the_words_it_stands_for() {
    let boxes = ww::column(vec![
        ww::entry("report", "Name", Msg::Text).key("usual"),
        ww::entry("report", "Name", Msg::Text).key("bold").weight(700),
    ]);
    let mut ui = ui_of(boxes);
    assert_eq!(ui.entry_style("usual").weight, Words::Plain.style(1.0).weight);
    assert_eq!(ui.entry_style("bold").weight, 700);
    let (usual, bold) = (ui.entry_layout("usual", "report", false).width, ui.entry_layout("bold", "report", false).width);
    assert!(bold > usual, "{bold} against {usual}");
    let rect = ui.spot("bold").unwrap().rect;
    let pad = ui.entry_pad("bold");
    assert_eq!(ui.entry_hit("bold", rect.x + pad + bold + 2.0), "report".len());
    ui.focus = Some("bold".into());
    ui.edit("bold").unwrap().move_to(6, false);
    let caret = ui.typing().unwrap().area.0 as f32;
    assert!((caret - (rect.x + pad + bold)).abs() <= 1.0, "the caret stands after the bold words");
}
