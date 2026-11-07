//! Tests of editor.

use super::*;

#[test]
fn unfinished_words_are_shown_not_typed() {
    let look = Look { font: None, size: 15.0, wrap: true, numbers: false };
    let mut pane = Pane::new(Doc::new("ab".into()), look);
    let compose = |preedit: &str, commit: &str| Input::Composed { delete: (0, 0), commit: commit.into(), preedit: preedit.into(), caret: Some((3, 3)) };
    assert!(matches!(pane.key(&compose("\u{304b}", "")), Some(Did::Moved)));
    assert_eq!(pane.doc.text(), "ab");
    assert_eq!(pane.preedit, ("\u{304b}".to_string(), Some(3)));
    assert!(matches!(pane.key(&compose("", "\u{304b}")), Some(Did::Edited)));
    assert_eq!(pane.doc.text(), "\u{304b}ab");
    assert!(pane.preedit.0.is_empty());
}

fn painted(text: &str) -> Pane {
    let theme = crate::theme::Theme::new(false, "#fbc711");
    let (fonts, icons) = (crate::text::Fonts::new(), crate::icon::Icons::new());
    let pen = Pen { theme: &theme, fonts: &fonts, icons: &icons, scale: 1.0 };
    let mut pane = Pane::new(Doc::new(text.into()), Look { font: None, size: 15.0, wrap: true, numbers: false });
    let mut data = vec![0u8; 300 * 100 * 4];
    pane.paint(&mut PixmapMut::from_bytes(&mut data, 300, 100).unwrap(), &pen);
    pane
}

#[test]
fn a_click_with_shift_held_extends_the_selection() {
    let mut pane = painted("one two three");
    pane.doc.select(0, 0);
    pane.pointer(Pointer::Down(250.0, 12.0));
    assert_eq!(pane.doc.selection(), 13..13, "a plain click moves the caret");
    pane.doc.select(0, 0);
    pane.hold_shift(true);
    pane.pointer(Pointer::Down(200.0, 12.0));
    assert_eq!(pane.doc.selection(), 0..13);
}

#[test]
fn a_column_that_could_not_be_measured_is_not_made_nought() {
    let mut pane = Pane::new(Doc::new("abc\ndef".into()), Look { font: None, size: 15.0, wrap: true, numbers: false });
    pane.doc.select(1, 1);
    let down = Input::Key { sym: Keysym::Down, text: None, mods: Mods::default(), repeat: false };
    pane.key(&down);
    assert_eq!(pane.goal, None);
    pane.goal = Some(7.0);
    pane.doc.select(1, 1);
    pane.key(&down);
    assert_eq!(pane.goal, Some(7.0));
}

#[test]
fn words_from_a_method_or_the_clipboard_lose_controls_and_overrides() {
    let mut pane = Pane::new(Doc::new(String::new()), Look { font: None, size: 15.0, wrap: true, numbers: false });
    pane.paste("a\r\nb\rc\u{0}d\u{1b}e\u{202e}f\tg");
    assert_eq!(pane.doc.text(), "a\nb\ncdef\tg");
    let commit = Input::Composed { delete: (0, 0), commit: "x\u{0}y".into(), preedit: String::new(), caret: None };
    pane.key(&commit);
    assert!(pane.doc.text().ends_with("xy"));
}

#[test]
fn words_to_check() {
    let line = "It's Bob's car2go, isn't it? 'Quoted' x_y";
    let found: Vec<&str> = words(line).map(|r| &line[r]).collect();
    assert_eq!(found, vec!["It's", "Bob's", "isn't", "it", "Quoted"]);
}

#[test]
fn rows_find_their_places() {
    let row = Row { start: 0, end: 3, top: 0.0, height: 10.0, stops: vec![(0, 0.0), (1, 8.0), (2, 16.0), (3, 24.0)] };
    assert_eq!(row.nearest(11.0), 1);
    assert_eq!(row.nearest(13.0), 2);
    assert_eq!(row.nearest(100.0), 3);
    assert_eq!(row.x_of(2), 16.0);
}
