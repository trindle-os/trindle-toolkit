//! An outline beside a long page: choosing a heading goes there, and the heading being read is marked.

use super::*;
use super::super::outline::{outline, Entry};
use super::super::panel::{side_panel, Side};
use super::super::theme::Theme;
use super::super::widget as ww;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Go(usize),
}

fn entries() -> Vec<Entry> {
    vec![Entry::new("Start", 0, "h0"), Entry::new("Setting up", 1, "h1"), Entry::new("Using it", 0, "h2"), Entry::new("The end", 0, "h3")]
}

fn page(current: Option<usize>) -> Ui<Msg> {
    let mut items = Vec::new();
    for (i, e) in entries().iter().enumerate() {
        items.push(ww::section(e.title.clone()).key(format!("h{i}")));
        items.push(ww::label("Words ".repeat(400)));
    }
    let content = ww::scroll("page", ww::column(items).gap(16.0));
    let mut ui = Ui::new(side_panel(content, outline("outline", &entries(), current, Msg::Go), 220.0, 1.0, Side::Start), Theme::new(true, "#fbc711"));
    ui.size = (800, 400);
    ui.settle();
    ui
}

const KEYS: [&str; 4] = ["h0", "h1", "h2", "h3"];

#[test]
fn going_to_a_heading_brings_it_to_the_top_and_it_is_then_the_one_read() {
    let mut ui = page(Some(0));
    assert_eq!(ui.reading(&KEYS), Some(0));
    assert!(ui.go_to("h2"));
    let (top, area) = (ui.spot("h2").unwrap().rect.y, ui.spot("page").unwrap().rect.y);
    assert!((top - area).abs() < 1.0, "{top} {area}");
    assert_eq!(ui.reading(&KEYS), Some(2));
    assert!(!ui.go_to("h2"), "already there");
    assert_eq!(ui.reading(&["missing"]), None);
}

#[test]
fn choosing_a_row_of_the_outline_says_which_and_a_reader_hears_the_titles() {
    let mut ui = page(Some(0));
    let r = ui.spot("outline").unwrap().rect;
    let row = ui.px(theme::sheet().row);
    let (x, y) = (r.x + 40.0, r.y + row * 2.5);
    let now = Instant::now();
    let mut sent = ui.input(&Input::Press { button: Button::Left, x, y }, now).messages;
    sent.extend(ui.input(&Input::Release { button: Button::Left, x, y }, now).messages);
    assert_eq!(sent, vec![Msg::Go(2)]);
    let told = ui.access("Test");
    assert!(told.nodes.iter().any(|(_, n)| n.label().is_some_and(|l| l.trim() == "Setting up")));
}
