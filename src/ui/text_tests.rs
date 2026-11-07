//! Words and language in a real window: rich words with links, right-to-left text boxes.

use super::*;
use super::super::rich::{parse, Span};
use super::super::theme::Theme;
use super::super::widget::{self as ww};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Open(String),
    Typed(String),
    Did(super::super::complete::Did),
}

fn ui_of(tree: Node<Msg>) -> Ui<Msg> {
    let mut ui = Ui::new(tree, Theme::new(true, "#fbc711"));
    ui.size = (400, 300);
    ui.settle();
    ui
}

fn shot(ui: &mut Ui<Msg>) -> tiny_skia::Pixmap {
    let mut pix = tiny_skia::Pixmap::new(400, 300).unwrap();
    ui.paint(&mut pix.as_mut(), 1.0);
    pix
}

fn key(sym: Keysym, shift: bool) -> Input {
    Input::Key { sym, text: None, mods: Mods { shift, ..Mods::default() }, repeat: false }
}

fn click(ui: &mut Ui<Msg>, x: f32, y: f32) -> Vec<Msg> {
    let now = Instant::now();
    ui.input(&Input::PointerMove { x, y }, now);
    ui.input(&Input::Press { button: Button::Left, x, y }, now);
    ui.input(&Input::Release { button: Button::Left, x, y }, now).messages
}

fn links() -> Node<Msg> {
    ww::column(vec![ww::rich(parse("Read **the** [guide](g) or [help](h) now")).on_link(Msg::Open).key("r")]).padding(10.0)
}

fn middle(ui: &Ui<Msg>, span: usize) -> (f32, f32) {
    let r = ui.spot("r").unwrap().rect;
    let (x, y, w, h) = ui.rich_flow("r").unwrap().bands(span)[0];
    (r.x + x + w / 2.0, r.y + y + h / 2.0)
}

#[test]
fn a_link_in_rich_words_is_pressed_by_the_pointer_and_plain_words_are_not() {
    let mut ui = ui_of(links());
    let (x, y) = middle(&ui, 5);
    assert_eq!(click(&mut ui, x, y), vec![Msg::Open("h".into())]);
    let (x, y) = middle(&ui, 0);
    assert!(click(&mut ui, x, y).is_empty());
}

#[test]
fn tab_goes_from_link_to_link_and_enter_follows_the_one_it_is_on() {
    let mut ui = ui_of(ww::column(vec![links(), ww::entry("", "After", Msg::Typed).key("e")]));
    let now = Instant::now();
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.input(&key(Keysym::Return, false), now).messages, vec![Msg::Open("g".into())]);
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.input(&key(Keysym::Return, false), now).messages, vec![Msg::Open("h".into())]);
    ui.input(&key(Keysym::Tab, false), now);
    assert_eq!(ui.focus.as_deref(), Some("e"), "after the last link the keyboard leaves");
    ui.input(&key(Keysym::Tab, true), now);
    assert_eq!(ui.rich_lit("r"), Some(5), "coming back lands on the last link");
}

#[test]
fn a_screen_reader_hears_the_paragraph_and_each_link_and_can_follow_one() {
    let mut ui = ui_of(links());
    let told = ui.access("Test");
    let para = told.nodes.iter().find(|(_, n)| n.role() == accesskit::Role::Paragraph).unwrap().1.clone();
    assert_eq!(para.label(), Some("Read the guide or help now"));
    let found: Vec<(accesskit::NodeId, &str)> = told.nodes.iter().filter(|(_, n)| n.role() == accesskit::Role::Link).map(|(id, n)| (*id, n.label().unwrap())).collect();
    assert_eq!(found.iter().map(|f| f.1).collect::<Vec<_>>(), ["guide", "help"]);
    let req = accesskit::ActionRequest { action: accesskit::Action::Click, target_tree: accesskit::TreeId::ROOT, target_node: found[1].0, data: None };
    assert_eq!(ui.act(&req).messages, vec![Msg::Open("h".into())]);
}

#[test]
fn a_link_is_coloured_and_never_underlined() {
    let plain = || ww::column(vec![ww::rich(vec![Span::plain("Read the guide or help now")]).key("r")]).padding(10.0);
    let linked = || ww::column(vec![ww::rich(parse("Read the [guide](g) or help now")).on_link(Msg::Open).key("r")]).padding(10.0);
    let (a, b) = (shot(&mut ui_of(plain())), shot(&mut ui_of(linked())));
    let ui = ui_of(linked());
    let r = ui.spot("r").unwrap().rect;
    let (x, y, w, h) = ui.rich_flow("r").unwrap().bands(1)[0];
    let px = |p: &tiny_skia::Pixmap, x: f32, y: f32| { let c = p.pixel(x as u32, y as u32).unwrap(); [c.red(), c.green(), c.blue()] };
    let inks = |p: &tiny_skia::Pixmap| -> std::collections::HashSet<[u8; 3]> {
        (0..w as u32).flat_map(|dx| (0..h as u32).map(move |dy| (dx, dy))).map(|(dx, dy)| px(p, r.x + x + dx as f32, r.y + y + dy as f32)).collect()
    };
    assert_ne!(inks(&a), inks(&b), "the link's words are in another colour");
    let ground = px(&b, r.x - 4.0, r.y + y);
    let flow = ui.rich_flow("r").unwrap();
    let baseline = flow.pieces.iter().find(|p| p.span == 1).unwrap().layout.lines[0].baseline;
    for dy in (baseline + 1.0) as u32..(h + 4.0) as u32 {
        let inked = (0..w as u32).filter(|&dx| px(&b, r.x + x + dx as f32, r.y + y + dy as f32) != ground).count();
        assert!((inked as f32) < w * 0.6, "row {dy} is inked {inked} of {w}");
    }
}

#[test]
fn suggestions_show_under_the_box_and_the_keys_go_through_them() {
    use super::super::complete::Complete;
    let all: Vec<String> = ["Apple", "Apricot", "Banana"].map(String::from).to_vec();
    let mut c = Complete::new("");
    let view = |c: &Complete| ww::column(vec![c.view("fruit", "Fruit", &all, Msg::Did)]).padding(10.0);
    let mut ui = ui_of(view(&c));
    let now = Instant::now();
    let feed = |ui: &mut Ui<Msg>, c: &mut Complete, input: Input| {
        let reply = ui.input(&input, now);
        for m in reply.messages {
            if let Msg::Did(d) = m {
                c.update(d, &all);
            }
        }
        ui.set_view(view(c));
        ui.settle();
    };
    feed(&mut ui, &mut c, key(Keysym::Tab, false));
    feed(&mut ui, &mut c, Input::Key { sym: Keysym::a, text: Some("a".into()), mods: Mods::default(), repeat: false });
    feed(&mut ui, &mut c, Input::Key { sym: Keysym::p, text: Some("p".into()), mods: Mods::default(), repeat: false });
    let list = ui.spot("fruit-suggestions").unwrap().rect;
    assert!(list.y >= ui.spot("fruit").unwrap().rect.y + ui.spot("fruit").unwrap().rect.h, "the list is under the box, in the page");
    let told = ui.access("Test");
    assert!(told.nodes.iter().any(|(_, n)| n.role() == accesskit::Role::ComboBox));
    let options: Vec<&str> = told.nodes.iter().filter(|(_, n)| n.role() == accesskit::Role::ListBoxOption).filter_map(|(_, n)| n.label()).collect();
    assert_eq!(options, ["Apple", "Apricot"]);
    assert!(told.nodes.iter().any(|(_, n)| n.live() == Some(accesskit::Live::Polite) && n.label().or(n.value()).is_some_and(|l| l.contains('2'))));
    feed(&mut ui, &mut c, key(Keysym::Down, false));
    feed(&mut ui, &mut c, key(Keysym::Down, false));
    feed(&mut ui, &mut c, key(Keysym::Up, false));
    assert_eq!((c.lit, ui.focus.as_deref()), (Some(0), Some("fruit")));
    feed(&mut ui, &mut c, key(Keysym::Return, false));
    assert_eq!(c.text, "Apple");
    assert!(ui.spot("fruit-suggestions").is_none(), "taken, the list goes");
}

#[test]
fn a_code_view_colours_words_by_kind_and_a_reader_hears_every_line() {
    use super::super::code::{self, Lang};
    let src = "fn main() {\n    // say hello\n    let n = 42;\n}";
    let mut rust = ui_of(ww::column(vec![code::view(src, Lang::Rust, "Code").key("c")]).padding(10.0));
    let mut plain = ui_of(ww::column(vec![code::view(src, Lang::Plain, "Code").key("c")]).padding(10.0));
    let (a, b) = (shot(&mut rust), shot(&mut plain));
    assert_ne!(a.data(), b.data(), "Rust is coloured, plain text is not");
    let inks = |p: &tiny_skia::Pixmap| p.pixels().iter().map(|c| (c.red(), c.green(), c.blue())).collect::<std::collections::HashSet<_>>().len();
    assert!(inks(&a) > inks(&b));
    let told = rust.access("Test");
    let words: Vec<String> = told.nodes.iter().filter_map(|(_, n)| n.value().map(String::from)).collect();
    for line in ["fn main() {", "    // say hello", "    let n = 42;"] {
        assert!(words.iter().any(|w| w.contains(line.trim())), "{line} in {words:?}");
    }
}

#[test]
fn a_help_page_shows_headings_words_and_links_that_are_followed() {
    use super::super::help;
    let page = help::markdown("# Getting started\n\nOpen [the guide](guide.md) first.\n\n- one\n- two");
    let mut ui = ui_of(ww::column(vec![help::view(&page, Msg::Open)]).padding(10.0));
    let told = ui.access("Test");
    assert!(told.nodes.iter().any(|(_, n)| n.role() == accesskit::Role::Heading && n.label() == Some("Getting started")));
    let link = told.nodes.iter().find(|(_, n)| n.role() == accesskit::Role::Link).map(|(id, _)| *id).unwrap();
    let req = accesskit::ActionRequest { action: accesskit::Action::Click, target_tree: accesskit::TreeId::ROOT, target_node: link, data: None };
    assert_eq!(ui.act(&req).messages, vec![Msg::Open("guide.md".into())]);
    let said = ui.tree.said();
    assert!(said.contains(&"\u{2022}".to_string()) && said.contains(&"two".to_string()));
}

#[test]
fn a_rich_text_box_takes_typing_from_the_window_and_draws_bold() {
    use super::super::editing::rich_box::{Event, RichBox};
    let b = RichBox::new("one two");
    let view = |b: &RichBox| ww::column(vec![b.node("Note", 120.0, |e| Msg::Typed(format!("{e:?}"))).key("b")]).padding(10.0);
    let mut ui = ui_of(view(&b));
    let now = Instant::now();
    ui.input(&key(Keysym::Tab, false), now);
    let reply = ui.input(&Input::Key { sym: Keysym::x, text: Some("x".into()), mods: Mods::default(), repeat: false }, now);
    assert_eq!(reply.messages.len(), 1, "the box hears the key");
    b.apply(Event::Key(Input::Key { sym: Keysym::x, text: Some("x".into()), mods: Mods::default(), repeat: false }));
    assert_eq!(b.text(), "one twox");
    let before = shot(&mut ui_of(view(&b)));
    b.select(0, 3);
    b.apply(Event::Key(Input::Key { sym: Keysym::b, text: None, mods: Mods { ctrl: true, ..Mods::default() }, repeat: false }));
    assert_eq!(b.markup(), "**one** twox");
    assert_ne!(before.data(), shot(&mut ui_of(view(&b))).data());
}

fn hebrew_box(text: &str) -> Ui<Msg> {
    ui_of(ww::column(vec![ww::entry(text, "Name", Msg::Typed).key("e").width(ww::Length::Fixed(300.0))]).padding(10.0))
}

#[test]
fn right_to_left_words_in_a_text_box_stand_at_its_right() {
    let mut ui = hebrew_box("שלום");
    let r = ui.spot("e").unwrap().rect;
    let pix = shot(&mut ui);
    let field = pix.pixel(r.x as u32 + 4, (r.y + r.h / 2.0) as u32).unwrap();
    let rows = (r.h * 0.3) as u32..(r.h * 0.7) as u32;
    let inked = |from: f32, to: f32| (from as u32..to as u32).any(|x| rows.clone().any(|y| pix.pixel(x, r.y as u32 + y).unwrap() != field));
    let mid = r.x + r.w / 2.0;
    assert!(inked(mid, r.x + r.w - 12.0), "the words are in the right half");
    assert!(!inked(r.x + 4.0, mid), "and none in the left");
}

#[test]
fn the_arrows_move_the_caret_the_way_they_point_in_right_to_left_words() {
    let mut ui = hebrew_box("שלום");
    let now = Instant::now();
    ui.input(&key(Keysym::Tab, false), now);
    let end = "שלום".len();
    ui.edit("e").unwrap().move_to(end, false);
    ui.input(&key(Keysym::Right, false), now);
    assert_eq!(ui.edits["e"].caret, end - 2);
    ui.input(&key(Keysym::Left, false), now);
    assert_eq!(ui.edits["e"].caret, end);
    let mut latin = hebrew_box("abc");
    latin.input(&key(Keysym::Tab, false), now);
    latin.edit("e").unwrap().move_to(3, false);
    latin.input(&key(Keysym::Left, false), now);
    assert_eq!(latin.edits["e"].caret, 2);
}

#[test]
fn a_selection_across_both_directions_covers_only_what_it_holds() {
    let fonts = crate::text::Fonts::new();
    let layout = fonts.layout("abc שלום", &crate::text::Style::new(16.0), None);
    let bands = layout.mark_bands(&[2..6]);
    assert_eq!(bands.len(), 2, "{bands:?}");
    assert_eq!(layout.mark_bands(&[0..layout.lines[0].range.end]).len(), 1);
}
