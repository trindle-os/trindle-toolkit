//! Nothing moves because something was pressed: scrolling areas and puzzles of cards hold still.

use super::*;
use super::shared_tests::key;
use super::super::theme::Theme;
use super::super::widget::{self as ww, Length};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Go,
    Flick(bool),
    Pick(usize),
}

const W: u32 = 1278;
const H: u32 = 692;

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(true, "#fbc711"));
    ui.size = (W, H);
    ui.settle();
    ui
}

fn wheel(y: f32) -> Input {
    Input::Scroll { x: 0.0, y, steps: true }
}

fn page(name: &str, rows: usize) -> Node<Msg> {
    let mut items: Vec<Node<Msg>> = (0..rows).map(|i| {
        ww::row(vec![ww::label(format!("Setting {i}")).width(Length::Fill), ww::switch(i % 2 == 0, format!("Setting {i}"), Msg::Flick)])
    }).collect();
    items.push(ww::button("go-next-symbolic", "Go").on_press(Msg::Go).key("go"));
    ww::scroll(name, ww::column(items).gap(8.0).padding(20.0)).height(Length::Fill)
}

fn window(page: Node<Msg>) -> Node<Msg> {
    let side = ww::card(ww::scroll("side", ww::column((0..60).map(|i| ww::label(format!("Page {i}"))).collect())))
        .width(Length::Fixed(220.0)).height(Length::Fill);
    ww::row(vec![side, page])
}

fn place(ui: &Ui<Msg>, area: &str) -> f32 {
    ui.offset(ui.spot(area).unwrap_or_else(|| panic!("no area {area}")))
}

fn scrolled(ui: &mut Ui<Msg>, area: &str, by: f32) -> f32 {
    let now = Instant::now();
    ui.settle();
    let r = ui.spot(area).unwrap().rect;
    ui.input(&Input::PointerMove { x: r.x + r.w / 2.0, y: r.y + r.h / 2.0 }, now);
    ui.input(&wheel(by), now);
    let at = place(ui, area);
    assert!(at > 0.0, "{area} scrolled");
    at
}

#[test]
fn a_scrolled_page_keeps_its_place_when_the_window_changes_shape_above_it() {
    let mut ui = ui_of(window(page("page:wifi", 60)));
    let at = scrolled(&mut ui, "page:wifi", 6.0);
    let side = scrolled(&mut ui, "side", 3.0);
    let switch = ui.spots.iter().find(|s| s.id.starts_with("page:wifi/") && matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Switch { .. })))
        .map(|s| (s.id.clone(), s.rect)).expect("a switch on the page");
    ui.focus = Some(switch.0.clone());
    let same = |ui: &mut Ui<Msg>, why: &str| {
        ui.settle();
        assert_eq!(place(ui, "page:wifi"), at, "the page, {why}");
        assert_eq!(place(ui, "side"), side, "the side bar, {why}");
        assert_eq!(ui.focus.as_deref(), Some(switch.0.as_str()), "the keyboard, {why}");
    };
    let panel = ww::placed(ww::card(ww::label("About this page")).floats(), |_, _| (300.0, 200.0));
    ui.set_view(ww::layers(vec![window(page("page:wifi", 60)), panel]));
    same(&mut ui, "with a panel floating over the window");
    assert_eq!(ui.spot(&switch.0).unwrap().rect, switch.1, "the switch itself");
    ui.set_view(window(page("page:wifi", 60)));
    same(&mut ui, "with the panel gone again");
    ui.set_view(ww::column(vec![ww::label("Reading the settings"), window(page("page:wifi", 60))]));
    same(&mut ui, "with a line above it");
    ui.set_view(ww::row(vec![ww::label("New"), window(page("page:wifi", 60)), ww::label("Also new")]));
    same(&mut ui, "with neighbours either side");
    ui.set_view(window(page("page:wifi", 60)));
    same(&mut ui, "as it was");
    assert_eq!(ui.spot(&switch.0).unwrap().rect, switch.1);
}

#[test]
fn a_page_away_for_a_view_comes_back_where_it_was() {
    let mut ui = ui_of(window(page("page:wifi", 60)));
    let at = scrolled(&mut ui, "page:wifi", 6.0);
    for _ in 0..2 {
        ui.set_view(window(ww::label("Reading...")));
        ui.settle();
        assert!(ui.spot("page:wifi").is_none());
    }
    ui.set_view(window(page("page:wifi", 60)));
    ui.settle();
    assert_eq!(place(&ui, "page:wifi"), at);
    ui.set_view(window(ww::label("Reading...")));
    ui.input(&Input::PointerMove { x: 600.0, y: 300.0 }, Instant::now());
    ui.set_view(window(page("page:wifi", 60)));
    ui.settle();
    assert_eq!(place(&ui, "page:wifi"), at);
    ui.set_view(window(ww::label("Reading...")));
    ui.input(&Input::Press { button: Button::Left, x: 600.0, y: 300.0 }, Instant::now());
    ui.set_view(window(page("page:wifi", 60)));
    ui.settle();
    assert_eq!(place(&ui, "page:wifi"), 0.0);
}

#[test]
fn another_page_starts_at_its_top_and_the_side_bar_stays() {
    let mut ui = ui_of(window(page("page:wifi", 60)));
    scrolled(&mut ui, "page:wifi", 6.0);
    let side = scrolled(&mut ui, "side", 3.0);
    ui.set_view(window(page("page:sound", 60)));
    ui.settle();
    assert_eq!(place(&ui, "page:sound"), 0.0, "a new page is at its top");
    assert_eq!(place(&ui, "side"), side, "the side bar did not move");
    ui.set_view(window(page("page:wifi", 60)));
    ui.settle();
    assert_eq!(place(&ui, "page:wifi"), 0.0);
    assert_eq!(place(&ui, "side"), side);
}

#[test]
fn what_changes_on_a_page_leaves_its_place() {
    let mut ui = ui_of(window(page("page:wifi", 60)));
    let at = scrolled(&mut ui, "page:wifi", 6.0);
    for rows in [61, 75, 40, 60] {
        ui.set_view(window(page("page:wifi", rows)));
        ui.settle();
        assert_eq!(place(&ui, "page:wifi"), at, "with {rows} rows");
    }
    ui.set_view(window(page("page:wifi", 22)));
    ui.settle();
    let s = ui.spot("page:wifi").unwrap();
    assert_eq!(place(&ui, "page:wifi"), (s.full - s.rect.h).max(0.0).round());
    ui.set_view(window(page("page:wifi", 60)));
    ui.settle();
    assert_eq!(place(&ui, "page:wifi"), at);
}

#[test]
fn a_redraw_moves_neither_the_keyboard_nor_the_page() {
    let now = Instant::now();
    let mut ui = ui_of(window(page("page:wifi", 60)));
    ui.focus = Some("go".into());
    ui.reveal("go", None);
    assert!(place(&ui, "page:wifi") > 0.0);
    let r = ui.spot("page:wifi").unwrap().rect;
    ui.input(&Input::PointerMove { x: r.x + r.w / 2.0, y: r.y + r.h / 2.0 }, now);
    ui.input(&wheel(-4.0), now);
    let at = place(&ui, "page:wifi");
    for _ in 0..3 {
        ui.set_view(window(page("page:wifi", 60)));
        ui.settle();
        assert_eq!(place(&ui, "page:wifi"), at, "a redraw does not bring the keyboard's widget back into sight");
        assert_eq!(ui.focus.as_deref(), Some("go"));
    }
    let first = |on: bool| {
        let mut items: Vec<Node<Msg>> = vec![ww::button("go-next-symbolic", "First").on_press(Msg::Go).key("first").first()];
        items.extend((0..60).map(|i| ww::switch(on, format!("Setting {i}"), Msg::Flick)));
        items.push(ww::button("go-next-symbolic", "Last").on_press(Msg::Go).key("last"));
        ww::scroll("page", ww::column(items).gap(8.0))
    };
    let mut ui = ui_of(first(false));
    assert_eq!(ui.focus.as_deref(), Some("first"));
    ui.input(&Input::PointerMove { x: 300.0, y: 300.0 }, now);
    ui.input(&wheel(1000.0), now);
    let at = place(&ui, "page");
    let last = ui.spot("last").unwrap().rect;
    ui.input(&Input::PointerMove { x: last.x + 4.0, y: last.y + 4.0 }, now);
    ui.input(&Input::Press { button: Button::Left, x: last.x + 4.0, y: last.y + 4.0 }, now);
    ui.input(&Input::Release { button: Button::Left, x: last.x + 4.0, y: last.y + 4.0 }, now);
    ui.set_view(first(true));
    ui.settle();
    assert_eq!(place(&ui, "page"), at, "pressing a button at the end leaves the page there");
    assert_ne!(ui.focus.as_deref(), Some("first"));
    assert_eq!(ui.input(&key(Keysym::Shift_L), now).messages, Vec::new());
    assert_eq!(place(&ui, "page"), at);
}

#[test]
fn a_row_the_person_pressed_is_not_scrolled_to_but_one_the_program_chose_is() {
    let now = Instant::now();
    let view = |chosen: Option<usize>| {
        let rows = (0..40).map(|i| ww::item(format!("Row {i}"))).collect();
        ww::scroll("page", ww::column(vec![
            ww::space().height(Length::Fixed(300.0)),
            ww::list(rows, chosen, Msg::Pick).key("rows"),
        ]))
    };
    let mut ui = ui_of(view(None));
    let row = ui.row_of("rows");
    let top = ui.spot("rows").unwrap().rect.y;
    let cut = ((H as f32 - top) / row).floor() as usize;
    let y = top + cut as f32 * row + 2.0;
    assert!(y < H as f32 && y + row > H as f32, "row {cut} is cut by the edge");
    ui.input(&Input::PointerMove { x: 100.0, y }, now);
    let mut said = ui.input(&Input::Press { button: Button::Left, x: 100.0, y }, now).messages;
    said.extend(ui.input(&Input::Release { button: Button::Left, x: 100.0, y }, now).messages);
    assert_eq!(said, vec![Msg::Pick(cut)]);
    ui.set_view(view(Some(cut)));
    ui.settle();
    assert_eq!(place(&ui, "page"), 0.0, "nothing moved for the row they pressed");
    ui.set_view(view(Some(39)));
    ui.settle();
    assert!(place(&ui, "page") > 0.0);
}

#[test]
fn places_and_arrangements_never_pile_up() {
    let mut ui = ui_of(window(cards_page("page:0", &[14; 6])));
    for n in 0..200 {
        let name = format!("page:{n}");
        ui.set_view(window(cards_page(&name, &[14; 6])));
        scrolled(&mut ui, &name, 2.0);
        if n % 3 == 0 {
            ui.set_view(window(ww::label("Reading...")));
            ui.settle();
        }
        assert!(ui.scrolls.len() <= 2, "{} places kept", ui.scrolls.len());
        assert!(ui.areas.len() <= 2);
        assert!(ui.puzzles.len() <= 1);
        assert!(ui.held.len() <= 3, "{} arrangements held", ui.held.len());
    }
    for n in 0..200u32 {
        ui.size = (900 + (n * 37) % 700, H);
        ui.laid = None;
        ui.set_view(window(cards_page("page:same", &[3 + (n as usize % 4); 6])));
        ui.settle();
        assert!(ui.held.len() <= 3, "{} arrangements held", ui.held.len());
    }
}

fn a_card(i: usize, rows: usize, words: &str) -> Node<Msg> {
    let mut lines = vec![ww::heading(format!("Card {i}")).key(format!("name{i}"))];
    lines.extend((0..rows).map(|r| ww::row(vec![ww::label(format!("{words} {r}")).width(Length::Fill), ww::switch(r % 2 == 0, "A switch", Msg::Flick)])));
    ww::card(ww::column(lines).gap(8.0)).beside().span(4, 6, 300.0)
}

fn cards_page(name: &str, rows: &[usize]) -> Node<Msg> {
    cards_page_with(name, rows, usize::MAX, "A setting")
}

fn cards_page_with(name: &str, rows: &[usize], odd: usize, words: &str) -> Node<Msg> {
    let mut all = vec![ww::page_title("A page"), ww::headline("Everything is as it should be.").width(Length::Fill)];
    all.extend(rows.iter().enumerate().map(|(i, &n)| a_card(i, n, if i == odd { words } else { "A setting" })));
    ww::scroll(name, ww::bento(300.0, 12.0, all).padding(20.0)).height(Length::Fill)
}

fn cards(ui: &Ui<Msg>, n: usize) -> (Vec<Rect>, Vec<usize>) {
    let mut seen = Vec::new();
    let rects = (0..n).map(|i| {
        let name = ui.spot(&format!("name{i}")).unwrap_or_else(|| panic!("card {i}"));
        let path = &name.path[..name.path.len() - 2];
        let at = ui.spots.iter().position(|s| s.path == path).expect("the card");
        seen.push((at, i));
        ui.spots[at].rect
    }).collect();
    seen.sort();
    (rects, seen.into_iter().map(|(_, i)| i).collect())
}

#[allow(clippy::needless_range_loop)]
fn held_still(was: &(Vec<Rect>, Vec<usize>), now: &(Vec<Rect>, Vec<usize>), k: usize, why: &str) {
    assert_eq!(was.1, now.1, "the order, {why}");
    let n = was.0.len();
    let beside = |a: &Rect, b: &Rect| a.x < b.x + b.w - 0.5 && b.x < a.x + a.w - 0.5;
    let mut under = vec![false; n];
    under[k] = true;
    loop {
        let more: Vec<usize> = (0..n).filter(|&t| !under[t] && (0..n).any(|s| under[s] && beside(&was.0[s], &was.0[t]) && was.0[s].y < was.0[t].y)).collect();
        if more.is_empty() {
            break;
        }
        more.into_iter().for_each(|t| under[t] = true);
    }
    for t in 0..n {
        let (a, b) = (was.0[t], now.0[t]);
        assert_eq!((a.x, a.w), (b.x, b.w), "card {t} keeps its column and width, {why}");
        if t == k || !under[t] {
            assert_eq!(a.y, b.y, "card {t} is not under card {k} and stays where it is, {why}");
        }
    }
    for a in 0..n {
        for b in a + 1..n {
            let (p, q) = (now.0[a], now.0[b]);
            assert!(!(beside(&p, &q) && p.y < q.y + q.h - 0.5 && q.y < p.y + p.h - 0.5), "cards {a} and {b} overlap, {why}");
        }
    }
}

const ROWS: [usize; 7] = [4, 2, 6, 3, 5, 2, 4];

#[test]
fn a_packed_puzzle_holds_still_when_a_cards_contents_change() {
    for k in 0..ROWS.len() {
        let mut ui = ui_of(window(cards_page("page", &ROWS)));
        let was = cards(&ui, ROWS.len());
        assert!(was.0.iter().any(|r| r.x > was.0[0].x + 1.0), "the cards sit side by side: {:?}", was.0);
        let mut rows = ROWS;
        for (to, why) in [(ROWS[k] + 1, "a row more"), (ROWS[k] + 9, "many rows more"), (1, "rows fewer"), (0, "no rows"), (ROWS[k], "as it was")] {
            rows[k] = to;
            ui.set_view(window(cards_page("page", &rows)));
            ui.settle();
            let now = cards(&ui, ROWS.len());
            held_still(&was, &now, k, &format!("card {k} with {why}"));
            if to == ROWS[k] {
                assert_eq!(was.0, now.0, "back as it was");
            }
        }
        let long = "A setting whose words have grown far longer than they were when the page was first packed, so they wrap";
        ui.set_view(window(cards_page_with("page", &ROWS, k, long)));
        ui.settle();
        held_still(&was, &cards(&ui, ROWS.len()), k, &format!("card {k} with longer words"));
    }
}

#[test]
fn a_card_that_grows_pushes_down_what_is_under_it() {
    let mut ui = ui_of(window(cards_page("page", &ROWS)));
    let was = cards(&ui, ROWS.len());
    let beside = |a: &Rect, b: &Rect| a.x < b.x + b.w - 0.5 && b.x < a.x + a.w - 0.5;
    let below: Vec<usize> = (1..ROWS.len()).filter(|&t| beside(&was.0[0], &was.0[t]) && was.0[t].y > was.0[0].y).collect();
    assert!(!below.is_empty(), "{:?}", was.0);
    let mut rows = ROWS;
    rows[0] += 12;
    ui.set_view(window(cards_page("page", &rows)));
    ui.settle();
    let now = cards(&ui, ROWS.len());
    assert!(now.0[0].h > was.0[0].h + 100.0);
    for t in below {
        assert!(now.0[t].y >= now.0[0].y + now.0[0].h, "card {t} went down with it");
        assert!(now.0[t].y > was.0[t].y);
    }
}

#[test]
fn the_first_layout_and_every_later_one_agree() {
    let mut ui = ui_of(window(cards_page("page", &ROWS)));
    let first = ui.spots.clone();
    for _ in 0..3 {
        ui.set_view(window(cards_page("page", &ROWS)));
        ui.settle();
        assert_eq!(ui.spots, first);
    }
}

#[test]
fn a_puzzle_is_packed_afresh_for_other_cards_another_width_or_a_page_opened_anew() {
    let grown = { let mut r = ROWS; r[1] += 9; r };
    let fresh = |tree: Node<Msg>, w: u32| {
        let mut ui = Ui::new(tree, Theme::new(true, "#fbc711"));
        ui.size = (w, H);
        ui.settle();
        ui.spots
    };
    let mut ui = ui_of(window(cards_page("page", &ROWS)));
    ui.set_view(window(cards_page("page", &grown)));
    ui.settle();
    let held = ui.spots.clone();
    let anew = fresh(window(cards_page("page", &grown)), W);
    assert_ne!(held, anew, "the grown page packs differently from nothing, or this test says nothing");
    let more: Vec<usize> = grown.iter().copied().chain([3]).collect();
    ui.set_view(window(cards_page("page", &more)));
    ui.settle();
    assert_eq!(ui.spots, fresh(window(cards_page("page", &more)), W));
    ui.set_view(window(cards_page("page", &ROWS)));
    ui.settle();
    ui.set_view(window(cards_page("other", &ROWS)));
    ui.settle();
    ui.set_view(window(cards_page("page", &grown)));
    ui.settle();
    assert_eq!(ui.spots, anew);
    let mut ui = ui_of(window(cards_page("page", &ROWS)));
    ui.set_view(window(cards_page("page", &grown)));
    ui.settle();
    assert_eq!(ui.spots, held);
    ui.size = (760, H);
    ui.laid = None;
    ui.settle();
    assert_eq!(ui.spots, fresh(window(cards_page("page", &grown)), 760));
}

#[test]
fn a_little_wider_or_narrower_keeps_the_arrangement() {
    let mut ui = ui_of(window(cards_page("page", &ROWS)));
    let was = cards(&ui, ROWS.len());
    let grown = { let mut r = ROWS; r[2] += 7; r };
    ui.set_view(window(cards_page("page", &grown)));
    for w in [W - 14, W + 30, W - 40, W] {
        ui.size = (w, H);
        ui.laid = None;
        ui.settle();
        let now = cards(&ui, ROWS.len());
        assert_eq!(was.1, now.1, "the order at {w}");
        let left = now.0.iter().map(|r| r.x).fold(f32::MAX, f32::min);
        for (a, b) in was.0.iter().zip(&now.0) {
            let scale = (w as f32 - 220.0) / (W as f32 - 220.0);
            assert!(((a.x - was.0[0].x) * scale - (b.x - left)).abs() < 3.0 + (scale - 1.0).abs() * 60.0, "{a:?} {b:?} at {w}");
            assert!((a.w * scale - b.w).abs() < 3.0 + (scale - 1.0).abs() * 60.0, "{a:?} {b:?} at {w}");
        }
    }
}

#[test]
fn a_window_inside_the_window_stays_where_it_opened() {
    let view = |name: &'static str, rows: usize| {
        let mut lines: Vec<Node<Msg>> = vec![ww::button("go-next-symbolic", "Show more").on_press(Msg::Go).key("more")];
        lines.extend((0..rows).map(|i| ww::label(format!("Line {i}"))));
        let ask = ww::card(ww::column(lines).gap(8.0)).floats().width(Length::Fixed(400.0));
        ww::layers(vec![
            page("page", 10),
            ww::placed(ask, |(w, h), (aw, ah)| (((aw - w) / 2.0).max(0.0), ((ah - h) / 2.0).max(0.0))).stays(name),
        ])
    };
    let mut ui = ui_of(view("ask", 2));
    let was = ui.spot("more").unwrap().rect;
    for rows in [6, 1, 8, 2] {
        ui.set_view(view("ask", rows));
        ui.settle();
        assert_eq!(ui.spot("more").unwrap().rect, was, "the button pressed stays under the pointer with {rows} lines");
    }
    ui.set_view(view("ask", 22));
    ui.settle();
    let card = ui.spots.iter().find(|s| matches!(ui.tree.at(&s.path).map(|n| &n.what), Some(What::Card(..)))).unwrap().rect;
    assert!(card.y >= 0.0 && card.y + card.h <= H as f32 + 0.5, "{card:?}");
    ui.set_view(view("ask", 2));
    ui.settle();
    assert_eq!(ui.spot("more").unwrap().rect, was);
    ui.set_view(view("ask", 12));
    ui.settle();
    ui.set_view(view("other", 12));
    ui.settle();
    let middle = ui.spot("more").unwrap().rect;
    assert!(middle.y < was.y - 20.0, "{middle:?} {was:?}");
    ui.set_view(page("page", 10));
    ui.settle();
    ui.set_view(view("other", 2));
    ui.settle();
    assert_eq!(ui.spot("more").unwrap().rect, was);
    ui.size = (W, H - 100);
    ui.laid = None;
    ui.set_view(view("other", 2));
    ui.settle();
    assert_eq!(ui.spot("more").unwrap().rect.y, was.y - 50.0);
}
