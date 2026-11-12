//! Text boxes of many lines: wrapping, Enter, moving by line, height, scrolling, undo, the input method and the screen reader.

use super::*;
use super::super::spelling::Speller;
use super::super::widget::{column, entry, Length};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Note(String),
    Go,
}

impl<M: Clone> Ui<M> {
    pub(super) fn entry_hit(&self, id: &str, x: f32) -> usize {
        self.entry_hit_at(id, x, 0.0)
    }
}

struct Page {
    text: String,
    lines: usize,
    submit: bool,
    high: Option<f32>,
}

impl Page {
    fn new(text: &str) -> Page {
        Page { text: text.into(), lines: 5, submit: true, high: None }
    }

    fn view(&self) -> Node<Msg> {
        let mut note = entry(self.text.clone(), "What were you doing?", Msg::Note).key("note").lines(self.lines);
        if self.submit {
            note = note.on_submit(Msg::Go);
        }
        if let Some(h) = self.high {
            note = note.height(Length::Fixed(h));
        }
        column(vec![note]).padding(10.0)
    }

    fn ui(&self) -> Ui<Msg> {
        let mut ui = Ui::new(self.view(), Theme::new(false, "#fbc711"));
        ui.size = (300, 400);
        ui.focus = Some("note".into());
        ui
    }

    fn feed(&mut self, ui: &mut Ui<Msg>, input: &Input) -> Reply<Msg> {
        let reply = ui.input(input, Instant::now());
        for m in &reply.messages {
            if let Msg::Note(t) = m {
                self.text = t.clone();
            }
        }
        ui.set_view(self.view());
        reply
    }
}

fn press(sym: Keysym, ctrl: bool, shift: bool) -> Input {
    Input::Key { sym, text: None, mods: Mods { ctrl, shift, ..Mods::default() }, repeat: false }
}

fn typed(t: &str) -> Input {
    Input::Key { sym: Keysym::NoSymbol, text: Some(t.into()), mods: Mods::default(), repeat: false }
}

fn paint(ui: &mut Ui<Msg>) {
    let mut pix = tiny_skia::Pixmap::new(300, 400).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
}

fn line_high(ui: &Ui<Msg>) -> f32 {
    ui.texts.get(&ui.fonts, " ", &Words::Plain.style(1.0), None).height
}

#[test]
fn words_wrap_at_the_box_width() {
    let page = Page::new(&"word ".repeat(30));
    let mut ui = page.ui();
    paint(&mut ui);
    let layout = ui.entry_layout("note", &page.text, false);
    let room = ui.spot("note").unwrap().rect.w - 2.0 * ui.entry_pad("note");
    assert!(layout.lines.len() > 3, "{} lines", layout.lines.len());
    assert!(layout.lines.iter().all(|l| l.width <= room), "no line is wider than the box");
}

#[test]
fn enter_adds_a_line_and_only_ctrl_enter_submits() {
    let mut page = Page::new("");
    let mut ui = page.ui();
    page.feed(&mut ui, &typed("a"));
    assert!(page.feed(&mut ui, &press(Keysym::Return, false, false)).messages.iter().all(|m| *m != Msg::Go));
    page.feed(&mut ui, &typed("b"));
    assert_eq!(page.text, "a\nb");
    assert_eq!(page.feed(&mut ui, &press(Keysym::Return, true, false)).messages, vec![Msg::Go]);
    assert_eq!(page.text, "a\nb", "Ctrl+Enter adds no line");
}

#[test]
fn ctrl_enter_does_nothing_without_a_submit_action() {
    let mut page = Page { submit: false, ..Page::new("a") };
    let mut ui = page.ui();
    assert!(page.feed(&mut ui, &press(Keysym::Return, true, false)).messages.is_empty());
    assert_eq!(page.text, "a");
}

#[test]
fn up_and_down_move_by_line_and_home_and_end_go_to_the_line_ends() {
    let mut page = Page::new("abcdef\nghijkl\nmno");
    let mut ui = page.ui();
    paint(&mut ui);
    let across = |ui: &Ui<Msg>, at: usize| ui.entry_layout("note", "abcdef\nghijkl\nmno", false).caret(at).0;
    ui.edit("note").unwrap().move_to(10, false);
    page.feed(&mut ui, &press(Keysym::Up, false, false));
    let up = ui.edits["note"].caret;
    assert!(up < 7 && (across(&ui, up) - across(&ui, 10)).abs() < 12.0, "up to {up}");
    page.feed(&mut ui, &press(Keysym::Down, false, false));
    page.feed(&mut ui, &press(Keysym::Down, false, false));
    assert!((14..=17).contains(&ui.edits["note"].caret), "down to {}", ui.edits["note"].caret);
    page.feed(&mut ui, &press(Keysym::Down, false, false));
    assert_eq!(ui.edits["note"].caret, 17, "Down on the last line goes to its end");
    ui.edit("note").unwrap().move_to(10, false);
    page.feed(&mut ui, &press(Keysym::Home, false, false));
    assert_eq!(ui.edits["note"].caret, 7);
    page.feed(&mut ui, &press(Keysym::End, false, false));
    assert_eq!(ui.edits["note"].caret, 13);
    page.feed(&mut ui, &press(Keysym::Up, false, true));
    assert_eq!(ui.edits["note"].selection().end, 13, "Shift with Up selects");
}

#[test]
fn a_box_is_at_least_n_lines_tall_and_natural_height_counts_them() {
    let height = |lines: usize, text: &str| {
        let page = Page { lines, ..Page::new(text) };
        let mut ui = page.ui();
        paint(&mut ui);
        (ui.spot("note").unwrap().rect.h, ui.natural_height(300.0), line_high(&ui))
    };
    let (box_five, natural_five, line) = height(5, "");
    let (box_one, natural_one, _) = height(1, "");
    assert!((box_five - box_one - 4.0 * line).abs() < 1.0, "{box_five} {box_one} {line}");
    assert!((natural_five - natural_one - 4.0 * line).abs() < 1.0, "{natural_five} {natural_one} {line}");
    assert!(box_five >= 5.0 * line);
    assert_eq!(height(5, "one\ntwo").0, box_five, "two lines do not shrink it");
    assert!(height(5, "1\n2\n3\n4\n5\n6\n7\n8").0 > box_five + 2.0 * line, "it grows past five lines");
}

#[test]
fn past_the_lines_in_sight_the_text_scrolls_to_keep_the_caret_in_view() {
    let mut page = Page { high: Some(90.0), ..Page::new("") };
    let mut ui = page.ui();
    for n in 0..12 {
        page.feed(&mut ui, &typed(&n.to_string()));
        page.feed(&mut ui, &press(Keysym::Return, false, false));
        paint(&mut ui);
        let rect = ui.spot("note").unwrap().rect;
        let (_, top, h) = ui.entry_layout("note", &page.text, false).caret(page.text.len());
        let slid = ui.edits["note"].scroll_y;
        assert!(top + h - slid <= rect.h - 1.0 && top >= slid, "line {n}: caret at {top}, slid {slid}, box {}", rect.h);
    }
    assert!(ui.edits["note"].scroll_y > 0.0);
    page.feed(&mut ui, &press(Keysym::Home, true, false));
    paint(&mut ui);
    assert_eq!(ui.edits["note"].scroll_y, 0.0, "back at the top");
}

#[test]
fn undo_takes_back_a_newline() {
    let mut page = Page::new("");
    let mut ui = page.ui();
    for input in [typed("a"), press(Keysym::Return, false, false), typed("b")] {
        page.feed(&mut ui, &input);
    }
    page.feed(&mut ui, &press(Keysym::z, true, false));
    assert_eq!(page.text, "a\n");
    page.feed(&mut ui, &press(Keysym::z, true, false));
    assert_eq!(page.text, "a");
    page.feed(&mut ui, &press(Keysym::y, true, false));
    assert_eq!(page.text, "a\n");
}

#[test]
fn a_screen_reader_hears_many_lines_only_from_a_box_of_many() {
    let role = |lines: usize| {
        let page = Page { lines, ..Page::new("a\nb") };
        let mut ui = page.ui();
        ui.access("T").nodes.into_iter().find(|(_, p)| p.label() == Some("What were you doing?")).map(|(_, p)| p.role())
    };
    assert_eq!(role(5), Some(accesskit::Role::MultilineTextInput));
    assert_eq!(role(1), Some(accesskit::Role::TextInput));
}

#[test]
fn the_input_method_commits_words_over_several_lines_and_the_caret_follows() {
    let mut page = Page::new("");
    let mut ui = page.ui();
    paint(&mut ui);
    let first = ui.typing().unwrap().area.1;
    page.feed(&mut ui, &Input::Composed { delete: (0, 0), commit: "日本\nにほん\nend".into(), preedit: "".into(), caret: None });
    assert_eq!(page.text, "日本\nにほん\nend");
    paint(&mut ui);
    let t = ui.typing().unwrap();
    assert!(t.area.1 as f32 >= first as f32 + 2.0 * line_high(&ui) - 1.0, "{} then {}", first, t.area.1);
    page.feed(&mut ui, &Input::Composed { delete: (0, 0), commit: "".into(), preedit: "x".into(), caret: Some((1, 1)) });
    assert_eq!(page.text, "日本\nにほん\nend", "a word being made is not yet the text");
}

#[test]
fn pasted_lines_stay_lines_and_a_click_puts_the_caret_on_its_line() {
    let mut page = Page::new("");
    let mut ui = page.ui();
    paint(&mut ui);
    let reply = ui.paste("one\r\ntwo\nthree\tx\u{202e}");
    for m in &reply.messages {
        if let Msg::Note(t) = m {
            page.text = t.clone();
        }
    }
    assert_eq!(page.text, "one\ntwo\nthree x");
    ui.set_view(page.view());
    paint(&mut ui);
    let (r, line) = (ui.spot("note").unwrap().rect, line_high(&ui));
    let at = ui.entry_hit_at("note", r.x + ui.entry_pad("note") + 1.0, r.y + (r.h - 5.0 * line) / 2.0 + 1.5 * line);
    assert_eq!(at, 4, "the start of the second line");
}

struct Knows;

impl Speller for Knows {
    fn known(&self, w: &str) -> bool {
        w == "good"
    }
}

#[test]
fn a_misspelt_word_on_a_later_line_is_waved_under() {
    let wave = |text: &str| {
        let note = entry(text, "Note", |_| ()).key("note").lines(5).spelling(&Knows);
        let mut ui = Ui::new(column(vec![note]).padding(10.0), Theme::new(false, "#fbc711"));
        ui.size = (300, 400);
        let mut pix = tiny_skia::Pixmap::new(300, 400).unwrap();
        ui.paint(&mut pix.as_mut(), 1.0);
        let (r, want) = (ui.spot("note").unwrap().rect, ui.theme.spelling);
        let near = |p: tiny_skia::PremultipliedColorU8| {
            let c = p.demultiply();
            (c.blue() as f32 - want.b * 255.0).abs() < 40.0 && (c.green() as f32 - want.g * 255.0).abs() < 40.0 && c.red() < 120
        };
        let lower = (r.y + r.h / 3.0) as u32;
        (r.x as u32..(r.x + r.w) as u32).flat_map(|x| (lower..(r.y + r.h) as u32).map(move |y| (x, y))).filter(|&(x, y)| near(pix.pixel(x, y).unwrap())).count()
    };
    let long = "good ".repeat(30);
    assert!(wave(&format!("{long}wrod")) > 6, "the word is on a wrapped line");
    assert_eq!(wave(&format!("{long}good")), 0);
}

fn long_page() -> (Page, Ui<Msg>) {
    let page = Page { high: Some(90.0), ..Page::new(&(0..12).map(|n| n.to_string()).collect::<Vec<_>>().join("\n")) };
    let mut ui = page.ui();
    paint(&mut ui);
    ui.edit("note").unwrap().move_to(0, false);
    paint(&mut ui);
    (page, ui)
}

fn over_box(ui: &mut Ui<Msg>, dy: f32) -> (f32, f32) {
    let r = ui.spot("note").unwrap().rect;
    let at = (r.x + r.w / 2.0, r.y + r.h / 2.0 + dy);
    ui.input(&Input::PointerMove { x: at.0, y: at.1 }, Instant::now());
    at
}

#[test]
fn the_wheel_scrolls_a_long_box_by_lines_clamped_at_both_ends_without_moving_the_caret() {
    let (_, mut ui) = long_page();
    assert_eq!(ui.edits["note"].scroll_y, 0.0);
    over_box(&mut ui, 0.0);
    ui.input(&Input::Scroll { x: 0.0, y: 1.0, steps: true }, Instant::now());
    let line = line_high(&ui);
    assert!((ui.edits["note"].scroll_y - 3.0 * line).abs() < 1.0, "three lines, not {}", ui.edits["note"].scroll_y / line);
    paint(&mut ui);
    paint(&mut ui);
    assert!((ui.edits["note"].scroll_y - 3.0 * line).abs() < 1.0, "painting does not pull the view back to the caret");
    assert_eq!(ui.edits["note"].caret, 0);
    ui.input(&Input::Scroll { x: 0.0, y: 100.0, steps: true }, Instant::now());
    paint(&mut ui);
    let (layout, rect) = (ui.entry_layout("note", &ui.edits["note"].text.clone(), false), ui.spot("note").unwrap().rect);
    let end = layout.height - (rect.h - 2.0 * ui.entry_rise("note"));
    assert!((ui.edits["note"].scroll_y - end).abs() < 1.0, "stops at the bottom: {} of {end}", ui.edits["note"].scroll_y);
    ui.input(&Input::Scroll { x: 0.0, y: -100.0, steps: true }, Instant::now());
    paint(&mut ui);
    assert_eq!(ui.edits["note"].scroll_y, 0.0, "stops at the top");
    assert_eq!(ui.edits["note"].caret, 0);
}

#[test]
fn typing_after_a_wheel_turn_brings_the_caret_back_into_view() {
    let (mut page, mut ui) = long_page();
    over_box(&mut ui, 0.0);
    ui.input(&Input::Scroll { x: 0.0, y: 100.0, steps: true }, Instant::now());
    paint(&mut ui);
    page.feed(&mut ui, &typed("x"));
    paint(&mut ui);
    assert_eq!(ui.edits["note"].scroll_y, 0.0);
}

fn line_text(n: usize) -> String {
    ["abcdefgh", "ab", "abcdefgh"][n].to_string()
}

fn caret_x(ui: &Ui<Msg>, page: &Page) -> f32 {
    ui.entry_layout("note", &page.text, false).caret(ui.edits["note"].caret).0
}

#[test]
fn up_and_down_keep_the_column_they_started_from_across_a_short_line() {
    let text = (0..3).map(line_text).collect::<Vec<_>>().join("\n");
    let mut page = Page::new(&text);
    let mut ui = page.ui();
    paint(&mut ui);
    ui.edit("note").unwrap().move_to(6, false);
    let start = caret_x(&ui, &page);
    page.feed(&mut ui, &press(Keysym::Down, false, false));
    assert_eq!(ui.edits["note"].caret, 11, "the short line ends sooner");
    page.feed(&mut ui, &press(Keysym::Down, false, false));
    assert_eq!(ui.edits["note"].caret, 18);
    assert!((caret_x(&ui, &page) - start).abs() < 1.0);
    page.feed(&mut ui, &press(Keysym::Up, false, false));
    page.feed(&mut ui, &press(Keysym::Up, false, false));
    assert_eq!(ui.edits["note"].caret, 6, "and back up again");
}

#[test]
fn any_other_change_forgets_the_column_up_and_down_were_keeping() {
    let text = (0..3).map(line_text).collect::<Vec<_>>().join("\n");
    let others: [(&str, Box<dyn Fn(&mut Page, &mut Ui<Msg>)>); 6] = [
        ("left", Box::new(|p, u| { p.feed(u, &press(Keysym::Left, false, false)); })),
        ("right", Box::new(|p, u| { p.feed(u, &press(Keysym::Left, false, false)); p.feed(u, &press(Keysym::Right, false, false)); p.feed(u, &press(Keysym::Left, false, false)); })),
        ("home", Box::new(|p, u| { p.feed(u, &press(Keysym::Home, false, false)); })),
        ("end", Box::new(|p, u| { p.feed(u, &press(Keysym::Left, false, false)); p.feed(u, &press(Keysym::End, false, false)); p.feed(u, &press(Keysym::Left, false, false)); })),
        ("typing", Box::new(|p, u| { p.feed(u, &typed("x")); })),
        ("paste", Box::new(|p, u| {
            for m in u.paste("y").messages {
                if let Msg::Note(t) = m {
                    p.text = t;
                }
            }
            u.set_view(p.view());
        })),
    ];
    for (name, other) in others {
        let mut page = Page::new(&text);
        let mut ui = page.ui();
        paint(&mut ui);
        ui.edit("note").unwrap().move_to(6, false);
        page.feed(&mut ui, &press(Keysym::Down, false, false));
        other(&mut page, &mut ui);
        paint(&mut ui);
        let here = caret_x(&ui, &page);
        page.feed(&mut ui, &press(Keysym::Down, false, false));
        assert!((caret_x(&ui, &page) - here).abs() < 12.0, "{name}: from {here} to {}", caret_x(&ui, &page));
    }
}

#[test]
fn a_click_forgets_the_column_up_and_down_were_keeping() {
    let text = (0..3).map(line_text).collect::<Vec<_>>().join("\n");
    let mut page = Page::new(&text);
    let mut ui = page.ui();
    paint(&mut ui);
    ui.edit("note").unwrap().move_to(6, false);
    page.feed(&mut ui, &press(Keysym::Down, false, false));
    let (r, line) = (ui.spot("note").unwrap().rect, line_high(&ui));
    let (x, y) = (r.x + ui.entry_pad("note") + 1.0, r.y + (r.h - 5.0 * line) / 2.0 + 1.5 * line);
    ui.input(&Input::Press { button: Button::Left, x, y }, Instant::now());
    ui.input(&Input::Release { button: Button::Left, x, y }, Instant::now());
    assert_eq!(ui.edits["note"].caret, 9);
    page.feed(&mut ui, &press(Keysym::Down, false, false));
    assert_eq!(ui.edits["note"].caret, 12, "straight down from the start of the second line");
}

fn drag_out(dy: f32) -> (Ui<Msg>, Instant) {
    let (_, mut ui) = long_page();
    let r = ui.spot("note").unwrap().rect;
    let now = Instant::now();
    let start = if dy > 0.0 { (r.x + 12.0, r.y + ui.entry_rise("note") + 2.0) } else { (r.x + 12.0, r.y + r.h - ui.entry_rise("note") - 2.0) };
    ui.input(&Input::PointerMove { x: start.0, y: start.1 }, now);
    if dy < 0.0 {
        ui.input(&Input::Scroll { x: 0.0, y: 100.0, steps: true }, now);
    }
    ui.input(&Input::Press { button: Button::Left, x: start.0, y: start.1 }, now);
    let out = if dy > 0.0 { r.y + r.h + dy } else { r.y + dy };
    ui.input(&Input::PointerMove { x: r.x + r.w - 5.0, y: out }, now);
    paint(&mut ui);
    (ui, now)
}

#[test]
fn a_drag_held_below_the_box_scrolls_the_text_by_lines_and_the_selection_grows_past_the_lines_in_sight() {
    let (mut ui, now) = drag_out(20.0);
    let line = line_high(&ui);
    let first = ui.edits["note"].caret;
    let before = ui.edits["note"].scroll_y;
    assert!(ui.next_wake(now).is_some(), "the drag asks to be called again");
    let mut t = now;
    for _ in 0..4 {
        t += Duration::from_millis(200);
        assert!(ui.tick(t));
        paint(&mut ui);
    }
    assert!(ui.edits["note"].scroll_y >= before + 3.0 * line - 1.0, "scrolled {} from {before}", ui.edits["note"].scroll_y);
    assert!(ui.edits["note"].caret > first, "the selection grew with the view");
    for _ in 0..60 {
        t += Duration::from_millis(200);
        ui.tick(t);
        paint(&mut ui);
    }
    let text = ui.edits["note"].text.clone();
    let (layout, rect) = (ui.entry_layout("note", &text, false), ui.spot("note").unwrap().rect);
    let end = layout.height - (rect.h - 2.0 * ui.entry_rise("note"));
    assert!((ui.edits["note"].scroll_y - end).abs() < 1.0, "stops at the end");
    assert_eq!(ui.edits["note"].caret, text.len());
    assert!(!ui.tick(t + Duration::from_millis(200)), "nothing left to do");
    ui.input(&Input::Release { button: Button::Left, x: 0.0, y: 0.0 }, t);
    assert!(ui.next_wake(t).is_none(), "let go, it stops asking");
}

#[test]
fn a_drag_held_above_the_box_scrolls_the_text_back_up() {
    let (mut ui, now) = drag_out(-20.0);
    let was = ui.edits["note"].scroll_y;
    assert!(was > 0.0);
    let mut t = now;
    for _ in 0..4 {
        t += Duration::from_millis(200);
        assert!(ui.tick(t));
        paint(&mut ui);
    }
    assert!(ui.edits["note"].scroll_y < was - 2.0 * line_high(&ui));
}

#[test]
fn a_drag_inside_the_box_does_not_scroll_it() {
    let (_, mut ui) = long_page();
    let now = Instant::now();
    let r = ui.spot("note").unwrap().rect;
    ui.input(&Input::PointerMove { x: r.x + 12.0, y: r.y + 20.0 }, now);
    ui.input(&Input::Press { button: Button::Left, x: r.x + 12.0, y: r.y + 20.0 }, now);
    ui.input(&Input::PointerMove { x: r.x + 40.0, y: r.y + r.h - 12.0 }, now);
    assert!(!ui.tick(now + Duration::from_millis(300)));
    assert_eq!(ui.edits["note"].scroll_y, 0.0);
}

fn second_line_pixels(placeholder: &str) -> Vec<tiny_skia::PremultipliedColorU8> {
    let note = entry("", placeholder, Msg::Note).key("note").lines(5);
    let mut ui = Ui::new(column(vec![note]).padding(10.0), Theme::new(false, "#fbc711"));
    ui.size = (300, 400);
    let mut pix = tiny_skia::Pixmap::new(300, 400).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    let (r, line) = (ui.spot("note").unwrap().rect, line_high(&ui));
    let top = (r.y + ui.entry_rise("note") + line + 2.0) as u32;
    (r.x as u32 + 4..(r.x + r.w) as u32 - 4).flat_map(|x| (top..top + line as u32 - 4).map(move |y| (x, y))).map(|(x, y)| pix.pixel(x, y).unwrap()).collect()
}

#[test]
fn a_long_placeholder_wraps_at_the_box_width_in_a_box_of_many_lines() {
    let short = second_line_pixels("Tell us");
    let long = second_line_pixels(&"Tell us what happened. ".repeat(4));
    let differ = short.iter().zip(&long).filter(|(a, b)| a != b).count();
    assert!(differ > 10, "a long one goes on to a second line ({differ} pixels differ)");
}

fn height_of(lines: usize) -> f32 {
    let note = entry("", "Note", |_| ()).key("note").lines(lines);
    let mut ui = Ui::new(column(vec![note]).padding(10.0), Theme::new(false, "#fbc711"));
    ui.size = (300, 400);
    ui.natural_height(300.0)
}

#[test]
fn no_lines_is_one_and_a_huge_number_of_lines_is_capped() {
    assert_eq!(height_of(0), height_of(1));
    assert!(height_of(usize::MAX) < 200.0 * height_of(1), "{}", height_of(usize::MAX));
    assert!(height_of(usize::MAX) > height_of(5));
}

#[test]
fn one_word_wider_than_the_box_still_lays_out_and_the_caret_moves_through_it() {
    let mut page = Page::new(&"w".repeat(300));
    let mut ui = page.ui();
    paint(&mut ui);
    let room = ui.spot("note").unwrap().rect.w - 2.0 * ui.entry_pad("note");
    let layout = ui.entry_layout("note", &page.text, false);
    assert!(layout.lines.iter().all(|l| l.width <= room + 1.0), "every line fits: {:?}", layout.lines.iter().map(|l| l.width).collect::<Vec<_>>());
    page.feed(&mut ui, &press(Keysym::Up, false, false));
    page.feed(&mut ui, &press(Keysym::Down, false, false));
    assert!(ui.edits["note"].caret <= 300);
}

#[test]
fn a_box_narrower_than_its_padding_does_not_break() {
    let note = entry("some words\nmore", "Note", Msg::Note).key("note").lines(3).width(Length::Fixed(4.0));
    let mut ui = Ui::new(column(vec![note]).padding(10.0), Theme::new(false, "#fbc711"));
    ui.size = (300, 400);
    ui.focus = Some("note".into());
    paint(&mut ui);
    assert!(ui.entry_hit_at("note", 12.0, 30.0) <= 15);
    ui.input(&press(Keysym::Up, false, false), Instant::now());
    ui.input(&press(Keysym::Down, false, false), Instant::now());
    paint(&mut ui);
}

#[test]
fn right_to_left_and_combining_letters_keep_the_caret_on_a_letter_edge() {
    for text in ["שלום\nעולם יפה", "e\u{301}e\u{301}\nx\u{301}\u{302}y"] {
        let mut page = Page::new(text);
        let mut ui = page.ui();
        paint(&mut ui);
        ui.edit("note").unwrap().move_to(0, false);
        for key in [Keysym::Down, Keysym::End, Keysym::Up, Keysym::Home, Keysym::Down, Keysym::Right] {
            page.feed(&mut ui, &press(key, false, false));
            paint(&mut ui);
            let at = ui.edits["note"].caret;
            assert!(page.text.is_char_boundary(at), "{text:?} {key:?} {at}");
            assert!(unicode_segmentation::UnicodeSegmentation::grapheme_indices(page.text.as_str(), true).any(|(i, _)| i == at) || at == page.text.len(), "{text:?} {key:?} {at} is inside a letter");
        }
    }
}

#[test]
fn a_huge_paste_is_cut_short_in_a_box_of_many_lines_and_of_one() {
    let huge = "ab \n".repeat(2_000_000);
    assert!(crate::ui::outside::many_lines(&huge).len() <= 1 << 20);
    assert!(crate::ui::outside::one_line(&huge).len() <= 1 << 20);
}

#[test]
fn repeated_pastes_cannot_take_the_text_past_one_mebibyte() {
    let mut page = Page::new("");
    let mut ui = page.ui();
    for _ in 0..3 {
        for m in ui.paste(&"ab \n".repeat(200_000)).messages {
            if let Msg::Note(t) = m {
                page.text = t;
            }
        }
        ui.set_view(page.view());
    }
    assert!(ui.edits["note"].text.len() <= 1 << 20, "{} bytes", ui.edits["note"].text.len());
    assert!(ui.edits["note"].text.len() > 1 << 19, "the first pastes went in");
}

#[test]
fn text_the_program_sets_past_one_mebibyte_is_cut_short() {
    let page = Page::new(&"a".repeat(3 << 20));
    let mut ui = page.ui();
    ui.settle();
    assert!(ui.edit("note").unwrap().text.len() <= 1 << 20);
}

#[test]
fn two_texts_of_one_length_get_their_own_layouts() {
    let ui = Page::new("").ui();
    let style = Words::Plain.style(1.0);
    let (a, b) = (ui.texts.get(&ui.fonts, "abcd", &style, None), ui.texts.get(&ui.fonts, "wxyz", &style, None));
    assert_eq!((&**a.whole(), &**b.whole()), ("abcd", "wxyz"));
}

#[test]
fn asking_for_the_same_words_again_shares_the_one_layout() {
    let ui = Page::new("").ui();
    let style = Words::Plain.style(1.0);
    let (a, b) = (ui.texts.get(&ui.fonts, "abcd", &style, None), ui.texts.get(&ui.fonts, "abcd", &style, None));
    assert!(Rc::ptr_eq(&a, &b));
    assert_eq!(ui.texts.laid(), 1);
}

#[test]
fn a_cut_at_one_mebibyte_lands_on_a_letter_edge_and_leaves_valid_text() {
    for (name, text) in [("japanese", "あ".repeat(400_000)), ("emoji after one byte", format!("a{}", "😀".repeat(300_000))), ("combining", "e\u{301}".repeat(400_000))] {
        let cut = crate::edit::cut_short(&text, crate::edit::MOST_TEXT);
        assert!(cut.len() <= 1 << 20 && cut.len() > (1 << 20) - 4, "{name}: {} bytes", cut.len());
        assert!(text.starts_with(cut) && cut.chars().count() > 0, "{name}");
        let mut ui = Page::new(&text).ui();
        ui.settle();
        let held = ui.edit("note").unwrap().text.clone();
        assert!(held.len() <= 1 << 20 && text.starts_with(&held), "{name}: the box holds a whole-letter start of the text");
        let pasted = crate::ui::outside::many_lines(&text);
        assert!(pasted.len() <= 1 << 20 && pasted.chars().all(|c| c != '\u{fffd}'), "{name}");
    }
}
